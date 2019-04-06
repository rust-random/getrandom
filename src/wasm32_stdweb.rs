// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for WASM via stdweb
use std::num::NonZeroU32;
use std::sync::atomic::{AtomicUsize, Ordering};

use stdweb::{js, _js_impl};
use stdweb::unstable::TryInto;
use stdweb::web::error::Error as WebError;

use crate::Error;

// replace with AtomicU8 on stabilization and MSRV bump
static RNG_STATE: AtomicUsize = AtomicUsize::new(0);

const STATE_INIT_DONE: usize = 1 << 0;
const STATE_USE_BROWSER: usize = 1 << 1;

pub fn getrandom_inner(dest: &mut [u8]) -> Result<(), Error> {
    let state = RNG_STATE.load(Ordering::Acquire);
    let use_browser = if state & STATE_INIT_DONE != 0 {
        state & STATE_USE_BROWSER != 0
    } else {
        let use_browser = getrandom_init()?;
        RNG_STATE.store(
            if use_browser {
                STATE_INIT_DONE | STATE_USE_BROWSER
            } else {
                STATE_INIT_DONE
            },
            Ordering::Release,
        );
        use_browser
    };
    getrandom_fill(use_browser, dest)
}

fn getrandom_init() -> Result<bool, Error> {
    let result = js! {
        try {
            if (
                typeof self === "object" &&
                typeof self.crypto === "object" &&
                typeof self.crypto.getRandomValues === "function"
            ) {
                return { success: true, ty: 1 };
            }

            if (typeof require("crypto").randomBytes === "function") {
                return { success: true, ty: 2 };
            }

            return { success: false, error: new Error("not supported") };
        } catch(err) {
            return { success: false, error: err };
        }
    };

    if js!{ return @{ result.as_ref() }.success } == true {
        let ty = js!{ return @{ result }.ty };

        if ty == 1 { Ok(true) }
        else if ty == 2 { Ok(false) }
        else { unreachable!() }
    } else {
        let err: WebError = js!{ return @{ result }.error }.try_into().unwrap();
        error!("getrandom unavailable: {}", err);
        Err(Error::UNAVAILABLE)
    }
}

fn getrandom_fill(use_browser: bool, dest: &mut [u8]) -> Result<(), Error> {
    for chunk in dest.chunks_mut(65536) {
        let len = chunk.len() as u32;
        let ptr = chunk.as_mut_ptr() as i32;

        let result = match use_browser {
            true => js! {
                try {
                    let array = new Uint8Array(@{ len });
                    self.crypto.getRandomValues(array);
                    HEAPU8.set(array, @{ ptr });

                    return { success: true };
                } catch(err) {
                    return { success: false, error: err };
                }
            },
            false => js! {
                try {
                    let bytes = require("crypto").randomBytes(@{ len });
                    HEAPU8.set(new Uint8Array(bytes), @{ ptr });

                    return { success: true };
                } catch(err) {
                    return { success: false, error: err };
                }
            }
        };

        if js!{ return @{ result.as_ref() }.success } != true {
            let err: WebError = js!{ return @{ result }.error }.try_into().unwrap();
            error!("getrandom failed: {}", err);
            return Err(Error::UNKNOWN)
        }
    }
    Ok(())
}

#[inline(always)]
pub fn error_msg_inner(_: NonZeroU32) -> Option<&'static str> { None }
