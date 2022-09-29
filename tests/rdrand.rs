// We only test the RDRAND-based RNG source on supported architectures.
#![cfg(any(target_arch = "x86_64", target_arch = "x86"))]

// rdrand.rs expects to be part of the getrandom main crate, so we need these
// additional imports to get rdrand.rs to compile.
use getrandom::Error;
#[macro_use]
extern crate cfg_if;
#[path = "../src/rdrand.rs"]
mod rdrand;
#[path = "../src/util.rs"]
mod util;

pub fn getrandom(dst: &mut [u8]) -> Result<(), Error> {
    if dst.is_empty() {
        return Ok(());
    }
    unsafe { rdrand::getrandom_inner(dst.as_mut_ptr(), dst.len()) }
}

mod common;
