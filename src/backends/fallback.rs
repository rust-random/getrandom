//! An implementation which calls out to an externally defined function if no other is provided.

#[cfg(getrandom_no_external_fallback)]
compile_error! {
    "getrandom does not have a suitable backend for this target, requiring the use of a fallback.\
    However, `getrandom_no_external_fallback` has been set, indicating the use of a fallback is prohibited.\
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

/// Sets the fallback [`Backend`](crate::Backend).
/// 
/// # Examples
/// 
/// ```ignore
/// struct MyBackend;
/// 
/// impl Backend for MyBackend { /* ... */ }
/// 
/// set_backend!(MyBackend);
/// ```
#[macro_export]
macro_rules! set_backend {
    ($t: ty) => {
        #[no_mangle]
        extern "Rust" fn __getrandom_v03_fallback_fill_uninit(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
            <$t as $crate::Backend>::fill_uninit(dest)
        }

        #[no_mangle]
        extern "Rust" fn __getrandom_v03_fallback_u32() -> Result<u32, Error> {
            <$t as $crate::Backend>::u32()
        }

        #[no_mangle]
        extern "Rust" fn __getrandom_v03_fallback_u64() -> Result<u64, Error> {
            <$t as $crate::Backend>::u64()
        }
    };
}