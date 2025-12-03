//! Helpers built around pointer-sized atomics.
use core::{
    ptr,
    sync::atomic::{AtomicPtr, AtomicUsize, Ordering},
};

macro_rules! lazy_atomic {
    ($name:ident $(<$($gen:ident),+>)?, $atomic:ty, $value:ty, $uninit:expr) => {
        /// Lazily initialized static value backed by a single atomic.
        ///
        /// `unsync_init` will invoke `init` until it returns a value other than
        /// the sentinel `UNINIT`, then cache that value for subsequent calls.
        /// Multiple callers may race to run `init`; only the returned value is
        /// guaranteed to be observed, not any side effects.
        pub(crate) struct $name$(<$($gen),+>)?($atomic);

        impl<$($($gen),+)? > $name$(<$($gen),+>)? {
            const UNINIT: $value = $uninit;

            pub const fn new() -> Self {
                Self(<$atomic>::new(Self::UNINIT))
            }

            #[cold]
            fn do_init(&self, init: impl FnOnce() -> $value) -> $value {
                let val = init();
                self.0.store(val, Ordering::Relaxed);
                val
            }

            #[inline]
            pub fn unsync_init(&self, init: impl FnOnce() -> $value) -> $value {
                // Relaxed ordering is fine, as we only have a single atomic variable.
                let val = self.0.load(Ordering::Relaxed);
                if val != Self::UNINIT {
                    val
                } else {
                    self.do_init(init)
                }
            }
        }
    };
}

lazy_atomic!(LazyUsize, AtomicUsize, usize, usize::MAX);
lazy_atomic!(LazyPtr<T>, AtomicPtr<T>, *mut T, ptr::dangling_mut());

/// Lazily initializes a cached bool; reuses `LazyUsize` to avoid sentinel
/// issues with `AtomicBool`.
pub(crate) struct LazyBool(LazyUsize);

impl LazyBool {
    pub const fn new() -> Self {
        Self(LazyUsize::new())
    }

    pub fn unsync_init(&self, init: impl FnOnce() -> bool) -> bool {
        self.0.unsync_init(|| usize::from(init())) != 0
    }
}
