use std::str::FromStr;

use chrono::{Days, TimeDelta, Utc};
use meos::{
    boxes::{r#box::Box as MeosBox, stbox::STBox, tbox::TBox},
    collections::{
        base::span_set::SpanSet,
        datetime::{date_span_set::DateSpanSet, tstz_span_set::TsTzSpanSet},
        number::{float_span::FloatSpan, float_span_set::FloatSpanSet, int_span_set::IntSpanSet},
    },
    init,
    temporal::{
        number::tint::{TInt, TIntInstant, TIntSequence, TIntSequenceSet},
        tbool::TBoolSequence,
        ttext::TTextSequence,
    },
    WKBVariant,
};

use meos::MeosEnum;

use meos::temporal::temporal::{OrderedTemporal, Temporal};
use meos::temporal::ttext::TTextTrait;

fn main() {
    init();

    let float_span_set = FloatSpanSet::from_str("{[17.5, 18.5), [19.5, 20.5)}").unwrap();
    let _float_span_set2 = FloatSpanSet::from_str("{[19.5, 23.5), [45.5, 67.5)}").unwrap();

    println!("{:?}", float_span_set.scale(6.1));

    println!(
        "{:?}",
        IntSpanSet::from_str("{[1, 2), [3, 4), [5, 6), [7, 8), [9, 10)}")
            .unwrap()
            .spans()
    );

    let span: FloatSpan = (67.0..5434.9).into();
    let other: FloatSpan = (6000.8..=7000.9).into();
    let other_other: FloatSpan = (9999.8..=234324.9).into();
    let span_set: FloatSpanSet = [span, other, other_other].iter().collect();
    println!("{:?}", span_set.into_iter().collect::<Vec<FloatSpan>>());

    let a = DateSpanSet::from_str("{[2019-09-08, 2019-09-10], [2019-09-11, 2019-09-12]}").unwrap();
    println!("{a:?}");
    let span_set: IntSpanSet = [(2019..2023).into(), (2029..2030).into()].iter().collect();
    println!("b{span_set:?}");
    let span_set = TsTzSpanSet::from_str("{[2019-09-08 00:00:00+00, 2019-09-10 00:00:00+00], [2019-09-11 00:00:00+00, 2019-09-12 00:00:00+00]}").unwrap();
    let shifted_scaled_span_set =
        span_set.shift_scale(Some(TimeDelta::days(5)), Some(TimeDelta::days(10)));
    println!("a{shifted_scaled_span_set:?}");
    let tbox = TBox::from_str("TBOXFLOAT XT([0, 10),[2020-06-01, 2020-06-05])").unwrap();
    println!("{tbox:?}");

    let stbox = STBox::from_str("STBOX Z((1.0, 2.0, 3.0), (4.0, 5.0, 6.0))").unwrap();
    let wkb = stbox.as_wkb(WKBVariant::NDR);
    println!("{stbox:?} {wkb:?}");
    println!("{:?}", WKBVariant::Extended | WKBVariant::NDR);

    let tint: TIntSequence = "[1@2001-01-01, 2@2001-01-03, 2@2001-01-04, 2@2001-01-05)"
        .parse()
        .unwrap();

    let yatint = TIntSequence::from_mfjson(
        r#"{"type":"MovingInteger","bbox":[10,25],"period":{"begin":"2001-01-01T18:00:00+01","end":"2001-01-01T18:10:00+01"},"values":[10,25],"datetimes":["2001-01-01T18:00:00+01",
"2001-01-01T18:10:00+01"],"lowerInc":true,"upperInc":true,
"interpolation":"Discrete"}"#,
    );

    println!("{yatint:?}");
    println!(
        "{}",
        yatint.as_mfjson(true, meos::temporal::JSONCVariant::Pretty, 3, "4326")
    );

    let tint2: TIntSequence = "{3@2001-01-01, 5@2001-01-03, 9@2001-01-04, 111@2001-01-05}"
        .parse()
        .unwrap();
    println!("{:?}", tint.values());

    println!("{tint:?}");

    println!("{:?}", tint2.at_value(&111));

    println!("{}", tint2.always_greater(&tint).unwrap());

    let tbool: TBoolSequence = "[true@2001-01-01 08:00:00, false@2001-01-03 08:00:00]"
        .parse()
        .unwrap();
    let tbool2: TBoolSequence = "[false@2001-01-01 08:00:00, true@2001-01-03 08:00:00]"
        .parse()
        .unwrap();
    println!("{:?}", tbool | true);

    let ttext: TTextSequence = "{AAA@2001-01-01 08:00:00, BBB@2001-01-03 08:00:00}"
        .parse()
        .unwrap();

    let tinst = (4, Utc::now());
    let tinst2 = (6, Utc::now().checked_add_days(Days::new(1)).unwrap());

    let tinst3 = (8, Utc::now().checked_add_days(Days::new(2)).unwrap());

    let tinst4 = (9, Utc::now().checked_add_days(Days::new(3)).unwrap());
    let tinst5 = (10, Utc::now().checked_add_days(Days::new(4)).unwrap());
    let tinst6 = (11, Utc::now().checked_add_days(Days::new(5)).unwrap());

    let vector: TIntSequence = [tinst, tinst2, tinst3].into_iter().collect();
    let vector2: TIntSequence = [tinst4, tinst5, tinst6].into_iter().collect();

    unsafe {
        println!("{:?}", TIntInstant::from(tinst).inner().read());
    }
    let a = TIntInstant::from(tinst).append_instant(
        (4, Utc::now().checked_add_days(Days::new(1)).unwrap()).into(),
        None,
        None,
    );

    unsafe {
        println!("{:?}", vector.inner().read());
    }
    println!("{vector:?}");

    let ss: TIntSequenceSet = [vector, vector2].into_iter().collect();
    println!("{:?}", ss);

    println!("{ttext:?}");
    println!("{:?}", ttext.at_value(&String::from("AAA")));
    println!("{:?}", ttext.concatenate_str("uwu"));
}
