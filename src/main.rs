use std::str::FromStr;

use meos::collections::{
    base::span_set::SpanSet,
    number::{
        float_span::FloatSpan, float_span_set::FloatSpanSet, int_span::IntSpan,
        int_span_set::IntSpanSet,
    },
};

fn main() {
    unsafe {
        meos_sys::meos_initialize(std::ptr::null(), None);
    };

    let float_span_set = FloatSpanSet::from_str("{[17.5, 18.5), [19.5, 20.5)}").unwrap();
    let _float_span_set2 = FloatSpanSet::from_str("{[19.5, 23.5), [45.5, 67.5)}").unwrap();

    println!("{:?}", float_span_set.scale(6.1));

    println!(
        "{:?}",
        IntSpanSet::from_str("{[17, 18), [19, 20]}")
            .unwrap()
            .scale(2)
    );

    let span: FloatSpan = (67.0..5434.9).into();
    let other: FloatSpan = (5000.8..=6000.9).into();
    // let span_set: FloatSpanSet = vec![span, other].iter().collect();
    println!("{:?}", (span & other).unwrap());

    let _int_span: IntSpan = (3..9).into();

    unsafe {
        meos_sys::meos_finalize();
    }
}
