// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use core::fmt;
use core::num::NonZeroU32;

/// A small and `no_std` compatible error type.
///
/// The [`Error::raw_os_error()`] will indicate if the error is from the OS, and
/// if so, which error code the OS gave the application. If such an error is
/// encountered, please consult with your system documentation.
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Error(NonZeroU32);

// This NonZeroU32 in Error has enough room for two types of errors:
//   - OS Errors:       in range [1, 1 << 31) (i.e. positive i32 values)
//   - Custom Errors:   in range [1 << 31, 1 << 32) (in blocks of 1 << 16)
const CUSTOM_START: u32 = 1 << 31;
const BLOCK_SIZE: u32 = 1 << 16;

impl Error {
    /// Create a new error from a raw OS error number (errno).
    #[inline]
    pub fn from_os_error(errno: i32) -> Self {
        assert!(errno > 0);
        Self(NonZeroU32::new(errno as u32).unwrap())
    }

    /// Crate a custom error in the provided block (group of 2^16 error codes).
    /// The provided block must not be negative, and block 0 is reserved for
    /// custom errors in the `getrandom` crate.
    #[inline]
    pub fn custom_error(block: i16, code: u16) -> Self {
        assert!(block >= 0);
        let n = CUSTOM_START + (block as u16 as u32) * BLOCK_SIZE + (code as u32);
        Self(NonZeroU32::new(n).unwrap())
    }

    /// Extract the raw OS error code (if this error came from the OS)
    ///
    /// This method is identical to `std::io::Error::raw_os_error()`, except
    /// that it works in `no_std` contexts. If this method returns `None`, the
    /// error value can still be formatted via the `Diplay` implementation.
    #[inline]
    pub fn raw_os_error(&self) -> Option<i32> {
        self.try_os_error().ok()
    }

    /// Extract the bare error code.
    ///
    /// This code can either come from the underlying OS, or be a custom error.
    /// Use [`raw_os_error()`] to disambiguate.
    #[inline]
    pub fn code(&self) -> NonZeroU32 {
        self.0
    }

    /// Helper method for creating internal errors
    #[allow(dead_code)]
    pub(crate) fn internal(code: u16) -> Self {
        Self::custom_error(0, code)
    }

    /// Returns either the OS error or a (block, code) pair
    fn try_os_error(&self) -> Result<i32, (i16, u16)> {
        if self.0.get() < CUSTOM_START {
            Ok(self.0.get() as i32)
        } else {
            let offset = self.0.get() - CUSTOM_START;
            Err(((offset / BLOCK_SIZE) as i16, (offset % BLOCK_SIZE) as u16))
        }
    }
}

#[cfg(any(unix, target_os = "redox"))]
fn os_err_desc(errno: i32, buf: &mut [u8]) -> Option<&str> {
    let buf_ptr = buf.as_mut_ptr() as *mut libc::c_char;
    if unsafe { libc::strerror_r(errno, buf_ptr, buf.len()) } != 0 {
        return None;
    }

    // Take up to trailing null byte
    let idx = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    core::str::from_utf8(&buf[..idx]).ok()
}

#[cfg(not(any(unix, target_os = "redox")))]
fn os_err_desc(_errno: i32, _buf: &mut [u8]) -> Option<&str> {
    None
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut dbg = f.debug_struct("Error");
        match self.try_os_error() {
            Ok(errno) => {
                dbg.field("os_error", &errno);
                let mut buf = [0u8; 128];
                if let Some(desc) = os_err_desc(errno, &mut buf) {
                    dbg.field("description", &desc);
                }
            }
            Err((0, code)) => {
                dbg.field("internal_code", &code);
                if let Some(desc) = internal_desc(code) {
                    dbg.field("description", &desc);
                }
            }
            Err((block, code)) => {
                dbg.field("block", &block);
                dbg.field("custom_code", &code);
            }
        }
        dbg.finish()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.try_os_error() {
            Ok(errno) => {
                let mut buf = [0u8; 128];
                match os_err_desc(errno, &mut buf) {
                    Some(desc) => f.write_str(desc),
                    None => write!(f, "OS Error: {}", errno),
                }
            }
            Err((0, code)) => match internal_desc(code) {
                Some(desc) => f.write_str(desc),
                None => write!(f, "Internal Error: {}", code),
            },
            Err((block, code)) => write!(f, "Custom Error: block={}, code={}", block, code),
        }
    }
}

impl From<NonZeroU32> for Error {
    fn from(code: NonZeroU32) -> Self {
        Self(code)
    }
}

/// Internal Error constants
pub(crate) const UNSUPPORTED: u16 = 0;
pub(crate) const UNKNOWN_IO_ERROR: u16 = 1;
pub(crate) const SEC_RANDOM_FAILED: u16 = 2;
pub(crate) const RTL_GEN_RANDOM_FAILED: u16 = 3;
pub(crate) const FAILED_RDRAND: u16 = 4;
pub(crate) const NO_RDRAND: u16 = 5;
pub(crate) const BINDGEN_CRYPTO_UNDEF: u16 = 6;
pub(crate) const BINDGEN_GRV_UNDEF: u16 = 7;
pub(crate) const STDWEB_NO_RNG: u16 = 8;
pub(crate) const STDWEB_RNG_FAILED: u16 = 9;

fn internal_desc(code: u16) -> Option<&'static str> {
    match code {
        UNSUPPORTED => Some("getrandom: this target is not supported"),
        UNKNOWN_IO_ERROR => Some("Unknown std::io::Error"),
        SEC_RANDOM_FAILED => Some("SecRandomCopyBytes: call failed"),
        RTL_GEN_RANDOM_FAILED => Some("RtlGenRandom: call failed"),
        FAILED_RDRAND => Some("RDRAND: failed multiple times: CPU issue likely"),
        NO_RDRAND => Some("RDRAND: instruction not supported"),
        BINDGEN_CRYPTO_UNDEF => Some("wasm-bindgen: self.crypto is undefined"),
        BINDGEN_GRV_UNDEF => Some("wasm-bindgen: crypto.getRandomValues is undefined"),
        STDWEB_NO_RNG => Some("stdweb: no randomness source available"),
        STDWEB_RNG_FAILED => Some("stdweb: failed to get randomness"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::Error;
    use core::mem::size_of;

    #[test]
    fn test_size() {
        assert_eq!(size_of::<Error>(), 4);
        assert_eq!(size_of::<Result<(), Error>>(), 4);
    }
}
