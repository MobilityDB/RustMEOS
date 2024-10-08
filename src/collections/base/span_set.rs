use std::{
    ffi::{CStr, CString},
    ptr,
};

use crate::WKBVariant;

use super::{collection::Collection, span::Span};

pub trait SpanSet: Collection + FromIterator<Self::SpanType> {
    type SpanType: Span;
    /// Type used to represent subsets (duration, widths, etc.)
    type SubsetType;
    fn inner(&self) -> *const meos_sys::SpanSet;

    /// Creates a new `Span` from a WKB representation.
    ///
    /// ## Arguments
    /// * `hexwkb` - A string slice containing the WKB representation.
    ///
    /// ## Returns
    /// * A new `Span` instance.
    fn from_wkb(wkb: &[u8]) -> Self {
        let span = unsafe { meos_sys::spanset_from_wkb(wkb.as_ptr(), wkb.len()) };
        Self::from_inner(span)
    }

    /// Creates a new `Span` from a hexadecimal WKB representation.
    ///
    /// ## Arguments
    /// * `hexwkb` - A string slice containing the hexadecimal WKB representation.
    ///
    /// ## Returns
    /// * A new `Span` instance.
    fn from_hexwkb(hexwkb: &[u8]) -> Self {
        let c_string = CString::new(hexwkb).expect("Cannot create CString");
        let span = unsafe { meos_sys::spanset_from_hexwkb(c_string.as_ptr()) };
        Self::from_inner(span)
    }

    fn copy(&self) -> Self {
        let inner = unsafe { meos_sys::spanset_copy(self.inner()) };
        Self::from_inner(inner)
    }

    fn from_inner(inner: *mut meos_sys::SpanSet) -> Self;

    fn as_wkb(&self, variant: WKBVariant) -> &[u8] {
        unsafe {
            let mut size = 0;
            let wkb =
                meos_sys::spanset_as_wkb(self.inner(), variant.into(), ptr::addr_of_mut!(size));
            std::slice::from_raw_parts(wkb, size)
        }
    }

    fn as_hexwkb(&self, variant: WKBVariant) -> &[u8] {
        unsafe {
            let mut size = 0;
            let wkb =
                meos_sys::spanset_as_hexwkb(self.inner(), variant.into(), ptr::addr_of_mut!(size));
            CStr::from_ptr(wkb).to_bytes()
        }
    }

    fn num_spans(&self) -> i32 {
        unsafe { meos_sys::spanset_num_spans(self.inner()) }
    }

    fn start_span(&self) -> Self::SpanType {
        let span = unsafe { meos_sys::spanset_start_span(self.inner()) };
        Span::from_inner(span)
    }

    fn end_span(&self) -> Self::SpanType {
        let span = unsafe { meos_sys::spanset_end_span(self.inner()) };
        Span::from_inner(span)
    }

    fn span_n(&self, n: i32) -> Self::SpanType {
        let span = unsafe { meos_sys::spanset_span_n(self.inner(), n) };
        Span::from_inner(span)
    }

    fn spans(&self) -> Vec<Self::SpanType> {
        let spans = unsafe { meos_sys::spanset_spanarr(self.inner()) };
        let size = self.num_spans() as usize;
        unsafe {
            Vec::from_raw_parts(spans, size, size)
                .iter()
                .map(|&span| Span::from_inner(span))
                .collect()
        }
    }

    fn width(&self, ignore_gaps: bool) -> Self::Type;

    /// Return a new `SpanSet` with the lower and upper bounds shifted by `delta`.
    fn shift(&self, delta: Self::SubsetType) -> Self;

    /// Return a new `SpanSet` with the lower and upper bounds scaled so that the width is `width`.
    fn scale(&self, width: Self::SubsetType) -> Self;

    /// Return a new `SpanSet` with the lower and upper bounds shifted by `delta` and scaled so that the width is `width`.
    fn shift_scale(&self, delta: Option<Self::SubsetType>, width: Option<Self::SubsetType>)
        -> Self;

    fn intersection(&self, other: &Self) -> Option<Self> {
        let result = unsafe { meos_sys::intersection_spanset_spanset(self.inner(), other.inner()) };
        if !result.is_null() {
            Some(Self::from_inner(result))
        } else {
            None
        }
    }

    fn union(&self, other: &Self) -> Option<Self> {
        let result = unsafe { meos_sys::union_spanset_spanset(self.inner(), other.inner()) };
        if !result.is_null() {
            Some(Self::from_inner(result))
        } else {
            None
        }
    }

    fn hash(&self) -> u32 {
        unsafe { meos_sys::spanset_hash(self.inner()) }
    }

    fn distance_to_value(&self, value: &Self::Type) -> Self::SubsetType;

    fn distance_to_span_set(&self, other: &Self) -> Self::SubsetType;

    fn distance_to_span(&self, span: &Self::SpanType) -> Self::SubsetType;
}

macro_rules! impl_iterator {
    ($type:ty) => {
        impl IntoIterator for $type {
            type Item = <$type as SpanSet>::SpanType;

            type IntoIter = std::vec::IntoIter<Self::Item>;

            fn into_iter(self) -> Self::IntoIter {
                self.spans().into_iter()
            }
        }

        impl FromIterator<<$type as SpanSet>::SpanType> for $type {
            fn from_iter<T: IntoIterator<Item = <$type as SpanSet>::SpanType>>(iter: T) -> Self {
                iter.into_iter().collect()
            }
        }

        impl<'a> FromIterator<&'a <$type as SpanSet>::SpanType> for $type {
            fn from_iter<T: IntoIterator<Item = &'a <$type as SpanSet>::SpanType>>(
                iter: T,
            ) -> Self {
                let mut iter = iter.into_iter();
                let first = iter.next().unwrap();
                iter.fold(first.to_spanset(), |acc, item| {
                    (acc | item.to_spanset()).unwrap()
                })
            }
        }
    };
}

pub(crate) use impl_iterator;
