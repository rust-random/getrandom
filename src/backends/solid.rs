//! Implementation for SOLID
use crate::Error;
use core::mem::MaybeUninit;

extern "C" {
    pub fn SOLID_RNG_SampleRandomBytes(buffer: *mut u8, length: usize) -> i32;
}

pub struct Implementation;

unsafe impl crate::Backend for Implementation {
    #[inline]
    fn fill_uninit(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
        let ret =
            unsafe { SOLID_RNG_SampleRandomBytes(dest.as_mut_ptr().cast::<u8>(), dest.len()) };
        if ret >= 0 {
            Ok(())
        } else {
            Err(Error::from_neg_error_code(ret))
        }
    }
}
