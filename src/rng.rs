//! rand_core adapter

use crate::Error;
use rand_core::{TryCryptoRng, TryRngCore};

/// An RNG over the operating-system's random data source
///
/// This is a zero-sized struct. It can be freely constructed with just `SysRng`.
///
/// This struct is also available as [`rand::rngs::SysRng`] when using [rand].
///
/// # Usage example
///
/// `SysRng` implements [`TryRngCore`]:
/// ```
/// use getrandom::{rand_core::TryRngCore, SysRng};
///
/// let mut key = [0u8; 32];
/// SysRng.try_fill_bytes(&mut key).unwrap();
/// ```
///
/// Using it as an [`RngCore`] is possible using [`TryRngCore::unwrap_err`]:
/// ```
/// use getrandom::rand_core::{TryRngCore, RngCore};
/// use getrandom::SysRng;
///
/// let mut rng = SysRng.unwrap_err();
/// let random_u64 = rng.next_u64();
/// ```
///
/// [rand]: https://crates.io/crates/rand
/// [`rand::rngs::SysRng`]: https://docs.rs/rand/latest/rand/rngs/struct.SysRng.html
/// [`RngCore`]: rand_core::RngCore
#[derive(Clone, Copy, Debug, Default)]
pub struct SysRng;

impl TryRngCore for SysRng {
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

impl TryCryptoRng for SysRng {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_os_rng() {
        let x = SysRng.try_next_u64().unwrap();
        let y = SysRng.try_next_u64().unwrap();
        assert!(x != 0);
        assert!(x != y);
    }

    #[test]
    fn test_construction() {
        assert!(SysRng.try_next_u64().unwrap() != 0);
    }
}
