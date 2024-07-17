use std::ffi::{c_char, CString};

pub trait Span {
    type Type;
    fn inner(&self) -> *const meos_sys::Span;

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
            CString::from_raw(hexwkb_ptr as *mut c_char)
                .into_string()
                .unwrap()
        }
    }

    fn hash(&self) -> u32 {
        unsafe { meos_sys::span_hash(self.inner()) }
    }

    fn lower(&self) -> Self::Type;
}
