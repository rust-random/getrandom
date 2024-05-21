//! Implementation for Windows 7 and 8
//!
//! For targets where we cannot use ProcessPrng (added in Windows 10), we use
//! RtlGenRandom. See windows.rs for a more detailed discussion of the Windows
//! RNG APIs (and why we don't use BCryptGenRandom). On versions prior to
//! Windows 10, this implementation works, while on Windows 10 and later, this
//! is a thin wrapper around ProcessPrng.
//!
//! This implementation will not work on UWP targets, but those targets require
//! Windows 10 regardless, so can use the ProcessPrng implementation.
use crate::Error;
use core::{ffi::c_void, mem::MaybeUninit};

// This code is based on that produced by windows-bindgen with the APIs:
//   Windows.Win32.Foundation.TRUE
//   Windows.Win32.Security.Authentication.Identity.RtlGenRandom
// but we avoid using windows-targets as it doesn't support older Windows.
#[link(name = "advapi32")]
extern "system" {
    #[link_name = "SystemFunction036"]
    fn RtlGenRandom(randombuffer: *mut c_void, randombufferlength: u32) -> BOOLEAN;
}
#[repr(transparent)]
#[derive(PartialEq, Eq)]
pub struct BOOLEAN(pub u8);
pub const TRUE: BOOLEAN = BOOLEAN(1u8);

pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    // Prevent overflow of u32
    for chunk in dest.chunks_mut(u32::max_value() as usize) {
        let ret = unsafe { RtlGenRandom(chunk.as_mut_ptr().cast::<c_void>(), chunk.len() as u32) };
        if ret != TRUE {
            return Err(Error::WINDOWS_RTL_GEN_RANDOM);
        }
    }
    Ok(())
}
