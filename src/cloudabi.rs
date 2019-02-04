// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use error::Error;

extern "C" {
    fn cloudabi_sys_random_get(buf: *mut u8, len: usize) -> u16;
}

pub fn getrandom(dest: &mut [u8]) -> Result<(), Error> {
    let errno = unsafe { cloudabi_sys_random_get(dest.as_ptr(), dest.len()) };
    if errno == 0 {
        Ok(())
    } else {
        Err(Error::Unknown)
    }
}
