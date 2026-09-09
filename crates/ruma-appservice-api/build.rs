fn main() {
    // Emitted unconditionally rather than from an environment variable: a
    // consumer building from a git source does not apply `.cargo/config.toml`.
    println!("cargo:rustc-cfg=ruma_unstable_exhaustive_types");
    println!("cargo:rerun-if-changed=build.rs");
}
