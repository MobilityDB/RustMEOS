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
