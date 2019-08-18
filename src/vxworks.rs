// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for VxWorks
use crate::error::Error;
use core::sync::atomic::{AtomicUBool, Ordering::Relaxed};

static RNG_INIT: AtomicBool = AtomicBool::new(false);

pub fn getrandom_inner(dest: &mut [u8]) -> Result<(), Error> {
    while !RNG_INIT.load(Relaxed) {
        let ret = unsafe { libc::randSecure() };
        if ret < 0 {
            return Err(last_os_error());
        } else if ret > 0 {
            RNG_INIT.store(true, Relaxed);
            break;
        }
        unsafe { libc::usleep(10) };
    }

    // Prevent overflow of i32
    for chunk in dest.chunks_mut(i32::max_value() as usize) {
        let ret = unsafe { randABytes(chunk.as_mut_ptr(), chunk.len() as i32) };
        if ret < 0 {
            return Err(last_os_error());
        }
    }
    Ok(())
}
