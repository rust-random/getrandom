#![feature(test)]
#![feature(maybe_uninit_as_bytes)]

extern crate test;

use std::mem::MaybeUninit;

// Used to benchmark the throughput of getrandom in an optimal scenario.
// The buffer is hot, and does not require initialization.
#[inline(always)]
fn bench_getrandom<const N: usize>(b: &mut test::Bencher) {
    b.bytes = N as u64;
    b.iter(|| {
        let mut buf = [0u8; N];
        getrandom::getrandom(&mut buf[..]).unwrap();
        test::black_box(buf);
    });
}

// Used to benchmark the throughput of getrandom is a slightly less optimal
// scenario. The buffer is still hot, but requires initialization.
#[inline(always)]
fn bench_getrandom_uninit<const N: usize>(b: &mut test::Bencher) {
    b.bytes = N as u64;
    b.iter(|| {
        let mut buf: MaybeUninit<[u8; N]> = MaybeUninit::uninit();
        let _ = getrandom::getrandom_uninit(buf.as_bytes_mut()).unwrap();
        let buf: [u8; N] = unsafe { buf.assume_init() };
        test::black_box(buf)
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

// 16 bytes (128 bits) is the size of an 128-bit AES key/nonce.
bench!(aes128, 128 / 8);

// 32 bytes (256 bits) is the seed sized used for rand::thread_rng
// and the `random` value in a ClientHello/ServerHello for TLS.
// This is also the size of a 256-bit AES/HMAC/P-256/Curve25519 key
// and/or nonce.
bench!(p256, 256 / 8);

// A P-384/HMAC-384 key and/or nonce.
bench!(p384, 384 / 8);

// Initializing larger buffers is not the primary use case of this library, as
// this should normally be done by a userspace CSPRNG. However, we have a test
// here to see the effects of a lower (amortized) syscall overhead.
bench!(page, 4096);
