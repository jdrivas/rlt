extern crate chrono;
use crate::track;
use std::error::Error;
use std::io;
use std::path;

#[derive(Debug)]
pub struct Album {
    pub tracks: Vec<track::Track>,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub track_total: Option<u32>,
    pub disk_total: Option<u32>,
}

// TODO(jdr): Learn to use lifetimes and get rid of these tk.* clones?
// Assume that there is only one album in a directory.
pub fn album_from_path(p: path::PathBuf) -> Result<(Album, Vec<path::PathBuf>), Box<dyn Error>> {
    if !p.is_dir() {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            format!("File is not a directory: {} ", p.as_path().display()),
        )));
    }

    let (tracks, files) = track::files_from(p)?;
    let album = album_from_tracks(tracks);

    return Ok((album, files));
}

pub fn album_from_tracks(tks: Vec<track::Track>) -> Album {
    let mut album = Album {
        tracks: Vec::new(),
        title: None,
        artist: None,
        track_total: None,
        disk_total: None,
    };

    for tk in tks {
        album.tracks.push(tk);
    }

    // TODO(jdr): I've never really been satisfied with simply
    // taking the first one in the list ....
    // These might just as well be functions since they are referencing
    // internal values.
    if album.tracks.len() > 0 {
        album.title = album.tracks[0].album.clone();
        album.artist = album.tracks[0].artist.clone();
        album.disk_total = album.tracks[0].disk_total;
        album.track_total = album.tracks[0].track_total;
        if album.track_total.is_none() {
            album.track_total = Some(album.tracks.len() as u32);
        }
    }
    return album;
}
