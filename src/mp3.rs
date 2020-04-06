use crate::track;
use mp3_metadata;
// use puremp3;
use std::error::Error;
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
        // if self.file.is_none() {
        //     self.file = Some(File::open(&self.path)?);
        // }
        // let f = self.file.as_mut().expect("Shouldn't happen!");
        self.meta = Some(mp3_metadata::read_from_file(self.path.clone())?);
        return Ok(true);
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
            }
            println!("There are {} frames.", md.frames.len());
            if md.frames.len() > 0 {
                println!("Here is the fist: {:?}", md.frames[0]);
            }
        };

        return Ok(Some(tk));
    }
}
