// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A dummy implementation for unsupported targets which always returns
//! `Err(UNAVAILABLE_ERROR)`
use super::UNAVAILABLE_ERROR;

pub fn getrandom_os(dest: &mut [u8]) -> Result<(), Error> {
    Err(UNAVAILABLE_ERROR)
}
