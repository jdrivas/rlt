use crate::mpeg4::boxes;
use crate::mpeg4::boxes::{ilst, mdia, stbl};
use std::convert::TryFrom;
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
    Special(usize), // Sizes can only be u32 so we can't skip more than that.
    NotContainer,
}

/// Box Specificaiton
///
/// BoxSpec identifies properties of a box and carrys the basic informaiton.
// TOD(jdr); do we really need to carry around bt_id?
#[derive(PartialEq, Eq)]
pub struct BoxSpec {
    pub bt_id: u32,               // 32bit CC equivelant
    pub container: ContainerType, // Indicates if it's a container and full or simple, or other
    pub full: bool,
}

impl BoxSpec {
    pub fn code_string(&self) -> String {
        String::from_utf8_lossy(&self.bt_id.to_be_bytes()).to_string()
    }
}

impl fmt::Debug for BoxSpec {
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
        // write!(f, "{} [{}] {}{}", self.type_str(), self.value(), fb, cntr)
        write!(
            f,
            "{:?}[{:0x?}] {}{}",
            self.code_string(),
            self.bt_id,
            fb,
            cntr
        )
    }
}

// #[derive(Debug)]
// pub enum BoxType {
//     FTYP(BoxSpec),
//     MOOV(BoxSpec),
//     TRAK(BoxSpec),
//     Unknown(BoxSpec),
// }

/// Lookup BoxTypes.
/// We look up based on hte 32bit BoxType
/// Which is really a representation of a four character code, e,g: *b"moov".
// impl From<u32> for BoxType {
//     fn from(t: u32) -> BoxType {
//         match t {
//             0x6d_6f_6f_76 => MOOV,
//             0x66_74_79_70 => FTYP,
//             0x74_72_61_6b => TRAK,
//             _ => BoxType::Unknown(BoxSpec {
//                 // bt_code: String::from_utf8_lossy(&t.to_be_bytes())
//                 //     .to_string()
//                 //     .as_str(),
//                 bt_id: t,
//                 container: ContainerType::NotContainer,
//                 full: false,
//             }), //
//         }
//     }
// }

// impl BoxType {
//     pub fn spec(&self) -> &BoxSpec {
//         match self {
//             BoxType::FTYP(s) | BoxType::MOOV(s) | BoxType::TRAK(s) | BoxType::Unknown(s) => s,
//         }
//     }
//     pub fn code_string(&self) -> String {
//         self.spec().code_string()
//     }
// }

// const FTYP: BoxType = BoxType::FTYP(BoxSpec {
//     bt_id: 0x66_74_79_70,
//     container: ContainerType::NotContainer,
//     full: false,
// });
// const MOOV: BoxType = BoxType::MOOV(BoxSpec {
//     bt_id: 0x6d_6f_6f_76,
//     container: ContainerType::Container,
//     full: false,
// });
// const TRAK: BoxType = BoxType::TRAK(BoxSpec {
//     bt_id: 0x74_72_61_6b,
//     container: ContainerType::Container,
//     full: false,
// });

macro_rules! def_boxes {
    ($($box_name:ident, $id:literal, $cc:expr, $container:expr, $full:expr, $comment_name:literal, $comment_path:literal;) * ) => {

        #[derive(Debug)]
        pub enum BoxType {
            $($box_name(BoxSpec)), *,
            Unknown(BoxSpec),
        }

        /// $comment_name
        /// $comment_path
        $(pub const $box_name: BoxType = BoxType::$box_name(BoxSpec{bt_id: $id, container: $container, full: $full}); )*


        impl BoxType {
            pub fn spec(&self) -> &BoxSpec {
                match self {
                    $(BoxType::$box_name(s) |)* BoxType::Unknown(s) => s,
                }
            }

            pub fn code_string(&self) -> String {
                self.spec().code_string()
            }
        }

        impl From<u32> for BoxType {
            fn from(t: u32) -> BoxType {
                match t {
                    $($id => $box_name,)*
                    _ => BoxType::Unknown(BoxSpec {
                        bt_id: t,
                        container: ContainerType::NotContainer,
                        full: false,
                    }),
                }
            }
        }

    };
}

def_boxes! {
    // Essential track boxes for sound
    FTYP, 0x66_74_79_70, b"ftyp", ContainerType::NotContainer,  false,  "File Container",                       "/ftyp";
    TKHD, 0x74_6b_68_64, b"tkhd", ContainerType::NotContainer,  true,   "Track Header",                          "/movv/trak/tkhd";
    TRAK, 0x74_72_61_6b, b"trak", ContainerType::Container,     false,  "Track Container",                       "/moov/trak";
    DINF, 0x64_69_6e_66, b"dinf", ContainerType::Container,     false,  "Data Container",                       "//moov/trak/mdia/minf/dinf";
    DREF, 0x64_72_65_66, b"dref", ContainerType::NotContainer,  true,   "Data Reference - sources of media",    "/moov/trak/mdia/minf/dref";
    HDLR, 0x68_64_6c_72, b"hdlr", ContainerType::NotContainer,  true,   "Handler - general data handler",       "/moov/trak/mdia/hdlr, /movvo,udata/meta/hdlr";
    META, 0x6d_65_74_61, b"meta", ContainerType::Container,     true,   "Metadata Container",                   "/moov/meta, /moov/trak/meta, /moov/udata/meta";
    MINF, 0x6d_69_6e_66, b"minf", ContainerType::Container,     false,   "Media Information Container",          "/moov/meta, /moov/trak/meta, /moov/udata/meta";
    MDHD, 0x6d_64_68_64, b"mdhd", ContainerType::NotContainer,  true,   "Media Data Header",                    "/moov/trak/mdia/mdhd";
    MDIA, 0x6d_64_69_61, b"mdia", ContainerType::Container,     false,  "Media Container",                      "/moov/trak/mdia";
    MOOV, 0x6d_6f_6f_76, b"moov", ContainerType::Container,     false,  "Top Movie Meta Data Container",        "/moov";
    MVHD, 0x6d_76_68_64, b"mvhd", ContainerType::NotContainer,  true,   "Movie Box Header",                     "/moov/mvhd";
    SMHD, 0x73_6d_68_64, b"smhd", ContainerType::NotContainer,  true,   "Sound Media Header",                    "/moov/trak/minf/smhd";
    STBL, 0x73_74_62_6c, b"stbl", ContainerType::Container,     false,      "Sound Data Container",                 "/moov/trak/mdia/minf/stbl";
    UDTA, 0x75_64_74_61, b"udta", ContainerType::Container,     false,  "User Data Container",                   "/moov/udta";

    // ILST is the apple meta data block
    ILST, 0x69_6c_73_74, b"ilst", ContainerType::Container,     false, "ILST - Apple metadata container",        "/mnoov/udata/meta/ilst";
    DISK, 0x64_69_73_6b, b"disk", ContainerType::Container,     false, "Disk number and total disks",            "/moov/udata/meta/ilst/disk";
    TRKN, 0x7_472_6b_6e, b"trkn", ContainerType::Container,     false, "Track number and total tracks",          "/moov/udata/meta/ilist/trkn";

    // pub const DISK: [u8; 4] = *b"disk"; // Disk Number and Total Disks
}

macro_rules! def_box {
    ($x:ident, $y:expr, $z:expr, $v:expr) => {};
}

// macro_rules! def_box {
//     ($id:ident , $def:expr , $cont:expr , $fl:expr) => {
//         pub const $id: BoxSpec = BoxSpec {
//             code: $def,
//             //  val: 0x64_61_74_61,
//             container: $cont,
//             full: $fl,
//         };
//     };
// }

// const MOVE: BT = BT::MOOV(BoxType {
//     cc: b"moov",
//     container: ContainerType::Container,
//     full: false,
// });

// #[derive(Debug)]
// struct TheBox<'a> {ß
//     pub size: u32,
//     pub box_type: BT,
//     pub buf: &'a [u8],
//     pub vf: Option<boxes::VersionFlag>,
// }

// pub fn test() {
//     let s = 0x108;
//     let buf: &[u8] = &[0; 8];
//     let bi = 0x6d_6f_6f_76;

//     let b = TheBox {
//         size: s,
//         box_type: BT::from(3),
//         buf: buf,
//         vf: None,
//     };

//     match b.box_type {
//         BT::MOOV(bxt) => println!("Got it: {:?}", bxt),
//         BT::Unknown(i) => println!("Unknown {:?}", i.to_be_bytes()),
//         _ => println!("Didn't get it."),
//     }
// }

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
/*
macro_rules! def_box {
    ($id:ident , $def:expr , $cont:expr , $fl:expr) => {
        pub const $id: BoxSpec = BoxSpec {
            code: $def,
            //  val: 0x64_61_74_61,
            container: $cont,
            full: $fl,
        };
    };
}

// Containers



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
*/
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
