//! Implementation for ESP-IDF
use crate::Backend;
use crate::Error;
use core::ffi::c_void;

extern "C" {
    fn esp_fill_random(buf: *mut c_void, len: usize) -> u32;
}

pub struct EspIdfBackend;

unsafe impl Backend for EspIdfBackend {
    #[inline]
    unsafe fn fill_ptr(dest: *mut u8, len: usize) -> Result<(), Error> {
        // Not that NOT enabling WiFi, BT, or the voltage noise entropy source (via `bootloader_random_enable`)
        // will cause ESP-IDF to return pseudo-random numbers based on the voltage noise entropy, after the initial boot process:
        // https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/system/random.html
        //
        // However tracking if some of these entropy sources is enabled is way too difficult to implement here
        esp_fill_random(dest.cast(), len);
        Ok(())
    }
}
