extern crate std;

use crate::Error;
use std::io;

impl From<Error> for io::Error {
    fn from(err: Error) -> Self {
        #[cfg(not(target_os = "uefi"))]
        match err.raw_os_error() {
            Some(errno) => io::Error::from_raw_os_error(errno),
            None => io::Error::new(io::ErrorKind::Other, err),
        }
        #[cfg(target_os = "uefi")]
        {
            io::Error::new(io::ErrorKind::Other, err)
        }
    }
}

impl std::error::Error for Error {}
