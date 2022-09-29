// Copyright 2021 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for Nintendo 3DS
use crate::util_libc::sys_fill_exact;
use crate::Error;

pub unsafe fn getrandom_inner(dst: *mut u8, len: usize) -> Result<(), Error> {
    // TODO: use `cast` on MSRV bump to 1.38
    sys_fill_exact(dst, len, |cdst, clen| {
        libc::getrandom(cdst as *mut libc::c_void, clen, 0)
    })
}
