//! Implementation for Linux / Android without `/dev/urandom` fallback
use crate::{util_libc, Error};
use core::mem::MaybeUninit;

pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    util_libc::sys_fill_exact(dest, getrandom_syscall)
}

pub fn getrandom_syscall(buf: &mut [MaybeUninit<u8>]) -> libc::ssize_t {
    let res: libc::c_long = unsafe {
        libc::syscall(
            libc::SYS_getrandom,
            buf.as_mut_ptr().cast::<core::ffi::c_void>(),
            buf.len(),
            0,
        )
    };

    const _: () =
        assert!(core::mem::size_of::<libc::c_long>() == core::mem::size_of::<libc::ssize_t>());
    res.try_into()
        .expect("c_long to ssize_t conversion is lossless")
}
