use crate::track;
use mp3_metadata;

// use puremp3;
use std::error::Error;
use std::time::Duration;
// use std::fs::File;
// use std::io::{Seek, SeekFrom};
use std::path::PathBuf;

#[derive(Default)]
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
    fn is_candidate(&mut self) -> Result<bool, Box<dyn Error>> {
        match mp3_metadata::read_from_file(self.path.clone()) {
            Ok(md) => {
                self.meta = Some(md);
                return Ok(true);
            }
            Err(e) => match e {
                mp3_metadata::Error::FileError => return Err(Box::new(e)),
                _ => return Ok(false),
            },
        }
    }

    fn get_track(&mut self) -> Result<Option<track::Track>, Box<dyn Error>> {
        if self.meta.is_none() {
            self.meta = Some(mp3_metadata::read_from_file(self.path.clone())?);
        }

        let mut tk = track::Track {
            file_format: FORMAT_NAME.to_string(),
            ..Default::default()
        };

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
                let f = &md.frames[0];
                let i = md.frames.len() - 1;
                let lf = &md.frames[i];
                let lfd;
                if let Some(d) = lf.duration {
                    lfd = d;
                } else {
                    lfd = Duration::new(0, 0);
                }
                let mf = track::MPEGFormat {
                    bitrate: f.bitrate,
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
