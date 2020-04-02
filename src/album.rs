extern crate chrono;
use crate::track;
// use chrono::DateTime;
// use chrono::offset::TimeZone;
// use std::collections::hash_map::Values;
// use std::collections::HashMap;
use std::io;
use std::path;

#[derive(Debug)]
pub struct Album {
    pub tracks: Vec<track::Track>,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub disk_total: Option<u32>,
    // release: Option<DateTime<dyn TimeZone>>,
}

// TODO(jdr): Learn to use lifetimes and get rid of these tk.* clones?
// Assume that there is only one album in a directory.
pub fn albums_from(p: path::PathBuf) -> io::Result<(Album, Vec<path::PathBuf>)> {
    if !p.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("File is not a directory: {} ", p.as_path().display()),
        ));
    }

    let mut album = Album {
        tracks: Vec::new(),
        title: None,
        artist: None,
        disk_total: None,
    };
    let (tracks, files) = track::files_from(p)?;
    for tk in tracks {
        album.tracks.push(tk);
    }

    // TODO(jdr): I've never really been satisfied with the is
    // take the first one you find choice.
    // These might just as well be functions since they are referencing
    // internal values.
    if album.tracks.len() > 0 {
        album.title = album.tracks[0].album.clone();
        album.artist = album.tracks[0].artist.clone();
        album.disk_total = album.tracks[0].disk_total;
    }

    return Ok((album, files));
}
