use std::thread::sleep;
use std::time::Duration;
use std::ops::MulAssign;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::{Acquire, Release, Relaxed};
use std::sync::atomic::fence;

// Writer-preference rwlock.
pub struct RWW {
    // High haf of word counts active readers (ar).
    // Low half of word except the lsb counts waiting writers (ww).
    // The least significant bit indicates the active writer (aw).
    counter: AtomicU64,
}

// Parameters for exponential backoff.
const BASE: u32 = 1;
const LIMIT: u32 = 1;
const MULT: u32 = 1;

const WW_MASK: u64 = (1 << 32) - 1;
const AR_MASK: u64 = WW_MASK << 32;

#[inline]
fn from_counter(counter: u64) -> (u32, u32, bool) {
    ((counter & AR_MASK >> 32) as u32,
     (counter & WW_MASK) as u32,
     counter & 1 == 1)
}

#[inline]
fn to_counter(ar: u32, ww: u32, aw: bool) -> u64 {
    let new_ar: u64 = (ar as u64) << 32;
    let new_ww: u64 = ww as u64;
    let new_aw: u64 = if aw { 1 } else { 0 };
    new_ar + new_ww + new_aw
}

impl RWW {
    pub fn new() -> RWW {
        RWW {
            counter: AtomicU64::new(0),
        }
    }

    pub fn w_lock(&mut self) {
        let delay = BASE;
        let mut delay_duration = Duration::new(0, delay);
        loop {
            let mut counter = self.counter.load(Relaxed);

            let (ar, ww, aw) = from_counter(counter);
            // No active writer nor readers
            if !aw && ar == 0 {
                if self.counter.compare_and_swap(counter, to_counter(ar, ww, true), Relaxed) == counter {
                    break
                }
            } else if self.counter.compare_and_swap(counter, to_counter(ar, ww+1, aw), Relaxed) == counter {

                // Registered as waiting writer.
                loop {
                    counter = self.counter.load(Relaxed);
                    let (ar, ww, aw) = from_counter(counter);
                    // No active writer nor readers
                    if !aw && ar == 0 {
                        if self.counter.compare_and_swap(counter, to_counter(ar, ww-1, true), Relaxed) == counter {
                            fence(Acquire);
                            return
                        }

                        // Exponential backoff
                        sleep(delay_duration);
                        if delay * MULT > LIMIT {
                            delay_duration = Duration::new(0, LIMIT);
                        } else {
                            delay_duration.mul_assign(MULT);
                        }
                    }
                }
            }
        }

        // Make sure that the lock is properly acquired and it is
        // visible to other threads.
        fence(Acquire);
    }

    pub fn r_lock(&mut self) {
        loop {
            let counter = self.counter.load(Relaxed);
            let (ar, ww, aw) = from_counter(counter);
            if ww == 0 && !aw {
                if self.counter.compare_and_swap(counter, to_counter(ar+1,0,false), Relaxed) == counter {
                    break
                }
                sleep(Duration::new(0, ww * BASE));
            }
        }

        fence(Acquire)
    }

    pub fn w_unlock(&mut self) {
        fence(Release);

        loop {
            let counter = self.counter.load(Relaxed);
            let (ar, ww, _) = from_counter(counter);
            if self.counter.compare_and_swap(counter, to_counter(ar, ww, false), Relaxed) == counter {
                break
            }
        }
    }

    pub fn r_unlock(&mut self) {
        fence(Release);

        loop {
            let counter = self.counter.load(Relaxed);
            let (ar, ww, aw) = from_counter(counter);
            if self.counter.compare_and_swap(counter, to_counter(ar-1, ww, aw), Relaxed) == counter {
                break
            }
        }
    }
}
