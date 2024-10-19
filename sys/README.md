# meos-sys

Low level [MEOS](https://libmeos.org/) C API bindings for MEOS.

It provides C-interface as is. If you want to use a more Rust-friendly crate,
use the [meos](https://github.com/MobilityDB/meos-rs) crate.

You can also find it on [crates.io](https://crates.io/crates/meos).

## Build

The build by default will use system-installed MEOS, `pkg-config` is used to automatically detect MEOS

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

## Binding generation
By default, meos-sys will use the pregenerated bindings for the 1.2 version, the 1.1 ones is also available. Alternatively, you can generate your own bindings from your `libmeos` installation by specifying the `bindgen` feature.
