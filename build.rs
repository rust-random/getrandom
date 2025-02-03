fn main() {
    // Automatically detect cfg(sanitize = "memory") even if cfg(sanitize) isn't
    // supported. Build scripts get cfg() info, even if the cfg is unstable.
    println!("cargo:rerun-if-changed=build.rs");
    let sanitizers = std::env::var("CARGO_CFG_SANITIZE").unwrap_or_default();
    if sanitizers.contains("memory") {
        println!("cargo:rustc-cfg=getrandom_msan");
    }

    if cfg!(target_feature = "crt-static") {
        match std::process::Command::new(std::env::var_os("RUSTC").unwrap())
            .arg("--target")
            .arg(std::env::var("TARGET").unwrap())
            .arg("--out-dir")
            .arg(std::env::var("OUT_DIR").unwrap())
            .args(["--crate-type=bin", "-"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .spawn()
        {
            Err(err) => {
                println!("cargo:warning=failed to spawn compiler: {}", err);
            }
            Ok(mut child) => {
                use std::io::{BufRead as _, Write as _};

                let std::process::Child { stdin, stderr, .. } = &mut child;
                let mut stdin = stdin.take().unwrap();
                stdin
                    .write_all(
                        r#"
    use std::ffi::{c_uint, c_void};
    extern "C" {
        fn getrandom(buf: *mut c_void, buflen: usize, flags: c_uint) -> isize;
    }
    fn main() -> std::io::Result<()> {
        use std::convert::TryFrom as _;
        let mut buf = [0; 1];
        let _: usize = usize::try_from(unsafe { getrandom(buf.as_mut_ptr().cast(), buf.len(), 0) })
            .map_err(|std::num::TryFromIntError { .. }| std::io::Error::last_os_error())?;
        Ok(())
    }
    "#
                        .as_bytes(),
                    )
                    .unwrap();

                std::mem::drop(stdin); // Send EOF.

                // Trampoline stdout to cargo warnings.
                let stderr = stderr.take().unwrap();
                let stderr = std::io::BufReader::new(stderr);
                for line in stderr.lines() {
                    let line = line.unwrap();
                    println!("cargo:warning={line}");
                }

                let status = child.wait().unwrap();
                if status.code() == Some(0) {
                    println!("cargo:rustc-cfg=has_libc_getrandom");
                }
            }
        }
    }
}
