#![feature(test)]
extern crate test;

use std::mem::MaybeUninit;

// Used to benchmark the throughput of getrandom in an optimal scenario.
// The buffer is hot, and does not require initialization.
#[inline(always)]
fn bench_getrandom<const N: usize>(b: &mut test::Bencher) {
    b.iter(|| {
        let mut buf = [0u8; N];
        getrandom::getrandom(&mut buf[..]).unwrap();
        test::black_box(&buf);
    });
}

// Used to benchmark the throughput of getrandom is a slightly less optimal
// scenario. The buffer is still hot, but requires initialization.
#[inline(always)]
fn bench_getrandom_uninit<const N: usize>(b: &mut test::Bencher) {
    b.iter(|| {
        // TODO: When the feature `maybe_uninit_as_bytes` is available, use:
        // ```
        // let mut buf: MaybeUninit<[u8; N]> = MaybeUninit::uninit();
        // getrandom::getrandom_uninit(buf.as_bytes_mut()).unwrap();
        // test::black_box(unsafe { buf.assume_init() })
        // ```
        // since that is the shape we expect most callers to have.
        let mut buf = [MaybeUninit::new(0u8); N];
        let buf = getrandom::getrandom_uninit(&mut buf[..]).unwrap();
        test::black_box(&buf);
    });
}

macro_rules! bench {
    ( $name:ident, $size:expr ) => {
        pub mod $name {
            #[bench]
            pub fn bench_getrandom(b: &mut test::Bencher) {
                super::bench_getrandom::<{ $size }>(b);
            }

            #[bench]
            pub fn bench_getrandom_uninit(b: &mut test::Bencher) {
                super::bench_getrandom_uninit::<{ $size }>(b);
            }
        }
    };
}

// 32 bytes (256 bits) is the seed sized used for rand::thread_rng
// and the `random` value in a ClientHello/ServerHello for TLS.
// This is also the size of a 256-bit AES/HMAC/P-256/Curve25519 key
// and/or nonce.
bench!(p256, 256 / 8);

// A P-384/HMAC-384 key and/or nonce.
bench!(p384, 384 / 8);
