// extern crate chrono;
extern crate num_format;
use crate::album;
use crate::track;
use format::consts::FORMAT_CLEAN;
use num_format::{Locale, ToFormattedString};
use prettytable::{format, Table};

use std::env;
use std::error::Error;
use std::io;
use std::path;
use std::time::Duration;

const NONE_SHORT: &str = "-";

/// lists files and audio files separately dispaying
/// metadata of the audio file if found.
pub fn list_files(fname: String) -> Result<(), Box<dyn Error>> {
  let mut p = path::PathBuf::from(fname);

  let album;
  let files;
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
      table.add_row(row!["Track", "Title", "Duration", "Rate", "Depth", "File"]);
    }
    let mut ld = None;
    for t in album.tracks {
      // Handle disc numbers and the track header wtih them.
      let cd = t.disk_number;
      if ld != cd {
        ld = cd;
        if cd.is_some() {
          table.add_row(row![format!("\nDisk: {}", cd.unwrap())]);
          table.add_row(row!["Track", "Title", "Duration", "Rate", "Depth", "Path"]);
        }
      }
      // display track data.
      let pn = path_file_name(&t.path);
      table.add_row(row![
        t.tracks_display(),
        t.title.unwrap_or(NONE_SHORT.to_string()),
        format_duration(&t.format.duration(), true),
        format!("{} KHz", (t.format.sample_rate as f64 / 1000.0)),
        format!("{} bits", t.format.bits_per_sample.to_string()),
        pn,
      ]);
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

  let mut table; // We'll be reusing this below for formating output.
  let (tracks, _) = track::files_from(p)?;
  if tracks.len() > 0 {
    // this is overkill as we sould only get one file back.
    for tk in tracks {
      // Display track details
      let fsize = match tk.path.as_path().metadata() {
        Ok(md) => md.len().to_formatted_string(&Locale::en),
        Err(_) => "<Unknown>".to_string(),
      };

      println!("Track Details");
      struct Te<'a>(&'a str, String);
      let ts = [
        Te("File", path_file_name(&tk.path)),
        Te("Artist", tk.artist.unwrap_or(NONE_SHORT.to_string())),
        Te("Album", tk.album.unwrap_or(NONE_SHORT.to_string())),
        Te("Title", tk.title.unwrap_or(NONE_SHORT.to_string())),
        Te(
          "Track",
          tk.track_number
            .map_or(NONE_SHORT.to_string(), |v| v.to_string()),
        ),
        Te(
          "Track Total",
          tk.track_total
            .map_or(NONE_SHORT.to_string(), |v| v.to_string()),
        ),
        Te(
          "Disk Number",
          tk.disk_number
            .map_or(NONE_SHORT.to_string(), |v| v.to_string()),
        ),
        Te(
          "Disk Total",
          tk.disk_total
            .map_or(NONE_SHORT.to_string(), |v| v.to_string()),
        ),
        Te(
          "Sample Rate",
          format!(
            "{} Hz",
            tk.format.sample_rate.to_formatted_string(&Locale::en)
          ),
        ),
        Te(
          "Sample Size",
          format!("{} bits", tk.format.bits_per_sample.to_string()),
        ),
        Te(
          "Samples",
          tk.format.total_samples.to_formatted_string(&Locale::en),
        ),
        Te(
          "Channels",
          tk.format.channels.to_formatted_string(&Locale::en),
        ),
        Te("Duration", format_duration(&tk.format.duration(), false)),
        Te("Size", format!("{} bytes", fsize)),
      ];
      table = Table::new();
      table.set_format(*FORMAT_CLEAN);
      for t in &ts {
        table.add_row(row![t.0, t.1]);
      }
      table.printstd();
      println!();

      // Display Comments
      println!("Metadata");
      table = Table::new();
      table.set_format(*FORMAT_CLEAN);
      table.add_row(row!["Key", "Value"]);
      let mut vs: Vec<_> = tk.comments.iter().collect();
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
    }
  } else {
    println!("{}", fname);
  }

  Ok(())
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
  return pn.to_string_lossy().into_owned();
}
