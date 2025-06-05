#![no_std]
#![no_main]

#[panic_handler]
fn handle(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

use getrandom::{Backend, Error};

/// This implementation fills using a constant value of 4.
///
/// WARNING: this custom implementation is for testing purposes ONLY!
struct ConstantBackend;

unsafe impl Backend for ConstantBackend {
    #[inline]
    fn fill_uninit(dest: &mut [core::mem::MaybeUninit<u8>]) -> Result<(), Error> {
        for out in dest {
            // Chosen by fair dice roll
            out.write(4);
        }

        Ok(())
    }
}

#[cfg(feature = "fallback")]
const _: () = {
    getrandom::set_backend!(ConstantBackend);
};

// This second call will cause the following compile-time error:
/*
error: symbol `__getrandom_v03_fallback_fill_uninit` is already defined
  --> src\main.rs:43:1
   |
43 | getrandom::set_backend!(ConstantBackend);
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: this error originates in the macro `getrandom::set_backend` (in Nightly builds, run with -Z macro-backtrace for more info)

error: could not compile `fallback_test` (bin "fallback_test") due to 1 previous error
*/
#[cfg(feature = "fail-double-definition")]
const _: () = {
    getrandom::set_backend!(ConstantBackend);
};

#[no_mangle]
pub extern "C" fn _start() {
    let mut dest = [0u8; 13];
    let result = getrandom::fill(&mut dest);
    assert!(result.is_ok());
    assert_eq!(dest, [4; 13]);
}
