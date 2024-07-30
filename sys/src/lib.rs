//! Low level [MEOS](https://libmeos.org/) C API bindings for MEOS >= 1.1.
//!
//! It provides C-interface as is. If you want to use a more Rust-friendly crate,
//! use the [meos](https://github.com/MobilityDB/RustMEOS) crate.

//! You can also find it on [crates.io](https://crates.io/crates/meos).
//!
//! The build will use system-installed MEOS if available.
//!
//! This documentation is generated based on MEOS 1.1.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!("../prebuilt-bindings/meos_1.1.rs");
