//! Entry in to the program functionality, the top level run function called from main, as well as
//! support for the interactive loop.
//!
//! The cmd module defines the grammar, parsing, and execution of functions. This module
//! provides the runtie entrypoint and interactive processing loop includin the async update
//! of the prompt in interactive.
extern crate clap;
extern crate structopt;
use structopt::StructOpt;

use crate::cmd::{
    parse_app, parse_interactive, AppCmds, FilePath, ICmds, InteractiveCommands, ParseResult,
    RootSubcommand,
};
use crate::completion::PathCompleter;

// use linefeed::complete::PathCompleter;
use linefeed::{Interface, ReadResult};
use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

/// Configuration information for the application.
pub struct Config {
    /// Path for the history file. If None, then no history is kept.
    pub history_path: Option<String>,
}

/// Top level entrypoint to run the command and return an error.
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
                        subcmd: RootSubcommand::InteractiveSubcommand(InteractiveCommands::List(
                            FilePath {
                                path: vec![p.as_path().to_str().unwrap().to_string()],
                            },
                        )),
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

/// Interactive readloop functionality for ICmds.
/// This supports an asynchronous prompt update capability.
// TODO(Jdr) Need to automatically udpate
// based on current directory.
pub fn readloop(
    history_path: Option<PathBuf>,
    tx: mpsc::Sender<PromptUpdate>,
    rx: mpsc::Receiver<PromptUpdate>,
) -> Result<(), Box<dyn Error>> {
    // Async prompt deamon.
    let tx1 = mpsc::Sender::clone(&tx);
    prompt_start_up(tx1);

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
            Ok(Some(ReadResult::Eof)) => {
                println!("Use the \"quit\" command to exit the applicaiton.")
            }
            Ok(Some(ReadResult::Signal(s))) => println!("Caught signal: {:?}", s),
            Err(e) => eprintln!("Failed on readline: {:?}", e),
        };
    }
    Ok(())
}

/// Used to send asynchronous updates for the interactive command line prompt.
pub struct PromptUpdate {
    /// Value for the prompt.
    pub new_prompt: String,
}

// const TIME_FMT: &str = "%a %b %e %Y %T";
// TODO(jdr): Sunset this.
fn prompt_start_up(tx: mpsc::Sender<PromptUpdate>) {
    thread::spawn(move || loop {
        send_directory(&tx);
        thread::sleep(Duration::from_millis(1000));
    });
}

/// Get's the current directory and sends a message
/// to update the prompt string.
pub fn send_directory(tx: &mpsc::Sender<PromptUpdate>) {
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
