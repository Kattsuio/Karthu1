fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").map(|s| s == "windows").unwrap_or(false) {
        let d = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        println!("cargo:rustc-link-arg-bins={}/res/karthus.res", d);
        println!("cargo:rerun-if-changed=res/karthus.res");
    }
}
