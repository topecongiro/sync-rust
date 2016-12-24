use std::thread;
use std::time::Duration;
use std::ops::MulAssign;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::{Release, Acquire};

pub struct RWLock {
    n: AtomicUsize,
}

const WA_FLAG: usize = 1;
const RC_INC: usize = 2;
const BASE: u32 = 1;
// const LIMIT: u32 = 1;
const MULTIPLIER: u32 = 1;

impl RWLock {
    pub fn new() -> RWLock {
        RWLock {
            n: AtomicUsize::new(0),
        }
    }

    // This function is not correct, but given as is for the proof-of-concept.
    pub fn wlock(&mut self) {
        let mut delay = Duration::new(0, BASE);
        while self.n.compare_and_swap(0, WA_FLAG, Acquire) != self.n.load(Acquire) {
            thread::sleep(delay);
            delay.mul_assign(MULTIPLIER);
        }
    }

    pub fn wunlock(&mut self) {
        self.n.fetch_add(!WA_FLAG, Release);
    }

    pub fn rlock(&mut self) {
        self.n.fetch_add(RC_INC, Acquire);
        while self.n.load(Acquire) & WA_FLAG == 1 {
        }
    }

    pub fn runlock(&mut self) {
        self.n.fetch_add(!RC_INC, Release);
    }
}
