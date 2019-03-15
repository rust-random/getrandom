// Copyright 2019 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Interface to the random number generator of the operating system.
//!
//! # Platform sources
//!
//! | OS               | interface
//! |------------------|---------------------------------------------------------
//! | Linux, Android   | [`getrandom`][1] system call if available, otherwise [`/dev/urandom`][2] after reading from `/dev/random` once
//! | Windows          | [`RtlGenRandom`][3]
//! | macOS, iOS       | [`SecRandomCopyBytes`][4]
//! | FreeBSD          | [`kern.arandom`][5]
//! | OpenBSD, Bitrig  | [`getentropy`][6]
//! | NetBSD           | [`/dev/urandom`][7] after reading from `/dev/random` once
//! | Dragonfly BSD    | [`/dev/random`][8]
//! | Solaris, illumos | [`getrandom`][9] system call if available, otherwise [`/dev/random`][10]
//! | Fuchsia OS       | [`cprng_draw`][11]
//! | Redox            | [`rand:`][12]
//! | CloudABI         | [`random_get`][13]
//! | Haiku            | `/dev/random` (identical to `/dev/urandom`)
//! | SGX              | RDRAND
//! | Web browsers     | [`Crypto.getRandomValues`][14] (see [Support for WebAssembly and ams.js][14])
//! | Node.js          | [`crypto.randomBytes`][15] (see [Support for WebAssembly and ams.js][16])
//!
//! Getrandom doesn't have a blanket implementation for all Unix-like operating
//! systems that reads from `/dev/urandom`. This ensures all supported operating
//! systems are using the recommended interface and respect maximum buffer
//! sizes.
//!
//! ## Support for WebAssembly and ams.js
//!
//! The three Emscripten targets `asmjs-unknown-emscripten`,
//! `wasm32-unknown-emscripten` and `wasm32-experimental-emscripten` use
//! Emscripten's emulation of `/dev/random` on web browsers and Node.js.
//!
//! The bare WASM target `wasm32-unknown-unknown` tries to call the javascript
//! methods directly, using either `stdweb` or `wasm-bindgen` depending on what
//! features are activated for this crate. Note that if both features are
//! enabled `wasm-bindgen` will be used. If neither feature is enabled,
//! `getrandom` will always fail.
//!
//! ## Early boot
//!
//! It is possible that early in the boot process the OS hasn't had enough time
//! yet to collect entropy to securely seed its RNG, especially on virtual
//! machines.
//!
//! Some operating systems always block the thread until the RNG is securely
//! seeded. This can take anywhere from a few seconds to more than a minute.
//! Others make a best effort to use a seed from before the shutdown and don't
//! document much.
//!
//! A few, Linux, NetBSD and Solaris, offer a choice between blocking and
//! getting an error; in these cases we always choose to block.
//!
//! On Linux (when the `genrandom` system call is not available) and on NetBSD
//! reading from `/dev/urandom` never blocks, even when the OS hasn't collected
//! enough entropy yet. To avoid returning low-entropy bytes, we first read from
//! `/dev/random` and only switch to `/dev/urandom` once this has succeeded.
//!
//! # Error handling
//!
//! We always choose failure over returning insecure "random" bytes. In general,
//! on supported platforms, failure is highly unlikely, though not impossible.
//! If an error does occur, then it is likely that it will occur on every call to
//! `getrandom`, hence after the first successful call one can be reasonably
//! confident that no errors will occur.
//!
//! On unsupported platforms, `getrandom` always fails with [`Error::UNAVAILABLE`].
//!
//! ## Error codes
//! The crate uses the following custom error codes:
//! - `0x57f4c500` (dec: 1475659008) - an unknown error. Constant:
//! [`Error::UNKNOWN`]
//! - `0x57f4c501` (dec: 1475659009) - no generator is available. Constant:
//! [`Error::UNAVAILABLE`]
//! - `0x57f4c580` (dec: 1475659136) - `self.crypto` is undefined,
//! `wasm-bindgen` specific error.
//! - `0x57f4c581` (dec: 1475659137) - `crypto.getRandomValues` is undefined,
//! `wasm-bindgen` specific error.
//!
//! These codes are provided for reference only and should not be matched upon
//! (but you can match on `Error` constants). The codes may change in future and
//! such change will not be considered a breaking one.
//!
//! Other error codes will originate from an underlying system. In case if such
//! error is encountered, please consult with your system documentation.
//!
//! [1]: http://man7.org/linux/man-pages/man2/getrandom.2.html
//! [2]: http://man7.org/linux/man-pages/man4/urandom.4.html
//! [3]: https://msdn.microsoft.com/en-us/library/windows/desktop/aa387694.aspx
//! [4]: https://developer.apple.com/documentation/security/1399291-secrandomcopybytes?language=objc
//! [5]: https://www.freebsd.org/cgi/man.cgi?query=random&sektion=4
//! [6]: https://man.openbsd.org/getentropy.2
//! [7]: http://netbsd.gw.com/cgi-bin/man-cgi?random+4+NetBSD-current
//! [8]: https://leaf.dragonflybsd.org/cgi/web-man?command=random&section=4
//! [9]: https://docs.oracle.com/cd/E88353_01/html/E37841/getrandom-2.html
//! [10]: https://docs.oracle.com/cd/E86824_01/html/E54777/random-7d.html
//! [11]: https://fuchsia.googlesource.com/zircon/+/HEAD/docs/syscalls/cprng_draw.md
//! [12]: https://github.com/redox-os/randd/blob/master/src/main.rs
//! [13]: https://github.com/NuxiNL/cloudabi/blob/v0.20/cloudabi.txt#L1826
//! [14]: https://www.w3.org/TR/WebCryptoAPI/#Crypto-method-getRandomValues
//! [15]: https://nodejs.org/api/crypto.html#crypto_crypto_randombytes_size_callback
//! [16]: #support-for-webassembly-and-amsjs

#![doc(html_logo_url = "https://www.rust-lang.org/logos/rust-logo-128x128-blk.png",
       html_favicon_url = "https://www.rust-lang.org/favicon.ico",
       html_root_url = "https://rust-random.github.io/rand/")]

#![no_std]

#![cfg_attr(feature = "stdweb", recursion_limit="128")]

#[cfg(not(target_env = "sgx"))]
#[macro_use] extern crate std;

// We have to do it here because we load macros
#[cfg(all(target_arch = "wasm32", not(target_os = "emscripten"),
          feature = "wasm-bindgen"))]
extern crate wasm_bindgen;
#[cfg(all(target_arch = "wasm32", not(target_os = "emscripten"),
          not(feature = "wasm-bindgen"),
          feature = "stdweb"))]
#[macro_use] extern crate stdweb;

#[cfg(feature = "log")] #[macro_use] extern crate log;
#[allow(unused)]
#[cfg(not(feature = "log"))] macro_rules! error { ($($x:tt)*) => () }

#[cfg(any(
    target_os = "android",
    target_os = "netbsd",
    target_os = "solaris",
    target_os = "illumos",
    target_os = "redox",
    target_os = "dragonfly",
    target_os = "haiku",
    target_os = "linux",
    target_arch = "wasm32",
))]
mod utils;
mod error;
pub use error::Error;

// System-specific implementations.
//
// These should all provide getrandom_inner with the same signature as getrandom.

macro_rules! mod_use {
    ($cond:meta, $module:ident) => {
        #[$cond]
        mod $module;
        #[$cond]
        use $module::{getrandom_inner, error_msg_inner};
    }
}

mod_use!(cfg(target_os = "android"), linux_android);
mod_use!(cfg(target_os = "bitrig"), openbsd_bitrig);
mod_use!(cfg(target_os = "cloudabi"), cloudabi);
mod_use!(cfg(target_os = "dragonfly"), dragonfly_haiku);
mod_use!(cfg(target_os = "emscripten"), emscripten);
mod_use!(cfg(target_os = "freebsd"), freebsd);
mod_use!(cfg(target_os = "fuchsia"), fuchsia);
mod_use!(cfg(target_os = "haiku"), dragonfly_haiku);
mod_use!(cfg(target_os = "ios"), macos);
mod_use!(cfg(target_os = "linux"), linux_android);
mod_use!(cfg(target_os = "macos"), macos);
mod_use!(cfg(target_os = "netbsd"), netbsd);
mod_use!(cfg(target_os = "openbsd"), openbsd_bitrig);
mod_use!(cfg(target_os = "redox"), redox);
mod_use!(cfg(any(target_os = "solaris", target_os = "illumos")), solarish);
mod_use!(cfg(windows), windows);
mod_use!(cfg(target_env = "sgx"), sgx);

mod_use!(
    cfg(all(
        target_arch = "wasm32",
        not(target_os = "emscripten"),
        feature = "wasm-bindgen"
    )),
    wasm32_bindgen
);

mod_use!(
    cfg(all(
        target_arch = "wasm32",
        not(target_os = "emscripten"),
        not(feature = "wasm-bindgen"),
        feature = "stdweb",
    )),
    wasm32_stdweb
);

mod_use!(
    cfg(not(any(
        target_os = "android",
        target_os = "bitrig",
        target_os = "cloudabi",
        target_os = "dragonfly",
        target_os = "emscripten",
        target_os = "freebsd",
        target_os = "fuchsia",
        target_os = "haiku",
        target_os = "illumos",
        target_os = "ios",
        target_os = "linux",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "redox",
        target_os = "solaris",
        target_env = "sgx",
        windows,
        all(
            target_arch = "wasm32",
            any(
                target_os = "emscripten",
                feature = "wasm-bindgen",
                feature = "stdweb",
            ),
        ),
    ))),
    dummy
);


/// Fill `dest` with random bytes from the system's preferred random number
/// source.
///
/// This function returns an error on any failure, including partial reads. We
/// make no guarantees regarding the contents of `dest` on error.
///
/// Blocking is possible, at least during early boot; see module documentation.
///
/// In general, `getrandom` will be fast enough for interactive usage, though
/// significantly slower than a user-space CSPRNG; for the latter consider
/// [`rand::thread_rng`](https://docs.rs/rand/*/rand/fn.thread_rng.html).
pub fn getrandom(dest: &mut [u8]) -> Result<(), error::Error> {
    getrandom_inner(dest)
}

// Due to rustwasm/wasm-bindgen#201 this can't be defined in the inner os
// modules, so hack around it for now and place it at the root.
#[cfg(all(feature = "wasm-bindgen", target_arch = "wasm32"))]
#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub mod __wbg_shims {

    // `extern { type Foo; }` isn't supported on 1.22 syntactically, so use a
    // macro to work around that.
    macro_rules! rust_122_compat {
        ($($t:tt)*) => ($($t)*)
    }

    rust_122_compat! {
        extern crate wasm_bindgen;

        pub use wasm_bindgen::prelude::*;

        #[wasm_bindgen]
        extern "C" {
            pub type Function;
            #[wasm_bindgen(constructor)]
            pub fn new(s: &str) -> Function;
            #[wasm_bindgen(method)]
            pub fn call(this: &Function, self_: &JsValue) -> JsValue;

            pub type This;
            #[wasm_bindgen(method, getter, structural, js_name = self)]
            pub fn self_(me: &This) -> JsValue;
            #[wasm_bindgen(method, getter, structural)]
            pub fn crypto(me: &This) -> JsValue;

            #[derive(Clone, Debug)]
            pub type BrowserCrypto;

            // TODO: these `structural` annotations here ideally wouldn't be here to
            // avoid a JS shim, but for now with feature detection they're
            // unavoidable.
            #[wasm_bindgen(method, js_name = getRandomValues, structural, getter)]
            pub fn get_random_values_fn(me: &BrowserCrypto) -> JsValue;
            #[wasm_bindgen(method, js_name = getRandomValues, structural)]
            pub fn get_random_values(me: &BrowserCrypto, buf: &mut [u8]);

            #[wasm_bindgen(js_name = require)]
            pub fn node_require(s: &str) -> NodeCrypto;

            #[derive(Clone, Debug)]
            pub type NodeCrypto;

            #[wasm_bindgen(method, js_name = randomFillSync, structural)]
            pub fn random_fill_sync(me: &NodeCrypto, buf: &mut [u8]);
        }
    }
}
