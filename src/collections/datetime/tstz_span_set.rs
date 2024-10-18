use std::ffi::{c_void, CStr, CString};
use std::ptr;

use chrono::DateTime;
use chrono::Datelike;
use chrono::TimeDelta;
use chrono::Utc;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{BitAnd, BitOr};

use crate::collections::base::*;
use crate::errors::ParseError;
use crate::utils::to_meos_timestamp;

use super::tstz_span::TsTzSpan;
use crate::utils::create_interval;

pub struct TsTzSpanSet {
    _inner: ptr::NonNull<meos_sys::SpanSet>,
}

impl Drop for TsTzSpanSet {
    fn drop(&mut self) {
        unsafe {
            libc::free(self._inner.as_ptr() as *mut c_void);
        }
    }
}

impl Collection for TsTzSpanSet {
    impl_collection!(spanset, DateTime<Utc>);
    fn contains(&self, content: &DateTime<Utc>) -> bool {
        unsafe { meos_sys::contains_spanset_date(self.inner(), content.num_days_from_ce()) }
    }
}

impl SpanSet for TsTzSpanSet {
    type SpanType = TsTzSpan;
    type SubsetType = TimeDelta;
    fn inner(&self) -> *const meos_sys::SpanSet {
        self._inner.as_ptr()
    }

    fn from_inner(inner: *mut meos_sys::SpanSet) -> Self {
        Self {
            _inner: ptr::NonNull::new(inner).expect("Null pointers not allowed"),
        }
    }

    fn width(&self, _ignore_gaps: bool) -> Self::Type {
        unimplemented!("Not implemented for date")
    }

    /// Return a new `TsTzSpanSet` with the lower and upper bounds shifted by `delta`.
    ///
    /// ## Arguments
    /// * `delta` - The value to shift by.
    ///
    /// ## Returns
    /// A new `TsTzSpanSet` instance.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::datetime::tstz_span_set::TsTzSpanSet;
    /// # use meos::meos_initialize;
    /// # use std::str::FromStr;
    /// # use meos::collections::base::span_set::SpanSet;
    /// use chrono::TimeDelta;
    /// # meos_initialize();
    /// let span_set = TsTzSpanSet::from_str("{[2019-09-08 00:00:00+00, 2019-09-10 00:00:00+00], [2019-09-16 00:00:00+00, 2019-09-20 00:00:00+00]}").unwrap();
    /// let shifted_span_set = span_set.shift(TimeDelta::days(5));
    ///
    /// let expected_shifted_span_set =
    ///     TsTzSpanSet::from_str("{[2019-09-18 00:00:00+00, 2019-09-20 00:00:00+00], [2019-09-26 00:00:00+00, 2019-09-30 00:00:00+00]}").unwrap();
    /// assert_eq!(shifted_span_set, expected_shifted_span_set);
    /// ```
    fn shift(&self, delta: TimeDelta) -> TsTzSpanSet {
        self.shift_scale(Some(delta), None)
    }

    /// Return a new `TsTzSpanSet` with the lower and upper bounds scaled so that the width is `width`.
    ///
    /// ## Arguments
    /// * `width` - The new width.
    ///
    /// ## Returns
    /// A new `TsTzSpanSet` instance.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::datetime::tstz_span_set::TsTzSpanSet;
    /// # use meos::meos_initialize;
    /// # use std::str::FromStr;
    /// # use meos::collections::base::span_set::SpanSet;
    /// use chrono::TimeDelta;
    /// # meos_initialize();
    /// let span_set = TsTzSpanSet::from_str("{[2019-09-08 00:00:00+00, 2019-09-10 00:00:00+00], [2019-09-13 00:00:00+00, 2019-09-15 00:00:00+00]}").unwrap();
    /// let scaled_span_set = span_set.scale(TimeDelta::days(5));
    ///
    /// let expected_scaled_span_set =
    ///     TsTzSpanSet::from_str("{[2019-09-08 00:00:00+00, 2019-09-10 20:34:17.142857+00], [2019-09-15 03:25:42.857142+00, 2019-09-18 00:00:00+00]}").unwrap();
    /// assert_eq!(scaled_span_set, expected_scaled_span_set);
    /// ```
    fn scale(&self, width: TimeDelta) -> TsTzSpanSet {
        self.shift_scale(None, Some(width))
    }

    /// Return a new `TsTzSpanSet` with the lower and upper bounds shifted by `delta` and scaled so that the width is `width`.
    ///
    /// ## Arguments
    /// * `delta` - The value to shift by.
    /// * `width` - The new width.
    ///
    /// ## Returns
    /// A new `TsTzSpanSet` instance.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::datetime::tstz_span_set::TsTzSpanSet;
    /// # use meos::meos_initialize;
    /// # use std::str::FromStr;
    /// # use meos::collections::base::span_set::SpanSet;
    /// use chrono::TimeDelta;
    /// # meos_initialize();
    /// let span_set = TsTzSpanSet::from_str("{[2019-09-08 00:00:00+00, 2019-09-10 00:00:00+00], [2019-09-11 00:00:00+00, 2019-09-12 00:00:00+00]}").unwrap();
    /// let shifted_scaled_span_set = span_set.shift_scale(Some(TimeDelta::days(5)), Some(TimeDelta::days(10)));
    ///
    /// let expected_shifted_scaled_span_set =
    ///     TsTzSpanSet::from_str("{[2019-09-18 00:00:00+00, 2019-09-28 00:00:00+00], [2019-10-03 00:00:00+00, 2019-10-08 00:00:00+00]}").unwrap();
    /// assert_eq!(shifted_scaled_span_set, expected_shifted_scaled_span_set);
    /// ```
    fn shift_scale(&self, delta: Option<TimeDelta>, width: Option<TimeDelta>) -> TsTzSpanSet {
        let d = {
            if let Some(d) = delta {
                &*Box::new(create_interval(d)) as *const meos_sys::Interval
            } else {
                std::ptr::null()
            }
        };

        let w = {
            if let Some(w) = width {
                &*Box::new(create_interval(w)) as *const meos_sys::Interval
            } else {
                std::ptr::null()
            }
        };

        let modified = unsafe { meos_sys::tstzspanset_shift_scale(self.inner(), d, w) };
        TsTzSpanSet::from_inner(modified)
    }

    /// Calculates the distance between this `TsTzSpanSet` and a specific timestamp (`value`).
    ///
    /// ## Arguments
    /// * `value` - A timestamp represented by `TimeDelta` from the Unix epoch.
    ///
    /// ## Returns
    /// A `TimeDelta` representing the distance in seconds between this `TsTzSpanSet` and the given timestamp.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::datetime::tstz_span_set::TsTzSpanSet;
    /// # use meos::collections::base::span_set::SpanSet;
    /// # use chrono::{TimeDelta, TimeZone, Utc};
    /// # use meos::meos_initialize;
    /// use std::str::FromStr;
    /// # meos_initialize();
    /// let span_set = TsTzSpanSet::from_str("{[2019-09-08 00:00:00+00, 2019-09-10 00:00:00+00], [2019-09-11 00:00:00+00, 2019-09-12 00:00:00+00]}").unwrap();
    /// let timestamp = Utc.with_ymd_and_hms(2019, 9, 5, 0, 0, 0).unwrap();
    /// let distance = span_set.distance_to_value(&timestamp);
    /// assert_eq!(distance, TimeDelta::days(3));
    /// ```
    fn distance_to_value(&self, value: &DateTime<Utc>) -> TimeDelta {
        unsafe {
            TimeDelta::seconds(
                meos_sys::distance_spanset_timestamptz(self.inner(), to_meos_timestamp(value))
                    as i64, // It returns the number of seconds: https://github.com/MobilityDB/MobilityDB/blob/6b60876817b53bc33967f7df50ab8e1f482b0133/meos/src/general/span_ops.c#L2292
            )
        }
    }

    /// Calculates the distance between this `TsTzSpanSet` and another `TsTzSpanSet`.
    ///
    /// ## Arguments
    /// * `other` - Another `TsTzSpanSet` to calculate the distance to.
    ///
    /// ## Returns
    /// A `TimeDelta` representing the distance in seconds between the two span sets.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::datetime::tstz_span_set::TsTzSpanSet;
    /// # use crate::meos::collections::base::span_set::SpanSet;
    /// # use chrono::{TimeDelta, TimeZone, Utc};
    /// # use meos::meos_initialize;
    /// use std::str::FromStr;
    /// # meos_initialize();
    /// let span_set1 = TsTzSpanSet::from_str("{[2019-09-08 00:00:00+00, 2019-09-10 00:00:00+00], [2019-09-11 00:00:00+00, 2019-09-12 00:00:00+00]}").unwrap();
    /// let span_set2 = TsTzSpanSet::from_str("{[2018-08-07 00:00:00+00, 2018-08-17 00:00:00+00], [2018-10-17 00:00:00+00, 2018-10-20 00:00:00+00]}").unwrap();
    /// let distance = span_set1.distance_to_span_set(&span_set2);
    /// assert_eq!(distance, TimeDelta::days(323));
    /// ```
    fn distance_to_span_set(&self, other: &Self) -> TimeDelta {
        unsafe {
            TimeDelta::seconds(meos_sys::distance_tstzspanset_tstzspanset(
                self.inner(),
                other.inner(),
            ) as i64)
        }
    }

    /// Calculates the distance between this `TsTzSpanSet` and a `TsTzSpan`.
    ///
    /// ## Arguments
    /// * `other` - A `TsTzSpan` to calculate the distance to.
    ///
    /// ## Returns
    /// A `TimeDelta` representing the distance in seconds between the span set and the span.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::datetime::tstz_span_set::TsTzSpanSet;
    /// # use meos::collections::base::span_set::SpanSet;
    /// # use meos::collections::datetime::tstz_span::TsTzSpan;
    /// # use meos::collections::base::span::Span;
    /// # use chrono::{TimeDelta, TimeZone, Utc};
    /// # use meos::meos_initialize;
    /// use std::str::FromStr;
    /// # meos_initialize();
    /// let span_set = TsTzSpanSet::from_str("{[2019-09-08 00:00:00+00, 2019-09-10 00:00:00+00], [2019-09-11 00:00:00+00, 2019-09-12 00:00:00+00]}").unwrap();
    /// let span = TsTzSpan::from_str("[2018-08-07 00:00:00+00, 2018-08-17 00:00:00+00]").unwrap();
    /// let distance = span_set.distance_to_span(&span);
    /// assert_eq!(distance, TimeDelta::days(387));
    /// ```
    fn distance_to_span(&self, span: &Self::SpanType) -> Self::SubsetType {
        unsafe {
            TimeDelta::seconds(
                meos_sys::distance_tstzspanset_tstzspan(self.inner(), span.inner()) as i64,
            )
        }
    }
}

impl Clone for TsTzSpanSet {
    fn clone(&self) -> TsTzSpanSet {
        self.copy()
    }
}

impl_iterator!(TsTzSpanSet);

impl Hash for TsTzSpanSet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let hash = unsafe { meos_sys::spanset_hash(self.inner()) };
        state.write_u32(hash);

        let _ = state.finish();
    }
}

impl std::str::FromStr for TsTzSpanSet {
    type Err = ParseError;
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        CString::new(string).map_err(|_| ParseError).map(|string| {
            let inner = unsafe { meos_sys::tstzspanset_in(string.as_ptr()) };
            Self::from_inner(inner)
        })
    }
}

impl std::cmp::PartialEq for TsTzSpanSet {
    fn eq(&self, other: &Self) -> bool {
        unsafe { meos_sys::spanset_eq(self.inner(), other.inner()) }
    }
}

impl Debug for TsTzSpanSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out_str = unsafe { meos_sys::tstzspanset_out(self.inner()) };
        let c_str = unsafe { CStr::from_ptr(out_str) };
        let str = c_str.to_str().map_err(|_| std::fmt::Error)?;
        let result = f.write_str(str);
        unsafe { libc::free(out_str as *mut c_void) };
        result
    }
}

impl BitAnd<TsTzSpanSet> for TsTzSpanSet {
    type Output = Option<TsTzSpanSet>;
    /// Computes the dateersection of two `TsTzSpanSet`s.
    ///
    /// ## Arguments
    ///
    /// * `other` - Another `TsTzSpanSet` to dateersect with.
    ///
    /// ## Returns
    ///
    /// * `Some(TsTzSpanSet)` - A new `TsTzSpanSet` containing the dateersection, if it exists.
    /// * `None` - If the dateersection is empty.
    ///
    /// ## Example
    ///
    /// ```
    /// # use meos::collections::datetime::tstz_span_set::TsTzSpanSet;
    /// # use meos::meos_initialize;
    /// # use std::str::FromStr;
    /// # use meos::collections::base::span_set::SpanSet;
    /// # meos_initialize();
    /// let span_set1 = TsTzSpanSet::from_str("{[2019-09-08 00:00:00+00, 2019-09-10 00:00:00+00], [2019-09-15 00:00:00+00, 2019-09-20 00:00:00+00]}").unwrap();
    /// let span_set2 = TsTzSpanSet::from_str("{[2019-09-15 00:00:00+00, 2019-09-30 00:00:00+00], [2019-11-11 00:00:00+00, 2019-11-12 00:00:00+00]}").unwrap();
    ///
    /// let expected_result = TsTzSpanSet::from_str("{[2019-09-15 00:00:00+00, 2019-09-20 00:00:00+00]}").unwrap();
    /// assert_eq!((span_set1 & span_set2).unwrap(), expected_result);
    /// ```
    fn bitand(self, other: TsTzSpanSet) -> Self::Output {
        self.intersection(&other)
    }
}

impl BitOr for TsTzSpanSet {
    type Output = Option<TsTzSpanSet>;
    /// Computes the union of two `TsTzSpanSet`s.
    ///
    /// ## Arguments
    ///
    /// * `other` - Another `TsTzSpanSet` to union with.
    ///
    /// ## Returns
    ///
    /// * `Some(TsTzSpanSet)` - A new `TsTzSpanSet` containing the union.
    /// * `None` - If the union is empty.
    ///
    /// ## Example
    ///
    /// ```
    /// # use meos::collections::datetime::tstz_span_set::TsTzSpanSet;
    /// # use meos::meos_initialize;
    /// # use std::str::FromStr;
    /// # use meos::collections::base::span_set::SpanSet;
    /// # meos_initialize();
    /// let span_set1 = TsTzSpanSet::from_str("{[2019-09-08 00:00:00+00, 2019-09-10 00:00:00+00], [2019-09-15 00:00:00+00, 2019-09-20 00:00:00+00]}").unwrap();
    /// let span_set2 = TsTzSpanSet::from_str("{[2019-09-15 00:00:00+00, 2019-09-30 00:00:00+00], [2019-11-11 00:00:00+00, 2019-11-12 00:00:00+00]}").unwrap();
    ///
    /// let expected_result = TsTzSpanSet::from_str("{[2019-09-08 00:00:00+00, 2019-09-10 00:00:00+00], [2019-09-15 00:00:00+00, 2019-09-30 00:00:00+00], [2019-11-11 00:00:00+00, 2019-11-12 00:00:00+00]}").unwrap();
    /// assert_eq!((span_set1 | span_set2).unwrap(), expected_result)
    /// ```
    fn bitor(self, other: Self) -> Self::Output {
        self.union(&other)
    }
}
