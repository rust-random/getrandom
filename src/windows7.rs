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

// Binding to the Windows.Win32.Security.Authentication.Identity.RtlGenRandom
// API. Don't use windows-targets as it doesn't support Windows 7 targets.
#[link(name = "advapi32")]
extern "system" {
    #[link_name = "SystemFunction036"]
    fn RtlGenRandom(randombuffer: *mut c_void, randombufferlength: u32) -> BOOLEAN;
}
type BOOLEAN = u8;
const TRUE: BOOLEAN = 1u8;

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
