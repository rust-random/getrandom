//! Lazy caches backed by a single atomic value.
//!
//! Each cache starts in an "uninitialized" sentinel state. Initialization is
//! intentionally unsynchronized: concurrent callers may race and run `init`
//! more than once. Once a non-sentinel value is produced, it is cached and
//! reused by subsequent calls.
//!
//! For fallible initialization (`try_unsync_init`), only successful values are
//! cached; errors are returned to the caller and are not cached.
//!
//! These helpers use `Ordering::Relaxed` because they are only intended to
//! publish the cached value itself. Callers must not rely on this mechanism to
//! synchronize unrelated memory side effects performed by `init`.

#![allow(dead_code)]

use core::{
    ptr::{self, NonNull},
    sync::atomic::{AtomicPtr, AtomicUsize, Ordering},
};

pub(crate) struct LazyNonNull<T>(AtomicPtr<T>);

impl<T> LazyNonNull<T> {
    pub const fn new() -> Self {
        Self(AtomicPtr::new(ptr::null_mut()))
    }

    #[cold]
    fn do_init(&self, init: impl FnOnce() -> NonNull<T>) -> NonNull<T> {
        let val = init();
        self.0.store(val.as_ptr(), Ordering::Relaxed);
        val
    }

    #[cold]
    fn try_do_init<E>(
        &self,
        init: impl FnOnce() -> Result<NonNull<T>, E>,
    ) -> Result<NonNull<T>, E> {
        let val = init()?;
        self.0.store(val.as_ptr(), Ordering::Relaxed);
        Ok(val)
    }

    #[inline]
    pub fn unsync_init(&self, init: impl FnOnce() -> NonNull<T>) -> NonNull<T> {
        match NonNull::new(self.0.load(Ordering::Relaxed)) {
            Some(val) => val,
            None => self.do_init(init),
        }
    }

    #[inline]
    pub fn try_unsync_init<E>(
        &self,
        init: impl FnOnce() -> Result<NonNull<T>, E>,
    ) -> Result<NonNull<T>, E> {
        match NonNull::new(self.0.load(Ordering::Relaxed)) {
            Some(val) => Ok(val),
            None => self.try_do_init(init),
        }
    }
}

pub(crate) struct LazyBool(AtomicUsize);

impl LazyBool {
    const UNINIT: usize = usize::MAX;

    pub const fn new() -> Self {
        Self(AtomicUsize::new(Self::UNINIT))
    }

    #[cold]
    fn do_init(&self, init: impl FnOnce() -> bool) -> bool {
        let val = usize::from(init());
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
