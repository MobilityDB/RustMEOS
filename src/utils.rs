use chrono::{DateTime, TimeZone, Utc};

use crate::collections::datetime::MICROSECONDS_UNTIL_2000;

pub(crate) fn create_interval(t: chrono::TimeDelta) -> meos_sys::Interval {
    let time_in_microseconds = t.num_microseconds().unwrap_or(0);
    let total_days = t.num_days() as i32;

    meos_sys::Interval {
        time: time_in_microseconds,
        day: total_days,
        month: 0,
    }
}

pub(crate) fn from_interval(interval: meos_sys::Interval) -> chrono::TimeDelta {
    let time_in_microseconds = interval.time;
    let days = interval.day as i64;
    let months = interval.month as i64;

    chrono::TimeDelta::microseconds(time_in_microseconds)
        + chrono::TimeDelta::days(days + months * 30) // meos assumes 30 days per month
}

pub(crate) fn to_meos_timestamp<Tz: TimeZone>(dt: &DateTime<Tz>) -> i64 {
    dt.timestamp_micros() - MICROSECONDS_UNTIL_2000
}

pub(crate) fn from_meos_timestamp(timestamp: meos_sys::TimestampTz) -> DateTime<Utc> {
    DateTime::from_timestamp_micros(timestamp + MICROSECONDS_UNTIL_2000)
        .expect("Failed to parse DateTime")
}
