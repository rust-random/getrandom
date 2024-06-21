//! Implementation for Hermit
use crate::Error;
use core::mem::MaybeUninit;

extern "C" {
    fn sys_read_entropy(buffer: *mut u8, length: usize, flags: u32) -> isize;
}

pub fn getrandom_inner(mut dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    while !dest.is_empty() {
        let res = unsafe { sys_read_entropy(dest.as_mut_ptr().cast::<u8>(), dest.len(), 0) };
        match usize::try_from(res) {
            Ok(res) if res > 0 => dest = dest.get_mut(res..).ok_or(Error::UNEXPECTED)?,
            _ => {
                let err = u32::try_from(res.unsigned_abs())
                    .ok()
                    .map_or(Error::UNEXPECTED, Error::from_os_error);
                return Err(err);
            }
        }
    }
    Ok(())
}
