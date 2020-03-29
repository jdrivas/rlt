// extern crate chrono;
extern crate num_format;
use crate::track;
use format::consts::FORMAT_CLEAN;
use num_format::{Locale, ToFormattedString};
use prettytable::{format, Table};

use std::env;
use std::io;
use std::path;
use std::time::Duration;

/// lists files and audio files separately dispaying
/// metadata of the audio file if found.
pub fn list_files(fname: String) -> io::Result<()> {
  let path;

  if fname.len() > 0 {
    path = path::PathBuf::from(fname);
  } else {
    path = env::current_dir()?;
  }

  let (tracks, files) = track::files_from(path)?;

  if files.len() > 0 {
    for f in files {
      println!("{}", path_file_name(&f));
    }
  }

  if tracks.len() > 0 {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_CLEAN);

    // println!("Audio Tracks:");
    println!("");
    table.add_row(row![
      "Path", "Track", "Artist", "Album", "Title", "Rate", "Depth", "Duration"
    ]);
    for t in tracks {
      let pn = path_file_name(&t.path);
      table.add_row(row![
        pn,
        format!("{} of {}", t.track_number, t.total_tracks),
        t.artist,
        t.album,
        t.title,
        t.format.sample_rate.to_formatted_string(&Locale::en),
        t.format.bits_per_sample.to_string(),
        // format_duration(&t.duration()),
        format_duration(&t.duration, true),
      ]);
    }
    table.printstd();
  }

  Ok(())
}

pub fn describe_file(fname: String) -> io::Result<()> {
  // Only do a single file at a time.
  let p = path::PathBuf::from(&fname);
  if !p.as_path().is_file() {
    return Err(io::Error::new(
      io::ErrorKind::Other,
      format!("{} is not a file.", p.as_path().display()),
    ));
  }

  let mut table; // We'll be reusing this below for formating output.
  let (tracks, _) = track::files_from(p)?;
  if tracks.len() > 0 {
    // this is overkill as we sould only get one file back.
    for tk in tracks {
      // Print track details
      let fsize;
      if let Ok(md) = tk.path.as_path().metadata() {
        fsize = md.len();
      } else {
        fsize = 0;
      }
      println!("Track Details");
      struct Te<'a>(&'a str, String);
      let ts = [
        Te("File", path_file_name(&tk.path)),
        Te("Artist", tk.artist),
        Te("Album", tk.album),
        Te("Title", tk.title),
        Te(
          "Sample Rate",
          tk.format.sample_rate.to_formatted_string(&Locale::en),
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
        Te("Duration", format_duration(&tk.duration, false)),
        Te("Size", fsize.to_formatted_string(&Locale::en)),
      ];
      table = Table::new();
      table.set_format(*FORMAT_CLEAN);
      for t in &ts {
        table.add_row(row![t.0, t.1]);
      }
      table.printstd();
      println!();

      // Print Comments
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
          i = i + 1;
          table.add_row(row!["", v[i]]);
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

// Deal with the gymnastics of getting the file
// name out of the path.
fn path_file_name(p: &path::PathBuf) -> String {
  let pn;
  match p.as_path().file_name() {
    Some(f) => pn = f,
    None => pn = p.as_path().as_os_str(),
  }
  return pn.to_string_lossy().into_owned();
}
