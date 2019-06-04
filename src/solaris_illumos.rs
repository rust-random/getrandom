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
extern crate libc;
extern crate std;

use crate::Error;
use std::fs::File;
use std::io::Read;
use std::num::NonZeroU32;
use std::sync::atomic::{AtomicIsize, AtomicUsize, Ordering};
use std::os::unix::io::{RawFd, AsRawFd, FromRawFd};
use std::{io, mem};

#[cfg(target_os = "illumos")]
type GetRandomFn = unsafe extern "C" fn(*mut u8, libc::size_t, libc::c_uint) -> libc::ssize_t;
#[cfg(target_os = "solaris")]
type GetRandomFn = unsafe extern "C" fn(*mut u8, libc::size_t, libc::c_uint) -> libc::c_int;

static RNG_FN: AtomicUsize = AtomicUsize::new(0);
// replace with AtomicI32 on stabilization and MSRV bump
static RNG_FD: AtomicIsize = AtomicIsize::new(-1);
// replace with AtomicU8 on stabilization and MSRV bump
static RNG_STATE: AtomicUsize = AtomicUsize::new(0);

const STATE_INIT_ONGOING: usize = 1 << 0;
const STATE_INIT_DONE: usize = 1 << 1;

pub fn getrandom_inner(dest: &mut [u8]) -> Result<(), Error> {
    let f = RNG_FN.load(Ordering::Acquire);
    if f != 0 {
        let f: GetRandomFn = unsafe { mem::transmute(f) };
        return use_fn(f, dest);
    }
    let fd = RNG_FD.load(Ordering::Acquire) as i32;
    if fd != -1 {
        return use_fd(fd, dest);
    }

    loop {
        let state = RNG_STATE.fetch_or(STATE_INIT_ONGOING, Ordering::AcqRel);
        if state & STATE_INIT_DONE != 0 { break; }
        if state & STATE_INIT_ONGOING != 0 {
            std::thread::yield_now();
            continue;
        }

        let f = fetch_getrandom();
        if f == 0 {
            let f = match File::open("/dev/random") {
                Ok(f) => f,
                Err(err) => {
                    RNG_STATE.store(0, Ordering::Release);
                    return Err(err.into());
                },
            };
            RNG_FD.store(f.as_raw_fd() as isize, Ordering::SeqCst);
            mem::forget(f);
        } else {
            RNG_FN.store(f, Ordering::SeqCst);
        }
        RNG_STATE.store(STATE_INIT_DONE, Ordering::SeqCst);
        break;
    }

    let f = RNG_FN.load(Ordering::Acquire);
    if f != 0 {
        let f: GetRandomFn = unsafe { mem::transmute(f) };
        return use_fn(f, dest);
    }
    let fd = RNG_FD.load(Ordering::Acquire) as i32;
    use_fd(fd, dest)
}


fn use_fn(f: GetRandomFn, dest: &mut [u8]) -> Result<(), Error> {
    for chunk in dest.chunks_mut(256) {
        let ret = unsafe {
            f(chunk.as_mut_ptr(), chunk.len(), 0) as libc::ssize_t
        };

        if ret == -1 || ret != chunk.len() as libc::ssize_t {
            let err: Error = io::Error::last_os_error().into();
            error!("getrandom syscall failed: {}", err);
            return Err(err);
        }
    }
    Ok(())
}

fn use_fd(fd: RawFd, dest: &mut [u8]) -> Result<(), Error> {
    let mut f = unsafe { File::from_raw_fd(fd) };
    for chunk in dest.chunks_mut(256) {
        f.read_exact(chunk)?
    }
    mem::forget(f);
    Ok(())
}

/// returns 0 if fetch has failed and function pointer otherwise
fn fetch_getrandom() -> usize {
    let name = "getrandom\0";
    unsafe {
        libc::dlsym(libc::RTLD_DEFAULT, name.as_ptr() as *const _) as usize
    }
}

#[inline(always)]
pub fn error_msg_inner(_: NonZeroU32) -> Option<&'static str> { None }
