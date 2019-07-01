// Copyright 2019 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::util::LazyUsize;
use core::marker::PhantomData;
use core::mem;

// A "weak" binding to a C function that may or may not be present at runtime.
// Used for supporting newer OS features while still building on older systems.
// F must be a function pointer of type `unsafe extern "C" fn`. Based off of the
// weak! macro in libstd.
pub struct Weak<F> {
    name: &'static str,
    addr: LazyUsize,
    _marker: PhantomData<F>,
}

impl<F> Weak<F> {
    // Construct a binding to a C function with a given name. This function is
    // unsafe because `name` _must_ be null terminated, and if the symbol is
    // present at runtime it _must_ have type F.
    pub const unsafe fn new(name: &'static str) -> Self {
        Self {
            name,
            addr: LazyUsize::new(),
            _marker: PhantomData,
        }
    }

    // Returns the function pointer if it is present at runtime. Otherwise,
    // return None, indicating the function does not exist.
    pub fn func(&self) -> Option<F> {
        assert_eq!(mem::size_of::<Option<F>>(), mem::size_of::<usize>());

        let addr = self.addr.unsync_init(|| unsafe {
            libc::dlsym(libc::RTLD_DEFAULT, self.name.as_ptr() as *const _) as usize
        });
        unsafe { mem::transmute_copy(&addr) }
    }
}
