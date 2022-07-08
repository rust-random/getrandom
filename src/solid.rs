// Copyright 2021 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for SOLID
use core::mem::MaybeUninit;
use core::num::NonZeroU32;

use crate::Error;

extern "C" {
    pub fn SOLID_RNG_SampleRandomBytes(buffer: *mut u8, length: usize) -> i32;
}

pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    let ret = unsafe { SOLID_RNG_SampleRandomBytes(dest.as_mut_ptr() as *mut u8, dest.len()) };
    if ret >= 0 {
        Ok(())
    } else {
        // ITRON error numbers are always negative, so we negate it so that it
        // falls in the dedicated OS error range (1..INTERNAL_START).
        Err(NonZeroU32::new((-ret) as u32).unwrap().into())
    }
}
