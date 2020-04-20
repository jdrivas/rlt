use crate::file;
use crate::file::FileFormat;
use crate::track;
// use mp4parse;
use std::error::Error;
use std::io::{Read, Seek};

pub struct Mp4;

const FTYP_HEADER: &[u8] = b"ftype";
const M4A_HEADER: &[u8] = b"M4A";
// const M4B_HEADER: &[u8] = b"M4B";
// const M4P_HEADER: &[u8] = b"M4P";

pub fn identify(b: &[u8]) -> Option<FileFormat> {
    if b.len() >= 12 {
        if &b[4..8] == FTYP_HEADER {
            match &b[8..11] {
                b if b == M4A_HEADER => return Some(FileFormat::MP4A(Mp4 {})),
                // b if b == M4B_HEADER => return Some(FileFormat::MP4B),
                // b if b == M4P_HEADER => return Some(FileFormat::MP4P),
                _ => return None,
            }
        }
    }

    return None;
}

const FORMAT_NAME: &str = "mpeg-4";
impl file::Decoder for Mp4 {
    fn name(&self) -> &str {
        FORMAT_NAME
    }

    fn get_track(&mut self, _r: impl Read + Seek) -> Result<Option<track::Track>, Box<dyn Error>> {
        let _tk = track::Track {
            ..Default::default()
        };
        return Ok(None);
    }
}
