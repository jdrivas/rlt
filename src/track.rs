use crate::flac::Flac;
use crate::id3::Id3;
use crate::wav::Wav;
// use id3::Tag as ID3Tag;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
// use std::io::SeekFrom;
use std::io::{Read, Seek};
use std::path;
use std::time::Duration;
use std::vec::Vec;

#[derive(Default, Debug)]
pub struct SampleFormat {
  pub sample_rate: u32, // Hertz
  pub channels: u8,
  pub bits_per_sample: u16,
  pub total_samples: u64,
  pub duration: Duration,
}

const BILLION: u64 = 1_000_000_000;
impl SampleFormat {
  pub fn duration(&self) -> Duration {
    // Compute duration
    let mut ns = self.total_samples as f64 / self.sample_rate as f64;
    ns = ns * BILLION as f64;
    return Duration::from_nanos(ns as u64);
  }
}

// #[derive(Default, Debug)]
#[derive(Debug)]
pub struct Track {
  pub path: path::PathBuf,
  pub title: Option<String>,
  pub artist: Option<String>,
  pub album: Option<String>,
  pub track_number: Option<u32>,
  pub track_total: Option<u32>,
  pub disk_number: Option<u32>,
  pub disk_total: Option<u32>,
  pub comments: HashMap<String, Vec<String>>,
  pub format: SampleFormat,
}

impl Default for Track {
  fn default() -> Self {
    Track {
      path: path::PathBuf::default(),
      title: None,
      artist: None,
      album: None,
      track_number: None,
      track_total: None,
      disk_number: None,
      disk_total: None,
      format: SampleFormat::default(),
      comments: HashMap::new(),
    }
  }
}

// const EMPTY_VALUE: &str = "<EMPTY>";
const EMPTY_SMALL: &str = "-";

impl Track {
  pub fn tracks_display(&self) -> String {
    match self.track_total {
      Some(tt) => match self.track_number {
        Some(tn) => format!("{:2} of {:02}", tn, tt),
        None => format!("{:2}", tt),
      },
      None => match self.track_number {
        Some(tn) => format!("{:2}", tn),
        None => EMPTY_SMALL.to_string(),
      },
    }
  }
}
/// Read track(s) from a file or directory.
pub fn files_from(
  p: path::PathBuf,
) -> std::result::Result<(Vec<Track>, Vec<path::PathBuf>), Box<dyn Error>> {
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

  // Filter them into regular files and tracks getting data for the tracks.
  let mut files = Vec::new();
  let mut tracks = Vec::new();
  for p in paths {
    if p.as_path().is_dir() {
      // Directories are not traversed, just listed.
      files.push(p);
    } else {
      match get_track(&p)? {
        Some(mut tk) => {
          // Some format don't provide track titles.
          // Let's use the file name if they don't.
          if tk.title.is_none() {
            let f_n = match p.as_path().file_name() {
              Some(s) => s.to_string_lossy().into_owned(),
              None => tk.path.as_path().to_string_lossy().into_owned(),
            };
            tk.title = Some(f_n);
          }
          tk.path = p;
          tracks.push(tk);
        }
        None => files.push(p),
      }
    }
  }

  tracks.sort_by(|a, b| {
    a.disk_number
      .cmp(&b.disk_number)
      .then(a.track_number.cmp(&b.track_number))
  });

  Ok((tracks, files))
}

pub trait Decoder {
  fn is_candidate<R: Read + Seek>(r: R) -> Result<bool, Box<dyn Error>>;
  fn get_track<R: Read + Seek>(r: R) -> Result<Option<Track>, Box<dyn Error>>;
}

pub fn get_track(p: &path::PathBuf) -> Result<Option<Track>, Box<dyn Error>> {
  // TODO(jdr): I just gotta believe there is a better way.
  // Would really like to get these into one structure rather than two.
  // An array of tuples (Flac::is_canddiate, Flac::get_track) the compiler discovers
  // as type of the first specific decoder object (e.g. flac::Flac), really as you'd expect
  // because those are different real objects.
  // Not sure how to define a struct to take one of these as there are no real objects.
  // Also - it would be good to just read these in somehow, though dynamic libraries
  // in a well known place seems about too clever by 1/2.
  let candidates = [Flac::is_candidate, Wav::is_candidate, Id3::is_candidate];
  let gets = [Flac::get_track, Wav::get_track, Id3::get_track];

  let f = File::open(&p)?;
  for i in 0..candidates.len() {
    if candidates[i](&f)? {
      return Ok(gets[i](&f)?);
    }
  }

  return Ok(None);
}
