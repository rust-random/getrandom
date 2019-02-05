use std::cell::RefCell;
use std::ops::DerefMut;
use std::{io, hint};

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
        None => unsafe { hint::unreachable_unchecked() },
    }
}

