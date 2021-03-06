//! Output for the application.
extern crate num_format;
use crate::album;
use crate::file;
use crate::mpeg4;
use crate::track;

use format::consts::FORMAT_CLEAN;
use num_format::{Locale, ToFormattedString};
use prettytable::{format, Cell, Row, Table};

use std::env;
use std::error::Error;
use std::io;
use std::io::Read;
use std::path::PathBuf;
use std::time::Duration;
const NONE_SHORT: &str = "-";

/// Display regular files and audio files, separately dispaying
/// metadata of the audio file if found.
pub fn list_files(mut p: PathBuf) -> Result<(), Box<dyn Error>> {
  if !p.exists() {
    p = env::current_dir()?
  }

  // Make sure we can find it ....
  let album;
  let files;
  if !p.as_path().exists() {
    return Err(Box::new(std::io::Error::new(
      std::io::ErrorKind::NotFound,
      format!("File not found: {}", p.as_path().display()),
    )));
  }
  // If it's a file get the track build an album around it.
  // Otherwise, build an album around the files in the directory.
  if p.is_file() {
    let (tracks, f) = track::files_from(p)?;
    album = album::album_from_tracks(tracks);
    files = f;
  } else {
    p = dir_or_cwd(p)?;
    let (a, f) = album::album_from_path(p)?;
    album = a;
    files = f;
  }

  if !album.tracks.is_empty() {
    // Display album information
    println!();
    println!(
      "Album: {}",
      album.title.unwrap_or_else(|| { NONE_SHORT.to_string() })
    );
    println!(
      "Artist: {}",
      album.artist.unwrap_or_else(|| { NONE_SHORT.to_string() })
    );
    if album.disk_total.unwrap_or(0) > 0 {
      println!(
        "Disks: {}",
        album
          .disk_total
          .map_or(NONE_SHORT.to_string(), |v| v.to_string())
      );
    }
    println!(
      "Total Tracks: {}",
      album
        .track_total
        .map_or(NONE_SHORT.to_string(), |v| v.to_string())
    );

    // Display track information
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_CLEAN);

    // Most cases we have no disc numbers so add the track header now
    // TODO(jdr): Checking the first track for this seems to work, but feels wrong.
    if album.tracks[0].disk_number.is_none() {
      table.add_row(title_row(&album.tracks[0].format));
    }
    let mut ld = None;
    for t in &album.tracks {
      // Handle disc numbers and the track header wtih them.
      let cd = t.disk_number;
      if ld != cd {
        ld = cd;
        if let Some(cd) = cd {
          table.add_row(row![format!("\nDisk: {}", cd)]);
          table.add_row(title_row(&t.format));
        }
      }
      // display track data.
      let pn = path_file_name(&t.path);
      if let Some(c) = &t.format {
        match c {
          track::CodecFormat::PCM(f) => {
            table.add_row(row![
              t.tracks_display(),
              t.title.as_ref().unwrap_or(&NONE_SHORT.to_string()),
              format_duration(&f.duration(), true),
              format!("{} KHz", (f.sample_rate as f64 / 1000.0)),
              format!("{} bits", f.bits_per_sample.to_string()),
              t.file_format.as_ref().unwrap_or(&NONE_SHORT.to_string()),
              pn,
            ]);
          }
          track::CodecFormat::MPEG3(f) => {
            table.add_row(row![
              t.tracks_display(),
              t.title.as_ref().unwrap_or(&NONE_SHORT.to_string()),
              format_duration(&f.duration, true),
              format!("{} ", f.sample_rate.to_string()),
              format!("{:>3} Kbps", f.bitrate.to_string()),
              f.version_string(),
              f.layer_string(),
              t.file_format.as_ref().unwrap_or(&NONE_SHORT.to_string()),
              pn,
            ]);
          }
          track::CodecFormat::MPEG4(f) => {
            table.add_row(row![
              t.tracks_display(),
              t.title.as_ref().unwrap_or(&NONE_SHORT.to_string()),
              format_duration(&f.duration(), true),
              format!("{} KHz", (f.sample_rate() / 1000.0)),
              format!("{} bits", f.bits_per_sample.to_string()),
              format!("{}", f.codec),
              format!(
                "{} Kbps",
                (f.avg_bitrate / 1000).to_formatted_string(&Locale::en)
              ),
              pn,
            ]);
          }
        }
      } else {
        table.add_row(row![
          t.tracks_display(),
          t.title.as_ref().unwrap_or(&NONE_SHORT.to_string()),
          NONE_SHORT.to_string(),
          NONE_SHORT.to_string(),
          NONE_SHORT.to_string(),
          t.file_format.as_ref().unwrap_or(&NONE_SHORT.to_string()),
          pn,
        ]);
      }
    }
    table.printstd();
  }

  // let (tracks, files) = track::files_from(path)?;
  if !files.is_empty() {
    println!("\nFiles:");
    for f in files {
      println!("{}", path_file_name(&f));
    }
  }

  Ok(())
}

/// Prints a detailed description of an audio file, including listing
/// all found metadata.
pub fn describe_file(p: PathBuf) -> Result<(), Box<dyn Error>> {
  // Only do a single file at a time.
  if !p.is_file() {
    return Err(Box::new(io::Error::new(
      io::ErrorKind::Other,
      format!("{} is not a file.", p.as_path().display()),
    )));
  }

  let (tracks, files) = track::files_from(p)?;
  if !tracks.is_empty() {
    // this is overkill as we sould only get one file back.
    for tk in tracks {
      describe_track(tk)?;
    }
  } else if !files.is_empty() {
    for f in files {
      println!("{}", f.display());
    }
  }

  Ok(())
}

struct Te<'a>(&'a str, String);

fn print_te_list(v: Vec<Te>) {
  let mut table = Table::new();
  table.set_format(*FORMAT_CLEAN);
  for t in &v {
    table.add_row(row![t.0, t.1]);
  }
  table.printstd();
}

/// Prints a detailed track description, including listing of all found
/// metadata.
fn describe_track(tk: track::Track) -> Result<(), Box<dyn Error>> {
  let (fs_str, fs_int) = match tk.path.as_path().metadata() {
    Ok(md) => (md.len().to_formatted_string(&Locale::en), md.len()),
    Err(_) => ("<Unknown>".to_string(), 0),
  };

  let mut tes = Vec::<Te>::new();

  // Basic track info.
  tes.push(Te("File", path_file_name(&tk.path)));
  tes.push(Te(
    "File Format",
    tk.file_format.unwrap_or_else(|| NONE_SHORT.to_string()),
  ));
  tes.push(Te(
    "Album",
    tk.album.unwrap_or_else(|| NONE_SHORT.to_string()),
  ));
  tes.push(Te(
    "Album Artist",
    tk.album_artist.unwrap_or_else(|| NONE_SHORT.to_string()),
  ));
  tes.push(Te(
    "Title",
    tk.title.unwrap_or_else(|| NONE_SHORT.to_string()),
  ));
  tes.push(Te(
    "Artist",
    tk.artist.unwrap_or_else(|| NONE_SHORT.to_string()),
  ));
  tes.push(Te(
    "Track",
    tk.track_number
      .map_or(NONE_SHORT.to_string(), |v| v.to_string()),
  ));
  tes.push(Te(
    "Track Total",
    tk.track_total
      .map_or(NONE_SHORT.to_string(), |v| v.to_string()),
  ));
  tes.push(Te(
    "Disk Number",
    tk.disk_number
      .map_or(NONE_SHORT.to_string(), |v| v.to_string()),
  ));
  tes.push(Te(
    "Disk Total",
    tk.disk_total
      .map_or(NONE_SHORT.to_string(), |v| v.to_string()),
  ));

  // Codec Specific
  if let Some(c) = tk.format {
    match c {
      // PCM
      track::CodecFormat::PCM(sf) => {
        tes.push(Te(
          "Sample Rate",
          format!("{} Hz", sf.sample_rate.to_formatted_string(&Locale::en)),
        ));
        tes.push(Te(
          "Sample Size",
          format!("{} bits", sf.bits_per_sample.to_string()),
        ));
        tes.push(Te(
          "Samples",
          sf.total_samples.to_formatted_string(&Locale::en),
        ));
        tes.push(Te("Channels", sf.channels.to_formatted_string(&Locale::en)));
        tes.push(Te("Duration", format_duration(&sf.duration(), false)));
      }

      // MP3
      track::CodecFormat::MPEG3(f) => {
        tes.push(Te("Version", f.version_string()));
        tes.push(Te("Layer", f.layer_string()));
        tes.push(Te("Bitrate", format!("{} kbps", f.bitrate.to_string())));
        tes.push(Te(
          "Sample Rate",
          format!("{} KHz", f.sample_rate.to_formatted_string(&Locale::en)),
        ));
      }

      // Mpeg4
      track::CodecFormat::MPEG4(f) => {
        tes.push(Te(
          "Sample Rate",
          // format!("{} Hz", f.sample_rate().to_formatted_string(&Locale::en)),
          format!("{:08.2} Hz", f.sample_rate(),),
        ));
        tes.push(Te(
          "Sample Size",
          format!("{} bits", f.bits_per_sample.to_string()),
        ));
        tes.push(Te(
          "Samples",
          f.total_samples.to_formatted_string(&Locale::en),
        ));
        tes.push(Te("Codec", format!("{}", f.codec)));
        tes.push(Te(
          "Average Bit Rate",
          format!("{} bps", f.avg_bitrate.to_formatted_string(&Locale::en)),
        ));
        tes.push(Te(
          "Maxium Bit Rate",
          format!("{} bps", f.max_bitrate.to_formatted_string(&Locale::en)),
        ));
        tes.push(Te("Channels", f.channels.to_formatted_string(&Locale::en)));
        tes.push(Te("Channel Config", format!("{}", f.channel_config)));
        tes.push(Te("Duration", format_duration(&f.duration(), false)));
        tes.push(Te("DRM Protection", f.protected.to_string()));
        tes.push(Te(
          "DRM Type",
          f.protection_scheme
            .map_or(NONE_SHORT.to_string(), |x| format!("{}", x)),
        ));
      }
    }
  }

  // Tail of basic track
  tes.push(Te("File Size", format!("{} bytes", fs_str)));
  // Extra metadata (not the Hashes of collected metadata) we want to display.
  if let Some(track::FormatMetadata::MP4(mmd)) = &tk.metadata {
    let data_percent = if fs_int == 0 {
      "".to_string()
    } else {
      format!("- {:0.2}%", (mmd.media_size as f64 / fs_int as f64) * 100.0)
    };
    tes.push(Te(
      "Media Size:",
      format!(
        "{} bytes {}",
        mmd.media_size.to_formatted_string(&Locale::en),
        data_percent
      ),
    ));
    tes.push(Te("Creation Date:", format!("{}", mmd.creation)));
    tes.push(Te("Modification Date:", format!("{}", mmd.modification)));
  }

  // Display.
  print_te_list(tes);
  println!();

  // Display Tagged Metadata.
  if let Some(md) = &tk.metadata {
    let o = std::io::stdout();
    match md {
      track::FormatMetadata::Flac(fmd) => fmd.print(o)?,
      track::FormatMetadata::ID3(imd) => imd.print(o)?,
      track::FormatMetadata::MP4(mmd) => mmd.print(o)?,
    }
  }
  Ok(())
}

/// Prints the structure of an audio file (e.g. a list of boxtypes and sizes in order
/// found in the file and intended based on box containment).
/// Currently is only implemented for MPEG4 files.
pub fn display_structure(p: PathBuf) -> Result<(), Box<dyn Error>> {
  if p.is_file() {}
  // let p = get_file_only_path(&fname)?;
  file::display_structure(&p)?;
  Ok(())
}

// TODO(jdr): Move most of this into a function, probably in file that reads and
// uses identify to figure out which find to call.
/// Displays an MPEG4 box based on a path like /moov/udta/ilst/trkn.
/// Displays the Box information (type, size, container or not) and the data
/// assocaited with the box.
pub fn display_find_path(p: PathBuf, find_path: String) -> Result<(), Box<dyn Error>> {
  if !p.is_file() {
    return Err(Box::new(io::Error::new(
      io::ErrorKind::Other,
      format!("{} is not a file.", p.as_path().display()),
    )));
  }

  let mut file = std::fs::File::open(p).unwrap();
  if let Some(ft) = file::identify(&mut file)? {
    match ft {
      file::FileFormat::MPEG4(_) => {
        let mut vbuf = Vec::<u8>::new();
        let _n = file.read_to_end(&mut vbuf);
        let buf = vbuf.as_slice();
        if let Some(bx) = mpeg4::find::find_box(&find_path, buf) {
          println!("{:?}", bx);
          mpeg4::util::dump_buffer(bx.buf);
        } else {
          println!("Couldn't find box in path: {}", find_path);
        };
      }
      _ => println!("Can't perform find on {} files.", ft),
    }
  } else {
    println!("Can perform find on regular files",)
  }

  Ok(())
}

// UTIL

fn title_row(f: &Option<track::CodecFormat>) -> Row {
  if let Some(c) = f {
    match c {
      track::CodecFormat::PCM(_) => pcm_title_row(),
      track::CodecFormat::MPEG3(_) => mpeg3_title_row(),
      track::CodecFormat::MPEG4(_) => mpeg4_title_row(),
    }
  } else {
    pcm_title_row()
  }
}

const PCM_LIST_TITLES: [&str; 7] = [
  "Track", "Title", "Duration", "Rate", "Depth", "Format", "File",
];

fn pcm_title_row() -> Row {
  let mut r = Row::empty();
  for s in &PCM_LIST_TITLES {
    r.add_cell(Cell::new(s));
  }
  r
}

const MPEG_LIST_TITLES: [&str; 9] = [
  "Track",
  "Title",
  "Duration",
  "Sample Rate",
  "Bitrate",
  "Version",
  "Layer",
  "Format",
  "File",
];

fn mpeg3_title_row() -> Row {
  let mut r = Row::empty();
  for s in &MPEG_LIST_TITLES {
    r.add_cell(Cell::new(s));
  }
  r
}

const MPEG4_LIST_TITLES: [&str; 8] = [
  "Track",
  "Title",
  "Duration",
  "Sample Rate",
  "Depth",
  "Codec",
  "Bitrate",
  "File",
];

fn mpeg4_title_row() -> Row {
  let mut r = Row::empty();
  for s in &MPEG4_LIST_TITLES {
    r.add_cell(Cell::new(s));
  }
  r
}

fn format_duration(d: &Duration, col: bool) -> String {
  let m = d.as_secs() / 60;
  let s = d.as_secs() - 60 * m;
  if col {
    return format!("{:2}:{:02}", m, s);
  } else {
    return format!("{}:{:02}", m, s);
  }
}

fn dir_or_cwd(p: PathBuf) -> io::Result<PathBuf> {
  if p.is_dir() {
    return Ok(p);
  }
  env::current_dir()
}

// Deal with the gymnastics of getting the file
// name out of the path.
fn path_file_name(p: &PathBuf) -> String {
  let pn = match p.as_path().file_name() {
    Some(f) => f,
    None => p.as_path().as_os_str(),
  };
  let mut ps = pn.to_string_lossy().into_owned();
  if p.as_path().is_dir() {
    ps += "/";
  }
  ps
}
