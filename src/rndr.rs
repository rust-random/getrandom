//! RNDR register backend for aarch64 targets
// Arm Architecture Reference Manual for A-profile architecture
// ARM DDI 0487K.a, ID032224, D23.2.147 RNDR, Random Number

#[cfg(any(not(target_feature = "rand"), not(target_arch = "aarch64")))]
compile_error!("The RNDR backend requires the `rand` target feature to be enabled at compile time");

use crate::{util::slice_as_uninit, Error};
use core::arch::asm;
use core::mem::{size_of, MaybeUninit};

const RETRY_LIMIT: usize = 5;

// Read a random number from the aarch64 rndr register
//
// Callers must ensure that FEAT_RNG is available on the system
// The function assumes that the RNDR register is available
// If it fails to read a random number, it will retry up to 5 times
// After 5 failed reads the function will return None
#[target_feature(enable = "rand")]
unsafe fn rndr() -> Option<u64> {
    for _ in 0..RETRY_LIMIT {
        let mut x: u64;
        let mut nzcv: u64;

        // AArch64 RNDR register is accessible by s3_3_c2_c4_0
        asm!(
            "mrs {x}, RNDR",
            "mrs {nzcv}, NZCV",
            x = out(reg) x,
            nzcv = out(reg) nzcv,
        );

        // If the hardware returns a genuine random number, PSTATE.NZCV is set to 0b0000
        if nzcv == 0 {
            return Some(x);
        }
    }

    None
}

pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    unsafe { rndr_exact(dest).ok_or(Error::FAILED_RNDR) }
}

#[target_feature(enable = "rand")]
unsafe fn rndr_exact(dest: &mut [MaybeUninit<u8>]) -> Option<()> {
    let mut chunks = dest.chunks_exact_mut(size_of::<u64>());
    for chunk in chunks.by_ref() {
        let src = rndr()?.to_ne_bytes();
        chunk.copy_from_slice(slice_as_uninit(&src));
    }

    let tail = chunks.into_remainder();
    let n = tail.len();
    if n > 0 {
        let src = rndr()?.to_ne_bytes();
        tail.copy_from_slice(slice_as_uninit(&src[..n]));
    }
    Some(())
}
