extern crate bytes;
extern crate lazy_static;
// use lazy_static;
use crate::file;
use crate::file::FileFormat;
use crate::track;
// use byteorder::{BigEndian, ReadBytesExt};
use bytes::buf::Buf;
// use mp4parse;
// use std::convert::TryFrom;
// use std::collections::HashSet;
use std::error::Error;
use std::io::{Read, Seek};
use std::str::from_utf8;
// use std::string::String;

pub struct Mp4;

const FTYP_HEADER: &[u8] = b"ftyp";
const M4A_HEADER: &[u8] = b"M4A";
// const M4B_HEADER: &[u8] = b"M4B";
// const M4P_HEADER: &[u8] = b"M4P";

pub fn identify(b: &[u8]) -> Option<FileFormat> {
    if b.len() >= 12 {
        if &b[4..8] == FTYP_HEADER {
            match &b[8..11] {
                b if b == M4A_HEADER => return Some(FileFormat::MP4A(Mp4 {})),
                // b if b == M4B_HEADER => return Some(FileFormat::MP4B),
                // b if b == M4P_HEADER => return Some(FileFormat::MP4P),
                _ => return None,
            }
        }
    }

    return None;
}

const FORMAT_NAME: &str = "mpeg-4";
impl file::Decoder for Mp4 {
    fn name(&self) -> &str {
        FORMAT_NAME
    }

    fn get_track(
        &mut self,
        mut r: impl Read + Seek,
    ) -> Result<Option<track::Track>, Box<dyn Error>> {
        // Starthere.
        let mut vbuf = Vec::<u8>::new();
        let _n = r.read_to_end(&mut vbuf)?;
        let mut buf = vbuf.as_slice();
        read_file(&mut buf)?;
        // read_boxes(&mut buf)?;

        return Ok(None);

        /*
                let mut mc = mp4parse::MediaContext::new();
                match mp4parse::read_mp4(&mut r, &mut mc) {
                    Ok(_) => (),
                    Err(e) => eprintln!("Failed to parse mp4:{:?}", e),
                }

                // println!("timescale: {:?}", mc.timescale);
                // println!("Mnovied extends Box: {:?}", mc.mvex);
                // println!("Number of ProtectionSystemSpecificHeaders: {:?}", mc.psshs);
                // println!("Tracks: {}", mc.tracks.len());
                for t in &mc.tracks {
                    //     println!("ID: {}", t.id);
                    //     println!("Type: {:?}", t.track_type);
                    //     println!("Empty Duration: {:?}", t.empty_duration);
                    //     println!("Media Time: {:?}", t.media_time);
                    // println!("Timescale: {:?}", t.timescale);
                    // println!("Duration: {:?}", t.duration);
                    //     println!("Track ID: {:?}", t.track_id);
                    //     println!("Track Header: {:?}", t.tkhd);
                    //     match &t.stsd {
                    //         Some(sd) => {
                    //             println!("Sample Description boxes: {:?}", sd.descriptions.len());
                    //             println!("Sample Entry: {:?}", sd.descriptions[0]);
                    //         }
                    //         None => println!("Sample Description boxes: None"),
                    //     }
                }

                // TODO(jdr): Fix this and decide how to handle multiple tracks.
                // Assume 1 track
                let mut tk = track::Track {
                    ..Default::default()
                };

                if mc.tracks.len() > 0 {
                    if mc.tracks.len() != 1 {
                        eprintln!("There were {} tracks, excepted 1.", mc.tracks.len());
                    }
                    if let Some(sd) = &mc.tracks[0].stsd {
                        if sd.descriptions.len() > 0 {
                            if sd.descriptions.len() != 1 {
                                eprintln!(
                                    "There were {} sample description entries, expected 1",
                                    sd.descriptions.len()
                                );
                            }
                            match &sd.descriptions[0] {
                                mp4parse::SampleEntry::Audio(ase) => {
                                    let mut duration = 0;
                                    if let Some(d) = mc.tracks[0].duration {
                                        duration = d.0;
                                    }
                                    tk.format = Some(track::CodecFormat::PCM(track::PCMFormat {
                                        sample_rate: ase.samplerate as u32,
                                        bits_per_sample: ase.samplesize,
                                        total_samples: duration,
                                        ..Default::default()
                                    }));
                                    // eprintln!("Codec Specific: {:?}", ase.codec_specific);
                                    // eprintln!(
                                    //     "Protection Info size: {} entires",
                                    //     ase.protection_info.len()
                                    // );
                                }
                                _ => eprintln!("Non-audio format."),
                            }
                        }
                    }
                }

                // let _ = read_atoms(r)?;
                return Ok(Some(tk));
        */
    }
}

fn read_file(buf: &mut impl Buf) -> Result<(), Box<dyn Error>> {
    let (brand, version, compat_brands) = read_ftyp(buf);
    let cb = compat_brands
        .iter()
        .map(|v| match from_utf8(v) {
            Ok(s) => s.to_string(),
            Err(_) => format!("{:?}", v),
        })
        .collect::<Vec<String>>();

    println!(
        "MPEG-4 file: {} - {:?}, {:?}",
        from_utf8(&brand)?,
        version,
        cb
    );
    return read_boxes(buf);
}

fn read_boxes(buf: &mut impl Buf) -> Result<(), Box<dyn Error>> {
    // Read in file type box.

    loop {
        let (size, b_type) = read_box_header(buf);
        // This is some hackery for the Apple ilist
        // box which conatins boxes with
        // boxtype:  0xA9 + <3-byte-string>.
        // I'd like to compare to str in the match.
        // so create a lossy, then grab it from the
        // Cow (copy-on-write smartpointer) that from_utf8_lossy returns.
        // with into_owned() and finally get the str out of the string.
        // I could, and have, compared against the actual [u8,4] but it makes
        // for slightly messy dsecriptions of the tokens.
        // ... and this doesn't quite work as expected in the compares.
        // println!("{}", 0xA9 as char);
        // let sb_type = String::from_utf8_lossy(&b_type);
        // match sb_type.into_owned().as_str() {
        //     "moov" => println!("moov"),
        //     "©alb" => println!("Calbum"),
        //     _ => println!("not moov"),
        // }
        // Do this just for printout.
        println!("{:?}  [{}]", String::from_utf8_lossy(&b_type), size);

        // Keep the compare as [u8;4] for now.
        // It's not that bad, and is probably faster as  it sould
        // be tricial for the compiler to generate efficcient compare
        // code for the [u8;4]s.
        let next;
        match &b_type {
            // Container boxes.
            b"moov" | b"trak" | b"udta" | b"mdia" | b"minf" | b"dinf" | b"ilst" => {
                let mut b = &buf.bytes()[0..size - 8];
                read_boxes(&mut b)?;
                println!("------  {:?}", from_utf8(&b_type)?);
                next = size - 8;
            }
            // Full Box and container.
            b"meta" => {
                let (v, f) = read_version_flag(buf);
                println!("\tversion: {:?}", v);
                println!("\tflags: {:?}", f);

                let mut b = &buf.bytes()[0..size - (8 + 4)];
                read_boxes(&mut b)?;
                println!("------  {:?}", from_utf8(&b_type)?);
                next = size - (8 + 4);
            } // full box witdth
            b"mvhd" => {
                let (v, f) = read_version_flag(buf);

                println!("\tVersion: {:?}", v);
                println!("\tFlags: {:?}", f);

                // Store seconds since begining of 1904
                let creation; // second in Jan 1, 1904.
                let modification; // second in Jan 1, 1904.
                let timescale; // units in one second.
                let duration; // length in timescale.
                if v == 1 {
                    creation = buf.get_u64();
                    modification = buf.get_u64();
                    timescale = buf.get_u32();
                    duration = buf.get_u64();
                    28
                } else {
                    creation = buf.get_u32() as u64;
                    modification = buf.get_u32() as u64;
                    timescale = buf.get_u32();
                    duration = buf.get_u32() as u64;
                    16
                };

                println!("\tCreation: {:?} [{:0x}]", creation, creation);
                println!("\tModification: {:?} [{:0x}]", modification, modification);
                println!("\tTimescale: {:?} [{:0x}]", timescale, timescale);
                println!("\tDuration: {:?} [{:0x}]", duration, duration);

                let rate = buf.get_u32(); // playback speed.
                let volume = buf.get_u16();
                buf.advance(10); // reserved.
                println!("\tRate: {} [{:0x}]", rate, rate);
                println!("\tVolume: {} [{:0x}]", volume, volume);

                let mut matrix: [u8; 36] = [0; 36]; // 4 x 9
                buf.copy_to_slice(&mut matrix);

                buf.advance(24); //  Quickitime values. (predefined 0 in standard MP4).

                let next_track_id = buf.get_u32();
                println!("\tNext Track: {}", next_track_id);

                // next = bytes_left;
                next = 0;
            }
            b"tkhd" => {
                // Flag values.
                // bit 0 = track enabled (disabled if 0)
                // bit 1 = track in movie
                // bit 2 = track in preview.
                // bit 3 = track is aspect ratio. Wdith & Height ar not pxiels, but
                // only and indicatio of the desired aspect ratio.
                let (v, f) = read_version_flag(buf);
                println!("\tVersion: {:?}", v);
                println!("\tFlags: {:?}", f);

                let creation;
                let modification;
                let track_id;
                let reserved;
                let duration;
                if v == 1 {
                    creation = buf.get_u64();
                    modification = buf.get_u64();
                    track_id = buf.get_u32();
                    reserved = buf.get_u32();
                    duration = buf.get_u64();
                } else {
                    creation = buf.get_u32() as u64;
                    modification = buf.get_u32() as u64;
                    track_id = buf.get_u32();
                    reserved = buf.get_u32();
                    duration = buf.get_u32() as u64;
                }
                println!("\tCreation: {:?} [{:0x}]", creation, creation);
                println!("\tModification: {:?} [{:0x}]", modification, modification);
                println!("\tTrack ID: {:?} [{:0x}]", track_id, track_id);
                println!("\tDuration: {:?} [{:0x}]", duration, duration);
                println!("\tReserved: {:?} [{:0x}]", reserved, reserved);

                let res2 = buf.get_u64();
                let layer = buf.get_u16();
                let alt_group = buf.get_u16();
                let volume = buf.get_u16();
                let res3 = buf.get_u16();
                let mut matrix: [u8; 36] = [0; 36];
                buf.copy_to_slice(&mut matrix);
                let width = buf.get_u32();
                let height = buf.get_u32();
                println!("\tReserved 2: {:?} [{:0x}]", res2, res2);
                println!("\tLayer: {:?} [{:0x}]", layer, layer);
                println!("\tAlt Group: {:?} [{:0x}]", alt_group, alt_group);
                println!("\tVolume {:?} [{:0x}]", volume, volume);
                println!("\tRes 3 {:?} [{:0x}]", res3, res3);
                println!("\tWidth {:?} [{:0x}]", width, width);
                println!("\tHeight {:?} [{:0x}]", height, height);

                next = 0;
            }
            b"mdhd" => {
                let mut read = 8;
                let (v, f) = read_version_flag(buf);
                read += 4;

                println!("\tversion: {}", v);
                println!("\tflags: {:#010b} = {:#03x}", f, f);

                // Store seconds since begining of 1904
                let creation; // second in Jan 1, 1904.
                let modification; // second in Jan 1, 1904.
                let timescale; // units in one second.
                let duration; // length in timescale.
                read += if v == 1 {
                    creation = buf.get_u64();
                    modification = buf.get_u64();
                    timescale = buf.get_u32();
                    duration = buf.get_u64();
                    28
                } else {
                    creation = buf.get_u32() as u64;
                    modification = buf.get_u32() as u64;
                    timescale = buf.get_u32();
                    duration = buf.get_u32() as u64;
                    16
                };
                println!("\tCreation: {:?} [{:0x}]", creation, creation);
                println!("\tModification: {:?} [{:0x}]", modification, modification);
                println!("\tTimescale: {:?} [{:0x}]", timescale, timescale);
                println!("\tDuration: {:?} [{:0x}]", duration, duration);

                // TODO(jdr): convert this to the proper representation.
                let lang = buf.get_u16(); // ISO-639-2/T language code (MSB = 0, then 3 groups of 5 bits.
                read += 2;

                let qual = buf.get_u16(); // predefined = 0; Quicktime Quality value. Normal = 0;
                read += 2;

                println!("\tLanguage code: {:?} [{:0x}]", lang, lang);
                println!("\tQuicktime Quality: {:?} [{:0x}]", qual, qual);

                println!(
                    "\tSize: {}, Read: {}, Size - Read: {}",
                    size,
                    read,
                    size - read
                );
                if size - read != 0 {
                    eprintln!("Mismatch between expected bytes read and size of box.");
                    eprintln!("Size = {}, Bytes read = {}", size, read);
                }
                next = size - read;
            }
            // FullBox with handler type and name.
            b"hdlr" => {
                let (v, f, h_type, name) = read_hdlr(buf);
                println!("\tversion: {:?}", v);
                println!("\tflags: {:?}", f);
                println!("\thandler_type: {:?}", from_utf8(&h_type)?);
                println!("\tname: {:?}", name);
                next = size - (8 + 4 + 4 + 4 + name.len() + 1);
            }
            b"smhd" => {
                let mut read = 8;
                let (v, f) = read_version_flag(buf);
                read += 4;

                println!("\tversion: {}", v);
                println!("\tflags: {:#010b} = {:#03x}", f, f);

                // 0 is center, positive is right; negative is left.
                // This is an 8.8 fixed point number.
                // Supposidly 1.0 is full right, -1.0 is full left
                // but that doesn't use the full range of 8.8.
                let balance = buf.get_i16();
                read += 2;
                let res = buf.get_u16(); // reserved = 0;
                read += 2;

                println!("\tBalance: {}", balance);
                println!("\tReserved: {}", res);

                println!(
                    "\tSize: {}, Read: {}, Size - Read: {}",
                    size,
                    read,
                    size - read
                );
                if size - read != 0 {
                    eprintln!("Mismatch between expected bytes read and size of box.");
                    eprintln!("Size = {}, Bytes read = {}", size, read);
                }

                next = size - read;
            }
            [0xa9, b'n', b'a', b'm']
            | [0xa9, b'a', b'l', b'b']
            | [0xa9, b'A', b'R', b'T']
            | [0xa9, b'c', b'm', b't']
            | [0xa9, b'd', b'a', b'y']
            | [0xa9, b'g', b'e', b'n']
            | [0xa9, b't', b'o', b'o']
            | [0xa9, b'w', b'r', b't']
            | b"covr"
            | b"cpil"
            | b"disk"
            | b"gnre"
            | b"pgap"
            | b"tmpo"
            | b"trkn" => {
                let (v, f, data) = read_data_box(buf);
                match data {
                    DataBoxContent::Data(d) => match &b_type {
                        b"disk" => {
                            let disk = u16::from_be_bytes([d[2], d[3]]);
                            let disks = u16::from_be_bytes([d[4], d[5]]);
                            println!("\tDisk: {}", disk);
                            println!("\tDisks: {}", disks);
                            println!("\tContents: {:?}", d);
                        }
                        b"trkn" => {
                            let track = u16::from_be_bytes([d[2], d[3]]);
                            let tracks = u16::from_be_bytes([d[4], d[5]]);
                            println!("\tTrack: {}", track);
                            println!("\tTracks: {}", tracks);
                            println!("\tContents: {:?}", d);
                        }
                        _ => {
                            let mut pb = d.as_slice();
                            if d.len() > 32 {
                                pb = &d[0..32];
                            }
                            println!("\tDataBox::Data");
                            println!("\tLength: {}", d.len());
                            println!("\tContent: {:?}", pb);
                        }
                    },
                    DataBoxContent::Text(s) => {
                        println!("\tDataBoxContent::Text");
                        println!("\tValue: {:?}", s);
                    }
                    DataBoxContent::Byte(b) => {
                        println!("\tDataBoxContent::Byte");
                        eprintln!("\tValue: {:?}", b);
                    }
                }
                println!("\tversion: {}", v);
                println!("\tflags: {:#010b} = {:#03x}", f, f);
                next = 0;
            }
            b"----" => {
                let (mean, name, data) = read_apple_info_box(buf);
                println!("\tApple Additional Info Box");
                println!("\tMean Value: {:?}", mean);
                println!("\tName Value: {:?}", name);
                println!("\tValue: {:?}", data);
                next = 0;
            }
            b"data" => {
                let (v, f, data) = read_data(buf, size);
                println!("\tversion: {:?}", v);
                println!("\tflags: {:#010b} = {:024x}", f, f);
                println!("\tContents: {:?}", data);
                next = 0; //size - read;
            }
            b"free" | b"skip" => {
                println!("\t{} Byte of free space in the current box.", size);
                next = size - 8;
            }
            b"wide" => {
                println!(
                    "\t{} Bytes of free space to lengthen (widden) the file box.",
                    size
                );
                next = size - 8;
            }
            b"mdat" => {
                println!("\tMedia data. {} bytes", size);
                next = size - 8;
            }
            _ => {
                println!("\tDefault.");
                match from_utf8(&b_type) {
                    Ok(s) => println!("\tFound type: {}", s),
                    Err(_) => println!("\tFound type: {:?}", b_type),
                };
                // println!("\t * Found type: {}", from_utf8(&b_type)?);
                next = size - 8;
            }
        }

        // TODO(jdr) If size == 0, box goes to end of file.
        // println!("\tNext is: {}", next);
        // println!("\tRemaining is: {}", buf.remaining());
        // println!("\tRemaining - Next: {}", buf.remaining() - next);

        if buf.remaining() <= next {
            break;
        }
        buf.advance(next);
    }

    Ok(())
}

// TODO(jdr) - this will fail on 32 bit machines.
// because we're reading in 64 bit size values as usize
// which will wrap on a 32 bit machine.
// Read Size and box type.
// Read 4 bytes of size (big endian)
// Size is the number of bytes in the box
// including all fields and contained
// boxes.
// Read 4 bytes of box type.
// Read 4 bytes of the version and flags.
// Returns the Version, Flags, Size, and the Type of the box.
fn read_box_header(buf: &mut impl Buf) -> (usize, [u8; 4]) {
    let ss = buf.get_u32();
    let mut b_type: [u8; 4] = [0; 4];
    buf.copy_to_slice(&mut b_type);
    let size: usize;
    if ss == 1 {
        size = buf.get_u64() as usize;
    } else {
        size = ss as usize;
    }
    return (size, b_type);
}

// fn read_box_header(buf: &mut impl Buf) -> (u8, u32, usize, [u8; 4]) {
//     let (size, b_type) = read_box_size_type(buf);
//     let (v, f) = read_version_flag(buf);
//     return (v, f, size, b_type);
// }

// Reads 4 bytes
// Return the top byte as the version
// number in a u8.
// Returns the bottom three bytes as the flags in a u32.
fn read_version_flag(buf: &mut impl Buf) -> (u8, u32) {
    let mut f = buf.get_u32();
    let v = (f >> 28) as u8; // High byte is the version
    f &= 0x00FFFFFF; // bottom three bytes are the flags.
    return (v, f);
}

// TODO(jdr): check for the ftyp and return an error if not found?
/// Read an ftyp box contents.
/// returns the major brand, the minor version, and the compatible brands.
fn read_ftyp(buf: &mut impl Buf) -> ([u8; 4], u32, Vec<[u8; 4]>) {
    let (size, t) = read_box_header(buf); // t = fytp.
    let mut brand: [u8; 4] = [0; 4];
    buf.copy_to_slice(&mut brand);
    let version = buf.get_u32();

    let mut left = size - 16;

    let mut c_brands = Vec::<[u8; 4]>::new();
    while left > 0 {
        let mut cb: [u8; 4] = [0; 4];
        buf.copy_to_slice(&mut cb);
        c_brands.push(cb);
        left -= 4;
    }
    return (brand, version, c_brands);
}

fn read_hdlr(buf: &mut impl Buf) -> (u8, u32, [u8; 4], String) {
    let (v, f) = read_version_flag(buf);
    let _ = buf.get_u32(); // predefined as 0.

    let mut handler_type: [u8; 4] = [0; 4];
    buf.copy_to_slice(&mut handler_type);

    let mut name = String::new();
    let mut c;
    loop {
        c = buf.get_u8();
        if c == b'\0' {
            break;
        }
        name.push(c as char);
    }

    (v, f, handler_type, name)
}

// TODO(jdr): return the raw bytes
// if we don't get a good flag value.
#[derive(Debug)]
pub enum DataBoxContent {
    Text(String),
    Data(Vec<u8>),
    Byte(u8),
}

const IMPLICIT_FLAGS: u32 = 0;
const TEXT_FLAGS: u32 = 1;
const JPEG_FLAGS: u32 = 13;
const PNG_FLAGS: u32 = 14;
const BYTE_FLAGS: u32 = 21;

// TODDO(jdr): Check for flag-type size mismatch errors and return them.
// E.g. When the flags say BYTE_FLAGS, but size says there is more than one
// byte left to read.
// TOOD(jdr): Add support for the image types outside of just a data array.
fn read_data(buf: &mut impl Buf, size: usize) -> (u8, u32, DataBoxContent) {
    let (v, f) = read_version_flag(buf);

    let _ = buf.get_u32(); // predefined as 0.

    let value;
    match f {
        TEXT_FLAGS => {
            let mut sv = String::new();
            let mut c;
            for _ in 8..(size - 8) {
                c = buf.get_u8();
                sv.push(c as char);
            }
            value = DataBoxContent::Text(sv);
        }
        IMPLICIT_FLAGS | JPEG_FLAGS | PNG_FLAGS => {
            let mut dv = Vec::<u8>::new();
            for _ in 8..(size - 8) {
                dv.push(buf.get_u8());
            }
            value = DataBoxContent::Data(dv);
        }
        BYTE_FLAGS => {
            value = DataBoxContent::Byte(buf.get_u8());
            // 8 bytes of size + 'data' header,
            // 4 bytes v/flag, 4 bytes of reserve, 1 byte of returned data.
            buf.advance(size - 17);
        }
        _ => {
            eprintln!("Unknown 'data' Box flag value: {}", f);
            let mut dv = Vec::<u8>::new();
            for _ in 8..(size - 8) {
                dv.push(buf.get_u8());
            }
            value = DataBoxContent::Data(dv);
        }
    }
    return (v, f, value);
}

fn read_apple_info_box(buf: &mut impl Buf) -> (String, String, DataBoxContent) {
    let (s, t) = read_box_header(buf);
    let (v, f) = read_version_flag(buf);
    if &t != b"mean" {
        eprintln!(
            "Expected box type {:?}, got: {:}",
            "mean",
            from_utf8(&t).unwrap()
        );
        let (_, _, d) = read_data(buf, s);
        return ("".to_string(), "".to_string(), d);
    }

    // let (v, f) = read_version_flag(buf);
    let mut mean_val = String::new();
    for _ in 0..(s - 12) {
        mean_val.push(buf.get_u8() as char)
    }

    let (nb_s, t) = read_box_header(buf); // this read 4 + 4 = 8 bytes
    match &t {
        b"name" => {
            let (nb_v, nb_f) = read_version_flag(buf); // this read 4 bytes of v/f. set to 0/0
            let mut name_val = String::new();
            for _ in 0..(nb_s - 12) {
                name_val.push(buf.get_u8() as char);
            }
            // println!("\tName box size: {}", nb_s);
            let (v, _, d) = read_data_box(buf);
            return (mean_val, name_val, d);
            // let (nb_v, nb_f, data) = read_data(buf, name_box_s);
        }
        _ => {
            println!(
                "Unknown inner box type. Expected {:?}, got: {:?}",
                "name",
                from_utf8(&t).unwrap()
            );
            let (_, _, d) = read_data(buf, nb_s);
            return (mean_val, "".to_string(), d);
        }
    }
}

fn read_data_box(buf: &mut impl Buf) -> (u8, u32, DataBoxContent) {
    let (s, t) = read_box_header(buf);
    match &t {
        b"data" => {
            return read_data(buf, s);
        }
        _ => {
            eprintln!("ERROR: EXPECTED A DATA BLOCK: Got: {:?}", t);
            eprintln!("name: {:?}", from_utf8(&t).unwrap());
            return read_data(buf, s);
        }
    };
}
