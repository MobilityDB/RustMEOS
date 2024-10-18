use chrono::Days;

mod date_span;
pub use date_span::DateSpan;

mod date_span_set;
pub use date_span_set::DateSpanSet;

mod tstz_span;
pub use tstz_span::TsTzSpan;

mod tstz_span_set;
pub use tstz_span_set::TsTzSpanSet;

/// Needed since MEOS uses as a baseline date 2000-01-01
pub(crate) const DAYS_UNTIL_2000: Days = Days::new(730_120);
pub(crate) const MICROSECONDS_UNTIL_2000: i64 = 946684800000000;
