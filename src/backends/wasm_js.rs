//! Implementation for WASM based on Web and Node.js
use crate::Error;
use core::mem::MaybeUninit;

pub use crate::util::{inner_u32, inner_u64};

#[cfg(not(all(target_arch = "wasm32", any(target_os = "unknown", target_os = "none"))))]
compile_error!("`wasm_js` backend can be enabled only for OS-less WASM targets!");

use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

// Maximum buffer size allowed in `Crypto.getRandomValuesSize` is 65536 bytes.
// See https://developer.mozilla.org/en-US/docs/Web/API/Crypto/getRandomValues
const MAX_BUFFER_SIZE: usize = 65536;

pub fn fill_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    CRYPTO.with(|crypto| {
        let crypto = crypto.as_ref().ok_or(Error::WEB_CRYPTO)?;
        inner(crypto, dest)
    })
}

#[cfg(not(target_feature = "atomics"))]
fn inner(crypto: &Crypto, dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    for chunk in dest.chunks_mut(MAX_BUFFER_SIZE) {
        if crypto.get_random_values(chunk).is_err() {
            return Err(Error::WEB_GET_RANDOM_VALUES);
        }
    }
    Ok(())
}

#[cfg(target_feature = "atomics")]
fn inner(crypto: &Crypto, dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
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

        if crypto.get_random_values(sub_buf).is_err() {
            return Err(Error::WEB_GET_RANDOM_VALUES);
        }

        // SAFETY: `sub_buf`'s length is the same length as `chunk`
        unsafe { sub_buf.raw_copy_to_ptr(chunk.as_mut_ptr().cast::<u8>()) };
    }
    Ok(())
}

#[wasm_bindgen]
extern "C" {
    // Web Crypto API: Crypto interface (https://www.w3.org/TR/WebCryptoAPI/)
    type Crypto;
    // Holds the global `Crypto` object.
    #[wasm_bindgen(thread_local_v2, js_namespace = globalThis, js_name = crypto)]
    static CRYPTO: Option<Crypto>;
    // Crypto.getRandomValues()
    #[cfg(not(target_feature = "atomics"))]
    #[wasm_bindgen(method, js_name = getRandomValues, catch)]
    fn get_random_values(this: &Crypto, buf: &mut [MaybeUninit<u8>]) -> Result<(), JsValue>;
    #[cfg(target_feature = "atomics")]
    #[wasm_bindgen(method, js_name = getRandomValues, catch)]
    fn get_random_values(this: &Crypto, buf: &js_sys::Uint8Array) -> Result<(), JsValue>;
}
