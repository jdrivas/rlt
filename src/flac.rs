use crate::track;
use metaflac::{Block, Tag};
use std::error::Error;
use std::io::{Read, Seek};

const DISCTOTAL: &str = "DISCTOTAL";
const DISCNUMBER: &str = "DISCNUMBER";
const RELEASEDATE: &str = "RELEASE_DATE";
const VENDOR: &str = "VENDOR";
const ALT_TOTALTRACKS: &str = "TRACKTOTAL";

pub struct Flac;
impl track::Decoder for Flac {
    fn is_candidate<R: Read + Seek>(r: R) -> Result<bool, Box<dyn Error>> {
        let mut r = r;
        return Ok(Tag::is_candidate(&mut r));
    }

    fn get_track<R: Read + Seek>(r: R) -> Result<Option<track::Track>, Box<dyn Error>> {
        let mut r = r;
        match Tag::read_from(&mut r) {
            Ok(t) => {
                let mut tk = track::Track {
                    ..Default::default()
                };
                hydrate(&t, &mut tk);
                return Ok(Some(tk));
            }
            Err(e) => {
                return match e.kind {
                    metaflac::ErrorKind::InvalidInput => Ok(None),
                    metaflac::ErrorKind::Io(k) => Err(Box::new(k)),
                    _ => Err(Box::new(e)),
                };
            }
        }
    }
}

fn hydrate(t: &Tag, tk: &mut track::Track) {
    for b in t.blocks() {
        match b {
            Block::StreamInfo(si) => si_hydrate(si, &mut tk.format),
            Block::VorbisComment(vc) => vorbis_hydrate(&vc, tk),
            _ => (), // TODO(jdr) should figure out how to attach arbitrary data to a track.
        }
    }
}

fn si_hydrate(si: &metaflac::block::StreamInfo, f: &mut track::SampleFormat) {
    f.sample_rate = si.sample_rate;
    f.channels = si.num_channels;
    f.bits_per_sample = si.bits_per_sample as u16;
    f.total_samples = si.total_samples;
}

fn vorbis_hydrate(vc: &metaflac::block::VorbisComment, tk: &mut track::Track) {
    // there really must be a way to collect
    // tuples of vc.title and self.title and
    // run them in a loop to do this.
    if let Some(ts) = vc.title() {
        tk.title = Some(ts.join("/")); // TODO: Is this what we want from the vector result?
    }

    tk.track_number = vc.track();

    if let Some(a) = vc.album() {
        tk.album = Some(a.join("/"));
    }

    if let Some(a) = vc.artist() {
        tk.artist = Some(a.join("/"));
    } else {
        if let Some(a) = vc.album_artist() {
            tk.artist = Some(a.join("/"));
        }
    }

    // copy the comments in.
    // TODO: Is there a more efficient way to do this?
    for (k, v) in &vc.comments {
        tk.comments.insert(k.clone(), v.clone());
    }

    // Check for alternate
    tk.track_total = vc.total_tracks();
    if tk.track_total.is_none() {
        if let Some(tt) = tk.comments.get(ALT_TOTALTRACKS) {
            if let Ok(t_total) = tt[0].parse::<u32>() {
                tk.track_total = Some(t_total);
            }
        }
    }

    // Now fill from comments.
    if let Some(dt) = tk.comments.get(DISCTOTAL) {
        if let Ok(d_total) = dt[0].parse::<u32>() {
            tk.disk_total = Some(d_total);
        }
    }
    if let Some(dn) = tk.comments.get(DISCNUMBER) {
        if let Ok(d_num) = dn[0].parse::<u32>() {
            tk.disk_number = Some(d_num);
        }
    }
}
