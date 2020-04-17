// use crate::track;
// use mp4parse;
// use std::error::Error;
// use std::io::{Read, Seek};

// pub struct Mp4;

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
