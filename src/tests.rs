//! Common tests and testing utilities
extern crate std;

use crate::Error;
use std::{mem::MaybeUninit, sync::mpsc, thread, vec, vec::Vec};

#[cfg(feature = "test-in-browser")]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

fn num_diff_bits(s1: &[u8], s2: &[u8]) -> usize {
    assert_eq!(s1.len(), s2.len());
    s1.iter()
        .zip(s2.iter())
        .map(|(a, b)| (a ^ b).count_ones() as usize)
        .sum()
}

// A function which fills a buffer with random bytes.
type FillFn<B> = fn(&mut [B]) -> Result<(), Error>;

// Helper trait for testing different `FillFn`s.
pub(crate) trait Byte: Sized + 'static {
    fn make_vec(len: usize, fill: FillFn<Self>) -> Vec<u8>;
}
impl Byte for u8 {
    fn make_vec(len: usize, fill: FillFn<u8>) -> Vec<u8> {
        let mut v = vec![0; len];
        fill(&mut v).unwrap();
        v
    }
}
impl Byte for MaybeUninit<u8> {
    fn make_vec(len: usize, fill: FillFn<MaybeUninit<u8>>) -> Vec<u8> {
        // Using Vec::spare_capacity_mut more consistently gives us truly
        // uninitialized memory regardless of optimization level.
        let mut v = Vec::with_capacity(len);
        fill(v.spare_capacity_mut()).unwrap();
        unsafe { v.set_len(len) }
        v
    }
}

// For calls of size `len`, count the number of bits which differ between calls
// and check that between 3 and 5 bits per byte differ. Probability of failure:
// ~ 10^(-30) = 2 * CDF[BinomialDistribution[8*256, 0.5], 3*256]
pub(crate) fn check_bits<B: Byte>(len: usize, fill: FillFn<B>) {
    let mut num_bytes = 0;
    let mut diff_bits = 0;
    while num_bytes < 256 {
        let v1 = B::make_vec(len, fill);
        let v2 = B::make_vec(len, fill);

        num_bytes += len;
        diff_bits += num_diff_bits(&v1, &v2);
    }

    // When the custom feature is enabled, don't check RNG quality.
    assert!(diff_bits > 3 * num_bytes);
    assert!(diff_bits < 5 * num_bytes);
}

pub(crate) fn check_multithreading<B: Byte>(fill: FillFn<B>) {
    let mut txs = vec![];
    for _ in 0..20 {
        let (tx, rx) = mpsc::channel();
        txs.push(tx);

        thread::spawn(move || {
            // wait until all the tasks are ready to go.
            rx.recv().unwrap();
            for _ in 0..100 {
                check_bits(1000, fill);
                thread::yield_now();
            }
        });
    }

    // start all the tasks
    for tx in txs.iter() {
        tx.send(()).unwrap();
    }
}

macro_rules! define_tests {
    ($fill:expr) => {
        #[cfg(all(target_family = "wasm", target_os = "unknown"))]
        use wasm_bindgen_test::wasm_bindgen_test as test;

        #[test]
        fn fill_zero() {
            $fill(&mut []).unwrap();
        }
        #[test]
        fn fill_small() {
            for len in 1..=64 {
                crate::tests::check_bits(len, $fill);
            }
        }
        #[test]
        fn fill_large() {
            crate::tests::check_bits(1_000, $fill);
        }
        #[test]
        fn fill_huge() {
            crate::tests::check_bits(1_000_000, $fill);
        }
        // On WASM, the thread API always fails/panics.
        #[test]
        #[cfg_attr(target_family = "wasm", ignore)]
        fn multithreading() {
            crate::tests::check_multithreading($fill)
        }
    };
}
pub(crate) use define_tests;

define_tests!(crate::getrandom);
