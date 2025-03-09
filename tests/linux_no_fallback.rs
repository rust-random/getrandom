//! This test ensures the Linux fallback path is not taken. As this test will
//! fail in some Linux configurations, it must be run manually and sequentally:
//!
//!   cargo test --test linux_no_fallback -- --test-threads 1
use std::fs::{read_dir, DirEntry};

mod common;

#[test]
fn dev_urandom_is_not_open() {
    fn is_urandom(entry: DirEntry) -> bool {
        let path = entry.path().canonicalize().expect("entry is valid");
        path.to_str() == Some("/dev/urandom")
    }
    let mut dir = read_dir("/proc/self/fd").expect("/proc/self exists");
    assert!(dir.all(|path| !is_urandom(path.expect("entry exists"))));
}
