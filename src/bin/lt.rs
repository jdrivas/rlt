extern crate git_version;
// use git_version::git_version;
use lt::cmd;
use std::error::Error;

// const GIT_VERSION: &str = git_version!();
fn main() -> Result<(), Box<dyn Error>> {
    // println!("Version: {}", GIT_VERSION);

    cmd::run(cmd::Config {
        history_path: Some(".lt_history".to_string()),
    })?;

    Ok(())
}
