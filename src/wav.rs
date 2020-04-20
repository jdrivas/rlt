use crate::file::{Decoder, FileFormat};
use crate::track;
use hound;
use std::error::Error;
use std::io::{Read, Seek};

#[derive(Default, Debug)]
pub struct Wav;

const RIFF_HEADER: &[u8] = b"RIFF";
const WAVE_HEADER: &[u8] = b"WAVE";
pub fn identify(b: &[u8]) -> Option<FileFormat> {
    if b.len() >= 12 {
        if &b[0..4] == RIFF_HEADER {
            match &b[8..12] {
                b if b == WAVE_HEADER => {
                    return Some(FileFormat::WAV(Wav {
                        ..Default::default()
                    }))
                }
                _ => return None,
            }
        }
    }
    return None;
}

const FORMAT_NAME: &str = "wav";

//
impl Decoder for Wav {
    fn name(&self) -> &str {
        FORMAT_NAME
    }
    /// Create a track with as much information as you have from the file.
    /// Wav files only provide basic format information, so really only
    /// track::SampleFormat.
    /// Thie means you need to set the title (and anything else for that matter
    /// on your own.
    /// TODO(jdr) fill out the rest of the wave spec (float etc).
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
        return Ok(Some(tk));
    }
}
