//! Implementation for Hermit
use crate::Error;
use core::mem::MaybeUninit;

// Note that `sys_secure_rand32` and `sys_secure_rand64` are implemented using `sys_read_entropy`:
// https://docs.rs/libhermit-rs/0.6.3/src/hermit/syscalls/entropy.rs.html#62-97
pub use crate::util::{inner_u32, inner_u64};

extern "C" {
    fn sys_read_entropy(buffer: *mut u8, length: usize, flags: u32) -> isize;
}

pub fn fill_inner(mut dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
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
