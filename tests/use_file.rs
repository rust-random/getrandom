// Test use_file on Linux in configurations where it might be used, even if
// the test runner supports the syscall.
#![cfg(all(target_os = "linux", not(feature = "linux_disable_fallback")))]

// rdrand.rs expects to be part of the getrandom main crate, so we need these
// additional imports to get rdrand.rs to compile.
use getrandom::Error;
#[macro_use]
extern crate cfg_if;
#[path = "../src/lazy.rs"]
mod lazy;
#[path = "../src/use_file.rs"]
mod use_file;
#[path = "../src/util.rs"]
mod util;
#[path = "../src/util_libc.rs"]
mod util_libc;

// The use_file implementation has the signature of getrandom_uninit(), but our
// tests expect getrandom_impl() to have the signature of getrandom().
fn getrandom_impl(dest: &mut [u8]) -> Result<(), Error> {
    use_file::getrandom_inner(unsafe { util::slice_as_uninit_mut(dest) })?;
    Ok(())
}
mod common;
