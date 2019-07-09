// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for FreeBSD
use crate::util_libc::fill_exact;
use crate::Error;
use core::num::NonZeroU32;
use core::ptr;

fn kern_arnd(buf: &mut [u8]) -> libc::ssize_t {
    static MIB: [libc::c_int; 2] = [libc::CTL_KERN, libc::KERN_ARND];
    let mut len = buf.len();
    let ret = unsafe {
        libc::sysctl(
            MIB.as_ptr(),
            MIB.len() as libc::c_uint,
            buf.as_mut_ptr() as *mut _,
            &mut len,
            ptr::null(),
            0,
        )
    };
    if ret == -1 {
        error!("freebsd: kern.arandom syscall failed");
        -1
    } else {
        len as libc::ssize_t
    }
}

pub fn getrandom_inner(dest: &mut [u8]) -> Result<(), Error> {
    fill_exact(dest, kern_arnd)
}

#[inline(always)]
pub fn error_msg_inner(_: NonZeroU32) -> Option<&'static str> {
    None
}
