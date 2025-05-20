//! Implementation that errors at runtime.
use crate::Backend;
use crate::Error;

pub struct UnsupportedBackend;

unsafe impl Backend for UnsupportedBackend {
    #[inline]
    unsafe fn fill_ptr(dest: *mut u8, len: usize) -> Result<(), Error> {
        Err(Error::UNSUPPORTED)
    }
}
