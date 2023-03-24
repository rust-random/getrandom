use std::env;
use std::process::Command;
use std::str;

fn main() {
    let minor_ver = rustc_minor().expect("failed to get rustc version");

    if minor_ver >= 40 {
        println!("cargo:rustc-cfg=getrandom_non_exhaustive");
    }
}

// Based on libc's implementation:
// https://github.com/rust-lang/libc/blob/74e81a50c2528b01507e9d03f594949c0f91c817/build.rs#L168-L205
fn rustc_minor() -> Option<u32> {
    let rustc = env::var_os("RUSTC")?;
    let output = Command::new(rustc).arg("--version").output().ok()?.stdout;
    let version = str::from_utf8(&output).ok()?;
    let mut pieces = version.split('.');
    if pieces.next()? != "rustc 1" {
        return None;
    }
    let minor = pieces.next()?;
    minor.parse().ok()
}
