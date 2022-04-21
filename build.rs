use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH");
    let os = env::var("CARGO_CFG_TARGET_OS");
    if arch == Ok("wasm32".into()) && os == Ok("unknown".into()) {
        // Emits virtual feature js which is used to enable
        // JavaScript bindings on wasm32-unknown-unknown
        println!("cargo:rustc-cfg=feature=\"js\"");
    }
}
