//! Lazily caches a non-null pointer in an `AtomicPtr`.
//!
//! Initialization is intentionally unsynchronized: concurrent callers may race
//! and run `init` more than once. Once a value is produced, it is cached and
//! reused by subsequent calls.
//!
//! For fallible initialization (`try_unsync_init`), only successful values are
//! cached; errors are returned to the caller and are not cached.
//!
//! Uses `Ordering::Relaxed` because this helper only publishes the cached
//! pointer value. Callers must not rely on this mechanism to synchronize
//! unrelated memory side effects performed by `init`.

use core::{
    ptr::{self, NonNull},
    sync::atomic::{AtomicPtr, Ordering},
};

pub(crate) struct LazyPtr<T>(AtomicPtr<T>);

#[allow(
    dead_code,
    reason = "callers typically use only one of `{try_,}unsync_init`"
)]
impl<T> LazyPtr<T> {
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
