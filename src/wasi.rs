//! Implementation for WASI
use crate::Error;
use core::mem::MaybeUninit;
use wasi::random_get;

pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    unsafe { random_get(dest.as_mut_ptr().cast::<u8>(), dest.len()) }
        .map_err(|e| Error::from_os_error(e.raw().into()))
}
