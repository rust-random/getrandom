/// Compile a block of tokens if `module` is supported on the target platform
/// This is a convenience macro in order to avoid repeating lists of
/// supported targets in `cfg_if` blocks.
///
/// The target configs in this macro are supposed to describe platform
/// compatibility for each of the modules, regardless of the policy choice
/// on which module is preferred for a specific target.
///
/// Usage:
///
/// ```ignore
/// cfg_if_module!(use_file, {
///     // any code that requires use_file to be supported
/// });
/// ```
macro_rules! cfg_if_module {
    ( $(util_libc, { $($tokens:tt)* })+ ) => {$(
        cfg_if! {
            if #[cfg(any(
                    target_os = "android", target_os = "linux", target_os = "solaris",
                    target_os = "netbsd", target_os = "haiku", target_os = "redox",
                    target_os = "nto", target_os = "aix", target_os = "vxworks",
                    target_os = "dragonfly", target_os = "freebsd", target_os = "hurd",
                    target_os = "illumos", target_os = "macos", target_os = "openbsd",
                    target_os = "vita", target_os = "emscripten", target_os = "horizon"
                ))] {
                    $($tokens)*
                }
        }
    )*};

    ( $(use_file, { $($tokens:tt)* })+ ) => {$(
        cfg_if! {
            if #[cfg(any(
                    target_os = "linux", target_os = "android", target_os = "macos",
                    target_os = "freebsd", target_os = "haiku", target_os = "redox",
                    target_os = "nto", target_os = "aix",
                ))] {
                    $($tokens)*
                }
        }
    )*};

    ( $(getentropy, { $($tokens:tt)* })+ ) => {$(
        cfg_if! {
            if #[cfg(any(
                    target_os = "macos", target_os = "openbsd",
                    target_os = "vita", target_os = "emscripten",
                ))] {
                    $($tokens)*
                }
        }
    )*};

    ( $(getrandom_libc, { $($tokens:tt)* })+ ) => {$(
        cfg_if! {
            if #[cfg(any(
                    target_os = "dragonfly", target_os = "freebsd",
                    target_os = "hurd", target_os = "illumos",
                    // Check for target_arch = "arm" to only include the 3DS. Does not
                    // include the Nintendo Switch (which is target_arch = "aarch64").
                    all(target_os = "horizon", target_arch = "arm"),
                ))] {
                    $($tokens)*
                }
        }
    )*};

    ( $(linux_android, { $($tokens:tt)* })+ ) => {$(
        cfg_if! {
            if #[cfg(any(
                    target_os = "linux", target_os = "android",
                ))] {
                    $($tokens)*
                }
        }
    )*};

    ( $(linux_android_with_fallback, { $($tokens:tt)* })+ ) => {$(
        cfg_if! {
            if #[cfg(all(
                not(feature = "linux_disable_fallback"),
                any(
                    // Rust supports Android API level 19 (KitKat) [0] and the next upgrade targets
                    // level 21 (Lollipop) [1], while `getrandom(2)` was added only in
                    // level 23 (Marshmallow). Note that it applies only to the "old" `target_arch`es,
                    // RISC-V Android targets sufficiently new API level, same will apply for potential
                    // new Android `target_arch`es.
                    // [0]: https://blog.rust-lang.org/2023/01/09/android-ndk-update-r25.html
                    // [1]: https://github.com/rust-lang/rust/pull/120593
                    all(
                        target_os = "android",
                        any(target_arch = "aarch64", target_arch = "arm",
                            target_arch = "x86", target_arch = "x86_64",
                        )
                    ),
                    // Only on these `target_arch`es Rust supports Linux kernel versions (3.2+)
                    // that precede the version (3.17) in which `getrandom(2)` was added:
                    // https://doc.rust-lang.org/stable/rustc/platform-support.html
                    all(
                        target_os = "linux",
                        any(
                            target_arch = "aarch64", target_arch = "arm", target_arch = "s390x",
                            target_arch = "powerpc", target_arch = "powerpc64",
                            target_arch = "x86", target_arch = "x86_64",
                            // Minimum supported Linux kernel version for MUSL targets
                            // is not specified explicitly (as of Rust 1.77) and they
                            // are used in practice to target pre-3.17 kernels.
                            target_env = "musl",
                        ),
                    )
                ),
                ))] {
                    $($tokens)*
                }
        }
    )*};

    ( $(solaris, { $($tokens:tt)* })+ ) => {$(
        cfg_if! {
            if #[cfg(target_os = "solaris")] {
                $($tokens)*
            }
        }
    )*};

    ( $(netbsd, { $($tokens:tt)* })+ ) => {$(
        cfg_if! {
            if #[cfg(target_os = "netbsd")] {
                $($tokens)*
            }
        }
    )*};

    ( $(fuchsia, { $($tokens:tt)* })+ ) => {$(
        cfg_if! {
            if #[cfg(target_os = "fuchsia")] {
                $($tokens)*
            }
        }
    )*};

    ( $(apple_other, { $($tokens:tt)* })+ ) => {$(
        cfg_if! {
            if #[cfg(any(
                target_os = "ios", target_os = "visionos", target_os = "watchos", target_os = "tvos"
            ))] {
                $($tokens)*
            }
        }
    )*};

    ( $(wasi, { $($tokens:tt)* })+ ) => {$(
        cfg_if! {
            if #[cfg(all(target_arch = "wasm32", target_os = "wasi"))] {
                $($tokens)*
            }
        }
    )*};

    ( $(hermit, { $($tokens:tt)* })+ ) => {$(
        cfg_if! {
            if #[cfg(target_os = "hermit")] {
                $($tokens)*
            }
        }
    )*};

    ( $(vxworks, { $($tokens:tt)* })+ ) => {$(
        cfg_if! {
            if #[cfg(target_os = "vxworks")] {
                $($tokens)*
            }
        }
    )*};

    ( $(solid, { $($tokens:tt)* })+ ) => {$(
        cfg_if! {
            if #[cfg(target_os = "solid_asp3")] {
                $($tokens)*
            }
        }
    )*};

    ( $(espidf, { $($tokens:tt)* })+ ) => {$(
        cfg_if! {
            if #[cfg(target_os = "espidf")] {
                $($tokens)*
            }
        }
    )*};

    ( $(windows7, { $($tokens:tt)* })+ ) => {$(
        cfg_if! {
            if #[cfg(all(windows, target_vendor = "win7"))] {
                $($tokens)*
            }
        }
    )*};

    ( $(windows, { $($tokens:tt)* })+ ) => {$(
        cfg_if! {
            if #[cfg(all(windows, not(target_vendor = "win7")))] {
                $($tokens)*
            }
        }
    )*};

    ( $(rdrand, { $($tokens:tt)* })+ ) => {$(
        cfg_if! {
            if #[cfg(any(
                    all(target_arch = "x86_64", target_env = "sgx"),
                    all(feature = "rdrand", any(target_arch = "x86_64", target_arch = "x86"))
                ))] {
                    $($tokens)*
                }
        }
    )*};

    ( $(js, { $($tokens:tt)* })+ ) => {$(
        cfg_if! {
            if #[cfg(all(
                    feature = "js", target_os = "unknown",
                    any(target_arch = "wasm32", target_arch = "wasm64"),
                ))] {
                    $($tokens)*
                }
        }
    )*};
}
