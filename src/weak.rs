use core::{
    ffi::c_void,
    ptr::NonNull,
    sync::atomic::{fence, AtomicPtr, Ordering},
};

// A "weak" binding to a C function that may or may not be present at runtime.
// Used for supporting newer OS features while still building on older systems.
// Based off of the DlsymWeak struct in libstd:
// https://github.com/rust-lang/rust/blob/1.61.0/library/std/src/sys/unix/weak.rs#L84
// except that the caller must manually cast self.ptr() to a function pointer.
pub struct Weak {
    addr: AtomicPtr<c_void>,
    link_fn: fn() -> *mut c_void,
}

impl Weak {
    // A non-null pointer value which indicates we are uninitialized. This
    // constant should ideally not be a valid address of a function pointer.
    // However, if by chance libc::dlsym does return UNINIT, there will not
    // be undefined behavior. libc::dlsym will just be called each time ptr()
    // is called. This would be inefficient, but correct.
    // TODO: Replace with core::ptr::invalid_mut(1) when that is stable.
    const UNINIT: *mut c_void = 1 as *mut c_void;

    // Construct a weak binding a C function.
    pub const fn new(link_fn: fn() -> *mut c_void) -> Self {
        Self {
            addr: AtomicPtr::new(Self::UNINIT),
            link_fn,
        }
    }

    // Return the address of a function if present at runtime. Otherwise,
    // return None. Multiple callers can call ptr() concurrently. It will
    // always return _some_ value returned by libc::dlsym. However, the
    // dlsym function may be called multiple times.
    pub fn ptr(&self) -> Option<NonNull<c_void>> {
        // Despite having only a single atomic variable (self.addr), we still
        // cannot always use Ordering::Relaxed, as we need to make sure a
        // successful call to dlsym() is "ordered before" any data read through
        // the returned pointer (which occurs when the function is called).
        // Our implementation mirrors that of the one in libstd, meaning that
        // the use of non-Relaxed operations is probably unnecessary.
        match self.addr.load(Ordering::Relaxed) {
            Self::UNINIT => {
                let addr = (self.link_fn)();
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
