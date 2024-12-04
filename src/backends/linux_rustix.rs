//! Implementation for Linux / Android without `/dev/urandom` fallback
use crate::{Error, MaybeUninit};
use rustix::rand::{getrandom_uninit, GetRandomFlags};

pub use crate::default_impls::{insecure_fill_uninit, insecure_u32, insecure_u64, u32, u64};

#[cfg(not(any(target_os = "android", target_os = "linux")))]
compile_error!("`linux_rustix` backend can be enabled only for Linux/Android targets!");

pub fn fill_uninit(mut dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    loop {
        let res = getrandom_uninit(dest, GetRandomFlags::empty()).map(|(res, _)| res.len());
        match res {
            Ok(0) => return Err(Error::UNEXPECTED),
            Ok(res_len) => {
                dest = dest.get_mut(res_len..).ok_or(Error::UNEXPECTED)?;
                if dest.is_empty() {
                    return Ok(());
                }
            }
            Err(rustix::io::Errno::INTR) => continue,
            Err(err) => {
                let code = err
                    .raw_os_error()
                    .wrapping_neg()
                    .try_into()
                    .map_err(|_| Error::UNEXPECTED)?;
                return Err(Error::from_os_error(code));
            }
        }
    }
}
