// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for VxWorks
use crate::{util::raw_chunks, util_libc::last_os_error, Error};
use core::sync::atomic::{AtomicBool, Ordering::Relaxed};

pub unsafe fn getrandom_inner(dst: *mut u8, len: usize) -> Result<(), Error> {
    static RNG_INIT: AtomicBool = AtomicBool::new(false);
    while !RNG_INIT.load(Relaxed) {
        let ret = libc::randSecure();
        if ret < 0 {
            return Err(Error::VXWORKS_RAND_SECURE);
        } else if ret > 0 {
            RNG_INIT.store(true, Relaxed);
            break;
        }
        libc::usleep(10);
    }

    // Prevent overflow of i32
    raw_chunks(dst, len, i32::max_value() as usize, |cdst, clen| {
        let ret = libc::randABytes(cdst, clen as i32);
        match ret {
            0 => Ok(()),
            _ => Err(last_os_error()),
        }
    })
}
