#![crate_name = "meos"]
#![crate_type = "lib"]
#![cfg_attr(doc, doc = include_str!("../README.md"))]
#![doc(html_logo_url = "https://libmeos.org/brand.svg")]
#![allow(refining_impl_trait)]
#![allow(clippy::non_canonical_partial_ord_impl)]

use std::{
    ffi::{c_void, CStr, CString},
    fmt::Debug,
    sync::Once,
};

use bitmask_enum::bitmask;
use boxes::Box as MeosBox;
pub use meos_sys as sys;

mod boxes;
pub use boxes::{STBox, TBox};

mod collections;
pub use collections::base::{Collection, Span, SpanSet};
pub use collections::datetime::{DateSpan, DateSpanSet, TsTzSpan, TsTzSpanSet};
pub use collections::number::*;

mod errors;
pub use errors::ParseError;

mod temporal;
pub use temporal::*;

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

/// Initializes the underlying MEOS platform.
///
/// This function must be called before any other PyMEOS-related function is used.
///
/// # Arguments
///
/// * `tz` - A string slice (`&str`) indicating the desired timezone to be used.
///
/// ## Example
/// ```
/// # use meos::meos_initialize;
///
/// meos_initialize();
/// ```
pub fn meos_initialize() {
    START.call_once(|| unsafe {
        meos_sys::meos_initialize();
        meos_sys::meos_initialize_error_handler(Some(error_handler));
        libc::atexit(finalize);
    });
}

pub fn meos_initialize_timezone(tz: &str) {
    unsafe {
        let ptr = CString::new(tz).unwrap();

        meos_sys::meos_initialize_timezone(ptr.as_ptr());
    }
}

fn factory<T: MeosEnum>(temporal: *mut meos_sys::Temporal) -> T {
    let temporal_type: TemporalSubtype = unsafe { (temporal.read().subtype as u32).into() };
    match temporal_type {
        TemporalSubtype::Instant => T::from_instant(temporal as *mut _),
        TemporalSubtype::Sequence => T::from_sequence(temporal as *mut _),
        TemporalSubtype::SequenceSet => T::from_sequence_set(temporal as *mut _),
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

pub trait MeosEnum: Debug + Sized + Temporal {
    fn from_instant(inner: *mut meos_sys::TInstant) -> Self;
    fn from_sequence(inner: *mut meos_sys::TSequence) -> Self;
    fn from_sequence_set(inner: *mut meos_sys::TSequenceSet) -> Self;

    /// Creates a temporal object from an MF-JSON string.
    ///
    /// ## Arguments
    /// * `mfjson` - The MF-JSON string.
    ///
    /// ## Returns
    /// A temporal object.
    fn from_mfjson(mfjson: &str) -> Self;

    /// Creates a temporal object from Well-Known Binary (WKB) bytes.
    ///
    /// ## Arguments
    /// * `wkb` - The WKB bytes.
    ///
    /// ## Returns
    /// A temporal object.
    fn from_wkb(wkb: &[u8]) -> Self {
        factory::<Self>(unsafe { meos_sys::temporal_from_wkb(wkb.as_ptr(), wkb.len()) })
    }

    /// Creates a temporal object from a hex-encoded WKB string.
    ///
    /// ## Arguments
    /// * `hexwkb` - The hex-encoded WKB string.
    ///
    /// ## Returns
    /// A temporal object.
    fn from_hexwkb(hexwkb: &[u8]) -> Self {
        let c_hexwkb = CString::new(hexwkb).unwrap();
        unsafe {
            let inner = meos_sys::temporal_from_hexwkb(c_hexwkb.as_ptr());
            factory::<Self>(inner)
        }
    }

    /// Creates a temporal object by merging multiple temporal objects.
    ///
    /// ## Arguments
    /// * `temporals` - The temporal objects to merge.
    ///
    /// ## Returns
    /// A merged temporal object.
    fn from_merge(temporals: &[Self]) -> Self {
        let mut t_list: Vec<_> = temporals.iter().map(Self::inner).collect();
        factory::<Self>(unsafe {
            meos_sys::temporal_merge_array(t_list.as_mut_ptr(), temporals.len() as i32)
        })
    }

    /// Returns the temporal object as an MF-JSON string.
    ///
    /// ## Arguments
    /// * `with_bbox` - Whether to include the bounding box in the output.
    /// * `flags` - The flags to use for the output.
    /// * `precision` - The precision to use for the output.
    /// * `srs` - The spatial reference system (SRS) to use for the output.
    ///
    /// ## Returns
    /// The temporal object as an MF-JSON string.
    fn as_mfjson(
        &self,
        with_bbox: bool,
        variant: JSONCVariant,
        precision: i32,
        srs: &str,
    ) -> String {
        let srs = CString::new(srs).unwrap();
        let out_str = unsafe {
            meos_sys::temporal_as_mfjson(
                self.inner(),
                with_bbox,
                variant as i32,
                precision,
                srs.as_ptr(),
            )
        };
        let c_str = unsafe { CStr::from_ptr(out_str) };
        let str = c_str.to_str().unwrap().to_owned();
        unsafe { libc::free(out_str as *mut c_void) };
        str
    }

    /// Returns the temporal object as Well-Known Binary (WKB) bytes.
    ///
    /// ## Returns
    /// The temporal object as WKB bytes.
    fn as_wkb(&self, variant: WKBVariant) -> &[u8] {
        unsafe {
            let mut size: usize = 0;
            let ptr = meos_sys::temporal_as_wkb(self.inner(), variant.into(), &mut size);
            std::slice::from_raw_parts(ptr, size)
        }
    }

    /// Returns the temporal object as a hex-encoded WKB string.
    ///
    /// ## Returns
    /// The temporal object as a hex-encoded WKB bytes.
    fn as_hexwkb(&self, variant: WKBVariant) -> &[u8] {
        unsafe {
            let mut size: usize = 0;
            let hexwkb_ptr = meos_sys::temporal_as_hexwkb(self.inner(), variant.into(), &mut size);

            CStr::from_ptr(hexwkb_ptr).to_bytes()
        }
    }
}

macro_rules! impl_from_str {
    ($type:ty) => {
        paste::paste! {
        impl FromStr for $type {
            type Err = ParseError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                CString::new(s).map_err(|_| ParseError).map(|string| {
                    let inner = unsafe { meos_sys::[<$type:lower _in>](string.as_ptr()) };
                    factory::<Self>(inner)
                })
            }
        }}
    };
}

pub(crate) use impl_from_str;
