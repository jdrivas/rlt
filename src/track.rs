use crate::flac;
// use crate::id3;
use crate::mp3;
use crate::wav;

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
  pub file_format: String,
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
      file_format: String::default(),
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
  fn is_candidate(&mut self) -> Result<bool, Box<dyn Error>>;
  fn get_track(&mut self) -> Result<Option<Track>, Box<dyn Error>>;
  // fn is_candidate<R: Read + Seek>(r: R) -> Result<bool, Box<dyn Error>>;
  // fn get_track<R: Read + Seek>(r: R) -> Result<Option<Track>, Box<dyn Error>>;
}

/// Get a track from a file specified by path.
/// This will try to read the file's meta-data against any installed
/// decoders. Currently we look at: Flac, ID3, and WAV.
/// Working on mp4.
pub fn get_track(p: &path::PathBuf) -> Result<Option<Track>, Box<dyn Error>> {
  // TODO(jdr)
  // To get here, instead of iplementing a trait on a unit struct (e.g. struct flac::Flac;),
  // I created separate functions and the struct below that captures them.
  // Moreover, I couldn't use the more generic definition for the arguments
  // that I wanted  fn<R: Read + Seek>(r: R), opting instead for the concreate:
  // fn< std::fs::File).
  // I like this better than all of the other alternatives however.
  // struct Decoder {
  //   is_candidate: fn(&mut std::fs::File) -> Result<bool, Box<dyn Error>>,
  //   get_track: fn(&mut std::fs::File) -> Result<Option<Track>, Box<dyn Error>>,
  // }

  // Decoders are checked in order. If a candidate is
  // found then the get_track is executed.
  // let decoders = [
  //   Decoder {
  //     is_candidate: flac::is_candidate,
  //     get_track: flac::get_track,
  //   },
  //   Decoder {
  //     is_candidate: mp3::is_candidate,
  //     get_track: mp3::get_track,
  //   },
  //   Decoder {
  //     is_candidate: id3::is_candidate,
  //     get_track: id3::get_track,
  //   },
  //   Decoder {
  //     is_candidate: wav::is_candidate,
  //     get_track: wav::get_track,
  //   },
  // ];

  // // let mut f = File::open(&p)?;
  // for d in &decoders {
  //   if (d.is_candidate)(p)? {
  //     return Ok((d.get_track)(p)?);
  //   }
  // }

  let mut decoders: Vec<Box<dyn Decoder>> = Vec::new();
  decoders.push(Box::new(flac::Flac::new(&p)));
  decoders.push(Box::new(wav::Wav::new(&p)));
  decoders.push(Box::new(mp3::Mp3::new(&p)));

  for d in &mut decoders {
    if d.is_candidate()? {
      return Ok(d.get_track()?);
    }
  }

  return Ok(None);
}
