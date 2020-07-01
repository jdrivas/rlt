//! Implementation of MPEG4 metadata reading.
extern crate chrono;
// use chrono::prelude::DateTime;
use chrono::{DateTime, Duration, NaiveDate, Utc};
// use std::time::{Duration, SystemTime, UNIX_EPOCH};
pub mod boxes;
pub mod find;
pub mod formats;
// pub mod boxes::box_types;
pub mod util;
use util::LevelStack;

use crate::file;
use crate::file::FileFormat;
use crate::track;
use boxes::box_types;
use boxes::box_types::BoxType;
use boxes::{ilst, mdia, read_box_size_type, stbl, MP4Buffer};
use formats::DRMSchemes;

use std::error::Error;
use std::io::{Read, Seek};

/// MPEG4 file model which includes data from an FTYP box.
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
    fn get_track(&mut self, r: impl Read + Seek) -> Result<Option<track::Track>, Box<dyn Error>> {
        let mut tk = track::Track {
            ..Default::default()
        };
        tk.metadata = Some(track::FormatMetadata::MP4(track::MPEG4Metadata {
            ..Default::default()
        }));
        tk.format = Some(track::CodecFormat::MPEG4(track::MPEG4AudioFormat {
            ..Default::default()
        }));

        // println!("File type: {:?}", self);
        read_track(r, &mut tk)?;
        // println!("Track: {:?}", &tk.title);
        // println!();

        Ok(Some(tk))
    }
}

impl Mpeg4 {
    // TODO(jdr): Should update to accept a Wrtier instead of defaulting to stdout.
    /// Display the structure an MP4 buffer on stdout.
    /// This prints the Box Type followed by the size
    /// a designtation as of Simple, Full, and Container
    /// as well the path to the particular box.
    /// Finally, this prints structure out using indentation
    /// to indicate conatiners.
    pub fn display_structure(&self, mut r: impl Read + Seek) -> Result<(), Box<dyn Error>> {
        let mut vbuf = Vec::<u8>::new();
        let _n = r.read_to_end(&mut vbuf)?;
        let boxes = MP4Buffer {
            buf: &mut vbuf.as_slice(),
        };

        // let boxes = MP4Buffer::read_from(r)?;

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
            if b.box_type.is_container() {
                tabs.push('\t');
            }
            l.add_box(b);
            l.check_and_complete_with(|l| {
                tabs.pop();
                if l.len() > 1 {
                    println!("{}<{}>", tabs, l.top().unwrap().box_type.four_cc());
                }
            })
        }
        Ok(())
    }
}

fn read_track(mut r: impl Read + Seek, mut tk: &mut track::Track) -> Result<(), Box<dyn Error>> {
    let mut vbuf = Vec::<u8>::new();
    let _n = r.read_to_end(&mut vbuf)?;
    let buf = vbuf.as_slice();
    let b: &mut &[u8] = &mut &(*buf);
    let boxes = MP4Buffer { buf: b };

    // Visiting each box will read the header of the box
    // and if it's a full box the version/flags.
    // Box data itself is only read by calling funtions
    // in the read_box_for_track function.
    let mut ls = LevelStack::new();
    for b in boxes {
        read_box_for_track(&mut tk, &mut ls, b);
    }
    Ok(())
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

    match &b.box_type {
        &box_types::DATA => {
            let db = ilst::get_data_box(&mut b);

            // This is used to determine where the data goes.
            // It's the previous box type that determines
            // the key for the metadata. eg. ilist/trkn/data implies that the
            // data is a track number.
            let bt = &path.top().unwrap().box_type;
            match db {
                // if let DataBoxContent::Text(v) = db {
                ilst::DataBoxContent::Text(v) => {
                    let val = String::from_utf8_lossy(v).to_string();

                    // Insert all the string pairs into the general metadata.
                    md.text.insert(
                        bt.four_cc(),
                        track::MetaEntry {
                            description: bt.spec().description.to_string(),
                            value: val.clone(),
                        },
                    );

                    // Then capture the specifics based on
                    // the box previous ilst box type.
                    match *bt {
                        box_types::XALB => tk.album = Some(val),
                        box_types::AARTC => tk.album_artist = Some(val),
                        box_types::XNAM => tk.title = Some(val),
                        box_types::XART | box_types::XARTC | box_types::AART => {
                            tk.artist = Some(val);
                        }
                        // For the sort order types
                        // give prioirty first to the actuals, above.
                        box_types::SOAL => {
                            tk.album.get_or_insert(val);
                        }
                        box_types::SOAR => {
                            tk.artist.get_or_insert(val);
                        }
                        box_types::SONM => {
                            tk.title.get_or_insert(val);
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
                    md.byte.insert(
                        bt.four_cc(),
                        track::MetaEntry {
                            description: bt.spec().description.to_string(),
                            value: v,
                        },
                    );
                }
            }
        }
        // This should appear as enclosed by an STSD.
        // However, there is usually only one entry if it's an MP4A.
        // and they should be just normal boxes.
        // DRMS is the Apple designation for its' fair play
        // protection.
        // DRMS _should_ have an enclsoing sinf box, which will have an frma box
        //  ( /moov ... /stsd/drms/sinf/frma) which will identify MPG4A as
        // the originally type.
        &box_types::MP4A | &box_types::DRMS => {
            let mut channels: u16 = 0;
            stbl::read_mp4a(
                &mut b,
                &mut channels,
                &mut format.bits_per_sample,
                &mut format.sr,
            );
            format.channels = channels as u8;
            // TODO(jdr): Change this to pull out the DRM protection
            // from the schm box.
            format.protected = !(b.box_type == box_types::MP4A);

            //     box_types::MP4A => false,
            //     box_types::DRMS => true,
            //     _ => false, // won't get here.
            // };

            // TODO(jdr): This might be better obtained from somewhere else.
            // e.g. FTYP.
            // tk.file_format = Some(String::from_utf8_lossy(fmt).into_owned());
        }
        // If this is present then we've also got a protection scheme.
        &box_types::PINF => format.protected = true,
        // This should also be present.
        // TODO(jdr): possibly we move the whole protection thing into here.
        &box_types::SCHM => {
            let mut v = 0;
            stbl::read_schm(&mut b, &mut v);
            format.protection_scheme = Some(DRMSchemes::from(v));
        }
            ,
        // This should be contained by an MP4A block, but could be comming
        // from multiple tracks.
        &box_types::ESDS => {
            let mut sampling_frequency: u32 = 0;
            stbl::read_esds(
                &mut b,
                &mut format.decoder,
                &mut format.avg_bitrate,
                &mut format.max_bitrate,
                &mut format.codec,
                &mut sampling_frequency,
                &mut format.channel_config,
            );
        }
        &box_types::MDHD => {
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

            md.modification = DateTime::<Utc>::from_utc(
                NaiveDate::from_ymd(1904, 1, 1).and_hms(0, 0, 0)
                    + Duration::seconds(modification as i64),
                Utc,
            );
            md.creation = DateTime::<Utc>::from_utc(
                NaiveDate::from_ymd(1904, 1, 1).and_hms(0, 0, 0)
                    + Duration::seconds(creation as i64),
                Utc,
            );
        }
        &box_types::MDAT => md.media_size = b.size,
        // TODO(jdr): This should require a flag to turn on the printing.
        box_types::BoxType::Unknown(s) => eprintln!("Unknown box type: {:?}", s),
        _ => (),
    }

    path.update(b);
}
