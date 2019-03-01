// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for NetBSD

use error::Error;
use utils::use_init;
use std::fs::File;
use std::io::Read;
use std::cell::RefCell;
use std::num::NonZeroU32;
use std::sync::atomic::{AtomicBool, Ordering};

static RNG_INIT: AtomicBool = AtomicBool::new(false);

thread_local!(static RNG_FILE: RefCell<Option<File>> = RefCell::new(None));

pub fn getrandom_inner(dest: &mut [u8]) -> Result<(), Error> {
    RNG_FILE.with(|f| {
        use_init(f, || {
            // read one byte from "/dev/random" to ensure that
            // OS RNG has initialized
            if !RNG_INIT.load(Ordering::Relaxed) {
                File::open("/dev/random")?.read_exact(&mut [0u8; 1])?;
                RNG_INIT.store(true, Ordering::Relaxed)
            }
            File::open("/dev/urandom").map_err(From::from)
        }, |f| f.read_exact(dest).map_err(From::from))
    })
}

#[inline(always)]
pub fn error_msg_inner(_: NonZeroU32) -> Option<&'static str> { None }
