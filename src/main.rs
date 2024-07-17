use std::ffi::{CStr, CString};

use meos::collections::{
    base::{span::Span, span_set::SpanSet},
    number::float_span_set::FloatSpanSet,
};

fn main() {
    unsafe {
        meos_sys::meos_initialize(std::ptr::null(), None);
    };
    // Define the input string
    let input = "POINT(1 1)@2000-01-01";

    // Convert the Rust string to a CString
    let c_string = CString::new(input).expect("CString::new failed");

    // Get a pointer to the C string
    let c_str_ptr = c_string.as_ptr();

    // Call the C function
    let result: *mut meos_sys::Temporal = unsafe { meos_sys::tgeogpoint_in(c_str_ptr) };
    unsafe {
        let ptr = meos_sys::temporal_as_mfjson(result, true, 3, 6, std::ptr::null_mut());
        let c_str: &CStr = CStr::from_ptr(ptr);

        // Convert &CStr to a Rust String
        println!("{}", c_str.to_string_lossy().into_owned());
    }

    let float_span_set = FloatSpanSet::from_string("{[17.5, 18.5), [19.5, 20.5)}");
    let start_span = float_span_set.start_span();

    println!("{}", start_span.lower());
}
