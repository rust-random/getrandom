#![cfg(all(
    target_os = "linux",
    target_arch = "aarch64",
    feature = "rndr",
    target_feature = "rand"
))]

use getrandom::Error;
#[macro_use]
extern crate cfg_if;
#[path = "../src/lazy.rs"]
mod lazy;
#[path = "../src/linux_android.rs"]
mod linux_android;
#[path = "../src/linux_android_with_fallback.rs"]
mod linux_android_with_fallback;
#[path = "../src/rndr.rs"]
mod rndr;
#[path = "../src/rndr_with_fallback.rs"]
mod rndr_with_fallback;
#[path = "../src/use_file.rs"]
mod use_file;
#[path = "../src/util.rs"]
mod util;
#[path = "../src/util_libc.rs"]
mod util_libc;

fn getrandom_impl(dest: &mut [u8]) -> Result<(), Error> {
    rndr_with_fallback::getrandom_inner(unsafe { util::slice_as_uninit_mut(dest) })?;
    Ok(())
}
mod common;
