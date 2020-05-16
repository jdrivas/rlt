extern crate bytes;
use bytes::buf::Buf;
use std::fmt;

// MP4Buffer and MP4Box BoxTypes.
//

/// Holds the buffer and supports
/// iteration over the MP4Boxes
/// inside the buffer.
pub struct MP4Buffer<'a, 'b> {
    pub buf: &'b mut &'a [u8],
}

pub struct MP4Box<'a> {
    pub size: u32,
    pub kind: &'a [u8],
    pub buf: &'a [u8],
    pub box_type: BoxType,
    pub path: Vec<&'a [u8]>,
}

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

static simple_container_kinds: [&[u8; 4]; 27] = [
    b"moov",
    b"trak",
    b"udta",
    b"mdia",
    b"minf",
    b"dinf",
    b"ilst",
    b"stbl",
    &[0xa9, b'a', b'l', b'b'], // Album
    &[0xa9, b'a', b'r', b't'], // Artist
    &[0xa9, b'A', b'R', b'T'], // Artist
    &[0xa9, b'c', b'm', b't'], // Comment
    &[0xa9, b'd', b'a', b'y'], // Year
    &[0xa9, b'g', b'e', b'n'], // Genre
    &[0xa9, b'g', b'r', b'p'], // Genre
    &[0xa9, b'l', b'y', b'r'], // Lyric
    &[0xa9, b'n', b'a', b'm'], // Title/Name
    &[0xa9, b't', b'o', b'o'], // Encoder
    &[0xa9, b'w', b'r', b't'], // wrtier/author
    b"aART",
    b"covr",
    b"cpil",
    b"disk",
    b"gnre",
    b"pgap",
    b"tmpo",
    b"trkn",
];
static full_container_kinds: [&[u8; 4]; 1] = [b"meta"];
static full_box_kinds: [&[u8; 4]; 3] = [b"mvhd", b"tkhd", b"data"];

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
    let bt = if full_box_kinds.iter().find(|v| kind == &v[..]).is_some() {
        let vf = get_version_flags(buf);
        read += 4;
        BoxType::Full(vf)
    } else if full_container_kinds
        .iter()
        .find(|v| kind == &v[..])
        .is_some()
    {
        let vf = get_version_flags(buf);
        read += 4;
        BoxType::FullContainer(vf)
    } else if simple_container_kinds
        .iter()
        .find(|v| kind == &v[..])
        .is_some()
    {
        BoxType::SimpleContainer
    } else {
        BoxType::Simple
    };

    // Buffer not read yet.
    let rest = &buf[0..(s as usize - read)];

    let b = MP4Box {
        size: s,
        kind: kind,
        buf: rest,
        box_type: bt,
        path: Vec::new(),
    };

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

// #[derive(Debug)]
pub enum DataBoxContent<'a> {
    Byte(u8),
    Text(&'a [u8]),
    Data(&'a [u8]),
}

impl fmt::Debug for DataBoxContent<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataBoxContent::Byte(v) => write!(f, "Byte({:04x})", v),
            DataBoxContent::Text(v) => write!(f, "Text({:?})", String::from_utf8_lossy(v)),
            DataBoxContent::Data(v) => {
                let l = v.len();
                if l > 32 {
                    write!(
                        f,
                        "Data({:x?} ... {:x?} len = {}",
                        &v[0..8],
                        &v[l - 8..l],
                        l
                    )
                } else {
                    write!(f, "Data({:x?})", v)
                }
            }
        }
    }
}

const IMPLICIT_FLAG: u32 = 0;
const TEXT_FLAG: u32 = 1;
const JPEG_FLAG: u32 = 13;
const PNG_FLAG: u32 = 14;
const BYTE_FLAG: u32 = 21;

// TODO(jdr): Think about getting rid of the buf.get_XX() calls.
// They modify the buffer point, which is probably not what we really
// want.
pub fn get_data_box<'a>(bx: &'a mut MP4Box) -> DataBoxContent<'a> {
    // println!("box: {:?}", bx);
    // println!("buff: {:x?}", bx.buf);
    // data box has a predfeined 0
    bx.buf.get_u32();
    if let BoxType::Full(vf) = &bx.box_type {
        match vf.flag {
            TEXT_FLAG => DataBoxContent::Text(&bx.buf),
            IMPLICIT_FLAG | JPEG_FLAG | PNG_FLAG => DataBoxContent::Data(&bx.buf),
            BYTE_FLAG => DataBoxContent::Byte(bx.buf.get_u8()),
            _ => DataBoxContent::Byte(b'0'), // The true cases here is an error.
        }
    } else {
        // This branch of the if is an error, so maybe we should return one?
        panic!("Read a data box that wasn't a BoxType:;Full()\n{:?}", bx);
    }
}
