#![cfg(feature = "sys_rng")]

use core::array::from_fn;
use getrandom::{
    SysRng, UnwrappingSysRng,
    rand_core::{RngCore, TryRngCore},
};

const N: usize = 32;

#[test]
fn test_sys_rng() {
    let x: [u64; N] = from_fn(|_| SysRng.try_next_u64().unwrap());
    let y: [u64; N] = from_fn(|_| SysRng.try_next_u64().unwrap());
    assert!(x.iter().all(|&val| val != 0));
    assert!(y.iter().all(|&val| val != 0));
    assert!(x != y);

    let x: [u32; N] = from_fn(|_| SysRng.try_next_u32().unwrap());
    let y: [u32; N] = from_fn(|_| SysRng.try_next_u32().unwrap());
    assert!(x.iter().all(|&val| val != 0));
    assert!(y.iter().all(|&val| val != 0));
    assert!(x != y);

    let mut x = [0u8; N];
    SysRng.try_fill_bytes(&mut x).unwrap();
    let mut y = [0u8; N];
    SysRng.try_fill_bytes(&mut y).unwrap();

    assert_ne!(x, [0; N]);
    assert_ne!(y, [0; N]);
    assert!(x != y);
}

#[test]
fn test_unwrapping_sys_rng() {
    let x: [u64; N] = from_fn(|_| UnwrappingSysRng.next_u64());
    let y: [u64; N] = from_fn(|_| UnwrappingSysRng.next_u64());
    assert!(x.iter().all(|&val| val != 0));
    assert!(y.iter().all(|&val| val != 0));
    assert!(x != y);

    let x: [u32; N] = from_fn(|_| UnwrappingSysRng.next_u32());
    let y: [u32; N] = from_fn(|_| UnwrappingSysRng.next_u32());
    assert!(x.iter().all(|&val| val != 0));
    assert!(y.iter().all(|&val| val != 0));
    assert!(x != y);

    let mut x = [0u8; N];
    UnwrappingSysRng.fill_bytes(&mut x);
    let mut y = [0u8; N];
    UnwrappingSysRng.fill_bytes(&mut y);

    assert_ne!(x, [0; N]);
    assert_ne!(y, [0; N]);
    assert!(x != y);
}
