//! Implementation for Linux / Android with `/dev/urandom` fallback
use crate::{
    lazy::LazyBool,
    linux_android,
    util_libc::{last_os_error, open_readonly, sys_fill_exact},
    Error,
};
use core::{
    mem::MaybeUninit,
    sync::atomic::{AtomicI32, Ordering},
};

pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    // getrandom(2) was introduced in Linux 3.17
    static HAS_GETRANDOM: LazyBool = LazyBool::new();
    if HAS_GETRANDOM.unsync_init(is_getrandom_available) {
        linux_android::getrandom_inner(dest)
    } else {
        use_file(dest)
    }
}

fn is_getrandom_available() -> bool {
    if linux_android::getrandom_syscall(&mut []) < 0 {
        match last_os_error().raw_os_error() {
            Some(libc::ENOSYS) => false, // No kernel support
            // The fallback on EPERM is intentionally not done on Android since this workaround
            // seems to be needed only for specific Linux-based products that aren't based
            // on Android. See https://github.com/rust-random/getrandom/issues/229.
            #[cfg(target_os = "linux")]
            Some(libc::EPERM) => false, // Blocked by seccomp
            _ => true,
        }
    } else {
        true
    }
}

// File descriptor is a "nonnegative integer" as per `open` man.
const FD_UNINIT: libc::c_int = -1;
const FD_ONGOING_INIT: libc::c_int = -2;

// See comment for `FD` in use_file.rs
static FD: AtomicI32 = AtomicI32::new(FD_UNINIT);

pub fn use_file(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    let mut fd = FD.load(Ordering::Acquire);
    if fd == FD_UNINIT || fd == FD_ONGOING_INIT {
        fd = open_or_wait()?;
    }
    sys_fill_exact(dest, |buf| unsafe {
        libc::read(fd, buf.as_mut_ptr().cast(), buf.len())
    })
}

#[cold]
pub(super) fn open_or_wait() -> Result<libc::c_int, Error> {
    loop {
        match FD.load(Ordering::Acquire) {
            FD_UNINIT => {
                let res = FD.compare_exchange_weak(
                    FD_UNINIT,
                    FD_ONGOING_INIT,
                    Ordering::AcqRel,
                    Ordering::Relaxed,
                );
                if res.is_ok() {
                    break;
                }
            }
            FD_ONGOING_INIT => futex_wait(),
            fd => return Ok(fd),
        }
    }

    let res = open_fd();
    let val = match res {
        Ok(fd) => fd,
        Err(_) => FD_UNINIT,
    };
    FD.store(val, Ordering::Release);
    futex_wake();
    res
}

fn futex_wait() {
    let op = libc::FUTEX_WAIT | libc::FUTEX_PRIVATE_FLAG;
    let timeout_ptr = core::ptr::null::<libc::timespec>();
    let ret = unsafe { libc::syscall(libc::SYS_futex, &FD, op, FD_ONGOING_INIT, timeout_ptr) };
    // FUTEX_WAIT should return either 0 or EAGAIN error
    debug_assert!({
        match ret {
            0 => true,
            -1 => last_os_error().raw_os_error() == Some(libc::EAGAIN),
            _ => false,
        }
    });
}

fn futex_wake() {
    let op = libc::FUTEX_WAKE | libc::FUTEX_PRIVATE_FLAG;
    let ret = unsafe { libc::syscall(libc::SYS_futex, &FD, op, libc::INT_MAX) };
    debug_assert!(ret >= 0);
}

fn open_fd() -> Result<libc::c_int, Error> {
    wait_until_rng_ready()?;
    // "/dev/urandom is preferred and sufficient in all use cases"
    let fd = open_readonly(b"/dev/urandom\0")?;
    debug_assert!(fd >= 0);
    Ok(fd)
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
fn wait_until_rng_ready() -> Result<(), Error> {
    let fd = open_readonly(b"/dev/random\0")?;
    let mut pfd = libc::pollfd {
        fd,
        events: libc::POLLIN,
        revents: 0,
    };

    let res = loop {
        // A negative timeout means an infinite timeout.
        let res = unsafe { libc::poll(&mut pfd, 1, -1) };
        if res >= 0 {
            // We only used one fd, and cannot timeout.
            debug_assert_eq!(res, 1);
            break Ok(());
        }
        let err = last_os_error();
        match err.raw_os_error() {
            Some(libc::EINTR) | Some(libc::EAGAIN) => continue,
            _ => break Err(err),
        }
    };
    unsafe { libc::close(fd) };
    res
}
