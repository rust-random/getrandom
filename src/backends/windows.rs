//! Implementation for Windows 10 and later
//!
//! On Windows 10 and later, ProcessPrng "is the primary interface to the
//! user-mode per-processer PRNGs" and only requires bcryptprimitives.dll,
//! making it a better option than the other Windows RNG APIs:
//!   - BCryptGenRandom: https://learn.microsoft.com/en-us/windows/win32/api/bcrypt/nf-bcrypt-bcryptgenrandom
//!     - Requires bcrypt.dll (which loads bcryptprimitives.dll anyway)
//!     - Can cause crashes/hangs as BCrypt accesses the Windows Registry:
//!       https://github.com/rust-lang/rust/issues/99341
//!     - Causes issues inside sandboxed code:
//!       https://issues.chromium.org/issues/40277768
//!   - CryptGenRandom: https://learn.microsoft.com/en-us/windows/win32/api/wincrypt/nf-wincrypt-cryptgenrandom
//!     - Deprecated and not available on UWP targets
//!     - Requires advapi32.lib/advapi32.dll (in addition to bcryptprimitives.dll)
//!     - Thin wrapper around ProcessPrng
//!   - RtlGenRandom: https://learn.microsoft.com/en-us/windows/win32/api/ntsecapi/nf-ntsecapi-rtlgenrandom
//!     - Deprecated and not available on UWP targets
//!     - Requires advapi32.dll (in addition to bcryptprimitives.dll)
//!     - Requires using name "SystemFunction036"
//!     - Thin wrapper around ProcessPrng
//!
//! For more information see the Windows RNG Whitepaper: https://aka.ms/win10rng
use crate::Error;
use core::mem::MaybeUninit;

pub use crate::default_impls::{insecure_fill_uninit, insecure_u32, insecure_u64, u32, u64};

// Binding to the Windows.Win32.Security.Cryptography.ProcessPrng API. As
// bcryptprimitives.dll lacks an import library, we use the windows-targets
// crate to link to it.
windows_targets::link!("bcryptprimitives.dll" "system" fn ProcessPrng(pbdata: *mut u8, cbdata: usize) -> BOOL);
#[allow(clippy::upper_case_acronyms)]
pub type BOOL = i32;
pub const TRUE: BOOL = 1i32;

pub fn fill_uninit(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    // ProcessPrng should always return TRUE, but we check just in case.
    match unsafe { ProcessPrng(dest.as_mut_ptr().cast::<u8>(), dest.len()) } {
        TRUE => Ok(()),
        _ => Err(Error::WINDOWS_PROCESS_PRNG),
    }
}

impl Error {
    /// Calling Windows ProcessPrng failed.
    pub(crate) const WINDOWS_PROCESS_PRNG: Error = Self::new_internal(10);
}
