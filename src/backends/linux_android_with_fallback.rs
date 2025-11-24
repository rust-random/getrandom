//! Implementation for Linux / Android with `/dev/urandom` fallback
use super::{sanitizer, use_file};
use crate::{Error, lazy, util_libc};
use core::{
    ffi::c_void,
    mem::{MaybeUninit, transmute},
    ptr,
};

pub use crate::util::{inner_u32, inner_u64};

type GetRandomFn = unsafe extern "C" fn(*mut c_void, libc::size_t, libc::c_uint) -> libc::ssize_t;

/// Sentinel value which indicates that `libc::getrandom` either not available,
/// or not supported by kernel.
const NOT_AVAILABLE: usize = usize::MAX;

#[cold]
#[inline(never)]
fn init() -> usize {
    // Use static linking to `libc::getrandom` on MUSL targets and `dlsym` everywhere else
    #[cfg(not(target_env = "musl"))]
    let fptr = unsafe { libc::dlsym(libc::RTLD_DEFAULT, c"getrandom".as_ptr()) } as usize;
    #[cfg(target_env = "musl")]
    let fptr = {
        let fptr: GetRandomFn = libc::getrandom;
        unsafe { transmute::<GetRandomFn, usize>(fptr) }
    };

    let res_ptr = if fptr != 0 {
        let getrandom_fn = unsafe { transmute::<usize, GetRandomFn>(fptr) };
        // Check that `getrandom` syscall is supported by kernel
        let res = unsafe { getrandom_fn(ptr::dangling_mut(), 0, 0) };
        if cfg!(getrandom_test_linux_fallback) {
            NOT_AVAILABLE
        } else if res.is_negative() {
            match util_libc::last_os_error().raw_os_error() {
                Some(libc::ENOSYS) => NOT_AVAILABLE, // No kernel support
                // The fallback on EPERM is intentionally not done on Android since this workaround
                // seems to be needed only for specific Linux-based products that aren't based
                // on Android. See https://github.com/rust-random/getrandom/issues/229.
                #[cfg(target_os = "linux")]
                Some(libc::EPERM) => NOT_AVAILABLE, // Blocked by seccomp
                _ => fptr,
            }
        } else {
            fptr
        }
    } else {
        NOT_AVAILABLE
    };

    #[cfg(getrandom_test_linux_without_fallback)]
    if res_ptr == NOT_AVAILABLE {
        panic!("Fallback is triggered with enabled `getrandom_test_linux_without_fallback`")
    }

    res_ptr
}

// Prevent inlining of the fallback implementation
#[inline(never)]
fn use_file_fallback(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    use_file::fill_inner(dest)
}

#[inline]
pub fn fill_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    static GETRANDOM_FN: lazy::LazyUsize = lazy::LazyUsize::new();
    let fptr = GETRANDOM_FN.unsync_init(init);

    if fptr == NOT_AVAILABLE {
        use_file_fallback(dest)
    } else {
        // note: `transmute` is currently the only way to convert a pointer into a function reference
        let getrandom_fn = unsafe { transmute::<usize, GetRandomFn>(fptr) };
        util_libc::sys_fill_exact(dest, |buf| unsafe {
            let ret = getrandom_fn(buf.as_mut_ptr().cast(), buf.len(), 0);
            sanitizer::unpoison_linux_getrandom_result(buf, ret);
            ret
        })
    }
}
