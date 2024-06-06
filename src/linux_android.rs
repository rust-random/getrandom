//! Implementation for Linux / Android without `/dev/urandom` fallback
use crate::{util_unix::sys_fill_exact, Error};
use core::mem::MaybeUninit;

pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    sys_fill_exact(dest, getrandom_syscall)
}

// The value of `EINTR` is not architecture-specific. It is checked against
// `libc::EINTR` by linux_android_with_fallback.rs.
pub const EINTR: i32 = 4;

// Also used by linux_android_with_fallback to check if the syscall is available.
cfg_if! {
    // TODO: Expand inilne assembly to other architectures to avoid depending
    // on libc on Linux.
    if #[cfg(all(target_arch = "x86_64", target_pointer_width = "64"))] {
        type Word = u64;
        type IWord = i64;

        // TODO(MSRV(1.78 feature(target_abi))): Enable this and remove `target_pointer_width`
        // restriction above.
        //
        // #[cfg(target_abi = "x32")]
        // const __X32_SYSCALL_BIT: Word = 0x40000000;
        //
        // #[cfg(target_abi = "x832")]
        // #[allow(non_upper_case_globals)]
        // pub const SYS_getrandom: IWord = 318 | __X32_SYSCALL_BIT;

        // #[cfg(not(target_abi = "x832"))]
        #[allow(non_upper_case_globals)]
        pub const SYS_getrandom: IWord = 318;

        pub fn getrandom_syscall(buf: &mut [MaybeUninit<u8>]) -> Result<usize, Error> {
            // Clamp request length to word size; no-op on regular (non-x32) x86_64.
            assert!(core::mem::size_of::<usize>() <= core::mem::size_of::<Word>());
            let buflen: Word = buf.len() as Word;
            let mut ret: IWord;
            let flags = 0;
            unsafe {
                core::arch::asm!(
                    "syscall",
                    inout("rax") SYS_getrandom => ret,
                    in("rdi") buf.as_mut_ptr(),
                    in("rsi") buflen,
                    in("rdx") flags,
                    lateout("rcx") _,
                    lateout("r11") _,
                    options(nostack),
                );
            }
            match Word::try_from(ret) {
                Ok(written) => {
                    // `buflen` can from a usize and the return value won't be
                    // larger than what we requested (otherwise that would be a
                    // buffer overflow), so this cast is lossless even if
                    // `usize` is smaller.
                    Ok(written as usize)
                },
                Err(_) => {
                    Err(u32::try_from(ret.unsigned_abs()).map_or(
                        Error::UNEXPECTED, Error::from_os_error))
                }
            }
        }
    } else {
        use crate::util_libc::last_os_error;
        pub use libc::SYS_getrandom;

        pub fn getrandom_syscall(buf: &mut [MaybeUninit<u8>]) -> Result<usize, Error> {
            let ret: libc::c_long = unsafe {
                libc::syscall(
                    SYS_getrandom,
                    buf.as_mut_ptr().cast::<core::ffi::c_void>(),
                    buf.len(),
                    0,
                )
            };
            const _:() = assert!(core::mem::size_of::<libc::c_long>() == core::mem::size_of::<isize>());
            usize::try_from(ret as isize).map_err(|_| last_os_error())
        }
    }
}
