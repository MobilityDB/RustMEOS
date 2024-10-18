use std::{
    cmp,
    ffi::{c_void, CStr, CString},
    fmt::Debug,
    hash::Hash,
    ops::{BitAnd, Range, RangeInclusive},
    ptr,
};

use chrono::{DateTime, Datelike, TimeDelta, TimeZone, Utc};

use crate::{
    collections::base::*,
    errors::ParseError,
    utils::{create_interval, from_interval, from_meos_timestamp, to_meos_timestamp},
    BoundingBox,
};

pub struct TsTzSpan {
    _inner: ptr::NonNull<meos_sys::Span>,
}

impl Drop for TsTzSpan {
    fn drop(&mut self) {
        unsafe {
            libc::free(self._inner.as_ptr() as *mut c_void);
        }
    }
}

impl Collection for TsTzSpan {
    impl_collection!(span, DateTime<Utc>);

    fn contains(&self, content: &DateTime<Utc>) -> bool {
        unsafe { meos_sys::contains_span_date(self.inner(), content.num_days_from_ce()) }
    }
}

impl Span for TsTzSpan {
    type SubsetType = TimeDelta;
    fn inner(&self) -> *const meos_sys::Span {
        self._inner.as_ptr()
    }

    /// Creates a new `TsTzSpan` from an inner podateer to a `meos_sys::Span`.
    ///
    /// # Arguments
    /// * `inner` - A podateer to the inner `meos_sys::Span`.
    ///
    /// ## Returns
    /// * A new `TsTzSpan` instance.
    fn from_inner(inner: *mut meos_sys::Span) -> Self {
        Self {
            _inner: ptr::NonNull::new(inner).expect("Null pointers not allowed"),
        }
    }

    /// Returns the lower bound of the span.
    ///
    /// ## Returns
    /// * The lower bound as a `DateTime`.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::datetime::tstz_span::TsTzSpan;
    /// # use meos::collections::base::span::Span;
    /// # use chrono::naive::NaiveDate;
    ///
    /// let from_ymd_opt = |y, m, d| NaiveDate::from_ymd_opt(y, m, d)
    ///                                 .unwrap().and_hms_opt(0, 0, 0)
    ///                                 .unwrap().and_utc();
    ///
    /// let span: TsTzSpan = (from_ymd_opt(2023, 1, 1)..from_ymd_opt(2023, 1, 15)).into();
    /// let lower = span.lower();
    /// assert_eq!(lower, from_ymd_opt(2023, 1, 1));
    /// ```
    fn lower(&self) -> Self::Type {
        let timestamp = unsafe { meos_sys::tstzspan_lower(self.inner()) };
        from_meos_timestamp(timestamp)
    }

    /// Returns the upper bound of the span.
    ///
    /// ## Returns
    /// * The upper bound as a `DateTime`.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::datetime::tstz_span::TsTzSpan;
    /// # use meos::collections::base::span::Span;
    /// # use chrono::naive::NaiveDate;
    ///
    /// let from_ymd_opt = |y, m, d| NaiveDate::from_ymd_opt(y, m, d)
    ///                                 .unwrap().and_hms_opt(0, 0, 0)
    ///                                 .unwrap().and_utc();
    ///
    /// let span: TsTzSpan = (from_ymd_opt(2023, 1, 1)..from_ymd_opt(2023, 1, 15)).into();
    /// let upper = span.upper();
    /// assert_eq!(upper, from_ymd_opt(2023, 1, 15));
    /// ```
    fn upper(&self) -> Self::Type {
        let timestamp = unsafe { meos_sys::tstzspan_upper(self.inner()) };
        from_meos_timestamp(timestamp)
    }

    /// Return a new `TsTzSpan` with the lower and upper bounds shifted by `delta`.
    ///
    /// # Arguments
    /// * `delta` - The value to shift by, as a `DateTime`.
    ///
    /// # Returns
    /// A new `TsTzSpan` instance.
    ///
    /// # Example
    /// ```
    /// # use meos::collections::datetime::tstz_span::TsTzSpan;
    /// # use meos::collections::base::span::Span;
    /// # use meos::meos_initialize;
    /// use chrono::naive::NaiveDate;
    /// use chrono::TimeDelta;
    /// # meos_initialize();
    ///
    /// let from_ymd_opt = |y, m, d| NaiveDate::from_ymd_opt(y, m, d)
    ///                                 .unwrap().and_hms_opt(0, 0, 0)
    ///                                 .unwrap().and_utc();
    ///
    /// let span: TsTzSpan = (from_ymd_opt(2023, 1, 1)..from_ymd_opt(2023, 1, 15)).into();
    /// let shifted_span = span.shift(TimeDelta::weeks(8));
    /// let expected_span: TsTzSpan = (from_ymd_opt(2023, 4, 23)..from_ymd_opt(2023, 5, 7)).into();
    /// assert_eq!(shifted_span, expected_span);
    /// ```
    fn shift(&self, delta: TimeDelta) -> TsTzSpan {
        self.shift_scale(Some(delta), None)
    }

    /// Return a new `TsTzSpan` with the lower and upper bounds scaled so that the width is `width`.
    ///
    /// # Arguments
    /// * `width` - The new width, as a `DateTime`.
    ///
    /// # Returns
    /// A new `TsTzSpan` instance.
    ///
    /// # Example
    /// ```
    /// # use meos::collections::datetime::tstz_span::TsTzSpan;
    /// # use meos::collections::base::span::Span;
    /// # use meos::meos_initialize;
    /// use chrono::naive::NaiveDate;
    /// use chrono::TimeDelta;
    /// # meos_initialize();
    ///
    /// let from_ymd_opt = |y, m, d| NaiveDate::from_ymd_opt(y, m, d)
    ///                                 .unwrap().and_hms_opt(0, 0, 0)
    ///                                 .unwrap().and_utc();
    ///
    /// let span: TsTzSpan = (from_ymd_opt(2023, 1, 1)..from_ymd_opt(2023, 1, 15)).into();
    /// let scaled_span = span.scale(TimeDelta::weeks(4));
    /// let expected_span: TsTzSpan = (from_ymd_opt(2023, 1, 1)..from_ymd_opt(2023, 2, 26)).into();
    /// assert_eq!(scaled_span, expected_span);
    /// ```
    fn scale(&self, width: TimeDelta) -> TsTzSpan {
        self.shift_scale(None, Some(width))
    }

    /// Return a new `TsTzSpan` with the lower and upper bounds shifted by `delta` and scaled so that the width is `width`.
    ///
    /// # Arguments
    /// * `delta` - The value to shift by, as a `DateTime`.
    /// * `width` - The new width, as a `DateTime`.
    ///
    /// # Returns
    /// A new `TsTzSpan` instance.
    ///
    /// # Example
    /// ```
    /// # use meos::collections::datetime::tstz_span::TsTzSpan;
    /// # use meos::collections::base::span::Span;
    /// use chrono::naive::NaiveDate;
    /// use chrono::TimeDelta;
    /// # use meos::meos_initialize;
    /// # meos_initialize();
    /// let from_ymd_opt = |y, m, d| NaiveDate::from_ymd_opt(y, m, d)
    ///                                 .unwrap().and_hms_opt(0, 0, 0)
    ///                                 .unwrap().and_utc();
    ///
    /// let span: TsTzSpan = (from_ymd_opt(2023, 1, 1)..from_ymd_opt(2023, 1, 15)).into();
    /// let shifted_scaled_span = span.shift_scale(Some(TimeDelta::weeks(4)), Some(TimeDelta::weeks(4)));
    /// let expected_span: TsTzSpan = (from_ymd_opt(2023, 2, 26)..from_ymd_opt(2023, 4, 23)).into();
    /// assert_eq!(shifted_scaled_span, expected_span);
    /// ```
    fn shift_scale(&self, delta: Option<TimeDelta>, width: Option<TimeDelta>) -> TsTzSpan {
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

        let modified = unsafe { meos_sys::tstzspan_shift_scale(self.inner(), d, w) };
        TsTzSpan::from_inner(modified)
    }
    /// Calculates the distance between this `TsTzSpan` and a specific timestamp (`value`).
    ///
    /// ## Arguments
    /// * `value` - Anvalue `TsTzSpan` to calculate the distance to.
    ///
    /// ## Returns
    /// A `TimeDelta` representing the distance in seconds between the two spans.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::datetime::tstz_span::TsTzSpan;
    /// # use meos::meos_initialize;
    /// use std::str::FromStr;
    /// # use meos::collections::base::span::Span;
    /// use chrono::TimeDelta;
    /// # meos_initialize();
    /// let span1 = TsTzSpan::from_str("[2019-09-08 00:00:00+00, 2019-09-10 00:00:00+00]").unwrap();
    /// let span2 = TsTzSpan::from_str("[2019-09-12 00:00:00+00, 2019-09-14 00:00:00+00]").unwrap();
    /// let distance = span1.distance_to_span(&span2);
    /// assert_eq!(distance, TimeDelta::days(2));
    /// ```
    fn distance_to_value(&self, value: &Self::Type) -> TimeDelta {
        unsafe {
            TimeDelta::seconds(
                meos_sys::distance_span_timestamptz(self.inner(), to_meos_timestamp(value)) as i64, // It returns the number of seconds: https://github.com/MobilityDB/MobilityDB/blob/6b60876817b53bc33967f7df50ab8e1f482b0133/meos/src/general/span_ops.c#L2292
            )
        }
    }

    /// Calculates the distance between this `TsTzSpan` and another `TsTzSpan`.
    ///
    /// ## Arguments
    /// * `other` - Another `TsTzSpan` to calculate the distance to.
    ///
    /// ## Returns
    /// A `TimeDelta` representing the distance in seconds between the two spans.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::datetime::tstz_span::TsTzSpan;
    /// # use meos::collections::base::span::Span;
    /// # use chrono::{TimeDelta, TimeZone, Utc};
    /// # use meos::meos_initialize;
    /// use std::str::FromStr;
    /// # meos_initialize();
    /// let span_set1 = TsTzSpan::from_str("[2019-09-08 00:00:00+00, 2019-09-10 00:00:00+00]").unwrap();
    /// let span_set2 = TsTzSpan::from_str("[2018-08-07 00:00:00+00, 2018-08-17 00:00:00+00]").unwrap();
    /// let distance = span_set1.distance_to_span(&span_set2);
    /// assert_eq!(distance, TimeDelta::days(387));
    /// ```
    fn distance_to_span(&self, other: &Self) -> TimeDelta {
        unsafe {
            TimeDelta::seconds(
                meos_sys::distance_tstzspan_tstzspan(self.inner(), other.inner()) as i64,
            )
        }
    }
}

impl TsTzSpan {
    pub fn duration(&self) -> TimeDelta {
        from_interval(unsafe { meos_sys::tstzspan_duration(self.inner()).read() })
    }
}

impl BoundingBox for TsTzSpan {}

impl Clone for TsTzSpan {
    fn clone(&self) -> Self {
        unsafe { Self::from_inner(meos_sys::span_copy(self.inner())) }
    }
}

impl Hash for TsTzSpan {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let hash = unsafe { meos_sys::span_hash(self.inner()) };
        state.write_u32(hash);

        let _ = state.finish();
    }
}

impl std::str::FromStr for TsTzSpan {
    type Err = ParseError;
    /// Parses a `TsTzSpan` from a string representation.
    ///
    /// ## Arguments
    /// * `string` - A string slice containing the representation.
    ///
    /// ## Returns
    /// * A `TsTzSpan` instance.
    ///
    /// ## Errors
    /// * Returns `ParseSpanError` if the string cannot be parsed.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::datetime::tstz_span::TsTzSpan;
    /// # use meos::collections::base::span::Span;
    /// # use std::str::FromStr;
    /// # use meos::{meos_initialize, meos_initialize_timezone};
    /// use chrono::NaiveDate;
    /// # meos_initialize();
    /// # meos_initialize_timezone("UTC");
    /// let from_ymd_opt = |y, m, d| NaiveDate::from_ymd_opt(y, m, d)
    ///                                 .unwrap().and_hms_opt(0, 0, 0)
    ///                                 .unwrap().and_utc();
    ///
    /// let span: TsTzSpan = "(2019-09-08, 2019-09-10)".parse().expect("Failed to parse span");
    /// assert_eq!(span.lower(), from_ymd_opt(2019, 9, 8));
    /// assert_eq!(span.upper(), from_ymd_opt(2019, 9, 10));
    /// ```
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        CString::new(string).map_err(|_| ParseError).map(|string| {
            let inner = unsafe { meos_sys::tstzspan_in(string.as_ptr()) };
            Self::from_inner(inner)
        })
    }
}

impl cmp::PartialEq for TsTzSpan {
    /// Checks if two `TsTzSpan` instances are equal.
    ///
    /// # Arguments
    /// * `other` - Another `TsTzSpan` instance.
    ///
    /// ## Returns
    /// * `true` if the spans are equal, `false` otherwise.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::datetime::tstz_span::TsTzSpan;
    /// # use meos::collections::base::span::Span;
    /// use chrono::naive::NaiveDate;
    ///
    /// let from_ymd_opt = |y, m, d| NaiveDate::from_ymd_opt(y, m, d)
    ///                                 .unwrap().and_hms_opt(0, 0, 0)
    ///                                 .unwrap().and_utc();
    ///
    /// let span1: TsTzSpan = (from_ymd_opt(1, 1, 1)..from_ymd_opt(2, 2, 2)).into();
    /// let span2: TsTzSpan = (from_ymd_opt(1, 1, 1)..from_ymd_opt(2, 2, 2)).into();
    /// assert_eq!(span1, span2);
    /// ```
    fn eq(&self, other: &Self) -> bool {
        unsafe { meos_sys::span_eq(self.inner(), other.inner()) }
    }
}

impl cmp::Eq for TsTzSpan {}

impl<Tz: TimeZone> From<Range<DateTime<Tz>>> for TsTzSpan {
    fn from(Range { start, end }: Range<DateTime<Tz>>) -> Self {
        let inner = unsafe {
            meos_sys::tstzspan_make(
                to_meos_timestamp(&start),
                to_meos_timestamp(&end),
                true,
                false,
            )
        };
        Self::from_inner(inner)
    }
}

impl<Tz: TimeZone> From<RangeInclusive<DateTime<Tz>>> for TsTzSpan {
    fn from(range: RangeInclusive<DateTime<Tz>>) -> Self {
        let inner = unsafe {
            meos_sys::tstzspan_make(
                to_meos_timestamp(range.start()),
                to_meos_timestamp(range.end()),
                true,
                true,
            )
        };
        Self::from_inner(inner)
    }
}

impl Debug for TsTzSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out_str = unsafe { meos_sys::tstzspan_out(self.inner()) };
        let c_str = unsafe { CStr::from_ptr(out_str) };
        let str = c_str.to_str().map_err(|_| std::fmt::Error)?;
        let result = f.write_str(str);
        unsafe { libc::free(out_str as *mut c_void) };
        result
    }
}

// Implement BitAnd for dateersection with TsTzSpan
impl BitAnd for TsTzSpan {
    type Output = Option<TsTzSpan>;
    /// Computes the dateersection of two `TsTzSpan` instances.
    ///
    /// # Arguments
    /// * `other` - Another `TsTzSpan` instance.
    ///
    /// ## Returns
    /// * An `Option<TsTzSpan>` containing the dateersection, or `None` if there is no dateersection.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::datetime::tstz_span::TsTzSpan;
    /// # use meos::collections::base::span::Span;
    /// # use std::str::FromStr;
    /// use chrono::naive::NaiveDate;
    ///
    /// let from_ymd_opt = |y, m, d| NaiveDate::from_ymd_opt(y, m, d)
    ///                                 .unwrap().and_hms_opt(0, 0, 0)
    ///                                 .unwrap().and_utc();
    ///
    /// let span1: TsTzSpan = (from_ymd_opt(1, 1, 1)..from_ymd_opt(1, 1, 11)).into();
    /// let span2: TsTzSpan = (from_ymd_opt(1, 1, 9)..from_ymd_opt(2, 1, 11)).into();
    /// let date_intersection = (span1 & span2).unwrap();
    ///
    /// assert_eq!(date_intersection, (from_ymd_opt(1, 1, 9)..from_ymd_opt(1, 1, 11)).into())
    /// ```
    fn bitand(self, other: Self) -> Self::Output {
        // Replace with actual function call or logic
        let result = unsafe { meos_sys::intersection_span_span(self.inner(), other.inner()) };
        if !result.is_null() {
            Some(TsTzSpan::from_inner(result))
        } else {
            None
        }
    }
}

impl PartialOrd for TsTzSpan {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let cmp = unsafe { meos_sys::span_cmp(self.inner(), other.inner()) };
        match cmp {
            -1 => Some(cmp::Ordering::Less),
            0 => Some(cmp::Ordering::Equal),
            1 => Some(cmp::Ordering::Greater),
            _ => None,
        }
    }
}

impl Ord for TsTzSpan {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.partial_cmp(other).expect(
            "Unreachable since for non-null and same types spans, we only return -1, 0, or 1",
        )
    }
}
