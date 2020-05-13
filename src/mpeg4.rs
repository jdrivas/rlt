extern crate bytes;
use crate::file;
use crate::file::FileFormat;
use crate::track;

use bytes::buf::Buf;
use std::error::Error;
use std::fmt;
use std::io::{Read, Seek};
// use std::str::from_utf8;

pub struct Mpeg4;

const FTYP_HEADER: &[u8] = b"ftyp";
const M42_HEADER: &[u8] = b"mp42";
const M4A_HEADER: &[u8] = b"M4A ";

pub fn identify(b: &[u8]) -> Option<FileFormat> {
    let mut ft = None;
    if b.len() >= 12 {
        if &b[4..8] == FTYP_HEADER {
            ft = match &b[8..12] {
                b if b == M42_HEADER => Some(FileFormat::MPEG4(Mpeg4 {})),
                b if b == M4A_HEADER => Some(FileFormat::MPEG4(Mpeg4 {})),
                // b if b == M4B_HEADER => return Some(FileFormat::MP4B),
                // b if b == M4P_HEADER => return Some(FileFormat::MP4P),
                _ => None,
            };
        }
    }

    return ft;
}

const FORMAT_NAME: &str = "MPEG-4";
impl file::Decoder for Mpeg4 {
    fn name(&self) -> &str {
        FORMAT_NAME
    }

    fn get_track(
        &mut self,
        mut r: impl Read + Seek,
    ) -> Result<Option<track::Track>, Box<dyn Error>> {
        let mut vbuf = Vec::<u8>::new();
        let _n = r.read_to_end(&mut vbuf);
        let buf = vbuf.as_slice();
        // read_buf(buf)?;
        display_structure(buf);
        Ok(None)
    }
}

fn display_structure(buf: &[u8]) {
    let b: &mut &[u8] = &mut &(*buf);
    let boxes = MP4Buffer { buf: b };

    let mut level = vec![(boxes.buf.len(), 0, String::from_utf8_lossy(b""))]; // (size, count, kind)
    let mut tabs = String::new();
    for b in boxes {
        println!(
            "{}{:?}  [{:?}] {:?}",
            tabs,
            String::from_utf8_lossy(b.kind),
            b.size,
            b.box_type,
        );

        // Implement the "recursion" for indented printout.
        level.last_mut().unwrap().1 += b.size as usize; // Add to the count.

        // Push a level for a new container.
        if b.box_type.is_container() {
            level.push((b.buf.len(), 0, String::from_utf8_lossy(b.kind))); // push a level
            tabs.push('\t');
        }

        // Pop levels when we've finished them.
        // size == count means we've read all the boxes in the conatiner.
        while level.last().unwrap().0 == level.last().unwrap().1 {
            tabs.pop();
            if level.len() > 1 {
                println!("{}<{}>", tabs, level.last().unwrap().2);
                // println!("{}|{}|", tabs, level.last().unwrap().2);
                // println!("{}{}", tabs, " ____");
            }
            level.pop();
            if level.len() == 0 {
                break;
            }
        }
    }
}

fn read_buf(buf: &[u8]) -> Result<(), Box<dyn Error>> {
    let b: &mut &[u8] = &mut &(*buf);
    let mut boxes = MP4Buffer { buf: b };
    let mut bx = FtypBox {
        brand: &[][..],
        flags: 0,
        version: 0,
        compat_brands: Vec::<&[u8]>::new(),
    };

    let mut f = get_box_reader(&mut bx);
    for mut b in &mut boxes {
        b.read(&mut f);
        println!("{:?}", b);
    }
    Ok(())
}

fn get_box_reader<'a>(bx: &'a mut FtypBox<'a>) -> impl FnMut(&mut &'a [u8], &[u8]) {
    move |buf: &mut &[u8], kind: &[u8]| match kind {
        b"ftyp" => {
            get_ftyp_box_values(
                buf,
                &mut bx.brand,
                &mut bx.version,
                &mut bx.flags,
                &mut bx.compat_brands,
            );
        }
        _ => (),
    }
}

// fn fill(buf &mut &[u8], kind: &[u8]){
//     let b = FtypBox {
//         brand: &[][..],
//         flags: 0,
//         version: 0,
//         compat_brands: Vec:<&u[8]>::new(),
//     }
// }

// This can be turned into a hash table if we like.
// fn get_box_type<'i>(kind: &[u8], buf: &mut &'i [u8]) -> (usize, Option<BoxType<'i>>) {
//     match kind {
//         b"ftyp" => get_ftyp_box(buf),
//         _ => (0, None),
//     }
// }

// MP4Buffer and MP4Box BoxTypes.
//
struct MP4Buffer<'a, 'b> {
    buf: &'b mut &'a [u8],
}

struct MP4Box<'a> {
    size: u32,
    kind: &'a [u8],
    buf: &'a [u8],
    box_type: BoxType,
}

#[derive(Debug)]
enum BoxType {
    Simple,
    Full(VersionFlag),
    SimpleContainer,
    FullContainer(VersionFlag),
}

impl BoxType {
    fn is_full(&self) -> bool {
        match self {
            BoxType::Simple | BoxType::SimpleContainer => false,
            _ => true,
        }
    }

    fn is_container(&self) -> bool {
        match self {
            BoxType::SimpleContainer | BoxType::FullContainer(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
struct VersionFlag {
    version: u8,
    flag: u32,
}

impl<'a> MP4Box<'a> {
    fn read(&mut self, rf: &mut impl FnMut(&mut &'a [u8], &[u8])) {
        let mut b: &'a [u8] = self.buf;
        rf(&mut b, self.kind);
        // self.buf.advance(self.size as usize);
    }
}

impl<'a> std::iter::Iterator for MP4Buffer<'a, '_> {
    type Item = MP4Box<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() == 0 {
            return None;
        }
        // println!("Next: Buf len: {:#0x}", self.buf.len());
        let (_s, b) = read_box(self.buf);
        // println!("Next end: Buff len = {:#0x}", self.buf.len());
        Some(b)
    }
}

impl fmt::Debug for MP4Box<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}  [{:?}] {:?} Buffer[{}]",
            String::from_utf8_lossy(self.kind),
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

static simple_container_kinds: [&[u8; 4]; 8] = [
    b"moov", b"trak", b"udta", b"mdia", b"minf", b"dinf", b"ilst", b"stbl",
];
static full_container_kinds: [&[u8; 4]; 1] = [b"meta"];
static full_box_kinds: [&[u8; 4]; 2] = [b"mvhd", b"tkhd"];

fn read_box<'i>(buf: &mut &'i [u8]) -> (usize, MP4Box<'i>) {
    // Read box header: [sssstttt]
    // s = 1 byte of size; 4 total.
    // t = 1 byet of box type; 4 total.
    let mut read = 0;
    let s = buf.get_u32();
    read += 4;

    let kind = &buf[0..4];
    buf.advance(4); // for the kind we referenced above.
    read += 4;

    let bt = if full_box_kinds.iter().find(|v| kind == &v[..]).is_some() {
        read += 4;
        BoxType::Full(get_version_flags(buf))
    } else if full_container_kinds
        .iter()
        .find(|v| kind == &v[..])
        .is_some()
    {
        read += 4;
        BoxType::FullContainer(get_version_flags(buf))
    } else if simple_container_kinds
        .iter()
        .find(|v| kind == &v[..])
        .is_some()
    {
        BoxType::SimpleContainer
    } else {
        BoxType::Simple
    };

    let rest = &buf[0..(s as usize - read)];

    let b = MP4Box {
        size: s,
        kind: kind,
        buf: rest,
        box_type: bt,
    };

    if !b.box_type.is_container() {
        buf.advance(s as usize - read);
    }

    return (s as usize, b);
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
struct FtypBox<'a> {
    brand: &'a [u8],
    version: u8,
    flags: u32,
    compat_brands: Vec<&'a [u8]>,
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
fn get_ftyp_box_values<'b>(
    buf: &mut &'b [u8],
    brand: &mut &'b [u8],
    version: &mut u8,
    flags: &mut u32,
    compat_brands: &mut Vec<&'b [u8]>,
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
