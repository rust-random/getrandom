#![cfg(feature = "sys_rng")]

use getrandom::rand_core::{RngCore, TryRngCore};
use getrandom::{SysRng, UnwrappedSysRng};

#[test]
fn test_sys_rng() {
    let x = SysRng.try_next_u64().unwrap();
    let y = SysRng.try_next_u64().unwrap();
    assert!(x != 0);
    assert!(x != y);
}

#[test]
fn test_construction() {
    assert!(SysRng.try_next_u64().unwrap() != 0);
}

#[test]
fn test_unwrapped_sys_rng() {
    let mut buf = [0u8; 128];
    UnwrappedSysRng::default().fill_bytes(&mut buf);
    assert!(buf != [0; 128]);
}
