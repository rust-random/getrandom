// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use std::cell::RefCell;
use std::ops::DerefMut;
use std::io;

/// If `f` contains `Some(T)` call `use_f` using contents of `f` as an argument,
/// otherwise initialize `f` value using `init_f`, store resulting value in `f`
/// and call `use_f`.
pub(crate) fn use_init<T, F, U>(f: &RefCell<Option<T>>, init_f: F, mut use_f: U)
    -> io::Result<()>
    where F: FnOnce() -> io::Result<T>, U: FnMut(&mut T) -> io::Result<()>
{
    let mut f = f.borrow_mut();
    let f: &mut Option<T> = f.deref_mut();
    match f {
        None => *f = Some(init_f()?),
        _ => (),
    }

    match f {
        Some(f) => use_f(f),
        None => unreachable!(),
    }
}
