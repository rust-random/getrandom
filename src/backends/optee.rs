use crate::Error;
use core::mem::MaybeUninit;

pub use crate::util::{inner_u32, inner_u64};

#[link(name = "utee")]
extern "C" {
    fn TEE_GenerateRandom(randomBuffer: *mut core::ffi::c_void, randomBufferLen: usize);
}

pub fn fill_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    unsafe { TEE_GenerateRandom(dest.as_mut_ptr() as *mut core::ffi::c_void, dest.len()) }
    Ok(())
}
