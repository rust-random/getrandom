//! An implementation which calls out to an externally defined function.
use crate::Error;
use core::mem::MaybeUninit;

pub struct Implementation;

unsafe impl crate::Backend for Implementation {
    #[inline]
    fn fill_uninit(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
        extern "Rust" {
            fn __getrandom_v03_custom(dest: *mut u8, len: usize) -> Result<(), Error>;
        }
        unsafe { __getrandom_v03_custom(dest.as_mut_ptr().cast(), dest.len()) }
    }
}
