//! An implementation which calls out to an externally defined function.
use crate::Backend;
use crate::Error;

pub struct LegacyCustomBackend;

unsafe impl Backend for LegacyCustomBackend {
    #[inline]
    unsafe fn fill_ptr(dest: *mut u8, len: usize) -> Result<(), Error> {
        extern "Rust" {
            fn __getrandom_v03_custom(dest: *mut u8, len: usize) -> Result<(), Error>;
        }
        __getrandom_v03_custom(dest, len)
    }
}
