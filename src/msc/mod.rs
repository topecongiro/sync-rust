use std::ptr;
use std::sync::atomic::fence;
use std::sync::atomic::{AtomicPtr, AtomicBool};
use std::sync::atomic::Ordering::{Release, Acquire, Relaxed};

pub struct Node {
    next: AtomicPtr<Node>,
    waiting: AtomicBool,
}

pub struct MSC {
    tail: AtomicPtr<Node>,
}

impl Node {
    pub fn new() -> Node {
        Node {
            next: AtomicPtr::new(ptr::null_mut()),
            waiting: AtomicBool::new(false),
        }
    }
}

impl MSC {
    pub fn new() -> MSC {
        MSC {
            tail: AtomicPtr::new(ptr::null_mut()),
        }
    }

    pub fn lock(&mut self, p: &mut Node) {
        // Setup local node
        p.next = AtomicPtr::new(ptr::null_mut());
        p.waiting.store(true, Relaxed);

        // Place your local node at the tail of the queue
        let prev = self.tail.swap(p, Release);

        // If the queue is not empty, then the lock is acquired by other thread
        // That case, you must enqueue your local node next to the prev node,
        // and spinlock on your local node's waiting flag.
        // This is safe because self.tail.swap is atomic, so only one thread can
        // place her local node at other node's next field
        if !prev.is_null() {
            unsafe {(*prev).next.store(p, Relaxed); }
            while p.waiting.load(Relaxed) {
            }
        }

        // Make sure that previous read/write occurs before entering critical section.
        // It might be the case that I am misuing fence()?
        fence(Acquire);
    }

    pub fn unlock(&mut self, p: &mut Node) {
        fence(Release);

        // Get the waiting thread's node if available.
        let mut succ = p.next.load(Relaxed);

        if succ.is_null() {
            // Try to place the null node on tail.
            // If it succeeds, just return.
            // Usually, tail == p because this thread is acquiring the lock and there is no waiting thread (succ.is_null()).
            if self.tail.compare_and_swap(p, ptr::null_mut(), Relaxed) == p {
                return;
            }
            // If the above CAS failed, it means that there is other thread trying to acquire the lock right now.
            // That case, wait until his node is set to p.next.
            while succ.is_null() {
                succ = p.next.load(Relaxed);
            }
        }

        // Release the lock: let the next node acquire the lock.
        unsafe { (*succ).waiting.store(false, Relaxed); }
    }
}

#[cfg(test)]
mod tests {
    use super::MSC;
    use super::Node;

    #[test]
    fn lock_unlock() {
        let mut l = MSC::new();
        let mut p = Node::new();
        l.lock(&mut p);
        l.unlock(&mut p);
    }
}
