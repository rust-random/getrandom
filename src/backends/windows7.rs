//! Legacy implementation for Windows XP and later
//!
//! For targets where we cannot use ProcessPrng (added in Windows 10), we use
//! RtlGenRandom. See windows.rs for a more detailed discussion of the Windows
//! RNG APIs (and why we don't use BCryptGenRandom). On versions prior to
//! Windows 10, this implementation is secure. On Windows 10 and later, this
//! implementation behaves identically to the windows.rs implementation, except
//! that it forces the loading of an additonal DLL (advapi32.dll).
//!
//! This implementation will not work on UWP targets (which lack advapi32.dll),
//! but such targets require Windows 10, so can use the standard implementation.
use crate::Error;
use core::{ffi::c_void, mem::MaybeUninit};

use crate::Backend;

pub struct WindowsLegacyBackend;

unsafe impl Backend for WindowsLegacyBackend {
    #[inline]
    unsafe fn fill_ptr(dest: *mut u8, len: usize) -> Result<(), Error> {
        let slice = core::slice::from_raw_parts_mut(dest.cast(), len);
        Self::fill_uninit(slice)
    }

    #[inline]
    fn fill_uninit(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
        // Prevent overflow of u32
        let chunk_size =
            usize::try_from(i32::MAX).expect("Windows does not support 16-bit targets");
        for chunk in dest.chunks_mut(chunk_size) {
            let chunk_len = u32::try_from(chunk.len()).expect("chunk size is bounded by i32::MAX");
            let ret = unsafe { RtlGenRandom(chunk.as_mut_ptr().cast::<c_void>(), chunk_len) };
            if ret != TRUE {
                return Err(Error::new_custom(WINDOWS_RTL_GEN_RANDOM));
            }
        }
        Ok(())
    }

    #[inline]
    fn describe_custom_error(n: u16) -> Option<&'static str> {
        if n == WINDOWS_RTL_GEN_RANDOM {
            Some("RtlGenRandom: Windows system function failure")
        } else {
            None
        }
    }
}

// Binding to the Windows.Win32.Security.Authentication.Identity.RtlGenRandom
// API. Don't use windows-targets as it doesn't support Windows 7 targets.
#[link(name = "advapi32")]
extern "system" {
    #[link_name = "SystemFunction036"]
    fn RtlGenRandom(randombuffer: *mut c_void, randombufferlength: u32) -> BOOLEAN;
}
#[allow(clippy::upper_case_acronyms)]
type BOOLEAN = u8;
const TRUE: BOOLEAN = 1u8;

/// Call to Windows [`RtlGenRandom`](https://docs.microsoft.com/en-us/windows/win32/api/ntsecapi/nf-ntsecapi-rtlgenrandom) failed.
const WINDOWS_RTL_GEN_RANDOM: u16 = 10;
