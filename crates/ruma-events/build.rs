use std::env;

fn main() {
    // Set the `ruma_identifiers_storage` configuration from an environment variable.
    if let Ok(value) = env::var("RUMA_IDENTIFIERS_STORAGE") {
        println!("cargo:rustc-cfg=ruma_identifiers_storage={value:?}");
    }

    println!("cargo:rerun-if-env-changed=RUMA_IDENTIFIERS_STORAGE");

    // Emitted unconditionally rather than from an environment variable: a
    // consumer building from a git source does not apply `.cargo/config.toml`.
    println!("cargo:rustc-cfg=ruma_unstable_exhaustive_types");
}
