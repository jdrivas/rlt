use crate::flac;
use crate::id3;
use crate::mp3;
use crate::mp4;
use crate::track::Track;
use crate::wav;
use std::error::Error;
use std::io::{Read, Seek, SeekFrom};

pub trait Decoder {
    // fn identify(b: &[u8]) -> Result<FileFormat, std::io::Error>;
    fn name(&self) -> &str;
    // fn is_candidate(&mut self) -> Result<bool, Box<dyn Error>>;
    fn get_track(&mut self, r: impl Read + Seek) -> Result<Option<Track>, Box<dyn Error>>;
    // fn is_candidate<R: Read + Seek>(r: R) -> Result<bool, Box<dyn Error>>;
    // fn get_track<R: Read + Seek>(r: R) -> Result<Option<Track>, Box<dyn Error>>;
}

pub enum FileFormat {
    Flac(flac::Flac),
    MP4A,
    MP4B,
    MP4P,
    MP3(mp3::Mp3),
    WAV(wav::Wav),
    ID3(id3::Id3),
}

// Something tells me there is a macro somewhere for this.
impl std::fmt::Debug for FileFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            FileFormat::Flac(_) => f.write_str("Flac")?,
            FileFormat::MP4A => f.write_str("MP4A")?,
            FileFormat::MP4B => f.write_str("MP4B")?,
            FileFormat::MP4P => f.write_str("MP4P")?,
            FileFormat::MP3(_) => f.write_str("MP3")?,
            FileFormat::WAV(_) => f.write_str("WAV")?,
            FileFormat::ID3(_) => f.write_str("ID3")?,
            // FileFormat::Unknown => f.write_str("Unknown")?,
        };
        Ok(())
    }
}

/// Read the first few bytes of the header to
/// see if you can figure out what kind of file you've got.
/// Return the reader with seek position  SeekFrom::Start(0).
pub fn identify(mut r: impl Read + Seek) -> Result<Option<FileFormat>, std::io::Error> {
    let mut buf = [0; 12];
    r.read(&mut buf)?;
    r.seek(SeekFrom::Start(0))?;

    // If a decoder retruns a valid result, we don't visit anymore
    // if it's possible for a decoder to handle more than one file type
    // then order will matter here.
    let is = [
        flac::identify,
        mp4::identify,
        wav::identify,
        mp3::identify,
        id3::identify,
    ];
    for id in &is {
        match id(&buf) {
            Some(ff) => return Ok(Some(ff)),
            None => (),
        }
    }

    return Ok(None);
}
