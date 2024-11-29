//! Implementation for ESP-IDF.
//!
//! Note that NOT enabling WiFi, BT, or the voltage noise entropy source
//! (via `bootloader_random_enable`) will cause ESP-IDF to return pseudo-random numbers based on
//! the voltage noise entropy, after the initial boot process:
//! https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/system/random.html
//!
//! However tracking if some of these entropy sources is enabled is way too difficult
//! to implement here.
use crate::Error;
use core::{ffi::c_void, mem::MaybeUninit};

pub use crate::default_impls::{insecure_fill_uninit, insecure_u32, insecure_u64};

#[cfg(not(target_os = "espidf"))]
compile_error!("`esp_idf` backend can be enabled only for ESP-IDF targets!");

extern "C" {
    fn esp_random() -> u32;
    fn esp_fill_random(buf: *mut c_void, len: usize);
}

pub fn u32() -> Result<u32, Error> {
    Ok(unsafe { esp_random() })
}

pub fn u64() -> Result<u64, Error> {
    let (a, b) = unsafe { (esp_random(), esp_random()) };
    let res = (u64::from(a) << 32) | u64::from(b);
    Ok(res)
}

pub fn fill_uninit(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    unsafe { esp_fill_random(dest.as_mut_ptr().cast(), dest.len()) };
    Ok(())
}
