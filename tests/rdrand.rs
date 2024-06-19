// We only test the RDRAND-based RNG source on supported architectures.
#![cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#![cfg_attr(feature = "unstable-sanitize", feature(cfg_sanitize))]

// rdrand.rs expects to be part of the getrandom main crate, so we need these
// additional imports to get rdrand.rs to compile.
use core::mem::MaybeUninit;
use getrandom::Error;
#[macro_use]
extern crate cfg_if;
#[path = "../src/lazy.rs"]
mod lazy;
#[path = "../src/rdrand.rs"]
mod rdrand;
#[path = "../src/util.rs"]
mod util;

use crate::util::slice_assume_init_mut;

fn getrandom_impl(dest: &mut [u8]) -> Result<(), Error> {
    rdrand::getrandom_inner(unsafe { util::slice_as_uninit_mut(dest) })?;
    Ok(())
}
fn getrandom_uninit_impl(dest: &mut [MaybeUninit<u8>]) -> Result<&mut [u8], Error> {
    rdrand::getrandom_inner(dest)?;
    Ok(unsafe { slice_assume_init_mut(dest) })
}

mod common;
