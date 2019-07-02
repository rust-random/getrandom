// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for Linux / Android
extern crate std;

use crate::util::LazyBool;
use crate::{use_file, Error};
use core::num::NonZeroU32;
use std::io;

fn syscall_getrandom(dest: &mut [u8], block: bool) -> Result<usize, io::Error> {
    let flags = if block { 0 } else { libc::GRND_NONBLOCK };
    let ret = unsafe { libc::syscall(libc::SYS_getrandom, dest.as_mut_ptr(), dest.len(), flags) };
    if ret < 0 {
        let err = io::Error::last_os_error();
        if err.raw_os_error() == Some(libc::EINTR) {
            return Ok(0); // Call was interrupted, try again
        }
        error!("Linux getrandom syscall failed with return value {}", ret);
        return Err(err);
    }
    Ok(ret as usize)
}

pub fn getrandom_inner(dest: &mut [u8]) -> Result<(), Error> {
    static HAS_GETRANDOM: LazyBool = LazyBool::new();
    if HAS_GETRANDOM.unsync_init(is_getrandom_available) {
        let mut start = 0;
        while start < dest.len() {
            start += syscall_getrandom(&mut dest[start..], true)?;
        }
        Ok(())
    } else {
        use_file::getrandom_inner(dest)
    }
}

fn is_getrandom_available() -> bool {
    match syscall_getrandom(&mut [], false) {
        Err(err) => match err.raw_os_error() {
            Some(libc::ENOSYS) => false, // No kernel support
            Some(libc::EPERM) => false,  // Blocked by seccomp
            _ => true,
        },
        Ok(_) => true,
    }
}

#[inline(always)]
pub fn error_msg_inner(_: NonZeroU32) -> Option<&'static str> {
    None
}
