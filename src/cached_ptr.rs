use core::{
    ffi::c_void,
    ptr::NonNull,
    sync::atomic::{fence, AtomicPtr, Ordering},
};

/// Cached pointer.
///
/// It's intended to be used for weak linking of a C function that may
/// or may not be present at runtime.
///
/// Based off of the DlsymWeak struct in libstd:
/// https://github.com/rust-lang/rust/blob/1.61.0/library/std/src/sys/unix/weak.rs#L84
/// except that the caller must manually cast self.ptr() to a function pointer.
pub struct CachedPtr {
    addr: AtomicPtr<c_void>,
}

impl CachedPtr {
    /// A non-null pointer value which indicates we are uninitialized.
    ///
    /// This constant should ideally not be a valid pointer.
    /// However, if by chance initialization function passed to the `get`
    /// method does return UNINIT, there will not be undefined behavior.
    /// The initialization function will just be called each time `get()`
    /// is called. This would be inefficient, but correct.
    const UNINIT: *mut c_void = !0usize as *mut c_void;

    /// Construct new `CachedPtr` in uninitialized state.
    pub const fn new() -> Self {
        Self {
            addr: AtomicPtr::new(Self::UNINIT),
        }
    }

    /// Return cached address initialized by `f`.
    ///
    /// Multiple callers can call `get` concurrently. It will always return
    /// _some_ value returned by `f`. However, `f` may be called multiple times.
    ///
    /// If cached pointer is null, this method returns `None`.
    pub fn get(&self, f: fn() -> *mut c_void) -> Option<NonNull<c_void>> {
        // Despite having only a single atomic variable (self.addr), we still
        // cannot always use Ordering::Relaxed, as we need to make sure a
        // successful call to `f` is "ordered before" any data read through
        // the returned pointer (which occurs when the function is called).
        // Our implementation mirrors that of the one in libstd, meaning that
        // the use of non-Relaxed operations is probably unnecessary.
        match self.addr.load(Ordering::Relaxed) {
            Self::UNINIT => {
                let addr = f();
                // Synchronizes with the Acquire fence below
                self.addr.store(addr, Ordering::Release);
                NonNull::new(addr)
            }
            addr => {
                let func = NonNull::new(addr)?;
                fence(Ordering::Acquire);
                Some(func)
            }
        }
    }
}
