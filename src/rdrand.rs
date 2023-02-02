// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for SGX using RDRAND instruction
use crate::{
    util::{slice_as_uninit, LazyBool},
    Error,
};
use core::mem::{size_of, MaybeUninit};

cfg_if! {
    if #[cfg(target_arch = "x86_64")] {
        use core::arch::x86_64 as arch;
        use arch::_rdrand64_step as rdrand_step;
    } else if #[cfg(target_arch = "x86")] {
        use core::arch::x86 as arch;
        use arch::_rdrand32_step as rdrand_step;
    }
}

// Recommendation from "Intel® Digital Random Number Generator (DRNG) Software
// Implementation Guide" - Section 5.2.1 and "Intel® 64 and IA-32 Architectures
// Software Developer’s Manual" - Volume 1 - Section 7.3.17.1.
const RETRY_LIMIT: usize = 10;

#[target_feature(enable = "rdrand")]
unsafe fn rdrand() -> Option<usize> {
    for _ in 0..RETRY_LIMIT {
        let mut val = 0;
        if rdrand_step(&mut val) == 1 {
            return Some(val as usize);
        }
    }
    None
}

// "rdrand" target feature requires "+rdrand" flag, see https://github.com/rust-lang/rust/issues/49653.
#[cfg(all(target_env = "sgx", not(target_feature = "rdrand")))]
compile_error!(
    "SGX targets require 'rdrand' target feature. Enable by using -C target-feature=+rdrand."
);

#[cfg(target_feature = "rdrand")]
fn is_rdrand_supported() -> bool {
    true
}

// TODO use is_x86_feature_detected!("rdrand") when that works in core. See:
// https://github.com/rust-lang-nursery/stdsimd/issues/464
#[cfg(not(target_feature = "rdrand"))]
fn is_rdrand_supported() -> bool {
    use crate::util::LazyBool;

    // SAFETY: All Rust x86 targets are new enough to have CPUID, and if CPUID
    // is supported, CPUID leaf 1 is always supported.
    const FLAG: u32 = 1 << 30;
    static HAS_RDRAND: LazyBool = LazyBool::new();
    HAS_RDRAND.unsync_init(|| unsafe { (arch::__cpuid(1).ecx & FLAG) != 0 })
}

pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    if !is_rdrand_supported() {
        return Err(Error::NO_RDRAND);
    }
    rdrand_exact(dest).ok_or(Error::FAILED_RDRAND)
}

fn rdrand_exact(dest: &mut [MaybeUninit<u8>]) -> Option<()> {
    // We use chunks_exact_mut instead of chunks_mut as it allows almost all
    // calls to memcpy to be elided by the compiler.
    let mut chunks = dest.chunks_exact_mut(size_of::<usize>());
    for chunk in chunks.by_ref() {
        // SAFETY: After this point, we know rdrand is supported, so calling
        // rdrand is not undefined behavior.
        let src = unsafe { rdrand() }?.to_ne_bytes();
        chunk.copy_from_slice(slice_as_uninit(&src));
    }

    let tail = chunks.into_remainder();
    let n = tail.len();
    if n > 0 {
        let src = unsafe { rdrand() }?.to_ne_bytes();
        tail.copy_from_slice(slice_as_uninit(&src[..n]));
    }
    Some(())
}
