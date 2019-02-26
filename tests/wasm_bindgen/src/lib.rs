// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Crate to test WASM with the `wasm-bindgen` lib.

#![doc(html_logo_url = "https://www.rust-lang.org/logos/rust-logo-128x128-blk.png")]

extern crate getrandom;
extern crate wasm_bindgen;
extern crate wasm_bindgen_test;

use std::slice;
use wasm_bindgen_test::*;
use wasm_bindgen::prelude::*;

use getrandom::getrandom;

#[wasm_bindgen]
pub fn test_gen() -> i32 {
    let mut int: i32 = 0;
    unsafe {
        let ptr = &mut int as *mut i32 as *mut u8;
        let slice = slice::from_raw_parts_mut(ptr, 4);
        getrandom(slice).unwrap();
    }
    int
}

#[wasm_bindgen_test]
fn test_call() {
    let mut buf = [0u8; 0];
    getrandom(&mut buf).unwrap();
}

#[wasm_bindgen_test]
fn test_diff() {
    let mut v1 = [0u8; 1000];
    getrandom(&mut v1).unwrap();

    let mut v2 = [0u8; 1000];
    getrandom(&mut v2).unwrap();

    let mut n_diff_bits = 0;
    for i in 0..v1.len() {
        n_diff_bits += (v1[i] ^ v2[i]).count_ones();
    }

    // Check at least 1 bit per byte differs. p(failure) < 1e-1000 with random input.
    assert!(n_diff_bits >= v1.len() as u32);
}
