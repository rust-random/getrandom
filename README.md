# getrandom

[![Build Status](https://travis-ci.org/rust-random/getrandom.svg?branch=master)](https://travis-ci.org/rust-random/getrandom)
[![Build Status](https://ci.appveyor.com/api/projects/status/github/rust-random/getrandom?svg=true)](https://ci.appveyor.com/project/rust-random/getrandom)
[![Crate](https://img.shields.io/crates/v/getrandom.svg)](https://crates.io/crates/getrandom)
[![Documentation](https://docs.rs/getrandom/badge.svg)](https://docs.rs/getrandom)
[![Dependency status](https://deps.rs/repo/github/rust-random/getrandom/status.svg)](https://deps.rs/repo/github/rust-random/getrandom)

A Rust library for retrieving random data from (operating) system source. It is
assumed that system always provides high-quality cryptographically secure random
data, ideally backed by hardware entropy sources. This crate derives its name
from Linux's `getrandom` function, but is cross platform, roughly supporting
the same set of platforms as Rust's `std` lib.

This is a low-level API. Most users should prefer using high-level random-number
library like [`rand`].

[`rand`]: https://crates.io/crates/rand

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
getrandom = "0.1"
```

Then invoke the `getrandom` function:

### With Fixed-Sized Arrays

```rust
fn get_random_buf() -> Result<[u8; 32], getrandom::Error> {
    let mut buf = [0u8; 32];
    getrandom::getrandom(&mut buf)?;
    Ok(buf)
}
```

### With Vectors

**Note:** Vectors are not `no_std` supported.

Using a vector, you can implement a function that takes a parameter of how many bytes you would like in return and also implement traits that fixed-sized arrays over a length of 32 elements do not implement (such as Debug).

```rust
use getrandom;

// Function that takes parameter n as the number of bytes requested and returns as a byte-vector
fn os_rand(n: usize) -> Result<Vec<u8>, getrandom:Error> {
    let mut buf: Vec<u8> = vec![0u8; n];
    getrandom::getrandom(&mut buf)?;
    Ok(buf)
}

// How To Use The Function And Return The Vector
fn main(){
    let random_48: Vec<u8> = os_rand(48).unwrap(); // 48 bytes
    let random_24: Vec<u8> = os_rand(24).unwrap(); // 24 bytes
}
```

## Features

This library is `no_std` for every supported target. However, getting randomness
usually requires calling some external system API. This means most platforms
will require linking against system libraries (i.e. `libc` for Unix,
`Advapi32.dll` for Windows, Security framework on iOS, etc...).

The `log` library is supported as an optional dependency. If enabled, error
reporting will be improved on some platforms.

For the `wasm32-unknown-unknown` target, one of the following features should be
enabled:

-   [`wasm-bindgen`](https://crates.io/crates/wasm_bindgen)
-   [`stdweb`](https://crates.io/crates/stdweb)

By default, compiling `getrandom` for an unsupported target will result in
a compilation error. If you want to build an application which uses `getrandom`
for such target, you can either:
- Use [`[replace]`][replace] or [`[patch]`][patch] section in your `Cargo.toml`
to switch to a custom implementation with a support of your target.
- Enable the `dummy` feature to have getrandom use an implementation that always
fails at run-time on unsupported targets.

[replace]: https://doc.rust-lang.org/cargo/reference/manifest.html#the-replace-section
[patch]: https://doc.rust-lang.org/cargo/reference/manifest.html#the-patch-section

## Minimum Supported Rust Version

This crate requires Rust 1.32.0 or later.

# License

The `getrandom` library is distributed under either of

 * [Apache License, Version 2.0](LICENSE-APACHE)
 * [MIT license](LICENSE-MIT)

at your option.
