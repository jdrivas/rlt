use crate::track;
use metaflac::{Block, Tag};
use std::error::Error;
use std::fs::File;
// use std::io::{Read, Seek};
use std::path::PathBuf;

const DISCTOTAL: &str = "DISCTOTAL";
const DISCNUMBER: &str = "DISCNUMBER";
// const RELEASEDATE: &str = "RELEASE_DATE";
// const VENDOR: &str = "VENDOR";
const ALT_TOTALTRACKS: &str = "TRACKTOTAL";

#[derive(Default, Debug)]
pub struct Flac {
    path: PathBuf,
    file: Option<File>,
}

impl Flac {
    pub fn new(p: &PathBuf) -> Flac {
        return Flac {
            path: p.clone(),
            ..Default::default()
        };
    }
}

const FORMAT_NAME: &str = "flac";

impl track::Decoder for Flac {
    fn name(&self) -> &str {
        FORMAT_NAME
    }
    /// Determine if the file is a Flac file.
    /// /// This will return the file to seek(SeekFrom::Start(0)), as
    /// if it had not been read.
    fn is_candidate(&mut self) -> Result<bool, Box<dyn Error>> {
        if self.file.is_none() {
            self.file = Some(File::open(&self.path)?);
        }
        return Ok(Tag::is_candidate(self.file.as_mut().unwrap()));
    }

    /// Create a track with as much information as you have from the file.
    /// Note, path is not set here, it has to be set separately - path information
    /// is not passed in this call.
    fn get_track(&mut self) -> Result<Option<track::Track>, Box<dyn Error>> {
        if self.file.is_none() {
            self.file = Some(File::open(&self.path)?);
        }
        match Tag::read_from(self.file.as_mut().unwrap()) {
            Ok(t) => {
                let mut tk = track::Track {
                    file_format: Some(FORMAT_NAME.to_string()),
                    path: self.path.clone(),
                    metadata: Some(track::FormatMetadata::Flac(track::FlacMetadata {
                        ..Default::default()
                    })),
                    ..Default::default()
                };
                hydrate(&t, &mut tk);
                return Ok(Some(tk));
            }
            Err(e) => {
                return match e.kind {
                    metaflac::ErrorKind::InvalidInput => Ok(None),
                    metaflac::ErrorKind::Io(k) => Err(Box::new(k)),
                    _ => Err(Box::new(e)),
                };
            }
        }
    }
}
fn hydrate(t: &Tag, tk: &mut track::Track) {
    for b in t.blocks() {
        match b {
            Block::StreamInfo(si) => si_hydrate(si, tk),
            Block::VorbisComment(vc) => vorbis_hydrate(&vc, tk),
            // Block::SeekTable(st) => println!("{:?}", st),
            // Block::CueSheet(cs) => println!("CueSheet: {:?}", cs),
            // Block::Application(ap) => println!("Application: {:?}", ap),
            // Block::Padding(pd) => println!("Padding: {:?}", pd),
            // Block::Picture(p) => println!("{:?}", p),
            // Block::Unknown(b) => println!("Unknown {:?}", b),
            _ => (), // TODO(jdr) should figure out how to attach arbitrary data to a track.
        }
    }
}

fn si_hydrate(si: &metaflac::block::StreamInfo, tk: &mut track::Track) {
    let f = track::PCMFormat {
        sample_rate: si.sample_rate,
        channels: si.num_channels,
        bits_per_sample: si.bits_per_sample as u16,
        total_samples: si.total_samples,
    };
    tk.format = Some(track::CodecFormat::PCM(f));
}

fn vorbis_hydrate(vc: &metaflac::block::VorbisComment, tk: &mut track::Track) {
    // there really must be a way to collect
    // tuples of vc.title and self.title and
    // run them in a loop to do this.
    if let Some(ts) = vc.title() {
        tk.title = Some(ts.join("/")); // TODO: Is this what we want from the vector result?
    }

    tk.track_number = vc.track();

    if let Some(a) = vc.album() {
        tk.album = Some(a.join("/"));
    }

    if let Some(a) = vc.artist() {
        tk.artist = Some(a.join("/"));
    } else {
        if let Some(a) = vc.album_artist() {
            tk.artist = Some(a.join("/"));
        }
    }

    // Fill in the flac metadata.
    // TODO: Is there a more efficient way to do this?
    // Can we hand ownership over?
    if let Some(md) = &mut tk.metadata {
        if let track::FormatMetadata::Flac(md) = md {
            for (k, v) in &vc.comments {
                md.comments.insert(k.clone(), v.clone());
            }

            // Check for alternate
            tk.track_total = vc.total_tracks();
            if tk.track_total.is_none() {
                if let Some(tt) = md.comments.get(ALT_TOTALTRACKS) {
                    if let Ok(t_total) = tt[0].parse::<u32>() {
                        tk.track_total = Some(t_total);
                    }
                }
            }

            // Now fill from comments.
            if let Some(dt) = md.comments.get(DISCTOTAL) {
                if let Ok(d_total) = dt[0].parse::<u32>() {
                    tk.disk_total = Some(d_total);
                }
            }
            if let Some(dn) = md.comments.get(DISCNUMBER) {
                if let Ok(d_num) = dn[0].parse::<u32>() {
                    tk.disk_number = Some(d_num);
                }
            }
        }
    };
}
