// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for the Solaris family
//!
//! Read from `/dev/random`, with chunks of limited size (1040 bytes).
//! `/dev/random` uses the Hash_DRBG with SHA512 algorithm from NIST SP 800-90A.
//! `/dev/urandom` uses the FIPS 186-2 algorithm, which is considered less
//! secure. We choose to read from `/dev/random`.
//!
//! Since Solaris 11.3 the `getrandom` syscall is available. To make sure we can
//! compile on both Solaris and on OpenSolaris derivatives, that do not have the
//! function, we do a direct syscall instead of calling a library function.
//!
//! We have no way to differentiate between Solaris, illumos, SmartOS, etc.
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
}

thread_local!(
    static RNG_SOURCE: RefCell<Option<RngSource>> = RefCell::new(None);
);

fn syscall_getrandom(dest: &mut [u8]) -> Result<(), Error> {
    // repalce with libc?
    const SYS_GETRANDOM: libc::c_long = 143;

    extern "C" {
        fn syscall(number: libc::c_long, ...) -> libc::c_long;
    }

    let ret = unsafe {
        syscall(SYS_GETRANDOM, dest.as_mut_ptr(), dest.len(), 0)
    };
    if ret == -1 || ret != dest.len() as i64 {
        return Err(io::Error::last_os_error().into());
    }
    Ok(())
}

pub fn getrandom_os(dest: &mut [u8]) -> Result<(), Error> {
    // The documentation says 1024 is the maximum for getrandom
    // and 1040 for /dev/random.
    RNG_SOURCE.with(|f| {
        use_init(f,
        || {
            let s = if is_getrandom_available() {
                RngSource::GetRandom
            } else {
                RngSource::Device(File::open("/dev/random")?)
            };
            Ok(s)
        }, |f| {
            match f {
                RngSource::GetRandom => for chunk in dest.chunks_mut(1024) {
                    syscall_getrandom(chunk)
                },
                RngSource::Device(f) => for chunk in dest.chunks_mut(1040) {
                    f.read_exact(dest).map_err(From::from)
                },
            }
        })
    })
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
