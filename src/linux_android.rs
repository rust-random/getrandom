//! Implementation for Linux / Android without `/dev/urandom` fallback
use crate::{util_unix::sys_fill_exact, Error};
use core::mem::MaybeUninit;

pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    sys_fill_exact(dest, getrandom_syscall)
}

// Also used by linux_android_with_fallback to check if the syscall is available.
pub fn getrandom_syscall(buf: &mut [MaybeUninit<u8>]) -> Result<usize, Error> {
    use crate::util_libc::last_os_error;

    let ret: libc::c_long = unsafe {
        libc::syscall(
            libc::SYS_getrandom,
            buf.as_mut_ptr().cast::<core::ffi::c_void>(),
            buf.len(),
            0,
        )
    };
    const _: () = assert!(core::mem::size_of::<libc::c_long>() == core::mem::size_of::<isize>());
    usize::try_from(ret as isize).map_err(|_| last_os_error())
}
