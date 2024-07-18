use std::ffi::CString;

use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{BitAnd, BitOr};
use std::str::FromStr;

use collection::{generate_collection_methods, Collection};

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
            let _ = Box::from_raw(self._inner);
        }
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
}

impl Collection for FloatSpanSet {
    generate_collection_methods!(spanset, float, f64);
}

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
        // Default of 3 decimal digits
        let result = unsafe { CString::from_raw(meos_sys::floatspanset_out(self._inner, 3)) };
        if let Ok(str) = result.to_str() {
            f.write_str(str)
        } else {
            Err(std::fmt::Error)
        }
    }
}

impl BitAnd<FloatSpanSet> for FloatSpanSet {
    type Output = Option<FloatSpanSet>;
    /// Computes the intersection of two `FloatSpanSet`s.
    ///
    /// # Arguments
    ///
    /// * `other` - Another `FloatSpanSet` to intersect with.
    ///
    /// # Returns
    ///
    /// * `Some(FloatSpanSet)` - A new `FloatSpanSet` containing the intersection, if it exists.
    /// * `None` - If the intersection is empty.
    ///
    /// # Example
    ///
    /// ```
    /// # use meos::collections::number::float_span::{FloatSpanSet, FloatSpan};
    /// # use meos::collections::base::span::Span;
    ///
    /// let span_set1 = FloatSpanSet::new();
    /// let span_set2 = FloatSpanSet::new();
    /// todo!()
    /// // Perform intersection
    /// if let Some(intersection_set) = span_set1.clone() & span_set2.clone() {
    ///     println!("Intersection: {:?}", intersection_set);
    /// } else {
    ///     println!("No intersection");
    /// }
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
    /// # Arguments
    ///
    /// * `other` - Another `FloatSpanSet` to union with.
    ///
    /// # Returns
    ///
    /// * `Some(FloatSpanSet)` - A new `FloatSpanSet` containing the union.
    /// * `None` - If the union is empty.
    ///
    /// # Example
    ///
    /// ```
    /// # use meos::collections::number::float_span::{FloatSpanSet, FloatSpan};
    /// # use meos::collections::base::span::Span;
    /// todo!()
    /// let span_set1 = FloatSpanSet::new();
    /// let span_set2 = FloatSpanSet::new();
    ///
    /// // Perform union
    /// if let Some(union_set) = span_set1.clone() | span_set2.clone() {
    ///     println!("Union: {:?}", union_set);
    /// } else {
    ///     println!("No union");
    /// }
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
