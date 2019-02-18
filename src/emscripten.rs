// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for Emscripten
use super::Error;
use std::fs::File;
use std::io::Read;
use std::cell::RefCell;
use super::utils::use_init;

thread_local!(static RNG_FILE: RefCell<Option<File>> = RefCell::new(None));

pub fn getrandom_os(dest: &mut [u8]) -> Result<(), Error> {
    // `Crypto.getRandomValues` documents `dest` should be at most 65536
    // bytes. `crypto.randomBytes` documents: "To minimize threadpool
    // task length variation, partition large randomBytes requests when
    // doing so as part of fulfilling a client request.
    RNG_FILE.with(|f| {
        use_init(f, || File::open("/dev/random").map_err(From::from), |f| {
            for chunk in dest.chunks_mut(65536) {
                f.read_exact(chunk)?;
            }
            Ok(())
        })
    })
}
