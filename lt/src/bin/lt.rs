extern crate git_version;
// use git_version::git_version;
use lt::run::{run, Config};
use std::error::Error;

// const GIT_VERSION: &str = git_version!();
fn main() -> Result<(), Box<dyn Error>> {
    // println!("Version: {}", GIT_VERSION);

    run(Config {
        history_path: Some(".lt_history".to_string()),
    })?;

    Ok(())
}
