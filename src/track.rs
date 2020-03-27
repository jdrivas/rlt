// extern crate chrono;
extern crate defer;

use metaflac::{Block, Tag};
use std::collections::HashMap;
use std::io;
use std::path;
use std::time::Duration;
use std::vec::Vec;

#[derive(Default, Debug)]
pub struct SampleFormat {
  pub sample_rate: u32, // Hertz
  pub channels: u8,
  pub bits_per_sample: u8,
  pub total_samples: u64,
}

impl SampleFormat {
  fn fill_with_flac(&mut self, si: &metaflac::block::StreamInfo) {
    self.sample_rate = si.sample_rate;
    self.channels = si.num_channels;
    self.bits_per_sample = si.bits_per_sample;
    self.total_samples = si.total_samples;
  }
}

// #[derive(Default, Debug)]
#[derive(Debug)]
pub struct Track {
  pub path: path::PathBuf,
  pub title: String,
  pub artist: String,
  pub album: String,
  pub track_number: u32,
  pub total_tracks: u32,
  pub duration: Duration,
  pub comments: HashMap<String, Vec<String>>,
  pub format: SampleFormat,
}

impl Default for Track {
  fn default() -> Self {
    Track {
      duration: Duration::from_secs(0),
      path: path::PathBuf::default(),
      title: String::default(),
      artist: String::default(),
      album: String::default(),
      track_number: 0,
      total_tracks: 0,
      format: SampleFormat::default(),
      comments: HashMap::new(),
    }
  }
}

const BILLION: u64 = 1_000_000_000;

impl Track {
  fn fill_from_tag(&mut self, t: &Tag) {
    for b in t.blocks() {
      match b {
        Block::StreamInfo(si) => self.format.fill_with_flac(&si),
        Block::VorbisComment(vc) => self.fill_with_vorbis(&vc),
        _ => (), // We'll just eat other blocks for now. println!("Block: {:?}", b),
      }
    }

    // Compute duration
    let mut ns = self.format.total_samples as f64 / self.format.sample_rate as f64;
    ns = ns * BILLION as f64;
    self.duration = Duration::from_nanos(ns as u64);
  }

  fn fill_with_vorbis(&mut self, vc: &metaflac::block::VorbisComment) {
    // there really must be a way to collect
    // tuples of vc.title and self.title and
    // run them in a loop to do this.
    if let Some(ts) = vc.title() {
      self.title = ts.join("/");
    }

    if let Some(tn) = vc.track() {
      self.track_number = tn;
    }

    if let Some(tt) = vc.total_tracks() {
      self.total_tracks = tt;
    }

    if let Some(a) = vc.album() {
      self.album = a.join("/");
    }

    if let Some(a) = vc.artist() {
      self.artist = a.join("/");
    } else {
      if let Some(a) = vc.album_artist() {
        self.artist = a.join("/");
      }
    }

    // copy the comments in.
    // TODO: Is there a more efficient way to do this?
    for (k, v) in &vc.comments {
      self.comments.insert(k.clone(), v.clone());
    }
  }
}

/// Read track(s) from a file or directory.
pub fn files_from(p: path::PathBuf) -> io::Result<(Vec<Track>, Vec<path::PathBuf>)> {
  // Get a list of paths we want to look at.
  let mut paths = Vec::new();
  if p.is_dir() {
    for f in p.read_dir()? {
      if let Ok(f) = f {
        paths.push(f.path());
      }
    }
  } else {
    if p.is_file() {
      paths.push(p);
    }
  }

  // Filter them into regular files and tracks and get data for the tracks.
  let mut files = Vec::new();
  let mut tracks = Vec::new();
  for p in paths {
    if p.as_path().is_dir() {
      // Directories are not traversed, just listed.
      files.push(p);
    } else {
      match Tag::read_from_path(&p) {
        Ok(t) => {
          let mut tk = Track {
            path: p,
            ..Default::default()
          };
          tk.fill_from_tag(&t);
          tracks.push(tk);
        }
        Err(e) => match e.kind {
          metaflac::ErrorKind::InvalidInput => files.push(p),
          metaflac::ErrorKind::Io(k) => return Err(k),
          _ => eprintln!("Metaflac Error: {}", e),
        },
      }
    }
  }

  Ok((tracks, files))
}
