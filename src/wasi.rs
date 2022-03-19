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
use wasi::random_get;

pub fn getrandom_inner(dest: &mut [u8]) -> Result<(), Error> {
    random_get(dest.as_mut_ptr(), dest.len()).map_err(|e: wasi::Errno| {
        // NOTE: `random_get` deals with return value of zero for us, so the
        // unwrap below won't panic. However in case the code ever changes we
        // don't want to use `new_unchecked` and cause UB.
        NonZeroU32::new(e.raw() as u32).unwrap().into()
    })
}
