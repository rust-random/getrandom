// Overwrite links to crate items with intra-crate links
//! [`Error::UNEXPECTED`]: Error::UNEXPECTED
//! [`fill`]: fill
//! [`fill_uninit`]: fill_uninit
//! [`u32`]: u32()
//! [`u64`]: u64()
//! [`insecure_fill`]: insecure_fill
//! [`insecure_fill_uninit`]: insecure_fill_uninit
//! [`insecure_u32`]: insecure_u32
//! [`insecure_u64`]: insecure_u64

#![no_std]
#![doc(
    html_logo_url = "https://www.rust-lang.org/logos/rust-logo-128x128-blk.png",
    html_favicon_url = "https://www.rust-lang.org/favicon.ico"
)]
#![doc = include_str!("../README.md")]
#![warn(rust_2018_idioms, unused_lifetimes, missing_docs)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(getrandom_sanitize, feature(cfg_sanitize))]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_ptr_alignment,
    clippy::cast_sign_loss,
    clippy::char_lit_as_u8,
    clippy::checked_conversions,
    clippy::fn_to_numeric_cast,
    clippy::fn_to_numeric_cast_with_truncation,
    clippy::ptr_as_ptr,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]

#[macro_use]
extern crate cfg_if;

use core::mem::MaybeUninit;

mod backends;
mod default_impls;
mod error;
mod util;

#[cfg(feature = "std")]
mod error_std_impls;

pub use crate::error::Error;

/// Fill `dst` with random bytes from the system's entropy source.
///
/// This function returns an error on any failure, including partial reads. We
/// make no guarantees regarding the contents of `dst` on error. If `dst` is
/// empty, `getrandom` immediately returns success, making no calls to the
/// underlying operating system.
///
/// Blocking is possible, at least during early boot; see module documentation.
///
/// In general, `getrandom` will be fast enough for interactive usage, though
/// significantly slower than a user-space CSPRNG; for the latter consider
/// [`rand::thread_rng`](https://docs.rs/rand/*/rand/fn.thread_rng.html).
///
/// # Examples
/// ```
/// # fn main() -> Result<(), getrandom::Error> {
/// let mut buf = [0u8; 32];
/// getrandom::fill(&mut buf)?;
/// # Ok(()) }
/// ```
#[inline]
pub fn fill(dst: &mut [u8]) -> Result<(), Error> {
    // SAFETY: The `&mut MaybeUninit<_>` reference doesn't escape,
    // and `fill_uninit` guarantees it will never de-initialize
    // any part of `dst`.
    fill_uninit(unsafe { util::slice_as_uninit_mut(dst) })?;
    Ok(())
}

/// Fill `dst` with **potentially insecure** random bytes from the system's entropy source.
///
/// See the ["insecure" functions][crate#insecure-functions] section for more information.
///
/// # Examples
/// ```
/// # fn main() -> Result<(), getrandom::Error> {
/// let mut buf = [0u8; 32];
/// getrandom::insecure_fill(&mut buf)?;
/// # Ok(()) }
/// ```
#[inline]
pub fn insecure_fill(dst: &mut [u8]) -> Result<(), Error> {
    // SAFETY: The `&mut MaybeUninit<_>` reference doesn't escape,
    // and `fill_uninit` guarantees it will never de-initialize
    // any part of `dst`.
    insecure_fill_uninit(unsafe { util::slice_as_uninit_mut(dst) })?;
    Ok(())
}

/// Fill potentially uninitialized buffer `dst` with random bytes from
/// the system's entropy source.
///
/// On successful completion this function is guaranteed to return a slice
/// which points to the same memory as `dst` and has the same length.
/// In other words, it's safe to assume that `dst` is initialized after
/// this function has returned `Ok`.
///
/// No part of `dst` will ever be de-initialized at any point, regardless
/// of what is returned.
///
/// # Examples
/// ```ignore
/// # // We ignore this test since `uninit_array` is unstable.
/// #![feature(maybe_uninit_uninit_array)]
/// # fn main() -> Result<(), getrandom::Error> {
/// let mut buf = core::mem::MaybeUninit::uninit_array::<1024>();
/// let buf: &mut [u8] = getrandom::fill_uninit(&mut buf)?;
/// assert_eq!(buf.len(), 1024);
/// # Ok(()) }
/// ```
#[inline]
pub fn fill_uninit(dst: &mut [MaybeUninit<u8>]) -> Result<&mut [u8], Error> {
    if !dst.is_empty() {
        backends::fill_uninit(dst)?;
    }

    // SAFETY: `dst` has been fully initialized by `imp::fill_inner` since it returned `Ok`
    Ok(unsafe { util::slice_assume_init_mut(dst) })
}

/// Fill potentially uninitialized buffer `dst` with **potentially insecure** random bytes
/// from the system's entropy source.
///
/// On successful completion this function is guaranteed to return a slice
/// which points to the same memory as `dst` and has the same length.
/// In other words, it's safe to assume that `dst` is initialized after
/// this function has returned `Ok`.
///
/// No part of `dst` will ever be de-initialized at any point, regardless
/// of what is returned.
///
/// See the ["insecure" functions][crate#insecure-functions] section for more information.
///
/// # Examples
/// ```ignore
/// # // We ignore this test since `uninit_array` is unstable.
/// #![feature(maybe_uninit_uninit_array)]
/// # fn main() -> Result<(), getrandom::Error> {
/// let mut buf = core::mem::MaybeUninit::uninit_array::<1024>();
/// let buf: &mut [u8] = getrandom::insecure_fill_uninit(&mut buf)?;
/// assert_eq!(buf.len(), 1024);
/// # Ok(()) }
/// ```
#[inline]
pub fn insecure_fill_uninit(dst: &mut [MaybeUninit<u8>]) -> Result<&mut [u8], Error> {
    if !dst.is_empty() {
        backends::insecure_fill_uninit(dst)?;
    }

    // SAFETY: `dst` has been fully initialized by `imp::fill_inner` since it returned `Ok`
    Ok(unsafe { util::slice_assume_init_mut(dst) })
}

/// Get random `u32` from the system's entropy source.
///
/// # Examples
/// ```
/// # fn main() -> Result<(), getrandom::Error> {
/// let rng_seed = getrandom::u32()?;
/// # Ok(()) }
/// ```
#[inline]
pub fn u32() -> Result<u32, Error> {
    backends::u32()
}

/// Get **potentially insecure** random `u32` from the system's entropy source.
///
/// See the ["insecure" functions][crate#insecure-functions] section for more information.
///
/// # Examples
/// ```
/// # fn main() -> Result<(), getrandom::Error> {
/// let rng_seed = getrandom::insecure_u32()?;
/// # Ok(()) }
/// ```
#[inline]
pub fn insecure_u32() -> Result<u32, Error> {
    backends::insecure_u32()
}

/// Get random `u64` from the system's entropy source.
///
/// # Examples
/// ```
/// # fn main() -> Result<(), getrandom::Error> {
/// let rng_seed = getrandom::u64()?;
/// # Ok(()) }
/// ```
#[inline]
pub fn u64() -> Result<u64, Error> {
    backends::u64()
}

/// Get **potentially insecure** random `u64` from the system's entropy source.
///
/// See the ["insecure" functions][crate#insecure-functions] section for more information.
///
/// # Examples
/// ```
/// # fn main() -> Result<(), getrandom::Error> {
/// let rng_seed = getrandom::insecure_u64()?;
/// # Ok(()) }
/// ```
#[inline]
pub fn insecure_u64() -> Result<u64, Error> {
    backends::insecure_u64()
}
