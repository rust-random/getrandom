//! Implementation for Hermit
use crate::Error;
use core::mem::MaybeUninit;

extern "C" {
    fn sys_read_entropy(buffer: *mut u8, length: usize, flags: u32) -> isize;
}

pub fn getrandom_inner(mut dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    while !dest.is_empty() {
        let res = unsafe { sys_read_entropy(dest.as_mut_ptr().cast::<u8>(), dest.len(), 0) };
        match res {
            res if res > 0 => {
                let len = usize::try_from(res).map_err(|_| Error::UNEXPECTED)?;
                dest = dest.get_mut(len..).ok_or(Error::UNEXPECTED)?;
            }
            code => {
                let err = u32::try_from(code.unsigned_abs())
                    .ok()
                    .map_or(Error::UNEXPECTED, Error::from_os_error);
                return Err(err);
            }
        }
    }
    Ok(())
}
