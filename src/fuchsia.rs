// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for Fuchsia Zircon
use super::Error;

#[link(name = "zircon")]
extern {
    fn zx_cprng_draw(buffer: *mut u8, len: usize);
}

pub fn getrandom(&mut self, dest: &mut [u8]) -> Result<(), Error> {
    for chunk in dest.chunks(256) {
        unsafe { zx_cprng_draw(chunk.as_mut_ptr(), chunk.len()) };
    }
    Ok(())
}
