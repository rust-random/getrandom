//! Implementation that's not random and does a no-op.
use crate::Error;
use core::mem::MaybeUninit;

pub use crate::util::{inner_u32, inner_u64};

pub fn fill_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    for byte in dest {
        byte.write(0);
    }
    Ok(())
}
