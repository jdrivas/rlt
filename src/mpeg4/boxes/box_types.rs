use crate::mpeg4::boxes::{ilst, mdia, stbl};
use std::fmt;

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
pub enum ContainerType {
    Container,
    Special(u32), // Sizes can only be u32 so we can't skip more than that.
    NotContainer,
}

#[derive(PartialEq, Eq)]
pub struct BoxType {
    pub cc: &'static [u8; 4], // character codes
    // val: Option<u32>,         // 32bit CC equivelant
    pub container: ContainerType, // Indicates if it's a container and full or simple, or other
    pub full: bool,
}

impl BoxType {
    pub fn ref_from(t: u32) -> &'static BoxType {
        match BOX_TYPES.iter().find(|v| v.value() == t) {
            Some(b) => b,
            None => &BT_NONE,
        }
    }

    pub fn value(&self) -> u32 {
        u32::from_be_bytes(*self.cc)
    }

    pub fn type_str(&self) -> &'static str {
        // String::from_utf8_lossy(self.cc).as_ref()
        match std::str::from_utf8(self.cc) {
            Ok(s) => s,
            Err(_) => "error",
        }
    }
}

impl fmt::Debug for BoxType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let cntr = match self.container {
            ContainerType::Container => " Container".to_string(),
            ContainerType::Special(s) => format!(" Special Container [{}]", s),
            ContainerType::NotContainer => "".to_string(),
        };
        let fb = match self.full {
            true => "Full Box",
            false => "Simple Box",
        };
        write!(f, "{} [{}] {}{}", self.type_str(), self.value(), fb, cntr)
    }
}

// Sentinel
// TODO(jdr): If the only use is to support a non-return from ret_from, then
// We can do something else (e.g. Option).
pub const BT_NONE: BoxType = BoxType {
    cc: b"NONE",
    // val: u32::from_be_bytes(*b"NONE"),
    container: ContainerType::NotContainer,
    full: false,
};

// impl Default for BoxType {
//     fn default() -> Self {
//         BT_NONE
//     }
// }

// impl std::convert::AsRef<u32> for BoxType {
//     fn as_ref(&self) -> &u32 {
//         &self.val
//     }
// }

// impl Into<u32> for BoxType {
//     fn into(self) -> u32 {
//         self.val
//     }
// }

// impl From<u32> for BT {
//     fn from(t: u32) -> BT {
//         match BOX_TYPES.iter().find(|v| v.val == t) {
//             Some(b) => **b,
//             None => NONE,
//         }
//     }
// }

// If we're going to list all of the box types here
// we might as well do this as a lazy statck and set up
// an iniitialization function to compute the val from the
// charcacter code string which we'll sepcify the macro that generates
// the value.
// so eg.
// def_box!(MOOV, b"moov", ContainerType::Simple, false);
// def_box!(STSD, b"stsd", ContainerType::Special(4), true);
//
// // We may be able to create this in a macro.
// // Consider (or something like this:);
// // boxes![MOOV,STSD];
// // Note: We don't do:
// // boxes![
// //      MOOV, b"moov", ContainerType::Simple, false;
// //      STSD, b"stsd", ContainerType::Special(4), true;
// // ]
// // The implication is that a couple of things happen here:
// // First we create the individual boxes, because we want to use them
// // on their own.
// //  e.g.
// // def_box!(MOOV, b"moov", ContainerType::Simple, false);
// // def_box!(STSD, b"stsd", ContainerType::Special(4), true);
// //
// // Then we create the array (or something like it:
// // lazy_static!{
// // static ref BoxTypes: Mutex<[&mut BT; n]> = Mutex::new([&MOOV, &STSD]);
// // }
// //
// // But for now, we'll do this by hand.
// const BoxTypes: [&mut BT; n] = [&MOOV, &STSD];
//
// Now for proper initialization we'll want:
// lazy_static!{
//      static ref BoxDB: Mutex<BTDB> = Mutex::new(DTDB::new());
// }
// struct BTDB {
//    db: HashMap;
// };
// impl DTDB {
//   fn new() -> DTDB {
//      let mut ht: HashMap::new();
//      for bt in &BoxTypes {
//          bt.value = u32.from_be_bytes(bt.cc);
//          ht.insert(bt.val, vt);
//      };
//      BTDB {
//         db: ht;
//      }
//   }
//
//    fn ref_from(&self, t: u32) -> &'static BT {
//         match self.db.get(t) {
//              Some(bt) => bt,
//              None => BT_NONE,
//         }
//    }
//
// }

// def_box!(MOOV, b"moov", ContainerType::Simple, false);

macro_rules! def_box {
    ($id:ident , $def:expr , $cont:expr , $fl:expr) => {
        pub const $id: BoxType = BoxType {
            cc: $def,
            //  val: 0x64_61_74_61,
            container: $cont,
            full: $fl,
        };
    };
}

// Containers

/// Data Information Box Container  
/// /moov/trak/mdia/dinf
def_box!(DINF, b"dinf", ContainerType::Container, false);
// pub const DINF: [u8; 4] = *b"dinf";
/// File Type Box  
/// This occurs before any variable-length box.
def_box!(FTYP, b"ftyp", ContainerType::NotContainer, false);

/// Meta Data Container   
/// /moov/meta & /moov/trak/meta
/// /mmov/udata/meta
def_box!(META, b"meta", ContainerType::Container, true);

/// Median Infomration Container  
/// /moov/trak/mdia/minf
def_box!(MINF, b"minf", ContainerType::Container, false);

/// Moov Container for all Metadata  
/// /moov
def_box!(MOOV, b"moov", ContainerType::Container, false);

/// Trak Container
/// /moov/trak
def_box!(TRAK, b"trak", ContainerType::Container, false);
/// User Data Container   
/// /moov/udta
def_box!(UDTA, b"udta", ContainerType::Container, false);

// Full Boxes
/// Data reference. Declares sources of media data.  
/// /moov/trak/mdia/minf/dinf
def_box!(DREF, b"dref", ContainerType::NotContainer, true);

/// Hanlder general handler header.  
/// /moov/trak/mdia, /moov/udata/meta
def_box!(HDLR, b"hdlr", ContainerType::NotContainer, true);

/// Movie Header  
/// /moov
def_box!(MVHD, b"mvhd", ContainerType::NotContainer, true);

/// Track Header  
/// /moov/trak
def_box!(TKHD, b"tkhd", ContainerType::NotContainer, true);

/// Sound Media Header  
/// /moov/trak/minf/smhd
def_box!(SMHD, b"smhd", ContainerType::NotContainer, true);

/// URL  
def_box!(URL_, b"url ", ContainerType::NotContainer, true);

const BOX_TYPES: [&'static BoxType; 26] = [
    &BT_NONE,
    &MOOV,
    &DINF,
    &FTYP,
    &META,
    &MINF,
    &TRAK,
    &UDTA,
    &DREF,
    &HDLR,
    &MVHD,
    &TKHD,
    &SMHD,
    &URL_,
    &mdia::MDIA,
    &mdia::MDHD,
    &stbl::STBL,
    &stbl::STCO,
    &stbl::STCO,
    &stbl::STSC,
    &stbl::STSD,
    &stbl::MP4A,
    &stbl::ESDS,
    &stbl::STTS,
    &stbl::STSZ,
    &ilst::ILST,
];

// pub fn test(v: u32) {
//     let r = BoxType::ref_from(v);
//     match r {
//         &MOOV => println!("Got it {:?}", r),
//         _ => println!("Didn't get it"),
//     }
// }

// Box Kind (box type in the MPEG spec, type is a rust keyword).
// Constants for the 4 byte box designator.

// Container Boxes

// This module for development purposes during the move to new Box Types.
/*
pub mod base {
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
}

pub static SIMPLE_CONTAINERS: [[u8; 4]; 29] = [
    base::MOOV, // Movie Data Container /moov
    base::TRAK, // Track Container /movv/trak
    mdia::MDIA, // Media Data Continaer /mdia
    base::MINF, // Media Information Container /moov/trak/mdia/minf
    base::DINF, // Data Information Container /moov/trac/mdia/minf/dinf
    base::UDTA, // User Data Container /moov/udata (in practice and followed by meta), /moov/meta/udata (in spec).
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
pub static FULL_CONTAINERS: [[u8; 4]; 3] = [base::META, stbl::STSD, stbl::ESDS];
pub static FULL_BOXES: [[u8; 4]; 13] = [
    base::DREF,
    base::HDLR,
    mdia::MDHD,
    base::MVHD,
    base::TKHD,
    base::SMHD,
    ilst::ESDS,
    ilst::DATA,
    stbl::STCO,
    stbl::STSC,
    stbl::STTS,
    stbl::STSZ,
    base::URL_,
];
*/
