use std::ptr;
use std::sync::atomic::fence;
use std::sync::atomic::AtomicPtr;
use std::sync::atomic::Ordering::{Acquire, Relaxed, AcqRel};

pub struct Node {
    prev: AtomicPtr<Node>,
    succ_must_wait: bool,
}

const DUMMY: Node = Node {
    prev: AtomicPtr::new(ptr::null_mut()),
    succ_must_wait: false,
};

pub struct CLH {
    tail: AtomicPtr<Node>,
}

impl Node {
    pub fn new() -> Node {
        Node {
            prev: AtomicPtr::new(ptr::null_mut()),
            succ_must_wait: false,
        }
    }
}


impl CLH {
    pub fn new() -> CLH {
        CLH {
            tail: AtomicPtr::new(&mut DUMMY),
        }
    }

    pub fn lock(&mut self, p: &mut Node) {
        p.succ_must_wait = true;
        let pred = self.tail.swap(p, AcqRel);
        p.prev = AtomicPtr::new(pred);
        while unsafe { (*pred).succ_must_wait } {
        }
        fence(Acquire);
    }

    pub fn unlock(&mut self, p: &mut Node) {
        let raw_p = p as *mut Node;
        let ref pred = unsafe { ptr::read(p.prev.load(Relaxed)) };
        p.succ_must_wait = false;
        unsafe { ptr::write(raw_p, ptr::read(pred)); }
    }
}

#[cfg(test)]
mod tests {
    use super::CLH;
    use super::Node;
    #[test]
    fn test_lock_unlock() {
        let mut clh = CLH::new();
        let mut p = Node::new();
        clh.lock(&mut p);
        clh.unlock(&mut p);
    }
}
