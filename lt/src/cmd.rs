//! The definition of the command line arguments for single and interactive use.
//! This module includes the "grammar" defnition as well as functions to parse command lines.

extern crate clap;
extern crate linefeed;
extern crate structopt;

use crate::display;
use crate::run::{prompt_start_up, readloop, send_directory, PromptUpdate};
use clap::AppSettings;

// use chrono::Local;

use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::sync::mpsc;
use std::{error, fmt};
use structopt::StructOpt;

// Command Parse Tree definition.

/// Definition of the top level of the commands, options, and configuration
/// available from the command line.
#[derive(Debug, StructOpt)]
#[structopt(name = "cli", version = "0.0.1", setting(AppSettings::NoBinaryName))]
pub struct AppCmds {
  #[structopt(skip)]
  pub history_path: Option<String>,

  /// File name for configuration.
  #[structopt(short = "c", long = "config", default_value = "cli.yaml")]
  pub config: String,

  /// Commands available for the application.
  #[structopt(subcommand)]
  pub subcmd: RootSubcommand,
}

/// Captures the interactive command (which only appears on the command line
/// and also includes all other commands.
#[derive(StructOpt, Debug)]
pub enum RootSubcommand {
  /// Run in interactive mode
  Interactive(FilePath),
  #[structopt(flatten)]
  InteractiveSubcommand(InteractiveCommands),
}

/// Captures all commands and options avialable to interactive (and so command line).
#[derive(StructOpt, Debug)]
#[structopt(name = "cli", version = "0.0.1", setting(AppSettings::NoBinaryName))]
pub struct ICmds {
  /// File name for configuration.
  #[structopt(short = "c", long = "config", default_value = "cli.yaml")]
  pub config: String,

  /// The actual Interactive Commands.
  #[structopt(subcommand)]
  pub subcmd: InteractiveCommands,
}

/// Interactive and so application command definitions.
#[derive(StructOpt, Debug)]
pub enum InteractiveCommands {
  /// End the program.
  #[structopt(name = "quit", alias = "exit")]
  Quit,

  /// Find and print structural elements of a file (e.g. Mpeg4 box details).
  #[structopt(name = "find")]
  Find(FindPath),

  /// List files in the provided directory.
  #[structopt(name = "list", alias = "ls")]
  List(FilePath),

  /// Print details of a track.
  #[structopt(name = "describe")]
  Describe(FilePath),

  /// Print out the meta structure of the file (e.g. all Mpeg4 boxe types and sizes in order).
  #[structopt(name = "structure")]
  Structure(FilePath),

  /// Change working directory.
  #[structopt(name = "cd")]
  CD(FilePath),
}

/// Abstracts an command argument for files,
/// providing a meahanism to get a PathBuf.
#[derive(StructOpt, Debug)]
pub struct FilePath {
  pub path: Vec<String>,
}

impl FilePath {
  /// Get a PathBuf for this FilePath.
  // Since we never go back to a FilePath
  // from a PatBuf this seemed easier than implementing
  // Into meachinsm.
  // TODO(jdr): revist, this probably should be an Into.
  pub fn path(&self) -> PathBuf {
    strings_to_pathbuf(&self.path)
  }
}

/// Abstracts the find argument to get both the find specification e.g. /moov/udta/ilst/trkn,
/// and the FilePath for file to be operating on.
#[derive(StructOpt, Debug)]
pub struct FindPath {
  pub find_path: String,
  pub file_path: Vec<String>,
}

/// Get a PathBuf for this FindPath
impl FindPath {
  pub fn file_path(&self) -> PathBuf {
    strings_to_pathbuf(&self.file_path)
  }
}

/// Take a string and create a PathBuf
fn strings_to_pathbuf(v: &[String]) -> PathBuf {
  let mut s = v.join(" ");
  // Assume we mean the current directory.
  if s.is_empty() {
    s = ".".to_string();
  }
  // The FileCompleter returns strings like "Jackson\ Browne".
  // This doesn't work with any system file manipulation.
  PathBuf::from(s.replace("\\", ""))
}

/// Parse and execute an AppCmds. This specifically sets up
/// the ability to run either a single command from InteractiveCmds
/// and return with a result, or to run an interactive loop
/// for commands from InteractiveCommands.
pub fn parse_app(opt: AppCmds) -> std::result::Result<ParseResult, Box<dyn Error>> {
  // Prompt updates for readline loop.
  let (tx, rx) = mpsc::channel();
  let tx1 = mpsc::Sender::clone(&tx);
  prompt_start_up(tx);

  match opt.subcmd {
    // Go into interactive mode.
    RootSubcommand::Interactive(p) => {
      // If a directory is given, CD to it.
      // let p = PathBuf::from(p.to_string());
      let p = p.path();
      if p.is_dir() {
        env::set_current_dir(p)?;
      } else {
        eprintln!("Can't cd to {:?}, it's not a directory.", p);
      }

      // Get the history "path", turn it into a PathBuf and
      // get a full absolute path from it.
      let hp = if let Some(ps) = opt.history_path {
        let f = PathBuf::from(ps).canonicalize()?;
        Some(f)
      } else {
        None
      };
      readloop(hp, tx1, rx)?;
      Ok(ParseResult::Exit)
    }
    // Just execute the given command.
    RootSubcommand::InteractiveSubcommand(c) => parse_interactive(c, &tx1),
  }
}

/// Command implementation
/// This maps InteractiveCommands into actions.
pub fn parse_interactive(
  cmd: InteractiveCommands,
  tx: &mpsc::Sender<PromptUpdate>,
) -> Result<ParseResult, Box<dyn Error>> {
  match cmd {
    InteractiveCommands::List(p) => {
      // display::list_files(PathBuf::from(p.to_string()))?;
      display::list_files(p.path())?;
      Ok(ParseResult::Complete)
    }
    InteractiveCommands::Describe(p) => {
      // display::describe_file(PathBuf::from(p.to_string()))?;
      display::describe_file(p.path())?;
      Ok(ParseResult::Complete)
    }
    InteractiveCommands::Structure(p) => {
      // display::display_structure(PathBuf::from(p.to_string()))?;
      display::display_structure(p.path())?;
      Ok(ParseResult::Complete)
    }
    InteractiveCommands::Find(p) => {
      display::display_find_path(p.file_path(), p.find_path)?;
      Ok(ParseResult::Complete)
    }
    InteractiveCommands::CD(p) => {
      // let p = PathBuf::from(p.to_string());
      let p = p.path();
      // println!("cd: {}", p.display());
      env::set_current_dir(p)?;
      send_directory(tx);
      Ok(ParseResult::Complete)
    }
    InteractiveCommands::Quit => Ok(ParseResult::Exit),
  }
}

// pub type Result<T> = std::result::Result<T, ParseError>;

/// Captures the state of a succesfull parse.
/// Specifically noting if its time to exit succesfully.
pub enum ParseResult {
  Complete,
  Exit,
}

/// Capture Clap and IO errorrs as they happen in the parsing.
#[derive(Debug)]
pub enum ParseError {
  Clap(clap::Error),
  IO(std::io::Error),
}

impl fmt::Display for ParseError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    // write!(f, "{:?}", self)
    match self {
      ParseError::Clap(e) => match e.kind {
        clap::ErrorKind::VersionDisplayed => write!(f, ""),
        clap::ErrorKind::HelpDisplayed => write!(f, "{}", e.message),
        _ => write!(f, "Parse error => {}", e),
      },
      ParseError::IO(e) => write!(f, "{}", e),
    }
  }
}

impl error::Error for ParseError {
  fn cause(&self) -> Option<&dyn error::Error> {
    None
  }

  fn description(&self) -> &str {
    "parse error!"
  }
}

impl From<clap::Error> for ParseError {
  fn from(err: clap::Error) -> ParseError {
    ParseError::Clap(err)
  }
}

impl From<std::io::Error> for ParseError {
  fn from(err: std::io::Error) -> ParseError {
    ParseError::IO(err)
  }
}
