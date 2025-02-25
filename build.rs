// Automatically detect cfg(sanitize = "memory") even if cfg(sanitize) isn't
// supported. Build scripts get cfg() info, even if the cfg is unstable.
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let santizers = std::env::var("CARGO_CFG_SANITIZE").unwrap_or_default();
    if santizers.contains("memory") {
        println!("cargo:rustc-cfg=getrandom_msan");
    }

    // Declare valida as a valid value for getrandom_backend cfg
    println!("cargo:rustc-check-cfg=cfg(getrandom_backend, values(\"valida\"))");
}
