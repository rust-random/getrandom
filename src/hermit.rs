// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for Hermit
use crate::Error;
use hermit_abi::secure_rand64;

pub fn getrandom_inner(dest: &mut [u8]) -> Result<(), Error> {
    unsafe {
        let mut chunks = dest.chunks_exact_mut(8);
        for chunk in &mut chunks {
            let bytes = secure_rand64()
                .ok_or(Error::HERMIT_NO_HARDWARE)?
                .to_ne_bytes();
            chunk.copy_from_slice(&bytes);
        }
        let rem = chunks.into_remainder();
        if !rem.is_empty() {
            let bytes = secure_rand64()
                .ok_or(Error::HERMIT_NO_HARDWARE)?
                .to_ne_bytes();
            rem.copy_from_slice(&bytes[..rem.len()]);
        }
    }
    Ok(())
}
