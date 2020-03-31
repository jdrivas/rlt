extern crate chrono;
use crate::track;
// use chrono::DateTime;
// use chrono::offset::TimeZone;
use std::collections::hash_map::Values;
use std::collections::HashMap;
use std::io;
use std::path;

#[derive(Debug)]
pub struct Album {
    pub tracks: Vec<track::Track>,
    pub title: String,
    pub artist: String,
    // release: Option<DateTime<dyn TimeZone>>,
}

// TODO(jdr): Learn to use lifetimes and get rid of these tk.* clones?
pub fn albums_from(p: path::PathBuf) -> io::Result<(Vec<Album>, Vec<path::PathBuf>)> {
    if !p.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("File is not a directory: {} ", p.as_path().display()),
        ));
    }

    let mut albums: HashMap<String, Album> = HashMap::new();
    let (tracks, files) = track::files_from(p)?;
    for tk in tracks {
        let an = tk.album.clone();
        if let Some(album) = albums.get_mut(&an) {
            album.tracks.push(tk);
        } else {
            let mut album = Album {
                tracks: Vec::new(),
                title: an.clone(),
                artist: tk.artist.clone(),
                // release: std::option::Option::None,
            };
            album.tracks.push(tk);
            albums.insert(an.clone(), album);
        }
    }

    // Move everything to a vector and return.
    let mut res: Vec<Album> = Vec::new();
    for (_, v) in albums.drain() {
        res.push(v);
    }
    return Ok((res, files));
}
