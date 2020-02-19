// Explicitly use the Custom RNG crate to link it in.
use wasm_bindgen_getrandom as _;

use getrandom::getrandom;
use wasm_bindgen_test::wasm_bindgen_test as test;
#[path = "../../../src/test_common.rs"]
mod test_common;
