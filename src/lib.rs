#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use chrono::{DateTime, NaiveTime, Utc};
use std::{
    ffi::{CStr, CString},
    time::Duration,
};

#[derive(Debug, Clone)]
pub struct Temporal<TBase> {
    _inner: TBase,
}

#[derive(Debug)]
pub enum TInterpolation {
    Linear,
    Stepwise,
}

impl TInterpolation {
    pub fn from_string(val: &str) -> Self {
        match val {
            "Linear" => TInterpolation::Linear,
            "Stepwise" => TInterpolation::Stepwise,
            _ => panic!("Unknown interpolation type"),
        }
    }

    pub fn to_string(&self) -> &str {
        match self {
            TInterpolation::Linear => "Linear",
            TInterpolation::Stepwise => "Stepwise",
        }
    }
}

pub trait Temporality: PartialOrd + PartialEq {
    type Base;
    type TG;
    type TI;
    type TS;
    type TSS;

    // fn from_base_time(value: Self::Base, base: Date) -> Self::TG;
    fn from_mfjson(mfjson: &str) -> Self::TG;
    fn from_wkb(wkb: &[u8]) -> Self::TG;
    fn from_hexwkb(hexwkb: &str) -> Self::TG;
    fn from_merge(temporals: Vec<Self::TG>) -> Self::TG;

    fn as_wkt(&self) -> String;
    fn as_mfjson(&self, with_bbox: bool, flags: i32, precision: i32, srs: Option<&str>) -> String;
    fn as_wkb(&self) -> Vec<u8>;
    fn as_hexwkb(&self) -> String;

    // fn bounding_box(&self) -> Box<dyn BoxTrait>;
    fn interpolation(&self) -> TInterpolation;
    // fn value_set(&self) -> HashSet<Self::Base>;
    fn values(&self) -> Vec<Self::Base>;
    fn start_value(&self) -> Self::Base;
    fn end_value(&self) -> Self::Base;
    fn min_value(&self) -> Self::Base;
    fn max_value(&self) -> Self::Base;
    fn value_at_timestamp(&self, timestamp: DateTime<Utc>) -> Self::Base;

    // fn time(&self) -> TsTzSpanSet;
    // fn duration(&self, ignore_gaps: bool) -> Duration;
    // fn tstzspan(&self) -> TsTzSpan;
    // fn timespan(&self) -> TsTzSpan;

    fn num_instants(&self) -> usize;
    fn start_instant(&self) -> Self::TI;
    fn end_instant(&self) -> Self::TI;
    fn min_instant(&self) -> Self::TI;
    fn max_instant(&self) -> Self::TI;
    fn instant_n(&self, n: usize) -> Self::TI;
    fn instants(&self) -> Vec<Self::TI>;

    fn num_timestamps(&self) -> usize;
    fn start_timestamp(&self) -> DateTime<Utc>;
    fn end_timestamp(&self) -> DateTime<Utc>;
    fn timestamp_n(&self, n: usize) -> DateTime<Utc>;
    fn timestamps(&self) -> Vec<DateTime<Utc>>;

    fn segments(&self) -> Vec<Self::TS>;

    fn set_interpolation(&self, interpolation: TInterpolation) -> Self::TG;
    fn shift_time(&self, delta: Duration) -> Self::TG;
    fn scale_time(&self, duration: Duration) -> Self::TG;
    fn shift_scale_time(&self, shift: Option<Duration>, duration: Option<Duration>) -> Self::TG;

    fn temporal_sample(&self, duration: Duration, start: Option<DateTime<Utc>>) -> Self::TG;
    fn temporal_precision(&self, duration: Duration, start: Option<DateTime<Utc>>) -> Self::TG;

    fn to_instant(&self) -> Self::TI;
    fn to_sequence(&self, interpolation: TInterpolation) -> Self::TS;
    fn to_sequenceset(&self, interpolation: TInterpolation) -> Self::TSS;

    fn append_instant(
        &self,
        instant: Self::TI,
        max_dist: Option<f64>,
        max_time: Option<Duration>,
    ) -> Self::TG;
    fn append_sequence(&self, sequence: Self::TS) -> Self::TG;
    fn merge(&self, other: Option<Vec<Self::TG>>) -> Self::TG;
    fn insert(&self, other: Self::TG, connect: bool) -> Self::TG;
    fn update(&self, other: Self::TG, connect: bool) -> Self::TG;
    fn delete(&self, other: NaiveTime, connect: bool) -> Self::TG;

    fn at(&self, other: NaiveTime) -> Self::TG;
    fn at_min(&self) -> Self::TG;
}

#[derive(Debug, PartialEq, Eq)]
struct TemporalError;

// impl FromStr for Temporal {
//     type Error = TemporalError;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         C::temporal_in
//     }
// }
