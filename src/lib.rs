// Copyright 2019 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
#![no_std]

#[cfg(any(
    target_os = "android",
    target_os = "netbsd",
    target_os = "solaris",
    target_os = "redox",
    target_os = "dragonfly",
    target_os = "haiku",
    target_os = "emscripten",
    target_os = "linux",
))]
#[macro_use] extern crate std;

#[cfg(any(
    target_os = "android",
    target_os = "netbsd",
    target_os = "solaris",
    target_os = "redox",
    target_os = "dragonfly",
    target_os = "haiku",
    target_os = "emscripten",
    target_os = "linux",
))]
mod utils;
mod error;
pub use error::Error;

macro_rules! mod_use {
    ($cond:meta, $module:ident) => {
        #[$cond]
        mod $module;
        #[$cond]
        pub use $module::getrandom;
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
mod_use!(cfg(target_os = "solaris"), solaris);
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
