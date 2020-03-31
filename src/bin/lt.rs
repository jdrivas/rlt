extern crate git_version;
// use git_version::git_version;
use lt::cmd;

// const GIT_VERSION: &str = git_version!();
fn main() -> cmd::Result<()> {
    // println!("Version: {}", GIT_VERSION);

    cmd::run(cmd::Config {
        history_path: Some(".lt_history".to_string()),
    })?;

    Ok(())
}
