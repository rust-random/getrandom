#![allow(dead_code)]
use crate::Error;
use core::mem::MaybeUninit;

// Fill a buffer by repeatedly invoking a system call. The `sys_fill` function
// must return `Ok(written)` where `written` is the number of bytes written,
// or otherwise an error.
pub fn sys_fill_exact(
    mut buf: &mut [MaybeUninit<u8>],
    sys_fill: impl Fn(&mut [MaybeUninit<u8>]) -> Result<usize, Error>,
) -> Result<(), Error> {
    while !buf.is_empty() {
        match sys_fill(buf) {
            Ok(res) if res > 0 => buf = buf.get_mut(res..).ok_or(Error::UNEXPECTED)?,
            Err(err) => {
                // We should try again if the call was interrupted.
                if err.raw_os_error() != Some(libc::EINTR) {
                    return Err(err);
                }
            }
            // Negative return codes not equal to -1 should be impossible.
            // EOF (ret = 0) should be impossible, as the data we are reading
            // should be an infinite stream of random bytes.
            _ => return Err(Error::UNEXPECTED),
        }
    }
    Ok(())
}
