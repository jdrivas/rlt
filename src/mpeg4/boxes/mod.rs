extern crate bytes;
// use lazy_static; // make sure extern crate with macro_use is defined in top/lib.rs
#[macro_use]
pub mod box_types;
pub mod ilst;
pub mod mdia;
pub mod stbl;

use box_types::{BoxType, ContainerType};
// use crate::mpeg4::util;
use bytes::buf::Buf;
// use std::collections::HashMap;
use std::fmt;
// use std::sync::Mutex;

/// Holds the buffer and supports
/// iteration over the MP4Boxes
/// in the buffer.
pub struct MP4Buffer<'a, 'b> {
    pub buf: &'b mut &'a [u8],
}

// TODO(jdr): The below needs to be fit into MP4Box.
pub enum BoxTypeHolder {
    Known(BoxType),   // These are the ones we know.
    Unknown([u8; 4]), // These are the ones we just pick up along the way and may want to display.
}

/// Holds header information from the box
/// and the buffer for the data assocaited
/// with the box.
pub struct MP4Box<'a> {
    pub size: u32,
    pub box_type: &'static BoxType,
    pub buf: &'a [u8],
    pub version_flag: Option<VersionFlag>,
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
            // String::from_utf8_lossy(&self.kind[..]),
            self.box_type.type_str(),
            self.size,
            self.box_type,
            self.buf.len(),
        )
    }
}

#[derive(Debug)]
pub struct VersionFlag {
    pub version: u8,
    pub flag: u32,
}

/// Captures the Simple and the Full Box
/// definitions (with or without Version & Flags)
/// and captures container versus not a conatiner.
// #[derive(Debug)]
// pub enum BoxType {
//     Simple,
//     Full(VersionFlag),
//     SimpleContainer,
//     FullContainer(VersionFlag),
// }

// impl BoxType {
//     pub fn is_full(&self) -> bool {
//         match self {
//             BoxType::Simple | BoxType::SimpleContainer => false,
//             _ => true,
//         }
//     }

//     pub fn is_container(&self) -> bool {
//         match self {
//             BoxType::SimpleContainer | BoxType::FullContainer(_) => true,
//             _ => false,
//         }
//     }
// }

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
    // t = 1 byte of box type; 4 total.
    // println!("Bufferhead {:x?}", &buf[0..8]);
    let mut read: usize = 0;
    let s = buf.get_u32();
    read += 4;

    let bt = buf.get_u32();
    read += 4;
    let bt = BoxType::ref_from(bt);

    // For full boxes, pick up the version and flags.
    // It would not be entirely unreasonable to pick
    // these up in specific box reader functions as needed.
    // But that seems repititous.
    let vf = if bt.full {
        read += 4;
        Some(get_version_flags(buf))
    } else {
        None
    };

    // There are some container boxes that are not
    // simply containers, but actually have data in them.
    // We need to skip past the data to point to the next
    // box in the continaer.
    if let ContainerType::Special(skip) = bt.container {
        // println!("\tSkipping special {:?}", bt);
        buf.advance(skip as usize);
        read += skip as usize;
    }

    // Buffer not read yet.
    let rest = &buf[0..(s as usize - read)];
    // println!("\tRest len: {}", rest.len());

    let b = MP4Box {
        size: s,
        buf: rest,
        box_type: bt,
        version_flag: vf,
    };

    // Move this buffer pointer along to the end of the box.
    if b.box_type.container == ContainerType::NotContainer {
        buf.advance(s as usize - read);
    }

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
