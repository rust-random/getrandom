//! Implementation for WASI Preview 2.
use crate::Backend;
use crate::Error;
use core::mem::MaybeUninit;
use wasi::random::random::get_random_u64;

pub struct WasiP2Backend;

unsafe impl Backend for WasiP2Backend {
    #[inline]
    unsafe fn fill_ptr(dest: *mut u8, len: usize) -> Result<(), Error> {
        let slice = core::slice::from_raw_parts_mut(dest as *mut MaybeUninit<u8>, len);
        Self::fill_uninit(slice)
    }

    #[inline]
    fn fill_uninit(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
        use core::ptr::copy_nonoverlapping;

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

    #[inline]
    fn u32() -> Result<u32, Error> {
        let val = get_random_u64();
        Ok(crate::util::truncate(val))
    }

    #[inline]
    fn u64() -> Result<u64, Error> {
        Ok(get_random_u64())
    }
}
