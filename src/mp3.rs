use crate::track;
use mp3_metadata;

// use puremp3;
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;
// use std::fs::File;
// use std::io::{Seek, SeekFrom};
use std::path::PathBuf;

#[derive(Default, Debug)]
pub struct Mp3 {
    path: PathBuf,
    meta: Option<mp3_metadata::MP3Metadata>,
}

impl Mp3 {
    pub fn new(p: &PathBuf) -> Mp3 {
        return Mp3 {
            path: p.clone(),
            ..Default::default()
        };
    }
}

const FORMAT_NAME: &str = "mpeg-3";
impl track::Decoder for Mp3 {
    fn name(&self) -> &str {
        FORMAT_NAME
    }

    // TODO: This doesn't seem to be descrinimating enough.
    // Seems to have lots of false positives.
    fn is_candidate(&mut self) -> Result<bool, Box<dyn Error>> {
        match mp3_metadata::read_from_file(self.path.clone()) {
            Ok(md) => {
                self.meta = Some(md);
                return Ok(true);
            }
            Err(e) => match e {
                mp3_metadata::Error::FileError => return Err(Box::new(e)),
                _ => {
                    // eprintln!("MP3 Error: {}", e);
                    return Ok(false);
                }
            },
        }
    }

    fn get_track(&mut self) -> Result<Option<track::Track>, Box<dyn Error>> {
        if self.meta.is_none() {
            self.meta = Some(mp3_metadata::read_from_file(self.path.clone())?);
        }

        // Create a track.
        let mut tk = track::Track {
            file_format: Some(FORMAT_NAME.to_string()),
            ..Default::default()
        };

        // Grab the metadata and fill in the track.
        // println!("Path: {}", self.path.as_path().display());
        if let Some(md) = &self.meta {
            if let Some(t) = &md.tag {
                tk.title = Some(t.title.clone());
                tk.artist = Some(t.artist.clone());
                tk.album = Some(t.album.clone());
            }
            for oi in &md.optional_info {
                tk.track_number = oi
                    .track_number
                    .as_ref()
                    .map_or(None, |v| Some(v.parse::<u32>().unwrap()));
                // println!("Time: {:?}", oi.length)
            }

            if md.frames.len() > 0 {
                // let mut br = HashMap::new();
                // let mut sf = HashMap::new();
                // for f in &md.frames {
                //     let mut c = br.entry(f.bitrate).or_insert(0);
                //     *c += 1;
                //     c = sf.entry(f.sampling_freq).or_insert(0);
                //     *c += 1;
                // }
                // for (k, v) in &br {
                //     println!("birate: {}  {} times", k, v)
                // }
                // for (k, v) in &sf {
                //     println!("sample freq: {}  {} times", k, v)
                // }

                // TODO(jdr) Clearly these are not really the right values.
                // Definitely needs more investigation. See the commented
                // code above.
                let f = &md.frames[0];
                let li = md.frames.len() - 1;
                let lf = &md.frames[li];
                let lfd;
                if let Some(d) = lf.duration {
                    lfd = d;
                } else {
                    lfd = Duration::new(0, 0);
                }
                let mf = track::MPEGFormat {
                    bitrate: f.bitrate,
                    sample_rate: f.sampling_freq,
                    duration: lf.position + lfd,
                    version: match f.version {
                        mp3_metadata::Version::Reserved => track::MPVersion::Reserved,
                        mp3_metadata::Version::MPEG1 => track::MPVersion::MPEG1,
                        mp3_metadata::Version::MPEG2 => track::MPVersion::MPEG2,
                        mp3_metadata::Version::MPEG2_5 => track::MPVersion::MPEG2_5,
                        mp3_metadata::Version::Unknown => track::MPVersion::Unknown,
                    },
                    layer: match f.layer {
                        mp3_metadata::Layer::Reserved => track::MPLayer::Reserved,
                        mp3_metadata::Layer::Layer1 => track::MPLayer::Layer1,
                        mp3_metadata::Layer::Layer2 => track::MPLayer::Layer2,
                        mp3_metadata::Layer::Layer3 => track::MPLayer::Layer3,
                        mp3_metadata::Layer::Unknown => track::MPLayer::Unknown,
                    },
                };
                tk.format = Some(track::CodecFormat::MPEG(mf));
            }
        };

        return Ok(Some(tk));
    }
}
