extern crate bytes;
// use lazy_static; // make sure extern crate with macro_use is defined in top/lib.rs
pub mod ilst;
pub mod mdia;
pub mod stbl;
// use crate::mpeg4::util;
use bytes::buf::Buf;
// use std::collections::HashMap;
use std::fmt;
// use std::sync::Mutex;

// box_def!(MOOV, b"moov", simple_header);

// type HeaderReader<'a> = fn(&mut &'a [u8]) -> MP4Box<'a>;

// pub const aMOOV: [u8; 4] = *b"moov";
// pub const MOOV: u32 = 0x6d_6f_6f_76;
// pub const MOOV_H_RDER: fn (buf: &mut &[u8]) -> MP4Box<'a> = read_box_descriptor;

// pub enum BT {
//     // MOOV(BoxKind<'a>),
//     MOOV,
//     // SMHD(BoxKind<'a>),
//     Unknown,
// }

// box_def!(MOVE, b"moov", read_box_header, registrar);

// This version creates constants.
// But then requires that somewhere all the constants are
// inserted into storage for lookup to answer quetsions like:
//           is_contianer()
//
// That database could be:
// lazy_static! {
//     static ref REGISTRAR: Mutex<HashMap<u32, BT>> = Mutex::new(HashMap::new());
// }
//
// Or it could be:
// static kinds: &BT ..
//
// Or could be even;
// static containers: &BT ....
//
// struct BT {
//     cc: &'static [u8; 4],
//     val: u32,
//     Option<container>: Some(Container(skip)) or None
//     full: bool,
// }

// const MOOV: BT = BT {
//     cc: b"moov",
//     val: 0x6d_6f_6f_76,
//     container: Some(Container(0)),
//     full: false,
// };

// This augments the above with a HeaderReader function that reads in the header
// instead of just specifying a constant number of bytes to read in the continaer.
// impl BT {
//     pub fn header_reader(&self) -> HeaderReader {
//         let f = match self {
//             BT::MOOV => read_box_header,
//             BT::Unknown => empty_reader,
//         };
//         f
//         // read_box_header
//     }
// }

// pub fn empty_reader<'a>(b: &mut &'a [u8]) -> MP4Box<'a> {
//     MP4Box {
//         size: 0,
//         kind: [0; 4],
//         buf: b"",
//         box_type: BoxType::Simple,
//     }
// }

// New

#[derive(Debug, PartialEq, Eq)]
enum ContainerType {
    Simple,
    Full,
    Special(u64),
    NotContainer,
}

#[derive(PartialEq, Eq)]
struct BT {
    cc: &'static [u8; 4],     // character codes
    val: u32,                 // 32bit CC equivelant
    container: ContainerType, // Some indicates this is a container, None indicates not.
    full: bool,               // Indicates this is a FullBox type.
}

impl std::convert::AsRef<u32> for BT {
    fn as_ref(&self) -> &u32 {
        &self.val
    }
}

impl Into<u32> for BT {
    fn into(self) -> u32 {
        self.val
    }
}

// impl From<u32> for BT {
//     fn from(t: u32) -> BT {
//         match BOX_TYPES.iter().find(|v| v.val == t) {
//             Some(b) => **b,
//             None => NONE,
//         }
//     }
// }

impl BT {
    fn ref_from(t: u32) -> &'static BT {
        match BOX_TYPES.iter().find(|v| v.val == t) {
            Some(b) => b,
            None => &NONE,
        }
    }
}

impl fmt::Debug for BT {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let cc = String::from_utf8_lossy(self.cc);
        let cntr = match self.container {
            ContainerType::Simple => "Container".to_string(),
            ContainerType::Full => "Container".to_string(),
            ContainerType::Special(s) => format!("Special Container [{}]", s),
            ContainerType::NotContainer => "".to_string(),
        };
        let fb = match self.full {
            true => " Full Box",
            false => " Simple Box",
        };
        write!(f, "{} [{}] {}{}", cc, self.val, fb, cntr)
    }
}

const BOX_TYPES: [&'static BT; 2] = [&NONE, &MOVE];

const MOVE: BT = BT {
    cc: b"moov",
    val: 0x6d_6f_6f_76,
    container: ContainerType::Simple, // A SimpleContainer.
    full: false,
};

const NONE: BT = BT {
    cc: b"NONE",
    val: 0,
    container: ContainerType::NotContainer,
    full: false,
};

fn test(v: u32) {
    match BT::ref_from(v) {
        &MOVE => println!("Got it"),
        _ => println!("Didn't get it"),
    }
}

// Box Kind (box type in the MPEG spec, type is a rust keyword).
// Constants for the 4 byte box designator.

// Container Boxes

/// Data Information Box Container  
/// /moov/trak/mdia/dinf
pub const DINF: [u8; 4] = *b"dinf";
/// File Type Box  
/// This occurs before any variable-length box.
pub const FTYP: [u8; 4] = *b"ftyp";
/// Meta Data Container   
/// /moov/meta & /moov/trak/meta
/// /mmov/udata/meta
pub const META: [u8; 4] = *b"meta";
/// Median Infomration Container  
/// /moov/trak/mdia/minf
pub const MINF: [u8; 4] = *b"minf";
/// Moov Container for all Metadata  
/// /moov
pub const MOOV: [u8; 4] = *b"moov";
/// Trak Container
/// /moov/trak
pub const TRAK: [u8; 4] = *b"trak";
/// User Data Container   
/// /moov/udta
pub const UDTA: [u8; 4] = *b"udta";

// Full Boxes
/// Data reference. Declares sources of media data.  
/// /moov/trak/mdia/minf/dinf
pub const DREF: [u8; 4] = *b"dref";
/// Hanlder general handler header.  
/// /moov/trak/mdia, /moov/udata/meta
pub const HDLR: [u8; 4] = *b"hdlr";
/// Movie Header  
/// /moov
pub const MVHD: [u8; 4] = *b"mvhd";
/// Track Header  
/// /moov/trak
pub const TKHD: [u8; 4] = *b"tkhd";
/// Sound Media Header  
/// /moov/trak/minf/smhd
pub const SMHD: [u8; 4] = *b"smhd";
/// URL  
pub const URL_: [u8; 4] = *b"url ";

static SIMPLE_CONTAINERS: [[u8; 4]; 29] = [
    MOOV,       // Movie Data Container /moov
    TRAK,       // Track Container /movv/trak
    mdia::MDIA, // Media Data Continaer /mdia
    MINF,       // Media Information Container /moov/trak/mdia/minf
    DINF,       // Data Information Container /moov/trac/mdia/minf/dinf
    UDTA, // User Data Container /moov/udata (in practice and followed by meta), /moov/meta/udata (in spec).
    ilst::ILST,
    ilst::XALB,
    ilst::XART,
    ilst::XARTC,
    ilst::XCMT,
    ilst::XDAY,
    ilst::XGEN,
    ilst::XGRP,
    ilst::XLRY,
    ilst::XNAM,
    ilst::XTOO,
    ilst::XWRT,
    ilst::AART,
    ilst::COVR,
    ilst::CPIL,
    ilst::DISK,
    ilst::GNRE,
    ilst::PGAP,
    ilst::TMPO,
    ilst::____,
    ilst::TRKN,
    stbl::STBL,
    stbl::MP4A,
];
static FULL_CONTAINERS: [[u8; 4]; 3] = [META, stbl::STSD, stbl::ESDS];
static FULL_BOXES: [[u8; 4]; 13] = [
    DREF,
    HDLR,
    mdia::MDHD,
    MVHD,
    TKHD,
    SMHD,
    ilst::ESDS,
    ilst::DATA,
    stbl::STCO,
    stbl::STSC,
    stbl::STTS,
    stbl::STSZ,
    URL_,
];

/// Holds the buffer and supports
/// iteration over the MP4Boxes
/// in the buffer.
pub struct MP4Buffer<'a, 'b> {
    pub buf: &'b mut &'a [u8],
}

/// Holds header information from the box
/// and the buffer for the data assocaited
/// with the box.
pub struct MP4Box<'a> {
    pub size: u32,
    // pub kind: &'a [u8],
    pub kind: [u8; 4],
    pub buf: &'a [u8],
    pub box_type: BoxType,
    // read_box_header: Option<fn(&mut &'a [u8]) -> MP4Box<'a>>,
    // pub path: Vec<&'a [u8]>,
}

/// Captures the Simple and the Full Box
/// definitions (with or without Version & Flags)
/// and captures container versus not a conatiner.
#[derive(Debug)]
pub enum BoxType {
    Simple,
    Full(VersionFlag),
    SimpleContainer,
    FullContainer(VersionFlag),
}

impl BoxType {
    pub fn is_full(&self) -> bool {
        match self {
            BoxType::Simple | BoxType::SimpleContainer => false,
            _ => true,
        }
    }

    pub fn is_container(&self) -> bool {
        match self {
            BoxType::SimpleContainer | BoxType::FullContainer(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct VersionFlag {
    pub version: u8,
    pub flag: u32,
}

// read calls the function provided and sending it this box.
impl<'a> MP4Box<'a> {
    pub fn read(&mut self, rf: &mut impl FnMut(&mut MP4Box<'a>)) {
        rf(self);
    }
}

impl<'a> std::iter::Iterator for MP4Buffer<'a, '_> {
    type Item = MP4Box<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() == 0 {
            return None;
        }
        // println!("Next: Buf len: {:#0x}", self.buf.len());
        let b = read_box_header(self.buf);
        // println!("Next end: Buff len = {:#0x}", self.buf.len());
        Some(b)
    }
}

impl fmt::Debug for MP4Box<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}  [{:?}] {:?} Buffer[{}]",
            String::from_utf8_lossy(&self.kind[..]),
            self.size,
            self.box_type,
            self.buf.len(),
        )
    }
}

// TODO(jdr): Consider replacing the Buf trait usage with
// something simpler like a macro that does:
//      let(int_bytes, rest) = split_at(std::mem::size_of::<u32>)
//      *buf =rest;
//      let int = u32::from_be_bytes(int_bytes.try_into().unwrap());
//
// Ok, not simpler exactly but perhaps with less cost than the get_u32()
// call actually resovles into.

// This does not read in the whole box and parse it, just enough
// to determine the size, and type(kind) of box along with.
// Version/Flags information if this box is identified as a
// FullBox (container or otherwise).

/// Read box descriptor reads in the size and 4 character code type, which w
/// call kind here (because type is a keyword in rust.
/// It will also assign a function
pub fn read_box_header<'i>(buf: &mut &'i [u8]) -> MP4Box<'i> {
    // Read box header: [sssstttt]
    // s = 1 byte of size; 4 total.
    // t = 1 byet of box type; 4 total.
    let mut read = 0;
    let s = buf.get_u32();
    read += 4;

    let kind = &buf[0..4];
    buf.advance(4); // for the kind we referenced above.
    read += 4;

    // println!("Read at top: {:?} [{:?}]", util::u8_to_string(kind), s);

    // Check strings for Box type: Full/Simple Container/Not-Container
    // TODO(jdr): consider a hash, or some other clever mechanism
    // to do this quickly.
    let bt = if FULL_BOXES.iter().find(|v| kind == &v[..]).is_some() {
        let vf = get_version_flags(buf);
        read += 4;
        BoxType::Full(vf)
    } else if FULL_CONTAINERS.iter().find(|v| kind == &v[..]).is_some() {
        let vf = get_version_flags(buf);
        read += 4;
        BoxType::FullContainer(vf)
    } else if SIMPLE_CONTAINERS.iter().find(|v| kind == &v[..]).is_some() {
        BoxType::SimpleContainer
    } else {
        BoxType::Simple
    };

    // TOD(jdr): This is all wrong. As is the immediately above.
    // I think I would like to assign read_header functions to boxes
    // and have those called in the next() read function.
    // I'm not yet happy with how we specifiy the constants
    // for each box type/kind and so want to revist that in the same
    // go go around. Perhaps we can specify a single integer, per
    // kind to read above the baseline header, which I would start
    // to call a descriptor - just the 32 byte size, and 4 byte type/kind.

    // Special circumstances, essentially these are container boxes
    // that are non-standard in that they have more data than a single
    // container. Which makes it difficult to read the box definitions
    // in them.
    match kind {
        b"stsd" => {
            // println!(
            //     "Reading an extra u32 on {:?} matching {:?}",
            //     util::u8_to_string(kind),
            //     util::u8_to_string(b"stsd")
            // );
            buf.get_u32();
            read += 4;
        }
        b"mp4a" => {
            // println!(
            //     "Reading more bytes on {:?} matching {:?}",
            //     util::u8_to_string(kind),
            //     util::u8_to_string(b"mp4a")
            // );
            buf.advance(28);
            read += 28;
        }
        _ => (),
    };

    // Buffer not read yet.
    let rest = &buf[0..(s as usize - read)];

    let mut b = MP4Box {
        size: s,
        kind: [0; 4],
        buf: rest,
        box_type: bt,
    };
    b.kind.copy_from_slice(kind);

    test(u32::from_be_bytes(b.kind));

    // Move this buffer point along.
    if !b.box_type.is_container() {
        buf.advance(s as usize - read);
    }

    // TODO(jdr): this is redundantly
    // stored in Box. Remove the size return.
    // println!("Read: {:?}", b);
    return b;
}

fn get_version_flags(buf: &mut &[u8]) -> VersionFlag {
    let mut vf = VersionFlag {
        flag: buf.get_u32(),
        version: 0,
    };

    vf.version = (vf.flag >> 28) as u8;
    vf.flag &= 0x00FFFFFF;

    return vf;
}

/// FTYP - file type box
///
/// Begins an MPEG-4 file and idenfities the specifications
/// to which the file complies. There are old style files that
/// do not have an FTYP box, they should be read as if they contained
/// an FTYP box with the major brand ='mp41', version = 0, and a single
/// compatible brand 'mp41'.
///
/// buf: Should be at least 8 bytes of buffer
///
/// brand: A 4 ascii character string.
///
/// version: A minor version of the brand.
///
/// compat_brands: AA vector of brands, with a 4 ascii character string in each.
// #[derive(Default)]
pub struct FtypBox<'a> {
    pub brand: &'a [u8],
    pub version: u8,
    pub flags: u32,
    pub compat_brands: Vec<&'a [u8]>,
}

// TODO(jdr): Consdier changing all of
// the argumetns but buff to options.
// The ideas is to only read the values
// that you have to. This will put tests
// for each option value in front of every read.
// As opposed to just doing the read. In the
// case where we're reading everything, that result
// will be something like:
//       testb %al, %al
//       je
// for each read we do. That's probably in the noise but ???

/// FTYP - file type box
///
/// Begins an MPEG-4 file and idenfities the specifications
/// to which the file complies. There are old style files that
/// do not have an FTYP box, they should be read as if they contained
/// an FTYP box with the major brand ='mp41', version = 0, and a single
/// compatible brand 'mp41'.
///
/// buf: Should be at least 8 bytes of buffer
///
/// brand: The place to store a 4 ascii character string.
///
/// version: A place to store a minor version of the brand.
///
/// compat_brands: A place to store a vector of brands, with a 4 ascii character string in each.
pub fn get_ftyp_box_values<'a>(
    buf: &mut &'a [u8],
    brand: &mut &'a [u8],
    version: &mut u8,
    flags: &mut u32,
    compat_brands: &mut Vec<&'a [u8]>,
) {
    // let mut read = 0;

    *brand = &buf[0..4];
    buf.advance(4);
    // read += 4;

    *flags = buf.get_u32();
    // read += 4;
    *version = (*flags >> 28) as u8;
    *flags &= 0x00FFFFFF;

    while buf.len() > 0 {
        compat_brands.push(&buf[0..4]);
        buf.advance(4);
        // read += 4;
    }
}

impl fmt::Debug for FtypBox<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut cb = Vec::new();
        for b in &self.compat_brands {
            cb.push(String::from_utf8_lossy(b));
        }
        write!(
            f,
            "version: {}, flags: {:#05x}, brand: {:?}, compatible brands: {:?}",
            self.version,
            self.flags,
            String::from_utf8_lossy(self.brand),
            cb,
        )
    }
}
