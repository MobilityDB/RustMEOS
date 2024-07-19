use std::ffi::{c_char, CStr, CString};

use super::{collection::Collection, span::Span};

pub trait SpanSet: Collection {
    type SpanType: Span;
    fn inner(&self) -> *const meos_sys::SpanSet;

    /// Creates a new `Span` from a WKB representation.
    ///
    /// ## Arguments
    /// * `hexwkb` - A string slice containing the WKB representation.
    ///
    /// ## Returns
    /// * A new `Span` instance.
    fn from_wkb(wkb: &[u8]) -> Self {
        let span = unsafe { meos_sys::spanset_from_wkb(wkb.as_ptr(), wkb.len()) };
        Self::from_inner(span)
    }

    /// Creates a new `Span` from a hexadecimal WKB representation.
    ///
    /// ## Arguments
    /// * `hexwkb` - A string slice containing the hexadecimal WKB representation.
    ///
    /// ## Returns
    /// * A new `Span` instance.
    fn from_hexwkb(hexwkb: &str) -> Self {
        let c_string = CString::new(hexwkb).expect("Cannot create CString");
        let span = unsafe { meos_sys::spanset_from_hexwkb(c_string.as_ptr()) };
        Self::from_inner(span)
    }

    fn copy(&self) -> impl SpanSet {
        let inner = unsafe { meos_sys::spanset_copy(self.inner()) };
        Self::from_inner(inner)
    }

    fn from_inner(inner: *mut meos_sys::SpanSet) -> Self;

    fn as_wkb(&self) -> Vec<u8> {
        unsafe {
            let mut size = 0;
            let wkb = meos_sys::spanset_as_wkb(self.inner(), 4, &mut size as *mut _);
            Vec::from_raw_parts(wkb, size, size)
        }
    }

    // TODO CHECK with Esteban the variant number
    fn as_hexwkb(&self) -> String {
        unsafe {
            let hexwkb_ptr = meos_sys::spanset_as_hexwkb(self.inner(), 1, std::ptr::null_mut());
            CStr::from_ptr(hexwkb_ptr as *mut c_char)
                .to_str()
                .unwrap()
                .to_owned()
        }
    }

    fn num_spans(&self) -> i32 {
        unsafe { meos_sys::spanset_num_spans(self.inner()) }
    }

    fn start_span(&self) -> Self::SpanType {
        let span = unsafe { meos_sys::spanset_start_span(self.inner()) };
        Span::from_inner(span)
    }

    fn end_span(&self) -> Self::SpanType {
        let span = unsafe { meos_sys::spanset_end_span(self.inner()) };
        Span::from_inner(span)
    }

    fn span_n(&self, n: i32) -> Self::SpanType {
        let span = unsafe { meos_sys::spanset_span_n(self.inner(), n) };
        Span::from_inner(span)
    }

    fn spans(&self) -> Vec<Self::SpanType> {
        let spans = unsafe { meos_sys::spanset_spans(self.inner()) };
        let size = self.num_spans() as usize;
        unsafe {
            Vec::from_raw_parts(spans, size, size)
                .iter()
                .map(|&span| Span::from_inner(span))
                .collect()
        }
    }

    fn width(&self, ignore_gaps: bool) -> Self::Type;

    /// Return a new `SpanSet` with the lower and upper bounds shifted by `delta`.
    fn shift(&self, delta: Self::Type) -> Self;

    /// Return a new `SpanSet` with the lower and upper bounds scaled so that the width is `width`.
    fn scale(&self, width: Self::Type) -> Self;

    /// Return a new `SpanSet` with the lower and upper bounds shifted by `delta` and scaled so that the width is `width`.
    fn shift_scale(&self, delta: Option<Self::Type>, width: Option<Self::Type>) -> Self;

    fn hash(&self) -> u32 {
        unsafe { meos_sys::spanset_hash(self.inner()) }
    }
}
