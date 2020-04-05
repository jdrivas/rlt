use crate::track;
use id3::Tag;
use std::error::Error;
use std::io::{Read, Seek};

pub struct Id3;

impl track::Decoder for Id3 {
    fn is_candidate<R: Read + Seek>(r: R) -> Result<bool, Box<dyn Error>> {
        return Ok(Tag::is_candidate(r)?);
    }

    fn get_track<R: Read + Seek>(_r: R) -> Result<Option<track::Track>, Box<dyn Error>> {
        // ID3
        println!("ID3Tag file.");
        let tk = track::Track {
            ..Default::default()
        };
        return Ok(Some(tk));
    }
}
