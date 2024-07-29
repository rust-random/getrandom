#![cfg(all(target_os = "linux", target_arch = "aarch64", feature = "rndr"))]

use getrandom::Error;
#[path = "../src/rndr.rs"]
mod rndr;
#[path = "../src/util.rs"]
mod util;

fn getrandom_impl(dest: &mut [u8]) -> Result<(), Error> {
    rndr::getrandom_inner(unsafe { util::slice_as_uninit_mut(dest) })?;
    Ok(())
}
mod common;
