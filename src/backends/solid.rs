//! Implementation for SOLID
use crate::Backend;
use crate::Error;

extern "C" {
    pub fn SOLID_RNG_SampleRandomBytes(buffer: *mut u8, length: usize) -> i32;
}

pub struct SolidBackend;

unsafe impl Backend for SolidBackend {
    #[inline]
    unsafe fn fill_ptr(dest: *mut u8, len: usize) -> Result<(), Error> {
        let ret = unsafe { SOLID_RNG_SampleRandomBytes(dest, len) };
        if ret >= 0 {
            Ok(())
        } else {
            Err(Error::from_neg_error_code(ret))
        }
    }
}
