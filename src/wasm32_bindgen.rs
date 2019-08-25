// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for WASM via wasm-bindgen
use js_sys::global;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{window, Crypto, WorkerGlobalScope};

use crate::util::LazyBool;
use crate::{error, Error};

pub fn getrandom_inner(dest: &mut [u8]) -> Result<(), Error> {
    static IS_NODE: LazyBool = LazyBool::new();
    if IS_NODE.unsync_init(|| node_crypto().is_some()) {
        if node_crypto().unwrap().random_fill_sync(dest).is_err() {
            return Err(error::BINDGEN_NODE_FAILED);
        }
    } else {
        let crypto = browser_crypto().ok_or(error::BINDGEN_WEB_CRYPTO)?;
        // https://developer.mozilla.org/en-US/docs/Web/API/Crypto/getRandomValues
        // > A QuotaExceededError DOMException is thrown if the
        // > requested length is greater than 65536 bytes.
        for chunk in dest.chunks_mut(65536) {
            if crypto.get_random_values_with_u8_array(chunk).is_err() {
                return Err(error::BINDGEN_WEB_FAILED);
            }
        }
    }
    Ok(())
}

fn node_crypto() -> Option<NodeCrypto> {
    node_require("crypto").ok()
}

fn browser_crypto() -> Option<Crypto> {
    // Support calling self.crypto in the main window or a Web Worker.
    if let Some(window) = window() {
        return window.crypto().ok();
    }
    let worker = global().dyn_into::<WorkerGlobalScope>().ok()?;
    worker.crypto().ok()
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_name = require)]
    fn node_require(s: &str) -> Result<NodeCrypto, JsValue>;

    #[derive(Clone, Debug)]
    type NodeCrypto;

    #[wasm_bindgen(catch, method, js_name = randomFillSync)]
    fn random_fill_sync(me: &NodeCrypto, buf: &mut [u8]) -> Result<(), JsValue>;
}
