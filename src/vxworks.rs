// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for VxWorks
use crate::error::{Error, ERRNO_NOT_POSITIVE};
use core::num::NonZeroU32;

extern "C" {
    fn randBytes (buf: *mut u8, length: i32) -> i32;
    // errnoLib.h
    fn errnoGet() -> i32;
}

pub fn getrandom_inner(dest: &mut [u8]) -> Result<(), Error> {
    // Prevent overflow of i32
    for chunk in dest.chunks_mut(i32::max_value() as usize) {
        let ret = randBytes(chunk.as_mut_ptr(), chunk.len() as i32);
        if ret == -1 {
            let errno = errnoGet();
            let err = if errno > 0 {
                Error::from(NonZeroU32::new(errno as u32).unwrap())
            } else {
                ERRNO_NOT_POSITIVE
            };
            return Err(err);
        }
    }
    Ok(())
}
