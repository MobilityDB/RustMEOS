use std::{
    cmp,
    ffi::{c_void, CStr, CString},
    fmt::Debug,
    hash::Hash,
    ops::{BitAnd, Range, RangeInclusive},
    ptr,
};

use collection::{impl_collection, Collection};
use span::Span;

use crate::{collections::base::*, errors::ParseError};

use super::number_span::NumberSpan;

pub struct FloatSpan {
    _inner: ptr::NonNull<meos_sys::Span>,
}

impl Drop for FloatSpan {
    fn drop(&mut self) {
        unsafe {
            libc::free(self._inner.as_ptr() as *mut c_void);
        }
    }
}

impl Collection for FloatSpan {
    impl_collection!(span, f64);
    fn contains(&self, content: &f64) -> bool {
        unsafe { meos_sys::contains_span_float(self.inner(), *content) }
    }
}

impl span::Span for FloatSpan {
    type SubsetType = Self::Type;
    fn inner(&self) -> *const meos_sys::Span {
        self._inner.as_ptr()
    }

    /// Creates a new `FloatSpan` from an inner pointer to a `meos_sys::Span`.
    ///
    /// # Arguments
    /// * `inner` - A pointer to the inner `meos_sys::Span`.
    ///
    /// ## Returns
    /// * A new `FloatSpan` instance.
    fn from_inner(inner: *mut meos_sys::Span) -> Self {
        Self {
            _inner: ptr::NonNull::new(inner).expect("Null pointers not allowed"),
        }
    }

    /// Returns the lower bound of the span.
    ///
    /// ## Returns
    /// * The lower bound as a `f64`.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::number::float_span::FloatSpan;
    /// # use meos::collections::base::span::Span;
    ///
    /// let span: FloatSpan = (12.9..67.8).into();
    /// let lower = span.lower();
    /// ```
    fn lower(&self) -> Self::Type {
        unsafe { meos_sys::floatspan_lower(self.inner()) }
    }

    /// Returns the upper bound of the span.
    ///
    /// ## Returns
    /// * The upper bound as a `f64`.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::number::float_span::FloatSpan;
    /// # use meos::collections::base::span::Span;
    ///
    /// let span: FloatSpan = (12.9..67.8).into();;
    ///
    /// assert_eq!(span.upper(), 67.8)
    /// ```
    fn upper(&self) -> Self::Type {
        unsafe { meos_sys::floatspan_upper(self.inner()) }
    }

    /// Return a new `FloatSpan` with the lower and upper bounds shifted by `delta`.
    ///
    /// # Arguments
    /// * `delta` - The value to shift by.
    ///
    /// # Returns
    /// A new `FloatSpan` instance.
    ///
    /// # Example
    /// ```
    /// # use meos::collections::number::float_span::FloatSpan;
    /// # use meos::collections::base::span::Span;
    ///
    /// let span: FloatSpan = (12.9..67.8).into();
    /// let shifted_span = span.shift(5.0);
    ///
    /// assert_eq!(shifted_span, (17.9..72.8).into())
    /// ```
    fn shift(&self, delta: f64) -> FloatSpan {
        self.shift_scale(Some(delta), None)
    }

    /// Return a new `FloatSpan` with the lower and upper bounds scaled so that the width is `width`.
    ///
    /// # Arguments
    /// * `width` - The new width.
    ///
    /// # Returns
    /// A new `FloatSpan` instance.
    ///
    /// # Example
    /// ```
    /// # use meos::collections::number::float_span::FloatSpan;
    /// # use meos::collections::base::span::Span;
    ///
    /// let span: FloatSpan = (12.9..67.8).into();
    /// let scaled_span = span.scale(10.0);
    ///
    /// assert_eq!(scaled_span, (12.9..22.9).into())
    /// ```
    fn scale(&self, width: f64) -> FloatSpan {
        self.shift_scale(None, Some(width))
    }

    /// Return a new `FloatSpan` with the lower and upper bounds shifted by `delta` and scaled so that the width is `width`.
    ///
    /// # Arguments
    /// * `delta` - The value to shift by.
    /// * `width` - The new width.
    ///
    /// # Returns
    /// A new `FloatSpan` instance.
    ///
    /// # Example
    /// ```
    /// # use meos::collections::number::float_span::FloatSpan;
    /// # use meos::collections::base::span::Span;
    ///
    /// let span: FloatSpan = (12.9..67.8).into();
    /// let shifted_scaled_span = span.shift_scale(Some(5.0), Some(10.0));
    ///
    /// assert_eq!(shifted_scaled_span, (17.9..27.9).into())
    /// ```
    fn shift_scale(&self, delta: Option<f64>, width: Option<f64>) -> FloatSpan {
        let d = delta.unwrap_or(0.0);
        let w = width.unwrap_or(0.0);
        let modified = unsafe {
            meos_sys::floatspan_shift_scale(self.inner(), d, w, delta.is_some(), width.is_some())
        };
        FloatSpan::from_inner(modified)
    }

    fn distance_to_value(&self, other: &Self::Type) -> f64 {
        unsafe { meos_sys::distance_span_float(self.inner(), *other) }
    }

    fn distance_to_span(&self, other: &Self) -> f64 {
        unsafe { meos_sys::distance_floatspan_floatspan(self.inner(), other.inner()) }
    }
}

impl NumberSpan for FloatSpan {}

impl Clone for FloatSpan {
    fn clone(&self) -> Self {
        unsafe { Self::from_inner(meos_sys::span_copy(self.inner())) }
    }
}

impl Hash for FloatSpan {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let hash = unsafe { meos_sys::span_hash(self.inner()) };
        state.write_u32(hash);

        state.finish();
    }
}

impl std::str::FromStr for FloatSpan {
    type Err = ParseError;
    /// Parses a `FloatSpan` from a string representation.
    ///
    /// ## Arguments
    /// * `string` - A string slice containing the representation.
    ///
    /// ## Returns
    /// * A `FloatSpan` instance.
    ///
    /// ## Errors
    /// * Returns `ParseSpanError` if the string cannot be parsed.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::number::float_span::FloatSpan;
    /// # use meos::collections::base::span::Span;
    /// # use std::str::FromStr;
    ///
    /// let span: FloatSpan = "(12.9, 67.8)".parse().expect("Failed to parse span");
    /// assert_eq!(span.lower(), 12.9);
    /// assert_eq!(span.upper(), 67.8);
    /// ```
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        CString::new(string).map_err(|_| ParseError).map(|string| {
            let inner = unsafe { meos_sys::floatspan_in(string.as_ptr()) };
            Self::from_inner(inner)
        })
    }
}

impl cmp::PartialEq for FloatSpan {
    /// Checks if two `FloatSpan` instances are equal.
    ///
    /// # Arguments
    /// * `other` - Another `FloatSpan` instance.
    ///
    /// ## Returns
    /// * `true` if the spans are equal, `false` otherwise.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::number::float_span::FloatSpan;
    /// # use meos::collections::base::span::Span;
    /// # use std::str::FromStr;
    ///
    /// let span1: FloatSpan = (12.9..67.8).into();
    /// let span2: FloatSpan = (12.9..67.8).into();
    /// assert_eq!(span1, span2);
    /// ```
    fn eq(&self, other: &Self) -> bool {
        unsafe { meos_sys::span_eq(self.inner(), other.inner()) }
    }
}

impl cmp::Eq for FloatSpan {}

impl From<Range<f64>> for FloatSpan {
    fn from(Range { start, end }: Range<f64>) -> Self {
        let inner = unsafe { meos_sys::floatspan_make(start, end, true, false) };
        Self::from_inner(inner)
    }
}

impl From<Range<f32>> for FloatSpan {
    fn from(Range { start, end }: Range<f32>) -> Self {
        let inner = unsafe { meos_sys::floatspan_make(start as f64, end as f64, true, false) };
        Self::from_inner(inner)
    }
}

impl From<RangeInclusive<f64>> for FloatSpan {
    fn from(range: RangeInclusive<f64>) -> Self {
        let inner = unsafe { meos_sys::floatspan_make(*range.start(), *range.end(), true, true) };
        Self::from_inner(inner)
    }
}

impl From<RangeInclusive<f32>> for FloatSpan {
    fn from(range: RangeInclusive<f32>) -> Self {
        let inner = unsafe {
            meos_sys::floatspan_make(*range.start() as f64, *range.end() as f64, true, true)
        };
        Self::from_inner(inner)
    }
}

impl Debug for FloatSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out_str = unsafe { meos_sys::floatspan_out(self.inner(), 3) };
        // Default of 3 decimal digits
        let c_str = unsafe { CStr::from_ptr(out_str) };
        let str = c_str.to_str().map_err(|_| std::fmt::Error)?;
        let result = f.write_str(str);
        unsafe { libc::free(out_str as *mut c_void) };
        result
    }
}

// Implement BitAnd for intersection with FloatSpan
impl BitAnd for FloatSpan {
    type Output = Option<FloatSpan>;
    /// Computes the intersection of two `FloatSpan` instances.
    ///
    /// # Arguments
    /// * `other` - Another `FloatSpan` instance.
    ///
    /// ## Returns
    /// * An `Option<FloatSpan>` containing the intersection, or `None` if there is no intersection.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::number::float_span::FloatSpan;
    /// # use meos::collections::base::span::Span;
    /// # use std::str::FromStr;
    ///
    /// let span1: FloatSpan = (12.9..67.8).into();
    /// let span2: FloatSpan = (50.0..80.0).into();
    /// let intersection = (span1 & span2).unwrap();
    ///
    /// assert_eq!(intersection, (50.0..67.8).into())
    /// ```
    fn bitand(self, other: FloatSpan) -> Self::Output {
        self.intersection(&other)
    }
}

impl PartialOrd for FloatSpan {
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

impl Ord for FloatSpan {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.partial_cmp(other).expect(
            "Unreachable since for non-null and same types spans, we only return -1, 0, or 1",
        )
    }
}
