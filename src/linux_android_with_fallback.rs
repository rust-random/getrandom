//! Implementation for Linux / Android with `/dev/urandom` fallback
use crate::{lazy::LazyBool, linux_android, use_file, Error};
use core::mem::MaybeUninit;

const _: () = assert!(linux_android::EINTR == libc::EINTR);
const _: () = assert!(linux_android::SYS_getrandom == libc::SYS_getrandom);

pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    // getrandom(2) was introduced in Linux 3.17
    static HAS_GETRANDOM: LazyBool = LazyBool::new();
    if HAS_GETRANDOM.unsync_init(is_getrandom_available) {
        linux_android::getrandom_inner(dest)
    } else {
        use_file::getrandom_inner(dest)
    }
}

fn is_getrandom_available() -> bool {
    match linux_android::getrandom_syscall(&mut []) {
        Err(err) if err.raw_os_error() == Some(libc::ENOSYS) => false, // No kernel support
        // The fallback on EPERM is intentionally not done on Android since this workaround
        // seems to be needed only for specific Linux-based products that aren't based
        // on Android. See https://github.com/rust-random/getrandom/issues/229.
        #[cfg(target_os = "linux")]
        Err(err) if err.raw_os_error() == Some(libc::EPERM) => false, // Blocked by seccomp
        _ => true,
    }
}
