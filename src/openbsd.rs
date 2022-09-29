// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for OpenBSD
use crate::{util::raw_chunks, util_libc::last_os_error, Error};

pub unsafe fn getrandom_inner(dst: *mut u8, len: usize) -> Result<(), Error> {
    // getentropy(2) was added in OpenBSD 5.6, so we can use it unconditionally.
    raw_chunks(dst, len, 256, |cdst, clen| {
        // TODO: use `cast` on MSRV bump to 1.38
        let ret = libc::getentropy(cdst as *mut libc::c_void, clen);
        match ret {
            -1 => Err(last_os_error()),
            _ => Ok(()),
        }
    })
}
