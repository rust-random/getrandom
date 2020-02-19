// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! An implementation which calls out to an externally defined function.
use crate::Error;
use core::num::NonZeroU32;

/// Register a function to be invoked by `getrandom` on custom targets.
///
/// This function will only be invoked on targets not supported by `getrandom`.
/// This prevents crate dependencies from either inadvertently or maliciously
/// overriding the secure RNG implementations in `getrandom`.
///
/// *This API requires the following crate features to be activated: `custom`*
#[macro_export]
macro_rules! register_custom_getrandom {
    ($path:path) => {
        // We use an extern "C" function to get the guarantees of a stable ABI.
        #[no_mangle]
        extern "C" fn __getrandom_custom(dest: *mut u8, len: usize) -> u32 {
            let slice = unsafe { ::core::slice::from_raw_parts_mut(dest, len) };
            match $path(slice) {
                Ok(()) => 0,
                Err(e) => e.code().get(),
            }
        }
    };
}

#[allow(dead_code)]
pub fn getrandom_inner(dest: &mut [u8]) -> Result<(), Error> {
    extern "C" {
        fn __getrandom_custom(dest: *mut u8, len: usize) -> u32;
    }
    let ret = unsafe { __getrandom_custom(dest.as_mut_ptr(), dest.len()) };
    match NonZeroU32::new(ret) {
        None => Ok(()),
        Some(code) => Err(Error::from(code)),
    }
}
