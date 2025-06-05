//! Implementation that errors at runtime.
use crate::Error;
use core::mem::MaybeUninit;

pub struct Implementation;

unsafe impl crate::Backend for Implementation {
    #[inline]
    fn fill_uninit(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
        Err(Error::UNSUPPORTED)
    }

    #[inline]
    fn u32() -> Result<u32, Error> {
        Err(Error::UNSUPPORTED)
    }

    #[inline]
    fn u64() -> Result<u64, Error> {
        Err(Error::UNSUPPORTED)
    }
}
