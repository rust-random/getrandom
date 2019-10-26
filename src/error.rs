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
///
/// Internally this type is a NonZeroU32, with certain values reserved for
/// certain purposes, see [`Error::INTERNAL_START`] and [`Error::CUSTOM_START`].
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Error(NonZeroU32);

// TODO: Convert to a function when min_version >= 1.33
macro_rules! internal_error {
    ($n:expr) => {
        Error(unsafe { NonZeroU32::new_unchecked(Error::INTERNAL_START + $n as u16 as u32) })
    };
}

impl Error {
    /// This target/platform is not supported by `getrandom`.
    pub const UNSUPPORTED: Error = internal_error!(0);
    /// The platform-specific `errno` returned a non-positive value.
    pub const ERRNO_NOT_POSITIVE: Error = internal_error!(1);
    /// Invalid conversion from a non-standard [`std::io::Error`](https://doc.rust-lang.org/std/io/struct.Error.html)
    pub const UNKNOWN_IO_ERROR: Error = internal_error!(2);
    /// Call to [`SecRandomCopyBytes`](https://developer.apple.com/documentation/security/1399291-secrandomcopybytes) failed.
    pub const SEC_RANDOM_FAILED: Error = internal_error!(3);
    /// Call to [`RtlGenRandom`](https://docs.microsoft.com/en-us/windows/win32/api/ntsecapi/nf-ntsecapi-rtlgenrandom) failed.
    pub const RTL_GEN_RANDOM_FAILED: Error = internal_error!(4);
    /// RDRAND instruction failed due to a hardware issue.
    pub const FAILED_RDRAND: Error = internal_error!(5);
    /// RDRAND instruction unsupported on this target.
    pub const NO_RDRAND: Error = internal_error!(6);
    /// Using `wasm-bindgen`, browser does not support `self.crypto`.
    pub const BINDGEN_CRYPTO_UNDEF: Error = internal_error!(7);
    /// Using `wasm-bindgen`, browser does not support `crypto.getRandomValues`.
    pub const BINDGEN_GRV_UNDEF: Error = internal_error!(8);
    /// Using `stdweb`, no cryptographic RNG is available.
    pub const STDWEB_NO_RNG: Error = internal_error!(9);
    /// Using `stdweb`, invoking a cryptographic RNG failed.
    pub const STDWEB_RNG_FAILED: Error = internal_error!(10);
    /// On VxWorks, random number generator is not yet initialized.
    pub const RAND_SECURE_FATAL: Error = internal_error!(11);

    /// Codes below this point represent OS Errors (i.e. positive i32 values).
    /// Codes at or above this point, but below [`Error::CUSTOM_START`] are
    /// reserved for use by the `rand` and `getrandom` crates.
    pub const INTERNAL_START: u32 = 1 << 31;

    /// Codes at or above this point can be used by users to define their own
    /// custom errors.
    pub const CUSTOM_START: u32 = (1 << 31) + (1 << 30);

    /// Extract the raw OS error code (if this error came from the OS)
    ///
    /// This method is identical to [`std::io::Error::raw_os_error()`][1], except
    /// that it works in `no_std` contexts. If this method returns `None`, the
    /// error value can still be formatted via the `Display` implementation.
    ///
    /// [1]: https://doc.rust-lang.org/std/io/struct.Error.html#method.raw_os_error
    #[inline]
    pub fn raw_os_error(self) -> Option<i32> {
        if self.0.get() < Self::INTERNAL_START {
            Some(self.0.get() as i32)
        } else {
            None
        }
    }

    /// Extract the bare error code.
    ///
    /// This code can either come from the underlying OS, or be a custom error.
    /// Use [`Error::raw_os_error()`] to disambiguate.
    #[inline]
    pub fn code(self) -> NonZeroU32 {
        self.0
    }
}

cfg_if! {
    if #[cfg(unix)] {
        fn os_err_desc(errno: i32, buf: &mut [u8]) -> Option<&str> {
            let buf_ptr = buf.as_mut_ptr() as *mut libc::c_char;
            if unsafe { libc::strerror_r(errno, buf_ptr, buf.len()) } != 0 {
                return None;
            }

            // Take up to trailing null byte
            let n = buf.len();
            let idx = buf.iter().position(|&b| b == 0).unwrap_or(n);
            core::str::from_utf8(&buf[..idx]).ok()
        }
    } else if #[cfg(target_os = "wasi")] {
        fn os_err_desc(errno: i32, _buf: &mut [u8]) -> Option<&str> {
            core::num::NonZeroU16::new(errno as u16)
                .and_then(wasi::wasi_unstable::error_str)
        }
    } else {
        fn os_err_desc(_errno: i32, _buf: &mut [u8]) -> Option<&str> {
            None
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dbg = f.debug_struct("Error");
        if let Some(errno) = self.raw_os_error() {
            dbg.field("os_error", &errno);
            let mut buf = [0u8; 128];
            if let Some(desc) = os_err_desc(errno, &mut buf) {
                dbg.field("description", &desc);
            }
        } else if let Some(desc) = internal_desc(*self) {
            dbg.field("internal_code", &self.0.get());
            dbg.field("description", &desc);
        } else {
            dbg.field("unknown_code", &self.0.get());
        }
        dbg.finish()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(errno) = self.raw_os_error() {
            let mut buf = [0u8; 128];
            match os_err_desc(errno, &mut buf) {
                Some(desc) => f.write_str(desc),
                None => write!(f, "OS Error: {}", errno),
            }
        } else if let Some(desc) = internal_desc(*self) {
            f.write_str(desc)
        } else {
            write!(f, "Unknown Error: {}", self.0.get())
        }
    }
}

impl From<NonZeroU32> for Error {
    fn from(code: NonZeroU32) -> Self {
        Self(code)
    }
}

fn internal_desc(error: Error) -> Option<&'static str> {
    match error {
        Error::UNSUPPORTED => Some("getrandom: this target is not supported"),
        Error::ERRNO_NOT_POSITIVE => Some("errno: did not return a positive value"),
        Error::UNKNOWN_IO_ERROR => Some("Unknown std::io::Error"),
        Error::SEC_RANDOM_FAILED => Some("SecRandomCopyBytes: call failed"),
        Error::RTL_GEN_RANDOM_FAILED => Some("RtlGenRandom: call failed"),
        Error::FAILED_RDRAND => Some("RDRAND: failed multiple times: CPU issue likely"),
        Error::NO_RDRAND => Some("RDRAND: instruction not supported"),
        Error::BINDGEN_CRYPTO_UNDEF => Some("wasm-bindgen: self.crypto is undefined"),
        Error::BINDGEN_GRV_UNDEF => Some("wasm-bindgen: crypto.getRandomValues is undefined"),
        Error::STDWEB_NO_RNG => Some("stdweb: no randomness source available"),
        Error::STDWEB_RNG_FAILED => Some("stdweb: failed to get randomness"),
        Error::RAND_SECURE_FATAL => Some("randSecure: VxWorks RNG module is not initialized"),
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
