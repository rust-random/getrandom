//! Implementation using libc::getrandom
use crate::{util_libc::sys_fill_exact, Error};
use core::mem::MaybeUninit;

pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    sys_fill_exact(dest, |buf| unsafe {
        libc::getrandom(buf.as_mut_ptr() as *mut libc::c_void, buf.len(), 0)
    })
}
