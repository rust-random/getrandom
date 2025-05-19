//! Implementation for iOS, tvOS, and watchOS where `getentropy` is unavailable.
use crate::Backend;
use crate::Error;
use core::{ffi::c_void, mem::MaybeUninit};

pub struct AppleOtherBackend;

unsafe impl Backend for AppleOtherBackend {
    #[inline]
    unsafe fn fill_ptr(dest: *mut u8, len: usize) -> Result<(), Error> {
        let dst_ptr = dest.cast::<c_void>();
        let ret = unsafe { libc::CCRandomGenerateBytes(dst_ptr, len) };
        if ret == libc::kCCSuccess {
            Ok(())
        } else {
            Err(Error::new_custom(IOS_RANDOM_GEN))
        }
    }

    #[inline]
    fn describe_custom_error(n: u16) -> Option<&'static str> {
        if n == IOS_RANDOM_GEN {
            Some("SecRandomCopyBytes: iOS Security framework failure")
        } else {
            None
        }
    }
}

/// Call to `CCRandomGenerateBytes` failed.
const IOS_RANDOM_GEN: u16 = 10;
