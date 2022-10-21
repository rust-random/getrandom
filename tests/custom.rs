// Test that a custom handler works on wasm32-unknown-unknown
#![cfg(all(
    target_arch = "wasm32",
    target_os = "unknown",
    feature = "custom",
    not(feature = "js")
))]

use getrandom::getrandom as getrandom_impl;

mod common;
mod custom_common;
