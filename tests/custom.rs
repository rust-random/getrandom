use core::num::NonZeroU32;
use getrandom::{getrandom, register_custom_getrandom, Error};
#[cfg(all(target_family = "wasm", target_os = "unknown"))]
use wasm_bindgen_test::wasm_bindgen_test as test;

const LEN7_CODE: u32 = Error::CUSTOM_START + 7;
// Returns a custom error if input is length 7, otherwise fills with 0x55.
fn mock_rng(buf: &mut [u8]) -> Result<(), Error> {
    // `getrandom` guarantees it will not call any implementation if the output
    // buffer is empty.
    assert!(!buf.is_empty());
    if buf.len() == 7 {
        return Err(NonZeroU32::new(LEN7_CODE).unwrap().into());
    }
    buf.fill(0x55);
    Ok(())
}

// Test registering a custom implementation, even on supported platforms.
register_custom_getrandom!(mock_rng);

// Invoking with an empty buffer should never call the custom implementation.
#[test]
fn custom_empty() {
    getrandom(&mut []).unwrap();
}

// On a supported platform, make sure the custom implementation isn't used. We
// test on a few common platfroms, rather than duplicating the lib.rs logic.
#[cfg(any(
    target_os = "linux",
    target_os = "windows",
    target_os = "macos",
    target_os = "espidf",
    target_os = "wasi",
    all(target_family = "wasm", target_os = "unknown", feature = "js"),
))]
#[test]
fn custom_not_used() {
    getrandom(&mut [0; 7]).unwrap();
}
// On an unsupported platform, make sure the custom implementation is used.
#[cfg(all(target_family = "wasm", target_os = "unknown", not(feature = "js")))]
#[test]
fn custom_used() {
    let err = getrandom(&mut [0; 7]).unwrap_err();
    assert_eq!(err.code().get(), LEN7_CODE);

    let mut buf = [0; 12];
    getrandom(&mut buf).unwrap();
    assert_eq!(buf, [0x55; 12]);
}
