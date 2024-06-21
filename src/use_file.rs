//! Implementations that just need to read from a file
use crate::{
    util_libc::{open_readonly, sys_fill_exact},
    Error,
};
use core::{
    cell::UnsafeCell,
    ffi::c_void,
    mem::MaybeUninit,
    sync::atomic::{AtomicI32, Ordering},
};

/// For all platforms, we use `/dev/urandom` rather than `/dev/random`.
/// For more information see the linked man pages in lib.rs.
///   - On Redox, only /dev/urandom is provided.
///   - On AIX, /dev/urandom will "provide cryptographically secure output".
///   - On Haiku and QNX Neutrino they are identical.
const FILE_PATH: &[u8] = b"/dev/urandom\0";

// std::os::fd::{BorrowedFd, OwnedFd} guarantee that -1 is not a valid file descriptor.
const FD_UNINIT: libc::c_int = -1;

// In theory `libc::c_int` could be something other than `i32`, but for the
// targets we currently support that use `use_file`, it is always `i32`.
// If/when we add support for a target where that isn't the case, we may
// need to use a different atomic type or make other accomodations. The
// compiler will let us know if/when that is the case, because the
// `FD.store(fd)` would fail to compile.
//
// The opening of the file, by libc/libstd/etc. may write some unknown
// state into in-process memory. (Such state may include some sanitizer
// bookkeeping, or we might be operating in a unikernal-like environment
// where all the "kernel" file descriptor bookkeeping is done in our
// process.) `get_fd_locked` stores into FD using `Ordering::Release` to
// ensure any such state is synchronized. `get_fd` loads from `FD` with
// `Ordering::Acquire` to synchronize with it.
static FD: AtomicI32 = AtomicI32::new(FD_UNINIT);

static FD_MUTEX: Mutex = Mutex::new();

pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    let mut fd = FD.load(Ordering::Acquire);
    if fd == FD_UNINIT {
        fd = open_or_wait()?;
    }
    sys_fill_exact(dest, |buf| unsafe {
        libc::read(fd, buf.as_mut_ptr().cast::<c_void>(), buf.len())
    })
}

#[cold]
fn open_or_wait() -> Result<libc::c_int, Error> {
    let _guard = FD_MUTEX.lock();
    let fd = match FD.load(Ordering::Acquire) {
        FD_UNINIT => {
            let fd = open_readonly(FILE_PATH)?;
            FD.store(fd, Ordering::Release);
            fd
        }
        fd => fd,
    };
    debug_assert!(fd >= 0);
    Ok(fd)
}

struct Mutex(UnsafeCell<libc::pthread_mutex_t>);

impl Mutex {
    const fn new() -> Self {
        Self(UnsafeCell::new(libc::PTHREAD_MUTEX_INITIALIZER))
    }

    fn lock(&self) -> MutexGuard<'_> {
        let r = unsafe { libc::pthread_mutex_lock(self.0.get()) };
        debug_assert_eq!(r, 0);
        MutexGuard(self)
    }
}

unsafe impl Sync for Mutex {}

struct MutexGuard<'a>(&'a Mutex);

impl<'a> Drop for MutexGuard<'a> {
    fn drop(&mut self) {
        let r = unsafe { libc::pthread_mutex_unlock(self.0 .0.get()) };
        debug_assert_eq!(r, 0);
    }
}
