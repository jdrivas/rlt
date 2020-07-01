//! List of known Mpeg4 boxes.

use crate::mpeg4::boxes::{BOX_HEADER_SIZE, FULL_BOX_HEADER_SIZE};
use lt_macro::define_boxes;
use std::fmt;

/// Display a u32 as 4 byte character codes, taking into account
/// the displayable copyright we expect and expclicitly converting
/// to hex for anything else that's not ASCII as rust stringification defines it.
///
/// Thanks to Mozilla and mp4parse for the thinking that went in to this.
/// All mistakes and ugliness mine.
/// The Table of boxes was stolen from them (with my hackery to
/// not have to write out the integers and just specify as four chcaracter codes).
/// FourCC was borrowed from U32BE and their FourCC.
pub struct FourCC(pub u32);

impl std::fmt::Display for FourCC {
    // It's unclear to me if this actually usess storage for the bytes or not.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match std::str::from_utf8(&self.0.to_be_bytes()) {
            Ok(s) => write!(f, "{}", s),
            Err(_) => {
                // The let presumably guarantees storage is used.
                let chars = self.0.to_be_bytes();
                // This also, presumably turns each value into a 32 bits.
                if chars[0] as char == '©' {
                    write!(
                        f,
                        "{}{}{}{}",
                        chars[0] as char, chars[1] as char, chars[2] as char, chars[3] as char
                    )
                } else {
                    write!(f, "{:?}", self.0)
                }
            }
        }
    }
}

impl std::fmt::Debug for FourCC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match std::str::from_utf8(&self.0.to_be_bytes()) {
            Ok(s) => write!(f, "{:?}", s),
            Err(_) => write!(f, "{:x?}", self.0),
        }
    }
}

/// Boxes are either a pure conatiner, a special container (has data),
/// or not a conatiner.
#[derive(Debug, PartialEq, Eq)]
pub enum ContainerType {
    Container,
    Special(usize), // Sizes can only be u32 so we can't skip more than that.
    NotContainer,
}

/// A BoxSpec identifies properties of a box and carrys the basic informaiton.
// TOD(jdr); do we really need to carry around bt_id?
#[derive(PartialEq, Eq)]
pub struct BoxSpec {
    pub bt_id: u32,                // 32bit CC equivelant
    pub container: ContainerType,  // Indicates if it's a container and full or simple, or other
    pub full: bool,                // Indicates a FullBox vs. a Box.
    pub description: &'static str, // Text description of box contents, pulled from table.
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
            "{}[{:0x?}] {}{}",
            FourCC(self.bt_id),
            self.bt_id,
            fb,
            cntr
        )
    }
}

macro_rules! def_boxes {
    ($($box_name:ident, $id:expr, $cc:literal, $container:expr, $full:expr, $comment_name:literal, $comment_path:literal;) * ) => {

        /// An enumeration item for each of the known MPEG4 boxes, and a catch-all uknknown.
        #[derive(Debug,PartialEq, Eq)]
        pub enum BoxType {
            $($box_name(BoxSpec)), *,
            Unknown(BoxSpec),
        }

            $(
                #[doc = $comment_name]
                #[doc = "  "]
                #[doc = $comment_path]
                pub const $box_name: BoxType = BoxType::$box_name(BoxSpec{bt_id: $id, container: $container, full: $full, description: $comment_name});
            )*

        impl BoxType {

            /// The ```BoxSpec`` for this ```BoxType``
            pub fn spec(&self) -> &BoxSpec {
                match self {
                    $(BoxType::$box_name(s) |)* BoxType::Unknown(s) => s,
                }
            }

            /// Determines if this BoxType references a container or not.
            pub fn is_container(&self) -> bool {
                self.spec().container != ContainerType::NotContainer
            }

            // The size of the header assocaited with the box in bytes.
            /// For a simple box this is: 8 bytes = SIZE (4 bytes) + Four Character Code (4 bytes).
            /// For a full box this is 12 bytes: Simple Box (8 bytes) + Version/Flags (4 bytes).
            pub fn header_size(&self) -> usize {
                if self.spec().full {
                    FULL_BOX_HEADER_SIZE
                } else {
                    BOX_HEADER_SIZE
                }
            }

            pub fn four_cc(&self) -> String {
                FourCC(self.spec().bt_id).to_string()
                // self.spec().code_string()
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
                        description: "Unknown",
                    }),
                }
            }
        }

        impl From<&[u8]> for BoxType {
            fn from(v: &[u8]) -> BoxType {
                let mut b: [u8;4] = [0;4];
                b.copy_from_slice(v);
                From::from(u32::from_be_bytes(b))
            }
        }

        impl From<[u8;4]> for BoxType {
            fn from(v: [u8;4]) -> BoxType {
                From::from(u32::from_be_bytes(v))
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

#[allow(unused_parens)]
define_boxes! {
//  Ident Code          Container Type                 Full    Description                             Path (these are examples and not complete)
    FTYP, b"ftyp",      ContainerType::NotContainer,   false,  "File Container",                       "/ftyp";
    DINF, b"dinf",      ContainerType::Container,      false,  "Data Container",                       "/moov/trak/mdia/minf/dinf";
    DREF, b"dref",      ContainerType::NotContainer,   true,   "Data Reference - sources of media",    "/moov/trak/mdia/minf/dref";
    HDLR, b"hdlr",      ContainerType::NotContainer,   true,   "Handler - general data handler",       "/moov/trak/mdia/hdlr, /movvo,udata/meta/hdlr";
    LINF, b"linf",      ContainerType::NotContainer,   true,   "UDTA Information Block",               "/moov/meta, /moov/trak/udta/linf";
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
    UUID, b"UUID",      ContainerType::NotContainer,   false,  "UUID is the user special type",        "uuid";
    MDAT, b"mdat",      ContainerType::NotContainer,   false,  "Media Data Box",                       "/mdat";
    FREE, b"free",      ContainerType::NotContainer,   false,  "Free Space",                           "/free";

    // Sample Table Boxes
    STBL, b"stbl",      ContainerType::Container,      false,  "Sample Table Box Container",           "/moov/trak/mdia/minf/stbl";
    CERT, b"cert",      ContainerType::NotContainer,   false,  "Protection information CERT",          "moov/track/mdia/minf/stbl/stsd/mp4a/pinf/schi/cert";
    CHTB, b"chtb",      ContainerType::NotContainer,   false,  "Protection information CHTB",          "moov/track/mdia/minf/stbl/stsd/mp4a/pinf/schi/cert";
    DRMS, b"drms",      ContainerType::Special(28),    false,  "Digital Rights Management",            "/moov/track/mdia/minf/stbl/stsd/drms";
    ESDS, b"esds",      ContainerType::NotContainer,   true,   "Elementary Stream Descriptor",         "/moov/track/mdia/minf/stbl/stsd/{mp4a,drms}/esds";
    FRMA, b"frma",      ContainerType::NotContainer,   false,  "Original Format Box",                  "moov/track/mdia/minf/stbl/stsd/drms/sinf/frma";
    MP4A, b"mp4a",      ContainerType::Special(28),    false,  "MPEG 4 Audio SampleEntry Box",         "/moov/track/mdia/minf/stbl/stsd/mp4a";
    PINF, b"pinf",      ContainerType::Container,      false,  "Protection Information Box",           "/moov/track/mdia/minf/stbl/mp4a/pinf";
    RIGH, b"righ",      ContainerType::NotContainer,   false,  "Protection information Rights",        "moov/track/mdia/minf/stbl/stsd/mp4a/pinf/schi/righ";
    SBTD, b"sbtd",      ContainerType::NotContainer,   true,  "Protection Information SBTD",           "/moov/track/mdia/minf/stbl/drms/sbtd";
    SCHI, b"schi",      ContainerType::Container,      false,  "Protection Information Container",     "/moov/track/mdia/minf/stbl/drms/schi";
    SCHM, b"schm",      ContainerType:NotContainer,    true,   "Protection Sceheme Informaiton Box",   "/moov/track/mdia/minf/stbl/drms/schm";
    SIGN, b"sign",      ContainerType::NotContainer,   false,  "Protection Scheme Information Box",    "/moov/track/mdia/minf/stbl/mp4a/pinf/sign";
    SINF, b"sinf",      ContainerType::Container,      false,  "Protection Scheme Information Box",    "/moov/track/mdia/minf/stbl/drms/sinf";
    STCO, b"stco",      ContainerType::NotContainer,   true,   "Chunk Offsets",                        "/moov/track/mdia/minf/stbl/stco";
    STSC, b"stsc",      ContainerType::NotContainer,   true,   "Sample to Chunk",                      "/moov/track/mdia/minf/stbl/stsc";
    STSD, b"stsd",      ContainerType::Special(4),     true,   "Sample Description",                   "/moov/track/mdia/minf/stbl/stsd";
    STTS, b"stts",      ContainerType::NotContainer,   true,   "Time to sample",                       "/movv/track/mdia/minf/stbl/stts";
    STSZ, b"stsz",      ContainerType::NotContainer,   true,   "Sample Sizes",                         "/moov/track/mdia/minf/stbl/stsz";
    USER, b"user",      ContainerType::NotContainer,   false,   "Sample Sizes",                         "/moov/track/mdia/minf/stbl/stsz";

    // ILST is Apples meta data block.
    ILST, b"ilst",      ContainerType::Container,      false,  "Item List - Apple metadata container", "/mnoov/udata/meta/ilst";
    AART, b"aart",      ContainerType::Container,      false,  "Artist",                               "/moov/udata/meta/ilst/aart";
    AARTC, b"aART",     ContainerType::Container,      false,  "Album Artist",                         "/moov/udata/meta/ilst/aART";
    AKID, b"akID",      ContainerType::Container,      false,  "Itunes ID ?",                          "/moov/udata/meta/ilst/akID";
    APID, b"apID",      ContainerType::Container,      false,  "Apple Store Account ID",               "/moov/udata/meta/ilst/aPID";
    ATID, b"atID",      ContainerType::Container,      false,  "Apple Store Album Title ID",           "/moov/udata/meta/ilst/atID";
    CATG, b"catg",      ContainerType::Container,      false,  "Category",                             "/moov/udata/meta/ilst/catg";
    COVR, b"covr",      ContainerType::Container,      false,  "Cover Art",                            "/moov/udata/meta/ilst/covr";
    CNID, b"cnID",      ContainerType::Container,      false,  "Apple Store Catalog ID",               "/moov/udata/meta/ilst/cmID";
    CMID, b"cmID",      ContainerType::Container,      false,  "Apple Store / ItunesID?",              "/moov/udata/meta/ilst/cmID";
    CPIL, b"cpil",      ContainerType::Container,      false,  "Compilation boolean",                  "/moov/udata/meta/ilst/cpil";
    CPRT, b"cprt",      ContainerType::Container,      false,  "Copyright",                            "/moov/udata/meta/ilst/cprt";
    DATA, b"data",      ContainerType::NotContainer,   true,   "Data box for ILST data",               "/moov/udata/meta/ilist/<ilst-entry>/data";
    DESC, b"desc",      ContainerType::NotContainer,   true,   "Description",                          "/moov/udata/meta/ilist/<ilst-entry>/desc";
    DISK, b"disk",      ContainerType::Container,      false,  "Disk number and total disks",          "/moov/udata/meta/ilst/disk";
    GEID, b"geID",      ContainerType::Container,      false,  "Genre ID",                             "/moov/udata/meta/ilst/geID";
    GNRE, b"gnre",      ContainerType::Container,      false,  "Genre",                                "/moov/udata/meta/ilst/gnre";
    HDVD, b"hdvd",      ContainerType::Container,      false,  "High Definition Video",                "/moov/udata/meta/ilst/hdvd";
    KEYW, b"keyw",      ContainerType::Container,      false,  "Key Word",                             "/moov/udata/meta/ilst/keyw";
    LDES, b"ldes",      ContainerType::Container,      false,  "Long Description",                     "/moov/udata/meta/ilst/ldes";
    OWNR, b"ownr",      ContainerType::Container,      false,  "Owner",                                "/moov/udata/meta/ilst/ownr";
    PGAP, b"pgap",      ContainerType::Container,      false,  "Program Gap boolean",                  "/moov/udata/meta/ilst/pgap";
    PCST, b"pcst",      ContainerType::Container,      false,  "Podcast",                              "/moov/udata/meta/ilst/pcst";
    PLID, b"plID",      ContainerType::Container,      false,  "ITunes Playlist ID",                   "/moov/udata/meta/ilst/pcst";
    PURD, b"purd",      ContainerType::Container,      false,  "Purchase Date",                        "/moov/udata/meta/ilst/purd";
    RATE, b"rate",      ContainerType::Container,      false,  "Rating",                               "/moov/udata/meta/ilst/rate";
    RTNG, b"rtng",      ContainerType::Container,      false,  "Advisory",                             "/moov/udata/meta/ilst/rtng";
    SFID, b"sfID",      ContainerType::Container,      false,  "Apple Store / ITunes ID?",             "/moov/udata/meta/ilst/sfID";
    SOAA, b"soaa",      ContainerType::Container,      false,  "Sort Album Artist",                    "/moov/udata/meta/ilst/soaa";
    SOAL, b"soal",      ContainerType::Container,      false,  "Sort ALbum",                           "/moov/udata/meta/ilst/soal";
    SOAR, b"soar",      ContainerType::Container,      false,  "Sort Artist",                          "/moov/udata/meta/ilst/soar";
    SOCO, b"soco",      ContainerType::Container,      false,  "Sort Composer",                        "/moov/udata/meta/ilst/soco";
    SONM, b"sonm",      ContainerType::Container,      false,  "Sort Name",                            "/moov/udata/meta/ilst/sonm";
    SOSN, b"sosn",      ContainerType::Container,      false,  "Sort Show",                            "/moov/udata/meta/ilst/sosn";
    STIK, b"stik",      ContainerType::Container,      false,  "Media Type",                           "/moov/udata/meta/ilst/stik";
    TMPO, b"tmpo",      ContainerType::Container,      false,  "Tempo guide",                          "/moov/udata/meta/ilst/tmpo";
    TRKN, b"trkn",      ContainerType::Container,      false,  "Track number and total tracks",        "/moov/udata/meta/ilist/trkn";
    TVEN, b"tven",      ContainerType::Container,      false,  "TV Episode Name",                      "/moov/udata/meta/ilist/tven";
    TVES, b"tves",      ContainerType::Container,      false,  "TV Episode Number",                    "/moov/udata/meta/ilist/tves";
    TVNN, b"tvnn",      ContainerType::Container,      false,  "TV Network Name",                      "/moov/udata/meta/ilist/tvnn";
    TVSH, b"tvsh",      ContainerType::Container,      false,  "TV Show Name",                         "/moov/udata/meta/ilist/tvsh";
    TVSN, b"tvsn",      ContainerType::Container,      false,  "TV Show Number",                       "/moov/udata/meta/ilist/tvsn";
    XID,  b"xid ",      ContainerType::Container,      false,  "Vendor Id",                            "/moov/udata/meta/ilist/xid ";
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
    NAME, b"name",      ContainerType::NotContainer,   true,   "Embeded in Special Item",              "/moov/udata/meta/ilst/----/name";
    MEAN, b"mean",      ContainerType::NotContainer,   true,   "Embedded in Special Item?",            "/moov/udata/meta/ilst/----/mean";
}
