extern crate git_version;
use git_version::git_version;
use lt::cmd;

const GIT_VERSION: &str = git_version!();
fn main() -> cmd::Result<()> {
    // let f = "test_audio/LRUSSELL FE 11 20 70 01.fl";
    // let f = "test_audio/LRUSSELL FE 11 20 70 01.flac";

    println!("Version: {}", GIT_VERSION);

    cmd::run(cmd::Config {
        history_path: Some(".lt_history".to_string()),
    })?;

    // track::display_blocks(track::read_blocks(f.to_string()));

    Ok(())
}
