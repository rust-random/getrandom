// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for SGX using RDRAND instruction
use crate::Error;
use core::mem;
use core::arch::x86_64::_rdrand64_step;
use core::num::NonZeroU32;

#[cfg(not(target_feature = "rdrand"))]
compile_error!("enable rdrand target feature!");

// Recommendation from "Intel® Digital Random Number Generator (DRNG) Software
// Implementation Guide" - Section 5.2.1 and "Intel® 64 and IA-32 Architectures
// Software Developer’s Manual" - Volume 1 - Section 7.3.17.1.
const RETRY_LIMIT: usize = 10;
const WORD_SIZE: usize = mem::size_of::<u64>();

fn rdrand() -> Result<[u8; WORD_SIZE], Error> {
    for _ in 0..RETRY_LIMIT {
        unsafe {
            // SAFETY: we've checked RDRAND support, and u64 can have any value.
            let mut el = mem::uninitialized();
            if _rdrand64_step(&mut el) == 1 {
                return Ok(el.to_ne_bytes());
            }
        };
    }
    error!("RDRAND failed, CPU issue likely");
    Err(Error::UNKNOWN)
}

pub fn getrandom_inner(dest: &mut [u8]) -> Result<(), Error> {
    // We use chunks_exact_mut instead of chunks_mut as it allows almost all
    // calls to memcpy to be elided by the compiler.
    let mut chunks = dest.chunks_exact_mut(WORD_SIZE);
    for chunk in chunks.by_ref() {
        chunk.copy_from_slice(&rdrand()?);
    }

    let tail = chunks.into_remainder();
    let n = tail.len();
    if n > 0 {
        tail.copy_from_slice(&rdrand()?[..n]);
    }
    Ok(())
}

#[inline(always)]
pub fn error_msg_inner(_: NonZeroU32) -> Option<&'static str> { None }
