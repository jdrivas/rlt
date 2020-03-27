// extern crate chrono;
use prettytable::{format, Table};
use std::env;
// use std::fs;
use crate::track;
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

  // Separate into files and tracks for separate display.
  // TODO(jdr); We do this twice, once in file_from
  // and then here. Not really efficient.
  let mut files = Vec::new();
  let mut tracks = Vec::new();
  let tks = track::files_from(path)?;
  for t in tks {
    match t {
      track::File::Track(tk) => tracks.push(tk),
      track::File::Path(p) => files.push(p),
    }
  }

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
        t.format.sample_rate.to_string(),
        t.format.bits_per_sample.to_string(),
        // format_duration(&t.duration()),
        format_duration(&t.duration),
      ]);
    }
    table.printstd();
  }

  Ok(())
}

fn format_duration(d: &Duration) -> String {
  let m = d.as_secs() / 60;
  let s = d.as_secs() - 60 * m;
  return format!("{:2}:{:02}", m, s);
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
