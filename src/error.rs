// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use core::num::NonZeroU32;
use core::convert::From;
use core::fmt;
#[cfg(not(target_env = "sgx"))]
use std::{io, error};

pub const UNKNOWN_ERROR: Error = Error(unsafe {
    NonZeroU32::new_unchecked(0x756e6b6e) // "unkn"
});

pub const UNAVAILABLE_ERROR: Error = Error(unsafe {
    NonZeroU32::new_unchecked(0x4e416e61) // "NAna"
});

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Error(NonZeroU32);

impl Error {
    pub fn code(&self) -> NonZeroU32 {
        self.0
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            UNKNOWN_ERROR => write!(f, "Getrandom Error: unknown"),
            UNAVAILABLE_ERROR => write!(f, "Getrandom Error: unavailable"),
            code => write!(f, "Getrandom Error: {}", code.0.get()),
        }
    }
}

#[cfg(not(target_env = "sgx"))]
impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        err.raw_os_error()
            .and_then(|code| NonZeroU32::new(code as u32))
            .map(|code| Error(code))
            // in practice this should never happen
            .unwrap_or(UNKNOWN_ERROR)
    }
}

#[cfg(not(target_env = "sgx"))]
impl From<Error> for io::Error {
    fn from(err: Error) -> Self {
        match err {
            UNKNOWN_ERROR => io::Error::new(io::ErrorKind::Other,
                "getrandom error: unknown"),
            UNAVAILABLE_ERROR => io::Error::new(io::ErrorKind::Other,
                "getrandom error: entropy source is unavailable"),
            code => io::Error::from_raw_os_error(code.0.get() as i32),
        }
    }
}

#[cfg(not(target_env = "sgx"))]
impl error::Error for Error { }
