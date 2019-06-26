// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for the Solaris family
//!
//! Read from `/dev/random`, with chunks of limited size (256 bytes).
//! `/dev/random` uses the Hash_DRBG with SHA512 algorithm from NIST SP 800-90A.
//! `/dev/urandom` uses the FIPS 186-2 algorithm, which is considered less
//! secure. We choose to read from `/dev/random`.
//!
//! Since Solaris 11.3 and mid-2015 illumos, the `getrandom` syscall is available.
//! To make sure we can compile on both Solaris and its derivatives, as well as
//! function, we check for the existance of getrandom(2) in libc by calling
//! libc::dlsym.
extern crate std;

use crate::{use_file, Error};
use core::mem;
use core::num::NonZeroU32;
use lazy_static::lazy_static;
use std::io;

#[cfg(target_os = "illumos")]
type GetRandomFn = unsafe extern "C" fn(*mut u8, libc::size_t, libc::c_uint) -> libc::ssize_t;
#[cfg(target_os = "solaris")]
type GetRandomFn = unsafe extern "C" fn(*mut u8, libc::size_t, libc::c_uint) -> libc::c_int;

fn libc_getrandom(rand: GetRandomFn, dest: &mut [u8]) -> Result<(), Error> {
    let ret = unsafe { rand(dest.as_mut_ptr(), dest.len(), 0) as libc::ssize_t };

    if ret == -1 || ret != dest.len() as libc::ssize_t {
        error!("getrandom syscall failed with ret={}", ret);
        Err(io::Error::last_os_error().into())
    } else {
        Ok(())
    }
}

pub fn getrandom_inner(dest: &mut [u8]) -> Result<(), Error> {
    lazy_static! {
        static ref GETRANDOM_FUNC: Option<GetRandomFn> = fetch_getrandom();
    }

    // 256 bytes is the lowest common denominator across all the Solaris
    // derived platforms for atomically obtaining random data.
    for chunk in dest.chunks_mut(256) {
        match *GETRANDOM_FUNC {
            Some(fptr) => libc_getrandom(fptr, chunk)?,
            None => use_file::getrandom_inner(chunk)?,
        };
    }
    Ok(())
}

fn fetch_getrandom() -> Option<GetRandomFn> {
    let name = "getrandom\0";
    let addr = unsafe { libc::dlsym(libc::RTLD_DEFAULT, name.as_ptr() as *const _) };
    unsafe { mem::transmute(addr) }
}

#[inline(always)]
pub fn error_msg_inner(_: NonZeroU32) -> Option<&'static str> {
    None
}
