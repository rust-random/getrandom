//! Utilities for using rustix.
//!
//! At this point in time it is only used on Linux-like operating systems.

use crate::Error;
use core::convert::TryInto;
use core::mem::MaybeUninit;
use core::num::NonZeroU32;

use rustix::fd::OwnedFd;
use rustix::fs;
use rustix::io::Errno;
use rustix::rand;

/// Convert a Rustix error to one of our errors.
pub(crate) fn cvt(err: Errno) -> Error {
    match TryInto::<u32>::try_into(err.raw_os_error())
        .ok()
        .and_then(NonZeroU32::new)
    {
        Some(code) => Error::from(code),
        None => Error::ERRNO_NOT_POSITIVE,
    }
}

/// Fill a buffer by repeatedly invoking a `rustix` call.
pub(crate) fn sys_fill_exact(
    mut buf: &mut [MaybeUninit<u8>],
    fill: impl Fn(&mut [MaybeUninit<u8>]) -> Result<(&mut [u8], &mut [MaybeUninit<u8>]), Errno>,
) -> Result<(), Error> {
    while !buf.is_empty() {
        // Read into the buffer.
        match fill(buf) {
            Err(err) => return Err(cvt(err)),
            Ok((_filled, unfilled)) => {
                buf = unfilled;
            }
        }
    }

    Ok(())
}

/// Open a file as read-only.
pub(crate) fn open_readonly(path: &str) -> Result<OwnedFd, Error> {
    loop {
        match fs::open(
            path,
            fs::OFlags::CLOEXEC | fs::OFlags::RDONLY,
            fs::Mode::empty(),
        ) {
            Ok(file) => return Ok(file),
            Err(Errno::INTR) => continue,
            Err(err) => return Err(cvt(err)),
        }
    }
}

pub(crate) fn getrandom_syscall(
    buf: &mut [MaybeUninit<u8>],
) -> Result<(&mut [u8], &mut [MaybeUninit<u8>]), Errno> {
    rand::getrandom_uninit(buf, rand::GetRandomFlags::empty())
}
