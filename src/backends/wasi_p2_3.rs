//! Implementation for WASIp2 and WASIp3.
use crate::Error;
use core::{mem::MaybeUninit, ptr::copy_nonoverlapping};

#[cfg(target_env = "p2")]
mod p2 {
    include!("./wasi_p2_3/p2/imports.rs");
    #[inline(never)]
    pub fn _link_custom_section_describing_imports() {}
}
#[cfg(target_env = "p2")]
use p2::*;

// Workaround to silence `unexpected_cfgs` warning
// on Rust version between 1.85 and 1.91
#[cfg(not(target_env = "p2"))]
#[cfg(target_env = "p3")]
mod p3 {
    include!("./wasi_p2_3/p3/imports.rs");
    #[inline(never)]
    pub fn _link_custom_section_describing_imports() {}
}
#[cfg(not(target_env = "p2"))]
#[cfg(target_env = "p3")]
use p3::*;

#[cfg(not(target_env = "p2"))]
#[cfg(not(target_env = "p3"))]
compile_error!("Unknown version of WASI (only previews 1, 2 and 3 are supported)");

// This is a bit subtle, in addition to the `include!` above, but the general
// idea is that `wit-bindgen` generates a custom section of type information
// needed by `wasm-component-ld`. That needs to make its way to the linker and
// that's particularly tricky to do unfortunately. To force the linker to at
// least witness the object file with the custom section in it a `#[used]`
// static here refers to a function in the `p2`/`p3` modules which is adjacent
// to the custom section. This is then coupled with a lack of `#[inline]` below
// such that whenever this file itself is used it'll force the linker to look at
// this `#[used]` static, then look at the type section, and include that.
//
// Note that the linker will strip this static as well as the referred-to
// function as it's actually dead, but the linker will still preserve the type
// information since it's in a custom section, which is what we want.
#[used]
static _FORCE_SECTION_REF: fn() = _link_custom_section_describing_imports;

use wasi::random::random::get_random_u64;

pub fn inner_u32() -> Result<u32, Error> {
    let val = get_random_u64();
    Ok(crate::util::truncate(val))
}

pub fn inner_u64() -> Result<u64, Error> {
    Ok(get_random_u64())
}

pub fn fill_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    let (prefix, chunks, suffix) = unsafe { dest.align_to_mut::<MaybeUninit<u64>>() };

    // We use `get_random_u64` instead of `get_random_bytes` because the latter creates
    // an allocation due to the Wit IDL [restrictions][0]. This should be fine since
    // the main use case of `getrandom` is seed generation.
    //
    // [0]: https://github.com/WebAssembly/wasi-random/issues/27
    if !prefix.is_empty() {
        let val = get_random_u64();
        let src = (&val as *const u64).cast();
        unsafe {
            copy_nonoverlapping(src, prefix.as_mut_ptr(), prefix.len());
        }
    }

    for dst in chunks {
        dst.write(get_random_u64());
    }

    if !suffix.is_empty() {
        let val = get_random_u64();
        let src = (&val as *const u64).cast();
        unsafe {
            copy_nonoverlapping(src, suffix.as_mut_ptr(), suffix.len());
        }
    }

    Ok(())
}
