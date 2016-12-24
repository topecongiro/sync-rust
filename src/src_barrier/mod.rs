use std::sync::atomic::{AtomicUsize, AtomicBool};
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::sync::atomic::fence;

pub struct Barrier {
    count: AtomicUsize,
    n: usize,
    sense: AtomicBool,
    local_sense: Vec<bool>,
}

impl Barrier {
    pub fn new(n_thread: usize) -> Barrier {
        Barrier {
            count: AtomicUsize::new(0),
            n: n_thread,
            sense: AtomicBool::new(true),
            local_sense: Vec::with_capacity(n_thread),
        }
    }

    pub fn cycle(&mut self, id: usize) {
        let s = !self.local_sense[id];
        self.local_sense[id] = s;
        if self.count.fetch_add(1, Release) == self.n - 1 {
            self.count.store(0, Relaxed);
            self.sense.store(s, Relaxed);
        } else {
            // spin
            while self.sense.load(Relaxed) != s {
            }
        }
        fence(Acquire);
    }
}
