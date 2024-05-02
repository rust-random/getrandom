//! Implementation using libc::getrandom
//!
//! Available since:
//!   - Linux Kernel 3.17, Glibc 2.25, Musl 1.1.20
//!   - Android API level 23 (Marshmallow)
//!   - NetBSD 10.0
//!   - FreeBSD 12.0
//!   - Solaris 11.3
//!   - Illumos since Dec 2018
//!   - DragonFly 5.7
//!   - Hurd Glibc 2.31
//!   - shim-3ds since Feb 2022
//!
//! For all platforms, we use the default randomness source (the one used
//! by /dev/urandom) rather than the /dev/random (GRND_RANDOM) source. For
//! more information see the linked man pages in lib.rs.
//!   - On Linux, "/dev/urandom is preferred and sufficient in all use cases".
//!   - On NetBSD, "there is no reason to ever use" GRND_RANDOM.
//!   - On Illumos, the default source is used for getentropy() and the like:
//!     https://github.com/illumos/illumos-gate/blob/89cf0c2ce8a47dcf555bb1596f9034f07b9467fa/usr/src/lib/libc/port/gen/getentropy.c#L33
//!   - On Solaris, both sources use FIPS 140-2 / NIST SP-900-90A DRBGs, see:
//!     https://blogs.oracle.com/solaris/post/solaris-new-system-calls-getentropy2-and-getrandom2
//!   - On Redox, only /dev/urandom is provided.
//!   - On AIX, /dev/urandom will "provide cryptographically secure output".
//!   - On Haiku, QNX Neutrino, DragonFly, and FreeBSD, they are identical.
use crate::{util_libc::sys_fill_exact, Error};
use core::mem::MaybeUninit;

// On Solaris 11.3, getrandom() will fail if bufsz > 1024 (bufsz > 133120 on Solaris 11.4).
// This issue is not present in Illumos's implementation of getrandom().
#[cfg(target_os = "solaris")]
const MAX_BYTES: usize = 1024;
#[cfg(not(target_os = "solaris"))]
const MAX_BYTES: usize = usize::MAX;

pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    for chunk in dest.chunks_mut(MAX_BYTES) {
        sys_fill_exact(chunk, |buf| unsafe {
            libc::getrandom(buf.as_mut_ptr() as *mut libc::c_void, buf.len(), 0)
        })?;
    }
    Ok(())
}
