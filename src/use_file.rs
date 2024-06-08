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
///   - On Linux, "/dev/urandom is preferred and sufficient in all use cases".
///   - On Redox, only /dev/urandom is provided.
///   - On AIX, /dev/urandom will "provide cryptographically secure output".
///   - On Haiku and QNX Neutrino they are identical.
const FILE_PATH: &[u8] = b"/dev/urandom\0";

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

    fn get_fd() -> Option<libc::c_int> {
        match FD.load(Ordering::Acquire) {
            FD_UNINIT => None,
            val => Some(val),
        }
    }

    #[cold]
    fn get_fd_locked() -> Result<libc::c_int, Error> {
        // This mutex is used to prevent multiple threads from opening file
        // descriptors concurrently, which could run into the limit on the
        // number of open file descriptors. Our goal is to have no more than one
        // file descriptor open, ever.
        //
        // SAFETY: We use the mutex only in this method, and we always unlock it
        // before returning, making sure we don't violate the pthread_mutex_t API.
        static MUTEX: Mutex = Mutex::new();
        unsafe { MUTEX.lock() };
        let _guard = DropGuard(|| unsafe { MUTEX.unlock() });

        if let Some(fd) = get_fd() {
            return Ok(fd);
        }

        // On Linux, /dev/urandom might return insecure values.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        wait_until_rng_ready()?;

        let fd = open_readonly(FILE_PATH)?;
        debug_assert!(fd != FD_UNINIT);
        FD.store(fd, Ordering::Release);

        Ok(fd)
    }

    // Use double-checked locking to avoid acquiring the lock if possible.
    if let Some(fd) = get_fd() {
        Ok(fd)
    } else {
        get_fd_locked()
    }
}

// Polls /dev/random to make sure it is ok to read from /dev/urandom.
//
// Polling avoids draining the estimated entropy from /dev/random;
// short-lived processes reading even a single byte from /dev/random could
// be problematic if they are being executed faster than entropy is being
// collected.
//
// OTOH, reading a byte instead of polling is more compatible with
// sandboxes that disallow `poll()` but which allow reading /dev/random,
// e.g. sandboxes that assume that `poll()` is for network I/O. This way,
// fewer applications will have to insert pre-sandbox-initialization logic.
// Often (blocking) file I/O is not allowed in such early phases of an
// application for performance and/or security reasons.
//
// It is hard to write a sandbox policy to support `libc::poll()` because
// it may invoke the `poll`, `ppoll`, `ppoll_time64` (since Linux 5.1, with
// newer versions of glibc), and/or (rarely, and probably only on ancient
// systems) `select`. depending on the libc implementation (e.g. glibc vs
// musl), libc version, potentially the kernel version at runtime, and/or
// the target architecture.
//
// BoringSSL and libstd don't try to protect against insecure output from
// `/dev/urandom'; they don't open `/dev/random` at all.
//
// OpenSSL uses `libc::select()` unless the `dev/random` file descriptor
// is too large; if it is too large then it does what we do here.
//
// libsodium uses `libc::poll` similarly to this.
#[cfg(any(target_os = "android", target_os = "linux"))]
fn wait_until_rng_ready() -> Result<(), Error> {
    let fd = open_readonly(b"/dev/random\0")?;
    let mut pfd = libc::pollfd {
        fd,
        events: libc::POLLIN,
        revents: 0,
    };
    let _guard = DropGuard(|| unsafe {
        libc::close(fd);
    });

    loop {
        // A negative timeout means an infinite timeout.
        let res = unsafe { libc::poll(&mut pfd, 1, -1) };
        if res >= 0 {
            debug_assert_eq!(res, 1); // We only used one fd, and cannot timeout.
            return Ok(());
        }
        let err = crate::util_libc::last_os_error();
        match err.raw_os_error() {
            Some(libc::EINTR) | Some(libc::EAGAIN) => continue,
            _ => return Err(err),
        }
    }
}

struct Mutex(UnsafeCell<libc::pthread_mutex_t>);

impl Mutex {
    const fn new() -> Self {
        Self(UnsafeCell::new(libc::PTHREAD_MUTEX_INITIALIZER))
    }
    unsafe fn lock(&self) {
        let r = libc::pthread_mutex_lock(self.0.get());
        debug_assert_eq!(r, 0);
    }
    unsafe fn unlock(&self) {
        let r = libc::pthread_mutex_unlock(self.0.get());
        debug_assert_eq!(r, 0);
    }
}

unsafe impl Sync for Mutex {}

struct DropGuard<F: FnMut()>(F);

impl<F: FnMut()> Drop for DropGuard<F> {
    fn drop(&mut self) {
        self.0()
    }
}
