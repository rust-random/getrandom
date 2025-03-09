//! This test forces the Linux fallback path to be taken. This test must be
//! run manually and sequentally:
//!
//!   cargo test --test linux_force_fallback -- --test-threads 1
use std::{
    ffi::c_void,
    fs::{read_dir, DirEntry},
    sync::atomic::{AtomicBool, Ordering},
};

use libc::{__errno_location, c_uint, size_t, ssize_t, ENOSYS};

static FAKE_GETRANDOM_CALLED: AtomicBool = AtomicBool::new(false);

// Override libc::getrandom to simulate failure
#[export_name = "getrandom"]
pub unsafe extern "C" fn fake_getrandom(_: *mut c_void, _: size_t, _: c_uint) -> ssize_t {
    FAKE_GETRANDOM_CALLED.store(true, Ordering::SeqCst);
    unsafe { *__errno_location() = ENOSYS };
    -1
}

mod common;

#[test]
fn fake_getrandom_is_called() {
    assert!(FAKE_GETRANDOM_CALLED.load(Ordering::SeqCst));
}

#[test]
fn dev_urandom_is_open() {
    fn is_urandom(entry: DirEntry) -> bool {
        let path = entry.path().canonicalize().expect("entry is valid");
        path.to_str() == Some("/dev/urandom")
    }
    let mut dir = read_dir("/proc/self/fd").expect("/proc/self exists");
    assert!(dir.any(|path| is_urandom(path.expect("entry exists"))));
}
