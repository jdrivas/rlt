use crate::track;
use id3::Tag;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

// pub struct Id3;

/// Determine if the file has id3 tags.
/// This will return the file to seek(SeekFrom::Start(0)), as
/// if it had not been read.
pub fn is_candidate(f: &mut File) -> Result<bool, Box<dyn Error>> {
    // eprintln!("Files is at position: {}", f.seek(SeekFrom::Current(0))?);
    return Ok(Tag::is_candidate(f)?);
}

/// Create a track with as much information as you have from the file.
/// Note, path is not set here, it has to be set separately - path information
/// is not passed in this call.
pub fn get_track(f: &mut File) -> Result<Option<track::Track>, Box<dyn Error>> {
    // ID3
    println!("ID3Tag file.");

    let tag = Tag::read_from(f)?;
    // for fr in tag.frames() {
    //     println!("frame: {:?}", fr);
    // }
    let tk = track::Track {
        title: tag.title().map_or(None, |v| Some(v.to_string())),
        artist: tag.artist().map_or(None, |v| Some(v.to_string())),
        album: tag.album().map_or(None, |v| Some(v.to_string())),
        track_number: tag.track(),
        track_total: tag.total_tracks(),
        disk_number: tag.disc(),
        disk_total: tag.total_discs(),
        ..Default::default()
    };
    return Ok(Some(tk));
}
