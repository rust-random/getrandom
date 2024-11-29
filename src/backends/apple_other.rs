//! Implementation for iOS, tvOS, and watchOS where `getentropy` is unavailable.
use crate::Error;
use core::{ffi::c_void, mem::MaybeUninit};

pub use crate::default_impls::{insecure_fill_uninit, insecure_u32, insecure_u64, u32, u64};

pub fn fill_uninit(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    let dst_ptr = dest.as_mut_ptr().cast::<c_void>();
    let ret = unsafe { libc::CCRandomGenerateBytes(dst_ptr, dest.len()) };
    if ret == libc::kCCSuccess {
        Ok(())
    } else {
        Err(Error::IOS_SEC_RANDOM)
    }
}
