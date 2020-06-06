use crate::mpeg4::boxes;
use boxes::box_types;
use boxes::{MP4Box, MP4Buffer};

/// Searches for a box in an MPG4 buffer.
///
/// Retruns `None` if no such box is found, other wise returns the Box
/// which matches the last type in the path.
///
///
/// # Examples
///
/// These doc tests are crashing!
// / ```
///  use lt::mpeg4::boxes;
/// use boxes::box_types;
/// use std::io::Read;
/// use lt::mpeg4::find::find_box;
///
/// let f = "/Volumes/London Backups/Itunes_Library/The Beatles/Abbey Road/01 Come Together.m4a";
/// let mut file = std::fs::File::open(f).unwrap();
/// let mut vbuf = Vec::<u8>::new();
/// let _n = file.read_to_end(&mut vbuf);
/// let buf = vbuf.as_slice();
///
/// // Return the moov box.
/// let mut bx = find_box("/moov", buf).unwrap();
///  assert_eq!(bx.box_type, box_types::MOOV);
///
/// // Find a specific path to a data box
/// bx = find_box("/moov/udta/meta/ilst/trkn/data", buf).unwrap();
/// assert_eq!(bx.box_type, box_types::DATA);
///
/// // Find the first data box in the stream.
/// let bx2 = find_box("data", buf).unwrap();
/// assert_eq!(bx.box_type, box_types::DATA);
/// assert_ne!(bx, bx2);
///
/// // A path is really about finding a box
/// // through a path that reads through the
/// // boxes in the path. No real structural
/// // relationship to MPEG and it's conained
/// // boxes model is implied.
/// bx = find_box("/moov/mdia/stts/udata/ilst", buf).unwrap();
/// assert_eq!(bx.box_type, box_types::ILST);
// / ```
///
pub fn find_box<'a>(path: &str, mut b: &'a [u8]) -> Option<MP4Box<'a>> {
    // create a stack of BoxTypes to searc for from the path
    // Create an MP4Buffer from the buf
    // Get boxes wait for the first box to match the top of the
    // stack.
    // When that is found pop it, check each subsequent box
    // against the top box, fail if it's not the same type
    // pop it and get the next box if it is.

    // println!("Staring with path: {:?}", path);

    let mut bts = Vec::new();
    let mut v: Vec<&str> = path.rsplit('/').collect();
    // eprintln!("V = {:?}", v);
    if v[v.len() - 1].is_empty() {
        v.remove(v.len() - 1);
    };
    if v.is_empty() {
        return None;
    }

    // Translate the path elements into a stack of BoxTypes
    for s in v {
        let bs = &s.as_bytes()[0..4];
        let bt: box_types::BoxType = From::from(bs);
        bts.push(bt);
    }
    // eprintln!("Stack: {:?}", bts);

    let boxes = MP4Buffer { buf: &mut b };
    for b in boxes {
        // eprintln!("Looking at {:?}", b);
        if b.box_type == *bts.last().unwrap() {
            bts.pop();
            if bts.is_empty() {
                return Some(b);
            }
        }
    }
    None
}

#[test]
fn test_find_box() {
    use std::fs::File;
    use std::io::Read;

    let f = "/Volumes/London Backups/Itunes_Library/The Beatles/Abbey Road/01 Come Together.m4a";
    let mut file = File::open(f).unwrap();
    let mut vbuf = Vec::<u8>::new();
    let _n = file.read_to_end(&mut vbuf);
    let buf = vbuf.as_slice();

    let mut bx = find_box("/ftyp", buf).unwrap();
    assert_eq!(bx.box_type, box_types::FTYP);

    // Note, that at least currrently, the
    // semantic meaning in the / is merely
    // as a spearator between box names
    // and that find just looks for that
    // sequence of boxes with any number of
    // other boxes in between.
    bx = find_box("/ftyp/moov", buf).unwrap();
    assert_eq!(bx.box_type, box_types::MOOV);

    bx = find_box("/moov/trak", buf).unwrap();
    assert_eq!(bx.box_type, box_types::TRAK);

    bx = find_box("trak/tkhd", buf).unwrap();
    assert_eq!(bx.box_type, box_types::TKHD);
}
