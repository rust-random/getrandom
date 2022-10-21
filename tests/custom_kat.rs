// Test that a custom handler works on wasm32-unknown-unknown.
#![cfg(all(
    target_arch = "wasm32",
    target_os = "unknown",
    feature = "custom",
    not(feature = "js")
))]

use wasm_bindgen_test::wasm_bindgen_test as test;
#[cfg(feature = "test-in-browser")]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

mod custom_common;

use custom_common::len7_err;
use getrandom::getrandom;

// This known-answer test cannot be in the same test suite as any other
// tests that use the `custom_common` implementation since the known answers
// depend on the exact state of `custom_common`.
#[test]
fn custom_rng_output() {
    let mut buf = [0u8; 4];
    assert_eq!(getrandom(&mut buf), Ok(()));
    assert_eq!(buf, [0, 1, 2, 3]);
    assert_eq!(getrandom(&mut buf), Ok(()));
    assert_eq!(buf, [4, 5, 6, 7]);
}

#[test]
fn rng_err_output() {
    assert_eq!(getrandom(&mut [0; 7]), Err(len7_err()));
}
