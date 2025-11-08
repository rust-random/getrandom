// WASI preview 2 requires enabled std
#![cfg_attr(not(all(target_arch = "wasm32", target_env = "p2")), no_std)]

#[cfg(not(any(test, all(target_arch = "wasm32", target_env = "p2"))))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    extern "C" {
        fn panic_nonexistent() -> !;
    }
    unsafe { panic_nonexistent() }
}

#[unsafe(no_mangle)]
pub extern "C" fn getrandom_wrapper(buf_ptr: *mut u8, buf_len: usize) -> u32 {
    let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr.cast(), buf_len) };
    let res = getrandom::fill_uninit(buf).map(|_| ());
    unsafe { core::mem::transmute(res) }
}

#[cfg(getrandom_backend = "custom")]
#[unsafe(no_mangle)]
unsafe extern "Rust" fn __getrandom_v03_custom(
    dest: *mut u8,
    len: usize,
) -> Result<(), getrandom::Error> {
    for i in 0..len {
        core::ptr::write(dest.add(i), i as u8);
    }
    Ok(())
}
