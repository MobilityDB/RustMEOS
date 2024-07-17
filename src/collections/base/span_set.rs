use std::ffi::{c_char, CString};

use super::span::Span;

pub trait SpanSet {
    type Type;
    fn inner(&self) -> *const meos_sys::SpanSet;

    fn from_string(string: &str) -> Self
    where
        Self: Sized;

    fn from_wkb(wkb: &[u8]) -> Self
    where
        Self: Sized;

    fn from_hexwkb(hexwkb: &str) -> Self
    where
        Self: Sized;

    fn copy(&self) -> Self
    where
        Self: Sized,
    {
        let inner = unsafe { meos_sys::spanset_copy(self.inner()) };
        Self::from_inner(inner)
    }

    fn from_inner(inner: *mut meos_sys::SpanSet) -> Self
    where
        Self: Sized;

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
            CString::from_raw(hexwkb_ptr as *mut c_char)
                .into_string()
                .unwrap()
        }
    }

    fn num_spans(&self) -> i32 {
        unsafe { meos_sys::spanset_num_spans(self.inner()) }
    }

    fn start_span(&self) -> impl Span;

    fn end_span(&self) -> impl Span;

    fn span_n(&self, n: i32) -> impl Span;

    fn spans(&self) -> Vec<impl Span>;

    fn hash(&self) -> u32 {
        unsafe { meos_sys::spanset_hash(self.inner()) }
    }
}
