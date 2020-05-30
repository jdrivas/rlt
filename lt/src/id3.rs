/*
use crate::file::{Decoder, FileFormat};
use crate::track;
use id3::Tag;
use std::error::Error;
// use std::fs::File;
use std::io::{Read, Seek};
// use std::path::PathBuf;

#[derive(Default, Debug)]
pub struct Id3;

pub fn identify(b: &[u8]) -> Option<FileFormat> {
    if b.len() >= 3 {
        if &b[0..3] == b"ID3" {
            return Some(FileFormat::ID3(Id3 {}));
        }
    }
    return None;
}

const FORMAT_NAME: &str = "ID3";

impl Decoder for Id3 {
    fn name(&self) -> &str {
        FORMAT_NAME
    }

    /// Create a track with as much information as you have from the file.
    /// Note, path is not set here, it has to be set separately - path information
    /// is not passed in this call.
    fn get_track(&mut self, r: impl Read + Seek) -> Result<Option<track::Track>, Box<dyn Error>> {
        // let tag = Tag::read_from(self.file.as_mut().unwrap())?;
        let tag = Tag::read_from(r)?;

        let omd;
        if tag.frames().count() > 0 {
            let mut md = track::ID3Metadata {
                ..Default::default()
            };

            for fr in tag.frames() {
                // eprintln!("Frame: {:?}", fr);
                match fr.content() {
                    id3::Content::Text(s) => {
                        // println!("Text: {:?}: {:?}", fr.id(), s);
                        md.text
                            .entry(fr.id().to_string())
                            .and_modify(|v| v.push(s.clone()))
                            .or_insert(vec![s.clone()]);
                        // eprintln!("md: {:?}", md);
                    }
                    id3::Content::Comment(c) => {
                        md.comments
                            .entry(fr.id().to_string())
                            .and_modify(|v| {
                                v.push((c.lang.clone(), c.description.clone(), c.text.clone()))
                            })
                            .or_insert(vec![(
                                c.lang.clone(),
                                c.description.clone(),
                                c.text.clone(),
                            )]);
                    }
                    _ => (),
                }
            }
            omd = Some(track::FormatMetadata::ID3(md));
        } else {
            omd = None;
        }

        let tk = track::Track {
            file_format: Some(FORMAT_NAME.to_string()),
            title: tag.title().map_or(None, |v| Some(v.to_string())),
            artist: tag.artist().map_or(None, |v| Some(v.to_string())),
            album: tag.album().map_or(None, |v| Some(v.to_string())),
            track_number: tag.track(),
            track_total: tag.total_tracks(),
            disk_number: tag.disc(),
            disk_total: tag.total_discs(),
            metadata: omd,
            ..Default::default()
        };

        return Ok(Some(tk));
    }
}
*/
