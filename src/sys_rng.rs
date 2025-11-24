use crate::Error;
use rand_core::{CryptoRng, RngCore, TryCryptoRng, TryRngCore};

/// A [`TryRngCore`] interface over the system's preferred random number source.
///
/// This is a zero-sized struct. It can be freely constructed with just `SysRng`.
///
/// This struct is also available as [`rand::rngs::SysRng`] when using [`rand`].
///
/// If you don't care about potential (but extremely unlikely in practice) errors,
/// you can use [`UnwrappingSysRng`] instead.
///
/// # Usage example
///
/// `SysRng` implements [`TryRngCore`]:
/// ```
/// use getrandom::{rand_core::TryRngCore, SysRng};
///
/// # fn main() -> Result<(), getrandom::Error> {
/// let mut key = [0u8; 32];
/// SysRng.try_fill_bytes(&mut key)?;
///
/// let x: u32 = SysRng.try_next_u32()?;
/// let y: u64 = SysRng.try_next_u64()?;
/// # Ok(()) }
/// ```
///
/// [`rand`]: https://crates.io/crates/rand
/// [`rand::rngs::SysRng`]: https://docs.rs/rand/latest/rand/rngs/struct.SysRng.html
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

/// A potentially-panicking [`RngCore`] interface over the system's preferred random number source.
///
/// This is a zero-sized struct. It can be freely constructed with just `UnwrappingSysRng`.
///
/// If possible, we recommend to use [`SysRng`] instead and to properly handle potential errors.
///
/// This struct is also available as [`rand::rngs::UnwrappingSysRng`] when using [`rand`].
///
/// # Usage example
///
/// `UnwrappingSysRng` implements [`RngCore`]:
/// ```
/// use getrandom::{rand_core::RngCore, UnwrappingSysRng};
///
/// let mut key = [0u8; 32];
/// UnwrappingSysRng.fill_bytes(&mut key);
///
/// let x: u32 = UnwrappingSysRng.next_u32();
/// let y: u64 = UnwrappingSysRng.next_u64();
/// ```
///
/// [`rand`]: https://crates.io/crates/rand
/// [`rand::rngs::UnwrappingSysRng`]: https://docs.rs/rand/latest/rand/rngs/struct.UnwrappingSysRng.html
/// [`RngCore`]: rand_core::RngCore
#[derive(Clone, Copy, Debug, Default)]
pub struct UnwrappingSysRng;

impl RngCore for UnwrappingSysRng {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        crate::u32().unwrap()
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        crate::u64().unwrap()
    }

    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        crate::fill(dest).unwrap()
    }
}

impl CryptoRng for UnwrappingSysRng {}
