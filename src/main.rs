use std::{
    ffi::{CStr, CString},
    str::FromStr,
};

use chrono::{NaiveDate, TimeDelta};
use meos::collections::{
    base::{span::Span, span_set::SpanSet},
    datetime::{date_span::DateSpan, date_span_set::DateSpanSet, tstz_span::TsTzSpan},
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
    let other_other: FloatSpan = (9999.8..=234324.9).into();
    let span_set: FloatSpanSet = vec![span, other, other_other].iter().collect();
    println!("{:?}", span_set.into_iter().collect::<Vec<FloatSpan>>());

    let _int_span: IntSpan = (3..9).into();

    unsafe {
        let dstr = CString::new("(2019-09-08, 2019-09-10)").unwrap();
        let ds = meos_sys::datespan_in(dstr.as_ptr());

        println!(
            "{:?}",
            CStr::from_ptr(meos_sys::datespan_out(ds)).to_str().unwrap()
        );

        let l = meos_sys::datespan_lower(ds);
        println!("{:?}", NaiveDate::from_num_days_from_ce_opt(l))
    }

    let a = DateSpanSet::from_str("{[2019-09-08, 2019-09-10], [2019-09-11, 2019-09-12]}").unwrap();
    println!("{a:?}");
    unsafe {
        meos_sys::meos_finalize();
    }
    println!("Cómo?");
}
