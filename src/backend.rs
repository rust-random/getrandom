//! An implementation which calls out to an externally defined function.
use crate::Error;
use core::{mem::MaybeUninit, slice};

/// Uses the provided [`Backend`].
/// This macro must be called exactly once, otherwise the final linking will fail due to either
/// duplicated or missing symbols.
#[macro_export]
macro_rules! set_backend {
    ($t: ty) => {
        const _: () = {
            #[inline]
            #[no_mangle]
            unsafe fn __getrandom_v04_backend_fill_ptr(
                dest: *mut u8,
                len: usize,
            ) -> Result<(), $crate::Error> {
                <$t as $crate::Backend>::fill_ptr(dest, len)
            }

            #[inline]
            #[no_mangle]
            unsafe fn __getrandom_v04_backend_fill_uninit(
                dest: &mut [core::mem::MaybeUninit<u8>],
            ) -> Result<(), $crate::Error> {
                <$t as $crate::Backend>::fill_uninit(dest)
            }

            #[inline]
            #[no_mangle]
            unsafe fn __getrandom_v04_backend_fill(dest: &mut [u8]) -> Result<(), $crate::Error> {
                <$t as $crate::Backend>::fill(dest)
            }

            #[inline]
            #[no_mangle]
            unsafe fn __getrandom_v04_backend_u32() -> Result<u32, $crate::Error> {
                <$t as $crate::Backend>::u32()
            }

            #[inline]
            #[no_mangle]
            unsafe fn __getrandom_v04_backend_u64() -> Result<u64, $crate::Error> {
                <$t as $crate::Backend>::u64()
            }

            #[inline]
            #[no_mangle]
            unsafe fn __getrandom_v04_backend_describe_custom_error(
                n: u16,
            ) -> Option<&'static str> {
                <$t as $crate::Backend>::describe_custom_error(n)
            }
        };
    };
}

/// Describes how `getrandom` can collect random values from a particular backend.
///
/// Implementers can pair this with [`set_backend`] to always use their [`Backend`],
/// or allow users to call it themselves if that's more appropriate.
///
/// # Safety
///
/// The implementation of this trait must produce sufficiently randomized values.
pub unsafe trait Backend {
    /// Writes `len` random values starting at `dest`.
    ///
    /// # Safety
    ///
    /// - `dest` must be a valid pointer at least `len` bytes long.
    unsafe fn fill_ptr(dest: *mut u8, len: usize) -> Result<(), Error>;

    /// Fill a slice of [`MaybeUninit`] bytes with random values.
    #[inline]
    fn fill_uninit(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
        let ptr = dest.as_mut_ptr().cast();
        let len = dest.len();

        // SAFETY: `ptr` is valid and exactly `len` bytes in size
        unsafe { Self::fill_ptr(ptr, len) }
    }

    /// Fill a slice of bytes with random values.
    #[inline]
    fn fill(dest: &mut [u8]) -> Result<(), Error> {
        // SAFETY: The `&mut MaybeUninit<_>` reference doesn't escape,
        // and `fill_uninit` guarantees it will never de-initialize
        // any part of `dest`.
        Self::fill_uninit(unsafe { crate::util::slice_as_uninit_mut(dest) })?;
        Ok(())
    }

    /// Generates a single random [`u32`] value.
    #[inline]
    fn u32() -> Result<u32, Error> {
        let mut res = MaybeUninit::<u32>::uninit();
        // SAFETY: the created slice has the same size as `res`
        let dst = unsafe {
            let p: *mut MaybeUninit<u8> = res.as_mut_ptr().cast();
            slice::from_raw_parts_mut(p, core::mem::size_of::<u32>())
        };
        Self::fill_uninit(dst)?;
        // SAFETY: `dst` has been fully initialized by `imp::fill_inner`
        // since it returned `Ok`.
        Ok(unsafe { res.assume_init() })
    }

    /// Generates a single random [`u64`] value.
    #[inline]
    fn u64() -> Result<u64, Error> {
        let mut res = MaybeUninit::<u64>::uninit();
        // SAFETY: the created slice has the same size as `res`
        let dst = unsafe {
            let p: *mut MaybeUninit<u8> = res.as_mut_ptr().cast();
            slice::from_raw_parts_mut(p, core::mem::size_of::<u64>())
        };
        Self::fill_uninit(dst)?;
        // SAFETY: `dst` has been fully initialized by `imp::fill_inner`
        // since it returned `Ok`.
        Ok(unsafe { res.assume_init() })
    }

    /// Describes a custom [`Error`] code reported by this [`Backend`].
    #[inline]
    #[expect(unused_variables)]
    fn describe_custom_error(n: u16) -> Option<&'static str> {
        None
    }
}

/// An implementation of [`Backend`] that relies on some other external implementation, paired
/// with a call to [`set_backend`].
pub(crate) struct ExternBackend;

unsafe impl Backend for ExternBackend {
    #[inline]
    unsafe fn fill_ptr(dest: *mut u8, len: usize) -> Result<(), Error> {
        extern "Rust" {
            fn __getrandom_v04_backend_fill_ptr(dest: *mut u8, len: usize) -> Result<(), Error>;
        }
        __getrandom_v04_backend_fill_ptr(dest, len)
    }

    #[inline]
    fn fill_uninit(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
        extern "Rust" {
            fn __getrandom_v04_backend_fill_uninit(
                dest: &mut [MaybeUninit<u8>],
            ) -> Result<(), Error>;
        }
        unsafe { __getrandom_v04_backend_fill_uninit(dest) }
    }

    #[inline]
    fn fill(dest: &mut [u8]) -> Result<(), Error> {
        extern "Rust" {
            fn __getrandom_v04_backend_fill(dest: &mut [u8]) -> Result<(), Error>;
        }
        unsafe { __getrandom_v04_backend_fill(dest) }
    }

    #[inline]
    fn u32() -> Result<u32, Error> {
        extern "Rust" {
            fn __getrandom_v04_backend_u32() -> Result<u32, Error>;
        }
        unsafe { __getrandom_v04_backend_u32() }
    }

    #[inline]
    fn u64() -> Result<u64, Error> {
        extern "Rust" {
            fn __getrandom_v04_backend_u64() -> Result<u64, Error>;
        }
        unsafe { __getrandom_v04_backend_u64() }
    }

    #[inline]
    fn describe_custom_error(n: u16) -> Option<&'static str> {
        extern "Rust" {
            fn __getrandom_v04_backend_describe_custom_error(n: u16) -> Option<&'static str>;
        }
        unsafe { __getrandom_v04_backend_describe_custom_error(n) }
    }
}
