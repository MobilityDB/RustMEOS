use std::ffi::{CStr, CString};

fn main() {
    unsafe {
        meos_sys::meos_initialize(0x0 as *const i8, None);
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
        let ptr = meos_sys::temporal_as_mfjson(result, true, 3, 6, 0x0 as *mut i8);
        let c_str: &CStr = CStr::from_ptr(ptr);

        // Convert &CStr to a Rust String
        println!("{}", c_str.to_string_lossy().into_owned());
    }
    println!("Hello, world!");
}
