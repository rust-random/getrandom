// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
#![allow(non_camel_case_types)]

use crate::Error;
use core::{
    convert::TryInto,
    ffi::{c_long, c_void},
    mem::MaybeUninit,
    num::NonZeroU32,
    ptr,
};

// same as Rust's libstd.
type BCRYPT_ALG_HANDLE = *mut c_void;
type NTSTATUS = c_long;

// "RNG\0"
const BCRYPT_RNG_ALGORITHM: &[u16] = &[b'R' as u16, b'N' as u16, b'G' as u16, 0];
const BCRYPT_USE_SYSTEM_PREFERRED_RNG: u32 = 0x00000002;

// Equivalent to the `NT_SUCCESS` C preprocessor macro.
// See: https://docs.microsoft.com/en-us/windows-hardware/drivers/kernel/using-ntstatus-values
fn nt_success(status: NTSTATUS) -> bool {
    status >= 0
}

/// Extract error code and turn into an `Error`
fn nt_error(status: NTSTATUS) -> Error {
    // We zeroize the highest bit, so the error code will reside
    // inside the range designated for OS codes.
    let code = status as u32 ^ (1 << 31);
    // SAFETY: the second highest bit is always equal to one,
    // so it's impossible to get zero. Unfortunately the type
    // system does not have a way to express this yet.
    let code = unsafe { NonZeroU32::new_unchecked(code) };
    Error::from(code)
}

#[link(name = "bcrypt")]
extern "system" {
    fn BCryptGenRandom(
        hAlgorithm: BCRYPT_ALG_HANDLE,
        pBuffer: *mut u8,
        cbBuffer: u32,
        dwFlags: u32,
    ) -> NTSTATUS;
    pub fn BCryptOpenAlgorithmProvider(
        phalgorithm: *mut BCRYPT_ALG_HANDLE,
        pszAlgId: *const u16,
        pszimplementation: *const u16,
        dwflags: u32,
    ) -> NTSTATUS;
    pub fn BCryptCloseAlgorithmProvider(hAlgorithm: BCRYPT_ALG_HANDLE, dwFlags: u32) -> NTSTATUS;
}

pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    // Prevent overflow of u32
    for chunk in dest.chunks_mut(u32::max_value() as usize) {
        if let Err(_) = Rng::SYSTEM.random(chunk) {
            fallback_rng(chunk)?;
        }
    }
    Ok(())
}

struct Rng {
    algorithm: BCRYPT_ALG_HANDLE,
    flags: u32,
}

impl Rng {
    const SYSTEM: Self = unsafe { Self::new(ptr::null_mut(), BCRYPT_USE_SYSTEM_PREFERRED_RNG) };

    /// Create the RNG from an existing algorithm handle.
    ///
    /// # Safety
    ///
    /// The handle must either be null or a valid algorithm handle.
    const unsafe fn new(algorithm: BCRYPT_ALG_HANDLE, flags: u32) -> Self {
        Self { algorithm, flags }
    }

    /// Open a handle to the RNG algorithm.
    fn open() -> Result<Self, Error> {
        use core::sync::atomic::AtomicPtr;
        use core::sync::atomic::Ordering::{Acquire, Release};

        // An atomic is used so we don't need to reopen the handle every time.
        static HANDLE: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());

        let mut handle = HANDLE.load(Acquire);
        if handle.is_null() {
            let status = unsafe {
                BCryptOpenAlgorithmProvider(
                    &mut handle,
                    BCRYPT_RNG_ALGORITHM.as_ptr(),
                    ptr::null(),
                    0,
                )
            };
            if nt_success(status) {
                // If another thread opens a handle first then use that handle instead.
                let result = HANDLE.compare_exchange(ptr::null_mut(), handle, Release, Acquire);
                if let Err(previous_handle) = result {
                    // Close our handle and return the previous one.
                    unsafe { BCryptCloseAlgorithmProvider(handle, 0) };
                    handle = previous_handle;
                }
                Ok(unsafe { Self::new(handle, 0) })
            } else {
                Err(nt_error(status))
            }
        } else {
            Ok(unsafe { Self::new(handle, 0) })
        }
    }

    fn random(&self, dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
        let len: u32 = dest.len().try_into().unwrap();
        // SAFETY: dest is valid, writable buffer of length len
        let ret = unsafe {
            BCryptGenRandom(
                self.algorithm,
                dest.as_mut_ptr() as *mut u8,
                len,
                self.flags,
            )
        };

        if nt_success(ret) {
            return Ok(());
        }

        Err(nt_error(ret))
    }
}

/// Generate random numbers using the fallback RNG function
#[inline(never)]
fn fallback_rng(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    Rng::open()?.random(dest)
}
