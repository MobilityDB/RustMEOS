fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let libmeos = cmake::Config::new("source")
        .define("MEOS", "1")
        .very_verbose(true)
        .build();
    println!("cargo:rustc-link-lib=dylib=json-c");
    println!("cargo:lib=meos");
    let search_path = format!("{}/lib", libmeos.display());
    assert!(std::path::Path::new(&search_path).exists());
    println!("cargo:search={}", search_path);
}
