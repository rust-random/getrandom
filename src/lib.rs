//! Interface to the operating system's random number generator.
//!
//! # Supported targets
//!
//! | Target             | Target Triple      | Implementation
//! | ------------------ | ------------------ | --------------
//! | Linux, Android     | `*‑linux‑*`        | [`getrandom`][1] system call if available, otherwise [`/dev/urandom`][2] after successfully polling `/dev/random`
//! | Windows 10+        | `*‑windows‑*`      | [`ProcessPrng`]
//! | Windows 7 and 8    | `*-win7‑windows‑*` | [`RtlGenRandom`]
//! | macOS              | `*‑apple‑darwin`   | [`getentropy`][3]
//! | iOS, tvOS, watchOS | `*‑apple‑ios`, `*-apple-tvos`, `*-apple-watchos` | [`CCRandomGenerateBytes`]
//! | FreeBSD            | `*‑freebsd`        | [`getrandom`][5]
//! | OpenBSD            | `*‑openbsd`        | [`getentropy`][7]
//! | NetBSD             | `*‑netbsd`         | [`getrandom`][16] if available, otherwise [`kern.arandom`][8]
//! | Dragonfly BSD      | `*‑dragonfly`      | [`getrandom`][9]
//! | Solaris            | `*‑solaris`        | [`getrandom`][11] (with `GRND_RANDOM`)
//! | illumos            | `*‑illumos`        | [`getrandom`][12]
//! | Fuchsia OS         | `*‑fuchsia`        | [`cprng_draw`]
//! | Redox              | `*‑redox`          | `/dev/urandom`
//! | Haiku              | `*‑haiku`          | `/dev/urandom` (identical to `/dev/random`)
//! | Hermit             | `*-hermit`         | [`sys_read_entropy`]
//! | Hurd               | `*-hurd-*`         | [`getrandom`][17]
//! | SGX                | `x86_64‑*‑sgx`     | [`RDRAND`]
//! | VxWorks            | `*‑wrs‑vxworks‑*`  | `randABytes` after checking entropy pool initialization with `randSecure`
//! | Emscripten         | `*‑emscripten`     | [`getentropy`][13]
//! | WASI 0.1           | `wasm32‑wasip1`    | [`random_get`]
//! | WASI 0.2           | `wasm32‑wasip2`    | [`get-random-u64`]
//! | SOLID              | `*-kmc-solid_*`    | `SOLID_RNG_SampleRandomBytes`
//! | Nintendo 3DS       | `*-nintendo-3ds`   | [`getrandom`][18]
//! | PS Vita            | `*-vita-*`         | [`getentropy`][13]
//! | QNX Neutrino       | `*‑nto-qnx*`       | [`/dev/urandom`][14] (identical to `/dev/random`)
//! | AIX                | `*-ibm-aix`        | [`/dev/urandom`][15]
//!
//! Pull Requests that add support for new targets to `getrandom` are always welcome.
//!
//! ## Opt-in backends
//!
//! `getrandom` also provides optional backends which can be enabled using `getrandom_backend`
//! configuration flag:
//!
//! | Backend name      | Target               | Target Triple        | Implementation
//! | ----------------- | -------------------- | -------------------- | --------------
//! | `linux_getrandom` | Linux, Android       | `*‑linux‑*`          | [`getrandom`][1] system call (without `/dev/urandom` fallback). Bumps minimum supported Linux kernel version to 3.17 and Android API level to 23 (Marshmallow).
//! | `rdrand`          | x86, x86-64          | `x86_64-*`, `i686-*` | [`RDRAND`] instruction
//! | `rndr`            | AArch64              | `aarch64-*`          | [`RNDR`] register
//! | `esp_idf`         | ESP-IDF              | `*‑espidf`           | [`esp_fill_random`]. WARNING: can return low quality entropy without proper hardware configuration!
//! | `wasm_js`         | Web Browser, Node.js | `wasm*‑*‑unknown`    | [`Crypto.getRandomValues`] if available, then [`crypto.randomFillSync`] if on Node.js (see [WebAssembly support])
//! | `custom`          | All targets          | `*`                  | User-provided custom implementation (see [custom backend])
//!
//! The configuration flag can be enabled either by specifying the `rustflags` field in
//! [`.cargo/config.toml`] (note that it can be done on a per-target basis), or by using
//! `RUSTFLAGS` environment variable:
//!
//! ```sh
//! RUSTFLAGS='--cfg getrandom_backend="linux_getrandom"' cargo build
//! ```
//!
//! Enabling an opt-in backend will replace backend used by default. Doing it for a wrong target
//! (e.g. using `linux_getrandom` while compiling for a Windows target) will result
//! in a compilation error. Be extremely carefull while using opt-in backends, since incorrect
//! configuration may result in vulnerable or in always panicking applications.
//!
//! Note that using an opt-in backend in a library (e.g. for tests or benchmarks)
//! WILL NOT have any effect on its downstream users.
//!
//! [`.cargo/config.toml`]: https://doc.rust-lang.org/cargo/reference/config.html
//!
//! ### WebAssembly support
//!
//! This crate fully supports the [WASI] and [Emscripten] targets. However,
//! the `wasm32-unknown-unknown` target (i.e. the target used by `wasm-pack`)
//! is not automatically supported since, from the target name alone, we cannot deduce
//! which JavaScript interface should be used (or if JavaScript is available at all).
//!
//! Instead, *if the `wasm_js` backend is enabled*, this crate will assume
//! that you are building for an environment containing JavaScript, and will
//! call the appropriate methods. Both web browser (main window and Web Workers)
//! and Node.js environments are supported, invoking the methods
//! [described above](#opt-in-backends) using the [`wasm-bindgen`] toolchain.
//!
//! To enable the `wasm_js` backend, you can add the following lines to your
//! project's `.cargo/config.toml` file:
//! ```toml
//! [target.wasm32-unknown-unknown]
//! rustflags = ['--cfg', 'getrandom_backend="wasm_js"']
//! ```
//!
//! #### Node.js ES module support
//!
//! Node.js supports both [CommonJS modules] and [ES modules]. Due to
//! limitations in wasm-bindgen's [`module`] support, we cannot directly
//! support ES Modules running on Node.js. However, on Node v15 and later, the
//! module author can add a simple shim to support the Web Cryptography API:
//! ```js
//! import { webcrypto } from 'node:crypto'
//! globalThis.crypto = webcrypto
//! ```
//! This crate will then use the provided `webcrypto` implementation.
//!
//! ### Custom backend
//!
//! If this crate does not support your target out of box or you have to use
//! a non-default entropy source, then you can provide a custom implementation.
//! You need to enable the custom backend as described in the [configuration flags]
//! section. Next, you need to define an `extern` function with the following
//! signature:
//!
//! ```
//! use getrandom::Error;
//!
//! #[no_mangle]
//! unsafe extern "Rust" fn __getrandom_custom(dest: *mut u8, len: usize) -> Result<(), Error> {
//!     todo!()
//! }
//! ```
//!
//! This function ideally should be defined in the root crate of your project,
//! e.g. in your `main.rs`. This function MUST be defined only once for your
//! project, i.e. upstream library crates SHOULD NOT define it outside of
//! tests and benchmarks. Improper configuration of this backend may result
//! in linking errors.
//!
//! The function accepts pointer to buffer which should be filled with random
//! data and length in bytes. Note that the buffer MAY be uninitialized.
//! On success the function should return 0 and fully fill the input buffer,
//! every other return result will be interpreted as an error code.
//!
//! If you are confident that `getrandom` is not used in your project, but
//! it gets pulled nevertheless by one of your dependencies, then you can
//! use the following custom backend which always returns "unsupported" error:
//! ```
//! use getrandom::Error;
//!
//! #[no_mangle]
//! unsafe extern "Rust" fn __getrandom_custom(dest: *mut u8, len: usize) -> Result<(), Error> {
//!     Err(Error::UNSUPPORTED)
//! }
//! ```
//!
//! ### Platform Support
//! This crate generally supports the same operating system and platform versions
//! that the Rust standard library does. Additional targets may be supported using
//! pluggable custom implementations.
//!
//! This means that as Rust drops support for old versions of operating systems
//! (such as old Linux kernel versions, Android API levels, etc) in stable releases,
//! `getrandom` may create new patch releases (`0.N.x`) that remove support for
//! outdated platform versions.
//!
//! ## `/dev/urandom` fallback on Linux and Android
//!
//! On Linux targets the fallback is present only if either `target_env` is `musl`,
//! or `target_arch` is one of the following: `aarch64`, `arm`, `powerpc`, `powerpc64`,
//! `s390x`, `x86`, `x86_64`. Other supported targets [require][platform-support]
//! kernel versions which support `getrandom` system call, so fallback is not needed.
//!
//! On Android targets the fallback is present only for the following `target_arch`es:
//! `aarch64`, `arm`, `x86`, `x86_64`. Other `target_arch`es (e.g. RISC-V) require
//! sufficiently high API levels.
//!
//! The fallback can be disabled by enabling the `linux_getrandom` opt-in backend.
//! Note that doing so will bump minimum supported Linux kernel version to 3.17 and
//! Android API level to 23 (Marshmallow).
//!
//! ## Early boot
//!
//! Sometimes, early in the boot process, the OS has not collected enough
//! entropy to securely seed its RNG. This is especially common on virtual
//! machines, where standard "random" events are hard to come by.
//!
//! Some operating system interfaces always block until the RNG is securely
//! seeded. This can take anywhere from a few seconds to more than a minute.
//! A few (Linux, NetBSD and Solaris) offer a choice between blocking and
//! getting an error; in these cases, we always choose to block.
//!
//! On Linux (when the `getrandom` system call is not available), reading from
//! `/dev/urandom` never blocks, even when the OS hasn't collected enough
//! entropy yet. To avoid returning low-entropy bytes, we first poll
//! `/dev/random` and only switch to `/dev/urandom` once this has succeeded.
//!
//! On OpenBSD, this kind of entropy accounting isn't available, and on
//! NetBSD, blocking on it is discouraged. On these platforms, nonblocking
//! interfaces are used, even when reliable entropy may not be available.
//! On the platforms where it is used, the reliability of entropy accounting
//! itself isn't free from controversy. This library provides randomness
//! sourced according to the platform's best practices, but each platform has
//! its own limits on the grade of randomness it can promise in environments
//! with few sources of entropy.
//!
//! ## Error handling
//!
//! We always choose failure over returning known insecure "random" bytes. In
//! general, on supported platforms, failure is highly unlikely, though not
//! impossible. If an error does occur, then it is likely that it will occur
//! on every call to `getrandom`, hence after the first successful call one
//! can be reasonably confident that no errors will occur.
//!
//! [1]: https://manned.org/getrandom.2
//! [2]: https://manned.org/urandom.4
//! [3]: https://www.unix.com/man-page/mojave/2/getentropy/
//! [4]: https://www.unix.com/man-page/mojave/4/urandom/
//! [5]: https://www.freebsd.org/cgi/man.cgi?query=getrandom&manpath=FreeBSD+12.0-stable
//! [7]: https://man.openbsd.org/getentropy.2
//! [8]: https://man.netbsd.org/sysctl.7
//! [9]: https://leaf.dragonflybsd.org/cgi/web-man?command=getrandom
//! [11]: https://docs.oracle.com/cd/E88353_01/html/E37841/getrandom-2.html
//! [12]: https://illumos.org/man/2/getrandom
//! [13]: https://github.com/emscripten-core/emscripten/pull/12240
//! [14]: https://www.qnx.com/developers/docs/7.1/index.html#com.qnx.doc.neutrino.utilities/topic/r/random.html
//! [15]: https://www.ibm.com/docs/en/aix/7.3?topic=files-random-urandom-devices
//! [16]: https://man.netbsd.org/getrandom.2
//! [17]: https://www.gnu.org/software/libc/manual/html_mono/libc.html#index-getrandom
//! [18]: https://github.com/rust3ds/shim-3ds/commit/b01d2568836dea2a65d05d662f8e5f805c64389d
//!
//! [`ProcessPrng`]: https://learn.microsoft.com/en-us/windows/win32/seccng/processprng
//! [`RtlGenRandom`]: https://learn.microsoft.com/en-us/windows/win32/api/ntsecapi/nf-ntsecapi-rtlgenrandom
//! [`Crypto.getRandomValues`]: https://www.w3.org/TR/WebCryptoAPI/#Crypto-method-getRandomValues
//! [`RDRAND`]: https://software.intel.com/en-us/articles/intel-digital-random-number-generator-drng-software-implementation-guide
//! [`RNDR`]: https://developer.arm.com/documentation/ddi0601/2024-06/AArch64-Registers/RNDR--Random-Number
//! [`CCRandomGenerateBytes`]: https://opensource.apple.com/source/CommonCrypto/CommonCrypto-60074/include/CommonRandom.h.auto.html
//! [`cprng_draw`]: https://fuchsia.dev/fuchsia-src/zircon/syscalls/cprng_draw
//! [`crypto.randomFillSync`]: https://nodejs.org/api/crypto.html#cryptorandomfillsyncbuffer-offset-size
//! [`esp_fill_random`]: https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/system/random.html#_CPPv415esp_fill_randomPv6size_t
//! [`random_get`]: https://github.com/WebAssembly/WASI/blob/snapshot-01/phases/snapshot/docs.md#-random_getbuf-pointeru8-buf_len-size---errno
//! [`get-random-u64`]: https://github.com/WebAssembly/WASI/blob/v0.2.1/wasip2/random/random.wit#L23-L28
//! [WebAssembly support]: #webassembly-support
//! [configuration flags]: #configuration-flags
//! [custom backend]: #custom-backend
//! [`wasm-bindgen`]: https://github.com/rustwasm/wasm-bindgen
//! [`module`]: https://rustwasm.github.io/wasm-bindgen/reference/attributes/on-js-imports/module.html
//! [CommonJS modules]: https://nodejs.org/api/modules.html
//! [ES modules]: https://nodejs.org/api/esm.html
//! [`sys_read_entropy`]: https://github.com/hermit-os/kernel/blob/315f58ff5efc81d9bf0618af85a59963ff55f8b1/src/syscalls/entropy.rs#L47-L55
//! [platform-support]: https://doc.rust-lang.org/stable/rustc/platform-support.html
//! [WASI]: https://github.com/CraneStation/wasi
//! [Emscripten]: https://www.hellorust.com/setup/emscripten/

#![doc(
    html_logo_url = "https://www.rust-lang.org/logos/rust-logo-128x128-blk.png",
    html_favicon_url = "https://www.rust-lang.org/favicon.ico",
    html_root_url = "https://docs.rs/getrandom/0.2.15"
)]
#![no_std]
#![warn(rust_2018_idioms, unused_lifetimes, missing_docs)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
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
// These should all provide getrandom_inner with the signature
// `fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error>`.
// The function MUST fully initialize `dest` when `Ok(())` is returned.
// The function MUST NOT ever write uninitialized bytes into `dest`,
// regardless of what value it returns.
cfg_if! {
    if #[cfg(getrandom_backend = "custom")] {
        #[path = "custom.rs"] mod imp;
    } else if #[cfg(getrandom_backend = "linux_getrandom")] {
        #[cfg(not(any(target_os = "android", target_os = "linux")))]
        compile_error!("`linux_getrandom` backend can be enabled only for Linux/Android targets!");
        mod util_libc;
        #[path = "linux_android.rs"] mod imp;
    } else if #[cfg(getrandom_backend = "rdrand")] {
        #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
        compile_error!("`rdrand` backend can be enabled only for x86 and x86-64 targets!");

        mod lazy;
        #[path = "rdrand.rs"] mod imp;
    } else if #[cfg(getrandom_backend = "rndr")] {
        #[path = "rndr.rs"] mod imp;
    } else if #[cfg(getrandom_backend = "wasm_js")] {
        #[cfg(not(all(
            any(target_arch = "wasm32", target_arch = "wasm64"),
            target_os = "unknown",
        )))]
        compile_error!("`wasm_js` backend can be enabled only on OS-less WASM targets!");
        #[path = "js.rs"] mod imp;
    } else if #[cfg(getrandom_backend = "esp_idf")] {
        #[cfg(not(target_os = "espidf"))]
        compile_error!("`esp_idf` backend can be enabled only for ESP-IDF targets!");
        #[path = "espidf.rs"] mod imp;
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
        mod lazy;
        mod util_libc;
        mod use_file;
        mod linux_android;
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

/// Fill `dest` with random bytes from the system's preferred random number
/// source.
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
#[inline]
pub fn getrandom(dest: &mut [u8]) -> Result<(), Error> {
    // SAFETY: The `&mut MaybeUninit<_>` reference doesn't escape, and
    // `getrandom_uninit` guarantees it will never de-initialize any part of
    // `dest`.
    getrandom_uninit(unsafe { slice_as_uninit_mut(dest) })?;
    Ok(())
}

/// Version of the `getrandom` function which fills `dest` with random bytes
/// returns a mutable reference to those bytes.
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
/// let buf: &mut [u8] = getrandom::getrandom_uninit(&mut buf)?;
/// # Ok(()) }
/// ```
#[inline]
pub fn getrandom_uninit(dest: &mut [MaybeUninit<u8>]) -> Result<&mut [u8], Error> {
    if !dest.is_empty() {
        imp::getrandom_inner(dest)?;
    }
    // SAFETY: `dest` has been fully initialized by `imp::getrandom_inner`
    // since it returned `Ok`.
    Ok(unsafe { slice_assume_init_mut(dest) })
}
