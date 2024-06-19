use super::{getrandom_impl, getrandom_uninit_impl};
use core::mem::MaybeUninit;
#[cfg(not(feature = "custom"))]
use getrandom::Error;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use wasm_bindgen_test::wasm_bindgen_test as test;

#[cfg(feature = "test-in-browser")]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[cfg(not(feature = "custom"))]
fn wrapped_getrandom(dest: &mut [u8]) -> Result<&mut [u8], Error> {
    getrandom_impl(dest).map(|()| dest)
}

// Test that APIs are happy with zero-length requests
#[test]
fn test_zero() {
    getrandom_impl(&mut []).unwrap();
}
#[test]
fn test_zero_uninit() {
    getrandom_uninit_impl(&mut []).unwrap();
}

// Return the number of bits in which s1 and s2 differ
#[cfg(not(feature = "custom"))]
fn num_diff_bits(s1: &[u8], s2: &[u8]) -> usize {
    assert_eq!(s1.len(), s2.len());
    s1.iter()
        .zip(s2.iter())
        .map(|(a, b)| (a ^ b).count_ones() as usize)
        .sum()
}

// Tests the quality of calling getrandom on two large buffers

#[cfg(not(feature = "custom"))]
fn test_diff_large<T: Copy>(initial: T, f: impl Fn(&mut [T]) -> Result<&mut [u8], Error>) {
    let mut v1 = [initial; 1000];
    let r1 = f(&mut v1).unwrap();

    let mut v2 = [initial; 1000];
    let r2 = f(&mut v2).unwrap();

    // Between 3.5 and 4.5 bits per byte should differ. Probability of failure:
    // ~ 2^(-94) = 2 * CDF[BinomialDistribution[8000, 0.5], 3500]
    let d = num_diff_bits(r1, r2);
    assert!(d > 3500);
    assert!(d < 4500);
}

#[cfg(not(feature = "custom"))]
#[test]
fn test_large() {
    test_diff_large(0u8, wrapped_getrandom);
}

#[cfg(not(feature = "custom"))]
#[test]
fn test_large_uninit() {
    test_diff_large(MaybeUninit::uninit(), getrandom_uninit_impl);
}

// Tests the quality of calling getrandom repeatedly on small buffers

#[cfg(not(feature = "custom"))]
fn test_diff_small<T: Copy>(initial: T, f: impl Fn(&mut [T]) -> Result<&mut [u8], Error>) {
    // For each buffer size, get at least 256 bytes and check that between
    // 3 and 5 bits per byte differ. Probability of failure:
    // ~ 2^(-91) = 64 * 2 * CDF[BinomialDistribution[8*256, 0.5], 3*256]
    for size in 1..=64 {
        let mut num_bytes = 0;
        let mut diff_bits = 0;
        while num_bytes < 256 {
            let mut s1 = vec![initial; size];
            let r1 = f(&mut s1).unwrap();
            let mut s2 = vec![initial; size];
            let r2 = f(&mut s2).unwrap();

            num_bytes += size;
            diff_bits += num_diff_bits(r1, r2);
        }
        assert!(diff_bits > 3 * num_bytes);
        assert!(diff_bits < 5 * num_bytes);
    }
}

#[cfg(not(feature = "custom"))]
#[test]
fn test_small() {
    test_diff_small(0u8, wrapped_getrandom);
}

#[cfg(not(feature = "custom"))]
#[test]
fn test_small_unnit() {
    test_diff_small(MaybeUninit::uninit(), getrandom_uninit_impl);
}

#[test]
fn test_huge() {
    let mut huge = [0u8; 100_000];
    getrandom_impl(&mut huge).unwrap();
}

#[test]
fn test_huge_uninit() {
    let mut huge = [MaybeUninit::uninit(); 100_000];
    getrandom_uninit_impl(&mut huge).unwrap();
    check_initialized(&huge);
}

#[allow(unused_variables)]
fn check_initialized(buf: &[MaybeUninit<u8>]) {
    #[cfg(feature = "unstable-sanitize")]
    {
        #[cfg(sanitize = "memory")]
        {
            use core::ffi::c_void;
            extern "C" {
                // void __msan_check_mem_is_initialized(const volatile void *x, size_t size);
                fn __msan_check_mem_is_initialized(x: *const c_void, size: usize);
            }
            unsafe {
                __msan_check_mem_is_initialized(buf.as_ptr().cast::<c_void>(), buf.len());
            }
        }
    }
}

// On WASM, the thread API always fails/panics
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_multithreading() {
    extern crate std;
    use std::{sync::mpsc::channel, thread, vec};

    let mut txs = vec![];
    for _ in 0..20 {
        let (tx, rx) = channel();
        txs.push(tx);

        thread::spawn(move || {
            // wait until all the tasks are ready to go.
            rx.recv().unwrap();
            let mut v = [0u8; 1000];

            for _ in 0..100 {
                getrandom_impl(&mut v).unwrap();
                thread::yield_now();
            }
        });
    }

    // start all the tasks
    for tx in txs.iter() {
        tx.send(()).unwrap();
    }
}
