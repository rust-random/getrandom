//! Linux-only safe RNDR register backend for aarch64 targets with fallback

#[cfg(any(
    not(any(target_os = "linux", target_os = "android")),
    not(target_feature = "rand"),
    not(target_arch = "aarch64")
))]
compile_error!(
    "The rndr_with_fallback backend requires the `rand` target feature to be enabled
    at compile time and can only be built for Linux or Android."
);

use crate::{lazy::LazyBool, linux_android_with_fallback, rndr, Error};
use core::arch::asm;
use core::mem::MaybeUninit;

// Check whether FEAT_RNG is available on the system
//
// Requires the caller either be running in EL1 or be on a system supporting MRS emulation.
// Due to the above, the implementation is currently restricted to Linux.
fn is_rndr_available() -> bool {
    let mut id_aa64isar0: u64;

    // If FEAT_RNG is implemented, ID_AA64ISAR0_EL1.RNDR (bits 60-63) are 0b0001
    // This is okay to do from EL0 in Linux because Linux will emulate MRS as per
    // https://docs.kernel.org/arch/arm64/cpu-feature-registers.html
    unsafe {
        asm!(
            "mrs {id}, ID_AA64ISAR0_EL1",
            id = out(reg) id_aa64isar0,
        );
    }

    (id_aa64isar0 >> 60) & 0xf >= 1
}

pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    static RNDR_AVAILABLE: LazyBool = LazyBool::new();
    if !RNDR_AVAILABLE.unsync_init(is_rndr_available) {
        return Err(Error::NO_RNDR);
    }

    // We've already checked that RNDR is available
    if rndr::getrandom_inner(dest).is_ok() {
        Ok(())
    } else {
        linux_android_with_fallback::getrandom_inner(dest)
    }
}
