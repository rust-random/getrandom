// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for WASM via stdweb
use core::mem;
use core::num::NonZeroU32;

use stdweb::unstable::TryInto;
use stdweb::web::error::Error as WebError;
use stdweb::{_js_impl, js};

use crate::Error;
use std::sync::Once;

#[derive(Clone, Copy, Debug)]
enum RngSource {
    Browser,
    Node,
}

pub fn getrandom_inner(dest: &mut [u8]) -> Result<(), Error> {
    assert_eq!(mem::size_of::<usize>(), 4);
    static ONCE: Once = Once::new();
    static mut RNG_SOURCE: Result<RngSource, Error> = Err(Error::UNAVAILABLE);

    // SAFETY: RNG_SOURCE is only written once, before being read.
    ONCE.call_once(|| unsafe {
        RNG_SOURCE = getrandom_init();
    });
    getrandom_fill(unsafe { RNG_SOURCE }?, dest)
}

fn getrandom_init() -> Result<RngSource, Error> {
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

    if js! { return @{ result.as_ref() }.success } == true {
        let ty = js! { return @{ result }.ty };

        if ty == 1 {
            Ok(RngSource::Browser)
        } else if ty == 2 {
            Ok(RngSource::Node)
        } else {
            unreachable!()
        }
    } else {
        let err: WebError = js! { return @{ result }.error }.try_into().unwrap();
        error!("getrandom unavailable: {}", err);
        Err(Error::UNAVAILABLE)
    }
}

fn getrandom_fill(source: RngSource, dest: &mut [u8]) -> Result<(), Error> {
    for chunk in dest.chunks_mut(65536) {
        let len = chunk.len() as u32;
        let ptr = chunk.as_mut_ptr() as i32;

        let result = match source {
            RngSource::Browser => js! {
                try {
                    let array = new Uint8Array(@{ len });
                    self.crypto.getRandomValues(array);
                    HEAPU8.set(array, @{ ptr });

                    return { success: true };
                } catch(err) {
                    return { success: false, error: err };
                }
            },
            RngSource::Node => js! {
                try {
                    let bytes = require("crypto").randomBytes(@{ len });
                    HEAPU8.set(new Uint8Array(bytes), @{ ptr });

                    return { success: true };
                } catch(err) {
                    return { success: false, error: err };
                }
            },
        };

        if js! { return @{ result.as_ref() }.success } != true {
            let err: WebError = js! { return @{ result }.error }.try_into().unwrap();
            error!("getrandom failed: {}", err);
            return Err(Error::UNKNOWN);
        }
    }
    Ok(())
}

#[inline(always)]
pub fn error_msg_inner(_: NonZeroU32) -> Option<&'static str> {
    None
}
