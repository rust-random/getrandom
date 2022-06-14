// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use crate::Error;

extern crate std;
use std::thread_local;

use js_sys::Uint8Array;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

// Maximum length for Node's crypto.getRandomValues
const NODE_MAX_BUFFER_SIZE: usize = (1 << 31) - 1;

// Maximum length Web's crypto. is 65536 bytes, see:
// https://developer.mozilla.org/en-US/docs/Web/API/Crypto/getRandomValues
const BROWSER_CRYPTO_BUFFER_SIZE: usize = 256;

enum RngSource {
    Node,
    Web(WebCrypto, Uint8Array),
    Failed(Error),
}

// JsValues are always per-thread, so we initialize RngSource for each thread.
//   See: https://github.com/rustwasm/wasm-bindgen/pull/955
thread_local!(
    static RNG_SOURCE: RngSource = getrandom_init();
);

pub(crate) fn getrandom_inner(dest: &mut [u8]) -> Result<(), Error> {
    RNG_SOURCE.with(|source| {
        match source {
            RngSource::Node => {
                for chunk in dest.chunks_mut(NODE_MAX_BUFFER_SIZE) {
                    if NODE_CRYPTO.random_fill_sync(chunk).is_err() {
                        return Err(Error::NODE_RANDOM_FILL_SYNC);
                    }
                }
            }
            RngSource::Web(crypto, buf) => {
                // getRandomValues does not work with all types of WASM memory,
                // so we initially write to browser memory to avoid exceptions.
                for chunk in dest.chunks_mut(BROWSER_CRYPTO_BUFFER_SIZE) {
                    // The chunk can be smaller than buf's length, so we call to
                    // JS to create a smaller view of buf without allocation.
                    let sub_buf = buf.subarray(0, chunk.len() as u32);

                    if crypto.get_random_values(&sub_buf).is_err() {
                        return Err(Error::WEB_GET_RANDOM_VALUES);
                    }
                    sub_buf.copy_to(chunk);
                }
            }
            RngSource::Failed(err) => return Err(*err),
        }
        Ok(())
    })
}

fn getrandom_init() -> RngSource {
    if *IS_NODE {
        if !NODE_CRYPTO.is_object() {
            return RngSource::Failed(Error::NODE_CRYPTO);
        }
        return RngSource::Node;
    }

    // Assume we are in some Web environment (browser or web worker). We get
    // `self.crypto` (called `msCrypto` on IE), so we can call
    // `crypto.getRandomValues`. If `crypto` isn't defined, we assume that
    // we are in an older web browser and the OS RNG isn't available.
    match web_crypto() {
        Ok(crypto) if crypto.is_object() => {
            let buf = Uint8Array::new_with_length(BROWSER_CRYPTO_BUFFER_SIZE as u32);
            RngSource::Web(crypto, buf)
        }
        _ => RngSource::Failed(Error::WEB_CRYPTO),
    }
}

#[wasm_bindgen]
extern "C" {
    // Web Crypto API (https://www.w3.org/TR/WebCryptoAPI/)
    type WebCrypto;
    #[wasm_bindgen(method, js_name = getRandomValues, catch)]
    fn get_random_values(this: &WebCrypto, buf: &Uint8Array) -> Result<(), JsValue>;

    // Node JS crypto module (https://nodejs.org/api/crypto.html)
    type NodeCrypto;
    #[wasm_bindgen(method, js_name = randomFillSync, catch)]
    fn random_fill_sync(this: &NodeCrypto, buf: &mut [u8]) -> Result<(), JsValue>;
}

#[wasm_bindgen(module = "/src/js.js")]
extern "C" {
    static IS_NODE: bool;
    static NODE_CRYPTO: NodeCrypto;

    #[wasm_bindgen(catch)]
    fn web_crypto() -> Result<WebCrypto, JsValue>;
}
