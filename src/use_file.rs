//! Implementations that just need to read from a file
use crate::{
    util_libc::{open_readonly, sys_fill_exact},
    Error,
};
use core::{
    ffi::c_void,
    mem::MaybeUninit,
    sync::atomic::{AtomicUsize, Ordering::Relaxed},
};
extern crate std;
use std::sync::{Mutex, PoisonError};

/// For all platforms, we use `/dev/urandom` rather than `/dev/random`.
/// For more information see the linked man pages in lib.rs.
///   - On Linux, "/dev/urandom is preferred and sufficient in all use cases".
///   - On Redox, only /dev/urandom is provided.
///   - On AIX, /dev/urandom will "provide cryptographically secure output".
///   - On Haiku and QNX Neutrino they are identical.
const FILE_PATH: &[u8] = b"/dev/urandom\0";
const FD_UNINIT: usize = usize::max_value();

// Do not inline this when it is the fallback implementation, but don't mark it
// `#[cold]` because it is hot when it is actually used.
#[cfg_attr(any(target_os = "android", target_os = "linux"), inline(never))]
pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    let fd = get_rng_fd()?;
    sys_fill_exact(dest, |buf| unsafe {
        libc::read(fd, buf.as_mut_ptr().cast::<c_void>(), buf.len())
    })
}

// Returns the file descriptor for the device file used to retrieve random
// bytes. The file will be opened exactly once. All subsequent calls will
// return the same file descriptor. This file descriptor is never closed.
fn get_rng_fd() -> Result<libc::c_int, Error> {
    static FD: AtomicUsize = AtomicUsize::new(FD_UNINIT);

    fn get_fd() -> Option<libc::c_int> {
        match FD.load(Relaxed) {
            FD_UNINIT => None,
            val => Some(val as libc::c_int),
        }
    }

    #[cold]
    fn get_fd_locked() -> Result<libc::c_int, Error> {
        static MUTEX: Mutex<()> = Mutex::new(());
        let _guard = MUTEX
            .lock()
            .map_err(|_: PoisonError<_>| Error::UNEXPECTED_FILE_MUTEX_POISONED)?;

        if let Some(fd) = get_fd() {
            return Ok(fd);
        }

        // On Linux, /dev/urandom might return insecure values.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        crate::imp::wait_until_rng_ready()?;

        let fd = open_readonly(FILE_PATH)?;
        // The fd always fits in a usize without conflicting with FD_UNINIT.
        debug_assert!(fd >= 0 && (fd as usize) < FD_UNINIT);
        FD.store(fd as usize, Relaxed);

        Ok(fd)
    }

    // Use double-checked locking to avoid acquiring the lock if possible.
    if let Some(fd) = get_fd() {
        Ok(fd)
    } else {
        get_fd_locked()
    }
}
