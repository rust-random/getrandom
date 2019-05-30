// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for Linux / Android
extern crate std;

use crate::Error;
use std::fs::File;
use std::io::Read;
use std::num::NonZeroU32;
use std::sync::atomic::{AtomicIsize, AtomicUsize, Ordering};
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::{io, mem, thread, time};

// replace with AtomicU8 on stabilization and MSRV bump
static RNG_STATE: AtomicUsize = AtomicUsize::new(0);
// replace with AtomicI32 on stabilization and MSRV bump
static RNG_FD: AtomicIsize = AtomicIsize::new(-1);

const STATE_INIT_ONGOING: usize = 1 << 0;
const STATE_USE_SYSCALL: usize = 1 << 1;
const STATE_USE_FD: usize = 1 << 2;

pub fn getrandom_inner(dest: &mut [u8]) -> Result<(), Error> {
    let state = RNG_STATE.load(Ordering::Acquire);
    if state & STATE_USE_SYSCALL != 0 {
        use_syscall(dest)
    } else if state & STATE_USE_FD != 0 {
        use_fd(dest)
    } else {
        init_loop(dest)
    }
}

fn init_loop(dest: &mut [u8]) -> Result<(), Error> {
    loop {
        let state = RNG_STATE.fetch_or(STATE_INIT_ONGOING, Ordering::AcqRel);

        if state & STATE_INIT_ONGOING != 0 {
            thread::yield_now();
            continue;
        }
        return if state & STATE_USE_SYSCALL != 0 {
            use_syscall(dest)
        } else if state & STATE_USE_FD != 0 {
            use_fd(dest)
        } else {
            init(dest)
        };
    }
}

fn init(dest: &mut [u8]) -> Result<(), Error> {
    match use_syscall(&mut []) {
        Ok(()) => {
            RNG_STATE.store(STATE_USE_SYSCALL, Ordering::Release);
            use_syscall(dest)
        },
        Err(err) if err.code().get() as i32 == libc::ENOSYS => {
            match init_fd() {
                Ok(fd) => {
                    RNG_FD.store(fd as isize, Ordering::SeqCst);
                    RNG_STATE.store(STATE_USE_FD, Ordering::SeqCst);
                    use_fd(dest)
                },
                Err(err) => {
                    RNG_STATE.store(0, Ordering::Release);
                    Err(err.into())
                }
            }
        },
        Err(err) => Err(err),
    }
}

fn init_fd() -> io::Result<i32> {
    // read one byte from "/dev/random" to ensure that OS RNG has initialized
    File::open("/dev/random")?.read_exact(&mut [0u8; 1])?;
    let f = File::open("/dev/urandom")?;
    let fd = f.as_raw_fd();
    mem::forget(f);
    Ok(fd)
}

fn use_syscall(dest: &mut [u8]) -> Result<(), Error> {
    let ret = unsafe {
        libc::syscall(libc::SYS_getrandom, dest.as_mut_ptr(), dest.len(), 0)
    };
    if ret < 0 || (ret as usize) != dest.len() {
        return Err(io::Error::last_os_error().into());
    }
    Ok(())
}

fn use_fd(dest: &mut [u8]) -> Result<(), Error> {
    unsafe {
        let fd = RNG_FD.load(Ordering::Acquire) as i32;
        let mut f = File::from_raw_fd(fd);
        f.read_exact(dest)?;
        mem::forget(f);
    }
    Ok(())
}

#[inline(always)]
pub fn error_msg_inner(_: NonZeroU32) -> Option<&'static str> { None }
