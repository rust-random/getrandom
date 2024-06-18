//! Implementations that just need to read from a file

extern crate std;

use crate::{util_libc::sys_fill_exact, Error};
use core::{ffi::c_void, mem::MaybeUninit};
use std::{fs::File, io, os::unix::io::AsRawFd as _, sync::OnceLock};

/// For all platforms, we use `/dev/urandom` rather than `/dev/random`.
/// For more information see the linked man pages in lib.rs.
///   - On Linux, "/dev/urandom is preferred and sufficient in all use cases".
///   - On Redox, only /dev/urandom is provided.
///   - On AIX, /dev/urandom will "provide cryptographically secure output".
///   - On Haiku and QNX Neutrino they are identical.
const FILE_PATH: &str = "/dev/urandom";

// Do not inline this when it is the fallback implementation, but don't mark it
// `#[cold]` because it is hot when it is actually used.
#[cfg_attr(any(target_os = "android", target_os = "linux"), inline(never))]
pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    // The opening of the file, by libc/libstd/etc. may write some unknown
    // state into in-process memory. (Such state may include some sanitizer
    // bookkeeping, or we might be operating in a unikernal-like environment
    // where all the "kernel" file descriptor bookkeeping is done in our
    // process.) Thus we avoid using (relaxed) atomics like we use in other
    // parts of the library.
    //
    // We prevent multiple threads from opening file descriptors concurrently,
    // which could run into the limit on the number of open file descriptors.
    // Our goal is to have no more than one file descriptor open, ever.
    //
    // We assume any call to `OnceLock::get_or_try_init` synchronizes-with
    // (Ordering::Acquire) the preceding call to `OnceLock::get_or_try_init`
    // after `init()` returns an `Ok` result (Ordering::Release). See
    // https://github.com/rust-lang/rust/issues/126239.
    static FILE: OnceLock<File> = OnceLock::new();
    let file = FILE.get_or_try_init(init)?;

    // TODO(MSRV feature(read_buf)): Use `std::io::Read::read_buf`
    sys_fill_exact(dest, |buf| unsafe {
        libc::read(
            file.as_raw_fd(),
            buf.as_mut_ptr().cast::<c_void>(),
            buf.len(),
        )
    })
}

#[cold]
fn init() -> Result<File, Error> {
    // On Linux, /dev/urandom might return insecure values.
    #[cfg(any(target_os = "android", target_os = "linux"))]
    wait_until_rng_ready()?;

    File::open(FILE_PATH).map_err(map_io_error)
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
    let file = File::open("/dev/random").map_err(map_io_error)?;
    let mut pfd = libc::pollfd {
        fd: file.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };

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

fn map_io_error(err: io::Error) -> Error {
    // TODO(MSRV feature(raw_os_error_ty)): Use `std::io::RawOsError`.
    type RawOsError = i32;

    err.raw_os_error()
        .map_or(Error::UNEXPECTED, |errno: RawOsError| {
            // RawOsError-to-u32 conversion is lossless for nonnegative values
            // if they are the same size.
            const _: () =
                assert!(core::mem::size_of::<RawOsError>() == core::mem::size_of::<u32>());

            match u32::try_from(errno) {
                Ok(code) if code != 0 => Error::from_os_error(code),
                _ => Error::ERRNO_NOT_POSITIVE,
            }
        })
}
