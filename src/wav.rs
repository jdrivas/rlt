use crate::track;
use hound;
use std::error::Error;
use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::path::PathBuf;

#[derive(Default)]
pub struct Wav {
    path: PathBuf,
    file: Option<File>,
}

impl Wav {
    pub fn new(p: &PathBuf) -> Wav {
        return Wav {
            path: p.clone(),
            ..Default::default()
        };
    }
}

const FORMAT_NAME: &str = "wav";

impl track::Decoder for Wav {
    /// Determine if the file is a wav file.
    /// /// This will return the file to seek(SeekFrom::Start(0)), as
    /// if it had not been read.
    fn is_candidate(&mut self) -> Result<bool, Box<dyn Error>> {
        if self.file.is_none() {
            self.file = Some(File::open(&self.path)?);
        }
        let f = self.file.as_mut().unwrap();

        if let Ok(_) = hound::read_wave_header(f) {
            f.seek(SeekFrom::Start(0))?;
            return Ok(true);
        } else {
            f.seek(SeekFrom::Start(0))?;
            return Ok(false);
        }
    }

    /// Create a track with as much information as you have from the file.
    /// Wav files only provide basic format information, so really onlyl
    /// track::SampleFormat.
    /// Thie means you need to set the title (and anything else for that matter
    /// on your own.
    /// Note, also, path is not set here, it has to be set separately (we don't get the
    /// path information in this call).
    fn get_track(&mut self) -> Result<Option<track::Track>, Box<dyn Error>> {
        if self.file.is_none() {
            self.file = Some(File::open(&self.path)?);
        }
        let f = self.file.as_mut().unwrap();

        let wr = hound::WavReader::new(f)?;
        let spec = wr.spec();
        let mut tk = track::Track {
            file_format: FORMAT_NAME.to_string(),
            ..Default::default()
        };
        let f = track::PCMFormat {
            sample_rate: spec.sample_rate,
            channels: spec.channels as u8,
            bits_per_sample: spec.bits_per_sample,
            total_samples: wr.duration() as u64,
        };
        tk.format = Some(track::CodecFormat::PCM(f));
        // tk.format.sample_rate = spec.sample_rate;
        // tk.format.channels = spec.channels as u8;
        // tk.format.bits_per_sample = spec.bits_per_sample;
        // tk.format.total_samples = wr.duration() as u64;
        // tk.format.???? = spec.sample_format; // TODO(jdr): Do we want to accomodate this somehow?
        return Ok(Some(tk));
    }
}
