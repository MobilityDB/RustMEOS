use std::env;
use std::ffi::OsString;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=MEOS_LIB_DIR");

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
