//! Implementation for Linux / Android with `/dev/urandom` fallback
use crate::{use_file, util_libc, Error};
use core::{
    ffi::c_void,
    mem,
    mem::MaybeUninit,
    ptr,
    sync::atomic::{AtomicPtr, Ordering},
};

type GetRandomFn = unsafe extern "C" fn(*mut c_void, libc::size_t, libc::c_uint) -> libc::ssize_t;

/// Sentinel value which indicates that `libc::getrandom` either not available,
/// or not supported by kernel.
const NOT_AVAILABLE: *mut c_void = usize::MAX as *mut c_void;

static GETRANDOM_FN: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());

#[cold]
fn init() -> *mut c_void {
    static NAME: &[u8] = b"getrandom\0";
    let name_ptr = NAME.as_ptr().cast::<libc::c_char>();
    let raw_ptr = unsafe { libc::dlsym(libc::RTLD_DEFAULT, name_ptr) };
    let res_ptr = if raw_ptr.is_null() {
        NOT_AVAILABLE
    } else {
        let fptr = unsafe { mem::transmute::<*mut c_void, GetRandomFn>(raw_ptr) };
        // Check that `getrandom` syscall is supported by kernel
        let res = unsafe { fptr(ptr::NonNull::dangling().as_ptr(), 0, 0) };
        if res < 0 {
            match util_libc::last_os_error().raw_os_error() {
                Some(libc::ENOSYS) => NOT_AVAILABLE, // No kernel support
                // The fallback on EPERM is intentionally not done on Android since this workaround
                // seems to be needed only for specific Linux-based products that aren't based
                // on Android. See https://github.com/rust-random/getrandom/issues/229.
                #[cfg(target_os = "linux")]
                Some(libc::EPERM) => NOT_AVAILABLE, // Blocked by seccomp
                _ => raw_ptr,
            }
        } else {
            raw_ptr
        }
    };

    GETRANDOM_FN.store(res_ptr, Ordering::Release);
    res_ptr
}

pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    // Despite being only a single atomic variable, we still cannot always use
    // Ordering::Relaxed, as we need to make sure a successful call to `init`
    // is "ordered before" any data read through the returned pointer (which
    // occurs when the function is called). Our implementation mirrors that of
    // the one in libstd, meaning that the use of non-Relaxed operations is
    // probably unnecessary.
    let mut raw_ptr = GETRANDOM_FN.load(Ordering::Acquire);
    if raw_ptr.is_null() {
        raw_ptr = init();
    }

    if raw_ptr == NOT_AVAILABLE {
        // prevent inlining of the fallback implementation
        #[inline(never)]
        fn inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
            use_file::getrandom_inner(dest)
        }

        inner(dest)
    } else {
        // note: `transume` is currently the only way to get function pointer
        let fptr = unsafe { mem::transmute::<*mut c_void, GetRandomFn>(raw_ptr) };
        util_libc::sys_fill_exact(dest, |buf| unsafe {
            fptr(buf.as_mut_ptr().cast(), buf.len(), 0)
        })
    }
}
