//! Implementation for Windows 10 and later
//!
//! On Windows 10 and later, ProcessPrng "is the primary interface to the
//! user-mode per-processer PRNGs" and only requires BCryptPrimitives.dll,
//! making it a better option than the other Windows RNG APIs:
//!   - BCryptGenRandom: https://learn.microsoft.com/en-us/windows/win32/api/bcrypt/nf-bcrypt-bcryptgenrandom
//!     - Requires Bcrypt.dll (which loads BCryptPrimitives.dll anyway)
//!     - Can cause crashes/hangs as BCrypt accesses the Windows Registry:
//!       https://github.com/rust-lang/rust/issues/99341
//!     - Causes issues inside sandboxed code:
//!       https://issues.chromium.org/issues/40277768
//!   - CryptGenRandom: https://learn.microsoft.com/en-us/windows/win32/api/wincrypt/nf-wincrypt-cryptgenrandom
//!     - Deprecated and not available on UWP targets
//!     - Requires Advapi32.lib/Advapi32.dll
//!     - Wrapper around ProcessPrng
//!   - RtlGenRandom: https://learn.microsoft.com/en-us/windows/win32/api/ntsecapi/nf-ntsecapi-rtlgenrandom
//!     - Deprecated and not available on UWP targets
//!     - Requires Advapi32.dll (and using name "SystemFunction036")
//!     - Wrapper around ProcessPrng
//! For more information see the Windows RNG Whitepaper: https://aka.ms/win10rng
use crate::Error;
use core::mem::MaybeUninit;

// ProcessPrng lacks an import library, so we use the windows-targets crate to
// link to it. The following code was generated via windows-bindgen with APIs:
//   Windows.Win32.Foundation.TRUE
//   Windows.Win32.Security.Cryptography.ProcessPrng
windows_targets::link!("bcryptprimitives.dll" "system" fn ProcessPrng(pbdata: *mut u8, cbdata: usize) -> BOOL);
#[repr(transparent)]
#[derive(PartialEq, Eq)]
pub struct BOOL(pub i32);
pub const TRUE: BOOL = BOOL(1i32);

pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    // ProcessPrng should always return TRUE, but we check just in case.
    match unsafe { ProcessPrng(dest.as_mut_ptr().cast::<u8>(), dest.len()) } {
        TRUE => Ok(()),
        _ => Err(Error::WINDOWS_PROCESS_PRNG),
    }
}
