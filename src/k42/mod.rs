use std::ptr;
use std::sync::atomic::fence;
use std::sync::atomic::AtomicPtr;
use std::sync::atomic::Ordering::{Release, Acquire, Relaxed};

use Lock;

// You must use nightly build and add #![feature(const_fn)] to allow
// function call in const declaration.
const WAITING: AtomicPtr<Node> = AtomicPtr::new(ptr::null_mut());

struct Node {
    tail: AtomicPtr<Node>,
    next: AtomicPtr<Node>,
}

pub struct K42 {
    q: Node,
}

impl K42 {
    pub fn new() -> K42 {
        K42 {
           q: Node {
               tail: AtomicPtr::new(ptr::null_mut()),
               next: AtomicPtr::new(ptr::null_mut()),
           }
        }
    }
}

impl Lock for K42 {
    fn lock(&mut self) {
        loop {
            let prev = self.q.tail.load(Relaxed);
            if prev.is_null() {
                let old_q: *mut Node = &mut self.q;
                if self.q.tail.compare_and_swap(ptr::null_mut(), old_q, Relaxed) == ptr::null_mut() {
                    break;
                }
            } else {
                let mut n = Node {
                    tail: WAITING,
                    next: AtomicPtr::new(ptr::null_mut()),
                };
                if self.q.tail.compare_and_swap(prev, &mut n, Acquire) == prev {
                    unsafe { (*prev).next.store(&mut n, Relaxed) }
                    while n.tail.load(Relaxed) == WAITING.into_inner() {
                    }
                    let mut succ = n.next.load(Relaxed);
                    if succ.is_null() {
                        self.q.next.store(ptr::null_mut(), Relaxed);
                        let old_q: *mut Node = &mut self.q;
                        if self.q.tail.compare_and_swap(&mut n, old_q, Relaxed) != &mut n {
                            while succ.is_null() {
                                succ = n.next.load(Relaxed);
                            }
                            self.q.next.store(succ, Relaxed);
                        }
                        break;
                    } else {
                        self.q.next.store(succ, Relaxed);
                        break;
                    }
                }
            }
        }
    }

    fn unlock(&mut self) {
        fence(Release);
        let mut succ = self.q.next.load(Relaxed);
        if succ.is_null() {
            let old_q: *mut Node = &mut self.q;
            if self.q.tail.compare_and_swap(old_q, ptr::null_mut(), Relaxed) == old_q {
                return
            }
            while succ.is_null() {
                succ = self.q.next.load(Relaxed);
            }
        }
        unsafe { (*succ).tail.store(ptr::null_mut(), Relaxed) }
    }
}
