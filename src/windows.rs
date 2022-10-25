// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::Error;
use core::{convert::TryInto, ffi::c_void, mem::MaybeUninit, num::NonZeroU32, ptr};

const BCRYPT_USE_SYSTEM_PREFERRED_RNG: u32 = 0x00000002;

#[link(name = "bcrypt")]
extern "system" {
    fn BCryptGenRandom(
        hAlgorithm: *mut c_void,
        pBuffer: *mut u8,
        cbBuffer: u32,
        dwFlags: u32,
    ) -> u32;
}

// BCryptGenRandom was introduced in Windows Vista. However, CNG Algorithm
// Pseudo-handles (specifically BCRYPT_RNG_ALG_HANDLE) weren't introduced
// until Windows 10, so we cannot use them yet. Note that on older systems
// these Pseudo-handles are interpreted as pointers, causing crashes if used.
fn bcrypt_random(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    // Will always succeed given the chunking in getrandom_inner().
    let len: u32 = dest.len().try_into().unwrap();
    // SAFETY: dest is valid, writable buffer of length len
    let ret = unsafe {
        BCryptGenRandom(
            ptr::null_mut(),
            dest.as_mut_ptr() as *mut u8,
            len,
            BCRYPT_USE_SYSTEM_PREFERRED_RNG,
        )
    };

    // NTSTATUS codes use the two highest bits for severity status.
    if ret >> 30 != 0b11 {
        return Ok(());
    }
    // We zeroize the highest bit, so the error code will reside
    // inside the range designated for OS codes.
    let code = ret ^ (1 << 31);
    // SAFETY: the second highest bit is always equal to one,
    // so it's impossible to get zero. Unfortunately the type
    // system does not have a way to express this yet.
    let code = unsafe { NonZeroU32::new_unchecked(code) };
    Err(Error::from(code))
}

pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    // Prevent overflow of u32
    for chunk in dest.chunks_mut(u32::max_value() as usize) {
        bcrypt_random(chunk)?;
    }
    Ok(())
}
