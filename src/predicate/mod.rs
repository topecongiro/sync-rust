use std::sync::atomic::fence;
use std::sync::atomic::Ordering::SeqCst;

pub struct Predicate {
    eval: fn () -> bool,
}

impl Predicate {
    pub fn new(pred: fn () -> bool) -> Predicate {
        Predicate {
            eval: pred,
        }
    }

    pub fn await(&self) {
        while (self.eval()) {
        }
        fence(SeqCst);
    }
}
