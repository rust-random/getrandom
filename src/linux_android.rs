// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for Linux / Android
extern crate std;
extern crate libc;

use super::Error;
use std::fs::File;
use std::io;
use std::io::Read;
use std::cell::RefCell;
use std::ops::DerefMut;

enum RngSource {
    GetRandom,
    Device(File),
    None,
}

thread_local!(
    static RNG_SOURCE: RefCell<RngSource> = RefCell::new(RngSource::None);
);

fn syscall_getrandom(dest: &mut [u8]) -> Result<(), io::Error> {
    let ret = unsafe {
        libc::syscall(libc::SYS_getrandom, dest.as_mut_ptr(), dest.len(), 0)
    };
    if ret == -1 || ret != dest.len() as i64 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

pub fn getrandom(dest: &mut [u8]) -> Result<(), Error> {
    RNG_SOURCE.with(|f| {
        let mut f = f.borrow_mut();
        let f: &mut RngSource = f.deref_mut();
        if let RngSource::None = f {
            *f = if is_getrandom_available() {
                RngSource::GetRandom
            } else {
                let mut buf = [0u8; 1];
                File::open("/dev/random")
                    .and_then(|mut f| f.read_exact(&mut buf))
                    .map_err(|_| Error::Unknown)?;
                let mut rng_file = File::open("/dev/urandom")
                    .map_err(|_| Error::Unknown)?;
                RngSource::Device(rng_file)
            }
        }
        if let RngSource::Device(f) = f {
            f.read_exact(dest)
                .map_err(|_| Error::Unknown)
        } else {
            syscall_getrandom(dest)
                .map_err(|_| Error::Unknown)
        }
    })?;
    Ok(())
}

fn is_getrandom_available() -> bool {
    use std::sync::atomic::{AtomicBool, ATOMIC_BOOL_INIT, Ordering};
    use std::sync::{Once, ONCE_INIT};

    static CHECKER: Once = ONCE_INIT;
    static AVAILABLE: AtomicBool = ATOMIC_BOOL_INIT;

    CHECKER.call_once(|| {
        let mut buf: [u8; 0] = [];
        let available = match syscall_getrandom(&mut buf) {
            Ok(()) => true,
            Err(ref err) if err.raw_os_error() == Some(libc::ENOSYS) => false,
            Err(_) => true,
        };
        AVAILABLE.store(available, Ordering::Relaxed);
    });

    AVAILABLE.load(Ordering::Relaxed)
}
