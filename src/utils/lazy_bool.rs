//! Lazily caches a `bool` in an `AtomicU8`.
//!
//! Initialization is intentionally unsynchronized: concurrent callers may race
//! and run `init` more than once. Once a value is produced, it is cached and
//! reused by subsequent calls.
//!
//! Uses `Ordering::Relaxed` because this helper only publishes the cached
//! value itself.

use core::sync::atomic::{AtomicU8, Ordering};

pub(crate) struct LazyBool(AtomicU8);

impl LazyBool {
    const UNINIT: u8 = 2;

    pub const fn new() -> Self {
        Self(AtomicU8::new(Self::UNINIT))
    }

    #[cold]
    fn do_init(&self, init: impl FnOnce() -> bool) -> bool {
        let val = u8::from(init());
        self.0.store(val, Ordering::Relaxed);
        val != 0
    }

    #[inline]
    pub fn unsync_init(&self, init: impl FnOnce() -> bool) -> bool {
        let val = self.0.load(Ordering::Relaxed);
        if val != Self::UNINIT {
            val != 0
        } else {
            self.do_init(init)
        }
    }
}
