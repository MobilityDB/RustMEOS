pub(crate) fn create_interval(t: chrono::TimeDelta) -> meos_sys::Interval {
    let time_in_microseconds = t.num_microseconds().unwrap_or(0);
    let total_days = t.num_days() as i32;

    meos_sys::Interval {
        time: time_in_microseconds,
        day: total_days,
        month: 0,
    }
}
