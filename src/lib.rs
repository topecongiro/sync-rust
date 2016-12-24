#![feature(const_fn)]
#![feature(integer_atomics)]
//#![feature(field_init_shorthand)]
pub mod ticket_lock;
pub mod spin_lock;
pub mod msc;
pub mod clh;
pub mod util;
pub mod k42;
pub mod flag;
pub mod src_barrier;
pub mod rw_lock;
pub mod rw_lock_w;
pub mod rw_lock_fair;

pub trait Lock {
    fn lock(&mut self);
    fn unlock(&mut self);
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
