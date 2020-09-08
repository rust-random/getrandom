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

/// Register a function to be invoked by `getrandom` on unsupported targets.
///
/// *This API requires the following Cargo features to be activated: `"custom"`*
///
/// ## Writing your own custom `getrandom` implementation
///
/// Users can define custom implementations either in their root crate or in a
/// target specific helper-crate. We will use the helper-crate approach in this
/// example, defining `dummy-getrandom`, an implementation that always fails.
///
/// First, in `dummy-getrandom/Cargo.toml` we depend on `getrandom`:
/// ```toml
/// [dependencies]
/// getrandom = { version = "0.2", features = ["custom"] }
/// ```
///
/// Next, in `dummy-getrandom/src/lib.rs`, we define our custom implementation and register it:
/// ```rust
/// use core::num::NonZeroU32;
/// use getrandom::{Error, register_custom_getrandom};
///
/// const MY_CUSTOM_ERROR_CODE: u32 = Error::CUSTOM_START + 42;
/// fn always_fail(buf: &mut [u8]) -> Result<(), Error> {
///     let code = NonZeroU32::new(MY_CUSTOM_ERROR_CODE).unwrap();
///     Err(Error::from(code))
/// }
///
/// register_custom_getrandom!(always_fail);
/// ```
/// the registered function must have the same type signature as
/// [`getrandom::getrandom`](crate::getrandom).
///
/// Now any user of `getrandom` (direct or indirect) on this target will use the
/// above custom implementation. See the
/// [usage documentation](index.html#use-a-custom-implementation) for information about
/// _using_ such a custom implementation.
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
