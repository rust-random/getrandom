#![allow(dead_code)]
use crate::Error;
use core::{mem::MaybeUninit, slice};

#[inline(always)]
#[allow(unused_unsafe)]
unsafe fn default_impl<T>(secure: bool) -> Result<T, Error> {
    let mut res = MaybeUninit::<T>::uninit();
    // SAFETY: the created slice has the same size as `res`
    let dst = unsafe {
        let p: *mut MaybeUninit<u8> = res.as_mut_ptr().cast();
        slice::from_raw_parts_mut(p, core::mem::size_of::<T>())
    };
    if secure {
        crate::fill_uninit(dst)?;
    } else {
        crate::insecure_fill_uninit(dst)?;
    }
    // SAFETY: `dst` has been fully initialized by `imp::fill_inner`
    // since it returned `Ok`.
    Ok(unsafe { res.assume_init() })
}

/// Default implementation of `inner_u32` on top of `getrandom::fill_uninit`
pub fn u32() -> Result<u32, Error> {
    unsafe { default_impl(true) }
}

/// Default implementation of `inner_u64` on top of `getrandom::fill_uninit`
pub fn u64() -> Result<u64, Error> {
    unsafe { default_impl(true) }
}

/// Default implementation of `insecure_fill_inner` on top of `getrandom::fill_uninit`
pub fn insecure_fill_uninit(dst: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    crate::fill_uninit(dst).map(|_| ())
}

/// Default implementation of `inner_u32` on top of `getrandom::insecure_fill_uninit`
pub fn insecure_u32() -> Result<u32, Error> {
    unsafe { default_impl(false) }
}

/// Default implementation of `inner_insecure_u64` on top of `getrandom::insecure_fill_uninit`
pub fn insecure_u64() -> Result<u64, Error> {
    unsafe { default_impl(false) }
}
