//! Reader functionality for Apple ilst generic metadata box and it's descendents.
use crate::mpeg4::boxes::{MP4Box, FULL_BOX_HEADER_SIZE};
use bytes::buf::Buf;
use std::fmt;

// #[derive(Debug)]
pub enum DataBoxContent<'a> {
    Byte(u8),
    Text(&'a [u8]),
    Data(&'a [u8]),
}

impl fmt::Debug for DataBoxContent<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataBoxContent::Byte(v) => write!(f, "Byte({:04x})", v),
            DataBoxContent::Text(v) => write!(f, "Text({:?})", String::from_utf8_lossy(v)),
            DataBoxContent::Data(v) => {
                let l = v.len();
                if l > 32 {
                    write!(
                        f,
                        "Data({:x?} ... {:x?} len = {}",
                        &v[0..8],
                        &v[l - 8..l],
                        l
                    )
                } else {
                    write!(f, "Data({:x?})", v)
                }
            }
        }
    }
}

const IMPLICIT_FLAG: u32 = 0;
const TEXT_FLAG: u32 = 1;
const JPEG_FLAG: u32 = 13;
const PNG_FLAG: u32 = 14;
const BYTE_FLAG: u32 = 21;

// TODO(jdr): Think about getting rid of the buf.get_XX() calls.
// They modify the buffer point, which is probably not what we really
// want.
pub fn get_data_box<'a>(bx: &'a mut MP4Box) -> DataBoxContent<'a> {
    // println!("box: {:?}", bx);
    // println!("buff: {:x?}", bx.buf);

    // Read past the full box (size, type, flags/version)
    bx.buf.advance(FULL_BOX_HEADER_SIZE);

    // data box has a predfeined 0
    bx.buf.get_u32(); //
    if let Some(vf) = &bx.version_flag {
        match vf.flag {
            TEXT_FLAG => DataBoxContent::Text(&bx.buf),
            IMPLICIT_FLAG | JPEG_FLAG | PNG_FLAG => DataBoxContent::Data(&bx.buf),
            BYTE_FLAG => DataBoxContent::Byte(bx.buf.get_u8()),
            _ => DataBoxContent::Byte(b'0'), // The true cases here is an error.
        }
    } else {
        // This branch of the if is an error, so maybe we should return one?
        panic!("Read a data box that wasn't a BoxType::Full()\n{:?}", bx);
    }
}
