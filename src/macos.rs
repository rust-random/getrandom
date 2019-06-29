// Copyright 2019 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for macOS
extern crate std;

use crate::{use_file, Error};
use core::mem;
use core::num::NonZeroU32;
use lazy_static::lazy_static;
use std::io;

type GetEntropyFn = unsafe extern "C" fn(*mut u8, libc::size_t) -> libc::c_int;

fn fetch_getentropy() -> Option<GetEntropyFn> {
    let name = "getentropy\0";
    let addr = unsafe { libc::dlsym(libc::RTLD_DEFAULT, name.as_ptr() as *const _) };
    unsafe { mem::transmute(addr) }
}

pub fn getrandom_inner(dest: &mut [u8]) -> Result<(), Error> {
    lazy_static! {
        static ref GETENTROPY_FUNC: Option<GetEntropyFn> = fetch_getentropy();
    }
    if let Some(fptr) = *GETENTROPY_FUNC {
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
