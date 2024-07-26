use std::str::FromStr;

use chrono::TimeDelta;
use meos::{
    collections::{
        base::span_set::SpanSet,
        datetime::{date_span_set::DateSpanSet, tstz_span_set::TsTzSpanSet},
        number::{float_span::FloatSpan, float_span_set::FloatSpanSet, int_span_set::IntSpanSet},
    },
    init, WKBVariant,
};

fn main() {
    init();

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
    let other: FloatSpan = (6000.8..=7000.9).into();
    let other_other: FloatSpan = (9999.8..=234324.9).into();
    let span_set: FloatSpanSet = vec![span, other, other_other].iter().collect();
    println!("{:?}", span_set.into_iter().collect::<Vec<FloatSpan>>());

    let a = DateSpanSet::from_str("{[2019-09-08, 2019-09-10], [2019-09-11, 2019-09-12]}").unwrap();
    println!("{a:?}");
    let span_set: IntSpanSet = [(2019..2023).into(), (2029..2030).into()].iter().collect();
    println!("b{span_set:?}");
    let span_set = TsTzSpanSet::from_str("{[2019-09-08 00:00:00+00, 2019-09-10 00:00:00+00], [2019-09-11 00:00:00+00, 2019-09-12 00:00:00+00]}").unwrap();
    let shifted_scaled_span_set =
        span_set.shift_scale(Some(TimeDelta::days(5)), Some(TimeDelta::days(10)));
    println!("a{shifted_scaled_span_set:?}");
    println!("{:?}", WKBVariant::Extended | WKBVariant::NDR)
}
