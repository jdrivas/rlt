//! Interface to WAV file format reading.
//!
//! Wav files only provide basic format information, so really only
//! track::SampleFormat.
//! Thie means you need to set the title (and anything else for that matter
//! on your own.
use crate::file::{Decoder, FileFormat};
use crate::track;
use hound;
use std::error::Error;
use std::io::{Read, Seek};

#[derive(Default, Debug)]

/// Wav file format reader.
///
/// Implements the `Deecoder` trait, so has `get_track`.
pub struct Wav;

const RIFF_HEADER: &[u8] = b"RIFF";
const WAVE_HEADER: &[u8] = b"WAVE";

/// Read the first 12 bytes of the buffer and return A `Wav` strcut
/// if this buffer has the right identifier,
/// return None otherwise.
///
/// Will also return None if there are less than 12 bytes in the buffer.
pub fn identify(b: &[u8]) -> Option<FileFormat> {
    if b.len() >= 12 && &b[0..4] == RIFF_HEADER {
        match &b[8..12] {
            b if b == WAVE_HEADER => Some(FileFormat::WAV(Wav {})),
            _ => None,
        }
    } else {
        None
    }
}

const FORMAT_NAME: &str = "wav";

//
impl Decoder for Wav {
    /// Return the name of this format; "wav".
    fn name(&self) -> &str {
        FORMAT_NAME
    }
    /// Create a track with as much information as you have from the file.
    /// Wav files only provide basic format information, so really only
    /// track::SampleFormat.
    /// Thie means you need to set the title (and anything else for that matter
    /// on your own.
    // TODO(jdr) fill out the rest of the wave spec (float etc).
    fn get_track(&mut self, r: impl Read + Seek) -> Result<Option<track::Track>, Box<dyn Error>> {
        let wr = hound::WavReader::new(r)?;
        let spec = wr.spec();
        let mut tk = track::Track {
            // path: self.path.clone(),
            file_format: Some(FORMAT_NAME.to_string()),
            ..Default::default()
        };
        let f = track::PCMFormat {
            sample_rate: spec.sample_rate,
            channels: spec.channels as u8,
            bits_per_sample: spec.bits_per_sample,
            total_samples: wr.duration() as u64,
        };
        tk.format = Some(track::CodecFormat::PCM(f));
        Ok(Some(tk))
    }
}
