use crate::track;
use hound;
use std::error::Error;
use std::io::{Read, Seek, SeekFrom};

pub struct Wav;
impl track::Decoder for Wav {
    fn is_candidate<R: Read + Seek>(r: R) -> Result<bool, Box<dyn Error>> {
        let mut r = r;
        if let Ok(_) = hound::read_wave_header(&mut r) {
            r.seek(SeekFrom::Start(0))?;
            return Ok(true);
        } else {
            return Ok(false);
        }
    }

    fn get_track<R: Read + Seek>(r: R) -> Result<Option<track::Track>, Box<dyn Error>> {
        let mut r = r;
        let wr = hound::WavReader::new(&mut r)?;
        let spec = wr.spec();
        let mut tk = track::Track {
            ..Default::default()
        };
        tk.format.sample_rate = spec.sample_rate;
        tk.format.channels = spec.channels as u8;
        tk.format.bits_per_sample = spec.bits_per_sample;
        tk.format.total_samples = wr.duration() as u64;
        // tk.format.???? = spec.sample_format; // TODO(jdr): Do we want to accomodate this somehow?
        return Ok(Some(tk));
    }
}
