use crate::mpeg4::boxes;
use crate::mpeg4::boxes::{ilst, mdia, stbl};
use lt_macro::box_db;
use std::convert::TryFrom;
use std::fmt;

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

macro_rules! def_boxes {
    ($($box_name:ident, $id:expr, $cc:literal, $container:expr, $full:expr, $comment_name:literal, $comment_path:literal;) * ) => {

        #[derive(Debug)]
        pub enum BoxType {
            $($box_name(BoxSpec)), *,
            Unknown(BoxSpec),
        }

            $(
                #[doc = $comment_name]
                #[doc = "  "]
                #[doc = $comment_path]
                pub const $box_name: BoxType = BoxType::$box_name(BoxSpec{bt_id: $id, container: $container, full: $full});
            )*


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


// This is totally over eneginineered and there has to be a better way to do this.
// On the plus side, I leanred how to write a macro_rules macro and a proc-macro.
// Though, I'm sure that this could be very much better done. In particular the proc-macro
// probably wants much re-writing and it's like I can get rid of the macro_rules macro
// above entirely.
// Other things to consider;
// 1. The From<u32> lookup function may want to be rewritten to use a hash-table depending
// on wether or not some clever compiler work turned the match into a table lookup. Seems
// unlikely.
// 2. Can probably get rid fo the b"abcd" and turn it directly into "abcd" , since that's what 
// the macro does anyway. On the other hand being able to direclty use the four character codes could
// be handy?
box_db! {
    FTYP, b"ftyp",   ContainerType::NotContainer,    false,  "File Container",                       "/ftyp";
    DINF, b"dinf",   ContainerType::Container,       false,  "Data Container",                       "//moov/trak/mdia/minf/dinf";
    DREF, b"dref",   ContainerType::NotContainer,    true,   "Data Reference - sources of media",    "/moov/trak/mdia/minf/dref";
    HDLR, b"hdlr",   ContainerType::NotContainer,    true,   "Handler - general data handler",       "/moov/trak/mdia/hdlr, /movvo,udata/meta/hdlr";
    META, b"meta",   ContainerType::Container,       true,   "Metadata Container",                   "/moov/meta, /moov/trak/meta, /moov/udata/meta";
    MINF, b"minf",   ContainerType::Container,       false,  "Media Information Container",          "/moov/meta, /moov/trak/meta, /moov/udata/meta";
    MDHD, b"mdhd",   ContainerType::NotContainer,    true,   "Media Data Header",                    "/moov/trak/mdia/mdhd";
    MDIA, b"mdia",   ContainerType::Container,       false,  "Media Container",                      "/moov/trak/mdia";
    MOOV, b"moov",   ContainerType::Container,       false,  "Top Movie Meta Data Container",        "/moov";
    MVHD, b"mvhd",   ContainerType::NotContainer,    true,   "Movie Box Header",                     "/moov/mvhd";
    SMHD, b"smhd",   ContainerType::NotContainer,    true,   "Sound Media Header",                   "/moov/trak/minf/smhd";
    STBL, b"stbl",   ContainerType::Container,       false,  "Sound Data Container",                 "/moov/trak/mdia/minf/stbl";
    TKHD, b"tkhd",   ContainerType::NotContainer,    true,   "Track Header",                         "/movv/trak/tkhd";
    TRAK, b"trak",   ContainerType::Container,       false,  "Track Container",                      "/moov/trak";
    UDTA, b"udta",   ContainerType::Container,       false,  "User Data Container",                  "/moov/udta";

    // ILST is Apples meta data block.
    ILST, b"ilst",   ContainerType::Container,       false,  "Item List - Apple metadata container", "/mnoov/udata/meta/ilst";
    DATA, b"data",   ContainerType::NotContainer,    true,   "Data box for ILST data",               "/moov/udata/meta/ilist/<ilst-md>/data";
    DISK, b"disk",   ContainerType::Container,       false,  "Disk number and total disks",          "/moov/udata/meta/ilst/disk";
    TRKN, b"trkn",   ContainerType::Container,       false,  "Track number and total tracks",        "/moov/udata/meta/ilist/trkn";
    XALB, b"\xa9alb",ContainerType::Container,       false,  "Album title",                          "/moov/udata/meta/ilst/©alb";
    XNAM, b"\xa9nam",ContainerType::Container,       false,  "Title/Name",                           "/moov/udata/meta/ilst/©nam";
}

// def_boxes! {
//     FTYP, 0x66_74_79_70, b"ftyp",   ContainerType::NotContainer,    false,  "File Container",                       "/ftyp";
// }

// def_boxes! {
// // box_db! {
//     //TYPE  VALUE       Char Code   Container                       Full    Description                             PATH
//     // Essential MOOV boxes for sound
//     // FTYP, code_to_lit(b"ftyp"), b"ftyp",   ContainerType::NotContainer,    false,  "File Container",                       "/ftyp";
//     FTYP, 0x66_74_79_70, b"ftyp",   ContainerType::NotContainer,    false,  "File Container",                       "/ftyp";
//     DINF, 0x64_69_6e_66, b"dinf",   ContainerType::Container,       false,  "Data Container",                       "//moov/trak/mdia/minf/dinf";
//     DREF, 0x64_72_65_66, b"dref",   ContainerType::NotContainer,    true,   "Data Reference - sources of media",    "/moov/trak/mdia/minf/dref";
//     HDLR, 0x68_64_6c_72, b"hdlr",   ContainerType::NotContainer,    true,   "Handler - general data handler",       "/moov/trak/mdia/hdlr, /movvo,udata/meta/hdlr";
//     META, 0x6d_65_74_61, b"meta",   ContainerType::Container,       true,   "Metadata Container",                   "/moov/meta, /moov/trak/meta, /moov/udata/meta";
//     MINF, 0x6d_69_6e_66, b"minf",   ContainerType::Container,       false,  "Media Information Container",          "/moov/meta, /moov/trak/meta, /moov/udata/meta";
//     MDHD, 0x6d_64_68_64, b"mdhd",   ContainerType::NotContainer,    true,   "Media Data Header",                    "/moov/trak/mdia/mdhd";
//     MDIA, 0x6d_64_69_61, b"mdia",   ContainerType::Container,       false,  "Media Container",                      "/moov/trak/mdia";
//     MOOV, 0x6d_6f_6f_76, b"moov",   ContainerType::Container,       false,  "Top Movie Meta Data Container",        "/moov";
//     MVHD, 0x6d_76_68_64, b"mvhd",   ContainerType::NotContainer,    true,   "Movie Box Header",                     "/moov/mvhd";
//     SMHD, 0x73_6d_68_64, b"smhd",   ContainerType::NotContainer,    true,   "Sound Media Header",                   "/moov/trak/minf/smhd";
//     STBL, 0x73_74_62_6c, b"stbl",   ContainerType::Container,       false,  "Sound Data Container",                 "/moov/trak/mdia/minf/stbl";
//     TKHD, 0x74_6b_68_64, b"tkhd",   ContainerType::NotContainer,    true,   "Track Header",                         "/movv/trak/tkhd";
//     TRAK, 0x74_72_61_6b, b"trak",   ContainerType::Container,       false,  "Track Container",                      "/moov/trak";
//     UDTA, 0x75_64_74_61, b"udta",   ContainerType::Container,       false,  "User Data Container",                  "/moov/udta";

//     // ILST is the apple meta data block
//     ILST, 0x69_6c_73_74, b"ilst",   ContainerType::Container,       false,  "Item List - Apple metadata container", "/mnoov/udata/meta/ilst";
//     DATA, 0x64_61_74_61, b"data",   ContainerType::NotContainer,    true,   "Data box for ILST data",               "/moov/udata/meta/ilist/<ilst-md>/data";
//     DISK, 0x64_69_73_6b, b"disk",   ContainerType::Container,       false,  "Disk number and total disks",          "/moov/udata/meta/ilst/disk";
//     TRKN, 0x74_72_6b_6e, b"trkn",   ContainerType::Container,       false,  "Track number and total tracks",        "/moov/udata/meta/ilist/trkn";
//     XALB, 0xa9_61_6c_62, b"\xa9alb",ContainerType::Container,       false,  "Album title",                          "/moov/udata/meta/ilst/©alb";
//     XNAM, 0xa9_6e_61_6d, b"\xa9nam",ContainerType::Container,       false,  "Title/Name",                           "/moov/udata/meta/ilst/©nam";
// }

// Containers

/*
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
