use std::{
    cmp,
    ffi::{c_void, CStr, CString},
    fmt::Debug,
    hash::Hash,
    ops::{BitAnd, Range, RangeInclusive},
    ptr,
};

use crate::{collections::base::*, errors::ParseError};

use super::number_span::NumberSpan;

pub struct IntSpan {
    _inner: ptr::NonNull<meos_sys::Span>,
}

impl Drop for IntSpan {
    fn drop(&mut self) {
        unsafe {
            libc::free(self._inner.as_ptr() as *mut c_void);
        }
    }
}

impl Collection for IntSpan {
    impl_collection!(span, i32);
    fn contains(&self, content: &i32) -> bool {
        unsafe { meos_sys::contains_span_int(self.inner(), *content) }
    }
}

impl Span for IntSpan {
    type SubsetType = Self::Type;
    fn inner(&self) -> *const meos_sys::Span {
        self._inner.as_ptr()
    }

    /// Creates a new `IntSpan` from an inner pointer to a `meos_sys::Span`.
    ///
    /// # Arguments
    /// * `inner` - A pointer to the inner `meos_sys::Span`.
    ///
    /// ## Returns
    /// * A new `IntSpan` instance.
    fn from_inner(inner: *mut meos_sys::Span) -> Self {
        Self {
            _inner: ptr::NonNull::new(inner).expect("Null pointers not allowed"),
        }
    }

    /// Returns the lower bound of the span.
    ///
    /// ## Returns
    /// * The lower bound as a `i32`.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::number::int_span::IntSpan;
    /// # use meos::collections::base::span::Span;
    ///
    /// let span: IntSpan = (12..67).into();
    /// let lower = span.lower();
    /// ```
    fn lower(&self) -> Self::Type {
        unsafe { meos_sys::intspan_lower(self.inner()) }
    }

    /// Returns the upper bound of the span.
    ///
    /// ## Returns
    /// * The upper bound as a `i32`.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::number::int_span::IntSpan;
    /// # use meos::collections::base::span::Span;
    ///
    /// let span: IntSpan = (12..67).into();;
    ///
    /// assert_eq!(span.upper(), 67)
    /// ```
    fn upper(&self) -> Self::Type {
        unsafe { meos_sys::intspan_upper(self.inner()) }
    }

    /// Return a new `IntSpan` with the lower and upper bounds shifted by `delta`.
    ///
    /// # Arguments
    /// * `delta` - The value to shift by.
    ///
    /// # Returns
    /// A new `IntSpan` instance.
    ///
    /// # Example
    /// ```
    /// # use meos::collections::number::int_span::IntSpan;
    /// # use meos::collections::base::span::Span;
    ///
    /// let span: IntSpan = (12..67).into();
    /// let shifted_span = span.shift(5);
    ///
    /// assert_eq!(shifted_span, (17..72).into())
    /// ```
    fn shift(&self, delta: i32) -> IntSpan {
        self.shift_scale(Some(delta), None)
    }

    /// Return a new `IntSpan` with the lower and upper bounds scaled so that the width is `width`.
    ///
    /// # Arguments
    /// * `width` - The new width.
    ///
    /// # Returns
    /// A new `IntSpan` instance.
    ///
    /// # Example
    /// ```
    /// # use meos::collections::number::int_span::IntSpan;
    /// # use meos::collections::base::span::Span;
    ///
    /// let span: IntSpan = (12..67).into();
    /// let scaled_span = span.scale(10);
    ///
    /// assert_eq!(scaled_span, (12..23).into())
    /// ```
    fn scale(&self, width: i32) -> IntSpan {
        self.shift_scale(None, Some(width))
    }

    /// Return a new `IntSpan` with the lower and upper bounds shifted by `delta` and scaled so that the width is `width`.
    ///
    /// # Arguments
    /// * `delta` - The value to shift by.
    /// * `width` - The new width.
    ///
    /// # Returns
    /// A new `IntSpan` instance.
    ///
    /// # Example
    /// ```
    /// # use meos::collections::number::int_span::IntSpan;
    /// # use meos::collections::base::span::Span;
    ///
    /// let span: IntSpan = (12..67).into();
    /// let shifted_scaled_span = span.shift_scale(Some(5), Some(10));
    ///
    /// assert_eq!(shifted_scaled_span, (17..28).into())
    /// ```
    fn shift_scale(&self, delta: Option<i32>, width: Option<i32>) -> IntSpan {
        let d = delta.unwrap_or(0);
        let w = width.unwrap_or(0);
        let modified = unsafe {
            meos_sys::intspan_shift_scale(self.inner(), d, w, delta.is_some(), width.is_some())
        };
        IntSpan::from_inner(modified)
    }

    /// Calculates the distance between this `IntSpan` and an int.
    ///
    /// ## Arguments
    /// * `value` - An `i32` to calculate the distance to.
    ///
    /// ## Returns
    /// An `i32` representing the distance between the span and the value.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::number::int_span::IntSpan;
    /// # use meos::collections::base::span::Span;
    ///
    /// let span: IntSpan = (12..67).into();
    /// let distance = span.distance_to_value(&8);
    ///
    /// assert_eq!(distance, 4);
    /// ```
    fn distance_to_value(&self, value: &i32) -> i32 {
        unsafe { meos_sys::distance_span_int(self.inner(), *value) }
    }

    /// Calculates the distance between this `IntSpan` and another `IntSpan`.
    ///
    /// ## Arguments
    /// * `other` - An `IntSpan` to calculate the distance to.
    ///
    /// ## Returns
    /// An `i32` representing the distance between the two spans.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::number::int_span::IntSpan;
    /// # use meos::collections::base::span::Span;
    ///
    /// let span1: IntSpan = (12..67).into();
    /// let span2: IntSpan = (10..11).into();
    /// let distance = span1.distance_to_span(&span2);
    ///
    /// assert_eq!(distance, 2);
    /// ```
    fn distance_to_span(&self, other: &Self) -> i32 {
        unsafe { meos_sys::distance_intspan_intspan(self.inner(), other.inner()) }
    }
}

impl NumberSpan for IntSpan {}

impl Clone for IntSpan {
    fn clone(&self) -> Self {
        unsafe { Self::from_inner(meos_sys::span_copy(self.inner())) }
    }
}

impl Hash for IntSpan {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let hash = unsafe { meos_sys::span_hash(self.inner()) };
        state.write_u32(hash);

        let _ = state.finish();
    }
}

impl std::str::FromStr for IntSpan {
    type Err = ParseError;
    /// Parses a `IntSpan` from a string representation.
    ///
    /// ## Arguments
    /// * `string` - A string slice containing the representation.
    ///
    /// ## Returns
    /// * A `IntSpan` instance.
    ///
    /// ## Errors
    /// * Returns `ParseSpanError` if the string cannot be parsed.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::number::int_span::IntSpan;
    /// # use meos::collections::base::span::Span;
    /// # use std::str::FromStr;
    ///
    /// let span: IntSpan = "(12, 67)".parse().expect("Failed to parse span");
    /// assert_eq!(span.lower(), 13);
    /// assert_eq!(span.upper(), 67);
    /// ```
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        CString::new(string).map_err(|_| ParseError).map(|string| {
            let inner = unsafe { meos_sys::intspan_in(string.as_ptr()) };
            Self::from_inner(inner)
        })
    }
}

impl cmp::PartialEq for IntSpan {
    /// Checks if two `IntSpan` instances are equal.
    ///
    /// # Arguments
    /// * `other` - Another `IntSpan` instance.
    ///
    /// ## Returns
    /// * `true` if the spans are equal, `false` otherwise.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::number::int_span::IntSpan;
    /// # use meos::collections::base::span::Span;
    /// # use std::str::FromStr;
    ///
    /// let span1: IntSpan = (12..67).into();
    /// let span2: IntSpan = (12..67).into();
    /// assert_eq!(span1, span2);
    /// ```
    fn eq(&self, other: &Self) -> bool {
        unsafe { meos_sys::span_eq(self.inner(), other.inner()) }
    }
}

impl cmp::Eq for IntSpan {}

impl From<Range<i32>> for IntSpan {
    fn from(Range { start, end }: Range<i32>) -> Self {
        let inner = unsafe { meos_sys::intspan_make(start, end, true, false) };
        Self::from_inner(inner)
    }
}

impl From<RangeInclusive<i32>> for IntSpan {
    fn from(range: RangeInclusive<i32>) -> Self {
        let inner = unsafe { meos_sys::intspan_make(*range.start(), *range.end(), true, true) };
        Self::from_inner(inner)
    }
}

impl From<RangeInclusive<f32>> for IntSpan {
    fn from(range: RangeInclusive<f32>) -> Self {
        let inner = unsafe {
            meos_sys::intspan_make(*range.start() as i32, *range.end() as i32, true, true)
        };
        Self::from_inner(inner)
    }
}

impl Debug for IntSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out_str = unsafe { meos_sys::intspan_out(self.inner()) };
        let c_str = unsafe { CStr::from_ptr(out_str) };
        let str = c_str.to_str().map_err(|_| std::fmt::Error)?;
        let result = f.write_str(str);
        unsafe { libc::free(out_str as *mut c_void) };
        result
    }
}

// Implement BitAnd for intersection with IntSpan
impl BitAnd for IntSpan {
    type Output = Option<IntSpan>;
    /// Computes the intersection of two `IntSpan` instances.
    ///
    /// # Arguments
    /// * `other` - Another `IntSpan` instance.
    ///
    /// ## Returns
    /// * An `Option<IntSpan>` containing the intersection, or `None` if there is no intersection.
    ///
    /// ## Example
    /// ```
    /// # use meos::collections::number::int_span::IntSpan;
    /// # use meos::collections::base::span::Span;
    /// # use std::str::FromStr;
    ///
    /// let span1: IntSpan = (12..67).into();
    /// let span2: IntSpan = (50..90).into();
    /// let intersection = (span1 & span2).unwrap();
    ///
    /// assert_eq!(intersection, (50..67).into())
    /// ```
    fn bitand(self, other: Self) -> Self::Output {
        self.intersection(&other)
    }
}

impl PartialOrd for IntSpan {
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

impl Ord for IntSpan {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.partial_cmp(other).expect(
            "Unreachable since for non-null and same types spans, we only return -1, 0, or 1",
        )
    }
}
