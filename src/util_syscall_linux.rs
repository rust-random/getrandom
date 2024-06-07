// Support for raw system calls on Linux.
//
// # Sanitizers
//
// Currently only Memory Sanitizer is actively supported.
//
// TODO: Support address sanitizer, in particular in `pre_write_range`.
//
// ## Memory Sanitizer
//
// See https://github.com/llvm/llvm-project/commit/ac9ee01fcbfac745aaedca0393a8e1c8a33acd8d:
// LLVM uses:
// ```c
//   COMMON_INTERCEPTOR_ENTER(ctx, getrandom, buf, buflen, flags);
//   SSIZE_T n = REAL(getrandom)(buf, buflen, flags);
//   if (n > 0) {
//     COMMON_INTERCEPTOR_WRITE_RANGE(ctx, buf, n);
//   }
// ```
// and:
// ```c
// #define PRE_SYSCALL(name) \
//     SANITIZER_INTERFACE_ATTRIBUTE void __sanitizer_syscall_pre_impl_##name
// #define PRE_WRITE(p, s) COMMON_SYSCALL_PRE_WRITE_RANGE(p, s)
// #define POST_WRITE(p, s) COMMON_SYSCALL_POST_WRITE_RANGE(p, s)
// PRE_SYSCALL(getrandom)(void *buf, uptr count, long flags) {
//   if (buf) {
//     PRE_WRITE(buf, count);
//   }
// }
//
// POST_SYSCALL(getrandom)(long res, void *buf, uptr count, long flags) {
//   if (res > 0 && buf) {
//     POST_WRITE(buf, res);
//   }
// }
// ```

use core::mem::MaybeUninit;

// MSAN defines:
//
// ```c
// #define COMMON_INTERCEPTOR_ENTER(ctx, func, ...)              \
//   if (msan_init_is_running)                                   \
//     return REAL(func)(__VA_ARGS__);                           \
//   ENSURE_MSAN_INITED();                                       \
//   MSanInterceptorContext msan_ctx = {IsInInterceptorScope()}; \
//   ctx = (void *)&msan_ctx;                                    \
//   (void)ctx;                                                  \
//   InterceptorScope interceptor_scope;                         \
//   __msan_unpoison(__errno_location(), sizeof(int));
// ```
//
// * We assume that memory sanitizer will not use the this crate during the
//   initialization of msan, so we don't have to worry about
//   `msan_init_is_running`.
// * We assume that rustc/LLVM initializes MSAN before executing any Rust code,
//   so we don't need to call `ENSURE_MSAN_INITED`.
// * Notice that `COMMON_INTERCEPTOR_WRITE_RANGE` doesn't use `ctx`, which
//   means it is oblivious to `IsInInterceptorScope()`, so we don't have to
//   call it. More generally, we don't have to worry about interceptor scopes
//   because we are not an interceptor.
// * We don't read from `__errno_location()` so we don't need to unpoison it.
//
// Consequently, MSAN's `COMMON_INTERCEPTOR_ENTER` is a no-op.
//
// MSAN defines:
// ```c
// #define COMMON_SYSCALL_PRE_WRITE_RANGE(p, s) \
//   do {                                       \
//   } while (false)
// ```
// So MSAN's PRE_SYSCALL hook is also a no-op.
//
// Consequently, we have nothing to do before invoking the syscall unless/until
// we support other sanitizers like ASAN.
#[allow(unused_variables)]
pub fn pre_write_range(_ptr: *mut MaybeUninit<u8>, _size: usize) {}

// MSNA defines:
// ```c
// #define COMMON_INTERCEPTOR_WRITE_RANGE(ctx, ptr, size) \
//    __msan_unpoison(ptr, size)
// ```
// and:
// ```c
// #define COMMON_SYSCALL_POST_WRITE_RANGE(p, s) __msan_unpoison(p, s)
// ```
#[allow(unused_variables)]
pub unsafe fn post_write_range(ptr: *mut MaybeUninit<u8>, size: usize) {
    #[cfg(feature = "unstable-sanitize")]
    {
        #[cfg(sanitize = "memory")]
        {
        }
    }
}
