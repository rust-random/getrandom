// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for Redox, DragonFly, Haiku, NetBsd, Emscripten
extern crate std;

use crate::Error;
use std::fs::File;
use std::io::Read;
use std::num::NonZeroU32;
use std::sync::atomic::{AtomicIsize, AtomicUsize, Ordering};
use std::os::unix::io::{RawFd, AsRawFd, FromRawFd};
use std::{io, mem};

#[cfg(target_os = "redox")]
const FILE_PATH: &'static str = "rand:";
#[cfg(target_os = "netbsd")]
const FILE_PATH: &'static str = "/dev/urandom";
#[cfg(any(target_os = "dragonfly", target_os = "emscripten", target_os = "haiku"))]
const FILE_PATH: &'static str = "/dev/random";

// replace with AtomicI32 on stabilization and MSRV bump
static RNG_FD: AtomicIsize = AtomicIsize::new(-1);
// replace with AtomicU8 on stabilization and MSRV bump
static RNG_STATE: AtomicUsize = AtomicUsize::new(0);

const STATE_INIT_ONGOING: usize = 1 << 0;
const STATE_INIT_DONE: usize = 1 << 1;

pub fn getrandom_inner(dest: &mut [u8]) -> Result<(), Error> {
    let fd = RNG_FD.load(Ordering::Acquire);
    // Redox uses usize for descriptors, but `-1i32 as usize` is
    // still an invalid descriptor value, see:
    // https://github.com/redox-os/syscall/blob/master/src/error.rs#L22
    let fd = if fd != -1 { fd as RawFd } else { init_loop()? };
    let res = use_fd(fd, dest);
    if let Err(err) = res {
        error!("failed to read random data: {}", err);
        return Err(err.into());
    }
    Ok(())
}

fn init_loop() -> Result<RawFd, Error> {
    loop {
        let state = RNG_STATE.fetch_or(STATE_INIT_ONGOING, Ordering::AcqRel);
        if state & STATE_INIT_DONE != 0 {
            // initialization is complete, use fd from atomic
            return Ok(RNG_FD.load(Ordering::Acquire) as RawFd);
        } else if state & STATE_INIT_ONGOING == 0 {
            // start initialization and return resulting fd
            match init_fd() {
                Ok(fd) => {
                    RNG_FD.store(fd as isize, Ordering::SeqCst);
                    RNG_STATE.store(STATE_INIT_DONE, Ordering::SeqCst);
                    return Ok(fd)
                },
                Err(err) => {
                    RNG_STATE.store(0, Ordering::Release);
                    error!("initialization has failed: {}", err);
                    return Err(err.into());
                }
            }
        }
        std::thread::yield_now();
    }
}

fn use_fd(fd: RawFd, dest: &mut [u8]) -> io::Result<()> {
    let mut f = unsafe { File::from_raw_fd(fd) };
    if cfg!(target_os = "emscripten") {
        // `Crypto.getRandomValues` documents `dest` should be at most 65536 bytes.
        for chunk in dest.chunks_mut(65536) {
            f.read_exact(chunk)?;
        }
    } else {
        f.read_exact(dest)?;
    }
    mem::forget(f);
    Ok(())
}

fn init_fd() -> io::Result<RawFd> {
    if cfg!(target_os = "netbsd") {
        // read one byte from "/dev/random" to ensure that OS RNG has initialized
        File::open("/dev/random")?.read_exact(&mut [0u8; 1])?;
    }
    let f = File::open(FILE_PATH)?;
    let fd = f.as_raw_fd();
    mem::forget(f);
    Ok(fd)
}

#[inline(always)]
pub fn error_msg_inner(_: NonZeroU32) -> Option<&'static str> { None }
