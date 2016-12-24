use Lock;

use std::sync::atomic::{AtomicUsize, Ordering};

pub struct TicketLock {
    next: AtomicUsize,
    owner: AtomicUsize,
}

impl TicketLock {
    pub fn new() -> TicketLock {
        return TicketLock {
            next: AtomicUsize::new(0),
            owner: AtomicUsize::new(0),
        }
    }
}

impl Lock for TicketLock {
    fn lock(&mut self) {
        let next = self.next.fetch_add(1, Ordering::Release);
        while next < self.owner.load(Ordering::Acquire) { }
    }

    fn unlock(&mut self) {
        self.owner.fetch_add(1, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::super::Lock;
    use super::TicketLock;
    #[test]
    fn lock_unlock() {
        let mut tl = TicketLock::new();
        tl.lock();
        tl.unlock();
        tl.lock();
        tl.unlock();
    }
}
