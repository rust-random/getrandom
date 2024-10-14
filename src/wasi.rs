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
    // This linking is vendored from the wasi crate:
    // https://docs.rs/wasi/0.11.0+wasi-snapshot-preview1/src/wasi/lib_generated.rs.html#2344-2350
    #[link(wasm_import_module = "wasi_snapshot_preview1")]
    extern "C" {
        fn random_get(arg0: i32, arg1: i32) -> i32;
    }

    // Based on the wasi code:
    // https://docs.rs/wasi/0.11.0+wasi-snapshot-preview1/src/wasi/lib_generated.rs.html#2046-2062
    // Note that size of an allocated object can not be bigger than isize::MAX bytes.
    // WASI 0.1 supports only 32-bit WASM, so casting length to `i32` is safe.
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let ret = unsafe { random_get(dest.as_mut_ptr() as i32, dest.len() as i32) };
    match ret {
        0 => Ok(()),
        code => {
            let err = u32::try_from(code)
                .map(Error::from_os_error)
                .unwrap_or(Error::UNEXPECTED);
            Err(err)
        }
    }
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
        if val == 42 {
            panic!();
        }
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
