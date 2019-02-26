// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for MacOS / iOS
extern crate libc;

use super::Error;
use std::io;
use self::libc::{c_int, size_t};

enum SecRandom {}

/// Essentially a null pointer (type `SecRandomRef`)
#[allow(non_upper_case_globals)]
const kSecRandomDefault: *const SecRandom = 0 as *const SecRandom;

#[link(name = "Security", kind = "framework")]
extern {
    fn SecRandomCopyBytes(
        rnd: *const SecRandom, count: size_t, bytes: *mut u8,
    ) -> c_int;
}

pub fn getrandom_inner(dest: &mut [u8]) -> Result<(), Error> {
    let ret = unsafe {
        SecRandomCopyBytes(
            kSecRandomDefault,
            dest.len() as size_t,
            dest.as_mut_ptr(),
        )
    };
    if ret == -1 {
        Err(io::Error::last_os_error().into())
    } else {
        Ok(())
    }
}
