//! The model for a collection of audio tracks.

use crate::track;
use std::error::Error;
use std::io;
use std::path;

/// A group of tracks with common metadata ie. an album.
#[derive(Debug)]
pub struct Album {
    /// List of tracks for this Album.
    pub tracks: Vec<track::Track>,
    /// Album title.
    pub title: Option<String>,
    /// Album artist.
    pub artist: Option<String>,
    /// Number of tracks in the album.
    pub track_total: Option<u32>,
    /// Number of disks in the album.
    pub disk_total: Option<u32>,
}

// TODO(jdr): Learn to use lifetimes and get rid of these tk.* clones?
// Assume that there is only one album in a directory.
/// Capture all the tracks in a directory and treat them as a group.
/// Metadata is collected as in album_from_tracks.
pub fn album_from_path(p: path::PathBuf) -> Result<(Album, Vec<path::PathBuf>), Box<dyn Error>> {
    if !p.is_dir() {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            format!("File is not a directory: {} ", p.as_path().display()),
        )));
    }

    let (tracks, files) = track::files_from(p)?;
    let album = album_from_tracks(tracks);

    Ok((album, files))
}

/// Create an album from a group of tracks.
/// This will look to the first track in the group to gather Album
/// metadata. This is not optimal and should be fixed.
/// However, the use case is intended for a grouping of tracks
/// that are actually part of an album.
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
    if !album.tracks.is_empty() {
        album.title = album.tracks[0].album.clone();
        album.artist = album.tracks[0].artist.clone();
        album.disk_total = album.tracks[0].disk_total;
        album.track_total = Some(album.tracks.len() as u32);
    }
    album
}
