use crate::Error;

extern crate std;
use std::{mem::MaybeUninit, sync::mpsc::channel, thread, vec, vec::Vec};

#[cfg(feature = "test-in-browser")]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

// Return the number of bits in which s1 and s2 differ
fn num_diff_bits(s1: &[u8], s2: &[u8]) -> usize {
    assert_eq!(s1.len(), s2.len());
    s1.iter()
        .zip(s2.iter())
        .map(|(a, b)| (a ^ b).count_ones() as usize)
        .sum()
}

// Tests the quality of calling getrandom on two large buffers
pub(crate) fn check_diff_large(make_vec: fn(usize) -> Vec<u8>) {
    let v1 = make_vec(1000);
    let v2 = make_vec(1000);

    // Between 3.5 and 4.5 bits per byte should differ. Probability of failure:
    // ~ 2^(-94) = 2 * CDF[BinomialDistribution[8000, 0.5], 3500]
    let d = num_diff_bits(&v1, &v2);
    assert!(d > 3500);
    assert!(d < 4500);
}

// Tests the quality of calling getrandom repeatedly on small buffers
pub(crate) fn check_small(make_vec: fn(usize) -> Vec<u8>) {
    // For each buffer size, get at least 256 bytes and check that between
    // 3 and 5 bits per byte differ. Probability of failure:
    // ~ 2^(-91) = 64 * 2 * CDF[BinomialDistribution[8*256, 0.5], 3*256]
    for size in 1..=64 {
        let mut num_bytes = 0;
        let mut diff_bits = 0;
        while num_bytes < 256 {
            let s1 = make_vec(size);
            let s2 = make_vec(size);
            num_bytes += size;
            diff_bits += num_diff_bits(&s1, &s2);
        }
        assert!(diff_bits > 3 * num_bytes);
        assert!(diff_bits < 5 * num_bytes);
    }
}

pub(crate) fn check_multithreading(make_vec: fn(usize) -> Vec<u8>) {
    let mut txs = vec![];
    for _ in 0..20 {
        let (tx, rx) = channel();
        txs.push(tx);

        thread::spawn(move || {
            // wait until all the tasks are ready to go.
            rx.recv().unwrap();
            for _ in 0..100 {
                make_vec(1000);
                thread::yield_now();
            }
        });
    }

    // start all the tasks
    for tx in txs.iter() {
        tx.send(()).unwrap();
    }
}

// Helper trait for testing different kinds of functions.
// DUMMY generic parameter is needed to avoid conflicting implementations.
pub(crate) trait FillFn<const DUMMY: usize> {
    fn make_vec(self, len: usize) -> Vec<u8>;
}
impl<F: Fn(&mut [u8]) -> Result<(), Error>> FillFn<0> for F {
    fn make_vec(self, len: usize) -> Vec<u8> {
        let mut v = vec![0; len];
        self(&mut v).unwrap();
        v
    }
}
impl<F: Fn(&mut [MaybeUninit<u8>]) -> Result<(), Error>> FillFn<1> for F {
    fn make_vec(self, len: usize) -> Vec<u8> {
        let mut v = Vec::with_capacity(len);
        self(v.spare_capacity_mut()).unwrap();
        unsafe { v.set_len(len) };
        v
    }
}
impl<F: Fn(&mut [MaybeUninit<u8>]) -> Result<&mut [u8], Error>> FillFn<2> for F {
    fn make_vec(self, len: usize) -> Vec<u8> {
        let mut v = Vec::with_capacity(len);
        let ret = self(v.spare_capacity_mut()).unwrap();
        assert_eq!(ret.len(), len);
        assert_eq!(ret.as_ptr(), v.as_ptr());
        unsafe { v.set_len(len) };
        v
    }
}

macro_rules! define_tests {
    ($fill:path) => {
        use crate::tests::FillFn;
        #[cfg(all(target_family = "wasm", target_os = "unknown"))]
        use wasm_bindgen_test::wasm_bindgen_test as test;

        #[test]
        fn zero() {
            $fill.make_vec(0);
        }
        #[test]
        fn diff_large() {
            crate::tests::check_diff_large(|len| $fill.make_vec(len));
        }
        #[test]
        fn small() {
            crate::tests::check_small(|len| $fill.make_vec(len));
        }
        #[test]
        fn huge() {
            $fill.make_vec(100_000);
        }
        // On WASM, the thread API always fails/panics.
        #[test]
        #[cfg_attr(target_family = "wasm", ignore)]
        fn multithreading() {
            crate::tests::check_multithreading(|len| $fill.make_vec(len));
        }
    };
}
pub(crate) use define_tests;

mod init {
    super::define_tests!(crate::getrandom);
}
mod uninit {
    super::define_tests!(crate::getrandom_uninit);
}
