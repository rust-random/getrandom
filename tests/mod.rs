use std::mem::MaybeUninit;

use getrandom::{getrandom, getrandom_uninit};

#[cfg(getrandom_browser_test)]
use wasm_bindgen_test::wasm_bindgen_test as test;
#[cfg(getrandom_browser_test)]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[test]
fn test_zero() {
    // Test that APIs are happy with zero-length requests
    getrandom(&mut [0u8; 0]).unwrap();
    let res = getrandom_uninit(&mut []).unwrap();
    assert!(res.is_empty());
}

// Return the number of bits in which s1 and s2 differ
fn num_diff_bits(s1: &[u8], s2: &[u8]) -> usize {
    assert_eq!(s1.len(), s2.len());
    s1.iter()
        .zip(s2.iter())
        .map(|(a, b)| (a ^ b).count_ones() as usize)
        .sum()
}

fn uninit_array<const N: usize>() -> [MaybeUninit<u8>; N] {
    [const { MaybeUninit::uninit() }; N]
}

// Tests the quality of calling getrandom on two large buffers
#[test]
fn test_diff() {
    const N: usize = 1000;
    let mut v1 = [0u8; N];
    let mut v2 = [0u8; N];
    getrandom(&mut v1).unwrap();
    getrandom(&mut v2).unwrap();

    let mut t1 = uninit_array::<N>();
    let mut t2 = uninit_array::<N>();
    let r1 = getrandom_uninit(&mut t1).unwrap();
    let r2 = getrandom_uninit(&mut t2).unwrap();
    assert_eq!(r1.len(), N);
    assert_eq!(r2.len(), N);

    // Between 3.5 and 4.5 bits per byte should differ. Probability of failure:
    // ~ 2^(-94) = 2 * CDF[BinomialDistribution[8000, 0.5], 3500]
    let d1 = num_diff_bits(&v1, &v2);
    assert!(d1 > 3500);
    assert!(d1 < 4500);
    let d2 = num_diff_bits(r1, r2);
    assert!(d2 > 3500);
    assert!(d2 < 4500);
}

// Tests the quality of calling getrandom repeatedly on small buffers
#[test]
fn test_small() {
    let mut buf1 = [0u8; 64];
    let mut buf2 = [0u8; 64];
    // For each buffer size, get at least 256 bytes and check that between
    // 3 and 5 bits per byte differ. Probability of failure:
    // ~ 2^(-91) = 64 * 2 * CDF[BinomialDistribution[8*256, 0.5], 3*256]
    for size in 1..=64 {
        let mut num_bytes = 0;
        let mut diff_bits = 0;
        while num_bytes < 256 {
            let s1 = &mut buf1[..size];
            let s2 = &mut buf2[..size];

            getrandom(s1).unwrap();
            getrandom(s2).unwrap();

            num_bytes += size;
            diff_bits += num_diff_bits(s1, s2);
        }
        assert!(diff_bits > 3 * num_bytes);
        assert!(diff_bits < 5 * num_bytes);
    }
}

#[test]
fn test_huge() {
    let mut huge = [0u8; 100_000];
    getrandom(&mut huge).unwrap();
}

#[test]
#[cfg_attr(
    target_arch = "wasm32",
    ignore = "The thread API always fails/panics on WASM"
)]
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
                getrandom(&mut v).unwrap();
                thread::yield_now();
            }
        });
    }

    // start all the tasks
    for tx in txs.iter() {
        tx.send(()).unwrap();
    }
}

#[cfg(getrandom_backend = "custom")]
mod custom {
    struct Xoshiro128PlusPlus {
        s: [u32; 4],
    }

    impl Xoshiro128PlusPlus {
        fn new(mut seed: u64) -> Self {
            const PHI: u64 = 0x9e3779b97f4a7c15;
            let mut s = [0u32; 4];
            for val in s.iter_mut() {
                seed = seed.wrapping_add(PHI);
                let mut z = seed;
                z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
                z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
                z = z ^ (z >> 31);
                *val = z as u32;
            }
            Self { s }
        }

        fn next_u32(&mut self) -> u32 {
            let res = self.s[0]
                .wrapping_add(self.s[3])
                .rotate_left(7)
                .wrapping_add(self.s[0]);

            let t = self.s[1] << 9;

            self.s[2] ^= self.s[0];
            self.s[3] ^= self.s[1];
            self.s[1] ^= self.s[2];
            self.s[0] ^= self.s[3];

            self.s[2] ^= t;

            self.s[3] = self.s[3].rotate_left(11);

            res
        }
    }

    // This implementation uses current timestamp as a PRNG seed.
    //
    // WARNING: this custom implementation is for testing purposes ONLY!
    #[no_mangle]
    unsafe fn __getrandom_custom(dest: *mut u8, len: usize) -> u32 {
        use std::time::{SystemTime, UNIX_EPOCH};

        assert_ne!(len, 0);

        if len == 142 {
            return getrandom::Error::CUSTOM_START + 142;
        }

        let dest_u32 = dest.cast::<u32>();
        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let mut rng = Xoshiro128PlusPlus::new(ts.as_nanos() as u64);
        for i in 0..len / 4 {
            let val = rng.next_u32();
            core::ptr::write_unaligned(dest_u32.add(i), val);
        }
        if len % 4 != 0 {
            let start = 4 * (len / 4);
            for i in start..len {
                let val = rng.next_u32();
                core::ptr::write_unaligned(dest.add(i), val as u8);
            }
        }
        0
    }

    // Test that enabling the custom feature indeed uses the custom implementation
    #[test]
    fn test_custom() {
        let mut buf = [0u8; 142];
        let res = getrandom::getrandom(&mut buf);
        assert!(res.is_err());
    }
}
