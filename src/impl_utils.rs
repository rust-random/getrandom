/// Implement utilities used in multiple backends.
// TODO: remove `pub(crate)` after the `linux_android_with_fallback` backend is removed
#[allow(unused_macros, reason = "not used in all backends")]
macro_rules! impl_utils {
    (last_os_error) => {
        pub(crate) fn last_os_error() -> $crate::Error {
            // We assume that on all targets which use the `util_libc` module `c_int` is equal to `i32`
            let errno: i32 = unsafe { get_errno() };

            if errno > 0 {
                let code = errno
                    .checked_neg()
                    .expect("Positive number can be always negated");
                Error::from_neg_error_code(code)
            } else {
                Error::ERRNO_NOT_POSITIVE
            }
        }
    };

    (sys_fill_exact) => {
        /// Fill a buffer by repeatedly invoking `sys_fill`.
        ///
        /// The `sys_fill` function:
        ///   - should return -1 and set errno on failure
        ///   - should return the number of bytes written on success
        pub(crate) fn sys_fill_exact(
            mut buf: &mut [core::mem::MaybeUninit<u8>],
            sys_fill: impl Fn(&mut [core::mem::MaybeUninit<u8>]) -> libc::ssize_t,
        ) -> Result<(), Error> {
            while !buf.is_empty() {
                let res = sys_fill(buf);
                match res {
                    res if res > 0 => {
                        let len = usize::try_from(res).map_err(|_| Error::UNEXPECTED)?;
                        buf = buf.get_mut(len..).ok_or(Error::UNEXPECTED)?;
                    }
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
    };

    (get_errno) => {
        cfg_if! {
            if #[cfg(any(target_os = "netbsd", target_os = "openbsd", target_os = "android", target_os = "cygwin"))] {
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
                unsafe extern "C" {
                    // Not provided by libc: https://github.com/rust-lang/libc/issues/1995
                    fn __errno() -> *mut libc::c_int;
                }
                use __errno as errno_location;
            } else if #[cfg(target_os = "aix")] {
                use libc::_Errno as errno_location;
            } else {
                compile_error!("`errno_location` is not provided for the target!");
            }
        }

        unsafe fn get_errno() -> libc::c_int {
            unsafe { core::ptr::read(errno_location()) }
        }
    };

    (unpoison) => {
        /// Unpoisons `buf` if MSAN support is enabled.
        ///
        /// Most backends do not need to unpoison their output. Rust language- and
        /// library- provided functionality unpoisons automatically. Similarly, libc
        /// either natively supports MSAN and/or MSAN hooks libc-provided functions
        /// to unpoison outputs on success. Only when all of these things are
        /// bypassed do we need to do it ourselves.
        ///
        /// The call to unpoison should be done as close to the write as possible.
        /// For example, if the backend partially fills the output buffer in chunks,
        /// each chunk should be unpoisoned individually. This way, the correctness of
        /// the chunking logic can be validated (in part) using MSAN.
        unsafe fn unpoison(buf: &mut [core::mem::MaybeUninit<u8>]) {
            cfg_if! {
                if #[cfg(getrandom_msan)] {
                    unsafe extern "C" {
                        fn __msan_unpoison(a: *mut core::ffi::c_void, size: usize);
                    }
                    let a = buf.as_mut_ptr().cast();
                    let size = buf.len();
                    unsafe {
                        __msan_unpoison(a, size);
                    }
                } else {
                    let _ = buf;
                }
            }
        }
    };

    (unpoison_linux_getrandom_result) => {
        /// Interprets the result of the `getrandom` syscall of Linux, unpoisoning any
        /// written part of `buf`.
        ///
        /// `buf` must be the output buffer that was originally passed to the `getrandom`
        /// syscall.
        ///
        /// `ret` must be the result returned by `getrandom`. If `ret` is negative or
        /// larger than the length of `buf` then nothing is done.
        ///
        /// Memory Sanitizer only intercepts `getrandom` on this condition (from its
        /// source code):
        /// ```c
        /// #define SANITIZER_INTERCEPT_GETRANDOM \
        ///   ((SI_LINUX && __GLIBC_PREREQ(2, 25)) || SI_FREEBSD || SI_SOLARIS)
        /// ```
        /// So, effectively, we have to assume that it is never intercepted on Linux.
        unsafe fn unpoison_linux_getrandom_result(buf: &mut [core::mem::MaybeUninit<u8>], ret: isize) {
            $crate::impl_utils!(unpoison);

            if let Ok(bytes_written) = usize::try_from(ret) {
                if let Some(written) = buf.get_mut(..bytes_written) {
                    unsafe { unpoison(written) }
                }
            }
        }
    };

    ($($util:ident),* $(,)?) => {
        $($crate::impl_utils!($util);)*
    };
}

pub(crate) use impl_utils;
