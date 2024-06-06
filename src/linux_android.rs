//! Implementation for Linux / Android without `/dev/urandom` fallback
use crate::{util_unix::sys_fill_exact, Error};
use core::mem::MaybeUninit;

pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    sys_fill_exact(dest, getrandom_syscall)
}

// Also used by linux_android_with_fallback to check if the syscall is available.
pub fn getrandom_syscall(buf: &mut [MaybeUninit<u8>]) -> libc::ssize_t {
    unsafe {
        libc::syscall(
            libc::SYS_getrandom,
            buf.as_mut_ptr().cast::<core::ffi::c_void>(),
            buf.len(),
            0,
        ) as libc::ssize_t
    }
}
