use crate::flac;
// use crate::id3;
use crate::mp3;
use crate::mp4;
use crate::mpeg4;
use crate::track::Track;
use crate::wav;
use std::error::Error;
use std::io::{Read, Seek, SeekFrom};

pub trait Decoder {
    fn name(&self) -> &str;
    fn get_track(&mut self, r: impl Read + Seek) -> Result<Option<Track>, Box<dyn Error>>;
}

pub enum FileFormat {
    Flac(flac::Flac),
    MPEG4(mpeg4::Mpeg4),
    MP4A(mp4::Mp4),
    MP3(mp3::Mp3),
    WAV(wav::Wav),
    // ID3(id3::Id3),
}

// Something tells me there is a macro somewhere for this.
impl std::fmt::Debug for FileFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            FileFormat::Flac(_) => f.write_str("Flac")?,
            FileFormat::MPEG4(_) => f.write_str("MPEG-4")?,
            FileFormat::MP4A(_) => f.write_str("MP4A")?,
            // FileFormat::MP4B => f.write_str("MP4B")?,
            // FileFormat::MP4P => f.write_str("MP4P")?,
            FileFormat::MP3(_) => f.write_str("MP3")?,
            FileFormat::WAV(_) => f.write_str("WAV")?,
            // FileFormat::ID3(_) => f.write_str("ID3")?,
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

    // If a decoder retruns a valid result, we don't visit any more of them.
    // If it's possible for a decoder to handle more than one file type
    // then order will matter here.
    let ids = [
        flac::identify,
        mpeg4::identify,
        mp4::identify,
        wav::identify,
        mp3::identify,
        // id3::identify,
    ];
    for id in &ids {
        match id(&buf) {
            Some(ff) => return Ok(Some(ff)),
            None => (),
        }
    }

    return Ok(None);
}
