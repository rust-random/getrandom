use crate::Error;
use core::{convert::TryInto, mem::MaybeUninit, num::NonZeroU32};

extern "C" {
    fn sys_read_entropy(buffer: *mut u8, length: usize, flags: u32) -> isize;
}

pub fn getrandom_inner(mut dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    while !dest.is_empty() {
        let res = unsafe { sys_read_entropy(dest.as_mut_ptr() as *mut u8, dest.len(), 0) };
        if res > 0 {
            dest = dest.get_mut(res as usize..).ok_or(Error::UNEXPECTED)?;
        } else {
            // We should not get `res` equal to zero or smaller than `-i32::MAX`.
            // If we get such unexpected value after all, we will return `Error::UNEXPECTED`.
            let err = res
                .checked_neg()
                .and_then(|val| val.try_into().ok())
                .and_then(NonZeroU32::new)
                .map(Into::into)
                .unwrap_or(Error::UNEXPECTED);
            return Err(err);
        }
    }
    Ok(())
}
