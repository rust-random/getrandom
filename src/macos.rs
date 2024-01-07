// Copyright 2023-2024 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for macOS
use crate::{util_libc::last_os_error, Error};
use core::mem::MaybeUninit;

extern "C" {
    // Supported as of macOS 10.12+.
    fn getentropy(buf: *mut u8, size: libc::size_t) -> libc::c_int;
}

pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    for chunk in dest.chunks_mut(256) {
        let ret = unsafe { getentropy(chunk.as_mut_ptr() as *mut u8, chunk.len()) };
        if ret != 0 {
            return Err(last_os_error());
        }
    }
    Ok(())
}
