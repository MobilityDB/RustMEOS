use std::ffi::{c_void, CStr, CString};

use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{BitAnd, BitOr};

use collection::{impl_collection, Collection};
use span::Span;
use span_set::impl_iterator;

use crate::collections::base::span_set::SpanSet;
use crate::collections::base::*;
use crate::errors::ParseError;

use super::int_span::IntSpan;
use super::number_span_set::NumberSpanSet;

pub struct IntSpanSet {
    _inner: *const meos_sys::SpanSet,
}

impl Drop for IntSpanSet {
    fn drop(&mut self) {
        unsafe {
            libc::free(self._inner as *mut c_void);
        }
    }
}

impl Collection for IntSpanSet {
    impl_collection!(spanset, i32);
    fn contains(&self, content: &i32) -> bool {
        unsafe { meos_sys::contains_spanset_int(self.inner(), *content) }
    }
}

impl span_set::SpanSet for IntSpanSet {
    type SpanType = IntSpan;
    type SubsetType = <Self as Collection>::Type;
    fn inner(&self) -> *const meos_sys::SpanSet {
        self._inner
    }

    fn from_inner(inner: *const meos_sys::SpanSet) -> Self {
        Self { _inner: inner }
    }

    fn width(&self, ignore_gaps: bool) -> Self::Type {
        unsafe { meos_sys::intspanset_width(self.inner(), ignore_gaps) }
    }

    /// Return a new `IntSpanSet` with the lower and upper bounds shifted by `delta`.
    ///
    /// ## Arguments
    /// * `delta` - The value to shift by.
    ///
    /// ## Returns
    /// A new `IntSpanSet` instance.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::number::int_span_set::IntSpanSet;
    /// # use std::str::FromStr;
    /// # use meos::collections::base::span_set::SpanSet;
    ///
    /// let span = IntSpanSet::from_str("{[17, 18), [19, 20)}").unwrap();
    /// let shifted_span = span.shift(5);
    ///
    /// let expected_shifted_span =
    ///     IntSpanSet::from_str("{[22, 23), [24, 25)}").unwrap();
    /// assert_eq!(shifted_span, expected_shifted_span);
    /// ```
    fn shift(&self, delta: i32) -> IntSpanSet {
        self.shift_scale(Some(delta), None)
    }

    /// Return a new `IntSpanSet` with the lower and upper bounds scaled so that the width is `width`.
    ///
    /// ## Arguments
    /// * `width` - The new width.
    ///
    /// ## Returns
    /// A new `IntSpanSet` instance.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::number::int_span_set::IntSpanSet;
    /// # use std::str::FromStr;
    /// # use meos::collections::base::span_set::SpanSet;
    ///
    /// let span = IntSpanSet::from_str("{[17, 18), [19, 23)}").unwrap();
    /// let scaled_span = span.scale(5);
    ///
    /// let expected_scaled_span =
    ///     IntSpanSet::from_str("{[17, 18), [19, 23)}").unwrap();
    /// assert_eq!(scaled_span, expected_scaled_span);
    /// ```
    fn scale(&self, width: i32) -> IntSpanSet {
        self.shift_scale(None, Some(width))
    }

    /// Return a new `IntSpanSet` with the lower and upper bounds shifted by `delta` and scaled so that the width is `width`.
    ///
    /// ## Arguments
    /// * `delta` - The value to shift by.
    /// * `width` - The new width.
    ///
    /// ## Returns
    /// A new `IntSpanSet` instance.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::number::int_span_set::IntSpanSet;
    /// # use std::str::FromStr;
    /// # use meos::collections::base::span_set::SpanSet;
    ///
    /// let span = IntSpanSet::from_str("{[17, 18), [19, 20)}").unwrap();
    /// let shifted_scaled_span = span.shift_scale(Some(5), Some(2));
    ///
    /// let expected_shifted_scaled_span =
    ///     IntSpanSet::from_str("{[22, 23), [24, 25)}").unwrap();
    /// assert_eq!(shifted_scaled_span, expected_shifted_scaled_span);
    /// ```
    fn shift_scale(&self, delta: Option<i32>, width: Option<i32>) -> IntSpanSet {
        let d = delta.unwrap_or(0);
        let w = width.unwrap_or(0);
        let modified = unsafe {
            meos_sys::intspanset_shift_scale(self._inner, d, w, delta.is_some(), width.is_some())
        };
        IntSpanSet::from_inner(modified)
    }

    /// Calculates the distance between this `IntSpanSet` and an integer (`value`).
    ///
    /// ## Arguments
    /// * `value` - An i32 to calculate the distance to.
    ///
    /// ## Returns
    /// An `i32` representing the distance between the span set and the value.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::number::int_span_set::IntSpanSet;
    /// # use meos::collections::base::span_set::SpanSet;
    /// let span_set: IntSpanSet = [(2019..2023).into(), (2029..2030).into()].iter().collect();
    /// let distance = span_set.distance_to_value(&2032);
    /// assert_eq!(distance, 3);
    /// ```
    fn distance_to_value(&self, value: &Self::Type) -> i32 {
        unsafe { meos_sys::distance_spanset_int(self.inner(), *value) }
    }

    /// Calculates the distance between this `IntSpanSet` and another `IntSpanSet`.
    ///
    /// ## Arguments
    /// * `other` - An `IntSpanSet` to calculate the distance to.
    ///
    /// ## Returns
    /// An `i32` representing the distance between the two spansets.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::number::int_span_set::IntSpanSet;
    /// # use meos::collections::base::span_set::SpanSet;
    /// # use meos::collections::base::span::Span;
    ///
    /// let span_set1: IntSpanSet = [(2019..2023).into(), (2029..2030).into()].iter().collect();
    /// let span_set2: IntSpanSet = [(2049..2050).into(), (2059..2600).into()].iter().collect();
    /// let distance = span_set1.distance_to_span_set(&span_set2);
    ///
    /// assert_eq!(distance, 20);
    /// ```
    fn distance_to_span_set(&self, other: &Self) -> i32 {
        unsafe { meos_sys::distance_intspanset_intspanset(self.inner(), other.inner()) }
    }

    /// Calculates the distance between this `IntSpanSet` and a `IntSpan`.
    ///
    /// ## Arguments
    /// * `other` - A `IntSpan` to calculate the distance to.
    ///
    /// ## Returns
    /// A `TimeDelta` representing the distance in seconds between the span set and the span.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::number::int_span_set::IntSpanSet;
    /// # use meos::collections::base::span_set::SpanSet;
    /// # use meos::collections::base::span::Span;
    /// # use meos::collections::number::int_span::IntSpan;
    ///
    /// let span_set: IntSpanSet = [(2019..2023).into(), (2029..2030).into()].iter().collect();
    /// let span: IntSpan = (2009..2010).into();
    /// let distance = span_set.distance_to_span(&span);
    /// assert_eq!(distance, 10);
    /// ```
    fn distance_to_span(&self, span: &Self::SpanType) -> Self::SubsetType {
        unsafe { meos_sys::distance_intspanset_intspan(self.inner(), span.inner()) }
    }
}

impl NumberSpanSet for IntSpanSet {}

impl Clone for IntSpanSet {
    fn clone(&self) -> IntSpanSet {
        self.copy()
    }
}

impl_iterator!(IntSpanSet);

impl Hash for IntSpanSet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let hash = unsafe { meos_sys::spanset_hash(self._inner) };
        state.write_u32(hash);

        state.finish();
    }
}

impl std::str::FromStr for IntSpanSet {
    type Err = ParseError;
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        CString::new(string).map_err(|_| ParseError).map(|string| {
            let inner = unsafe { meos_sys::intspanset_in(string.as_ptr()) };
            Self::from_inner(inner)
        })
    }
}

impl std::cmp::PartialEq for IntSpanSet {
    fn eq(&self, other: &Self) -> bool {
        unsafe { meos_sys::spanset_eq(self._inner, other._inner) }
    }
}

impl Debug for IntSpanSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out_str = unsafe { meos_sys::intspanset_out(self._inner) };
        let c_str = unsafe { CStr::from_ptr(out_str) };
        let str = c_str.to_str().map_err(|_| std::fmt::Error)?;
        let result = f.write_str(str);
        unsafe { libc::free(out_str as *mut c_void) };
        result
    }
}

impl BitAnd<IntSpanSet> for IntSpanSet {
    type Output = Option<IntSpanSet>;
    /// Computes the intersection of two `IntSpanSet`s.
    ///
    /// ## Arguments
    ///
    /// * `other` - Another `IntSpanSet` to intersect with.
    ///
    /// ## Returns
    ///
    /// * `Some(IntSpanSet)` - A new `IntSpanSet` containing the intersection, if it exists.
    /// * `None` - If the intersection is empty.
    ///
    /// ## Example
    ///
    /// ```
    /// # use meos::collections::number::int_span_set::IntSpanSet;
    /// # use std::str::FromStr;
    /// # use meos::collections::base::span_set::SpanSet;
    ///
    /// let span_set1 = IntSpanSet::from_str("{[17, 18), [19, 20)}").unwrap();
    /// let span_set2 = IntSpanSet::from_str("{[19, 23), [45, 67)}").unwrap();
    ///
    /// let expected_result = IntSpanSet::from_str("{[19, 20)}").unwrap();
    /// assert_eq!((span_set1 & span_set2).unwrap(), expected_result);
    /// ```
    fn bitand(self, other: IntSpanSet) -> Self::Output {
        self.intersection(&other)
    }
}

impl BitOr for IntSpanSet {
    type Output = Option<IntSpanSet>;
    /// Computes the union of two `IntSpanSet`s.
    ///
    /// ## Arguments
    ///
    /// * `other` - Another `IntSpanSet` to union with.
    ///
    /// ## Returns
    ///
    /// * `Some(IntSpanSet)` - A new `IntSpanSet` containing the union.
    /// * `None` - If the union is empty.
    ///
    /// ## Example
    ///
    /// ```
    /// # use meos::collections::number::int_span_set::IntSpanSet;
    /// # use std::str::FromStr;
    /// # use meos::collections::base::span_set::SpanSet;
    ///
    /// let span_set1 = IntSpanSet::from_str("{[17, 18), [19, 20)}").unwrap();
    /// let span_set2 = IntSpanSet::from_str("{[19, 23), [45, 67)}").unwrap();
    ///
    /// let expected_result = IntSpanSet::from_str("{[17, 18), [19, 23), [45, 67)}").unwrap();
    /// assert_eq!((span_set1 | span_set2).unwrap(), expected_result)
    /// ```
    fn bitor(self, other: Self) -> Self::Output {
        self.union(&other)
    }
}
