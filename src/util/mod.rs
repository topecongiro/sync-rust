use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

pub trait AtomicPrimitive {
    fn test_and_set(&mut self) -> bool;
}

impl AtomicPrimitive for AtomicBool {
    fn test_and_set(&mut self) -> bool {
        return self.swap(true, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicBool;
    use util::AtomicPrimitive;

    #[test]
    fn test_tas() {
        let mut t = AtomicBool::new(true);
        assert!(t.test_and_set());
    }
}
