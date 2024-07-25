use std::ffi::CString;

use libc::c_void;

pub struct TBox {
    _inner: *mut meos_sys::TBox,
}

impl TBox {
    pub fn new_from_string(string: &str) -> Self {
        let c_string = CString::new(string).unwrap();
        unsafe {
            let inner = meos_sys::tbox_in(c_string.as_ptr());
            TBox { _inner: inner }
        }
    }

    pub fn copy(&self) -> Self {
        unsafe {
            let inner_copy = meos_sys::tbox_copy(self._inner);
            TBox { _inner: inner_copy }
        }
    }

    pub fn from_wkb(wkb: &[u8]) -> Self {
        unsafe {
            let inner = meos_sys::tbox_from_wkb(wkb.as_ptr(), wkb.len());
            TBox { _inner: inner }
        }
    }

    pub fn from_hexwkb(hexwkb: &str) -> Self {
        let c_hexwkb = CString::new(hexwkb).unwrap();
        unsafe {
            let inner = meos_sys::tbox_from_hexwkb(c_hexwkb.as_ptr());
            TBox { _inner: inner }
        }
    }

    pub fn from_value(value: TBoxValue) -> Self {
        unsafe {
            let inner = match value {
                TBoxValue::Int(val) => int_to_meos_sys::tbox(val),
                TBoxValue::Float(val) => float_to_meos_sys::tbox(val),
                TBoxValue::Span(span) => span_to_meos_sys::tbox(span.inner),
            };
            TBox { _inner: inner }
        }
    }

    pub fn from_time(time: DateTime<Utc>) -> Self {
        // Convert DateTime<Utc> to the expected timestamp format for MEOS
        let timestamptz = time.timestamp();
        unsafe {
            let inner = timestamptz_to_meos_sys::tbox(timestamptz);
            TBox { _inner: inner }
        }
    }

    pub fn from_tnumber(temporal: TNumber) -> Self {
        unsafe {
            let inner = tnumber_to_meos_sys::tbox(temporal.inner);
            TBox { _inner: inner }
        }
    }

    pub fn to_floatspan(&self) -> FloatSpan {
        unsafe {
            let inner_span = meos_sys::tbox_to_floatspan(self._inner);
            FloatSpan { inner: inner_span }
        }
    }

    pub fn to_tstzspan(&self) -> TsTzSpan {
        unsafe {
            let inner_span = meos_sys::tbox_to_tstzspan(self._inner);
            TsTzSpan { inner: inner_span }
        }
    }

    pub fn as_wkb(&self) -> Vec<u8> {
        unsafe {
            let mut size: usize = 0;
            let ptr = meos_sys::tbox_as_wkb(self._inner, 4, &mut size);
            Vec::from_raw_parts(ptr as *mut u8, size, size)
        }
    }

    pub fn as_hexwkb(&self) -> String {
        unsafe {
            let ptr = meos_sys::tbox_as_hexwkb(self._inner, -1);
            CString::from_raw(ptr as *mut i8).into_string().unwrap()
        }
    }

    pub fn to_string(&self, max_decimals: i32) -> String {
        unsafe {
            let c_str = meos_sys::tbox_out(self._inner, max_decimals);
            CString::from_raw(c_str as *mut i8).into_string().unwrap()
        }
    }
}
