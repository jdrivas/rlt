#[macro_use]
extern crate prettytable;

use lt::cmd;
fn main() -> cmd::Result<()> {
    // let f = "test_audio/LRUSSELL FE 11 20 70 01.fl";
    // let f = "test_audio/LRUSSELL FE 11 20 70 01.flac";

    cmd::run(cmd::Config {
        history_path: Some(".lt_history".to_string()),
    })?;

    // track::display_blocks(track::read_blocks(f.to_string()));

    Ok(())
}
