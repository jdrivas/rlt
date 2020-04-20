use lt::file;
use std::fs::File;

fn main() -> Result<(), std::io::Error> {
    let files = vec![
        "/Volumes/London Backups/Music/iTunes/iTunes Media/Music/Counting Crows/Across A Wire_ Live In New York City/1-01 Round Here.m4a",
        "/Volumes/Audio/TorrentTracks/Leon Russell/Leon Russell 1970-11-20 Fillmore East, NY, NY SBD Flac16/LRUSSELL FE 11 20 70 01.flac",
        "/Volumes/London Backups/iTunes_Library/ZZ Top/Comb 1_2/03 Track 03.mp3",
         "/Volumes/Audio/HDTracks/Keith Jarrett/The Köln Concert/1 Köln, January 24, 1975, Part I.wav",
    ];
    for f in files {
        let mut file = File::open(f)?;
        let t = file::identify(&mut file)?;
        println!("{}\n\t{:?}", f, t);
    }
    Ok(())
}
