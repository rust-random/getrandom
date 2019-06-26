// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for Windows
extern crate std;

use crate::Error;
use core::num::NonZeroU32;
use std::io;
use winapi::shared::minwindef::ULONG;
use winapi::um::ntsecapi::RtlGenRandom;
use winapi::um::winnt::PVOID;

pub fn getrandom_inner(dest: &mut [u8]) -> Result<(), Error> {
    // Prevent overflow of ULONG
    for chunk in dest.chunks_mut(ULONG::max_value() as usize) {
        let ret = unsafe { RtlGenRandom(chunk.as_mut_ptr() as PVOID, chunk.len() as ULONG) };
        if ret == 0 {
            error!("RtlGenRandom call failed");
            return Err(io::Error::last_os_error().into());
        }
    }
    Ok(())
}

#[inline(always)]
pub fn error_msg_inner(_: NonZeroU32) -> Option<&'static str> {
    None
}
