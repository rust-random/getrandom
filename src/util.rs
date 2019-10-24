// Copyright 2019 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use core::sync::atomic::{AtomicUsize, Ordering::Relaxed};

// This structure represents a lazily initialized static usize value. Useful
// when it is preferable to just rerun initialization instead of locking.
// Both unsync_init and sync_init will invoke an init() function until it
// succeeds, then return the cached value for future calls.
//
// Both methods support init() "failing". If the init() method returns UNINIT,
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
// observed and additional synchronization methods with be needed.
pub struct LazyUsize(AtomicUsize);

impl LazyUsize {
    pub const fn new() -> Self {
        Self(AtomicUsize::new(Self::UNINIT))
    }

    // The initialization is not completed.
    pub const UNINIT: usize = usize::max_value();
    // The initialization is currently running.
    pub const ACTIVE: usize = usize::max_value() - 1;

    // Runs the init() function at least once, returning the value of some run
    // of init(). Multiple callers can run their init() functions in parallel.
    // init() should always return the same value, if it succeeds.
    pub fn unsync_init(&self, init: impl FnOnce() -> usize) -> usize {
        // Relaxed ordering is fine, as we only have a single atomic variable.
        let mut val = self.0.load(Relaxed);
        if val == Self::UNINIT {
            val = init();
            self.0.store(val, Relaxed);
        }
        val
    }

    // Synchronously runs the init() function. Only one caller will have their
    // init() function running at a time, and exactly one successful call will
    // be run. init() returning UNINIT or ACTIVE will be considered a failure,
    // and future calls to sync_init will rerun their init() function.
    pub fn sync_init(&self, init: impl FnOnce() -> usize, mut wait: impl FnMut()) -> usize {
        // Common and fast path with no contention. Don't wast time on CAS.
        match self.0.load(Relaxed) {
            Self::UNINIT | Self::ACTIVE => {}
            val => return val,
        }
        // Relaxed ordering is fine, as we only have a single atomic variable.
        loop {
            match self.0.compare_and_swap(Self::UNINIT, Self::ACTIVE, Relaxed) {
                Self::UNINIT => {
                    let val = init();
                    self.0.store(
                        match val {
                            Self::UNINIT | Self::ACTIVE => Self::UNINIT,
                            val => val,
                        },
                        Relaxed,
                    );
                    return val;
                }
                Self::ACTIVE => wait(),
                val => return val,
            }
        }
    }
}

// Identical to LazyUsize except with bool instead of usize.
pub struct LazyBool(LazyUsize);

impl LazyBool {
    pub const fn new() -> Self {
        Self(LazyUsize::new())
    }

    pub fn unsync_init(&self, init: impl FnOnce() -> bool) -> bool {
        self.0.unsync_init(|| init() as usize) != 0
    }
}
