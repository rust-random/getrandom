use std::{env, ffi::OsString, process::Command};

/// Minor version of the Rust compiler in which win7 targets were inroduced
const WIN7_INTRODUCED_MINOR_VER: u64 = 78;

/// Tries to get the minor version of use Rust compiler.
///
/// If it fails for any reason, returns `None`.
///
/// Based on the `rustc_version` crate.
fn rustc_minor_version() -> Option<u64> {
    let rustc = env::var_os("RUSTC").unwrap_or_else(|| OsString::from("rustc"));
    let mut cmd = if let Some(wrapper) = env::var_os("RUSTC_WRAPPER").filter(|w| !w.is_empty()) {
        let mut cmd = Command::new(wrapper);
        cmd.arg(rustc);
        cmd
    } else {
        Command::new(rustc)
    };

    let out = cmd.arg("-vV").output().ok()?;

    if !out.status.success() {
        return None;
    }

    let stdout = str::from_utf8(&out.stdout).ok()?;

    // Assumes that the first line contains "rustc 1.xx.0-channel (abcdef 2025-01-01)"
    // where "xx" is the minor version which we want to extract
    let mut lines = stdout.lines();
    let first_line = lines.next()?;
    let minor_ver_str = first_line.split(".").nth(1)?;
    minor_ver_str.parse().ok()
}

fn main() {
    // Automatically detect cfg(sanitize = "memory") even if cfg(sanitize) isn't
    // supported. Build scripts get cfg() info, even if the cfg is unstable.
    println!("cargo:rerun-if-changed=build.rs");
    let santizers = std::env::var("CARGO_CFG_SANITIZE").unwrap_or_default();
    if santizers.contains("memory") {
        println!("cargo:rustc-cfg=getrandom_msan");
    }

    // Use `RtlGenRandom` on older compiler versions since win7 targets
    // were introduced only in Rust 1.78
    let win_legacy = rustc_minor_version()
        .map(|ver| ver < WIN7_INTRODUCED_MINOR_VER)
        .unwrap_or(false);
    if win_legacy {
        println!("cargo:rustc-cfg=getrandom_windows_legacy");
    }
}
