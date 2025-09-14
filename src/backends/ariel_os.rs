/// Implementation for Ariel OS
use ariel_os_random;
use core::ffi::c_void;
use crate::Error;
use core::mem::MaybeUninit;
use rand::RngCore;

pub use crate::util::{inner_u32, inner_u64};

pub fn fill_inner(_dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    let mut rng = ariel_os_random::crypto_rng();
    let buf: &mut [u8] =
        unsafe { core::slice::from_raw_parts_mut(_dest.as_mut_ptr().cast::<u8>(), _dest.len()) };
    rng.try_fill_bytes(buf)
        .map_err(|e| Error::from_neg_error_code(e.raw_os_error().unwrap()))
}
