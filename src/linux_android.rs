// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for Linux / Android
use crate::util::LazyBool;
use crate::util_libc::{last_os_error, sys_fill_exact};
use crate::{use_file, Error};

pub fn getrandom_inner(dest: &mut [u8]) -> Result<(), Error> {
    static HAS_GETRANDOM: LazyBool = LazyBool::new();
    if HAS_GETRANDOM.unsync_init(is_getrandom_available) {
        sys_fill_exact(dest, |buf| unsafe {
            libc::syscall(libc::SYS_getrandom, buf.as_mut_ptr(), buf.len(), 0) as libc::ssize_t
        })
    } else {
        use_file::getrandom_inner(dest)
    }
}

fn is_getrandom_available() -> bool {
    let res = unsafe { libc::syscall(libc::SYS_getrandom, 0usize, 0usize, libc::GRND_NONBLOCK) };
    if res < 0 {
        match last_os_error().raw_os_error() {
            Some(libc::ENOSYS) => false, // No kernel support
            Some(libc::EPERM) => false,  // Blocked by seccomp
            _ => true,
        }
    } else {
        true
    }
}
