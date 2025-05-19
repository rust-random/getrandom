//! Implementation for WASM based on Web and Node.js
use crate::Backend;
use crate::Error;
use core::mem::MaybeUninit;

#[cfg(not(all(target_arch = "wasm32", any(target_os = "unknown", target_os = "none"))))]
compile_error!("`wasm_js` backend can be enabled only for OS-less WASM targets!");

use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

pub struct WasmJsBackend;

unsafe impl Backend for WasmJsBackend {
    #[inline]
    unsafe fn fill_ptr(dest: *mut u8, len: usize) -> Result<(), Error> {
        let slice = core::slice::from_raw_parts_mut(dest as *mut MaybeUninit<u8>, len);
        Self::fill_uninit(slice)
    }

    #[cfg(not(target_feature = "atomics"))]
    #[inline]
    fn fill_uninit(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
        for chunk in dest.chunks_mut(MAX_BUFFER_SIZE) {
            if get_random_values(chunk).is_err() {
                return Err(Error::WEB_CRYPTO);
            }
        }
        Ok(())
    }

    #[cfg(target_feature = "atomics")]
    #[inline]
    fn fill_uninit(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
        // getRandomValues does not work with all types of WASM memory,
        // so we initially write to browser memory to avoid exceptions.
        let buf_len = usize::min(dest.len(), MAX_BUFFER_SIZE);
        let buf_len_u32 = buf_len
            .try_into()
            .expect("buffer length is bounded by MAX_BUFFER_SIZE");
        let buf = js_sys::Uint8Array::new_with_length(buf_len_u32);
        for chunk in dest.chunks_mut(buf_len) {
            let chunk_len = chunk
                .len()
                .try_into()
                .expect("chunk length is bounded by MAX_BUFFER_SIZE");
            // The chunk can be smaller than buf's length, so we call to
            // JS to create a smaller view of buf without allocation.
            let sub_buf = if chunk_len == buf_len_u32 {
                &buf
            } else {
                &buf.subarray(0, chunk_len)
            };

            if get_random_values(sub_buf).is_err() {
                return Err(Error::new_custom(WEB_CRYPTO));
            }

            sub_buf.copy_to_uninit(chunk);
        }
        Ok(())
    }

    #[inline]
    fn describe_custom_error(n: u16) -> Option<&'static str> {
        if n == WEB_CRYPTO {
            Some("Web Crypto API is unavailable")
        } else {
            None
        }
    }
}

#[wasm_bindgen]
extern "C" {
    // Crypto.getRandomValues()
    #[cfg(not(target_feature = "atomics"))]
    #[wasm_bindgen(js_namespace = ["globalThis", "crypto"], js_name = getRandomValues, catch)]
    fn get_random_values(buf: &mut [MaybeUninit<u8>]) -> Result<(), JsValue>;
    #[cfg(target_feature = "atomics")]
    #[wasm_bindgen(js_namespace = ["globalThis", "crypto"], js_name = getRandomValues, catch)]
    fn get_random_values(buf: &js_sys::Uint8Array) -> Result<(), JsValue>;
}

/// The environment does not support the Web Crypto API.
const WEB_CRYPTO: u16 = 10;

/// Maximum buffer size allowed in `Crypto.getRandomValuesSize` is 65536 bytes.
/// See https://developer.mozilla.org/en-US/docs/Web/API/Crypto/getRandomValues
const MAX_BUFFER_SIZE: usize = 65536;
