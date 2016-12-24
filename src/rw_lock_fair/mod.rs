use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::{Acquire, Release, Relaxed};
use std::sync::atomic::fence;
use std::thread::sleep;
use std::time::Duration;

pub struct RWFair {
    requests: AtomicU64,
    completions: AtomicU64,
}

const BASE: u32 = 1;
const LOW_MASK: u64 = 1 << 32 - 1;
const HIGH_MASK: u64 = !LOW_MASK;

#[inline]
fn from_u64(x: u64) -> (u32, u32) {
    ((x & HIGH_MASK >> 32) as u32, (x & LOW_MASK) as u32)
}

#[inline]
fn to_u64(h: u32, l: u32) -> u64 {
    (h as u64) << 32 + l as u64
}

impl RWFair {
    pub fn new() -> RWFair {
        RWFair {
            requests: AtomicU64::new(0),
            completions: AtomicU64::new(0),
        }
    }

    pub fn w_lock(&mut self) {
        let (mut rr, mut wr) = from_u64(self.requests.load(Relaxed));
        while self.requests.compare_and_swap(to_u64(rr, wr), (to_u64(rr, wr+1)), Relaxed) == to_u64(rr, wr) {
            let (_rr, _wr) = from_u64(self.requests.load(Relaxed));
            rr = _rr;
            wr = _wr;
        }

        loop {
            let (rc, wc) = from_u64(self.completions.load(Relaxed));
            if rc == rr && wc == wr {
                break
            }
            sleep(Duration::new(0, (wc - wr) * BASE));
        }

        fence(Acquire);
    }

    pub fn r_lock(&mut self) {
        let (mut rr, mut wr) = from_u64(self.requests.load(Relaxed));
        loop {
            if self.requests.compare_and_swap(to_u64(rr,wr), to_u64(rr+1,wr), Relaxed) == to_u64(rr,wr) {
                break
            }
            let (_rr, _wr) = from_u64(self.requests.load(Relaxed));
            rr = _rr;
            wr = _wr;
        }

        loop {
            let (_, wc) = from_u64(self.completions.load(Relaxed));
            if wc == wr {
                break
            }
            sleep(Duration::new(0, (wc - wr) * BASE));
        }

        fence(Acquire);
    }

    pub fn w_ulock(&mut self) {
        fence(Release);
        loop {
            let (rc, wc) = from_u64(self.completions.load(Relaxed));
            if (self.completions.compare_and_swap(to_u64(rc, wc), to_u64(rc, wc+1), Relaxed)) == to_u64(rc, wc) {
                break
            }
        }
    }

    pub fn r_ulock(&mut self) {
        fence(Release);
        loop {
            let (rc, wc) = from_u64(self.completions.load(Relaxed));
            if self.completions.compare_and_swap(to_u64(rc,wc), to_u64(rc+1, wc), Relaxed) == to_u64(rc, wc) {
                break
            }
        }
    }
}
