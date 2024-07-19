use std::ffi::{CStr, CString};

use super::{collection::Collection, span_set::SpanSet};

pub trait Span: Collection {
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
    fn from_hexwkb(hexwkb: &str) -> Self {
        let c_string = CString::new(hexwkb).expect("Cannot create CString");
        let span = unsafe { meos_sys::span_from_hexwkb(c_string.as_ptr()) };
        Self::from_inner(span)
    }

    fn copy(&self) -> Self
    where
        Self: Sized,
    {
        let inner = unsafe { meos_sys::span_copy(self.inner()) };
        Self::from_inner(inner)
    }

    fn from_inner(inner: *mut meos_sys::Span) -> Self
    where
        Self: Sized;

    // TODO CHECK with Esteban
    fn as_wkb(&self) -> Vec<u8> {
        unsafe {
            let mut size = 0;
            let wkb = meos_sys::span_as_wkb(self.inner(), 4, &mut size as *mut _);
            Vec::from_raw_parts(wkb, size, size)
        }
    }

    // TODO CHECK with Esteban the variant number
    fn as_hexwkb(&self) -> String {
        unsafe {
            let hexwkb_ptr = meos_sys::span_as_hexwkb(self.inner(), 1, std::ptr::null_mut());
            CStr::from_ptr(hexwkb_ptr).to_str().unwrap().to_owned()
        }
    }

    fn lower(&self) -> Self::Type;
    fn upper(&self) -> Self::Type;

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

    /// Checks if this span is adjacent to another span.
    ///
    /// # Arguments
    /// * `other` - A reference to another `Span`.
    ///
    /// ## Returns
    /// * `true` if the spans are adjacent, `false` otherwise.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::number::float_span::FloatSpan;
    /// # use meos::collections::base::span::Span;
    ///
    /// let span1: FloatSpan = (12.9..67.8).into();
    /// let span2: FloatSpan = (67.8..98.0).into();
    /// assert!(span1.is_adjacent(&span2));
    /// ```
    fn is_adjacent(&self, other: &Self) -> bool {
        unsafe { meos_sys::adjacent_span_span(self.inner(), other.inner()) }
    }

    /// Return a new `Span` with the lower and upper bounds shifted by `delta`.
    fn shift(&self, delta: Self::Type) -> Self;

    /// Return a new `Span` with the lower and upper bounds scaled so that the width is `width`.
    fn scale(&self, width: Self::Type) -> Self;

    fn to_spanset<T: SpanSet<Type = Self::Type>>(&self) -> T {
        unsafe { T::from_inner(meos_sys::span_to_spanset(self.inner())) }
    }

    /// Return a new `Span` with the lower and upper bounds shifted by `delta` and scaled so that the width is `width`.
    fn shift_scale(&self, delta: Option<Self::Type>, width: Option<Self::Type>) -> Self;
}
