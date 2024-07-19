use std::ffi::{c_void, CStr, CString};

use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{BitAnd, BitOr};
use std::str::FromStr;

use collection::{impl_collection, Collection};

use crate::collections::base::span_set::SpanSet;
use crate::collections::base::*;
use crate::errors::ParseSpanError;

use super::float_span::FloatSpan;

pub struct FloatSpanSet {
    _inner: *mut meos_sys::SpanSet,
}

impl Drop for FloatSpanSet {
    fn drop(&mut self) {
        unsafe {
            meos_sys::free(self._inner as *mut c_void);
        }
    }
}

impl Collection for FloatSpanSet {
    impl_collection!(spanset, float, f64);

    fn contains(&self, content: &f64) -> bool {
        unsafe { meos_sys::contains_spanset_float(self.inner(), *content) }
    }
}

impl span_set::SpanSet for FloatSpanSet {
    type SpanType = FloatSpan;
    fn inner(&self) -> *const meos_sys::SpanSet {
        self._inner
    }

    fn from_inner(inner: *mut meos_sys::SpanSet) -> Self
    where
        Self: Sized,
    {
        Self { _inner: inner }
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
            meos_sys::floatspanset_shift_scale(self._inner, d, w, delta.is_some(), width.is_some())
        };
        FloatSpanSet::from_inner(modified)
    }
}

// impl IntoIterator for FloatSpanSet {
//     type Item = FloatSpan;

//     type IntoIter = std::vec::IntoIter<Self::Item>;

//     fn into_iter(self) -> Self::IntoIter {
//         todo!()
//     }
// }

// impl FromIterator<FloatSpan> for FloatSpanSet {
//     fn from_iter<T: IntoIterator<Item = FloatSpan>>(iter: T) -> Self {
//         let mut iter = iter.into_iter();
//         let first = iter.next().unwrap();
//         iter.fold(first.to_spanset(), |acc, item| {
//             (acc | item.to_spanset()).unwrap()
//         })
//     }
// }

// impl<'a> FromIterator<&'a FloatSpan> for FloatSpanSet {
//     fn from_iter<T: IntoIterator<Item = &'a FloatSpan>>(iter: T) -> Self {
//         iter.into_iter().collect()
//     }
// }

impl Hash for FloatSpanSet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let hash = unsafe { meos_sys::spanset_hash(self._inner) };
        state.write_u32(hash);

        state.finish();
    }
}

impl std::str::FromStr for FloatSpanSet {
    type Err = ParseSpanError;
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        CString::new(string)
            .map_err(|_| ParseSpanError)
            .map(|string| {
                let inner = unsafe { meos_sys::floatspanset_in(string.as_ptr()) };
                Self::from_inner(inner)
            })
    }
}

impl From<String> for FloatSpanSet {
    fn from(value: String) -> Self {
        FloatSpanSet::from_str(&value).expect("Failed to parse the span")
    }
}

impl std::cmp::PartialEq for FloatSpanSet {
    fn eq(&self, other: &Self) -> bool {
        unsafe { meos_sys::spanset_eq(self._inner, other._inner) }
    }
}

impl Debug for FloatSpanSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out_str = unsafe { meos_sys::floatspanset_out(self._inner, 3) };
        let c_str = unsafe { CStr::from_ptr(out_str) };
        let str = c_str.to_str().map_err(|_| std::fmt::Error)?;
        let result = f.write_str(str);
        unsafe { meos_sys::free(out_str as *mut c_void) };
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
        // Replace with actual function call or logic
        let result = unsafe { meos_sys::intersection_spanset_spanset(other.inner(), self._inner) };
        if !result.is_null() {
            Some(FloatSpanSet::from_inner(result))
        } else {
            None
        }
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
        // Replace with actual function call or logic
        let result = unsafe { meos_sys::union_spanset_spanset(self._inner, other._inner) };
        if !result.is_null() {
            Some(FloatSpanSet::from_inner(result))
        } else {
            None
        }
    }
}
