//! An implementation which calls out to an externally defined function.
use crate::Error;
use core::mem::MaybeUninit;

pub use crate::default_impls::{insecure_fill_uninit, insecure_u32, insecure_u64, u32, u64};

pub fn fill_uninit(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    extern "Rust" {
        fn __getrandom_v03_custom(dest: *mut u8, len: usize) -> Result<(), Error>;
    }
    unsafe { __getrandom_v03_custom(dest.as_mut_ptr().cast(), dest.len()) }
}
