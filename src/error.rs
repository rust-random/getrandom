#[cfg(feature = "std")]
extern crate std;

use core::fmt;

// This private alias mirrors `std::io::RawOsError`:
// https://doc.rust-lang.org/std/io/type.RawOsError.html)
cfg_if::cfg_if!(
    if #[cfg(target_os = "uefi")] {
        // See the UEFI spec for more information:
        // https://uefi.org/specs/UEFI/2.10/Apx_D_Status_Codes.html
        type RawOsError = usize;
        type NonZeroRawOsError = core::num::NonZeroUsize;
        const UEFI_ERROR_FLAG: RawOsError = 1 << (RawOsError::BITS - 1);
    } else {
        type RawOsError = i32;
        type NonZeroRawOsError = core::num::NonZeroI32;
    }
);

/// A small and `no_std` compatible error type
///
/// The [`Error::raw_os_error()`] will indicate if the error is from the OS, and
/// if so, which error code the OS gave the application. If such an error is
/// encountered, please consult with your system documentation.
///
/// Internally this type is a NonZeroU32, with certain values reserved for
/// certain purposes, see [`Error::INTERNAL_START`] and [`Error::CUSTOM_START`].
///
/// *If this crate's `"std"` Cargo feature is enabled*, then:
/// - [`getrandom::Error`][Error] implements
///   [`std::error::Error`](https://doc.rust-lang.org/std/error/trait.Error.html)
/// - [`std::io::Error`](https://doc.rust-lang.org/std/io/struct.Error.html) implements
///   [`From<getrandom::Error>`](https://doc.rust-lang.org/std/convert/trait.From.html).
// note: on non-UEFI targets OS errors are represented as negative integers,
// while on UEFI targets OS errors have the highest bit set to 1.
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Error(NonZeroRawOsError);

impl Error {
    /// This target/platform is not supported by `getrandom`.
    pub const UNSUPPORTED: Error = Self::new_internal(0);
    /// The platform-specific `errno` returned a non-positive value.
    pub const ERRNO_NOT_POSITIVE: Error = Self::new_internal(1);
    /// Encountered an unexpected situation which should not happen in practice.
    pub const UNEXPECTED: Error = Self::new_internal(2);

    /// Deprecated.
    #[deprecated]
    pub const INTERNAL_START: u32 = 1 << 31;

    /// Deprecated.
    #[deprecated]
    pub const CUSTOM_START: u32 = (1 << 31) + (1 << 30);

    /// Internal errors can be in the range of 2^16..2^17
    const INTERNAL_START2: RawOsError = 1 << 16;
    /// Custom errors can be in the range of 2^17..(2^17 + 2^16)
    const CUSTOM_START2: RawOsError = 1 << 17;

    /// Creates a new instance of an `Error` from a particular OS error code.
    ///
    /// This method is analogous to [`std::io::Error::from_raw_os_error()`][1],
    /// except that it works in `no_std` contexts and `code` will be
    /// replaced with `Error::UNEXPECTED` in unexpected cases.
    ///
    /// [1]: https://doc.rust-lang.org/std/io/struct.Error.html#method.from_raw_os_error
    #[allow(dead_code)]
    pub(super) fn from_os_error(code: RawOsError) -> Self {
        match NonZeroRawOsError::new(code) {
            #[cfg(target_os = "uefi")]
            Some(code) if code.get() & UEFI_ERROR_FLAG != 0 => Self(code),
            #[cfg(not(target_os = "uefi"))]
            Some(code) if code.get() < 0 => Self(code),
            _ => Self::UNEXPECTED,
        }
    }

    /// Extract the raw OS error code (if this error came from the OS)
    ///
    /// This method is identical to [`std::io::Error::raw_os_error()`][1], except
    /// that it works in `no_std` contexts. On most targets this method returns
    /// `Option<i32>`, but some platforms (e.g. UEFI) may use a different primitive
    /// type like `usize`. Consult with the [`RawOsError`] docs for more information.
    ///
    /// If this method returns `None`, the error value can still be formatted via
    /// the `Display` implementation.
    ///
    /// [1]: https://doc.rust-lang.org/std/io/struct.Error.html#method.raw_os_error
    /// [`RawOsError`]: https://doc.rust-lang.org/std/io/type.RawOsError.html
    #[inline]
    pub fn raw_os_error(self) -> Option<RawOsError> {
        let code = self.0.get();
        #[cfg(target_os = "uefi")]
        {
            if code & UEFI_ERROR_FLAG != 0 {
                Some(code)
            } else {
                None
            }
        }

        #[cfg(not(target_os = "uefi"))]
        {
            if code >= 0 {
                return None;
            }
            #[cfg(not(target_os = "solid_asp3"))]
            let code = code.checked_neg()?;
            Some(code)
        }
    }

    /// Creates a new instance of an `Error` from a particular custom error code.
    pub const fn new_custom(n: u16) -> Error {
        // SAFETY: code > 0 as CUSTOM_START > 0 and adding `n` won't overflow `RawOsError`.
        let code = Error::CUSTOM_START2 + (n as RawOsError);
        Error(unsafe { NonZeroRawOsError::new_unchecked(code) })
    }

    /// Creates a new instance of an `Error` from a particular internal error code.
    pub(crate) const fn new_internal(n: u16) -> Error {
        // SAFETY: code > 0 as INTERNAL_START > 0 and adding `n` won't overflow `RawOsError`.
        let code = Error::INTERNAL_START2 + (n as RawOsError);
        Error(unsafe { NonZeroRawOsError::new_unchecked(code) })
    }

    fn internal_desc(&self) -> Option<&'static str> {
        let desc = match *self {
            Error::UNSUPPORTED => "getrandom: this target is not supported",
            Error::ERRNO_NOT_POSITIVE => "errno: did not return a positive value",
            Error::UNEXPECTED => "unexpected situation",
            #[cfg(any(
                target_os = "ios",
                target_os = "visionos",
                target_os = "watchos",
                target_os = "tvos",
            ))]
            Error::IOS_RANDOM_GEN => "SecRandomCopyBytes: iOS Security framework failure",
            #[cfg(all(windows, target_vendor = "win7"))]
            Error::WINDOWS_RTL_GEN_RANDOM => "RtlGenRandom: Windows system function failure",
            #[cfg(all(feature = "wasm_js", getrandom_backend = "wasm_js"))]
            Error::WEB_CRYPTO => "Web Crypto API is unavailable",
            #[cfg(target_os = "vxworks")]
            Error::VXWORKS_RAND_SECURE => "randSecure: VxWorks RNG module is not initialized",

            #[cfg(any(
                getrandom_backend = "rdrand",
                all(target_arch = "x86_64", target_env = "sgx")
            ))]
            Error::FAILED_RDRAND => "RDRAND: failed multiple times: CPU issue likely",
            #[cfg(any(
                getrandom_backend = "rdrand",
                all(target_arch = "x86_64", target_env = "sgx")
            ))]
            Error::NO_RDRAND => "RDRAND: instruction not supported",

            #[cfg(getrandom_backend = "rndr")]
            Error::RNDR_FAILURE => "RNDR: Could not generate a random number",
            #[cfg(getrandom_backend = "rndr")]
            Error::RNDR_NOT_AVAILABLE => "RNDR: Register not supported",
            _ => return None,
        };
        Some(desc)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dbg = f.debug_struct("Error");
        if let Some(errno) = self.raw_os_error() {
            dbg.field("os_error", &errno);
            #[cfg(feature = "std")]
            dbg.field("description", &std::io::Error::from_raw_os_error(errno));
        } else if let Some(desc) = self.internal_desc() {
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
            cfg_if! {
                if #[cfg(feature = "std")] {
                    std::io::Error::from_raw_os_error(errno).fmt(f)
                } else {
                    write!(f, "OS Error: {}", errno)
                }
            }
        } else if let Some(desc) = self.internal_desc() {
            f.write_str(desc)
        } else {
            write!(f, "Unknown Error: {}", self.0.get())
        }
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
