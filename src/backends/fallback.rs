//! An implementation which calls out to an externally defined function if no other is provided.

#[cfg(getrandom_no_custom_fallback)]
compile_error! {
    "getrandom does not have a suitable backend for this target, requiring the use of a fallback.\
    However, `getrandom_no_custom_fallback` has been set, indicating the use of a fallback is prohibited.\
    Consider providing a `custom` backend."
}

use crate::Error;
use core::mem::MaybeUninit;

pub struct Implementation;

unsafe impl crate::Backend for Implementation {
    #[inline]
    fn fill_uninit(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
        extern "Rust" {
            fn __getrandom_v03_fallback_fill_uninit(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error>;
        }
        unsafe { __getrandom_v03_fallback_fill_uninit(dest) }
    }

    #[inline]
    fn u32() -> Result<u32, Error> {
        extern "Rust" {
            fn __getrandom_v03_fallback_u32() -> Result<u32, Error>;
        }
        unsafe { __getrandom_v03_fallback_u32() }
    }

    #[inline]
    fn u64() -> Result<u64, Error> {
        extern "Rust" {
            fn __getrandom_v03_fallback_u64() -> Result<u64, Error>;
        }
        unsafe { __getrandom_v03_fallback_u64() }
    }
}
