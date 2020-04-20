// use crate::track;
// use crate::file;
use crate::file::FileFormat;
// use mp4parse;
// use std::error::Error;
// use std::io::{Read, Seek};

pub struct Mp4;

pub fn identify(b: &[u8]) -> Option<FileFormat> {
    if b.len() >= 12 {
        if &b[4..8] == b"ftyp" {
            match &b[8..11] {
                b if b == b"M4A" => return Some(FileFormat::MP4A),
                b if b == b"M4B" => return Some(FileFormat::MP4B),
                b if b == b"M4P" => return Some(FileFormat::MP4P),
                _ => return None,
            }
        }
    }

    return None;
}
// impl track::Decoder for Mp4 {
//     fn is_candidate<R: Read + Seek>(_r: R) -> Result<bool, Box<dyn Error>> {
//         return Ok(false);
//         //     let mut ctxt = mp4parse::MediaContext::new();
//         //     let mut r = r;
//         //     match mp4parse::read_mp4(&mut r, &mut ctxt) {
//         //         Ok(()) => {
//         //             println!("{:?}", ctxt);
//         //             return Ok(false);
//         //         }
//         //         Err(e) => {
//         //             eprintln!("Mp4 - {:?}", e);
//         //             eprintln!("{:?}", ctxt);
//         //             return Ok(false);
//         //         }
//         //     }
//     }

//     fn get_track<R: Read + Seek>(_r: R) -> Result<Option<track::Track>, Box<dyn Error>> {
//         let tk = track::Track {
//             ..Default::default()
//         };
//         return Ok(Some(tk));
//     }
// }
