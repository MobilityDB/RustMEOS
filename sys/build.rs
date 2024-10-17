use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

/// Detect MEOS config parameters using pkg-config (not available for all GEOS
/// versions)
fn detect_meos_via_pkg_config() -> Option<PathBuf> {
    use pkg_config::Config;

    let meos_pkg_config = Config::new().probe("meos");

    meos_pkg_config.map(|pk| pk.include_paths[0].clone()).ok()
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=MEOS_LIB_DIR");

    // If `bundled_proj` is on, use the git submodule to build MEOS from scratch
    let include_path = if cfg!(feature = "bundled_proj") {
        let meos_path = std::env::var("DEP_MEOSSRC_SEARCH").unwrap();

        println!("cargo:rustc-link-search=dylib={}", meos_path);

        // Tell cargo to tell rustc to link the system meos shared library.
        println!("cargo:rustc-link-lib=meos");

        meos_path
    // Else use pkg-config, using a default as a fallback
    } else {
        let pk_include_path = detect_meos_via_pkg_config();

        // Try to find the library manually
        if !pk_include_path.is_some() {
            let default_include_path = String::from("/usr/local/lib/");
            let lib_dir_env = env::var_os("MEOS_LIB_DIR")
                .map(OsString::into_string)
                .transpose()
                .ok()
                .flatten()
                .unwrap_or(default_include_path.clone());

            // Tell cargo to look for shared libraries in the specified directory
            println!("cargo:rustc-link-search={lib_dir_env}");

            // Tell cargo to tell rustc to link the system meos shared library.
            println!("cargo:rustc-link-lib=dylib=meos");
            default_include_path
        } else {
            pk_include_path.unwrap().display().to_string()
        }
    };

    #[cfg(feature = "buildtime_bindgen")]
    generate_bindings(include_path.into()).unwrap();

    #[cfg(not(feature = "buildtime_bindgen"))]
    let _ = include_path;
}

#[cfg(feature = "buildtime_bindgen")]
fn generate_bindings(include_path: std::path::PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        .clang_arg(format!("-I{}", include_path.to_string_lossy()))
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_path.join("bindings.rs"))?;

    Ok(())
}
