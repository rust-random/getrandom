// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementations that just need to read from a file
extern crate std;

use crate::Error;
use core::num::NonZeroU32;
use lazy_static::lazy_static;
use std::{fs::File, io::Read};

#[cfg(target_os = "redox")]
const FILE_PATH: &str = "rand:";
#[cfg(any(target_os = "android", target_os = "linux", target_os = "netbsd"))]
const FILE_PATH: &str = "/dev/urandom";
#[cfg(any(
    target_os = "dragonfly",
    target_os = "emscripten",
    target_os = "haiku",
    target_os = "macos",
    target_os = "solaris",
    target_os = "illumos"
))]
const FILE_PATH: &str = "/dev/random";

pub fn getrandom_inner(dest: &mut [u8]) -> Result<(), Error> {
    lazy_static! {
        static ref FILE: Result<File, Error> = init_file();
    }
    let mut f = FILE.as_ref()?;

    if cfg!(target_os = "emscripten") {
        // `Crypto.getRandomValues` documents `dest` should be at most 65536 bytes.
        for chunk in dest.chunks_mut(65536) {
            f.read_exact(chunk)?;
        }
    } else {
        f.read_exact(dest)?;
    }
    Ok(())
}

fn init_file() -> Result<File, Error> {
    if FILE_PATH == "/dev/urandom" {
        // read one byte from "/dev/random" to ensure that OS RNG has initialized
        File::open("/dev/random")?.read_exact(&mut [0u8; 1])?;
    }
    Ok(File::open(FILE_PATH)?)
}

#[inline(always)]
#[allow(dead_code)]
pub fn error_msg_inner(_: NonZeroU32) -> Option<&'static str> {
    None
}
