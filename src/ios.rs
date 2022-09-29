// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for iOS
use crate::Error;
use core::{ffi::c_void, ptr::null};

#[link(name = "Security", kind = "framework")]
extern "C" {
    fn SecRandomCopyBytes(rnd: *const c_void, count: usize, bytes: *mut u8) -> i32;
}

pub unsafe fn getrandom_inner(dst: *mut u8, len: usize) -> Result<(), Error> {
    // Apple's documentation guarantees kSecRandomDefault is a synonym for NULL.
    let ret = SecRandomCopyBytes(null(), len, dst);
    // errSecSuccess (from SecBase.h) is always zero.
    match ret {
        0 => Ok(()),
        _ => Err(Error::IOS_SEC_RANDOM),
    }
}
