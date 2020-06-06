pub mod boxes;
pub mod find;
// pub mod boxes::box_types;
pub mod util;
use util::LevelStack;

use crate::file;
use crate::file::FileFormat;
use crate::track;
use boxes::box_types;
use boxes::box_types::{BoxType, ContainerType};
use boxes::{ilst, mdia, read_box_size_type, stbl, MP4Buffer};

use std::error::Error;
use std::io::{Read, Seek};

#[derive(Default, Debug)]
pub struct Mpeg4 {
    brand: String,
    version: u8,
    flags: u32,
    compatible_brands: Vec<String>,
}

/// Takes the first few bytes and indicates
/// by returning a FileFormat if this module
/// can parse the file for metadata.
pub fn identify(mut b: &[u8]) -> Option<FileFormat> {
    let mut buf: &mut &[u8] = &mut b;

    let mut br: &[u8] = &[0];
    let mut cb: Vec<&[u8]> = Vec::new();
    let mut mp4 = Mpeg4 {
        ..Default::default()
    };

    // Read the box type.
    let (_, _, bt) = read_box_size_type(buf);
    let bt: BoxType = From::from(bt);
    if bt == box_types::FTYP {
        boxes::get_ftyp_box_values(&mut buf, &mut br, &mut mp4.version, &mut mp4.flags, &mut cb);
        mp4.brand = String::from_utf8_lossy(br).to_string();
        for s in cb {
            mp4.compatible_brands
                .push(String::from_utf8_lossy(s).to_string());
        }
        Some(file::FileFormat::MPEG4(mp4))
    } else {
        None
    }
}

const FORMAT_NAME: &str = "MPEG-4";
impl file::Decoder for Mpeg4 {
    fn name(&self) -> &str {
        FORMAT_NAME
    }

    /// Fill a track assumed to be in MPEG4 format from the provider Reed + Seek.
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
        tk.format = Some(track::CodecFormat::MPEG4(track::MPEG4AudioFormat {
            ..Default::default()
        }));

        println!("File type: {:?}", self);
        read_track(buf, &mut tk);
        println!("Track: {:?}", &tk.title);
        println!();

        Ok(Some(tk))
    }
}

impl Mpeg4 {
    /// Display the structure an MP4 buffer on stdout.
    /// This prints the Box Type followed by the size
    /// a designtation as of Simple, Full, and Container
    /// as well the path to the particular box.
    /// Finally, this prints structure out using indentation
    /// to indicate conatiners.
    pub fn display_structure(&self, mut r: impl Read + Seek) -> Result<(), Box<dyn Error>> {
        let mut vbuf = Vec::<u8>::new();
        let _n = r.read_to_end(&mut vbuf)?;
        let buf = vbuf.as_slice();
        let b: &mut &[u8] = &mut &(*buf);
        let boxes = MP4Buffer { buf: b };

        // let mut l = LevelStack::new(boxes.buf.len());
        let mut l = LevelStack::new();
        let mut tabs = String::new();
        for b in boxes {
            println!(
                "{}{} [{:?}]    {:?} - Path: {:?}",
                tabs,
                b.box_type.four_cc(),
                b.size,
                b.box_type,
                l.path_string(),
            );

            // Implement indenting with the level stack.
            // Can't put the tabs.push into the update with
            // call because we can't have two separate mutable
            // references to tabs 'live' at the same time.
            if b.box_type.spec().container != ContainerType::NotContainer {
                // if let Some(spec) = b.box_type.spec() {
                //     if spec.container != ContainerType::NotContainer {
                tabs.push('\t');
                // }
            }

            // println!("Adding box: {:?} to {:?}", b, l);
            l.add_box(b);
            while l.complete() {
                tabs.pop();
                if l.len() > 1 {
                    println!("{}<{}>", tabs, l.top().unwrap().box_type.four_cc());
                }
                l.pop();
                if l.is_empty() {
                    break;
                }
            }
        }
        Ok(())
    }
}

fn read_track(buf: &[u8], mut tk: &mut track::Track) {
    let b: &mut &[u8] = &mut &(*buf);
    let boxes = MP4Buffer { buf: b };
    let mut ls = LevelStack::new();

    // Visiting each box will read the header of the box
    // and if it's a full box the version/flags.
    // Box data itself is only read by calling funtions
    // in the read_box_for_track function.
    for b in boxes {
        read_box_for_track(&mut tk, &mut ls, b);
    }
}

fn read_box_for_track<'a>(tk: &mut track::Track, path: &'a mut LevelStack, mut b: boxes::MP4Box) {
    let format = if let track::CodecFormat::MPEG4(f) = tk.format.as_mut().unwrap() {
        f
    } else {
        panic!("CodecFormat not attached to track.");
    };

    let md = if let track::FormatMetadata::MP4(f) = tk.metadata.as_mut().unwrap() {
        f
    } else {
        panic!("Metadata not attached to track.")
    };

    match b.box_type {
        box_types::DATA => {
            let db = ilst::get_data_box(&mut b);

            // This is used in the data box read as it's
            // the previous box type (direct ilist child e.g. TRK ) that determines
            // the key for the metadata.
            let bt = &path.top().unwrap().box_type;
            match db {
                // if let DataBoxContent::Text(v) = db {
                ilst::DataBoxContent::Text(v) => {
                    let val = String::from_utf8_lossy(v).to_string();

                    // Insert all the string pairs into the general metadata.
                    md.text.insert(bt.four_cc(), val.clone());

                    // Then capture the specifics based on
                    // the box previous ilst box type.
                    match *bt {
                        box_types::XALB => tk.album = Some(val),
                        box_types::XNAM => tk.title = Some(val),
                        box_types::XART | box_types::XARTC | box_types::AART => {
                            tk.artist = Some(val)
                        }
                        _ => (),
                    }
                }
                ilst::DataBoxContent::Data(v) => match path.top().unwrap().box_type {
                    box_types::TRKN => {
                        tk.track_number = Some(u16::from_be_bytes([v[2], v[3]]) as u32);
                        tk.track_total = Some(u16::from_be_bytes([v[4], v[5]]) as u32);
                    }
                    box_types::DISK => {
                        tk.disk_number = Some(u16::from_be_bytes([v[2], v[3]]) as u32);
                        tk.disk_total = Some(u16::from_be_bytes([v[4], v[5]]) as u32);
                    }
                    _ => (),
                },
                ilst::DataBoxContent::Byte(v) => {
                    // TODO(jdr): Consider adding a text translation.
                    md.byte.insert(bt.four_cc(), v); // TOOO(jdr): Do we want to change the Track Metadata to tak str().
                }
            }
        }
        box_types::STSD => {
            let mut channels: u16 = 0;
            let mut fmt: &mut [u8; 4] = &mut [0; 4];
            stbl::get_short_audio_stsd(
                &mut b,
                &mut fmt,
                &mut channels,
                &mut format.bits_per_sample,
                &mut format.sr,
            );
            format.channels = channels as u8;

            // println!("Format: {:?}", format);
            // println!("Channels: {:?}", channels);

            // TODO(jdr): This might be better obtained from somewhere else.
            // e.g. FTYP.
            tk.file_format = Some(String::from_utf8_lossy(fmt).into_owned());
        }
        box_types::MDHD => {
            let mut creation: u64 = 0;
            let mut modification: u64 = 0;
            let mut timescale: u32 = 0;
            let mut language: u16 = 0;
            mdia::get_mdhd(
                &mut b,
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
    // path.update(b);
    path.add_box(b);
    while path.complete() {
        path.pop();
        if path.is_empty() {
            break;
        }
    }
}

/*
fn get_track_reader<'a>(
    tk: &'a mut track::Track,
    bsize: usize,
    path: &'a mut LevelStack,
) -> impl FnMut(&mut boxes::MP4Box<'a>) {
    // let mut path = LevelStack::new(bsize);
    move |b: &mut boxes::MP4Box| {
        let format;
        if let track::CodecFormat::MPEG4(f) = tk.format.as_mut().unwrap() {
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

        match b.box_type {
            box_types::DATA => {
                let db = ilst::get_data_box(b);

                // This is used in the data box read as it's
                // the previous box type (direct ilist child e.g. TRK ) that determines
                // the key for the metadata.
                let bt = path.top().unwrap().box_type;
                match db {
                    // if let DataBoxContent::Text(v) = db {
                    ilst::DataBoxContent::Text(v) => {
                        let val = String::from_utf8_lossy(v).to_string();

                        // Insert all the string pairs into the general metadata.
                        md.text.insert(bt.code_string(), val.clone());

                        // Then capture the specifics based on
                        // the box previous ilst box type.
                        match bt {
                            box_types::XALB => tk.album = Some(val),
                            box_types::XNAM => tk.title = Some(val),
                            box_types::XART | box_types::XARTC | box_types::AART => tk.artist = Some(val),
                            _ => (),
                        }
                    }
                    ilst::DataBoxContent::Data(v) => match path.top().unwrap().box_type {
                        box_types::TRKN => {
                            tk.track_number = Some(u16::from_be_bytes([v[2], v[3]]) as u32);
                            tk.track_total = Some(u16::from_be_bytes([v[4], v[5]]) as u32);
                        }
                        box_types::DISK => {
                            tk.disk_number = Some(u16::from_be_bytes([v[2], v[3]]) as u32);
                            tk.disk_total = Some(u16::from_be_bytes([v[4], v[5]]) as u32);
                        }
                        _ => (),
                    },
                    ilst::DataBoxContent::Byte(v) => {
                        // TODO(jdr): Consider adding a text translation.
                        md.byte.insert(bt.code_string(), v); // TOOO(jdr): Do we want to change the Track Metadata to tak str().
                    }
                }
            }
            box_types::STSD => {
                let mut channels: u16 = 0;
                let mut fmt: &mut [u8; 4] = &mut [0; 4];
                stbl::get_short_audio_stsd(
                    b,
                    &mut fmt,
                    &mut channels,
                    &mut format.bits_per_sample,
                    &mut format.sr,
                );
                format.channels = channels as u8;

                // TODO(jdr): This might be better obtained from somewhere else.
                // e.g. FTYP.
                tk.file_format = Some(String::from_utf8_lossy(fmt).into_owned());
            }
            box_types::MDHD => {
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
        // path.update(b);
        path.add_box(b);
        path.check_and_complete();
    }
}

*/
// fn get_track_reader<'a>(
//     tk: &'a mut track::Track,
//     bsize: usize,
//     path: &'a mut LevelStack,
// ) -> impl FnMut(&mut boxes::MP4Box<'a>) {
//     // let mut path = LevelStack::new(bsize);
//     move |b: &mut boxes::MP4Box| {
//         let format;
//         if let track::CodecFormat::MPEG4(f) = tk.format.as_mut().unwrap() {
//             format = f;
//         } else {
//             panic!("CodecFormat not attached to track.");
//         }

//         let md: &mut track::MPEG4Metadata;
//         if let track::FormatMetadata::MP4(f) = tk.metadata.as_mut().unwrap() {
//             md = f;
//         } else {
//             panic!("Metadata not attached to track.")
//         }

//         match b.box_type {
//             box_types::DATA => {
//                 let db = ilst::get_data_box(b);

//                 // This is used in the data box read as it's
//                 // the previous box type (direct ilist child e.g. TRK ) that determines
//                 // the key for the metadata.
//                 let bt = path.top().unwrap().box_type;
//                 match db {
//                     // if let DataBoxContent::Text(v) = db {
//                     ilst::DataBoxContent::Text(v) => {
//                         let val = String::from_utf8_lossy(v).to_string();

//                         // Insert all the string pairs into the general metadata.
//                         md.text.insert(bt.code_string(), val.clone());

//                         // Then capture the specifics based on
//                         // the box previous ilst box type.
//                         match bt {
//                             box_types::XALB => tk.album = Some(val),
//                             box_types::XNAM => tk.title = Some(val),
//                             box_types::XART | box_types::XARTC | box_types::AART => tk.artist = Some(val),
//                             _ => (),
//                         }
//                     }
//                     ilst::DataBoxContent::Data(v) => match path.top().unwrap().box_type {
//                         box_types::TRKN => {
//                             tk.track_number = Some(u16::from_be_bytes([v[2], v[3]]) as u32);
//                             tk.track_total = Some(u16::from_be_bytes([v[4], v[5]]) as u32);
//                         }
//                         box_types::DISK => {
//                             tk.disk_number = Some(u16::from_be_bytes([v[2], v[3]]) as u32);
//                             tk.disk_total = Some(u16::from_be_bytes([v[4], v[5]]) as u32);
//                         }
//                         _ => (),
//                     },
//                     ilst::DataBoxContent::Byte(v) => {
//                         // TODO(jdr): Consider adding a text translation.
//                         md.byte.insert(bt.code_string(), v); // TOOO(jdr): Do we want to change the Track Metadata to tak str().
//                     }
//                 }
//             }
//             box_types::STSD => {
//                 let mut channels: u16 = 0;
//                 let mut fmt: &mut [u8; 4] = &mut [0; 4];
//                 stbl::get_short_audio_stsd(
//                     b,
//                     &mut fmt,
//                     &mut channels,
//                     &mut format.bits_per_sample,
//                     &mut format.sr,
//                 );
//                 format.channels = channels as u8;

//                 // TODO(jdr): This might be better obtained from somewhere else.
//                 // e.g. FTYP.
//                 tk.file_format = Some(String::from_utf8_lossy(fmt).into_owned());
//             }
//             box_types::MDHD => {
//                 let mut creation: u64 = 0;
//                 let mut modification: u64 = 0;
//                 let mut timescale: u32 = 0;
//                 let mut language: u16 = 0;
//                 mdia::get_mdhd(
//                     b,
//                     &mut creation,
//                     &mut modification,
//                     &mut timescale,
//                     &mut format.total_samples,
//                     &mut language,
//                 );
//             }
//             _ => (),
//         }
//         // Update the path with this box.
//         // path.update(b);
//         path.add_box(b);
//         path.check_and_complete();
//     }
// }

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
