//! An implementation which calls out to an externally defined function.
use crate::Error;
use core::{mem::MaybeUninit, num::NonZeroU32};

pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    extern "Rust" {
        fn __getrandom_custom(dest: *mut u8, len: usize) -> u32;
    }
    let ret = unsafe { __getrandom_custom(dest.as_mut_ptr().cast(), dest.len()) };
    match NonZeroU32::new(ret) {
        None => Ok(()),
        Some(code) => Err(Error::from(code)),
    }
}
