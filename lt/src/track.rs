//! The model for an audio track generally, and as a function of file and audio codec format.

extern crate chrono;
use chrono::{DateTime, NaiveDateTime, Utc};

use crate::file;
use crate::file::{Decoder, FileFormat};

use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs::File;
// use std::io::SeekFrom;
// use std::io::{Read, Seek};
use std::io::Write;
use std::path;
use std::time::Duration;
use std::vec::Vec;

use crate::mpeg4;
use format::consts::FORMAT_CLEAN;
use mpeg4::formats::{AudioObjectTypes, ChannelConfig};
use prettytable::{format, Table};

//
// CodecFormats
//

//
// PCM
//

/// CodecFormats
/// Describes sample data based on underlying encoding.
#[derive(Debug)]
pub enum CodecFormat {
  /// Describes PCM audio data
  PCM(PCMFormat),
  /// Describes MPEG 3 Describes audio data
  MPEG3(MPEG3Format),
  /// Describes MPEG 4 Describes audio data.
  MPEG4(MPEG4AudioFormat),
}

/// PCM Codec Format
/// Basic PCM sample data.
#[derive(Default, Debug)]
pub struct PCMFormat {
  /// Sample rate in hertz.
  pub sample_rate: u32,
  /// Channels of audio (eg. 1 for mono, 2 for stereo etc.).
  pub channels: u8,
  /// Sample size (e.g. 16, 24, 48).
  pub bits_per_sample: u16,
  /// Numnber of samples for this track.
  /// Used to compute duraiton.
  pub total_samples: u64,
}

const BILLION: u64 = 1_000_000_000;
impl PCMFormat {
  /// Length of time for the track.
  pub fn duration(&self) -> Duration {
    // Compute duration
    let mut ns = self.total_samples as f64 / self.sample_rate as f64;
    ns *= BILLION as f64;
    Duration::from_nanos(ns as u64)
  }
}

//
// MPEG3
//

/// MPEG-3 Codec Format
/// Representation of MPeg layers 1 - layers 3.
pub struct MPEG3Format {
  /// Encoded  stream bitrate.
  pub bitrate: u16,
  /// Source sample rate.
  pub sample_rate: u16,
  /// Mpeg audio encoding version.
  pub version: MPVersion,
  /// Mpeg audio encoding layer.
  pub layer: MP3Layer,
  /// Track duration.
  pub duration: Duration,
}

/// Return a printable string for MPEG-3 version.
impl MPEG3Format {
  /// Printable string for MPEG audio version (e.g. MPEG-1).
  pub fn version_string(&self) -> String {
    match &self.version {
      MPVersion::Reserved => "Reserved".to_string(),
      MPVersion::MPEG1 => "MPEG-1".to_string(),
      MPVersion::MPEG2 => "MPEG-2".to_string(),
      MPVersion::MPEG2_5 => "MPEG-2.5".to_string(),
      MPVersion::Unknown => "Unknwon".to_string(),
    }
  }

  ///  Printable string for MPEG audio layer (e.g. Layer 3).
  pub fn layer_string(&self) -> String {
    match &self.layer {
      MP3Layer::Reserved => "Reserved".to_string(),
      MP3Layer::Layer1 => "Layer 1".to_string(),
      MP3Layer::Layer2 => "Layer 2".to_string(),
      MP3Layer::Layer3 => "Layer 3".to_string(),
      MP3Layer::Unknown => "Unknwon".to_string(),
    }
  }
}

/// Suitable for printing description of this format: Mpeg-1 - Layer 3.
impl fmt::Debug for MPEG3Format {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{} - {}", self.version_string(), self.layer_string())
  }
}

/// MPEG audio version.
#[derive(Debug)]
pub enum MPVersion {
  Reserved,
  MPEG1,
  MPEG2,
  MPEG2_5,
  Unknown,
}

/// MPEG audio layer.
#[derive(Debug)]
pub enum MP3Layer {
  Reserved,
  Layer1,
  Layer2,
  Layer3,
  Unknown,
}

//
// MPEG4
//

/// MPEG-4 CodecFormat
#[derive(Default, Debug)]
pub struct MPEG4AudioFormat {
  /// 16.16 fixed point sample rate.
  pub sr: u32,
  /// Channels of audio (eg. 1 for mono, 2 for stereo etc.).
  pub channels: u8,
  /// Channel Configuration
  pub channel_config: ChannelConfig,
  /// Sample size (e.g. 16, 24, 48).
  pub bits_per_sample: u16,
  /// Numnber of samples for this track.
  pub total_samples: u64,
  /// Audio Codec (e.g. AAC)
  pub codec: AudioObjectTypes,
  /// Geenral class of decoder used.
  pub decoder: u8,
  /// Maximum bitrate used by the stream.
  pub max_bitrate: u32,
  /// Average bitrate used by the stream.
  pub avg_bitrate: u32,
}

impl MPEG4AudioFormat {
  /// Samples per second.
  pub fn sample_rate(&self) -> f64 {
    f64::from(self.sr >> 16)
  }
  /// Legnth of the track
  pub fn duration(&self) -> Duration {
    let mut ns = self.total_samples as f64 / self.sample_rate();
    ns *= BILLION as f64;
    Duration::from_nanos(ns as u64)
  }
}

//
// Format Specific Metadata
//

/// Generalized access to audio metadata provided by
/// the underlying format. This can be thought of as a way
/// to get access to metadata that isn't otherwise captured
/// formally by the Track data structure.
/// This also allows for a bit of evolution in the formats as they grow.
#[derive(Debug)]
pub enum FormatMetadata {
  /// Flac specific metadata
  Flac(FlacMetadata),
  /// ID3 metadata format.
  ID3(ID3Metadata),
  /// Mpeg4 specific metadata.
  MP4(MPEG4Metadata),
}

//
// FLAC
//

/// Flac Format Metadata
///
/// Flac metadata is supplied with Key/Values where mutiple
/// strings can be supplied for the same key (though usually that's
/// not the case - it's just a single string).
#[derive(Debug, Default)]
pub struct FlacMetadata {
  /// Flac metadata is stored as comments and key/value pairs.
  pub comments: HashMap<String, Vec<String>>,
}

impl FlacMetadata {
  /// Print the metadata, as key values in columns, to a writer.
  pub fn print(&self, mut w: impl Write) -> Result<(), std::io::Error> {
    println!("Metadata");
    if !self.comments.is_empty() {
      let mut table = Table::new();
      table.set_format(*FORMAT_CLEAN);
      table.add_row(row!["Key", "Value"]);

      let mut vs: Vec<_> = self.comments.iter().collect();
      vs.sort();
      for (k, v) in vs {
        table.add_row(row![k, v[0]]);
        let mut i = 1;
        while i < v.len() {
          table.add_row(row!["", v[i]]);
          i += 1;
        }
      }

      // Dump the table to the writer.
      if let Err(e) = table.print(&mut w) {
        return Err(e);
      };
    } else {
      write!(w, "No Comments.")?;
    }
    Ok(())
  }
}

//
// ID3
//

/// ID3 Format Metadata
#[derive(Debug, Default)]
pub struct ID3Metadata {
  /// Text metadata is where metdata is usually stored.bool
  pub text: HashMap<String, Vec<String>>,
  /// Comment metadata is stored by key with values: Languages, Description, Text.
  pub comments: HashMap<String, Vec<(String, String, String)>>,
}

impl ID3Metadata {
  /// Print the metadata, as key values in columns, to a writer.
  pub fn print(&self, mut w: impl Write) -> Result<(), std::io::Error> {
    println!("Metadata");

    println!("\nText");
    if !self.text.is_empty() {
      let mut table = Table::new();
      table.set_format(*FORMAT_CLEAN);
      table.add_row(row!["Key", "Value"]);

      let mut vs: Vec<_> = self.text.iter().collect();
      vs.sort();
      for (k, v) in vs {
        table.add_row(row![k, v[0]]);
        let mut i = 1;
        while i < v.len() {
          table.add_row(row!["", v[i]]);
          i += 1;
        }
      }

      // Dump the table to the writer.
      if let Err(e) = table.print(&mut w) {
        return Err(e);
      };
    } else {
      write!(w, "No text.")?;
    }

    println!("\nComments");
    if !self.comments.is_empty() {
      let mut table = Table::new();
      table.set_format(*FORMAT_CLEAN);
      table.add_row(row!["Key", "Lang", "Description", "Text"]);

      let mut vs: Vec<_> = self.comments.iter().collect();
      vs.sort();
      for (k, v) in vs {
        table.add_row(row![k, v[0].0, v[0].1, v[0].2]);
        let mut i = 1;
        while i < v.len() {
          table.add_row(row!["", v[0].0, v[0].1, v[0].2]);
          i += 1;
        }
      }

      // Dump the table to the writer.
      if let Err(e) = table.print(&mut w) {
        return Err(e);
      };
    } else {
      write!(w, "No Comments.")?;
    }
    Ok(())
  }
}

//
// MPEG4
//

/// MPeg 4 Format Metadata
///
/// This is pulled from the ilst box found: /moov/udta/ilist.
/// The structure is
/// ```nothing
/// ilst
///     <key>
///         data
/// ```
/// Where the `<key>` is the four character code for the metadata type e.g. trkn for "track number".
///
/// The data box also has a type associated with it.
///
/// Here the data is stored in a `HashMap` with the `key` represtend in a string as the key.
/// The values are depened on the identified type of the data.
///
/// Type == 1, is stored as &[u8] which is converted to a string in the text `HashMap`.
///
/// Type == 21, is stored a single byte, in the byte `HashMap`.
///
/// Types: 0 (Implicit), 13(JPEG), 14(PMG) are not stored at the moment.
///
#[derive(Debug)]
pub struct MPEG4Metadata {
  /// Stored text type data as string, keyed off of the enclosing box's 4 character code e.g. `b"trkn"`.
  pub text: HashMap<String, String>,
  // pub data: HashMap<String, [u8]>, TODO(jdr): Figure out how to capture Data typed metadata.
  /// Stored single byte data as a single byte keyed on the enclsoing box's 4 character code e.g. `b"cpil"`.
  pub byte: HashMap<String, u8>,

  /// Time the media was created.
  pub creation: DateTime<Utc>,

  /// Time the media was last update.
  pub modification: DateTime<Utc>,

  /// The size of Media Box.
  /// This is the size of the actual media data, independent of
  /// the metadata. It could be expressed as a percentage of the file
  /// size.
  pub media_size: u32,
}

impl Default for MPEG4Metadata {
  fn default() -> Self {
    MPEG4Metadata {
      text: HashMap::<String, String>::default(),
      byte: HashMap::<String, u8>::default(),
      /// TODO(jdr): Seriously consider using Option's here.
      creation: DateTime::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc),
      modification: DateTime::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc),
      media_size: 0,
    }
  }
}

impl MPEG4Metadata {
  /// Print the metadata, as key values in columns, to a writer.
  pub fn print(&self, mut w: impl Write) -> Result<(), std::io::Error> {
    println!("Metadata");
    if !self.text.is_empty() || !self.byte.is_empty() {
      let mut table = Table::new();
      table.set_format(*FORMAT_CLEAN);
      table.add_row(row!["Key", "Value"]);

      // Print the text ones first.
      let mut vs: Vec<_> = self.text.iter().collect();
      vs.sort();
      for (k, v) in vs {
        table.add_row(row![k, v]);
      }

      // Then the single bytes ones.
      let mut vs: Vec<_> = self.byte.iter().collect();
      vs.sort();
      for (k, v) in vs {
        table.add_row(row![k, v]);
      }

      // Display the table.
      if let Err(e) = table.print(&mut w) {
        return Err(e);
      };
    } else {
      write!(w, "No Metadata.")?;
    }
    Ok(())
  }
}

// Track Definition

/// Captures general and codec specific metadata for a single audio track.
// #[derive(Default, Debug)]
#[derive(Debug)]
pub struct Track {
  /// File location.
  pub path: path::PathBuf,
  /// File Format (as opposed to data format (e.g. Mpeg4 File with Flac codec data).
  pub file_format: Option<String>,
  /// Track title.
  pub title: Option<String>,
  /// Artist name.
  pub artist: Option<String>,
  /// Album title.
  pub album: Option<String>,
  /// Track number out of a total number of tracks.
  pub track_number: Option<u32>,
  /// Total number of tracks for the album/collection this track is a part of.
  pub track_total: Option<u32>,
  /// The CD that this track appears on.
  pub disk_number: Option<u32>,
  /// The total number of CDs for the album that this track appears on.
  pub disk_total: Option<u32>,
  // pub comments: HashMap<String, Vec<String>>,
  /// Codec format details.
  pub format: Option<CodecFormat>,
  /// A collection of format specific metadata provided as key value pairs
  /// to capture metadata not otherwise provided by the Track and assocaited
  /// structs and enums.
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
  /// Utilitiy function to return a string that
  /// captures tracks as: " 1 of 09".
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
/// Read track(s) and regular files from a file or directory.
///
/// `PathBuf` provides the file or directory.
/// The first returned `Vec` are files that are audio files and so have track data.
/// The second `Vec` is a list of non-audio files as `PathBuf`.
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
  } else if p.is_file() {
    paths.push(p);
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
  let mut file = File::open(p.as_path())?;

  // Still not happy with this.
  // Need to figure out how to use the fact that these
  // are all file::Decoders.
  if let Some(f) = file::identify(&mut file)? {
    match f {
      FileFormat::Flac(mut d) => return Ok(d.get_track(&file)?),
      FileFormat::MPEG4(mut d) => return Ok(d.get_track(&file)?),
      FileFormat::MP4A(mut d) => return Ok(d.get_track(&file)?),
      FileFormat::WAV(mut d) => return Ok(d.get_track(&file)?),
      FileFormat::MP3(mut d) => return Ok(d.get_track(&file)?),
    }
  }
  Ok(None)
}
