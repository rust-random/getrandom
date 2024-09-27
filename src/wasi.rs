//! Implementation for WASI (preview 1 and 2)
//!
//! `target_env = "p1"` was introduced only in Rust 1.80, so on earlier compiler versions this
//! code will result in a compilation error.
use crate::Error;
use core::mem::MaybeUninit;

#[cfg(not(any(target_env = "p1", target_env = "p2")))]
compile_error!(
    "Unknown version of WASI (only previews 1 and 2 are supported) \
    or Rust version older than 1.80 was used"
);

#[cfg(target_env = "p1")]
pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    unsafe { wasi::random_get(dest.as_mut_ptr().cast::<u8>(), dest.len()) }
        .map_err(|e| Error::from_os_error(e.raw().into()))
}

#[cfg(target_env = "p2")]
pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    use core::ptr::copy_nonoverlapping;
    use wasi::random::random::get_random_u64;

    let (prefix, chunks, suffix) = unsafe { dest.align_to_mut::<MaybeUninit<u64>>() };

    // We use `get_random_u64` instead of `get_random_bytes` because the latter creates
    // an allocation due to the Wit IDL [restrictions][0]. This should be fine since
    // the main use case of `getrandom` is seed generation.
    //
    // [0]: https://github.com/WebAssembly/wasi-random/issues/27
    if !prefix.is_empty() {
        let val = get_random_u64();
        let src = (&val as *const u64).cast();
        unsafe {
            copy_nonoverlapping(src, prefix.as_mut_ptr(), prefix.len());
        }
    }

    for dst in chunks {
        dst.write(get_random_u64());
    }

    if !suffix.is_empty() {
        let val = get_random_u64();
        let src = (&val as *const u64).cast();
        unsafe {
            copy_nonoverlapping(src, suffix.as_mut_ptr(), suffix.len());
        }
    }

    Ok(())
}
