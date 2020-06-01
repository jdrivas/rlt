use crate::mpeg4::boxes;
use crate::mpeg4::boxes::{ilst, mdia, stbl};
use lt_macro::define_boxes;
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
        let fb = if self.full { "Full Box" } else { "Simple Box" };
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

        #[derive(Debug,PartialEq, Eq)]
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
// probably wants much re-writing and it's likely I can get rid of the macro_rules macro
// above entirely.
// Other things to consider;
// 1. The From<u32> lookup function may want to be rewritten to use a hash-table depending
// on wether or not some clever compiler work turned the match into a table lookup. Seems
// unlikely.
// 2. Can probably get rid fo the b"abcd" and turn it directly into "abcd" , since that's what
// the macro does anyway. On the other hand being able to direclty use the four character codes could
// be handy?

// The macros are not really that complicated in spirit.
// define_boxes, merely parses the table and adds a column between Ident and Code
// with an integer value literal by converting the 4 character code into an int by treating
// the cose as a u32 in big-endian format.
// I know, that's a lot of machineary to add a column you coud have typed.

// The table below is used to define:
//  - a BoxType enumeration one for each Ident in the table.
//  - a Constant, named by the identifer, of BoxType with the BoxSpec defined for the box type using values from the table.
//  - a From<u32> impl for BoxType, this is for doing a lookup and returing the constant enum
//    based on the integer representation of the four ascii char code defined in the Mpeg4 spec.
//  - Implementation of BoxType functions: spec() returning the BoxSpec for a BoxType and code_strin()
//    to print the ascii string from the box id integer.
//
//  An exmaple of the constant generate for FTYP is:
//     const FTYP: Boxtype = BoxType::FTYP(BoxSpec{bt_id: 0x66747970, conatiner: ContainerType::NotContainer, full: false});
//
// The table below is formed as:
//    - An identifer used for the Constant and the BoxType enumebrs: eg. FTYP (1st column).
//    - A byte string for the character code for the box: eg. b"ftyp" (2nd column).
//    - A bool determining if the box is a FullBox (has a version and flags defined for it). (3rd column).
//    - A ContainerType that describes if this is a Container, Special Conatiner (really not a pure conatiner), or NotContainer (4th column).
//    - A description which is currently used in the doc comments for the defined constants. (5th column).
//    - A path indicating where the box should normally be found in a box container hierarchy (6th column).
define_boxes! {
//  Ident Code          Container Type                 Full    Description                             Path
    FTYP, b"ftyp",      ContainerType::NotContainer,   false,  "File Container",                       "/ftyp";
    DINF, b"dinf",      ContainerType::Container,      false,  "Data Container",                       "/moov/trak/mdia/minf/dinf";
    DREF, b"dref",      ContainerType::NotContainer,   true,   "Data Reference - sources of media",    "/moov/trak/mdia/minf/dref";
    HDLR, b"hdlr",      ContainerType::NotContainer,   true,   "Handler - general data handler",       "/moov/trak/mdia/hdlr, /movvo,udata/meta/hdlr";
    META, b"meta",      ContainerType::Container,      true,   "Metadata Container",                   "/moov/meta, /moov/trak/meta, /moov/udata/meta";
    MINF, b"minf",      ContainerType::Container,      false,  "Media Information Container",          "/moov/meta, /moov/trak/meta, /moov/udata/meta";
    MDHD, b"mdhd",      ContainerType::NotContainer,   true,   "Media Data Header",                    "/moov/trak/mdia/mdhd";
    MDIA, b"mdia",      ContainerType::Container,      false,  "Media Container",                      "/moov/trak/mdia";
    MOOV, b"moov",      ContainerType::Container,      false,  "Top Movie Meta Data Container",        "/moov";
    MVHD, b"mvhd",      ContainerType::NotContainer,   true,   "Movie Box Header",                     "/moov/mvhd";
    SMHD, b"smhd",      ContainerType::NotContainer,   true,   "Sound Media Header",                   "/moov/trak/minf/smhd";
    TKHD, b"tkhd",      ContainerType::NotContainer,   true,   "Track Header",                         "/movv/trak/tkhd";
    TRAK, b"trak",      ContainerType::Container,      false,  "Track Container",                      "/moov/trak";
    UDTA, b"udta",      ContainerType::Container,      false,  "User Data Container",                  "/moov/udta";

    // Sample Table Boxes
    STBL, b"stbl",      ContainerType::Container,      false,  "Sample Table Box Container",           "/moov/trak/mdia/minf/stbl";
    ESDS, b"esds",      ContainerType::NotContainer,   true,   "Elementary Stream Descriptor",           "/moov/track/mdia/minf/stbl/stsd/mp4a/esds";
    MP4A, b"mp4a",      ContainerType::Special(28),    false,  "MPEG 4 Audio SampleEntry Box",         "/moov/track/mdia/minf/stbl/stsd/mp4a";
    STCO, b"stco",      ContainerType::NotContainer,   true,   "Chunk Offsets",                        "/moov/track/mdia/minf/stbl/stco";
    STSC, b"stsc",      ContainerType::NotContainer,   true,   "Sample to Chunk",                      "/moov/track/mdia/minf/stbl/stsc";
    STSD, b"stsd",      ContainerType::Special(4),     true,   "Sample Description",                    "/moov/track/mdia/minf/stbl/stsd";
    STTS, b"stts",      ContainerType::NotContainer,   true,   "Time to sample",                       "/movv/track/mdia/minf/stbl/stts";
    STSZ, b"stsz",      ContainerType::NotContainer,   true,   "Sample Sizes",                         "/moov/track/mdia/minf/stbl/stsz";

    // ILST is Apples meta data block.
    ILST, b"ilst",      ContainerType::Container,      false,  "Item List - Apple metadata container","/mnoov/udata/meta/ilst";
    AART, b"aart",      ContainerType::Container,      false,  "Artist",                              "/moov/udata/meta/ilst/disk";
    COVR, b"covr",      ContainerType::Container,      false,  "Cover Art",                           "/moov/udata/meta/ilst/covr";
    CPIL, b"cpil",      ContainerType::Container,      false,  "Compilation boolean",                 "/moov/udata/meta/ilst/cpil";
    DATA, b"data",      ContainerType::NotContainer,   true,   "Data box for ILST data",               "/moov/udata/meta/ilist/<ilst-md>/data";
    DISK, b"disk",      ContainerType::Container,      false,  "Disk number and total disks",          "/moov/udata/meta/ilst/disk";
    GNRE, b"gnre",      ContainerType::Container,      false,  "Genre",                                "/moov/udata/meta/ilst/gnre";
    PGAP, b"pgap",      ContainerType::Container,      false,  "Program Gap boolean",                  "/moov/udata/meta/ilst/gnre";
    TMPO, b"tmpo",      ContainerType::Container,      false,  "Tempo guide",                          "/moov/udata/meta/ilst/tmpo";
    TRKN, b"trkn",      ContainerType::Container,      false,  "Track number and total tracks",        "/moov/udata/meta/ilist/trkn";
    XALB, b"\xa9alb",   ContainerType::Container,      false,  "Album title",                          "/moov/udata/meta/ilst/©alb";
    XART, b"\xa9art",   ContainerType::Container,      false,  "Artist",                               "/moov/udata/meta/ilst/©art";
    XARTC,b"\xa9ART",   ContainerType::Container,      false,  "Artist",                               "/moov/udata/meta/ilst/©ART";
    XCMT, b"\xa9cmt",   ContainerType::Container,      false,  "Comment",                              "/moov/udata/meta/ilist/©cmt";
    XDAY, b"\xa9day",   ContainerType::Container,      false,  "Year",                                 "/moov/udata/meta/ilist/©day";
    XGEN, b"\xa9gen",   ContainerType::Container,      false,  "Genre",                                "/moov/udata/meta/ilist/©gen";
    XGRP, b"\xa9grp",   ContainerType::Container,      false,  "Group",                                "/moov/udata/meta/ilist/©grp";
    XLRY, b"\xa9lyr",   ContainerType::Container,      false,  "Lyric",                                "/moov/udata/meta/ilist/©lyr";
    XNAM, b"\xa9nam",   ContainerType::Container,      false,  "Title/Name",                           "/moov/udata/meta/ilst/©nam";
    XTOO, b"\xa9too",   ContainerType::Container,      false,  "Encoder",                              "/moov/udata/meta/ilst/©too";
    XWRT, b"\xa9wrt",   ContainerType::Container,      false,  "Writer/Author",                        "/moov/udata/meta/ilst/©wrt";
    ____, b"----",      ContainerType::Container,      false,  "Apple Special Item",                   "/moov/udata/meta/ilst/----";
}
