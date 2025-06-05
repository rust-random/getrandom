//! System-specific implementations.
//!
//! This module should provide `fill_inner` with the signature
//! `fn fill_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error>`.
//! The function MUST fully initialize `dest` when `Ok(())` is returned;
//! the function may need to use `sanitizer::unpoison` as well.
//! The function MUST NOT ever write uninitialized bytes into `dest`,
//! regardless of what value it returns.

use core::mem::MaybeUninit;

use crate::Error;

/// If an external fallback _may_ be used, use it.
/// If the fallback may not be used, the provided token trees will be included instead.
///
/// This is extracted into its own macro to allow using a fallback in multiple branches.
/// For example, on unsupported WASI targets.
#[allow(unused)]
// May be unused if a fallback is not required.
macro_rules! use_fallback_or {
    ($($tt: tt)*) => {
        cfg_if! {
            if #[cfg(feature = "custom-fallback")] {
                mod fallback;
                pub use fallback::Implementation;
            } else {
                $($tt)*
            }
        }
    }
}

cfg_if! {
    if #[cfg(getrandom_backend = "custom")] {
        mod custom;
        pub use custom::Implementation;
    } else if #[cfg(getrandom_backend = "linux_getrandom")] {
        mod getrandom;
        mod sanitizer;
        pub use getrandom::Implementation;
    } else if #[cfg(getrandom_backend = "linux_raw")] {
        mod linux_raw;
        mod sanitizer;
        pub use linux_raw::Implementation;
    } else if #[cfg(getrandom_backend = "rdrand")] {
        mod rdrand;
        pub use rdrand::Implementation;
    } else if #[cfg(getrandom_backend = "rndr")] {
        mod rndr;
        pub use rndr::Implementation;
    } else if #[cfg(getrandom_backend = "efi_rng")] {
        mod efi_rng;
        pub use efi_rng::Implementation;
    } else if #[cfg(all(getrandom_backend = "wasm_js"))] {
        cfg_if! {
            if #[cfg(feature = "wasm_js")] {
                mod wasm_js;
                pub use wasm_js::Implementation;
            } else {
                // Fallback not used here as the user indicated they intended on using "wasm_js",
                // but failed to activate the feature flag.
                compile_error!(concat!(
                    "The \"wasm_js\" backend requires the `wasm_js` feature \
                    for `getrandom`. For more information see: \
                    https://docs.rs/getrandom/", env!("CARGO_PKG_VERSION"), "/#webassembly-support"
                ));
            }
        }
    } else if #[cfg(getrandom_backend = "unsupported")] {
        mod unsupported;
        pub use unsupported::Implementation;
    } else if #[cfg(all(target_os = "linux", target_env = ""))] {
        mod linux_raw;
        mod sanitizer;
        pub use linux_raw::Implementation;
    } else if #[cfg(target_os = "espidf")] {
        mod esp_idf;
        pub use esp_idf::Implementation;
    } else if #[cfg(any(
        target_os = "haiku",
        target_os = "redox",
        target_os = "nto",
        target_os = "aix",
    ))] {
        mod use_file;
        pub use use_file::Implementation;
    } else if #[cfg(any(
        target_os = "macos",
        target_os = "openbsd",
        target_os = "vita",
        target_os = "emscripten",
    ))] {
        mod getentropy;
        pub use getentropy::Implementation;
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
        mod use_file;
        mod linux_android_with_fallback;
        mod sanitizer;
        pub use linux_android_with_fallback::Implementation;
    } else if #[cfg(any(
        target_os = "android",
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "hurd",
        target_os = "illumos",
        target_os = "cygwin",
        // Check for target_arch = "arm" to only include the 3DS. Does not
        // include the Nintendo Switch (which is target_arch = "aarch64").
        all(target_os = "horizon", target_arch = "arm"),
    ))] {
        mod getrandom;
        #[cfg(any(target_os = "android", target_os = "linux"))]
        mod sanitizer;
        pub use getrandom::Implementation;
    } else if #[cfg(target_os = "solaris")] {
        mod solaris;
        pub use solaris::Implementation;
    } else if #[cfg(target_os = "netbsd")] {
        mod netbsd;
        pub use netbsd::Implementation;
    } else if #[cfg(target_os = "fuchsia")] {
        mod fuchsia;
        pub use fuchsia::Implementation;
    } else if #[cfg(any(
        target_os = "ios",
        target_os = "visionos",
        target_os = "watchos",
        target_os = "tvos",
    ))] {
        mod apple_other;
        pub use apple_other::Implementation;
    } else if #[cfg(all(target_arch = "wasm32", target_os = "wasi"))] {
        cfg_if! {
            if #[cfg(target_env = "p1")] {
                mod wasi_p1;
                pub use wasi_p1::Implementation;
            } else if #[cfg(target_env = "p2")] {
                mod wasi_p2;
                pub use wasi_p2::Implementation;
            } else {
                use_fallback_or! {
                    compile_error!(
                        "Unknown version of WASI (only previews 1 and 2 are supported) \
                        or Rust version older than 1.80 was used"
                    );
                }
            }
        }
    } else if #[cfg(target_os = "hermit")] {
        mod hermit;
        pub use hermit::Implementation;
    } else if #[cfg(target_os = "vxworks")] {
        mod vxworks;
        pub use vxworks::Implementation;
    } else if #[cfg(target_os = "solid_asp3")] {
        mod solid;
        pub use solid::Implementation;
    } else if #[cfg(all(windows, any(target_vendor = "win7", getrandom_windows_legacy)))] {
        mod windows7;
        pub use windows7::Implementation;
    } else if #[cfg(windows)] {
        mod windows;
        pub use windows::Implementation;
    } else if #[cfg(all(target_arch = "x86_64", target_env = "sgx"))] {
        mod rdrand;
        pub use rdrand::Implementation;
    } else {
        use_fallback_or! {
            cfg_if! {
                if #[cfg(all(target_arch = "wasm32", any(target_os = "unknown", target_os = "none")))] {
                    compile_error!(concat!(
                        "The wasm32-unknown-unknown targets are not supported by default; \
                        you may need to enable the \"wasm_js\" configuration flag. Note \
                        that enabling the `wasm_js` feature flag alone is insufficient. \
                        For more information see: \
                        https://docs.rs/getrandom/", env!("CARGO_PKG_VERSION"), "/#webassembly-support"
                    ));
                } else {
                    compile_error!(concat!(
                        "target is not supported. You may need to define a custom backend see: \
                        https://docs.rs/getrandom/", env!("CARGO_PKG_VERSION"), "/#custom-backend"
                    ));
                }
            }
        }
    }
}

/// Provides entropy suitable for CSPRNG.
///
/// # Safety
///
/// Implementors must uphold the contracts of all methods provided by this trait.
pub unsafe trait Backend {
    /// Fill `dest` with pseudo-random values _or_ return an [`Error`].
    ///
    /// # Implementors
    ///
    /// - An implementation of this method _must_ totally fill `dest`.
    ///   If this cannot be done, an error should be returned.
    /// - The values filling `dest` _must_ be cryptographically secure.
    fn fill_uninit(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error>;

    /// Provides a pseudo-random [`u32`].
    ///
    /// # Implementors
    ///
    /// - The [`u32`] returned _must_ be cryptographically secure.
    #[inline]
    fn u32() -> Result<u32, Error> {
        crate::util::inner_u32()
    }

    /// Provides a pseudo-random [`u64`].
    ///
    /// - The [`u64`] returned _must_ be cryptographically secure.
    #[inline]
    fn u64() -> Result<u64, Error> {
        crate::util::inner_u64()
    }
}
