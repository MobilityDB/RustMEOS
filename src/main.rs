use std::str::FromStr;

use meos::collections::number::{float_span::FloatSpan, float_span_set::FloatSpanSet};

fn main() {
    unsafe {
        meos_sys::meos_initialize(std::ptr::null(), None);
    };

    let _float_span_set = FloatSpanSet::from_str("{[17.5, 18.5), [19.5, 20.5)}");

    let span: FloatSpan = (67.0..5434.9).into();
    let other: FloatSpan = (5000.8..=6000.9).into();
    println!("{:?}", (span & other).unwrap());

    unsafe {
        meos_sys::meos_finalize();
    }
}
