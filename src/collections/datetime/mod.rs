use chrono::Days;

pub mod date_span;
pub mod date_span_set;
pub mod tstz_span;
pub mod tstz_span_set;

/// Needed since MEOS uses as a baseline date 2000-01-01
pub(crate) const DAYS_UNTIL_2000: Days = Days::new(730_120);
pub(crate) const MICROSECONDS_UNTIL_2000: i64 = 946684800000000;
