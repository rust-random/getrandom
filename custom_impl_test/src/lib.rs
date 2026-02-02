use core::mem::MaybeUninit;
use getrandom::Error;

/// Chosen by fair dice roll.
const SEED: u64 = 0x9095_810F_1B2B_E175;

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

pub fn custom_impl(dst: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    let mut rng = Xoshiro128PlusPlus::new(SEED);

    let mut chunks = dst.chunks_exact_mut(4);
    for chunk in &mut chunks {
        let val = rng.next_u32();
        let dst_ptr = chunk.as_mut_ptr().cast::<u32>();
        unsafe { core::ptr::write_unaligned(dst_ptr, val) };
    }
    let rem = chunks.into_remainder();
    if !rem.is_empty() {
        let val = rng.next_u32();
        let src_ptr = &val as *const u32 as *const MaybeUninit<u8>;
        assert!(rem.len() <= 4);
        unsafe { core::ptr::copy(src_ptr, rem.as_mut_ptr(), rem.len()) };
    }
    Ok(())
}

#[cfg(getrandom_backend = "custom")]
#[unsafe(no_mangle)]
unsafe extern "Rust" fn __getrandom_v03_custom(dst_ptr: *mut u8, len: usize) -> Result<(), Error> {
    let dst = unsafe { core::slice::from_raw_parts_mut(dst_ptr.cast(), len) };
    custom_impl(dst)
}

#[cfg(getrandom_backend = "extern_impl")]
#[getrandom::implementation::fill_uninit]
fn my_fill_uninit_implementation(dst: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    custom_impl(dst)
}

#[test]
fn test_custom_fill() {
    let mut buf1 = [0u8; 256];
    getrandom::fill(&mut buf1).unwrap();

    let mut buf2 = [0u8; 256];
    custom_impl(unsafe { core::slice::from_raw_parts_mut(buf2.as_mut_ptr().cast(), buf2.len()) })
        .unwrap();

    assert_eq!(buf1, buf2);
}

#[test]
fn test_custom_u32() {
    let res = getrandom::u32().unwrap();
    assert_eq!(res, 0xEAD5_840A);
}

#[test]
fn test_custom_u64() {
    let res = getrandom::u64().unwrap();
    assert_eq!(res, 0xA856_FCC4_EAD5_840A);
}
