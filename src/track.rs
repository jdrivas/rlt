use crate::file;
use crate::file::{Decoder, FileFormat};

use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
// use std::io::SeekFrom;
// use std::io::{Read, Seek};
use std::path;
use std::time::Duration;
use std::vec::Vec;

// CodecFormats

#[derive(Default, Debug)]
pub struct PCMFormat {
  pub sample_rate: u32, // Hertz
  pub channels: u8,
  pub bits_per_sample: u16,
  pub total_samples: u64,
}

const BILLION: u64 = 1_000_000_000;
impl PCMFormat {
  pub fn duration(&self) -> Duration {
    // Compute duration
    let mut ns = self.total_samples as f64 / self.sample_rate as f64;
    ns = ns * BILLION as f64;
    return Duration::from_nanos(ns as u64);
  }
}

#[derive(Debug)]
pub struct MPEGFormat {
  pub bitrate: u16,
  pub sample_rate: u16,
  pub version: MPVersion,
  pub layer: MPLayer,
  pub duration: Duration,
}

impl MPEGFormat {
  pub fn version_string(&self) -> String {
    match &self.version {
      MPVersion::Reserved => "Reserved".to_string(),
      MPVersion::MPEG1 => "Mpeg-1".to_string(),
      MPVersion::MPEG2 => "Mpeg-2".to_string(),
      MPVersion::MPEG2_5 => "Mpeg-2.5".to_string(),
      MPVersion::Unknown => "Unknwon".to_string(),
    }
  }

  pub fn layer_string(&self) -> String {
    match &self.layer {
      MPLayer::Reserved => "Reserved".to_string(),
      MPLayer::Layer1 => "Layer-1".to_string(),
      MPLayer::Layer2 => "Layer-2".to_string(),
      MPLayer::Layer3 => "Layer-3".to_string(),
      MPLayer::Unknown => "Unknwon".to_string(),
    }
  }
}

#[derive(Debug)]
pub enum MPVersion {
  Reserved,
  MPEG1,
  MPEG2,
  MPEG2_5,
  Unknown,
}

#[derive(Debug)]
pub enum MPLayer {
  Reserved,
  Layer1,
  Layer2,
  Layer3,
  Unknown,
}

#[derive(Debug)]
pub enum CodecFormat {
  PCM(PCMFormat),
  MPEG(MPEGFormat),
}

// Metadata
#[derive(Debug)]
pub enum FormatMetadata {
  Flac(FlacMetadata),
  ID3(ID3Metadata),
}

#[derive(Debug, Default)]
pub struct FlacMetadata {
  pub comments: HashMap<String, Vec<String>>,
}

#[derive(Debug, Default)]
pub struct ID3Metadata {
  pub text: HashMap<String, Vec<String>>,
  pub comments: HashMap<String, Vec<(String, String, String)>>,
}

// Track Definition

// #[derive(Default, Debug)]
#[derive(Debug)]
pub struct Track {
  pub path: path::PathBuf,
  pub file_format: Option<String>,
  pub title: Option<String>,
  pub artist: Option<String>,
  pub album: Option<String>,
  pub track_number: Option<u32>,
  pub track_total: Option<u32>,
  pub disk_number: Option<u32>,
  pub disk_total: Option<u32>,
  // pub comments: HashMap<String, Vec<String>>,
  pub format: Option<CodecFormat>,
  pub metadata: Option<FormatMetadata>,
}

impl Default for Track {
  fn default() -> Self {
    Track {
      path: path::PathBuf::default(),
      file_format: None,
      title: None,
      artist: None,
      album: None,
      track_number: None,
      track_total: None,
      disk_number: None,
      disk_total: None,
      format: None,
      metadata: None,
      // comments: HashMap::new(),
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

/// Get a track from a file specified by path.
/// This will try to read the file's meta-data against any installed
/// decoders. Currently we look at: Flac, ID3, and WAV.
/// Working on mp4.
pub fn get_track(p: &path::PathBuf) -> Result<Option<Track>, Box<dyn Error>> {
  let file = File::open(p.as_path())?;

  if let Some(f) = file::identify(&file)? {
    match f {
      FileFormat::Flac(mut d) => return Ok(d.get_track(&file)?),
      FileFormat::WAV(mut d) => return Ok(d.get_track(&file)?),
      FileFormat::MP3(mut d) => return Ok(d.get_track(&file)?),
      FileFormat::ID3(mut d) => return Ok(d.get_track(&file)?),
      _ => return Ok(None),
    }
  }
  return Ok(None);
}
