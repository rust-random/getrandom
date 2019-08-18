// Copyright 2019 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use crate::error::ERRNO_NOT_POSITIVE;
use crate::util::LazyUsize;
use crate::Error;
use core::num::NonZeroU32;
use core::ptr::NonNull;

cfg_if! {
    if #[cfg(any(target_os = "netbsd", target_os = "openbsd", target_os = "android"))] {
        use libc::__errno as errno_location;
    } else if #[cfg(any(target_os = "linux", target_os = "emscripten", target_os = "redox"))] {
        use libc::__errno_location as errno_location;
    } else if #[cfg(any(target_os = "solaris", target_os = "illumos"))] {
        use libc::___errno as errno_location;
    } else if #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "dragonfly"))] {
        use libc::__error as errno_location;
    } else if #[cfg(target_os = "haiku")] {
        use libc::_errnop as errno_location;
    }
}

pub fn last_os_error() -> Error {
    #[cfg(not(target_os = "vxworks"))]
    let errno = unsafe { *errno_location() };
    #[cfg(target_os = "vxworks")]
    let errno = unsafe { libc::errnoGet() };
    if errno > 0 {
        Error::from(NonZeroU32::new(errno as u32).unwrap())
    } else {
        ERRNO_NOT_POSITIVE
    }
}

// Fill a buffer by repeatedly invoking a system call. The `sys_fill` function:
//   - should return -1 and set errno on failure
//   - should return the number of bytes written on success
pub fn sys_fill_exact(
    mut buf: &mut [u8],
    sys_fill: impl Fn(&mut [u8]) -> libc::ssize_t,
) -> Result<(), Error> {
    while !buf.is_empty() {
        let res = sys_fill(buf);
        if res < 0 {
            let err = last_os_error();
            // We should try again if the call was interrupted.
            if err.raw_os_error() != Some(libc::EINTR) {
                return Err(err);
            }
        } else {
            // We don't check for EOF (ret = 0) as the data we are reading
            // should be an infinite stream of random bytes.
            buf = &mut buf[(res as usize)..];
        }
    }
    Ok(())
}

// A "weak" binding to a C function that may or may not be present at runtime.
// Used for supporting newer OS features while still building on older systems.
// F must be a function pointer of type `unsafe extern "C" fn`. Based off of the
// weak! macro in libstd.
pub struct Weak {
    name: &'static str,
    addr: LazyUsize,
}

impl Weak {
    // Construct a binding to a C function with a given name. This function is
    // unsafe because `name` _must_ be null terminated.
    pub const unsafe fn new(name: &'static str) -> Self {
        Self {
            name,
            addr: LazyUsize::new(),
        }
    }

    // Return a function pointer if present at runtime. Otherwise, return null.
    pub fn ptr(&self) -> Option<NonNull<libc::c_void>> {
        let addr = self.addr.unsync_init(|| unsafe {
            libc::dlsym(libc::RTLD_DEFAULT, self.name.as_ptr() as *const _) as usize
        });
        NonNull::new(addr as *mut _)
    }
}

pub struct LazyFd(LazyUsize);

impl LazyFd {
    pub const fn new() -> Self {
        Self(LazyUsize::new())
    }

    // If init() returns Some(x), x should be nonnegative.
    pub fn init(&self, init: impl FnOnce() -> Option<libc::c_int>) -> Option<libc::c_int> {
        let fd = self.0.sync_init(
            || match init() {
                // OK as val >= 0 and val <= c_int::MAX < usize::MAX
                Some(val) => val as usize,
                None => LazyUsize::UNINIT,
            },
            || unsafe {
                // We are usually waiting on an open(2) syscall to complete,
                // which typically takes < 10us if the file is a device.
                // However, we might end up waiting much longer if the entropy
                // pool isn't initialized, but even in that case, this loop will
                // consume a negligible amount of CPU on most platforms.
                libc::usleep(10);
            },
        );
        match fd {
            LazyUsize::UNINIT => None,
            val => Some(val as libc::c_int),
        }
    }
}

cfg_if! {
    if #[cfg(any(target_os = "linux", target_os = "emscripten"))] {
        use libc::open64 as open;
    } else {
        use libc::open;
    }
}

// SAFETY: path must be null terminated, FD must be manually closed.
pub unsafe fn open_readonly(path: &str) -> Option<libc::c_int> {
    debug_assert!(path.as_bytes().last() == Some(&0));
    let fd = open(path.as_ptr() as *mut _, libc::O_RDONLY | libc::O_CLOEXEC);
    if fd < 0 {
        return None;
    }
    // O_CLOEXEC works on all Unix targets except for older Linux kernels (pre
    // 2.6.23), so we also use an ioctl to make sure FD_CLOEXEC is set.
    #[cfg(target_os = "linux")]
    libc::ioctl(fd, libc::FIOCLEX);
    Some(fd)
}
