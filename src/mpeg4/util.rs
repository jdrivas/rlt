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

    // Add a box to the stack
    pub fn add_box(&mut self, b: &boxes::MP4Box) {
        self.top().unwrap().count += b.size as usize;
        if b.box_type.is_container() {
            self.push_new(b.buf.len(), b.kind);
        }
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

    // Have we all of the box.
    pub fn end(&self) -> bool {
        self.levels.last().unwrap().size == self.levels.last().unwrap().count
    }

    pub fn check_and_finish(&mut self, mut f: impl FnMut()) {
        while self.end() {
            f();
            self.pop();
            if self.len() == 0 {
                break;
            }
        }
    }

    // Take the top box off the stack
    pub fn pop(&mut self) -> Option<BoxCounter> {
        self.levels.pop()
    }

    // Get the box top box from the stack
    pub fn top(&mut self) -> Option<&mut BoxCounter> {
        self.levels.last_mut()
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
        self.path()
            .into_iter()
            .map(|v| String::from_utf8_lossy(v).into_owned())
            .collect::<Vec<String>>()
            .join("/")
    }
}
