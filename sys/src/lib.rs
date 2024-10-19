//! Low level [MEOS](https://libmeos.org/) C API bindings for MEOS >= 1.1.
//!
//! It provides C-interface as is. If you want to use a more Rust-friendly crate,
//! use the [meos](https://github.com/MobilityDB/meos-rs) crate.

//! You can also find it on [crates.io](https://crates.io/crates/meos).
//!
//! The build will use system-installed MEOS if available.
//!
//! This documentation is generated based on MEOS >= 1.1.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[cfg(all(
    feature = "v1_1",
    not(feature = "v1_2"),
    not(feature = "buildtime_bindgen")
))]
include!("../prebuilt-bindings/meos_1.1.rs");

#[cfg(all(feature = "v1_2", not(feature = "buildtime_bindgen")))]
include!("../prebuilt-bindings/meos_1.2.rs");

#[cfg(feature = "buildtime_bindgen")]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
