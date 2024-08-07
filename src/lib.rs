#![allow(refining_impl_trait)]
use std::{
    ffi::{CStr, CString},
    fmt::Debug,
    sync::Once,
};

use bitmask_enum::bitmask;
use boxes::r#box::Box as MeosBox;
use collections::base::collection::Collection;
pub use meos_sys;

pub mod boxes;
pub mod collections;
pub mod errors;
pub mod temporal;
pub(crate) mod utils;

static START: Once = Once::new();

extern "C" fn finalize() {
    unsafe {
        meos_sys::meos_finalize();
    }
}

pub trait BoundingBox: Collection {}

impl<T> BoundingBox for T where T: MeosBox {}

unsafe extern "C" fn error_handler(_error_level: i32, _error_code: i32, message: *const i8) {
    let message = CStr::from_ptr(message).to_str().unwrap();
    panic!("{}", message);
}

pub fn init() {
    START.call_once(|| unsafe {
        let ptr = CString::new("UTC").unwrap();
        meos_sys::meos_initialize(ptr.as_ptr(), Some(error_handler));
        libc::atexit(finalize);
    });
}

fn factory<T: MeosEnum>(temporal: *const meos_sys::Temporal) -> T {
    let temporal_type: TemporalSubtype = unsafe { (temporal.read().subtype as u32).into() };
    match temporal_type {
        TemporalSubtype::Instant => T::from_instant(temporal as *const _),
        TemporalSubtype::Sequence => T::from_sequence(temporal as *const _),
        TemporalSubtype::SequenceSet => T::from_sequence_set(temporal as *const _),
        _ => unreachable!(),
    }
}

#[bitmask(u8)]
pub enum WKBVariant {
    /// Little endian encoding
    NDR = meos_sys::WKB_NDR as u8,
    /// Big endian encoding
    XDR = meos_sys::WKB_XDR as u8,
    /// Extended variant
    Extended = meos_sys::WKB_EXTENDED as u8,
}

#[derive(Debug, PartialEq)]
pub enum TemporalSubtype {
    Any = meos_sys::tempSubtype_ANYTEMPSUBTYPE as isize,
    Instant = meos_sys::tempSubtype_TINSTANT as isize,
    Sequence = meos_sys::tempSubtype_TSEQUENCE as isize,
    SequenceSet = meos_sys::tempSubtype_TSEQUENCESET as isize,
}

impl From<u32> for TemporalSubtype {
    fn from(value: u32) -> Self {
        match value {
            meos_sys::tempSubtype_ANYTEMPSUBTYPE => TemporalSubtype::Any,
            meos_sys::tempSubtype_TINSTANT => TemporalSubtype::Instant,
            meos_sys::tempSubtype_TSEQUENCE => TemporalSubtype::Sequence,
            meos_sys::tempSubtype_TSEQUENCESET => TemporalSubtype::SequenceSet,
            _ => TemporalSubtype::Any, // default case, as it's often the case for "unknown" or "any"
        }
    }
}

pub trait MeosEnum: Debug {
    fn from_instant(inner: *const meos_sys::TInstant) -> Self;
    fn from_sequence(inner: *const meos_sys::TSequence) -> Self;
    fn from_sequence_set(inner: *const meos_sys::TSequenceSet) -> Self;

    fn inner(&self) -> *const meos_sys::Temporal;
}
