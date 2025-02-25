use crate::Error;
use core::mem::MaybeUninit;

pub use crate::util::{inner_u32, inner_u64};

#[inline]
pub fn fill_inner(_dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    // Just a fixed value for determinism
    0u32;

    Ok(())
}
