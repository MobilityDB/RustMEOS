use std::ffi::{c_void, CStr, CString};

use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{BitAnd, BitOr};
use std::ptr;

use collection::{impl_collection, Collection};
use span_set::impl_iterator;

use crate::collections::base::span::Span;
use crate::collections::base::span_set::SpanSet;
use crate::collections::base::*;
use crate::errors::ParseError;

use super::float_span::FloatSpan;
use super::number_span_set::NumberSpanSet;

pub struct FloatSpanSet {
    _inner: ptr::NonNull<meos_sys::SpanSet>,
}

impl Drop for FloatSpanSet {
    fn drop(&mut self) {
        unsafe {
            libc::free(self._inner.as_ptr() as *mut c_void);
        }
    }
}

impl Collection for FloatSpanSet {
    impl_collection!(spanset, f64);

    fn contains(&self, content: &f64) -> bool {
        unsafe { meos_sys::contains_spanset_float(self.inner(), *content) }
    }
}

impl span_set::SpanSet for FloatSpanSet {
    type SpanType = FloatSpan;
    type SubsetType = <Self as Collection>::Type;
    fn inner(&self) -> *const meos_sys::SpanSet {
        self._inner.as_ptr()
    }

    fn from_inner(inner: *mut meos_sys::SpanSet) -> Self {
        Self {
            _inner: ptr::NonNull::new(inner).expect("Null pointers not allowed"),
        }
    }

    fn width(&self, ignore_gaps: bool) -> Self::Type {
        unsafe { meos_sys::floatspanset_width(self.inner(), ignore_gaps) }
    }

    /// Return a new `FloatSpanSet` with the lower and upper bounds shifted by `delta`.
    ///
    /// ## Arguments
    /// * `delta` - The value to shift by.
    ///
    /// ## Returns
    /// A new `FloatSpanSet` instance.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::number::float_span_set::FloatSpanSet;
    /// # use std::str::FromStr;
    /// # use meos::collections::base::span_set::SpanSet;
    ///
    /// let span = FloatSpanSet::from_str("{[17.5, 18.5), [19.5, 20.5)}").unwrap();
    /// let shifted_span = span.shift(5.0);
    ///
    /// let expected_shifted_span =
    ///     FloatSpanSet::from_str("{[22.5, 23.5), [24.5, 25.5)}").unwrap();
    /// assert_eq!(shifted_span, expected_shifted_span);
    /// ```
    fn shift(&self, delta: f64) -> FloatSpanSet {
        self.shift_scale(Some(delta), None)
    }

    /// Return a new `FloatSpanSet` with the lower and upper bounds scaled so that the width is `width`.
    ///
    /// ## Arguments
    /// * `width` - The new width.
    ///
    /// ## Returns
    /// A new `FloatSpanSet` instance.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::number::float_span_set::FloatSpanSet;
    /// # use std::str::FromStr;
    /// # use meos::collections::base::span_set::SpanSet;
    ///
    /// let span = FloatSpanSet::from_str("{[17.5, 18.5), [19.5, 20.5)}").unwrap();
    /// let scaled_span = span.scale(2.0);
    ///
    /// let expected_scaled_span =
    ///     FloatSpanSet::from_str("{[17.5, 18.1666666666666666666666), [18.833333333333333333333, 19.5)}").unwrap();
    /// assert_eq!(scaled_span, expected_scaled_span);
    /// ```
    fn scale(&self, width: f64) -> FloatSpanSet {
        self.shift_scale(None, Some(width))
    }

    /// Return a new `FloatSpanSet` with the lower and upper bounds shifted by `delta` and scaled so that the width is `width`.
    ///
    /// ## Arguments
    /// * `delta` - The value to shift by.
    /// * `width` - The new width.
    ///
    /// ## Returns
    /// A new `FloatSpanSet` instance.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::number::float_span_set::FloatSpanSet;
    /// # use std::str::FromStr;
    /// # use meos::collections::base::span_set::SpanSet;
    ///
    /// let span = FloatSpanSet::from_str("{[17.5, 18.5), [19.5, 20.5)}").unwrap();
    /// let shifted_scaled_span = span.shift_scale(Some(5.0), Some(2.5));
    ///
    /// let expected_shifted_scaled_span =
    ///     FloatSpanSet::from_str("{[22.5, 23.3333333333333333333), [24.16666666666666666, 25)}").unwrap();
    /// assert_eq!(shifted_scaled_span, expected_shifted_scaled_span);
    /// ```
    fn shift_scale(&self, delta: Option<f64>, width: Option<f64>) -> FloatSpanSet {
        let d = delta.unwrap_or(0.0);
        let w = width.unwrap_or(0.0);
        let modified = unsafe {
            meos_sys::floatspanset_shift_scale(self.inner(), d, w, delta.is_some(), width.is_some())
        };
        FloatSpanSet::from_inner(modified)
    }

    /// Calculates the distance between this `FloatSpanSet` and an integer (`value`).
    ///
    /// ## Arguments
    /// * `value` - An f64 to calculate the distance to.
    ///
    /// ## Returns
    /// An `f64` representing the distance between the span set and the value.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::number::float_span_set::FloatSpanSet;
    /// # use meos::collections::base::span_set::SpanSet;
    /// let span_set: FloatSpanSet = [(2019.0..2023.5).into(), (2029.0..2030.5).into()].iter().collect();
    /// let distance = span_set.distance_to_value(&2032.5);
    /// assert_eq!(distance, 2.0);
    /// ```
    fn distance_to_value(&self, other: &Self::Type) -> f64 {
        unsafe { meos_sys::distance_spanset_float(self.inner(), *other) }
    }

    /// Calculates the distance between this `FloatSpanSet` and another `FloatSpanSet`.
    ///
    /// ## Arguments
    /// * `other` - An `FloatSpanSet` to calculate the distance to.
    ///
    /// ## Returns
    /// An `f64` representing the distance between the two spansets.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::number::float_span_set::FloatSpanSet;
    /// # use meos::collections::base::span_set::SpanSet;
    /// # use meos::collections::base::span::Span;
    ///
    /// let span_set1: FloatSpanSet = [(2019.0..2023.5).into(), (2029.0..2030.5).into()].iter().collect();
    /// let span_set2: FloatSpanSet = [(2049.0..2050.5).into(), (2059.0..2600.5).into()].iter().collect();
    /// let distance = span_set1.distance_to_span_set(&span_set2);
    ///
    /// assert_eq!(distance, 18.5);
    fn distance_to_span_set(&self, other: &Self) -> f64 {
        unsafe { meos_sys::distance_floatspanset_floatspanset(self.inner(), other.inner()) }
    }

    /// Calculates the distance between this `FloatSpanSet` and a `FloatSpan`.
    ///
    /// ## Arguments
    /// * `other` - A `FloatSpan` to calculate the distance to.
    ///
    /// ## Returns
    /// A `TimeDelta` representing the distance in seconds between the span set and the span.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::number::float_span_set::FloatSpanSet;
    /// # use meos::collections::base::span_set::SpanSet;
    /// # use meos::collections::datetime::date_span::DateSpan;
    /// # use meos::collections::base::span::Span;
    /// # use meos::collections::number::float_span::FloatSpan;
    ///
    /// let span_set: FloatSpanSet = [(2019.0..2023.5).into(), (2029.0..2030.5).into()].iter().collect();
    /// let span: FloatSpan = (2009.0..2013.5).into();
    /// let distance = span_set.distance_to_span(&span);
    /// assert_eq!(distance, 5.5);
    /// ```
    fn distance_to_span(&self, span: &Self::SpanType) -> Self::SubsetType {
        unsafe { meos_sys::distance_floatspanset_floatspan(self.inner(), span.inner()) }
    }
}

impl NumberSpanSet for FloatSpanSet {}

impl Clone for FloatSpanSet {
    fn clone(&self) -> FloatSpanSet {
        self.copy()
    }
}

impl_iterator!(FloatSpanSet);

impl Hash for FloatSpanSet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let hash = unsafe { meos_sys::spanset_hash(self.inner()) };
        state.write_u32(hash);

        state.finish();
    }
}

impl std::str::FromStr for FloatSpanSet {
    type Err = ParseError;
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        CString::new(string).map_err(|_| ParseError).map(|string| {
            let inner = unsafe { meos_sys::floatspanset_in(string.as_ptr()) };
            Self::from_inner(inner)
        })
    }
}

impl std::cmp::PartialEq for FloatSpanSet {
    fn eq(&self, other: &Self) -> bool {
        unsafe { meos_sys::spanset_eq(self.inner(), other.inner()) }
    }
}

impl Debug for FloatSpanSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out_str = unsafe { meos_sys::floatspanset_out(self.inner(), 3) };
        let c_str = unsafe { CStr::from_ptr(out_str) };
        let str = c_str.to_str().map_err(|_| std::fmt::Error)?;
        let result = f.write_str(str);
        unsafe { libc::free(out_str as *mut c_void) };
        result
    }
}

impl BitAnd<FloatSpanSet> for FloatSpanSet {
    type Output = Option<FloatSpanSet>;
    /// Computes the intersection of two `FloatSpanSet`s.
    ///
    /// ## Arguments
    ///
    /// * `other` - Another `FloatSpanSet` to intersect with.
    ///
    /// ## Returns
    ///
    /// * `Some(FloatSpanSet)` - A new `FloatSpanSet` containing the intersection, if it exists.
    /// * `None` - If the intersection is empty.
    ///
    /// ## Example
    ///
    /// ```
    /// # use meos::collections::number::float_span_set::FloatSpanSet;
    /// # use std::str::FromStr;
    /// # use meos::collections::base::span_set::SpanSet;
    ///
    /// let span_set1 = FloatSpanSet::from_str("{[17.5, 18.5), [19.5, 20.5)}").unwrap();
    /// let span_set2 = FloatSpanSet::from_str("{[19.5, 23.5), [45.5, 67.5)}").unwrap();
    ///
    /// let expected_result = FloatSpanSet::from_str("{[19.5, 20.5)}").unwrap();
    /// assert_eq!((span_set1 & span_set2).unwrap(), expected_result);
    /// ```
    fn bitand(self, other: FloatSpanSet) -> Self::Output {
        self.intersection(&other)
    }
}

impl BitOr for FloatSpanSet {
    type Output = Option<FloatSpanSet>;
    /// Computes the union of two `FloatSpanSet`s.
    ///
    /// ## Arguments
    ///
    /// * `other` - Another `FloatSpanSet` to union with.
    ///
    /// ## Returns
    ///
    /// * `Some(FloatSpanSet)` - A new `FloatSpanSet` containing the union.
    /// * `None` - If the union is empty.
    ///
    /// ## Example
    ///
    /// ```
    /// # use meos::collections::number::float_span_set::FloatSpanSet;
    /// # use std::str::FromStr;
    /// # use meos::collections::base::span_set::SpanSet;
    ///
    /// let span_set1 = FloatSpanSet::from_str("{[17.5, 18.5), [19.5, 20.5)}").unwrap();
    /// let span_set2 = FloatSpanSet::from_str("{[19.5, 23.5), [45.5, 67.5)}").unwrap();
    ///
    /// let expected_result = FloatSpanSet::from_str("{[17.5, 18.5), [19.5, 23.5), [45.5, 67.5)}").unwrap();
    /// assert_eq!((span_set1 | span_set2).unwrap(), expected_result)
    /// ```
    fn bitor(self, other: Self) -> Self::Output {
        self.union(&other)
    }
}
