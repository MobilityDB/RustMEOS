# meos-sys

Low level [MEOS](https://libmeos.org/) C API bindings for MEOS.

It provides C-interface as is. If you want to use a more Rust-friendly crate,
use the [meos](https://github.com/MobilityDB/RustMEOS) crate.

You can also find it on [crates.io](https://crates.io/crates/meos).

## Version policy
Currently the bindings are generated manually using bindgen. The commit of the repository currently tracked is [this one](https://github.com/MobilityDB/MobilityDB/tree/226bfec72644031f66d77eb09093d71c76efd97f)

## Build

By default, the build will use system-installed MEOS if available, `pkg-config` is used to automatically detect MEOS

If MEOS is in a custom location, you can instead use the `MEOS_LIB_DIR` environment variable to
configure MEOS detection.

If `MEOS_LIB_DIR` is not also in your system's standard dynamic library search
path, you may need to add it to the dynamic library search path before
running the tests or executable produced by `cargo build`.

Linux:

```bash
LD_LIBRARY_PATH=<path to MEOS>/lib MEOS_LIB_DIR=<path to MEOS>/lib MEOS_VERSION=<version> cargo test

```

MacOS:

```bash
DYLD_FALLBACK_LIBRARY_PATH=<path to MEOS>/lib MEOS_LIB_DIR=<path to MEOS>/lib MEOS_VERSION=<version> cargo test

```

## Bindings

Pre-built bindings are available for 1.1