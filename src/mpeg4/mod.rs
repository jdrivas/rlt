pub mod boxes;
pub mod util;
use util::LevelStack;

use crate::file;
use crate::file::FileFormat;
use crate::track;
use boxes::ilst;
use boxes::mdia;
use boxes::stbl;
use boxes::MP4Buffer;

use std::error::Error;
use std::io::{Read, Seek};

pub struct Mpeg4;

const FTYP_HEADER: &[u8] = b"ftyp";
const M42_HEADER: &[u8] = b"mp42";
const M4A_HEADER: &[u8] = b"M4A ";

/// Takes the first few bytes and indicates
/// by returning a FileFormat if this module
/// can parse the file for metadata.
pub fn identify(b: &[u8]) -> Option<FileFormat> {
    let mut ft = None;
    if b.len() >= 12 {
        if &b[4..8] == FTYP_HEADER {
            ft = match &b[8..12] {
                b if b == M42_HEADER => Some(FileFormat::MPEG4(Mpeg4 {})),
                b if b == M4A_HEADER => Some(FileFormat::MPEG4(Mpeg4 {})),
                // b if b == M4B_HEADER => return Some(FileFormat::MP4B),
                // b if b == M4P_HEADER => return Some(FileFormat::MP4P),
                _ => None,
            };
        }
    }

    return ft;
}

const FORMAT_NAME: &str = "MPEG-4";
impl file::Decoder for Mpeg4 {
    fn name(&self) -> &str {
        FORMAT_NAME
    }

    fn get_track(
        &mut self,
        mut r: impl Read + Seek,
    ) -> Result<Option<track::Track>, Box<dyn Error>> {
        let mut vbuf = Vec::<u8>::new();
        let _n = r.read_to_end(&mut vbuf);
        let buf = vbuf.as_slice();
        let mut tk = track::Track {
            ..Default::default()
        };
        tk.metadata = Some(track::FormatMetadata::MP4(track::MPEG4Metadata {
            ..Default::default()
        }));
        tk.format = Some(track::CodecFormat::PCM(track::PCMFormat {
            ..Default::default()
        }));

        read_track(buf, &mut tk);
        println!("Track: {:?}", &tk.title);
        display_structure(buf);
        println!("");

        Ok(Some(tk))
    }
}

/// Display the structure an MP4 buffer on stdout.
/// This prints the Box Type followed by the size
/// a designtation as of Simple, Full, and Container
/// as well the path to the particular box.
/// Finally, this prints structure out using indentation
/// to indicate conatiners.
pub fn display_structure(buf: &[u8]) {
    let b: &mut &[u8] = &mut &(*buf);
    let boxes = MP4Buffer { buf: b };

    let mut l = LevelStack::new(boxes.buf.len());
    let mut tabs = String::new();
    for b in boxes {
        println!(
            "{}{:?}  [{:?}] {:?} - Path: {:?}",
            tabs,
            String::from_utf8_lossy(&b.kind[..]),
            b.size,
            b.box_type,
            l.path_string(),
        );

        // Implement indenting with the level stack.
        // Can't put the tabs.push into the update with
        // call because we can't have two separate mutable
        // references to tabs 'live' at the same time.
        if b.box_type.is_container() {
            tabs.push('\t');
        }
        l.update_with(
            &b,
            |_, _| {},
            |ls| {
                tabs.pop();
                if ls.len() > 1 {
                    if ls.len() > 1 {
                        println!(
                            "{}<{}>",
                            tabs,
                            String::from_utf8_lossy(&ls.top().unwrap().kind)
                        );
                    }
                }
            },
        );
    }
}

fn read_track(buf: &[u8], mut tk: &mut track::Track) {
    let b: &mut &[u8] = &mut &(*buf);
    let mut boxes = MP4Buffer { buf: b };
    let mut f = get_track_reader(&mut tk, buf.len());

    // Visit each box we read the header of
    // and for relevant boxes, and only relevant boxes,
    // read the data into the track, with the
    // the provided tracker reader.
    for mut b in &mut boxes {
        b.read(&mut f);
    }
}

// The returned track reader is a function
// that will check the kind (box type) of
// the box it's passed and only read data
// from those boxes that provide relevant
// information for the track.
// This meachanism is used so that
// the fucntion can be pased to the generic
// reader provided by MP4Box and called in
// the middle of a for loop: e.g. for b in boxes {b.read(f)}.
//
// TODO(jdr): This is probably more than we need.
// You could move this whole thing up into
// read_track and bypass the funcational program noise here.
// So instead of:
//
//      let mut f = get_track_reader(&mut tk, buf.len());
//      for mut b in &mut boxes {
//          b.read(&mut f);
//      }
//
// You could:
//   read_track(buf: &[u8], mut tk: &mut track::Track) {
//      let mut path = LevelStack::new(buf.len());
//      for mut b in &mut boxes {
//          match b.kind {
//            .... // rest of the function below including the reference to tk.
//          }
//       }
//   }
fn get_track_reader<'a>(
    tk: &'a mut track::Track,
    bsize: usize,
) -> impl FnMut(&mut boxes::MP4Box<'a>) {
    let mut path = LevelStack::new(bsize);
    move |b: &mut boxes::MP4Box| {
        let format;
        if let track::CodecFormat::PCM(f) = tk.format.as_mut().unwrap() {
            format = f;
        } else {
            panic!("CodecFormat not attached to track.");
        }

        let md: &mut track::MPEG4Metadata;
        if let track::FormatMetadata::MP4(f) = tk.metadata.as_mut().unwrap() {
            md = f;
        } else {
            panic!("Metadata not attached to track.")
        }

        match b.kind {
            ilst::DATA => {
                let db = ilst::get_data_box(b);

                // This used in the data box read as it's
                // the previous box type (ilst) that determines
                // the key for the metadata.
                let k = path.top().unwrap().kind;
                match db {
                    // if let DataBoxContent::Text(v) = db {
                    ilst::DataBoxContent::Text(v) => {
                        let val = String::from_utf8_lossy(v).to_string();

                        // Insert all the string pairs into the general metadata.
                        md.text
                            .insert(String::from_utf8_lossy(&k[..]).to_string(), val.clone());

                        // Then capture the specifics based on
                        // the box previous ilst box type.
                        match k {
                            ilst::XALB => tk.album = Some(val),
                            ilst::XNAM => tk.title = Some(val),
                            ilst::XART | ilst::XARTC => tk.artist = Some(val),
                            _ => (),
                        }
                    }
                    ilst::DataBoxContent::Data(v) => match &path.top().unwrap().kind {
                        &ilst::TRKN => {
                            tk.track_number = Some(u16::from_be_bytes([v[2], v[3]]) as u32);
                            tk.track_total = Some(u16::from_be_bytes([v[4], v[5]]) as u32);
                        }
                        &ilst::DISK => {
                            tk.disk_number = Some(u16::from_be_bytes([v[2], v[3]]) as u32);
                            tk.disk_total = Some(u16::from_be_bytes([v[4], v[5]]) as u32);
                        }
                        _ => (),
                    },
                    ilst::DataBoxContent::Byte(v) => {
                        // TODO(jdr): Consider adding a text translation.
                        md.byte
                            .insert(String::from_utf8_lossy(&k[..]).to_string(), v);
                    }
                }
            }
            stbl::STSD => {
                let mut channels: u16 = 0;
                let mut fmt: &mut [u8; 4] = &mut [0; 4];
                stbl::get_short_audio_stsd(
                    b,
                    &mut fmt,
                    &mut channels,
                    &mut format.bits_per_sample,
                    &mut format.sample_rate,
                );
                format.channels = channels as u8;

                // TODO(jdr): This might be better obtained from somewhere else.
                // e.g. FTYP.
                tk.file_format = Some(String::from_utf8_lossy(fmt).into_owned());
            }
            mdia::MDHD => {
                let mut creation: u64 = 0;
                let mut modification: u64 = 0;
                let mut timescale: u32 = 0;
                let mut language: u16 = 0;
                mdia::get_mdhd(
                    b,
                    &mut creation,
                    &mut modification,
                    &mut timescale,
                    &mut format.total_samples,
                    &mut language,
                );
            }
            _ => (),
        }
        // Update the path with this box.
        path.update(b);
    }
}

// fn read_buf(buf: &[u8]) -> Result<(), Box<dyn Error>> {
//     let b: &mut &[u8] = &mut &(*buf);
//     let mut boxes = MP4Buffer { buf: b };
//     let mut bx = FtypBox {
//         brand: &[][..],
//         flags: 0,
//         version: 0,
//         compat_brands: Vec::<&[u8]>::new(),
//     };

//     let mut f = get_box_reader(&mut bx);
//     for mut b in &mut boxes {
//         b.read(&mut f);
//         println!("{:?}", b);
//     }
//     Ok(())
// }

// // fn get_box_reader<'a>(bx: &'a mut FtypBox<'a>) -> impl FnMut(&mut &'a [u8], &[u8]) {
// fn get_box_reader<'a, 'b>(bx: &'a mut FtypBox<'a>) -> &mut impl FnMut(&'a boxes::MP4Box<'b>) {
//     // move |buf: &mut &[u8], kind: &[u8]| match kind {
//     &mut move |b: &'a mut boxes::MP4Box<'b>| {
//         // let mut buf = &mut &b.buf;
//         match b.kind {
//             b"ftyp" => {
//                 get_ftyp_box_values(
//                     &mut b.buf.clone(),
//                     &mut bx.brand,
//                     &mut bx.version,
//                     &mut bx.flags,
//                     &mut bx.compat_brands,
//                 );
//             }
//             _ => (),
//         }
//     }
// }
// // fn fill(buf &mut &[u8], kind: &[u8]){
//     let b = FtypBox {
//         brand: &[][..],
//         flags: 0,
//         version: 0,
//         compat_brands: Vec:<&u[8]>::new(),
//     }
// }

// This can be turned into a hash table if we like.
// fn get_box_type<'i>(kind: &[u8], buf: &mut &'i [u8]) -> (usize, Option<BoxType<'i>>) {
//     match kind {
//         b"ftyp" => get_ftyp_box(buf),
//         _ => (0, None),
//     }
// }
