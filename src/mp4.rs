extern crate bytes;
extern crate lazy_static;
// use lazy_static;
use crate::file;
use crate::file::FileFormat;
use crate::track;
// use byteorder::{BigEndian, ReadBytesExt};
use bytes::buf::Buf;
use mp4parse;
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
        let mut vbuf = Vec::<u8>::new();
        let _n = r.read_to_end(&mut vbuf)?;
        let mut buf = vbuf.as_slice();
        read_boxes(&mut buf)?;

        return Ok(None);

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
    }
}

// const NMBOX: [u8;4] = [0xa, b'n',b'a', b'm' ]
fn read_boxes(buf: &mut impl Buf) -> Result<(), Box<dyn Error>> {
    // Read in file type box.

    loop {
        let (size, b_type) = read_box_header(buf);
        // This is a hack for the Apple ilist
        // box which conatins boxes with
        // boxtype:  0xA9 + <3-byte-string>.
        let sb_type = match b_type[0] {
            0xa9 => from_utf8(&(b_type[1..4]))?,
            _ => from_utf8(&b_type)?,
        };
        println!("{:?}  [{}]", sb_type, size);
        let next;
        match &b_type {
            b"ftyp" => {
                let (brand, ver, cbrands) = read_ftyp(buf, size - 8);
                println!("\t{}- {}", from_utf8(&brand)?, ver);
                for b in &cbrands {
                    println!("\t\t{}", from_utf8(b)?);
                }
                next = 0;
            }
            // Container boxes.
            b"moov" | b"trak" | b"udta" | b"mdia" | b"minf" | b"dinf" | b"ilst" => {
                let mut b = &buf.bytes()[0..size - 8];
                read_boxes(&mut b)?;
                println!("------  {:?}", from_utf8(&b_type)?);
                next = size - 8;
            }
            // Just a FullBox.
            b"meta" => {
                let v = buf.get_u32();
                println!("\tversion/flags: {:?}", v);

                let mut b = &buf.bytes()[0..size - (8 + 4)];
                read_boxes(&mut b)?;
                println!("------  {:?}", from_utf8(&b_type)?);
                next = size - (8 + 4);
            }
            // FullBox with handler type eand name.
            b"hdlr" => {
                let (vf, h_type, name) = read_hdlr(buf);
                println!("\tversion/flags: {:?}", vf);
                println!("\thandler_type: {:?}", from_utf8(&h_type)?);
                println!("\tname: {:?}", name);
                next = size - (8 + 4 + 4 + 4 + name.len() + 1);
            }
            [0xa9, b'n', b'a', b'm']
            | [0xa9, b'A', b'R', b'T']
            | [0xa9, b'a', b'l', b'b']
            | b"trkn" => {
                next = 0;
            }
            b"data" => {
                let (vf, data) = read_data(buf, size);
                println!("\tversion/flags: {:?}", vf);
                println!("\tContents: {:?}", data);
                next = 0; //size - read;
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

        // go to the next one.
        // TODO(jdr) If size == 0, box goes to end of file.
        // let next = size as usize - 8;
        println!("\tNext is: {}", next);
        println!("\tRemaining is: {}", buf.remaining());
        println!("\tRemaining - Next: {}", buf.remaining() - next);
        if buf.remaining() <= next {
            break;
        }
        buf.advance(next);
    }
    // Read atom data. Size - header(8 bytes).

    Ok(())
}

// TODO(jdr) - consider reading extend_type boxes.
// TODO(jdr) - this will fail on 32 bit machines.
// Read Size and box type.
// Read 4 bytes of size (big endian)
// Size is the number of bytes in the box
// including all fields and contained
// boxes.
// Read 4 bytes of box type.
// Returns the Size and the Type of the box.
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

// Read an ftyp box contents (assume buff points directly passed the size and type.)
// returns the major brand and the minor version.
fn read_ftyp(buf: &mut impl Buf, size: usize) -> ([u8; 4], u32, Vec<[u8; 4]>) {
    let mut brand: [u8; 4] = [0; 4];
    buf.copy_to_slice(&mut brand);
    let v = buf.get_u32();
    let mut left = size - 8;
    let mut c_brands = Vec::<[u8; 4]>::new();
    while left > 0 {
        let mut cb: [u8; 4] = [0; 4];
        buf.copy_to_slice(&mut cb);
        c_brands.push(cb);
        left -= 4;
    }
    return (brand, v, c_brands);
}

fn read_hdlr(buf: &mut impl Buf) -> (u32, [u8; 4], String) {
    let vf = buf.get_u32();
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

    (vf, handler_type, name)
}

fn read_data(buf: &mut impl Buf, size: usize) -> (u32, String) {
    let vf = buf.get_u32();
    let _ = buf.get_u32(); // predefined as 0.

    let mut value = String::new();
    let mut c;
    // println!("Reading {} bytes", size);
    if size < 128 {
        for i in 8..(size - 8) {
            c = buf.get_u8();
            // println!("Reading buf[{}] = {}", i, c);
            value.push(c as char);
        }
    }
    return (vf, value);
}
