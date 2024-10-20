//! An implementation which calls out to an externally defined function.
use crate::Error;
use core::mem::MaybeUninit;

pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    extern "Rust" {
        fn __getrandom_v03_custom(dest: *mut u8, len: usize) -> Result<(), Error>;
    }
    unsafe { __getrandom_v03_custom(dest.as_mut_ptr().cast(), dest.len()) }
}
