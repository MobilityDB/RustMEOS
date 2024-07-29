use chrono::{DateTime, TimeDelta, TimeZone, Utc};

use crate::{
    collections::{base::collection::Collection, datetime::tstz_span::TsTzSpan},
    WKBVariant,
};

pub trait Box: Collection {
    fn from_wkb(wkb: &[u8]) -> Self;
    fn from_hexwkb(hexwkb: &[u8]) -> Self;
    fn from_time<Tz: TimeZone>(time: DateTime<Tz>) -> Self;
    fn from_temporal_span(span: TsTzSpan) -> Self;
    fn tstzspan(&self) -> TsTzSpan;
    fn as_wkb(&self, variant: WKBVariant) -> &[u8];
    fn as_hexwkb(&self, variant: WKBVariant) -> &[u8];
    fn round(&self, max_decimals: i32) -> Self;
    fn expand_time(&self, other: TimeDelta) -> Self;
    fn is_tmin_inclusive(&self) -> Option<bool>;
    fn is_tmax_inclusive(&self) -> Option<bool>;
    fn shift_scale_time(&self, delta: Option<TimeDelta>, width: Option<TimeDelta>) -> Self;
    fn intersection(&self, other: &Self) -> Option<Self>;
    fn union(&self, other: &Self, strict: bool) -> Option<Self>;
    fn nearest_approach_distance(&self, other: &Self) -> f64;
    fn has_x(&self) -> bool;
    fn has_t(&self) -> bool;
    fn xmin(&self) -> Option<f64>;
    fn xmax(&self) -> Option<f64>;
    fn tmin(&self) -> Option<DateTime<Utc>>;
    fn tmax(&self) -> Option<DateTime<Utc>>;
}
