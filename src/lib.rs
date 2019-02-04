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
