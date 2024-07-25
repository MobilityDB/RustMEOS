use std::ffi::{c_void, CStr, CString};

use chrono::Datelike;
use chrono::NaiveDate;
use chrono::TimeDelta;
use collection::{impl_collection, Collection};
use span::Span;
use span_set::impl_iterator;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{BitAnd, BitOr};
use std::str::FromStr;

use crate::collections::base::span_set::SpanSet;
use crate::collections::base::*;
use crate::errors::ParseError;

use super::date_span::DateSpan;

pub struct DateSpanSet {
    _inner: *mut meos_sys::SpanSet,
}

impl Drop for DateSpanSet {
    fn drop(&mut self) {
        unsafe {
            meos_sys::free(self._inner as *mut c_void);
        }
    }
}

impl Collection for DateSpanSet {
    impl_collection!(spanset, date, NaiveDate);
    fn contains(&self, content: &NaiveDate) -> bool {
        unsafe { meos_sys::contains_spanset_date(self.inner(), content.num_days_from_ce()) }
    }
}

impl span_set::SpanSet for DateSpanSet {
    type SpanType = DateSpan;
    type ScaleShiftType = TimeDelta;
    fn inner(&self) -> *const meos_sys::SpanSet {
        self._inner
    }

    fn from_inner(inner: *mut meos_sys::SpanSet) -> Self
    where
        Self: Sized,
    {
        Self { _inner: inner }
    }

    fn width(&self, _ignore_gaps: bool) -> Self::Type {
        unimplemented!("Not implemented for date")
    }

    /// Return a new `DateSpanSet` with the lower and upper bounds shifted by `delta`.
    ///
    /// ## Arguments
    /// * `delta` - The value to shift by.
    ///
    /// ## Returns
    /// A new `DateSpanSet` instance.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::datetime::date_span_set::DateSpanSet;
    /// # use meos::init;
    /// # use std::str::FromStr;
    /// # use meos::collections::base::span_set::SpanSet;
    /// use chrono::TimeDelta;
    /// # init();
    /// let span_set = DateSpanSet::from_str("{[2019-09-08, 2019-09-10], [2019-09-16, 2019-09-20]}").unwrap();
    /// let shifted_span_set = span_set.shift(TimeDelta::days(5));
    ///
    /// let expected_shifted_span_set =
    ///     DateSpanSet::from_str("{[2019-09-13, 2019-09-16), [2019-09-21, 2019-09-26)}").unwrap();
    /// assert_eq!(shifted_span_set, expected_shifted_span_set);
    /// ```
    fn shift(&self, delta: TimeDelta) -> DateSpanSet {
        self.shift_scale(Some(delta), None)
    }

    /// Return a new `DateSpanSet` with the lower and upper bounds scaled so that the width is `width`.
    ///
    /// ## Arguments
    /// * `width` - The new width.
    ///
    /// ## Returns
    /// A new `DateSpanSet` instance.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::datetime::date_span_set::DateSpanSet;
    /// # use meos::init;
    /// # use std::str::FromStr;
    /// # use meos::collections::base::span_set::SpanSet;
    /// use chrono::TimeDelta;
    /// # init();
    /// let span_set = DateSpanSet::from_str("{[2019-09-08, 2019-09-10], [2019-09-13, 2019-09-15]}").unwrap();
    /// let scaled_span_set = span_set.scale(TimeDelta::days(5));
    ///
    /// let expected_scaled_span_set =
    ///     DateSpanSet::from_str("{[2019-09-08, 2019-09-10), [2019-09-11, 2019-09-14)}").unwrap();
    /// assert_eq!(scaled_span_set, expected_scaled_span_set);
    /// ```
    fn scale(&self, width: TimeDelta) -> DateSpanSet {
        self.shift_scale(None, Some(width))
    }

    /// Return a new `DateSpanSet` with the lower and upper bounds shifted by `delta` and scaled so that the width is `width`.
    ///
    /// ## Arguments
    /// * `delta` - The value to shift by.
    /// * `width` - The new width.
    ///
    /// ## Returns
    /// A new `DateSpanSet` instance.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::datetime::date_span_set::DateSpanSet;
    /// # use meos::init;
    /// # use std::str::FromStr;
    /// # use meos::collections::base::span_set::SpanSet;
    /// use chrono::TimeDelta;
    /// # init();
    /// let span_set = DateSpanSet::from_str("{[2019-09-08, 2019-09-10], [2019-09-11, 2019-09-12]}").unwrap();
    /// let shifted_scaled_span_set = span_set.shift_scale(Some(TimeDelta::days(5)), Some(TimeDelta::days(10)));
    ///
    /// let expected_shifted_scaled_span_set =
    ///     DateSpanSet::from_str("{[2019-09-13, 2019-09-24)}").unwrap();
    /// assert_eq!(shifted_scaled_span_set, expected_shifted_scaled_span_set);
    /// ```
    fn shift_scale(&self, delta: Option<TimeDelta>, width: Option<TimeDelta>) -> DateSpanSet {
        let d = delta
            .unwrap_or_default()
            .num_days()
            .try_into()
            .expect("Number too big");
        let w = width
            .unwrap_or_default()
            .num_days()
            .try_into()
            .expect("Number too big");
        let modified = unsafe {
            meos_sys::datespanset_shift_scale(self._inner, d, w, delta.is_some(), width.is_some())
        };
        DateSpanSet::from_inner(modified)
    }
}

impl Clone for DateSpanSet {
    fn clone(&self) -> DateSpanSet {
        self.copy()
    }
}

impl_iterator!(DateSpanSet);

impl Hash for DateSpanSet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let hash = unsafe { meos_sys::spanset_hash(self._inner) };
        state.write_u32(hash);

        state.finish();
    }
}

impl std::str::FromStr for DateSpanSet {
    type Err = ParseError;
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        CString::new(string).map_err(|_| ParseError).map(|string| {
            let inner = unsafe { meos_sys::datespanset_in(string.as_ptr()) };
            Self::from_inner(inner)
        })
    }
}

impl From<String> for DateSpanSet {
    fn from(value: String) -> Self {
        DateSpanSet::from_str(&value).expect("Failed to parse the span")
    }
}

impl std::cmp::PartialEq for DateSpanSet {
    fn eq(&self, other: &Self) -> bool {
        unsafe { meos_sys::spanset_eq(self._inner, other._inner) }
    }
}

impl Debug for DateSpanSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out_str = unsafe { meos_sys::datespanset_out(self._inner) };
        let c_str = unsafe { CStr::from_ptr(out_str) };
        let str = c_str.to_str().map_err(|_| std::fmt::Error)?;
        let result = f.write_str(str);
        unsafe { meos_sys::free(out_str as *mut c_void) };
        result
    }
}

impl BitAnd<DateSpanSet> for DateSpanSet {
    type Output = Option<DateSpanSet>;
    /// Computes the dateersection of two `DateSpanSet`s.
    ///
    /// ## Arguments
    ///
    /// * `other` - Another `DateSpanSet` to dateersect with.
    ///
    /// ## Returns
    ///
    /// * `Some(DateSpanSet)` - A new `DateSpanSet` containing the dateersection, if it exists.
    /// * `None` - If the dateersection is empty.
    ///
    /// ## Example
    ///
    /// ```
    /// # use meos::collections::datetime::date_span_set::DateSpanSet;
    /// # use meos::init;
    /// # use std::str::FromStr;
    /// # use meos::collections::base::span_set::SpanSet;
    /// # init();
    /// let span_set1 = DateSpanSet::from_str("{[2019-09-08, 2019-09-10], [2019-09-15, 2019-09-20]}").unwrap();
    /// let span_set2 = DateSpanSet::from_str("{[2019-09-15, 2019-09-30], [2019-11-11, 2019-11-12]}").unwrap();
    ///
    /// let expected_result = DateSpanSet::from_str("{[2019-09-15, 2019-09-21)}").unwrap();
    /// assert_eq!((span_set1 & span_set2).unwrap(), expected_result);
    /// ```
    fn bitand(self, other: DateSpanSet) -> Self::Output {
        self.intersection(&other)
    }
}

impl BitOr for DateSpanSet {
    type Output = Option<DateSpanSet>;
    /// Computes the union of two `DateSpanSet`s.
    ///
    /// ## Arguments
    ///
    /// * `other` - Another `DateSpanSet` to union with.
    ///
    /// ## Returns
    ///
    /// * `Some(DateSpanSet)` - A new `DateSpanSet` containing the union.
    /// * `None` - If the union is empty.
    ///
    /// ## Example
    ///
    /// ```
    /// # use meos::collections::datetime::date_span_set::DateSpanSet;
    /// # use meos::init;
    /// # use std::str::FromStr;
    /// # use meos::collections::base::span_set::SpanSet;
    /// # init();
    /// let span_set1 = DateSpanSet::from_str("{[2019-09-08, 2019-09-10], [2019-09-15, 2019-09-20]}").unwrap();
    /// let span_set2 = DateSpanSet::from_str("{[2019-09-15, 2019-09-30], [2019-11-11, 2019-11-12]}").unwrap();
    ///
    /// let expected_result = DateSpanSet::from_str("{[2019-09-08, 2019-09-11), [2019-09-15, 2019-10-01), [2019-11-11, 2019-11-13)}").unwrap();
    /// assert_eq!((span_set1 | span_set2).unwrap(), expected_result)
    /// ```
    fn bitor(self, other: Self) -> Self::Output {
        self.union(&other)
    }
}
