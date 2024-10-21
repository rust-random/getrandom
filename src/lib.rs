#![no_std]
#![doc(
    html_logo_url = "https://www.rust-lang.org/logos/rust-logo-128x128-blk.png",
    html_favicon_url = "https://www.rust-lang.org/favicon.ico",
    html_root_url = "https://docs.rs/getrandom/0.2.15"
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

mod error;
mod util;

#[cfg(feature = "std")]
mod error_impls;

pub use crate::error::Error;
use crate::util::{slice_as_uninit_mut, slice_assume_init_mut};

// System-specific implementations.
//
// These should all provide fill_inner with the signature
// `fn fill_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error>`.
// The function MUST fully initialize `dest` when `Ok(())` is returned.
// The function MUST NOT ever write uninitialized bytes into `dest`,
// regardless of what value it returns.
cfg_if! {
    if #[cfg(getrandom_backend = "custom")] {
        #[path = "custom.rs"] mod imp;
    } else if #[cfg(getrandom_backend = "linux_getrandom")] {
        mod util_libc;
        #[path = "linux_android.rs"] mod imp;
    } else if #[cfg(getrandom_backend = "linux_rustix")] {
        #[path = "linux_rustix.rs"] mod imp;
    } else if #[cfg(getrandom_backend = "rdrand")] {
        mod lazy;
        #[path = "rdrand.rs"] mod imp;
    } else if #[cfg(getrandom_backend = "rndr")] {
        #[path = "rndr.rs"] mod imp;
    } else if #[cfg(getrandom_backend = "wasm_js")] {
        #[path = "wasm_js.rs"] mod imp;
    } else if #[cfg(getrandom_backend = "esp_idf")] {
        #[path = "esp_idf.rs"] mod imp;
    } else if #[cfg(any(
        target_os = "haiku",
        target_os = "redox",
        target_os = "nto",
        target_os = "aix",
    ))] {
        mod util_libc;
        #[path = "use_file.rs"] mod imp;
    } else if #[cfg(any(
        target_os = "macos",
        target_os = "openbsd",
        target_os = "vita",
        target_os = "emscripten",
    ))] {
        mod util_libc;
        #[path = "getentropy.rs"] mod imp;
    } else if #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "hurd",
        target_os = "illumos",
        // Check for target_arch = "arm" to only include the 3DS. Does not
        // include the Nintendo Switch (which is target_arch = "aarch64").
        all(target_os = "horizon", target_arch = "arm"),
    ))] {
        mod util_libc;
        #[path = "getrandom.rs"] mod imp;
    } else if #[cfg(any(
        // Rust supports Android API level 19 (KitKat) [0] and the next upgrade targets
        // level 21 (Lollipop) [1], while `getrandom(2)` was added only in
        // level 23 (Marshmallow). Note that it applies only to the "old" `target_arch`es,
        // RISC-V Android targets sufficiently new API level, same will apply for potential
        // new Android `target_arch`es.
        // [0]: https://blog.rust-lang.org/2023/01/09/android-ndk-update-r25.html
        // [1]: https://github.com/rust-lang/rust/pull/120593
        all(
            target_os = "android",
            any(
                target_arch = "aarch64",
                target_arch = "arm",
                target_arch = "x86",
                target_arch = "x86_64",
            ),
        ),
        // Only on these `target_arch`es Rust supports Linux kernel versions (3.2+)
        // that precede the version (3.17) in which `getrandom(2)` was added:
        // https://doc.rust-lang.org/stable/rustc/platform-support.html
        all(
            target_os = "linux",
            any(
                target_arch = "aarch64",
                target_arch = "arm",
                target_arch = "powerpc",
                target_arch = "powerpc64",
                target_arch = "s390x",
                target_arch = "x86",
                target_arch = "x86_64",
                // Minimum supported Linux kernel version for MUSL targets
                // is not specified explicitly (as of Rust 1.77) and they
                // are used in practice to target pre-3.17 kernels.
                target_env = "musl",
            ),
        )
    ))] {
        mod util_libc;
        mod use_file;
        #[path = "linux_android_with_fallback.rs"] mod imp;
    } else if #[cfg(any(target_os = "android", target_os = "linux"))] {
        mod util_libc;
        #[path = "linux_android.rs"] mod imp;
    } else if #[cfg(target_os = "solaris")] {
        mod util_libc;
        #[path = "solaris.rs"] mod imp;
    } else if #[cfg(target_os = "netbsd")] {
        mod util_libc;
        #[path = "netbsd.rs"] mod imp;
    } else if #[cfg(target_os = "fuchsia")] {
        #[path = "fuchsia.rs"] mod imp;
    } else if #[cfg(any(
        target_os = "ios",
        target_os = "visionos",
        target_os = "watchos",
        target_os = "tvos",
    ))] {
        #[path = "apple-other.rs"] mod imp;
    } else if #[cfg(all(target_arch = "wasm32", target_os = "wasi"))] {
        #[path = "wasi.rs"] mod imp;
    } else if #[cfg(target_os = "hermit")] {
        #[path = "hermit.rs"] mod imp;
    } else if #[cfg(target_os = "vxworks")] {
        mod util_libc;
        #[path = "vxworks.rs"] mod imp;
    } else if #[cfg(target_os = "solid_asp3")] {
        #[path = "solid.rs"] mod imp;
    } else if #[cfg(all(windows, target_vendor = "win7"))] {
        #[path = "windows7.rs"] mod imp;
    } else if #[cfg(windows)] {
        #[path = "windows.rs"] mod imp;
    } else if #[cfg(all(target_arch = "x86_64", target_env = "sgx"))] {
        mod lazy;
        #[path = "rdrand.rs"] mod imp;
    } else if #[cfg(all(
        any(target_arch = "wasm32", target_arch = "wasm64"),
        target_os = "unknown",
    ))] {
        compile_error!("the wasm*-unknown-unknown targets are not supported by \
                        default, you may need to enable the \"wasm_js\" \
                        configuration flag. For more information see: \
                        https://docs.rs/getrandom/#webassembly-support");
    } else {
        compile_error!("target is not supported. You may need to define \
                        a custom backend see: \
                        https://docs.rs/getrandom/#custom-backends");
    }
}

/// Fill `dest` with random bytes from the system's preferred random number source.
///
/// This function returns an error on any failure, including partial reads. We
/// make no guarantees regarding the contents of `dest` on error. If `dest` is
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
///
/// ```
/// # fn main() -> Result<(), getrandom::Error> {
/// let mut buf = [0u8; 32];
/// getrandom::fill(&mut buf)?;
/// # Ok(()) }
/// ```
#[inline]
pub fn fill(dest: &mut [u8]) -> Result<(), Error> {
    // SAFETY: The `&mut MaybeUninit<_>` reference doesn't escape,
    // and `fill_uninit` guarantees it will never de-initialize
    // any part of `dest`.
    fill_uninit(unsafe { slice_as_uninit_mut(dest) })?;
    Ok(())
}

/// Fill potentially uninitialized buffer `dest` with random bytes from
/// the system's preferred random number source and return a mutable
/// reference to those bytes.
///
/// On successful completion this function is guaranteed to return a slice
/// which points to the same memory as `dest` and has the same length.
/// In other words, it's safe to assume that `dest` is initialized after
/// this function has returned `Ok`.
///
/// No part of `dest` will ever be de-initialized at any point, regardless
/// of what is returned.
///
/// # Examples
///
/// ```ignore
/// # // We ignore this test since `uninit_array` is unstable.
/// #![feature(maybe_uninit_uninit_array)]
/// # fn main() -> Result<(), getrandom::Error> {
/// let mut buf = core::mem::MaybeUninit::uninit_array::<1024>();
/// let buf: &mut [u8] = getrandom::fill_uninit(&mut buf)?;
/// # Ok(()) }
/// ```
#[inline]
pub fn fill_uninit(dest: &mut [MaybeUninit<u8>]) -> Result<&mut [u8], Error> {
    if !dest.is_empty() {
        imp::fill_inner(dest)?;
    }

    #[cfg(getrandom_sanitize)]
    #[cfg(sanitize = "memory")]
    extern "C" {
        fn __msan_unpoison(a: *mut core::ffi::c_void, size: usize);
    }

    // SAFETY: `dest` has been fully initialized by `imp::fill_inner`
    // since it returned `Ok`.
    Ok(unsafe {
        #[cfg(getrandom_sanitize)]
        #[cfg(sanitize = "memory")]
        __msan_unpoison(dest.as_mut_ptr().cast(), dest.len());

        slice_assume_init_mut(dest)
    })
}
