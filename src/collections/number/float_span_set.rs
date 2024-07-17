use std::ffi::CString;

use crate::collections::base::span::Span;
use crate::collections::base::*;

use super::float_span::FloatSpan;

pub struct FloatSpanSet {
    _inner: *const meos_sys::SpanSet,
}

impl span_set::SpanSet for FloatSpanSet {
    type Type = f64;
    fn inner(&self) -> *const meos_sys::SpanSet {
        self._inner
    }

    fn from_string(string: &str) -> Self
    where
        Self: Sized,
    {
        let c_string = CString::new(string).expect("Cannot create cstring");
        let spanset = unsafe { meos_sys::floatspanset_in(c_string.as_ptr()) };
        Self::from_inner(spanset)
    }

    fn from_wkb(wkb: &[u8]) -> Self
    where
        Self: Sized,
    {
        let spanset = unsafe { meos_sys::spanset_from_wkb(wkb.as_ptr(), wkb.len()) };
        Self::from_inner(spanset)
    }

    fn from_hexwkb(hexwkb: &str) -> Self
    where
        Self: Sized,
    {
        let c_string = CString::new(hexwkb).expect("Cannot create cstring");
        let spanset = unsafe { meos_sys::spanset_from_hexwkb(c_string.as_ptr()) };
        Self::from_inner(spanset)
    }

    fn from_inner(inner: *mut meos_sys::SpanSet) -> Self
    where
        Self: Sized,
    {
        Self { _inner: inner }
    }

    fn start_span(&self) -> FloatSpan {
        let span = unsafe { meos_sys::spanset_start_span(self.inner()) };
        super::float_span::FloatSpan::from_inner(span)
    }

    fn end_span(&self) -> FloatSpan {
        let span = unsafe { meos_sys::spanset_end_span(self.inner()) };
        super::float_span::FloatSpan::from_inner(span)
    }

    fn span_n(&self, n: i32) -> impl span::Span {
        let span = unsafe { meos_sys::spanset_span_n(self.inner(), n) };
        super::float_span::FloatSpan::from_inner(span)
    }

    fn spans(&self) -> Vec<impl crate::collections::base::span::Span> {
        let spans = unsafe { meos_sys::spanset_spans(self.inner()) };
        let size = self.num_spans() as usize;
        unsafe {
            Vec::from_raw_parts(spans, size, size)
                .iter()
                .map(|&span| FloatSpan::from_inner(span))
                .collect()
        }
    }
}
