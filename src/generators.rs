//! Access functions for specific random number generators
#![allow(unused_imports)]

use crate::getrandom as default_getrandom;
use crate::util::slice_as_uninit_mut;
use crate::Error;

// Shared macro to generate access functions for each backend module
macro_rules! define_generator_fns {
    ($($module:ident),+) => {$(
        cfg_if_module!($module, {
            use crate::$module;
            #[doc = concat!(" Access function for the ", stringify!($module), " generator.")]
            pub fn $module(dest: &mut [u8]) -> Result<(), Error> {
                let uninit_dest = unsafe { slice_as_uninit_mut(dest) };
                if !uninit_dest.is_empty() {
                    $module::getrandom_inner(uninit_dest)?;
                }
                Ok(())
            }
        });
    )+};
}

// Generate access functions for all backends supported by the target platform
define_generator_fns!(
    use_file,
    getentropy,
    getrandom_libc,
    linux_android,
    linux_android_with_fallback,
    solaris,
    netbsd,
    fuchsia,
    apple_other,
    wasi,
    hermit,
    vxworks,
    solid,
    espidf,
    windows7,
    windows,
    rdrand,
    js
);

/// Fill `dest` with random bytes from a hardware random number generator
/// Returns an Error if no hardware RNGs are available
#[inline]
#[allow(unreachable_code)]
pub fn hardware(_dest: &mut [u8]) -> Result<(), Error> {
    cfg_if_module!(rdrand, {
        return rdrand(_dest);
    });
    Err(Error::NO_HW)
}

/// Fill `dest` with random bytes from a hardware random number generator
/// Falls back to default getrandom() if no hardware RNGs are available
#[inline]
#[allow(unreachable_code)]
pub fn hardware_with_fallback(dest: &mut [u8]) -> Result<(), Error> {
    if hardware(dest).is_ok() {
        Ok(())
    } else {
        default_getrandom(dest)
    }
}
