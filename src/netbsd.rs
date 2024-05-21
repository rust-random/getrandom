//! Implementation for NetBSD
use crate::{lazy::LazyPtr, util_libc::sys_fill_exact, Error};
use core::{ffi::c_void, mem::MaybeUninit, ptr};

fn kern_arnd(buf: &mut [MaybeUninit<u8>]) -> libc::ssize_t {
    static MIB: [libc::c_int; 2] = [libc::CTL_KERN, libc::KERN_ARND];
    let mut len = buf.len();
    let ret = unsafe {
        libc::sysctl(
            MIB.as_ptr(),
            MIB.len() as libc::c_uint,
            buf.as_mut_ptr() as *mut _,
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

type GetRandomFn = unsafe extern "C" fn(*mut u8, libc::size_t, libc::c_uint) -> libc::ssize_t;

// getrandom(2) was introduced in NetBSD 10.0
static GETRANDOM: LazyPtr = LazyPtr::new();

fn dlsym_getrandom() -> *mut c_void {
    static NAME: &[u8] = b"getrandom\0";
    let name_ptr = NAME.as_ptr() as *const libc::c_char;
    unsafe { libc::dlsym(libc::RTLD_DEFAULT, name_ptr) }
}

pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    let fptr = GETRANDOM.unsync_init(dlsym_getrandom);
    if !fptr.is_null() {
        let func: GetRandomFn = unsafe { core::mem::transmute(fptr) };
        return sys_fill_exact(dest, |buf| unsafe {
            func(buf.as_mut_ptr() as *mut u8, buf.len(), 0)
        });
    }

    // NetBSD will only return up to 256 bytes at a time, and
    // older NetBSD kernels will fail on longer buffers.
    for chunk in dest.chunks_mut(256) {
        sys_fill_exact(chunk, kern_arnd)?
    }
    Ok(())
}
