use std::ffi::CString;

use crate::collections::base::*;

pub struct FloatSpan {
    _inner: *const meos_sys::Span,
}

impl span::Span for FloatSpan {
    type Type = f64;
    fn inner(&self) -> *const meos_sys::Span {
        self._inner
    }

    fn from_string(string: &str) -> Self
    where
        Self: Sized,
    {
        let c_string = CString::new(string).expect("Cannot create cstring");
        let span = unsafe { meos_sys::floatspan_in(c_string.as_ptr()) };
        Self::from_inner(span)
    }

    fn from_wkb(wkb: &[u8]) -> Self
    where
        Self: Sized,
    {
        let span = unsafe { meos_sys::span_from_wkb(wkb.as_ptr(), wkb.len()) };
        Self::from_inner(span)
    }

    fn from_hexwkb(hexwkb: &str) -> Self
    where
        Self: Sized,
    {
        let c_string = CString::new(hexwkb).expect("Cannot create cstring");
        let span = unsafe { meos_sys::span_from_hexwkb(c_string.as_ptr()) };
        Self::from_inner(span)
    }

    fn from_inner(inner: *mut meos_sys::Span) -> Self
    where
        Self: Sized,
    {
        Self { _inner: inner }
    }

    fn lower(&self) -> f64 {
        unsafe { meos_sys::floatspan_lower(self.inner()) }
    }
}
