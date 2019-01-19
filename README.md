# Rand

[![Build Status](https://travis-ci.org/rust-random/getrandom.svg?branch=master)](https://travis-ci.org/rust-random/getrandom)
[![Build Status](https://ci.appveyor.com/api/projects/status/github/rust-random/getrandom?svg=true)](https://ci.appveyor.com/project/rust-random/getrandom)
[![Crate](https://img.shields.io/crates/v/getrandom.svg)](https://crates.io/crates/getrandom)
[![API](https://docs.rs/getrandom/badge.svg)](https://docs.rs/getrandom)

A Rust library to securely get random entropy. This crate derives its name from
Linux's `getrandom` function, but is cross platform, roughly supporting the same
set of platforms as Rust's `std` lib.


## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
getrandom = "0.1"
```

TODO


# License

The `getrandom` library is distributed under the terms of both the MIT license
and the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT), and
[COPYRIGHT](COPYRIGHT) for details.
