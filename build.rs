use std::{
    io::{BufRead as _, BufReader, Write as _},
    process::{Child, Stdio},
};

// Automatically detect cfg(sanitize = "memory") even if cfg(sanitize) isn't
// supported. Build scripts get cfg() info, even if the cfg is unstable.
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let santizers = std::env::var("CARGO_CFG_SANITIZE").unwrap_or_default();
    if santizers.contains("memory") {
        println!("cargo:rustc-cfg=getrandom_msan");
    }

    match cc::Build::new()
        .warnings_into_errors(true)
        .try_get_compiler()
    {
        Err(err) => {
            println!("cargo:warning=failed to get compiler: {}", err);
        }
        Ok(compiler) => {
            match compiler
                .to_command()
                .args(["-x", "c", "-", "-o", "-"])
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn()
            {
                Err(err) => {
                    println!("cargo:warning=failed to spawn compiler: {}", err);
                }
                Ok(mut child) => {
                    let Child { stdin, stderr, .. } = &mut child;
                    let mut stdin = stdin.take().unwrap();
                    stdin
                        .write_all(
                            r#"#include <sys/random.h>
int main() {
    char buf[1];
    return getrandom(buf, sizeof(buf), 0);
}"#
                            .as_bytes(),
                        )
                        .unwrap();
                    std::mem::drop(stdin); // Send EOF.

                    // Trampoline stdout to cargo warnings.
                    let stderr = stderr.take().unwrap();
                    let stderr = BufReader::new(stderr);
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
}
