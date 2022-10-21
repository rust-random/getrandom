// Common infrastructure for the custom* test suites (only).
use core::{
    num::NonZeroU32,
    sync::atomic::{AtomicU8, Ordering},
};
use getrandom::{register_custom_getrandom, Error};

pub fn len7_err() -> Error {
    NonZeroU32::new(Error::INTERNAL_START + 7).unwrap().into()
}

fn super_insecure_rng(buf: &mut [u8]) -> Result<(), Error> {
    // `getrandom` guarantees it will not call any implementation if the output
    // buffer is empty.
    assert!(!buf.is_empty());
    // Length 7 buffers return a custom error
    if buf.len() == 7 {
        return Err(len7_err());
    }
    // Otherwise, increment an atomic counter
    static COUNTER: AtomicU8 = AtomicU8::new(0);
    for b in buf {
        *b = COUNTER.fetch_add(1, Ordering::Relaxed);
    }
    Ok(())
}

register_custom_getrandom!(super_insecure_rng);
