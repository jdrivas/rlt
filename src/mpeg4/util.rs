use super::boxes;
use std::fmt;

// This got built because doing the various jobs with recursion
// got me into lots of fights with the borrow checker.
// So I just decieded that this was an easier path.
#[derive(Clone, Copy)]
pub struct BoxCounter {
    pub size: usize,
    pub count: usize,
    pub kind: [u8; 4],
}

impl fmt::Debug for BoxCounter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?} - Size: {}, Count: {}",
            String::from_utf8_lossy(&self.kind),
            self.size,
            self.count
        )
    }
}

#[derive(Debug)]
pub struct LevelStack {
    levels: Vec<BoxCounter>,
}

impl LevelStack {
    pub fn new(size: usize) -> LevelStack {
        let mut l = LevelStack { levels: vec![] };
        l.push_new(size, b"STRT");
        l
    }

    // Add a box and calcuate it's impact on the stack
    // 1. We increment the count against the current container
    // by the size of the new box. This box will take up
    // so many bytes of against the total in the enclosing container.
    // 2. If this is a container, add this box to the stack to account
    // for comming enclosed boxes.
    pub fn add_box(&mut self, b: &boxes::MP4Box) {
        self.top_mut().unwrap().count += b.size as usize;
        if b.box_type.is_container() {
            self.push_new(b.buf.len(), b.kind);
        }
    }

    // Adds the box and the runs the closure provdied.
    // Useful for example to increae the tab length with the box is a conatiner.
    // The closure is called with this LevelStack and the box that was passed in.
    pub fn add_box_with(
        &mut self,
        b: &boxes::MP4Box,
        mut f: impl FnMut(&LevelStack, &boxes::MP4Box),
    ) {
        self.add_box(b);
        f(self, b);
    }

    // Determine if container at the top of the stack
    // has been completed, and if so remove the box from
    // the stack. Continue to remove compledted containers
    // until you find one that is not complete or you exhause the
    // stack.
    pub fn check_and_complete(&mut self) {
        self.check_and_complete_with(|_| {}); // TODO(jdr): This really wants an Option to a closure.
    }

    // Check if the top container is complete, and if so remove and continue checking,
    // with a convenience for executing a closure
    // once for each container to be removed from the stack and so,
    // logically, when the container has been completed.
    // Useful for managing indentation in a display, for example.
    pub fn check_and_complete_with(&mut self, mut f: impl FnMut(&LevelStack)) {
        while self.complete() {
            f(self);
            self.pop();
            if self.len() == 0 {
                break;
            }
        }
    }

    /// Convenience function to add a new box and immediately
    /// check for completion.
    pub fn update(&mut self, b: &boxes::MP4Box) {
        self.add_box(b);
        self.check_and_complete();
    }

    /// Convenience with closures used as in add_with, and check_and_complete_with.
    pub fn update_with(
        &mut self,
        b: &boxes::MP4Box,
        a: impl FnMut(&LevelStack, &boxes::MP4Box),
        c: impl FnMut(&LevelStack),
    ) {
        self.add_box_with(b, a);
        self.check_and_complete_with(c);
    }

    pub fn push_new(&mut self, size: usize, kind: &[u8]) {
        let mut k: [u8; 4] = [0; 4];
        k.copy_from_slice(kind);
        self.levels.push(BoxCounter {
            size: size,
            count: 0,
            kind: k,
        });
    }

    /// Has the container on the top of the stack been completed?
    /// Practically this means
    pub fn complete(&self) -> bool {
        self.levels.last().unwrap().size == self.levels.last().unwrap().count
    }

    // Take the top box off the stack
    pub fn pop(&mut self) -> Option<BoxCounter> {
        self.levels.pop()
    }

    // Get the box top box from the stack
    pub fn top_mut(&mut self) -> Option<&mut BoxCounter> {
        self.levels.last_mut()
    }

    pub fn top(&self) -> Option<&BoxCounter> {
        self.levels.last()
    }

    // How many boxes on the stack.
    pub fn len(&self) -> usize {
        self.levels.len()
    }

    // What's the path to the current top box.
    pub fn path(&self) -> Vec<&[u8]> {
        let mut v = Vec::new();
        for l in &self.levels {
            v.push(&l.kind[..]);
        }
        v
    }

    // A string representation of the path.
    pub fn path_string(&self) -> String {
        let mut s = "/".to_string();
        if self.len() > 0 {
            s += &self.path()[1..] // skip the start marker.
                .into_iter()
                .map(|v| String::from_utf8_lossy(v).into_owned())
                .collect::<Vec<String>>()
                .join("/")
        }
        s
    }
}
