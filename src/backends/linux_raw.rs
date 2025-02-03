//! Implementation for Linux / Android using `asm!`-based syscalls.
use crate::{Error, MaybeUninit};

pub use crate::util::{inner_u32, inner_u64};

#[cfg(not(any(target_os = "android", target_os = "linux")))]
compile_error!("`linux_raw` backend can be enabled only for Linux/Android targets!");

#[allow(non_upper_case_globals)]
unsafe fn getrandom_syscall(buf: *mut u8, buflen: usize, flags: u32) -> isize {
    let r0;

    // Based on `rustix` and `linux-raw-sys` code.
    cfg_if! {
        if #[cfg(target_arch = "arm")] {
            const __NR_getrandom: u32 = 384;
            // In thumb-mode, r7 is the frame pointer and is not permitted to be used in
            // an inline asm operand, so we have to use a different register and copy it
            // into r7 inside the inline asm.
            // Theoretically, we could detect thumb mode in the build script, but several
            // register moves are cheap enough compared to the syscall cost, so we do not
            // bother with it.
            core::arch::asm!(
                "mov {tmp}, r7",
                "mov r7, {nr}",
                "svc 0",
                "mov r7, {tmp}",
                nr = const __NR_getrandom,
                tmp = out(reg) _,
                inlateout("r0") buf => r0,
                in("r1") buflen,
                in("r2") flags,
                options(nostack, preserves_flags)
            );
        } else if #[cfg(target_arch = "aarch64")] {
            const __NR_getrandom: u32 = 278;
            core::arch::asm!(
                "svc 0",
                in("x8") __NR_getrandom,
                inlateout("x0") buf => r0,
                in("x1") buflen,
                in("x2") flags,
                options(nostack, preserves_flags)
            );
        } else if #[cfg(target_arch = "loongarch64")] {
            const __NR_getrandom: u32 = 278;
            core::arch::asm!(
                "syscall 0",
                in("$a7") __NR_getrandom,
                inlateout("$a0") buf => r0,
                in("$a1") buflen,
                in("$a2") flags,
                options(nostack, preserves_flags)
            );
        } else if #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))] {
            const __NR_getrandom: u32 = 278;
            core::arch::asm!(
                "ecall",
                in("a7") __NR_getrandom,
                inlateout("a0") buf => r0,
                in("a1") buflen,
                in("a2") flags,
                options(nostack, preserves_flags)
            );
        } else if #[cfg(target_arch = "x86")] {
            const __NR_getrandom: isize = 355;
            // `int 0x80` is famously slow, but implementing vDSO is too complex
            // and `sysenter`/`syscall` have their own portability issues,
            // so we use the simple "legacy" way of doing syscalls.
            core::arch::asm!(
                "int $$0x80",
                inlateout("eax") __NR_getrandom => r0,
                in("ebx") buf,
                in("ecx") buflen,
                in("edx") flags,
                options(nostack, preserves_flags)
            );
        } else if #[cfg(target_arch = "x86_64")] {
            #[cfg(target_pointer_width = "64")]
            const __NR_getrandom: isize = 318;
            #[cfg(target_pointer_width = "32")]
            const __NR_getrandom: isize = 1073742142;

            core::arch::asm!(
                "syscall",
                inlateout("rax") __NR_getrandom => r0,
                in("rdi") buf,
                in("rsi") buflen,
                in("rdx") flags,
                lateout("rcx") _,
                lateout("r11") _,
                options(nostack, preserves_flags)
            );
        } else {
            compile_error!("`linux_raw` backend does not support this target arch");
        }
    }

    r0
}

pub fn fill_inner(mut dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    // Value of this error code is stable across all target arches.
    const EINTR: isize = -4;

    loop {
        let ret = unsafe { getrandom_syscall(dest.as_mut_ptr().cast(), dest.len(), 0) };
        match usize::try_from(ret) {
            Ok(0) => return Err(Error::UNEXPECTED),
            Ok(len) => {
                dest = dest.get_mut(len..).ok_or(Error::UNEXPECTED)?;
                if dest.is_empty() {
                    return Ok(());
                }
            }
            Err(_) if ret == EINTR => continue,
            Err(_) => {
                let code: u32 = ret
                    .wrapping_neg()
                    .try_into()
                    .map_err(|_| Error::UNEXPECTED)?;
                return Err(Error::from_os_error(code));
            }
        }
    }
}
