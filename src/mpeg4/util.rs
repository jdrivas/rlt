use super::boxes;
use box_types::{BoxSpec, BoxType, ContainerType};
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
            self.box_type.code_string(),
            self.size,
            self.count
        )
    }
}

/// A stack of BoxCounters used to capture containers and when they've been filled.
/// This particularly useful for understanding the context in which a box has been
/// found (ie the path to the box) as well as for doing things like
#[derive(Debug)]
pub struct LevelStack {
    levels: Vec<BoxCounter>,
}

impl LevelStack {
    // pub fn new(size: usize) -> LevelStack {
    pub fn new() -> LevelStack {
        LevelStack { levels: vec![] }
    }

    /// Add a box and calcuate it's impact on the stack
    /// 1. We increment the count against the current container
    /// by the size of the new box. This box will take up
    /// so many bytes of against the total in the enclosing container.
    /// 2. If this is a container, add this box to the stack to account
    /// for comming enclosed boxes.
    pub fn add_box(&mut self, b: boxes::MP4Box) {
        if self.levels.len() > 0 {
            self.levels.last_mut().unwrap().count += b.size as usize;
        }

        if b.box_type.spec().container != ContainerType::NotContainer {
            self.push_new(b.buf.len(), b.box_type);
        }
        // match b.box_type {
        // BoxType::Unknown(t) => self.push_new(
        //     b.buf.len(),
        //     BoxType::Unknown(BoxSpec {
        //         bt_id: t,
        //         container: ContainerType::NotContainer,
        //         full: false,
        //     }),
        // ),
        // _ => {
        //     if let Some(spec) = &b.box_type.spec() {
        //         if spec.container != ContainerType::NotContainer {
        //             self.push_new(b.buf.len(), b.box_type);
        //         }
        //     }
        // }
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

    fn push_new(&mut self, size: usize, bt: BoxType) {
        self.levels.push(BoxCounter {
            size: size,
            count: 0,
            box_type: bt,
        });
    }

    /// Determine if container at the top of the stack
    /// has been completed, and if so remove the box from
    /// the stack. Continue to remove compledted containers
    // /until you find one that is not complete or you exhause the
    /// stack.
    // pub fn check_and_complete(&'a mut self) {
    //     self.check_and_complete_with(|_| {}); // TODO(jdr): This really wants an Option to a closure.
    // }

    /// Check if the top container is complete, and if so remove and continue checking,
    /// with a convenience for executing a closure
    /// once for each container to be removed from the stack and so,
    /// logically, when the container has been completed.
    /// Useful for managing indentation in a display, for example.
    // pub fn check_and_complete_with('a &mut self, mut f: impl FnMut(&'a LevelStack)) {
    //     while self.complete() {
    //         f(self);
    //         self.pop();
    //         if self.len() == 0 {
    //             break;
    //         }
    //     }
    // }

    /// Convenience function to add a new box and immediately
    /// check for completion.
    // pub fn update(&'a mut self, b: &'a boxes::MP4Box) {
    //     self.add_box(b);
    //     self.check_and_complete();
    // }

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
        if self.levels.len() > 0 {
            self.levels.last().unwrap().size == self.levels.last().unwrap().count
        } else {
            false
        }
    }

    /// Take the top box off the stack.
    pub fn pop(&mut self) -> Option<BoxCounter> {
        self.levels.pop()
    }

    /// Get the top box from the stack as a mutable reference.
    // pub fn top_mut(&mut self) -> Option<&mut BoxCounter> {
    //     self.levels.last_mut()
    // }

    /// Get the top box from the stack.
    pub fn top(&self) -> Option<&BoxCounter> {
        self.levels.last()
    }

    /// How many boxes on the stack.
    pub fn len(&self) -> usize {
        self.levels.len()
    }

    /// What's the path to the current top box.
    pub fn path(&self) -> Vec<String> {
        let mut v = Vec::new();
        for l in &self.levels {
            v.push(l.box_type.code_string());
        }
        v
    }

    /// A string representation of the path.
    /// Note: We remeove the STRT sentitnel at the head
    /// and replace it with a single '/'.
    /// so paths look like: /moov/trak/mdia/minf/stbl
    pub fn path_string(&self) -> String {
        let mut s = "/".to_string();
        if self.len() > 0 {
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
    pub fn new_with(c: char) -> Tabs {
        Tabs {
            t: String::new(),
            c: c,
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

/// Get a string from a [u8;4];
pub fn kind_to_string(k: &[u8; 4]) -> String {
    String::from_utf8_lossy(k).into_owned()
}

pub fn u8_to_string(k: &[u8]) -> String {
    String::from_utf8_lossy(k).into_owned()
}
