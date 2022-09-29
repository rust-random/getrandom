#![feature(test)]
extern crate test;

#[inline(always)]
fn bench<const N: usize>(b: &mut test::Bencher) {
    b.iter(|| {
        let mut buf = [0u8; N];
        getrandom::getrandom(&mut buf[..]).unwrap();
        test::black_box(&buf);
    });
    b.bytes = N as u64;
}

#[inline(always)]
fn bench_raw<const N: usize>(b: &mut test::Bencher) {
    b.iter(|| {
        let mut buf = core::mem::MaybeUninit::<[u8; N]>::uninit();
        unsafe { getrandom::getrandom_raw(buf.as_mut_ptr().cast(), N).unwrap() };
        test::black_box(&buf);
    });
    b.bytes = N as u64;
}

// 32 bytes (256-bit) is the seed sized used for rand::thread_rng
const SEED: usize = 32;
// Common size of a page, 4 KiB
const PAGE: usize = 4096;
// Large buffer to get asymptotic performance, 2 MiB
const LARGE: usize = 1 << 21;

#[bench]
fn bench_seed(b: &mut test::Bencher) {
    bench::<SEED>(b);
}
#[bench]
fn bench_seed_raw(b: &mut test::Bencher) {
    bench_raw::<SEED>(b);
}

#[bench]
fn bench_page(b: &mut test::Bencher) {
    bench::<PAGE>(b);
}
#[bench]
fn bench_page_raw(b: &mut test::Bencher) {
    bench_raw::<PAGE>(b);
}

#[bench]
fn bench_large(b: &mut test::Bencher) {
    bench::<LARGE>(b);
}
#[bench]
fn bench_large_raw(b: &mut test::Bencher) {
    bench_raw::<LARGE>(b);
}
