use crate::mpeg4::boxes::{BoxType, MP4Box};
use bytes::buf::Buf;
use std::fmt;

pub const ILST: [u8; 4] = *b"ilst"; // Item LST Container - Apple Meta Data block /moov/udata/meta/ilst/[x1,x2,x3,x4]
pub const XALB: [u8; 4] = [0xa9, b'a', b'l', b'b']; // Album
pub const XART: [u8; 4] = [0xa9, b'a', b'r', b't']; // Artist
pub const XARTC: [u8; 4] = [0xa9, b'A', b'R', b'T']; // Artist
pub const XCMT: [u8; 4] = [0xa9, b'c', b'm', b't']; // Comment
pub const XDAY: [u8; 4] = [0xa9, b'd', b'a', b'y']; // Year
pub const XGEN: [u8; 4] = [0xa9, b'g', b'e', b'n']; // Genre
pub const XGRP: [u8; 4] = [0xa9, b'g', b'r', b'p']; // Group
pub const XLRY: [u8; 4] = [0xa9, b'l', b'y', b'r']; // Lyric
pub const XNAM: [u8; 4] = [0xa9, b'n', b'a', b'm']; // Title/Name
pub const XTOO: [u8; 4] = [0xa9, b't', b'o', b'o']; // Encoder
pub const XWRT: [u8; 4] = [0xa9, b'w', b'r', b't']; // Writer/Author
pub const AART: [u8; 4] = *b"aART"; // Artist
pub const COVR: [u8; 4] = *b"covr"; // Cover ARt
pub const CPIL: [u8; 4] = *b"cpil"; // Compilation boolean
pub const DISK: [u8; 4] = *b"disk"; // Disk Number and Total Disks
pub const GNRE: [u8; 4] = *b"gnre"; // Genre
pub const PGAP: [u8; 4] = *b"pgap"; // Program Gap
pub const TMPO: [u8; 4] = *b"tmpo"; // Tempo guide
pub const TRKN: [u8; 4] = *b"trkn"; // Track Number and Total Tracks.

pub const DATA: [u8; 4] = *b"data"; // DATA Full Box for the above ILST Header Containers

pub const ESDS: [u8; 4] = *b"esds"; // Included in the special box.

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
    // data box has a predfeined 0
    bx.buf.get_u32();
    if let BoxType::Full(vf) = &bx.box_type {
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
