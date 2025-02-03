//! Implementation for Linux / Android with `/dev/urandom` fallback
use super::use_file;
use crate::Error;
use core::mem::MaybeUninit;

pub use crate::util::{inner_u32, inner_u64};

#[cfg(not(has_libc_getrandom))]
#[inline]
pub fn fill_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    use_file::fill_inner(dest)
}

#[cfg(has_libc_getrandom)]
#[inline]
pub fn fill_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    use use_file::util_libc;

    #[path = "../lazy.rs"]
    mod lazy;

    static GETRANDOM_GOOD: lazy::LazyBool = lazy::LazyBool::new();

    #[cold]
    #[inline(never)]
    fn is_getrandom_good() -> bool {
        let dangling_ptr = core::ptr::NonNull::dangling().as_ptr();
        // Check that `getrandom` syscall is supported by kernel
        let res = unsafe { libc::getrandom(dangling_ptr, 0, 0) };
        if cfg!(getrandom_test_linux_fallback) {
            false
        } else if res.is_negative() {
            match util_libc::last_os_error().raw_os_error() {
                Some(libc::ENOSYS) => false, // No kernel support
                // The fallback on EPERM is intentionally not done on Android since this workaround
                // seems to be needed only for specific Linux-based products that aren't based
                // on Android. See https://github.com/rust-random/getrandom/issues/229.
                #[cfg(target_os = "linux")]
                Some(libc::EPERM) => false, // Blocked by seccomp
                _ => true,
            }
        } else {
            true
        }
    }

    #[inline(never)]
    fn use_file_fallback(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
        use_file::fill_inner(dest)
    }

    if !GETRANDOM_GOOD.unsync_init(is_getrandom_good) {
        use_file_fallback(dest)
    } else {
        util_libc::sys_fill_exact(dest, |buf| unsafe {
            libc::getrandom(buf.as_mut_ptr().cast(), buf.len(), 0)
        })
    }
}
