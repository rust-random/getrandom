//! Implementation for Linux / Android without `/dev/urandom` fallback
use crate::{util_libc, util_syscall_linux, Error};
use core::mem::MaybeUninit;

pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    util_libc::sys_fill_exact(dest, getrandom_syscall)
}

// Also used by linux_android_with_fallback to check if the syscall is available.
pub fn getrandom_syscall(buf: &mut [MaybeUninit<u8>]) -> libc::ssize_t {
    util_syscall_linux::pre_write_range(buf.as_mut_ptr(), buf.len());
    let res = unsafe {
        libc::syscall(
            libc::SYS_getrandom,
            buf.as_mut_ptr().cast::<core::ffi::c_void>(),
            buf.len(),
            0,
        )
    } as libc::ssize_t;
    if let Ok(written) = usize::try_from(res) {
        // XXX: LLVM has support to do this automatically if/when libc is
        // compiled with it, but glibc that ships in typical Linux distros
        // doesn't. Assume Android's Bionic is similar. `-Zsanitizer=memory`
        // is not compatible with `+crt-static` according to rustc.
        unsafe {
            util_syscall_linux::post_write_range(buf.as_mut_ptr(), written);
        }
    };
    res
}
