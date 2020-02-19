// We only test the CPU-based RNG source on supported architectures.
#![cfg(target_arch = "x86_64")]

#[path = "rdrand.rs"]
mod rdrand;
use rdrand::getrandom_inner as getrandom;
#[path = "test_common.rs"]
mod test_common;
