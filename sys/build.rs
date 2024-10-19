use std::env;
use std::ffi::OsString;

const MINIMUM_MEOS_VERSION: &str = "1.1.0";

/// Detect MEOS config parameters using pkg-config (not available for all MEOS
/// versions)
fn detect_meos_via_pkg_config() -> Result<pkg_config::Library, pkg_config::Error> {
    use pkg_config::Config;

    Config::new()
        .atleast_version(MINIMUM_MEOS_VERSION)
        .probe("meos")
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // If `bundled` is on, use the git submodule to build MEOS from scratch
    let include_path = if cfg!(feature = "bundled") {
        // Defined in `meos-src` build.rs file
        let meos_path = std::env::var("DEP_MEOSSRC_SEARCH").unwrap();

        // Tell cargo to tell rustc to link the system meos shared library.
        println!("cargo:rustc-link-search={meos_path}");
        println!("cargo:rustc-link-lib=meos");

        meos_path
    } else {
        let pk = detect_meos_via_pkg_config();

        match pk {
            Ok(meos) => meos.include_paths[0].clone().display().to_string(),
            Err(pkg_config_err) => {
                if matches!(pkg_config_err, pkg_config::Error::Command { cause, .. } if cause.kind() == std::io::ErrorKind::NotFound)
                {
                    panic!("Could not find `pkg-config` in your path. Please install it before running meos-sys-bind.");
                }
                // As a fallback, since pkg-config was not configured in meos until 1.2, we will try a default path.
                if cfg!(feature = "v1_1") {
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
                    panic!("Could not detect MEOS using pkg-config.");
                }
            }
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
    let out_path = std::path::PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_path.join("bindings.rs"))?;

    Ok(())
}
