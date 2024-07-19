use std::{
    cmp,
    ffi::{c_void, CStr, CString},
    fmt::Debug,
    hash::Hash,
    ops::{BitAnd, Range, RangeInclusive},
    str::FromStr,
};

use chrono::{Datelike, NaiveDate};
use collection::{impl_collection, Collection};
use span::Span;

use crate::{collections::base::*, errors::ParseSpanError};

pub struct DateSpan {
    _inner: *mut meos_sys::Span,
}

impl Drop for DateSpan {
    fn drop(&mut self) {
        unsafe {
            meos_sys::free(self._inner as *mut c_void);
        }
    }
}

impl Collection for DateSpan {
    impl_collection!(span, date, NaiveDate);

    fn contains(&self, content: &NaiveDate) -> bool {
        unsafe { meos_sys::contains_span_date(self.inner(), content.num_days_from_ce()) }
    }
}

impl span::Span for DateSpan {
    fn inner(&self) -> *const meos_sys::Span {
        self._inner
    }

    /// Creates a new `DateSpan` from an inner podateer to a `meos_sys::Span`.
    ///
    /// # Arguments
    /// * `inner` - A podateer to the inner `meos_sys::Span`.
    ///
    /// ## Returns
    /// * A new `DateSpan` instance.
    fn from_inner(inner: *mut meos_sys::Span) -> Self {
        Self { _inner: inner }
    }

    /// Returns the lower bound of the span.
    ///
    /// ## Returns
    /// * The lower bound as a `NaiveDate`.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::number::date_span::DateSpan;
    /// # use meos::collections::base::span::Span;
    ///
    /// let span: DateSpan = (12..67).into();
    /// let lower = span.lower();
    /// ```
    fn lower(&self) -> Self::Type {
        let num_of_days = unsafe { meos_sys::datespan_lower(self.inner()) };
        NaiveDate::from_num_days_from_ce_opt(num_of_days).expect("Wrong date returned from meos")
    }

    /// Returns the upper bound of the span.
    ///
    /// ## Returns
    /// * The upper bound as a `NaiveDate`.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::number::date_span::DateSpan;
    /// # use meos::collections::base::span::Span;
    ///
    /// let span: DateSpan = (12..67).dateo();;
    ///
    /// assert_eq!(span.upper(), 67)
    /// ```
    fn upper(&self) -> Self::Type {
        let num_of_days = unsafe { meos_sys::datespan_upper(self.inner()) };
        NaiveDate::from_num_days_from_ce_opt(num_of_days).expect("Wrong date returned from meos")
    }

    /// Return a new `DateSpan` with the lower and upper bounds shifted by `delta`.
    ///
    /// # Arguments
    /// * `delta` - The value to shift by.
    ///
    /// # Returns
    /// A new `DateSpan` instance.
    ///
    /// # Example
    /// ```
    /// # use meos::collections::number::date_span::DateSpan;
    /// # use meos::collections::base::span::Span;
    ///
    /// let span: DateSpan = (12..67).dateo();
    /// let shifted_span = span.shift(5);
    ///
    /// assert_eq!(shifted_span, (17..72).dateo())
    /// ```
    fn shift(&self, delta: NaiveDate) -> DateSpan {
        self.shift_scale(Some(delta), None)
    }

    /// Return a new `DateSpan` with the lower and upper bounds scaled so that the width is `width`.
    ///
    /// # Arguments
    /// * `width` - The new width.
    ///
    /// # Returns
    /// A new `DateSpan` instance.
    ///
    /// # Example
    /// ```
    /// # use meos::collections::number::date_span::DateSpan;
    /// # use meos::collections::base::span::Span;
    ///
    /// let span: DateSpan = (12..67).dateo();
    /// let scaled_span = span.scale(10);
    ///
    /// assert_eq!(scaled_span, (12..23).dateo())
    /// ```
    fn scale(&self, width: NaiveDate) -> DateSpan {
        self.shift_scale(None, Some(width))
    }

    /// Return a new `DateSpan` with the lower and upper bounds shifted by `delta` and scaled so that the width is `width`.
    ///
    /// # Arguments
    /// * `delta` - The value to shift by.
    /// * `width` - The new width.
    ///
    /// # Returns
    /// A new `DateSpan` instance.
    ///
    /// # Example
    /// ```
    /// # use meos::collections::number::date_span::DateSpan;
    /// # use meos::collections::base::span::Span;
    ///
    /// let span: DateSpan = (12..67).into();
    /// let shifted_scaled_span = span.shift_scale(Some(5), Some(10));
    ///
    /// assert_eq!(shifted_scaled_span, (17..28).dateo())
    /// ```
    fn shift_scale(&self, delta: Option<NaiveDate>, width: Option<NaiveDate>) -> DateSpan {
        let d = delta.unwrap_or_default().num_days_from_ce();
        let w = width.unwrap_or_default().num_days_from_ce();
        let modified = unsafe {
            meos_sys::datespan_shift_scale(self._inner, d, w, delta.is_some(), width.is_some())
        };
        DateSpan::from_inner(modified)
    }
}

impl Clone for DateSpan {
    fn clone(&self) -> Self {
        unsafe { Self::from_inner(meos_sys::span_copy(self._inner)) }
    }
}

impl Hash for DateSpan {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let hash = unsafe { meos_sys::span_hash(self._inner) };
        state.write_u32(hash);

        state.finish();
    }
}

impl std::str::FromStr for DateSpan {
    type Err = ParseSpanError;
    /// Parses a `DateSpan` from a string representation.
    ///
    /// ## Arguments
    /// * `string` - A string slice containing the representation.
    ///
    /// ## Returns
    /// * A `DateSpan` instance.
    ///
    /// ## Errors
    /// * Returns `ParseSpanError` if the string cannot be parsed.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::number::date_span::DateSpan;
    /// # use meos::collections::base::span::Span;
    /// # use std::str::FromStr;
    ///
    /// let span: DateSpan = "(12, 67)".parse().expect("Failed to parse span");
    /// assert_eq!(span.lower(), 13);
    /// assert_eq!(span.upper(), 67);
    /// ```
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        CString::new(string)
            .map_err(|_| ParseSpanError)
            .map(|string| {
                let inner = unsafe { meos_sys::datespan_in(string.as_ptr()) };
                Self::from_inner(inner)
            })
    }
}

impl From<String> for DateSpan {
    /// Converts a `String` dateo a `DateSpan`.
    ///
    /// ## Arguments
    /// * `value` - A `String` containing the representation of a `DateSpan`.
    ///
    /// ## Returns
    /// * A `DateSpan` instance.
    ///
    /// ## Panics
    /// * Panics if the string cannot be parsed dateo a `DateSpan`.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::number::date_span::DateSpan;
    /// # use std::string::String;
    /// # use meos::collections::base::span::Span;
    ///
    /// let span_str = String::from("(12, 67)");
    /// let span: DateSpan = span_str.dateo();
    /// assert_eq!(span.lower(), 13);
    /// assert_eq!(span.upper(), 67);
    /// ```
    fn from(value: String) -> Self {
        DateSpan::from_str(&value).expect("Failed to parse the span")
    }
}

impl cmp::PartialEq for DateSpan {
    /// Checks if two `DateSpan` instances are equal.
    ///
    /// # Arguments
    /// * `other` - Another `DateSpan` instance.
    ///
    /// ## Returns
    /// * `true` if the spans are equal, `false` otherwise.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::number::date_span::DateSpan;
    /// # use meos::collections::base::span::Span;
    /// # use std::str::FromStr;
    ///
    /// let span1: DateSpan = (12..67).dateo();
    /// let span2: DateSpan = (12..67).dateo();
    /// assert_eq!(span1, span2);
    /// ```
    fn eq(&self, other: &Self) -> bool {
        unsafe { meos_sys::span_eq(self._inner, other._inner) }
    }
}

impl cmp::Eq for DateSpan {}

impl From<Range<NaiveDate>> for DateSpan {
    fn from(Range { start, end }: Range<NaiveDate>) -> Self {
        let inner = unsafe {
            meos_sys::datespan_make(
                start.num_days_from_ce(),
                end.num_days_from_ce(),
                true,
                false,
            )
        };
        Self::from_inner(inner)
    }
}

impl From<RangeInclusive<NaiveDate>> for DateSpan {
    fn from(range: RangeInclusive<NaiveDate>) -> Self {
        let inner = unsafe {
            meos_sys::datespan_make(
                range.start().num_days_from_ce(),
                range.end().num_days_from_ce(),
                true,
                true,
            )
        };
        Self::from_inner(inner)
    }
}

impl Debug for DateSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out_str = unsafe { meos_sys::datespan_out(self._inner) };
        let c_str = unsafe { CStr::from_ptr(out_str) };
        let str = c_str.to_str().map_err(|_| std::fmt::Error)?;
        let result = f.write_str(str);
        unsafe { meos_sys::free(out_str as *mut c_void) };
        result
    }
}

// Implement BitAnd for dateersection with DateSpan
impl BitAnd for DateSpan {
    type Output = Option<DateSpan>;
    /// Computes the dateersection of two `DateSpan` instances.
    ///
    /// # Arguments
    /// * `other` - Another `DateSpan` instance.
    ///
    /// ## Returns
    /// * An `Option<DateSpan>` containing the dateersection, or `None` if there is no dateersection.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::number::date_span::DateSpan;
    /// # use meos::collections::base::span::Span;
    /// # use std::str::FromStr;
    ///
    /// let span1: DateSpan = (12..67).dateo();
    /// let span2: DateSpan = (50..90).dateo();
    /// let dateersection = (span1 & span2).unwrap();
    ///
    /// assert_eq!(dateersection, (50..67).dateo())
    /// ```
    fn bitand(self, other: Self) -> Self::Output {
        // Replace with actual function call or logic
        let result = unsafe { meos_sys::intersection_span_span(self._inner, other._inner) };
        if !result.is_null() {
            Some(DateSpan::from_inner(result))
        } else {
            None
        }
    }
}

impl PartialOrd for DateSpan {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let cmp = unsafe { meos_sys::span_cmp(self._inner, other._inner) };
        match cmp {
            -1 => Some(cmp::Ordering::Less),
            0 => Some(cmp::Ordering::Equal),
            1 => Some(cmp::Ordering::Greater),
            _ => None,
        }
    }
}

impl Ord for DateSpan {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.partial_cmp(other).expect(
            "Unreachable since for non-null and same types spans, we only return -1, 0, or 1",
        )
    }
}
