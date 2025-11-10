//! rand_core adapter

use crate::Error;
use rand_core::{TryCryptoRng, TryRngCore};

/// An RNG over the operating-system's random data source
///
/// This is a zero-sized struct. It can be freely constructed with just `OsRng`.
///
/// This struct is also available as [`rand::rngs::OsRng`] when using [rand].
///
/// # Usage example
/// ```
/// use getrandom::{rand_core::{TryRngCore, RngCore}, OsRng};
///
/// let mut key = [0u8; 32];
/// OsRng.try_fill_bytes(&mut key).unwrap();
///
/// let mut rng = OsRng.unwrap_err();
/// let random_u64 = rng.next_u64();
/// ```
///
/// [rand]: https://crates.io/crates/rand
/// [`rand::rngs::OsRng`]: https://docs.rs/rand/latest/rand/rngs/struct.OsRng.html
#[derive(Clone, Copy, Debug, Default)]
pub struct OsRng;

impl TryRngCore for OsRng {
    type Error = Error;

    #[inline]
    fn try_next_u32(&mut self) -> Result<u32, Error> {
        crate::u32()
    }

    #[inline]
    fn try_next_u64(&mut self) -> Result<u64, Error> {
        crate::u64()
    }

    #[inline]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        crate::fill(dest)
    }
}

impl TryCryptoRng for OsRng {}

#[test]
fn test_os_rng() {
    let x = OsRng.try_next_u64().unwrap();
    let y = OsRng.try_next_u64().unwrap();
    assert!(x != 0);
    assert!(x != y);
}

#[test]
fn test_construction() {
    assert!(OsRng.try_next_u64().unwrap() != 0);
}
