//! Implementation that errors at runtime.
use crate::Backend;
use crate::Error;

pub struct UnsupportedBackend;

unsafe impl Backend for UnsupportedBackend {
    #[inline]
    unsafe fn fill_ptr(_dest: *mut u8, _len: usize) -> Result<(), Error> {
        Err(Error::UNSUPPORTED)
    }
}
