use crate::mpeg4::boxes::box_types::{BoxType, ContainerType};
use crate::mpeg4::boxes::MP4Box;
use bytes::buf::Buf;
use std::fmt;

/*
def_box!(ILST, b"ilst", ContainerType::Container, false);
// pub const ILST: [u8; 4] = *b"ilst"; // Item LST Container - Apple Meta Data block /moov/udata/meta/ilst/[x1,x2,x3,x4]
def_box!(XALB, b"xalb", ContainerType::Container, false);
// pub const XALB: [u8; 4] = [0xa9, b'a', b'l', b'b']; // Album
def_box!(XART, b"xart", ContainerType::Container, false);
// pub const XART: [u8; 4] = [0xa9, b'a', b'r', b't']; // Artist
def_box!(
    XARTC,
    &[0xa9, b'A', b'R', b'T'],
    ContainerType::Container,
    false
);
// pub const XARTC: [u8; 4] = ; // Artist
def_box!(
    XCMT,
    &[0xa9, b'c', b'm', b't'],
    ContainerType::Container,
    false
);
// pub const XCMT: [u8; 4] = [0xa9, b'c', b'm', b't']; // Comment
def_box!(
    XDAY,
    &[0xa9, b'd', b'a', b'y'],
    ContainerType::Container,
    false
);
// pub const XDAY: [u8; 4] = [0xa9, b'd', b'a', b'y']; // Year
def_box!(
    XGEN,
    &[0xa9, b'g', b'e', b'n'],
    ContainerType::Container,
    false
);
// pub const XGEN: [u8; 4] = [0xa9, b'g', b'e', b'n']; // Genre
def_box!(
    XGRP,
    &[0xa9, b'g', b'r', b'p'],
    ContainerType::Container,
    false
);
// pub const XGRP: [u8; 4] = [0xa9, b'g', b'r', b'p']; // Group
def_box!(
    XLRY,
    &[0xa9, b'l', b'y', b'r'],
    ContainerType::Container,
    false
);
// pub const XLRY: [u8; 4] = [0xa9, b'l', b'y', b'r']; // Lyric
def_box!(
    XNAM,
    &[0xa9, b'n', b'a', b'm'],
    ContainerType::Container,
    false
);
// pub const XNAM: [u8; 4] = [0xa9, b'n', b'a', b'm']; // Title/Name
def_box!(
    XTOO,
    &[0xa9, b't', b'o', b'o'],
    ContainerType::Container,
    false
);
// pub const XTOO: [u8; 4] = [0xa9, b't', b'o', b'o']; // Encoder
def_box!(
    XWRT,
    &[0xa9, b'w', b'r', b't'],
    ContainerType::Container,
    false
);
// pub const XWRT: [u8; 4] = [0xa9, b'w', b'r', b't']; // Writer/Author
def_box!(____, b"----", ContainerType::Container, false);
// pub const ____: [u8; 4] = *b"----"; // Apple special item
def_box!(AART, b"aart", ContainerType::Container, false);
// pub const AART: [u8; 4] = *b"aART"; // Artist
def_box!(COVR, b"covr", ContainerType::Container, false);
// pub const COVR: [u8; 4] = *b"covr"; // Cover ARt
def_box!(CPIL, b"cpil", ContainerType::Container, false);
// pub const CPIL: [u8; 4] = *b"cpil"; // Compilation boolean
def_box!(DISK, b"disk", ContainerType::Container, false);
// pub const DISK: [u8; 4] = *b"disk"; // Disk Number and Total Disks
def_box!(GNRE, b"gnre", ContainerType::Container, false);
// pub const GNRE: [u8; 4] = *b"gnre"; // Genre
def_box!(PGAP, b"pgap", ContainerType::Container, false);
// pub const PGAP: [u8; 4] = *b"pgap"; // Program Gap
def_box!(TMPO, b"tmpo", ContainerType::Container, false);
// pub const TMPO: [u8; 4] = *b"tmpo"; // Tempo guide
def_box!(TRKN, b"trkn", ContainerType::Container, false);
// pub const TRKN: [u8; 4] = *b"trkn"; // Track Number and Total Tracks.

def_box!(DATA, b"data", ContainerType::Container, false);
// pub const DATA: [u8; 4] = *b"data"; // DATA Full Box for the above ILST Header Containers

def_box!(ESDS, b"ESDS", ContainerType::Container, false);
// pub const ESDS: [u8; 4] = *b"esds"; // Included in the special box.
*/
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
