//! Implementation for NetBSD
//!
//! `getrandom(2)` was introduced in NetBSD 10. To support older versions we
//! implement our own weak linkage to it, and provide a fallback based on the
//! KERN_ARND sysctl.
use crate::{util_libc::sys_fill_exact, Error};
use core::{
    cmp,
    ffi::c_void,
    mem::{self, MaybeUninit},
    ptr,
    sync::atomic::{AtomicPtr, Ordering},
};

unsafe extern "C" fn polyfill_using_kern_arand(
    buf: *mut c_void,
    buflen: libc::size_t,
    flags: libc::c_uint,
) -> libc::ssize_t {
    debug_assert_eq!(flags, 0);

    static MIB: [libc::c_int; 2] = [libc::CTL_KERN, libc::KERN_ARND];

    // NetBSD will only return up to 256 bytes at a time, and
    // older NetBSD kernels will fail on longer buffers.
    let mut len = cmp::min(buflen, 256);

    let ret = unsafe {
        libc::sysctl(
            MIB.as_ptr(),
            MIB.len() as libc::c_uint,
            buf,
            &mut len,
            ptr::null(),
            0,
        )
    };
    if ret == -1 {
        -1
    } else {
        len as libc::ssize_t
    }
}

type GetRandomFn = unsafe extern "C" fn(*mut c_void, libc::size_t, libc::c_uint) -> libc::ssize_t;

static GETRANDOM: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());

#[cold]
fn init() -> *mut c_void {
    static NAME: &[u8] = b"getrandom\0";
    let name_ptr = NAME.as_ptr().cast::<libc::c_char>();
    let mut ptr = unsafe { libc::dlsym(libc::RTLD_DEFAULT, name_ptr) };
    if ptr.is_null() {
        // Verify `polyfill_using_kern_arand` has the right signature.
        const POLYFILL: GetRandomFn = polyfill_using_kern_arand;
        ptr = POLYFILL as *mut c_void;
    }
    GETRANDOM.store(ptr, Ordering::Release);
    ptr
}

pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    // Despite being only a single atomic variable, we still cannot always use
    // Ordering::Relaxed, as we need to make sure a successful call to `init`
    // is "ordered before" any data read through the returned pointer (which
    // occurs when the function is called). Our implementation mirrors that of
    // the one in libstd, meaning that the use of non-Relaxed operations is
    // probably unnecessary.
    let mut fptr = GETRANDOM.load(Ordering::Acquire);
    if fptr.is_null() {
        fptr = init();
    }
    let fptr = unsafe { mem::transmute::<*mut c_void, GetRandomFn>(fptr) };
    sys_fill_exact(dest, |buf| unsafe {
        fptr(buf.as_mut_ptr().cast::<c_void>(), buf.len(), 0)
    })
}
