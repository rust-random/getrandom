//! Implementation for WASM based on Web and Node.js
use crate::Error;
use core::mem::MaybeUninit;

pub use crate::util::{inner_u32, inner_u64};

#[cfg(not(all(target_arch = "wasm32", any(target_os = "unknown", target_os = "none"))))]
compile_error!("`wasm_js` backend can be enabled only for OS-less WASM targets!");

use js_sys::{global, Uint8Array};
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue};

// Size of our temporary Uint8Array buffer used with WebCrypto methods
// Maximum is 65536 bytes see https://developer.mozilla.org/en-US/docs/Web/API/Crypto/getRandomValues
const WEB_CRYPTO_BUFFER_SIZE: u16 = 256;

pub fn fill_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    let global: Global = global().unchecked_into();
    let crypto = global.crypto();

    // getRandomValues does not work with all types of WASM memory,
    // so we initially write to browser memory to avoid exceptions.
    let buf = Uint8Array::new_with_length(WEB_CRYPTO_BUFFER_SIZE.into());
    for chunk in dest.chunks_mut(WEB_CRYPTO_BUFFER_SIZE.into()) {
        let chunk_len: u32 = chunk
            .len()
            .try_into()
            .expect("chunk length is bounded by WEB_CRYPTO_BUFFER_SIZE");
        // The chunk can be smaller than buf's length, so we call to
        // JS to create a smaller view of buf without allocation.
        let sub_buf = buf.subarray(0, chunk_len);

        if crypto.get_random_values(&sub_buf).is_err() {
            return Err(Error::WEB_GET_RANDOM_VALUES);
        }

        // SAFETY: `sub_buf`'s length is the same length as `chunk`
        unsafe { sub_buf.raw_copy_to_ptr(chunk.as_mut_ptr().cast::<u8>()) };
    }
    Ok(())
}

#[wasm_bindgen]
extern "C" {
    // Return type of js_sys::global()
    type Global;
    // Web Crypto API: Crypto interface (https://www.w3.org/TR/WebCryptoAPI/)
    type WebCrypto;
    // Getters for the WebCrypto API
    #[wasm_bindgen(method, getter)]
    fn crypto(this: &Global) -> WebCrypto;
    // Crypto.getRandomValues()
    #[wasm_bindgen(method, js_name = getRandomValues, catch)]
    fn get_random_values(this: &WebCrypto, buf: &Uint8Array) -> Result<(), JsValue>;
}
