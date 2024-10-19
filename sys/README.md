# meos-sys

Low level [MEOS](https://libmeos.org/) C API bindings for MEOS.

It provides C-interface as is. If you want to use a more Rust-friendly crate,
use the [meos](https://github.com/MobilityDB/meos-rs) crate.

You can also find it on [crates.io](https://crates.io/crates/meos).

Versions >= 1.1 are supported.

## Usage
You need to select as features what version do you want to obtain the bindings from (`v1_1`, `v1_2`), or alternatively, whether you want to build MEOS from scratch (`bundled`). This will mean adding to your `Cargo.toml`:
```toml
# Cargo.toml
[dependencies]
meos-sys = { version = "0.1.8", features = ["v1_2"] }
```

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

You can also enable the `bundled` feature to build MEOS from scratch. Note you will still have to have installed in your system GEOS, PROJ and JSON-C libraries. You can install all of them by running the following command in your (Debian based) machine:
```bash
sudo apt-get install libgeos-dev proj-bin libproj-dev proj-data libjson-c-dev
```

## Binding generation
The 1.2 and 1.1 versions are already available as prebuilt bindings. Alternatively, you can generate your own bindings from your `libmeos` installation by specifying the `bindgen` feature.

