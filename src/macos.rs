// Copyright 2019 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for macOS
use crate::{
    use_file,
    util::raw_chunks,
    util_libc::{last_os_error, Weak},
    Error,
};
use core::mem;

type GetEntropyFn = unsafe extern "C" fn(*mut u8, libc::size_t) -> libc::c_int;

pub unsafe fn getrandom_inner(dst: *mut u8, len: usize) -> Result<(), Error> {
    // getentropy(2) was added in 10.12, Rust supports 10.7+
    static GETENTROPY: Weak = unsafe { Weak::new("getentropy\0") };
    if let Some(fptr) = GETENTROPY.ptr() {
        let func: GetEntropyFn = mem::transmute(fptr);
        raw_chunks(dst, len, 256, |cdst, clen| {
            let ret = func(cdst, clen);
            match ret {
                0 => Ok(()),
                _ => Err(last_os_error()),
            }
        })
    } else {
        // We fallback to reading from /dev/random instead of SecRandomCopyBytes
        // to avoid high startup costs and linking the Security framework.
        use_file::getrandom_inner(dst, len)
    }
}
