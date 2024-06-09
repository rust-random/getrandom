//! Utilities for using rustix.
//!
//! At this point in time it is only used on Linux-like operating systems.

use crate::Error;
use core::convert::TryInto;
use core::mem::MaybeUninit;
use core::num::NonZeroU32;

use rustix::fd::OwnedFd;
use rustix::fs;
use rustix::io::Errno;
use rustix::rand;

/// Convert a Rustix error to one of our errors.
pub(crate) fn cvt(err: Errno) -> Error {
    match TryInto::<u32>::try_into(err.raw_os_error())
        .ok()
        .and_then(NonZeroU32::new)
    {
        Some(code) => Error::from(code),
        None => Error::ERRNO_NOT_POSITIVE,
    }
}

/// Fill a buffer by repeatedly invoking a `rustix` call.
pub(crate) fn sys_fill_exact(
    mut buf: &mut [MaybeUninit<u8>],
    fill: impl Fn(&mut [MaybeUninit<u8>]) -> Result<(&mut [u8], &mut [MaybeUninit<u8>]), Errno>,
) -> Result<(), Error> {
    while !buf.is_empty() {
        // Read into the buffer.
        match fill(buf) {
            Err(err) => return Err(cvt(err)),
            Ok((_filled, unfilled)) => {
                buf = unfilled;
            }
        }
    }

    Ok(())
}

/// Open a file as read-only.
pub(crate) fn open_readonly(path: &str) -> Result<OwnedFd, Error> {
    loop {
        match fs::open(
            path,
            fs::OFlags::CLOEXEC | fs::OFlags::RDONLY,
            fs::Mode::empty(),
        ) {
            Ok(file) => return Ok(file),
            Err(Errno::INTR) => continue,
            Err(err) => return Err(cvt(err)),
        }
    }
}

pub(crate) fn getrandom_syscall(
    buf: &mut [MaybeUninit<u8>],
) -> Result<(&mut [u8], &mut [MaybeUninit<u8>]), Errno> {
    rand::getrandom_uninit(buf, rand::GetRandomFlags::empty())
}

// The mutex isn't used in all Linux implementations.
#[allow(dead_code)]
mod mutex_impl {
    //! The following is derived from Rust's
    //! library/std/src/sys/unix/locks/futex_mutex.rs at revision
    //! 98815742cf2e914ee0d7142a02322cf939c47834.
    //! Also partially based on the rustix_futex_sync crate.

    use core::sync::atomic::{AtomicU32, Ordering};
    use rustix::thread::{FutexFlags, FutexOperation};

    pub(crate) struct Mutex {
        futex: AtomicU32,
    }

    const UNLOCKED: u32 = 0;
    const LOCKED: u32 = 1;
    const CONTENDED: u32 = 2;

    impl Mutex {
        pub(crate) const fn new() -> Self {
            Self {
                futex: AtomicU32::new(UNLOCKED),
            }
        }

        // This function is safe and is only unsafe for consistency with util_libc.rs
        pub(crate) unsafe fn lock(&self) {
            if self
                .futex
                .compare_exchange(UNLOCKED, LOCKED, Ordering::Acquire, Ordering::Relaxed)
                .is_err()
            {
                self.lock_contended();
            }
        }

        #[cold]
        fn lock_contended(&self) {
            // Spin first to speed things up if the lock is released quickly.
            let mut state = self.spin();

            // If it's unlocked now, attempt to take the lock
            // without marking it as contended.
            if state == UNLOCKED {
                match self.futex.compare_exchange(
                    UNLOCKED,
                    LOCKED,
                    Ordering::Acquire,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => return, // Locked!
                    Err(s) => state = s,
                }
            }

            loop {
                // Put the lock in contended state.
                // We avoid an unnecessary write if it as already set to 2,
                // to be friendlier for the caches.
                if state != CONTENDED && self.futex.swap(CONTENDED, Ordering::Acquire) == 0 {
                    // We changed it from 0 to 2, so we just successfully locked it.
                    return;
                }

                // Wait for the futex to change state, assuming it is still 2.
                futex_wait(&self.futex, CONTENDED);

                // Spin again after waking up.
                state = self.spin();
            }
        }

        /// Production-grade mutexes usually spin for a little to alleviate short-term contention.
        fn spin(&self) -> u32 {
            let mut spin = 100;

            loop {
                // We only use `load` (and not `swap` or `compare_exchange`)
                // while spinning, to be easier on the caches.
                let state = self.futex.load(Ordering::Relaxed);

                // We stop spinning when the mutex is unlocked (0),
                // but also when it's contended (2).
                if state != LOCKED || spin == 0 {
                    return state;
                }

                core::hint::spin_loop();
                spin -= 1;
            }
        }

        #[inline]
        pub unsafe fn unlock(&self) {
            if self.futex.swap(UNLOCKED, Ordering::Release) == CONTENDED {
                // We only wake up one thread. When that thread locks the mutex, it
                // will mark the mutex as contended (2) (see lock_contended above),
                // which makes sure that any other waiting threads will also be
                // woken up eventually.
                futex_wake(&self.futex);
            }
        }
    }

    /// Wait on a futex.
    pub fn futex_wait(futex: &AtomicU32, expected: u32) -> bool {
        use core::ptr::{null, null_mut};
        use core::sync::atomic::Ordering::Relaxed;

        loop {
            // No need to wait if the value already changed.
            if futex.load(Relaxed) != expected {
                return true;
            }

            let r = unsafe {
                // Use FUTEX_WAIT_BITSET rather than FUTEX_WAIT to be able to give an
                // absolute time rather than a relative time.
                rustix::thread::futex(
                    futex.as_ptr(),
                    FutexOperation::WaitBitset,
                    FutexFlags::PRIVATE,
                    expected,
                    null(),
                    null_mut(),
                    !0u32, // A full bitmask, to make it behave like a regular FUTEX_WAIT.
                )
            };

            match r {
                Err(rustix::io::Errno::TIMEDOUT) => return false,
                Err(rustix::io::Errno::INTR) => continue,
                _ => return true,
            }
        }
    }

    /// Wake up one thread blocked on futex_wait.
    ///
    /// Returns true if a thread was actually woken up.
    fn futex_wake(futex: &AtomicU32) -> bool {
        use core::ptr::{null, null_mut};

        unsafe {
            match rustix::thread::futex(
                futex.as_ptr(),
                FutexOperation::Wake,
                FutexFlags::PRIVATE,
                1,
                null(),
                null_mut(),
                0,
            ) {
                Err(_) | Ok(0) => false,
                _ => true,
            }
        }
    }
}

#[allow(unused)]
pub(crate) use mutex_impl::Mutex;
