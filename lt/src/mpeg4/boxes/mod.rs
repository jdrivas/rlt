//! Implementation and interface to the Box model for MPEG4.
extern crate bytes;
#[macro_use]
pub mod box_types;
pub mod ilst;
pub mod mdia;
pub mod stbl;

use box_types::{BoxType, ContainerType};
use bytes::buf::Buf;
// use std::error::Error;
use std::fmt;
// use std::io::{Read, Seek};

/// Box header is [size] + [type (four character code)]
pub const BOX_HEADER_SIZE: usize = 8;
/// Full box [size] + [type] + [version/flags]
pub const FULL_BOX_HEADER_SIZE: usize = BOX_HEADER_SIZE + 4;

/// Holds the buffer and supports
/// iteration over the MP4Boxes
/// in the buffer.
// TODO(jdr): This probably wants a rethink.
pub struct MP4Buffer<'a, 'b> {
    pub buf: &'b mut &'a [u8],
}

impl<'a> std::iter::Iterator for MP4Buffer<'a, '_> {
    type Item = MP4Box<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.is_empty() {
            return None;
        }
        // println!("Next: Buf len: {:#0x}", self.buf.len());
        let b = read_box_header(self.buf);
        // println!("Next end: Buff len = {:#0x}", self.buf.len());
        Some(b)
    }
}

/// Holds header information from the box
/// and the buffer for the data assocaited
/// with the box.
#[derive(PartialEq, Eq)]
pub struct MP4Box<'a> {
    pub size: u32,
    pub box_type: BoxType,
    pub buf: &'a [u8],
    pub version_flag: Option<VersionFlag>,
}

impl<'a> MP4Box<'a> {
    // read calls the function provided and sending it this box.
    pub fn read(&mut self, rf: &mut impl FnMut(&mut MP4Box<'a>)) {
        rf(self);
    }
}

impl fmt::Debug for MP4Box<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}  [{:10?}] {:?} Buffer[{}]",
            self.box_type.four_cc(),
            self.size,
            self.box_type,
            self.buf.len(),
        )
    }
}

/// For FullBoxes the version and the flags value.
#[derive(Debug, PartialEq, Eq)]
pub struct VersionFlag {
    pub version: u8,
    pub flag: u32,
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

/// Read box header reads in the size and 4 character code type as a u32.
/// This type is used to match to an enum for the known BoxTypes which
/// determine the details in the BoxSpec (Box/FullBox, Container/NotContainer.)
/// In the caes that the type is not known and Unknown type which can at
/// least be printed out is provided.
pub fn read_box_header<'i>(buf: &mut &'i [u8]) -> MP4Box<'i> {
    // Read box header: [sssstttt]
    // s = 1 byte of size; 4 total.
    // t = 1 byte of box type; 4 total.
    // println!("Bufferhead {:x?}", &buf[0..8]);
    let rest = &buf[0..buf.len()];
    let mut read: usize = 0;
    let (r, s, bt) = read_box_size_type(buf);
    read += r;

    // println!(
    //     "Creating rest. Buf.len() {}, Box size: {}, Bytes Read: {}",
    //     buf.len(),
    //     s,
    //     read
    // );
    let rest = &rest[0..(s as usize)];

    // println!("Read box header: {} [{}/0x{:0x?}]", FourCC(bt), s, s);
    let box_type = BoxType::from(bt);
    let box_spec = box_type.spec();

    // println!("Read Box Type: {:?}", box_type);
    // println!("\tSpec: {:?}", box_spec);

    // Read the VF
    let vf = if box_spec.full {
        read += 4;
        Some(get_version_flags(buf))
    } else {
        None
    };

    // println!("\tRest len: {}", rest.len());

    if let ContainerType::Special(skip) = box_spec.container {
        buf.advance(skip);
        read += skip as usize;
    }

    // Move this buffer pointer along to the end of the box.
    // This needs to happen after we've created rest.
    if box_spec.container == ContainerType::NotContainer {
        buf.advance(s as usize - read);
    }
    // } else {
    //     buf.advance(s as usize - read);
    // }

    #[allow(clippy::redundant_field_names)]
    MP4Box {
        size: s,
        buf: rest,
        box_type: box_type,
        version_flag: vf,
    }
}

fn get_version_flags(buf: &mut &[u8]) -> VersionFlag {
    let mut vf = VersionFlag {
        flag: buf.get_u32(),
        version: 0,
    };

    vf.version = (vf.flag >> 28) as u8;
    vf.flag &= 0x00_FF_FF_FF;

    vf
}

/// Read the MPEGBox Size and the type from a buffer.
///
/// Returns the number of bytes read, should be 8, the size, the box type).
pub fn read_box_size_type(buf: &mut &[u8]) -> (usize, u32, u32) {
    (8, buf.get_u32(), buf.get_u32())
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
// the arguments but buff to options.
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
    *flags &= 0x00_FF_FF_FF;

    while buf.is_empty() {
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
