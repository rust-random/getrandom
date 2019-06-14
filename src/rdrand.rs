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
use core::arch::x86_64::{__cpuid, _rdrand64_step};
use core::num::NonZeroU32;
use lazy_static::lazy_static;

// Recommendation from "Intel® Digital Random Number Generator (DRNG) Software
// Implementation Guide" - Section 5.2.1 and "Intel® 64 and IA-32 Architectures
// Software Developer’s Manual" - Volume 1 - Section 7.3.17.1.
const RETRY_LIMIT: usize = 10;
const WORD_SIZE: usize = mem::size_of::<u64>();

#[target_feature(enable = "rdrand")]
unsafe fn rdrand() -> Result<[u8; WORD_SIZE], Error> {
    for _ in 0..RETRY_LIMIT {
        let mut el = mem::uninitialized();
        if _rdrand64_step(&mut el) == 1 {
            return Ok(el.to_ne_bytes());
        }
    }
    error!("RDRAND failed, CPU issue likely");
    Err(Error::UNKNOWN)
}

// TODO use is_x86_feature_detected!("rdrand") when that works in core. See:
//   https://github.com/rust-lang-nursery/stdsimd/issues/464
fn is_rdrand_supported() -> bool {
    if cfg!(target_feature = "rdrand") {
        true
    } else if cfg!(target_env = "sgx") {
        false // No CPUID in SGX enclaves
    } else {
        // SAFETY: All x86_64 CPUs support CPUID leaf 1
        const FLAG: u32 = 1 << 30;
        lazy_static! {
            static ref HAS_RDRAND: bool = unsafe { __cpuid(1).ecx & FLAG != 0 };
        }
        *HAS_RDRAND
    }
}

pub fn getrandom_inner(dest: &mut [u8]) -> Result<(), Error> {
    if !is_rdrand_supported() {
        return Err(Error::UNAVAILABLE);
    }

    // SAFETY: After this point, rdrand is supported, so calling the rdrand
    // functions is not undefined behavior.
    unsafe { rdrand_exact(dest) }
}

#[target_feature(enable = "rdrand")]
unsafe fn rdrand_exact(dest: &mut [u8]) -> Result<(), Error> {
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
