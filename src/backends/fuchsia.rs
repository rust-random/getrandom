//! Implementation for Fuchsia Zircon
use crate::Error;
use core::mem::MaybeUninit;

pub use crate::default_impls::{insecure_fill_uninit, insecure_u32, insecure_u64, u32, u64};

#[link(name = "zircon")]
extern "C" {
    fn zx_cprng_draw(buffer: *mut u8, length: usize);
}

pub fn fill_uninit(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    unsafe { zx_cprng_draw(dest.as_mut_ptr().cast::<u8>(), dest.len()) }
    Ok(())
}
