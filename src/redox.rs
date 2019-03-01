// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for Redox
use error::Error;
use utils::use_init;
use std::fs::File;
use std::io::Read;
use std::cell::RefCell;
use std::num::NonZeroU32;

thread_local!(static RNG_FILE: RefCell<Option<File>> = RefCell::new(None));

pub fn getrandom_inner(dest: &mut [u8]) -> Result<(), Error> {
    RNG_FILE.with(|f| {
        use_init(f,
            || File::open("rand:").map_err(From::from),
            |f| f.read_exact(dest).map_err(From::from),
        )
    }).map_err(From::from)
}

#[inline(always)]
pub fn error_msg_inner(_: NonZeroU32) -> Option<&'static str> { None }
