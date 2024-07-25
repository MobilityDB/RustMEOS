use chrono::{Days, TimeDelta};

pub mod date_span;
pub mod date_span_set;
pub mod tstz_span;
pub mod tstz_span_set;

/// Needed since MEOS uses as a baseline date 2000-01-01
pub(crate) const DAYS_UNTIL_2000: Days = Days::new(730_120);

pub(crate) fn create_interval(t: TimeDelta) -> meos_sys::Interval {
    let time_in_microseconds = t.num_microseconds().unwrap_or(0);
    let total_days = t.num_days() as i32;

    meos_sys::Interval {
        time: time_in_microseconds,
        day: total_days,
        month: 0,
    }
}
