// Copyright 2019 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for macOS
extern crate std;

use crate::util_libc::Weak;
use crate::{use_file, Error};
use core::num::NonZeroU32;
use std::io;

type GetEntropyFn = unsafe extern "C" fn(*mut u8, libc::size_t) -> libc::c_int;

pub fn getrandom_inner(dest: &mut [u8]) -> Result<(), Error> {
    static GETENTROPY: Weak<GetEntropyFn> = unsafe { Weak::new("getentropy\0") };
    if let Some(fptr) = GETENTROPY.func() {
        for chunk in dest.chunks_mut(256) {
            let ret = unsafe { fptr(chunk.as_mut_ptr(), chunk.len()) };
            if ret != 0 {
                error!("getentropy syscall failed with ret={}", ret);
                return Err(io::Error::last_os_error().into());
            }
        }
        Ok(())
    } else {
        // We fallback to reading from /dev/random instead of SecRandomCopyBytes
        // to avoid high startup costs and linking the Security framework.
        use_file::getrandom_inner(dest)
    }
}

#[inline(always)]
pub fn error_msg_inner(_: NonZeroU32) -> Option<&'static str> {
    None
}
