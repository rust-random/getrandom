// Copyright 2019 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use core::sync::atomic::{AtomicUsize, Ordering};

// This structure represents a laziliy initialized static usize value. Useful
// when it is perferable to just rerun initialization instead of locking.
pub struct LazyUsize(AtomicUsize);

impl LazyUsize {
    pub const fn new() -> Self {
        Self(AtomicUsize::new(usize::max_value()))
    }

    // Runs the init() function at least once, returning the value of some run
    // of init(). Unlike std::sync::Once, the init() function may be run
    // multiple times. If init() returns usize::max_value(), the init() function
    // will always be retried on a future call to unsync_init(). This makes it
    // ideal for representing failure.
    pub fn unsync_init(&self, init: impl FnOnce() -> usize) -> usize {
        // Relaxed ordering is fine, as we only have a single atomic variable.
        if self.0.load(Ordering::Relaxed) == usize::max_value() {
            self.0.store(init(), Ordering::Relaxed)
        }
        self.0.load(Ordering::Relaxed)
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
