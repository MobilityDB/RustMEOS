#![allow(refining_impl_trait)]
use std::{ffi::CString, sync::Once};

pub use meos_sys;
pub mod boxes;
pub mod collections;
pub mod errors;

static START: Once = Once::new();

extern "C" fn finalize() {
    unsafe {
        meos_sys::meos_finalize();
    }
}

pub fn init() {
    START.call_once(|| unsafe {
        let ptr = CString::new("UTC").unwrap();
        meos_sys::meos_initialize(ptr.as_ptr(), None);
        libc::atexit(finalize);
    });
}
