#![allow(dead_code)]
use core::{mem::MaybeUninit, ptr};

/// Polyfill for `maybe_uninit_slice` feature's
/// `MaybeUninit::slice_assume_init_mut`. Every element of `slice` must have
/// been initialized.
#[inline(always)]
pub unsafe fn slice_assume_init_mut<T>(slice: &mut [MaybeUninit<T>]) -> &mut [T] {
    // SAFETY: `MaybeUninit<T>` is guaranteed to be layout-compatible with `T`.
    &mut *(slice as *mut [MaybeUninit<T>] as *mut [T])
}

#[inline]
pub fn uninit_slice_fill_zero(slice: &mut [MaybeUninit<u8>]) -> &mut [u8] {
    unsafe { ptr::write_bytes(slice.as_mut_ptr(), 0, slice.len()) };
    unsafe { slice_assume_init_mut(slice) }
}

#[inline(always)]
pub fn slice_as_uninit(slice: &[u8]) -> &[MaybeUninit<u8>] {
    // TODO: MSRV(1.76): Use `core::ptr::from_ref`.
    let ptr: *const [u8] = slice;
    // SAFETY: `MaybeUninit<u8>` is guaranteed to be layout-compatible with `u8`.
    // There is no risk of writing a `MaybeUninit<u8>` into the result since
    // the result isn't mutable.
    // FIXME: Avoid relying on this assumption and eliminate this cast.
    unsafe { &*(ptr as *const [MaybeUninit<u8>]) }
}

/// View an mutable initialized array as potentially-uninitialized.
///
/// This is unsafe because it allows assigning uninitialized values into
/// `slice`, which would be undefined behavior.
#[inline(always)]
pub unsafe fn slice_as_uninit_mut(slice: &mut [u8]) -> &mut [MaybeUninit<u8>] {
    // TODO: MSRV(1.76): Use `core::ptr::from_mut`.
    let ptr: *mut [u8] = slice;
    // SAFETY: `MaybeUninit<u8>` is guaranteed to be layout-compatible with `u8`.
    // FIXME: Avoid relying on this assumption and eliminate this cast.
    &mut *(ptr as *mut [MaybeUninit<u8>])
}
