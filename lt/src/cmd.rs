extern crate clap;
extern crate linefeed;
extern crate structopt;

use crate::display;
use clap::AppSettings;

// use chrono::Local;
use linefeed::complete::PathCompleter;
use linefeed::{Interface, ReadResult};
use std::env;
use std::error::Error;
use std::path::PathBuf;
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
/// Run will parse the command line argument and run execute the appropriate command.
/// If there are no commands run will assume and execute a list command on the current directory.
/// If the command given is not a valid command, run will assume a list command
/// on the first argument provided.
pub fn run(c: Config) -> Result<(), Box<dyn Error>> {
  match AppCmds::from_iter_safe(&env::args().collect::<Vec<String>>()[1..]) {
    Ok(mut opt) => {
      opt.history_path = c.history_path;
      parse_app(opt).map(|_a| ())? // Eat the return and return an ok.
    }
    Err(e) => match e.kind {
      // Bad command, try to run a List command.
      clap::ErrorKind::MissingArgumentOrSubcommand | clap::ErrorKind::UnknownArgument => {
        let p = if env::args().len() > 1 {
          PathBuf::from(&env::args().nth(1).unwrap())
        } else {
          env::current_dir()?
        };
        if p.exists() {
          parse_app(AppCmds {
            history_path: None,
            config: "".to_string(),
            subcmd: RootSubcommand::InteractiveSubcommand(InteractiveCommands::List(FilePath {
              path: vec![p.as_path().to_str().unwrap().to_string()],
            })),
          })?;
        } else {
          // File doesn't exist so say so and print help.
          eprintln!("File not found: \"{}\"\n", p.as_path().display());
          AppCmds::clap().write_long_help(&mut std::io::stderr())?;
        }
      }
      // Not an knonw command, so print the error message.
      _ => eprintln!("{:?}", e),
    },
  }

  Ok(())
}

// Command Parse Tree definition.

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
  /// Run in interactive mode
  Interactive(FilePath),
  #[structopt(flatten)]
  InteractiveSubcommand(InteractiveCommands),
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
  /// End the program
  #[structopt(name = "quit", alias = "exit")]
  Quit,

  /// Find Files
  #[structopt(name = "find")]
  Find(FindPath),

  /// List files
  #[structopt(name = "list", alias = "ls")]
  List(FilePath),

  /// Details on a track
  #[structopt(name = "describe")]
  Describe(FilePath),

  /// List files
  #[structopt(name = "structure")]
  Structure(FilePath),

  /// Change working directory
  #[structopt(name = "cd")]
  CD(FilePath),
}

#[derive(StructOpt, Debug)]
struct FilePath {
  path: Vec<String>,
}

impl FilePath {
  fn path(&self) -> PathBuf {
    strings_to_pathbuf(&self.path)
  }
}

#[derive(StructOpt, Debug)]
struct FindPath {
  find_path: String,
  file_path: Vec<String>,
}

impl FindPath {
  fn file_path(&self) -> PathBuf {
    strings_to_pathbuf(&self.file_path)
  }
}

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

// Parse and execute an AppCmds. This specifically sets up
// the ability to run either a single command from InteractiveCmds
// and return with a result, or to run an interactive loop
// for commands from InteractiveCommands.
fn parse_app(opt: AppCmds) -> std::result::Result<ParseResult, Box<dyn Error>> {
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

// Command implementation
fn parse_interactive(
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
      println!("cd: {}", p.display());
      env::set_current_dir(p)?;
      send_directory(tx);
      Ok(ParseResult::Complete)
    }
    InteractiveCommands::Quit => Ok(ParseResult::Exit),
  }
}

// pub type Result<T> = std::result::Result<T, ParseError>;

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

// TODO(Jdr) Need to automatically udpate
// based on current directory.
fn readloop(
  history_path: Option<PathBuf>,
  tx: mpsc::Sender<PromptUpdate>,
  rx: mpsc::Receiver<PromptUpdate>,
) -> Result<(), Box<dyn Error>> {
  // Set up read loop.

  let rl = Arc::new(Interface::new("cli").unwrap());
  rl.set_completer(Arc::new(PathCompleter));
  if let Err(e) = rl.set_prompt("cli> ") {
    eprintln!("Couldn't set prompt: {}", e)
  }

  if let Some(path) = history_path.as_ref() {
    if let Err(e) = rl.load_history(&path) {
      eprintln!("Failed to load history file {:?}: {}", path, e);
    }
  }

  loop {
    // Check for propt uppdate.
    let mut p = None;
    for pm in rx.try_iter() {
      p = Some(pm);
    }

    if let Some(p) = p {
      if let Err(e) = rl.set_prompt(&p.new_prompt) {
        eprintln!("Failed to set prompt: {:?}", e)
      }
    };

    let rl_res = rl.read_line_step(Some(Duration::from_millis(1000)));

    // process result if there is one.
    match rl_res {
      Ok(Some(ReadResult::Input(line))) => {
        let words: Vec<&str> = line.split_whitespace().collect();
        rl.add_history_unique(words.join(" "));
        match ICmds::from_iter_safe(words) {
          Ok(opt) => match parse_interactive(opt.subcmd, &tx) {
            Ok(ParseResult::Complete) => continue,
            Ok(ParseResult::Exit) => {
              if let Some(path) = history_path.as_ref() {
                if let Err(e) = rl.save_history(path) {
                  eprintln!("Failed to save history file: {:?} - {}", path, e);
                }
              }
              break;
            }
            Err(e) => eprintln!("RL-PI: {}", e),
          },
          Err(e) => eprintln!("RL - match: {}", e),
        }
      }
      // Check for a prompt update.
      Ok(None) => continue,
      Ok(Some(ReadResult::Eof)) => println!("Use the \"quit\" command to exit the applicaiton."),
      Ok(Some(ReadResult::Signal(s))) => println!("Caught signal: {:?}", s),
      Err(e) => eprintln!("Failed on readline: {:?}", e),
    };
  }
  Ok(())
}

struct PromptUpdate {
  new_prompt: String,
}

// const TIME_FMT: &str = "%a %b %e %Y %T";
fn prompt_start_up(tx: mpsc::Sender<PromptUpdate>) {
  thread::spawn(move || loop {
    send_directory(&tx);
    thread::sleep(Duration::from_millis(1000));
  });
}

fn send_directory(tx: &mpsc::Sender<PromptUpdate>) {
  let cd = match env::current_dir() {
    Ok(p) => p,
    Err(_) => PathBuf::from("Unknown)"),
  };
  // let cd;
  // if let Ok(p) = env::current_dir() {
  //   cd = p.as_path().to_owned()
  // } else {
  //   cd = path::Path::new("Unknown").to_owned();
  // }
  if let Err(e) = tx.send(PromptUpdate {
    new_prompt: format!("cli <{}> ", cd.display()),
  }) {
    eprintln!("Failed to send a new prompt: {:?}", e)
  }
}
