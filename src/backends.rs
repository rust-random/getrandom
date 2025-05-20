//! System-specific implementations.
//!
//! This module should provide `fill_inner` with the signature
//! `fn fill_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error>`.
//! The function MUST fully initialize `dest` when `Ok(())` is returned.
//! The function MUST NOT ever write uninitialized bytes into `dest`,
//! regardless of what value it returns.

cfg_if! {
    if #[cfg(getrandom_backend = "custom")] {
        mod custom;
        crate::set_backend!(custom::LegacyCustomBackend);
    } else if #[cfg(getrandom_backend = "linux_getrandom")] {
        mod getrandom;
        crate::set_backend!(getrandom::GetrandomBackend);
    } else if #[cfg(getrandom_backend = "linux_raw")] {
        mod linux_raw;
        crate::set_backend!(linux_raw::LinuxRawBackend);
    } else if #[cfg(getrandom_backend = "rdrand")] {
        mod rdrand;
        crate::set_backend!(rdrand::RdrandBackend);
    } else if #[cfg(getrandom_backend = "rndr")] {
        mod rndr;
        crate::set_backend!(rndr::RndrBackend);
    } else if #[cfg(getrandom_backend = "efi_rng")] {
        mod efi_rng;
        crate::set_backend!(efi_rng::UefiBackend);
    } else if #[cfg(all(getrandom_backend = "wasm_js", feature = "wasm_js"))] {
        mod wasm_js;
        crate::set_backend!(wasm_js::WasmJsBackend);
    } else if #[cfg(getrandom_backend = "unsupported")] {
        mod unsupported;
        crate::set_backend!(unsupported::UnsupportedBackend);
    } else if #[cfg(all(target_os = "linux", target_env = ""))] {
        mod linux_raw;
        crate::set_backend!(linux_raw::LinuxRawBackend);
    } else if #[cfg(target_os = "espidf")] {
        mod esp_idf;
        crate::set_backend!(esp_idf::EspIdfBackend);
    } else if #[cfg(any(
        target_os = "haiku",
        target_os = "redox",
        target_os = "nto",
        target_os = "aix",
    ))] {
        mod use_file;
        crate::set_backend!(use_file::UseFileBackend);
    } else if #[cfg(any(
        target_os = "macos",
        target_os = "openbsd",
        target_os = "vita",
        target_os = "emscripten",
    ))] {
        mod getentropy;
        crate::set_backend!(getentropy::GetentropyBackend);
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
        crate::set_backend!(linux_android_with_fallback::LinuxBackend);
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
        crate::set_backend!(getrandom::GetrandomBackend);
    } else if #[cfg(target_os = "solaris")] {
        mod solaris;
        crate::set_backend!(solaris::SolarisBackend);
    } else if #[cfg(target_os = "netbsd")] {
        mod netbsd;
        crate::set_backend!(netbsd::NetBsdBackend);
    } else if #[cfg(target_os = "fuchsia")] {
        mod fuchsia;
        crate::set_backend!(fuchsia::FuchsiaBackend);
    } else if #[cfg(any(
        target_os = "ios",
        target_os = "visionos",
        target_os = "watchos",
        target_os = "tvos",
    ))] {
        mod apple_other;
        crate::set_backend!(apple_other::AppleOtherBackend);
    } else if #[cfg(all(target_arch = "wasm32", target_os = "wasi"))] {
        cfg_if! {
            if #[cfg(target_env = "p1")] {
                mod wasi_p1;
                crate::set_backend!(wasi_p1::WasiP1Backend);
            } else if #[cfg(target_env = "p2")] {
                mod wasi_p2;
                crate::set_backend!(wasi_p2::WasiP2Backend);
            }
        }
    } else if #[cfg(target_os = "hermit")] {
        mod hermit;
        crate::set_backend!(hermit::HermitBackend);
    } else if #[cfg(target_os = "vxworks")] {
        mod vxworks;
        crate::set_backend!(vxworks::VxWorksBackend);
    } else if #[cfg(target_os = "solid_asp3")] {
        mod solid;
        crate::set_backend!(solid::SolidBackend);
    } else if #[cfg(all(windows, any(target_vendor = "win7", getrandom_windows_legacy)))] {
        mod windows7;
        crate::set_backend!(windows7::WindowsLegacyBackend);
    } else if #[cfg(windows)] {
        mod windows;
        crate::set_backend!(windows::WindowsBackend);
    } else if #[cfg(all(target_arch = "x86_64", target_env = "sgx"))] {
        mod rdrand;
        crate::set_backend!(rdrand::RdrandBackend);
    } else {
        compile_error!(concat!(
            "target is not supported. You may need to define a custom backend see: \
            https://docs.rs/getrandom/", env!("CARGO_PKG_VERSION"), "/#custom-backend"
        ));
    }
}
