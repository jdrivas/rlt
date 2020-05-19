extern crate bytes;
pub mod ilst;
pub mod mdia;
pub mod stbl;
use bytes::buf::Buf;
use std::fmt;

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

/// Box Kind (box type in the MPEG spec, type is a rust keyword).
/// Constants for the 4 byte box designator.

/// Container Boxes
pub const DINF: [u8; 4] = *b"dinf"; // Data Information Box Container /moov/trak/mdia/dinf
pub const META: [u8; 4] = *b"meta"; // Meta Data Container /moov/meta & /moov/trak/meta /mmov/udata/meta
pub const MINF: [u8; 4] = *b"minf"; // Median Infomration Container    /moov/trak/mdia/minf
pub const MOOV: [u8; 4] = *b"moov"; // Moov Container for all Metadata  /moov
pub const TRAK: [u8; 4] = *b"trak"; // Trak Container /moov/trak
pub const UDTA: [u8; 4] = *b"udta"; // User Data Container   /moov/udta

/// Full Boxes
pub const DREF: [u8; 4] = *b"dref"; // Data reference. Declares sources of media data. /moov/trak/mdia/minf/dinf
pub const HDLR: [u8; 4] = *b"hdlr"; // Hnalder general handler header. /moov/trak/mdia, /moov/udata/meta
pub const MVHD: [u8; 4] = *b"mvhd"; // Movie Header /moov
pub const TKHD: [u8; 4] = *b"tkhd"; // Track Header /moov/trak
pub const SMHD: [u8; 4] = *b"smhd";
pub const URL_: [u8; 4] = *b"url ";

static SIMPLE_CONTAINERS: [[u8; 4]; 27] = [
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
    ilst::TRKN,
    stbl::STBL,
];
static FULL_CONTAINERS: [[u8; 4]; 1] = [META];
static FULL_BOXES: [[u8; 4]; 14] = [
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
    stbl::STSD,
    stbl::STTS,
    stbl::STSZ,
    URL_,
];

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
fn read_box_header<'i>(buf: &mut &'i [u8]) -> MP4Box<'i> {
    // Read box header: [sssstttt]
    // s = 1 byte of size; 4 total.
    // t = 1 byet of box type; 4 total.
    let mut read = 0;
    let s = buf.get_u32();
    read += 4;

    let kind = &buf[0..4];
    buf.advance(4); // for the kind we referenced above.
    read += 4;

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

    // Buffer not read yet.
    let rest = &buf[0..(s as usize - read)];

    let mut b = MP4Box {
        size: s,
        kind: [0; 4],
        buf: rest,
        box_type: bt,
    };
    b.kind.copy_from_slice(kind);

    // Move this buffer point along.
    if !b.box_type.is_container() {
        buf.advance(s as usize - read);
    }

    // TODO(jdr): this is redundantly
    // stored in Box. Remove the size return.
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
pub fn get_ftyp_box_values<'a>(
    buf: &mut &'a [u8],
    brand: &mut &'a [u8],
    version: &mut u8,
    flags: &mut u32,
    compat_brands: &mut Vec<&'a [u8]>,
) -> usize {
    let mut read = 0;

    *brand = &buf[0..4];
    buf.advance(4);
    read += 4;

    *flags = buf.get_u32();
    *version = (*flags >> 28) as u8;
    *flags &= 0x00FFFFFF;
    (*buf).advance(4);
    read += 4;

    while buf.len() > 0 {
        compat_brands.push(&buf[0..4]);
        buf.advance(4);
        read += 4;
    }

    read
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
