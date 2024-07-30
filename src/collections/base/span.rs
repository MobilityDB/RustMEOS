use std::ffi::{CStr, CString};

use crate::WKBVariant;

use super::{collection::Collection, span_set::SpanSet};

pub trait Span: Collection {
    /// Type used to represent subsets (duration, widths, etc.)
    type SubsetType;
    fn inner(&self) -> *const meos_sys::Span;

    /// Creates a new `Span` from a WKB representation.
    ///
    /// # Arguments
    /// * `hexwkb` - A string slice containing the WKB representation.
    ///
    /// ## Returns
    /// * A new `Span` instance.
    fn from_wkb(wkb: &[u8]) -> Self {
        let span = unsafe { meos_sys::span_from_wkb(wkb.as_ptr(), wkb.len()) };
        Self::from_inner(span)
    }

    /// Creates a new `Span` from a hexadecimal WKB representation.
    ///
    /// # Arguments
    /// * `hexwkb` - A string slice containing the hexadecimal WKB representation.
    ///
    /// ## Returns
    /// * A new `Span` instance.
    fn from_hexwkb(hexwkb: &[u8]) -> Self {
        let c_string = CString::new(hexwkb).expect("Cannot create CString");
        let span = unsafe { meos_sys::span_from_hexwkb(c_string.as_ptr()) };
        Self::from_inner(span)
    }

    fn from_inner(inner: *const meos_sys::Span) -> Self;

    fn as_wkb(&self, variant: WKBVariant) -> &[u8] {
        unsafe {
            let mut size = 0;
            let wkb = meos_sys::span_as_wkb(self.inner(), variant.into(), &mut size as *mut _);
            std::slice::from_raw_parts(wkb, size)
        }
    }

    fn as_hexwkb(&self, variant: WKBVariant) -> &[u8] {
        unsafe {
            let mut size: usize = 0;
            let hexwkb_ptr = meos_sys::span_as_hexwkb(self.inner(), variant.into(), &mut size);
            CStr::from_ptr(hexwkb_ptr).to_bytes()
        }
    }

    fn lower(&self) -> Self::Type;
    fn upper(&self) -> Self::Type;

    fn distance_to_value(&self, value: &Self::Type) -> Self::SubsetType;
    fn distance_to_span(&self, other: &Self) -> Self::SubsetType;

    /// Checks if the lower bound of the span is inclusive.
    ///
    /// ## Returns
    /// * `true` if the lower bound is inclusive, `false` otherwise.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::number::float_span::FloatSpan;
    /// # use meos::collections::base::span::Span;
    /// # use std::str::FromStr;
    ///
    /// let span: FloatSpan = FloatSpan::from_str("[23.9, 78.8]").unwrap();
    /// assert!(span.is_lower_inclusive());
    ///
    /// let span: FloatSpan = FloatSpan::from_str("(23.9, 78.8]").unwrap();
    /// assert!(!span.is_lower_inclusive());
    /// ```
    fn is_lower_inclusive(&self) -> bool {
        unsafe { meos_sys::span_lower_inc(self.inner()) }
    }

    /// Checks if the upper bound of the span is inclusive.
    ///
    /// ## Returns
    /// * `true` if the upper bound is inclusive, `false` otherwise.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::number::float_span::FloatSpan;
    /// # use meos::collections::base::span::Span;
    /// # use std::str::FromStr;
    ///
    /// let span: FloatSpan = (23.9..=78.8).into();
    /// assert!(span.is_upper_inclusive());
    ///
    /// let span: FloatSpan = (23.9..78.8).into();
    /// assert!(!span.is_upper_inclusive());
    /// ```
    fn is_upper_inclusive(&self) -> bool {
        unsafe { meos_sys::span_upper_inc(self.inner()) }
    }

    /// Return a new `Span` with the lower and upper bounds shifted by `delta`.
    fn shift(&self, delta: Self::SubsetType) -> Self;

    /// Return a new `Span` with the lower and upper bounds scaled so that the width is `width`.
    fn scale(&self, width: Self::SubsetType) -> Self;

    /// Return a new `Span` with the lower and upper bounds shifted by `delta` and scaled so that the width is `width`.
    fn shift_scale(&self, delta: Option<Self::SubsetType>, width: Option<Self::SubsetType>)
        -> Self;

    fn to_spanset<T: SpanSet<Type = Self::Type>>(&self) -> T {
        unsafe { T::from_inner(meos_sys::span_to_spanset(self.inner())) }
    }

    fn intersection(&self, other: &Self) -> Option<Self> {
        let result = unsafe { meos_sys::intersection_span_span(self.inner(), other.inner()) };
        if !result.is_null() {
            Some(Self::from_inner(result))
        } else {
            None
        }
    }

    fn union<T: SpanSet<Type = Self::Type>>(&self, other: &Self) -> Option<T> {
        let result = unsafe { meos_sys::union_span_span(self.inner(), other.inner()) };
        if !result.is_null() {
            Some(T::from_inner(result))
        } else {
            None
        }
    }
}
