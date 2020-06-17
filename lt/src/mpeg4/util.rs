//! Utilty functions to support MPEG4 display and use.
use super::boxes;
use box_types::{BoxType, ContainerType};
use boxes::box_types;
use std::fmt;

/// This got built because doing the various jobs with recursion
/// got me into lots of fights with the borrow checker.
/// So I just decieded that this was an easier path.
// #[derive(Clone, Copy)]
pub struct BoxCounter {
    pub size: usize,
    pub count: usize,
    pub box_type: BoxType, // TODO(jdr): this should probably be a reference.
}

impl<'a> fmt::Debug for BoxCounter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?} - Size: {}, Count: {}",
            self.box_type.four_cc(),
            self.size,
            self.count
        )
    }
}

/// A stack of BoxCounters used to capture containers and when they've been filled.
/// This particularly useful for understanding the context in which a box has been
/// found (ie the path to the box) as well as for doing things like
#[derive(Debug, Default)]
pub struct LevelStack {
    levels: Vec<BoxCounter>,
}

impl LevelStack {
    // pub fn new(size: usize) -> LevelStack {
    pub fn new() -> LevelStack {
        LevelStack {
            ..Default::default()
        }
    }

    /// Add a box and calcuate it's impact on the stack
    /// 1. We increment the count against the current container
    /// by the size of the new box. This box will take up
    /// so many bytes of against the total in the enclosing container.
    /// 2. If this is a container, add this box to the stack to account
    /// for comming enclosed boxes.
    /// 3. If there is not box on the staic and this is
    /// not a container, we can safely ignore the box for level stacking purposes.
    pub fn add_box(&mut self, b: boxes::MP4Box) {
        if !self.levels.is_empty() {
            // println!("Adding box: {:?}", b);
            // println!("Stack: {:?}", self);
            let mut last = self.levels.last_mut().unwrap();
            last.count += b.size as usize;

            // Don't forget to count the goofy special boxes that are both
            // containers boxes that have things in them that count against
            // their sizes.
            if let ContainerType::Special(v) = last.box_type.spec().container {
                last.count += v;
            };
        }

        if b.box_type.is_container() {
            // self.push_new(b.buf.len(), b.box_type);
            self.push_new(b.size as usize, b.box_type);
        }
    }

    /// Adds the box and the runs the closure provdied.
    /// Useful for example to increae the tab length with the box is a conatiner.
    /// The closure is called with this LevelStack and the box that was passed in.
    // pub fn add_box_with(
    //     &mut self,
    //     b: boxes::MP4Box,
    //     mut f: impl FnMut(&LevelStack, &boxes::MP4Box),
    // ) {
    //     self.add_box(b);
    //     f(self, &b);
    // }

    fn push_new(&mut self, sz: usize, bt: BoxType) {
        self.levels.push(BoxCounter {
            size: sz,
            count: bt.header_size(), // act as if you've read in the header.
            box_type: bt,
        });
    }

    /// Determine if container at the top of the stack
    /// has been completed, and if so remove the box from
    /// the stack. Continue to remove compledted containers
    // /until you find one that is not complete or you exhause the
    /// stack.
    pub fn check_and_complete(&mut self) {
        self.check_and_complete_with(|_| {}); // TODO(jdr): This really wants an Option to a closure.
    }

    /// Check if the top container is complete, and if so remove and continue checking,
    /// with a convenience for executing a closure
    /// once for each container to be removed from the stack and so,
    /// logically, when the container has been completed.
    /// Useful for managing indentation in a display, for example.
    /// The closure is executed prior to the top being poped from the stack.
    pub fn check_and_complete_with(&mut self, mut f: impl FnMut(&LevelStack)) {
        while self.complete() {
            f(self);
            self.pop();
            if self.is_empty() {
                break;
            }
        }
    }

    /// Convenience function to add a new box and immediately
    /// check for completion.
    pub fn update(&mut self, b: boxes::MP4Box) {
        self.add_box(b);
        self.check_and_complete();
    }

    /// Convenience with closures used as in add_with, and check_and_complete_with.
    // pub fn update_with(
    //     &'a mut self,
    //     b: &'a boxes::MP4Box,
    //     add_f: impl FnMut(&LevelStack, &boxes::MP4Box),
    //     cmplt_f: impl FnMut(&LevelStack),
    // ) {
    //     self.add_box_with(b, add_f);
    //     self.check_and_complete_with(cmplt_f);
    // }

    /// Has the container on the top of the stack been completed?
    /// Practically this means if the size is equal to the count.
    pub fn complete(&self) -> bool {
        if !self.is_empty() {
            let last = self.levels.last().unwrap();
            // println!(
            //     "Checking complete: Size: {}, Count: {}, Diff: {}",
            //     last.size,
            //     last.count,
            //     last.size as i32 - last.count as i32
            // );
            last.size == last.count
        } else {
            false
        }
    }

    /// Take the top box off the stack.
    pub fn pop(&mut self) -> Option<BoxCounter> {
        self.levels.pop()
    }

    /// Get the top box from the stack as a mutable reference.
    pub fn top_mut(&mut self) -> Option<&mut BoxCounter> {
        self.levels.last_mut()
    }

    /// Get the top box counter from the stack.
    pub fn top(&self) -> Option<&BoxCounter> {
        self.levels.last()
    }

    /// How many box counters are on the stack.
    pub fn len(&self) -> usize {
        self.levels.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// What's the path to the current top box.
    pub fn path(&self) -> Vec<String> {
        let mut v = Vec::new();
        for l in &self.levels {
            v.push(l.box_type.four_cc());
        }
        v
    }

    /// A string representation of the path.
    pub fn path_string(&self) -> String {
        let mut s = "/".to_string();
        if !self.is_empty() {
            s += &self.path().join("/")
        }
        s
    }
}

/// Tabs
///  Helper wrapper for indenting and undenting.
/// Default to tabs, but could be used to add any single char.
///  If we find the need we could easily modify it to take strings
/// instead of chars as the indent token.
#[derive(Default)]
pub struct Tabs {
    t: String,
    c: char,
}

impl Tabs {
    pub fn new() -> Tabs {
        Tabs {
            t: String::new(),
            c: '\t',
        }
    }
    pub fn new_with(ch: char) -> Tabs {
        Tabs {
            t: String::new(),
            c: ch,
        }
    }

    pub fn indent(&mut self) {
        self.t.push(self.c);
    }

    pub fn outdent(&mut self) {
        self.t.pop();
    }
}

impl fmt::Display for Tabs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.t)
    }
}

// TODO(jdr): How to get a function/macro tprintln!(), that effectively
// does: println!("{} ....", tabs);
// tabs.println()?

#[allow(clippy::explicit_counter_loop)]
pub fn dump_buffer(buf: &[u8]) {
    let mut ascii: String = "".to_string();
    let mut line = 0;
    for (i, b) in buf.iter().enumerate() {
        // End of line.
        if i % 16 == 0 && line != 0 {
            println!("  {}", ascii);
            ascii = "".to_string();
        }
        if i % 4 == 0 {
            print!(" ");
            ascii += " ";
        }

        if b.is_ascii_alphanumeric() || b.is_ascii_punctuation() {
            ascii.push(*b as char);
        } else {
            ascii.push('.');
        }
        print!("{:02x} ", b);
        line += 1;
    }
    let left = 16 - buf.len() % 16;
    if left > 0 {
        let mut spaces: String = "".to_string();
        for _ in 0..left {
            spaces += "   ";
        }
        println!("{}   {}", spaces, ascii);
    }
    // println!("Left: {}", left);
}

#[cfg(test)]
mod tests {

    use super::*;
    // use box_types::{BoxSpec, BoxType, ContainerType};
    use box_types::*;
    use boxes::{MP4Box, MP4Buffer, BOX_HEADER_SIZE};
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_structure() {
        let f = "/Volumes/London Backups/Itunes_Library/The Beatles/Abbey Road/16 The End.m4a";
        let mut file = File::open(f).unwrap();
        let mut vbuf = Vec::<u8>::new();
        let _n = file.read_to_end(&mut vbuf);
        let boxes = MP4Buffer {
            buf: &mut vbuf.as_slice(),
        };

        let mut l = LevelStack::new();
        for b in boxes {
            l.add_box(b);
            while l.complete() {
                l.pop();
                if l.is_empty() {
                    break;
                }
            }
        }
        assert!(l.is_empty());
    }

    #[test]
    fn test_level_stack_smoke() {
        let mut tests = Vec::new();
        tests.push(vec![
            new_container(100, 0x20_20_20_20),
            new_simple_box(100 - BOX_HEADER_SIZE as u32, 0x_21_21_21_21),
        ]);
        tests.push(vec![
            new_container(100, 0x20_20_20_20),
            new_full_box(100 - BOX_HEADER_SIZE as u32, 0x_21_21_21_21),
        ]);

        for t in &mut tests {
            // First Container and simple box.
            let mut l = LevelStack::new();
            println!("Empty stack: {:?}", l);

            l.add_box(t.remove(0));
            println!("After Box 1 stack: {:?}", l);
            assert_eq!(l.top().unwrap().size, 100);
            assert_eq!(l.top().unwrap().count, BOX_HEADER_SIZE);
            assert_eq!(l.len(), 1);

            l.add_box(t.remove(0));
            println!("After Box 2 stack: {:?}", l);
            assert_eq!(l.top().unwrap().size, 100);
            assert_eq!(l.top().unwrap().count, 100);

            assert_eq!(l.complete(), true);
            assert_eq!(l.len(), 1);
        }
    }

    #[test]
    fn test_level_stack_basic_moov() {
        let mut boxes = Vec::new();

        // Push the top box, then subtract off the
        // size of the headers for containers and
        // the size of whole box for non-containers.
        boxes.push(new_defined_box(300, MOOV));
        let mut size_accum = 300 - MOOV.header_size();

        boxes.push(new_defined_box(size_accum, UDTA));
        size_accum -= UDTA.header_size();

        boxes.push(new_defined_box(size_accum, META));
        size_accum -= META.header_size();

        boxes.push(new_defined_box(20, HDLR));
        size_accum -= 20;

        boxes.push(new_defined_box(size_accum, ILST));
        size_accum -= ILST.header_size();

        boxes.push(new_defined_box(size_accum, TRKN));
        size_accum -= TRKN.header_size();

        // This box needs to pick up the slack for the original
        // size so it is the entire rest of the origianl top box.
        boxes.push(new_defined_box(size_accum, DATA));

        println!("Vector of boxes to push {:?}", boxes);

        // Add them all to the stack.
        let mut l = LevelStack::new();
        for b in boxes {
            l.add_box(b);
        }

        // Check that we've got one box counter
        // for each container.
        assert_eq!(l.len(), 5);
        assert_eq!(l.path_string(), "/moov/udta/meta/ilst/trkn");

        // The top container should be "filled" now.
        // So we shold be able to pop all of the boxes off
        // the stack.
        l.check_and_complete();
        assert!(l.is_empty());
    }

    // Box builders
    fn new_defined_box(s: usize, bt: BoxType) -> MP4Box<'static> {
        MP4Box {
            size: s as u32,
            buf: &[0],
            box_type: bt,
            version_flag: None,
        }
    }

    fn new_simple_box(s: u32, id: u32) -> MP4Box<'static> {
        new_empty_box(s, id, ContainerType::NotContainer, false)
    }

    fn new_full_box(s: u32, id: u32) -> MP4Box<'static> {
        new_empty_box(s, id, ContainerType::NotContainer, true)
    }

    fn new_container(s: u32, id: u32) -> MP4Box<'static> {
        new_empty_box(s, id, ContainerType::Container, false)
    }

    fn new_empty_box(s: u32, id: u32, ct: ContainerType, fl: bool) -> MP4Box<'static> {
        MP4Box {
            size: s,
            box_type: BoxType::Unknown(BoxSpec {
                bt_id: id,
                container: ct,
                full: fl,
            }),
            buf: &[0],
            version_flag: None,
        }
    }
}
