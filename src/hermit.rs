//! Implementation for Hermit
use crate::Error;
use core::mem::MaybeUninit;

extern "C" {
    fn sys_read_entropy(buffer: *mut u8, length: usize, flags: u32) -> isize;
}

pub fn getrandom_inner(mut dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    while !dest.is_empty() {
        let res = unsafe { sys_read_entropy(dest.as_mut_ptr().cast::<u8>(), dest.len(), 0) };
        // Positive `isize`s can be safely casted to `usize`
        if res > 0 && (res as usize) <= dest.len() {
            dest = &mut dest[res as usize..];
        } else {
            let err = if res < 0 {
                u32::try_from(res.unsigned_abs())
                    .ok()
                    .map_or(Error::UNEXPECTED, Error::from_os_error)
            } else {
                Error::UNEXPECTED
            };
            return Err(err);
        }
    }
    Ok(())
}
