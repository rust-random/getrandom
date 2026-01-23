//! System-specific implementations.
//!
//! This module should provide `fill_inner` with the signature
//! `fn fill_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error>`.
//! The function MUST fully initialize `dest` when `Ok(())` is returned;
//! the function may need to use `sanitizer::unpoison` as well.
//! The function MUST NOT ever write uninitialized bytes into `dest`,
//! regardless of what value it returns.

/// Declares this function as an external implementation of [`u32`](crate::u32).
#[cfg_attr(getrandom_extern_item_impls, eii(u32))]
pub(crate) fn inner_u32() -> Result<u32, crate::Error> {
    default_u32()
}

/// Declares this function as an external implementation of [`u64`](crate::u64).
#[cfg_attr(getrandom_extern_item_impls, eii(u64))]
pub(crate) fn inner_u64() -> Result<u64, crate::Error> {
    default_u64()
}

macro_rules! implementation {
    () => {
        use $crate::util::{inner_u32 as default_u32, inner_u64 as default_u64};

        /// Declares this function as an external implementation of [`fill_uninit`](crate::fill_uninit).
        #[cfg_attr(getrandom_extern_item_impls, eii(fill_uninit))]
        pub(crate) fn fill_inner(
            dest: &mut [::core::mem::MaybeUninit<u8>],
        ) -> Result<(), $crate::Error>;
    };
    ($backend:ident) => {
        use $backend::{inner_u32 as default_u32, inner_u64 as default_u64};

        /// Declares this function as an external implementation of [`fill_uninit`](crate::fill_uninit).
        #[cfg_attr(getrandom_extern_item_impls, eii(fill_uninit))]
        pub(crate) fn fill_inner(
            dest: &mut [::core::mem::MaybeUninit<u8>],
        ) -> Result<(), $crate::Error> {
            $backend::fill_inner(dest)
        }
    };
}

cfg_if! {
    if #[cfg(getrandom_backend = "custom")] {
        mod custom;
        implementation!(custom);
    } else if #[cfg(getrandom_backend = "linux_getrandom")] {
        mod getrandom;
        implementation!(getrandom);
    } else if #[cfg(getrandom_backend = "linux_raw")] {
        mod linux_raw;
        implementation!(linux_raw);
    } else if #[cfg(getrandom_backend = "rdrand")] {
        mod rdrand;
        implementation!(rdrand);
    } else if #[cfg(getrandom_backend = "rndr")] {
        mod rndr;
        implementation!(rndr);
    } else if #[cfg(getrandom_backend = "efi_rng")] {
        mod efi_rng;
        implementation!(efi_rng);
    } else if #[cfg(getrandom_backend = "windows_legacy")] {
        mod windows_legacy;
        implementation!(windows_legacy);
    } else if #[cfg(getrandom_backend = "unsupported")] {
        mod unsupported;
        implementation!(unsupported);
    } else if #[cfg(all(target_os = "linux", target_env = ""))] {
        mod linux_raw;
        implementation!(linux_raw);
    } else if #[cfg(target_os = "espidf")] {
        mod esp_idf;
        implementation!(esp_idf);
    } else if #[cfg(any(
        target_os = "haiku",
        target_os = "redox",
        target_os = "nto",
        target_os = "aix",
    ))] {
        mod use_file;
        implementation!(use_file);
    } else if #[cfg(any(
        target_os = "macos",
        target_os = "openbsd",
        target_os = "vita",
        target_os = "emscripten",
    ))] {
        mod getentropy;
        implementation!(getentropy);
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
                all(
                    target_env = "musl",
                    not(
                        any(
                            target_arch = "riscv64",
                            target_arch = "riscv32",
                        ),
                    ),
                ),
            ),
        )
    ))] {
        mod use_file;
        mod linux_android_with_fallback;
        implementation!(linux_android_with_fallback);
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
        implementation!(getrandom);
    } else if #[cfg(target_os = "solaris")] {
        mod solaris;
        implementation!(solaris);
    } else if #[cfg(target_os = "netbsd")] {
        mod netbsd;
        implementation!(netbsd);
    } else if #[cfg(target_os = "fuchsia")] {
        mod fuchsia;
        implementation!(fuchsia);
    } else if #[cfg(any(
        target_os = "ios",
        target_os = "visionos",
        target_os = "watchos",
        target_os = "tvos",
    ))] {
        mod apple_other;
        implementation!(apple_other);
    } else if #[cfg(all(target_arch = "wasm32", target_os = "wasi"))] {
        cfg_if! {
            if #[cfg(target_env = "p1")] {
                mod wasi_p1;
                implementation!(wasi_p1);
            } else {
                mod wasi_p2_3;
                implementation!(wasi_p2_3);
            }
        }
    } else if #[cfg(target_os = "hermit")] {
        mod hermit;
        implementation!(hermit);
    } else if #[cfg(target_os = "vxworks")] {
        mod vxworks;
        implementation!(vxworks);
    } else if #[cfg(target_os = "solid_asp3")] {
        mod solid;
        implementation!(solid);
    } else if #[cfg(all(windows, target_vendor = "win7"))] {
        mod windows_legacy;
        implementation!(windows_legacy);
    } else if #[cfg(windows)] {
        mod windows;
        implementation!(windows);
    } else if #[cfg(all(target_arch = "x86_64", target_env = "sgx"))] {
        mod rdrand;
        implementation!(rdrand);
    } else if #[cfg(all(target_arch = "wasm32", any(target_os = "unknown", target_os = "none")))] {
        cfg_if! {
            if #[cfg(feature = "wasm_js")] {
                mod wasm_js;
                implementation!(wasm_js);
            } else if #[cfg(getrandom_extern_item_impls)] {
                implementation!();
            } else {
                compile_error!(concat!(
                    "The wasm32-unknown-unknown targets are not supported by default; \
                    you may need to enable the \"wasm_js\" crate feature. \
                    For more information see: \
                    https://docs.rs/getrandom/", env!("CARGO_PKG_VERSION"), "/#webassembly-support"
                ));
            }
        }
    } else if #[cfg(getrandom_extern_item_impls)] {
        implementation!();
    } else {
        compile_error!(concat!(
            "target is not supported. You may need to define a custom backend see: \
            https://docs.rs/getrandom/", env!("CARGO_PKG_VERSION"), "/#custom-backend"
        ));
    }
}
