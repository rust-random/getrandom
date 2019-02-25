# Rand

[![Build Status](https://travis-ci.org/rust-random/getrandom.svg?branch=master)](https://travis-ci.org/rust-random/getrandom)
[![Build Status](https://ci.appveyor.com/api/projects/status/github/rust-random/getrandom?svg=true)](https://ci.appveyor.com/project/rust-random/getrandom)
[![Crate](https://img.shields.io/crates/v/getrandom.svg)](https://crates.io/crates/getrandom)
[![API](https://docs.rs/getrandom/badge.svg)](https://docs.rs/getrandom)

A Rust library to securely get random entropy. This crate derives its name from
Linux's `getrandom` function, but is cross platform, roughly supporting the same
set of platforms as Rust's `std` lib.

This is a low-level API. Most users should prefer a high-level random-number
library like [Rand] or a cryptography library.

[Rand]: https://crates.io/crates/rand


## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
getrandom = "0.1"
```

Then invoke the `getrandom` function:

```rust
fn get_random_buf() -> Result<[u8; 32], getrandom::Error> {
    let mut buf = [0u8; 32];
    getrandom::getrandom(&mut buf)?;
    buf
}
```

## Features

This library is `no_std` compatible on SGX but requires `std` on most platforms.

For WebAssembly (`wasm32`), Enscripten targets are supported directly; otherwise
one of the following features must be enabled:

-   [`wasm-bindgen`](https://crates.io/crates/wasm_bindgen)
-   [`stdweb`](https://crates.io/crates/stdweb)

## Versions

This crate requires Rustc version 1.28.0 or later due to usage of `NonZeroU32`.


# License

The `getrandom` library is distributed under the terms of both the MIT license
and the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT), and
[COPYRIGHT](COPYRIGHT) for details.
