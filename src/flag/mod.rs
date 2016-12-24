use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Release};

pub struct Flag {
    f: AtomicBool,
}

impl Flag {
    pub fn new() -> Flag {
        Flag {
            f: AtomicBool::new(false),

        }
    }

    pub fn set(&mut self) {
        self.f.store(true, Release)
    }

    pub fn await(&self) {
        while !self.f.load(Acquire) {
        }
    }

    // Acquire ordering is used to ensure that any subsequent updates are seen
    // to happen after the reset().
    pub fn reest(&mut self) {
        self.f.store(false, Acquire)
    }
}
