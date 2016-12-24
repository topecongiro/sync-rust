use Lock;

use std::sync::atomic::{AtomicBool, Ordering};

pub struct SpinLock {
    wait: AtomicBool,
}

impl SpinLock {
    pub fn new() -> SpinLock {
        return SpinLock {
            wait: AtomicBool::new(false),
        }
    }
}

impl Lock for SpinLock {
    fn lock(&mut self) {
        while self.wait.compare_and_swap(false, true, Ordering::Relaxed) {}
    }

    fn unlock(&mut self) {
        self.wait.store(false, Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn simple_lock_unlock() {
        use Lock;
        use super::SpinLock;

        let mut lock = SpinLock::new();
        lock.lock();
        lock.unlock();
    }
}
