#![allow(dead_code)]
use crate::Error;
use core::{mem::MaybeUninit, num::NonZeroU32};

cfg_if! {
    if #[cfg(any(target_os = "netbsd", target_os = "openbsd", target_os = "android"))] {
        use libc::__errno as errno_location;
    } else if #[cfg(any(target_os = "linux", target_os = "emscripten", target_os = "hurd", target_os = "redox", target_os = "dragonfly"))] {
        use libc::__errno_location as errno_location;
    } else if #[cfg(any(target_os = "solaris", target_os = "illumos"))] {
        use libc::___errno as errno_location;
    } else if #[cfg(any(target_os = "macos", target_os = "freebsd"))] {
        use libc::__error as errno_location;
    } else if #[cfg(target_os = "haiku")] {
        use libc::_errnop as errno_location;
    } else if #[cfg(target_os = "nto")] {
        use libc::__get_errno_ptr as errno_location;
    } else if #[cfg(any(all(target_os = "horizon", target_arch = "arm"), target_os = "vita"))] {
        extern "C" {
            // Not provided by libc: https://github.com/rust-lang/libc/issues/1995
            fn __errno() -> *mut libc::c_int;
        }
        use __errno as errno_location;
    } else if #[cfg(target_os = "aix")] {
        use libc::_Errno as errno_location;
    }
}

cfg_if! {
    if #[cfg(target_os = "vxworks")] {
        use libc::errnoGet as get_errno;
    } else {
        unsafe fn get_errno() -> libc::c_int { *errno_location() }
    }
}

pub fn last_os_error() -> Error {
    let errno: libc::c_int = unsafe { get_errno() };

    // c_int-to-u32 conversion is lossless for nonnegative values if they are the same size.
    const _: () = assert!(core::mem::size_of::<libc::c_int>() == core::mem::size_of::<u32>());

    u32::try_from(errno)
        .ok()
        .and_then(NonZeroU32::new)
        .map_or(Error::ERRNO_NOT_POSITIVE, Error::from)
}

// Fill a buffer by repeatedly invoking a system call. The `sys_fill` function:
//   - should return -1 and set errno on failure
//   - should return the number of bytes written on success
pub fn sys_fill_exact(
    mut buf: &mut [MaybeUninit<u8>],
    sys_fill: impl Fn(&mut [MaybeUninit<u8>]) -> libc::ssize_t,
) -> Result<(), Error> {
    while !buf.is_empty() {
        let res = sys_fill(buf);
        match res {
            res if res > 0 => buf = buf.get_mut(res as usize..).ok_or(Error::UNEXPECTED)?,
            -1 => {
                let err = last_os_error();
                // We should try again if the call was interrupted.
                if err.raw_os_error() != Some(libc::EINTR) {
                    return Err(err);
                }
            }
            // Negative return codes not equal to -1 should be impossible.
            // EOF (ret = 0) should be impossible, as the data we are reading
            // should be an infinite stream of random bytes.
            _ => return Err(Error::UNEXPECTED),
        }
    }
    Ok(())
}

/// Open a file in read-only mode.
///
/// # Panics
/// If `path` does not contain any zeros.
// TODO: Move `path` to `CStr` and use `CStr::from_bytes_until_nul` (MSRV 1.69)
// or C-string literals (MSRV 1.77) for statics
#[inline(always)]
pub fn open_readonly(path: &[u8]) -> Result<libc::c_int, Error> {
    assert!(path.iter().any(|&b| b == 0));
    loop {
        let fd = unsafe {
            libc::open(
                path.as_ptr().cast::<libc::c_char>(),
                libc::O_RDONLY | libc::O_CLOEXEC,
            )
        };
        if fd >= 0 {
            return Ok(fd);
        }
        let err = last_os_error();
        // We should try again if open() was interrupted.
        if err.raw_os_error() != Some(libc::EINTR) {
            return Err(err);
        }
    }
}

/// Thin wrapper around the `getrandom()` Linux system call
#[cfg(any(target_os = "android", target_os = "linux"))]
pub fn getrandom_syscall(buf: &mut [MaybeUninit<u8>]) -> libc::ssize_t {
    unsafe {
        libc::syscall(
            libc::SYS_getrandom,
            buf.as_mut_ptr().cast::<core::ffi::c_void>(),
            buf.len(),
            0,
        ) as libc::ssize_t
    }
}
