// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for WASI
use crate::Error;
use core::num::NonZeroU32;
use wasi::wasi_unstable::random_get;

pub fn getrandom_inner(dest: &mut [u8]) -> Result<(), Error> {
    // wasi uses NonZeroU16 for error codes
    // here compiler is able to eliminate `unwrap`
    random_get(dest).map_err(|e| {
        Error(NonZeroU32::new(e.get() as u32).unwrap())
    })
}
