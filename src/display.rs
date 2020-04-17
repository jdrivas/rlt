// extern crate chrono;
extern crate num_format;
use crate::album;
use crate::track;
use format::consts::FORMAT_CLEAN;
use num_format::{Locale, ToFormattedString};
use prettytable::{format, Cell, Row, Table};

use std::env;
use std::error::Error;
use std::io;
use std::path;
use std::time::Duration;

const NONE_SHORT: &str = "-";

/// lists files and audio files separately dispaying
/// metadata of the audio file if found.
pub fn list_files(fname: String) -> Result<(), Box<dyn Error>> {
  let mut p;
  if fname == "" {
    p = env::current_dir()?;
  } else {
    p = path::PathBuf::from(fname);
  }

  let album;
  let files;
  if !p.as_path().exists() {
    return Err(Box::new(std::io::Error::new(
      std::io::ErrorKind::NotFound,
      format!("File not found: {}", p.as_path().display()),
    )));
  }
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

  if album.tracks.len() > 0 {
    // Display album information
    println!("");
    println!("Album: {}", album.title.unwrap_or(NONE_SHORT.to_string()));
    println!("Artist: {}", album.artist.unwrap_or(NONE_SHORT.to_string()));
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
    // TODO(jdr): Checking the first rack for this seems to work, but feels wrong.
    if album.tracks[0].disk_number.is_none() {
      table.add_row(title_row(&album.tracks[0].format));
    }
    let mut ld = None;
    for t in &album.tracks {
      // Handle disc numbers and the track header wtih them.
      let cd = t.disk_number;
      if ld != cd {
        ld = cd;
        if cd.is_some() {
          table.add_row(row![format!("\nDisk: {}", cd.unwrap())]);
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
          track::CodecFormat::MPEG(f) => {
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
  if files.len() > 0 {
    println!("\nFiles:");
    for f in files {
      println!("{}", path_file_name(&f));
    }
  }

  Ok(())
}

pub fn describe_file(fname: String) -> Result<(), Box<dyn Error>> {
  // Only do a single file at a time.
  let p = path::PathBuf::from(&fname);
  if !p.as_path().is_file() {
    return Err(Box::new(io::Error::new(
      io::ErrorKind::Other,
      format!("{} is not a file.", p.as_path().display()),
    )));
  }

  let (tracks, _) = track::files_from(p)?;
  if tracks.len() > 0 {
    // this is overkill as we sould only get one file back.
    for tk in tracks {
      // Display track details

      describe_track(tk)?;
    }
  } else {
    println!("{}", fname);
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

fn describe_track(tk: track::Track) -> Result<(), Box<dyn Error>> {
  let fsize = match tk.path.as_path().metadata() {
    Ok(md) => md.len().to_formatted_string(&Locale::en),
    Err(_) => "<Unknown>".to_string(),
  };

  let mut tes = Vec::<Te>::new();

  // Basic track info.
  tes.push(Te("File", path_file_name(&tk.path)));
  tes.push(Te(
    "File Format",
    tk.file_format.unwrap_or(NONE_SHORT.to_string()),
  ));
  tes.push(Te("Artist", tk.artist.unwrap_or(NONE_SHORT.to_string())));
  tes.push(Te("Album", tk.album.unwrap_or(NONE_SHORT.to_string())));
  tes.push(Te("Title", tk.title.unwrap_or(NONE_SHORT.to_string())));
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

      // MPEG
      track::CodecFormat::MPEG(f) => {
        tes.push(Te("Version", f.version_string()));
        tes.push(Te("Layer", f.layer_string()));
        tes.push(Te("Bitrate", format!("{} kbps", f.bitrate.to_string())));
        tes.push(Te(
          "Sample Rate",
          format!("{} KHz", f.sample_rate.to_formatted_string(&Locale::en)),
        ));
      }
    }
  }

  // Tail basic track
  tes.push(Te("Size", format!("{} bytes", fsize)));
  print_te_list(tes);
  println!();

  // Display Metadata.
  if let Some(md) = tk.metadata {
    match md {
      track::FormatMetadata::Flac(fmd) => {
        println!("Metadata");
        let mut table = Table::new();
        table.set_format(*FORMAT_CLEAN);
        table.add_row(row!["Key", "Value"]);
        let mut vs: Vec<_> = fmd.comments.iter().collect();
        vs.sort();
        for (k, v) in vs {
          table.add_row(row![k, v[0]]);
          let mut i = 1;
          while i < v.len() {
            table.add_row(row!["", v[i]]);
            i = i + 1;
          }
          table.printstd();
        }
      }

      track::FormatMetadata::ID3(imd) => {
        println!("Metadata");

        println!("\nText");
        if imd.text.len() > 0 {
          let mut table = Table::new();
          table.set_format(*FORMAT_CLEAN);
          table.add_row(row!["Key", "Value"]);
          let mut vs: Vec<_> = imd.text.iter().collect();
          vs.sort();
          for (k, v) in vs {
            table.add_row(row![k, v[0]]);
            let mut i = 1;
            while i < v.len() {
              table.add_row(row!["", v[i]]);
              i = i + 1;
            }
          }
          table.printstd();
        } else {
          println!("No text.");
        }

        println!("\nComments");
        if imd.comments.len() > 0 {
          let mut table = Table::new();
          table.set_format(*FORMAT_CLEAN);
          table.add_row(row!["Key", "Lang", "Description", "Text"]);
          let mut vs: Vec<_> = imd.comments.iter().collect();
          vs.sort();
          for (k, v) in vs {
            table.add_row(row![k, v[0].0, v[0].1, v[0].2]);
            let mut i = 1;
            while i < v.len() {
              table.add_row(row!["", v[0].0, v[0].1, v[0].2]);
              i = i + 1;
            }
          }
          table.printstd();
        } else {
          println!("No Comments.")
        }
      }
    }
  }

  Ok(())
}

// UTIL

fn title_row(f: &Option<track::CodecFormat>) -> Row {
  if let Some(c) = f {
    match c {
      track::CodecFormat::PCM(_) => pcm_title_row(),
      track::CodecFormat::MPEG(_) => mpeg_title_row(),
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

fn mpeg_title_row() -> Row {
  let mut r = Row::empty();
  for s in &MPEG_LIST_TITLES {
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

fn dir_or_cwd(p: path::PathBuf) -> io::Result<path::PathBuf> {
  if p.is_dir() {
    return Ok(p);
  }
  return env::current_dir();
}

// Deal with the gymnastics of getting the file
// name out of the path.
fn path_file_name(p: &path::PathBuf) -> String {
  let pn = match p.as_path().file_name() {
    Some(f) => f,
    None => p.as_path().as_os_str(),
  };
  let mut ps = pn.to_string_lossy().into_owned();
  if p.as_path().is_dir() {
    ps += "/";
  }
  return ps;
}
