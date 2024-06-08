//! Implementation for Linux / Android without `/dev/urandom` fallback
use crate::Error;
use core::mem::MaybeUninit;

#[cfg(not(feature = "rustix"))]
use crate::util_libc;
#[cfg(feature = "rustix")]
use crate::util_rustix as util_libc;

pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    util_libc::sys_fill_exact(dest, util_libc::getrandom_syscall)
}
