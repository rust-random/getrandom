//! Implementation using getrandom(2).
//!
//! Available since:
//!   - Linux Kernel 3.17, Glibc 2.25, Musl 1.1.20
//!   - Android API level 23 (Marshmallow)
//!   - NetBSD 10.0
//!   - FreeBSD 12.0
//!   - illumos since Dec 2018
//!   - DragonFly 5.7
//!   - Hurd Glibc 2.31
//!   - shim-3ds since Feb 2022
//!
//! For these platforms, we always use the default pool and never set the
//! GRND_RANDOM flag to use the /dev/random pool. On Linux/Android/Hurd, using
//! GRND_RANDOM is not recommended. On NetBSD/FreeBSD/Dragonfly/3ds, it does
//! nothing. On illumos, the default pool is used to implement getentropy(2),
//! so we assume it is acceptable here.
use crate::Backend;
use crate::Error;
use core::mem::MaybeUninit;

#[path = "../util_libc.rs"]
mod util_libc;

pub struct GetrandomBackend;

unsafe impl Backend for GetrandomBackend {
    #[inline]
    unsafe fn fill_ptr(dest: *mut u8, len: usize) -> Result<(), Error> {
        let slice = core::slice::from_raw_parts_mut(dest.cast(), len);
        Self::fill_uninit(slice)
    }

    #[inline]
    fn fill_uninit(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
        util_libc::sys_fill_exact(dest, |buf| unsafe {
            libc::getrandom(buf.as_mut_ptr().cast(), buf.len(), 0)
        })
    }
}
