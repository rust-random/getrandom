//! Deterministic testing backend — seeded ChaCha12 RNG, single-thread only.

// This module is only compiled under `cfg(test)`, so `std` is always linked
// even though the crate is `#![no_std]`.
extern crate std;

pub use crate::util::{inner_u32, inner_u64};

use crate::Error;

use chacha20::ChaCha12Rng;
use core::mem::MaybeUninit;
use rand_core::{Rng, SeedableRng};
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

/// The RNG, initialised exactly once on first use.
static RNG: OnceLock<Mutex<HashMap<std::thread::ThreadId, ChaCha12Rng>>> = OnceLock::new();

#[inline]
pub fn fill_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    // get current thread id
    let id = std::thread::current().id();

    let rng = RNG.get_or_init(|| HashMap::new().into());

    let mut guard = rng.lock().unwrap();

    let entry = guard
        .entry(id)
        .or_insert_with(|| ChaCha12Rng::from_seed([42u8; 32]));

    // SAFETY: `fill_bytes` fully overwrites every byte of the slice, so
    // treating uninitialized `MaybeUninit<u8>` as `u8` for the purpose of
    // writing (never reading) is sound.
    let dest_init = unsafe { dest.assume_init_mut() };
    entry.fill_bytes(dest_init);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic() {
        let mut buf = [0u8; 32];
        crate::fill(&mut buf).unwrap();
        assert_eq!(
            [
                0x1b, 0x8c, 0x20, 0xcd, 0xe2, 0xdb, 0xb4, 0x3c, 0xd3, 0xc7, 0x9, 0xb2, 0x90, 0xac,
                0x50, 0xdc, 0xd2, 0xbe, 0x2a, 0x87, 0xa3, 0xa2, 0x45, 0x44, 0xb5, 0xa5, 0x10, 0x9b,
                0xc7, 0x6e, 0xa7, 0xfb,
            ],
            buf
        );
    }
    use std::thread;
    #[test]
    fn test_deterministic_multithread() {
        thread::spawn(|| {
            let mut buf = [0u8; 32];
            crate::fill(&mut buf).unwrap();
            assert_eq!(
                [
                    0x1b, 0x8c, 0x20, 0xcd, 0xe2, 0xdb, 0xb4, 0x3c, 0xd3, 0xc7, 0x9, 0xb2, 0x90,
                    0xac, 0x50, 0xdc, 0xd2, 0xbe, 0x2a, 0x87, 0xa3, 0xa2, 0x45, 0x44, 0xb5, 0xa5,
                    0x10, 0x9b, 0xc7, 0x6e, 0xa7, 0xfb,
                ],
                buf
            );
        });
        thread::spawn(|| {
            let mut buf = [0u8; 32];
            crate::fill(&mut buf).unwrap();
            assert_eq!(
                [
                    0x1b, 0x8c, 0x20, 0xcd, 0xe2, 0xdb, 0xb4, 0x3c, 0xd3, 0xc7, 0x9, 0xb2, 0x90,
                    0xac, 0x50, 0xdc, 0xd2, 0xbe, 0x2a, 0x87, 0xa3, 0xa2, 0x45, 0x44, 0xb5, 0xa5,
                    0x10, 0x9b, 0xc7, 0x6e, 0xa7, 0xfb,
                ],
                buf
            );
        });
    }
}
