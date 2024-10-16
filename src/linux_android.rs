//! Implementation for Linux / Android without `/dev/urandom` fallback
use crate::{util_libc, Error};
use core::mem::MaybeUninit;

#[cfg(not(any(target_os = "android", target_os = "linux")))]
compile_error!("`linux_getrandom` backend can be enabled only for Linux/Android targets!");

pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    util_libc::sys_fill_exact(dest, |buf| unsafe {
        libc::getrandom(buf.as_mut_ptr().cast(), buf.len(), 0)
    })
}
