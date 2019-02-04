// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for DragonFly / Haiku / Emscripten
use super::Error;
use std::fs::File;
use std::io::Read;
use std::cell::RefCell;
use std::ops::DerefMut;

thread_local!(static RNG_FILE: RefCell<Option<File>> = RefCell::new(None));

pub fn getrandom(dest: &mut [u8]) -> Result<(), Error> {
    // `Crypto.getRandomValues` documents `dest` should be at most 65536
    // bytes. `crypto.randomBytes` documents: "To minimize threadpool
    // task length variation, partition large randomBytes requests when
    // doing so as part of fulfilling a client request.
    for chunk in dest.chunks_mut(65536) {
        RNG_FILE.with(|f| {
            let mut f = f.borrow_mut();
            let f: &mut Option<File> = f.deref_mut();
            if let Some(f) = f {
                f.read_exact(chunk)
            } else {
                let mut rng_file = File::open("/dev/random")?;
                rng_file.read_exact(chunk)?;
                *f = Some(rng_file);
                Ok(())
            }
        }).map_err(|_| Error::Unknown)?;
    }
    Ok(())
}
