// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// Call was interrupted.
    ///
    /// Typically it can be retried.
    Interrupted,
    /// RNG source is unavailable on a given system.
    Unavailable,
    /// Unknown error.
    Unknown,
    #[doc(hidden)]
    __Nonexhaustive,
}
