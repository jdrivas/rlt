extern crate clap;
extern crate linefeed;
extern crate structopt;

use crate::display;
use clap::AppSettings;

use chrono::Local;
use linefeed::{Interface, ReadResult};
use std::env;
use std::sync::{mpsc, Arc};
use std::thread;

use std::time::Duration;
use std::{error, fmt};
use structopt::StructOpt;

/// Configuration information for the application.
pub struct Config {
  /// Path for the history file. If None, then no history is kept.
  pub history_path: Option<String>,
}

/// Top level interface to run the command and return an error.
pub fn run(c: Config) -> Result<()> {
  match AppCmds::from_iter_safe(&env::args().collect::<Vec<String>>()[1..]) {
    Ok(mut opt) => {
      opt.history_path = c.history_path;
      parse_app(opt).map(|_a| ())? // Eat the return and publish ok.
    }
    Err(e) => eprintln!("Top: {}", e),
  }
  Ok(())
}

#[derive(Debug, StructOpt)]
#[structopt(name = "cli", version = "0.0.1", setting(AppSettings::NoBinaryName))]
struct AppCmds {
  #[structopt(skip)]
  history_path: Option<String>,

  /// File name for configuration.
  #[structopt(short = "c", long = "config", default_value = "cli.yaml")]
  config: String,

  #[structopt(subcommand)]
  subcmd: RootSubcommand,
}

#[derive(StructOpt, Debug)]
enum RootSubcommand {
  Interactive,
  #[structopt(flatten)]
  InteractiveSubCommand(InteractiveCommands),
}

#[derive(StructOpt, Debug)]
#[structopt(name = "cli", version = "0.0.1", setting(AppSettings::NoBinaryName))]
struct ICmds {
  /// File name for configuration.
  #[structopt(short = "c", long = "config", default_value = "cli.yaml")]
  config: String,

  #[structopt(subcommand)]
  subcmd: InteractiveCommands,
}

#[derive(StructOpt, Debug)]
enum InteractiveCommands {
  /// End the program.
  #[structopt(name = "quit")]
  Quit,

  /// List files
  #[structopt(name = "list", alias = "ls")]
  List(Path),

  /// Details on a track
  #[structopt(name = "describe")]
  Describe(Path),

  /// Change working directory
  #[structopt(name = "cd")]
  CD(Path),
}

#[derive(StructOpt, Debug)]
struct Path {
  path: Vec<String>,
}

// Parse and execute an AppCmds. This specifically sets up
// the ability to run either a single command from InteractiveCmds
// and return with a reswult, or to run an interactive loop
// for commands from InteractiveCommands.
fn parse_app(opt: AppCmds) -> Result<ParseResult> {
  match opt.subcmd {
    RootSubcommand::Interactive => {
      readloop(opt.history_path)?;
      Ok(ParseResult::Exit)
    }
    RootSubcommand::InteractiveSubCommand(c) => parse_interactive(c),
  }
}

// Command implementation
fn parse_interactive(cmd: InteractiveCommands) -> Result<ParseResult> {
  match cmd {
    InteractiveCommands::List(p) => {
      display::list_files(p.path.join(" "))?;
      Ok(ParseResult::Complete)
    }
    InteractiveCommands::CD(p) => {
      let dir = p.path.join(" ");
      println!("cd: {}", dir);
      env::set_current_dir(dir)?;
      Ok(ParseResult::Complete)
    }
    InteractiveCommands::Describe(p) => {
      display::describe_file(p.path.join(" "))?;
      Ok(ParseResult::Complete)
    }
    InteractiveCommands::Quit => Ok(ParseResult::Exit),
  }
}

pub type Result<T> = std::result::Result<T, ParseError>;

enum ParseResult {
  Complete,
  Exit,
}

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

// Interactive readloop functionality for ICmds.
// This supports an asynchronous prompt update capability.
fn readloop(history_path: Option<String>) -> Result<()> {
  //
  // Prompt & Readline loop.
  let (tx, rx) = mpsc::channel();
  prompt_start_up(tx);

  // Set up read loop.
  let rl = Arc::new(Interface::new("cli").unwrap());
  if let Err(e) = rl.set_prompt("cli> ") {
    eprintln!("Couldn't set prompt: {}", e)
  }

  history_path.as_ref().map(|path| {
    if let Err(e) = rl.load_history(&path) {
      eprintln!("Failed to load history file {:?}: {}", path, e);
    }
  });

  loop {
    match rl.read_line_step(Some(Duration::from_millis(1000))) {
      Ok(Some(ReadResult::Input(line))) => {
        let words: Vec<&str> = line.split_whitespace().collect();
        rl.add_history_unique(words.join(" "));
        match ICmds::from_iter_safe(words) {
          Ok(opt) => match parse_interactive(opt.subcmd) {
            Ok(ParseResult::Complete) => continue,
            Ok(ParseResult::Exit) => {
              history_path.as_ref().map(|path| {
                if let Err(e) = rl.save_history(&path) {
                  eprintln!("Failed to save history file: {:?} - {}", path, e);
                }
              });
              break;
            }
            Err(e) => eprintln!("RL-PI: {}", e),
          },
          Err(e) => eprintln!("RL - match: {}", e),
        }
      }
      // Check for a prompt update.
      Ok(None) => {
        let mut p = None;
        // Eat all that have come in but that last.
        for pm in rx.try_iter() {
          p = Some(pm);
        }
        // If something new, then do the update.
        if let Some(p) = p {
          if let Err(e) = rl.set_prompt(&p.new_prompt) {
            eprintln!("Failed to set prompt: {:?}", e)
          }
        }
        continue;
      }
      Ok(Some(ReadResult::Eof)) => {
        println!("Use the \"quit\" command to exit the applicaiton.");
        continue;
      }
      Ok(Some(ReadResult::Signal(s))) => {
        println!("Caught signal: {:?}", s);
        continue;
      }
      Err(e) => eprintln!("Failed on readline: {:?}", e),
    }
  }
  Ok(())
}

struct PromptUpdate {
  new_prompt: String,
}

const TIME_FMT: &str = "%a %b %e %Y %T";
fn prompt_start_up(tx: mpsc::Sender<PromptUpdate>) {
  thread::spawn(move || {
    let mut i = 0;
    loop {
      thread::sleep(Duration::from_millis(1000));
      if let Err(e) = tx.send(PromptUpdate {
        new_prompt: String::from(format!(
          "cli <{}> ",
          Local::now().format(TIME_FMT).to_string()
        )),
      }) {
        eprintln!("Failed to send a new prompt: {:?}", e)
      }
      i = i + 1;
    }
  });
}
