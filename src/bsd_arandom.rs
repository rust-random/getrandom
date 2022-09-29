// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for FreeBSD and NetBSD
use crate::{util::raw_chunks, util_libc::sys_fill_exact, Error};
use core::ptr;

unsafe fn kern_arnd(dst: *mut u8, mut len: usize) -> libc::ssize_t {
    static MIB: [libc::c_int; 2] = [libc::CTL_KERN, libc::KERN_ARND];
    // TODO: use `cast` on MSRV bump to 1.38
    let ret = libc::sysctl(
        MIB.as_ptr(),
        MIB.len() as libc::c_uint,
        dst as *mut libc::c_void,
        &mut len,
        ptr::null(),
        0,
    );
    if ret == -1 {
        -1
    } else {
        len as libc::ssize_t
    }
}

pub unsafe fn getrandom_inner(dst: *mut u8, len: usize) -> Result<(), Error> {
    // getrandom(2) was introduced in FreeBSD 12.0 and NetBSD 10.0
    #[cfg(target_os = "freebsd")]
    {
        use crate::util_libc::Weak;
        static GETRANDOM: Weak = unsafe { Weak::new("getrandom\0") };
        type GetRandomFn =
            unsafe extern "C" fn(*mut u8, libc::size_t, libc::c_uint) -> libc::ssize_t;

        if let Some(fptr) = GETRANDOM.ptr() {
            let func: GetRandomFn = core::mem::transmute(fptr);
            return sys_fill_exact(dst, len, |cdst, clen| func(cdst, clen, 0));
        }
    }
    // Both FreeBSD and NetBSD will only return up to 256 bytes at a time, and
    // older NetBSD kernels will fail on longer buffers.
    raw_chunks(dst, len, 256, |cdst, clen| {
        sys_fill_exact(cdst, clen, |cdst2, clen2| kern_arnd(cdst2, clen2))
    })
}
