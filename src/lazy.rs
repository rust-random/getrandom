//! Helpers built around pointer-sized atomics.
#![cfg(target_has_atomic = "ptr")]
#![allow(dead_code)]
use core::{
    ffi::c_void,
    sync::atomic::{AtomicPtr, AtomicUsize, Ordering},
};

// This structure represents a lazily initialized static usize value. Useful
// when it is preferable to just rerun initialization instead of locking.
// unsync_init will invoke an init() function until it succeeds, then return the
// cached value for future calls.
//
// unsync_init supports init() "failing". If the init() method returns UNINIT,
// that value will be returned as normal, but will not be cached.
//
// Users should only depend on the _value_ returned by init() functions.
// Specifically, for the following init() function:
//      fn init() -> usize {
//          a();
//          let v = b();
//          c();
//          v
//      }
// the effects of c() or writes to shared memory will not necessarily be
// observed and additional synchronization methods may be needed.
struct LazyUsize(AtomicUsize);

impl LazyUsize {
    // The initialization is not completed.
    const UNINIT: usize = usize::MAX;

    const fn new() -> Self {
        Self(AtomicUsize::new(Self::UNINIT))
    }

    // Runs the init() function at most once, returning the value of some run of
    // init(). Multiple callers can run their init() functions in parallel.
    // init() should always return the same value, if it succeeds.
    fn unsync_init(&self, init: impl FnOnce() -> usize) -> usize {
        #[cold]
        fn do_init(this: &LazyUsize, init: impl FnOnce() -> usize) -> usize {
            let val = init();
            this.0.store(val, Ordering::Relaxed);
            val
        }

        // Relaxed ordering is fine, as we only have a single atomic variable.
        let val = self.0.load(Ordering::Relaxed);
        if val != Self::UNINIT {
            val
        } else {
            do_init(self, init)
        }
    }
}

// Identical to LazyUsize except with bool instead of usize.
pub(crate) struct LazyBool(LazyUsize);

impl LazyBool {
    pub const fn new() -> Self {
        Self(LazyUsize::new())
    }

    pub fn unsync_init(&self, init: impl FnOnce() -> bool) -> bool {
        self.0.unsync_init(|| usize::from(init())) != 0
    }
}

// This structure represents a lazily initialized static pointer value.
///
/// It's intended to be used for weak linking of a C function that may
/// or may not be present at runtime.
///
/// Based off of the DlsymWeak struct in libstd:
/// https://github.com/rust-lang/rust/blob/1.61.0/library/std/src/sys/unix/weak.rs#L84
/// except that the caller must manually cast self.ptr() to a function pointer.
pub struct LazyPtr {
    addr: AtomicPtr<c_void>,
}

impl LazyPtr {
    /// A non-null pointer value which indicates we are uninitialized.
    ///
    /// This constant should ideally not be a valid pointer. However,
    /// if by chance initialization function passed to the `unsync_init`
    /// method does return UNINIT, there will not be undefined behavior.
    /// The initialization function will just be called each time `get()`
    /// is called. This would be inefficient, but correct.
    const UNINIT: *mut c_void = !0usize as *mut c_void;

    /// Construct new `LazyPtr` in uninitialized state.
    pub const fn new() -> Self {
        Self {
            addr: AtomicPtr::new(Self::UNINIT),
        }
    }

    // Runs the init() function at most once, returning the value of some run of
    // init(). Multiple callers can run their init() functions in parallel.
    // init() should always return the same value, if it succeeds.
    pub fn unsync_init(&self, init: impl Fn() -> *mut c_void) -> *mut c_void {
        #[cold]
        fn do_init(this: &LazyPtr, init: impl Fn() -> *mut c_void) -> *mut c_void {
            let addr = init();
            this.addr.store(addr, Ordering::Release);
            addr
        }

        // Despite having only a single atomic variable (self.addr), we still
        // cannot always use Ordering::Relaxed, as we need to make sure a
        // successful call to `init` is "ordered before" any data read through
        // the returned pointer (which occurs when the function is called).
        // Our implementation mirrors that of the one in libstd, meaning that
        // the use of non-Relaxed operations is probably unnecessary.
        let val = self.addr.load(Ordering::Acquire);
        if val != Self::UNINIT {
            val
        } else {
            do_init(self, init)
        }
    }
}
