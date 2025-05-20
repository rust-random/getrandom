//! Implementation for Fuchsia Zircon
use crate::Backend;
use crate::Error;

#[link(name = "zircon")]
extern "C" {
    fn zx_cprng_draw(buffer: *mut u8, length: usize);
}

pub struct FuchsiaBackend;

unsafe impl Backend for FuchsiaBackend {
    #[inline]
    unsafe fn fill_ptr(dest: *mut u8, len: usize) -> Result<(), Error> {
        zx_cprng_draw(dest, len);
        Ok(())
    }
}
