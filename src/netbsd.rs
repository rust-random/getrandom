//! Implementation for NetBSD
use crate::{
    util_libc::{cstr, sys_fill_exact, Weak},
    Error,
};
use core::{ffi::c_void, mem::MaybeUninit, ptr};

fn kern_arnd(buf: &mut [MaybeUninit<u8>]) -> libc::ssize_t {
    static MIB: [libc::c_int; 2] = [libc::CTL_KERN, libc::KERN_ARND];
    let mut len = buf.len();
    let ret = unsafe {
        libc::sysctl(
            MIB.as_ptr(),
            MIB.len() as libc::c_uint,
            buf.as_mut_ptr().cast::<c_void>(),
            &mut len,
            ptr::null(),
            0,
        )
    };
    if ret == -1 {
        -1
    } else {
        len as libc::ssize_t
    }
}

type GetRandomFn = unsafe extern "C" fn(*mut u8, libc::size_t, libc::c_uint) -> libc::ssize_t;

pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    // getrandom(2) was introduced in NetBSD 10.0
    static GETRANDOM: Weak = Weak::new(cstr::unwrap_const_from_bytes_with_nul(b"getrandom\0"));
    if let Some(fptr) = GETRANDOM.ptr() {
        let func: GetRandomFn = unsafe { core::mem::transmute(fptr) };
        return sys_fill_exact(dest, |buf| unsafe {
            func(buf.as_mut_ptr().cast::<u8>(), buf.len(), 0)
        });
    }

    // NetBSD will only return up to 256 bytes at a time, and
    // older NetBSD kernels will fail on longer buffers.
    for chunk in dest.chunks_mut(256) {
        sys_fill_exact(chunk, kern_arnd)?
    }
    Ok(())
}
