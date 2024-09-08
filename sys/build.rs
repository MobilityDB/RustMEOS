use std::env;
use std::ffi::OsString;

/// Detect MEOS config parameters using pkg-config (not available for all GEOS
/// versions)
fn detect_meos_via_pkg_config() -> bool {
    use pkg_config::Config;

    let meos_pkg_config = Config::new().probe("meos");

    meos_pkg_config.is_ok()
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=MEOS_LIB_DIR");

    let success = detect_meos_via_pkg_config();

    // Try to find the library manually
    if !success {
        let lib_dir_env = env::var_os("MEOS_LIB_DIR")
            .map(OsString::into_string)
            .transpose()
            .ok()
            .flatten()
            .unwrap_or(String::from("/usr/local/lib/"));

        // Tell cargo to look for shared libraries in the specified directory
        println!("cargo:rustc-link-search={lib_dir_env}");

        // Tell cargo to tell rustc to link the system meos shared library.
        println!("cargo:rustc-link-lib=dylib=meos");
    }
}
