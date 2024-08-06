use std::{
    ffi::{c_void, CStr, CString},
    fmt::Debug,
    hash::Hash,
    ptr,
    str::FromStr,
};

use chrono::{DateTime, TimeZone};

use crate::{
    boxes::tbox::TBox,
    collections::{
        base::{
            collection::{impl_collection, Collection},
            span::Span,
            span_set::SpanSet,
        },
        datetime::{tstz_span::TsTzSpan, tstz_span_set::TsTzSpanSet},
        number::int_span_set::IntSpanSet,
    },
    errors::ParseError,
    temporal::{
        temporal::{
            impl_always_and_ever_value_equality_functions,
            impl_always_and_ever_value_functions_with_ordering, impl_simple_traits_for_temporal,
            OrderedTemporal, Temporal,
        },
        tinstant::TInstant,
        tsequence::TSequence,
        tsequence_set::TSequenceSet,
    },
    utils::to_meos_timestamp,
};

use super::tnumber::{impl_temporal_for_tnumber, TNumber};

pub trait TInt:
    Temporal<Type = i32, TI = TIntInst, TS = TIntSeq, TSS = TIntSeqSet, TBB = TBox>
{
    // ------------------------- Transformations -------------------------------

    /// Returns a new `TNumber` with the value dimension shifted by `shift` and scaled so the value dimension has width `width`.
    ///
    /// # Arguments
    /// * `shift` - Value to shift
    /// * `width` - Value representing the width of the new temporal number
    ///
    /// # Safety
    /// This function uses unsafe code to call the `meos_sys::tint_shift_scale_value` or
    /// `meos_sys::tfloat_shift_scale_value` functions.
    fn shift_scale_value(&self, shift: Option<Self::Type>, width: Option<Self::Type>) -> Self {
        let d = shift.unwrap_or_default();
        let w = width.unwrap_or_default();
        let modified = unsafe { meos_sys::tint_shift_scale_value(self.inner(), d, w) };
        Self::from_inner_as_temporal(modified)
    }
}

macro_rules! impl_debug {
    ($type:ty) => {
        impl Debug for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let out_str = unsafe { meos_sys::tint_out(self.inner()) };
                let c_str = unsafe { CStr::from_ptr(out_str) };
                let str = c_str.to_str().map_err(|_| std::fmt::Error)?;
                let result = f.write_str(str);
                unsafe { libc::free(out_str as *mut c_void) };
                result
            }
        }
    };
}

pub struct TIntInst {
    _inner: *const meos_sys::TInstant,
}

impl TInstant for TIntInst {
    fn from_inner(inner: *mut meos_sys::TInstant) -> Self {
        Self { _inner: inner }
    }

    fn inner_as_tinstant(&self) -> *const meos_sys::TInstant {
        self._inner
    }

    fn from_value_and_timestamp<Tz: TimeZone>(value: Self::Type, timestamp: DateTime<Tz>) -> Self {
        Self::from_inner(unsafe { meos_sys::tintinst_make(value, to_meos_timestamp(&timestamp)) })
    }
}

impl TInt for TIntInst {}

impl_temporal_for_tnumber!(TIntInst, meos_sys::TInstant, i32, Int);
impl_debug!(TIntInst);

pub struct TIntSeq {
    _inner: *const meos_sys::TSequence,
}
impl TIntSeq {
    /// Creates a temporal object from a value and a TsTz span.
    ///
    /// ## Arguments
    /// * `value` - Base value.
    /// * `time_span` - Time object to use as the temporal dimension.
    ///
    /// ## Returns
    /// A new temporal object.
    pub fn from_value_and_tstz_span<Tz: TimeZone>(value: i32, time_span: TsTzSpan) -> Self {
        Self::from_inner(unsafe { meos_sys::tintseq_from_base_tstzspan(value, time_span.inner()) })
    }
}

impl TSequence for TIntSeq {
    fn from_inner(inner: *const meos_sys::TSequence) -> Self {
        Self { _inner: inner }
    }

    fn inner_as_tsequence(&self) -> *const meos_sys::TSequence {
        self._inner
    }
}

impl TInt for TIntSeq {}

impl_temporal_for_tnumber!(TIntSeq, meos_sys::TSequence, i32, Int);
impl_debug!(TIntSeq);

pub struct TIntSeqSet {
    _inner: *const meos_sys::TSequenceSet,
}

impl TIntSeqSet {
    /// Creates a temporal object from a base value and a TsTz span set.
    ///
    /// ## Arguments
    /// * `value` - Base value.
    /// * `time_span_set` - Time object to use as the temporal dimension.
    ///
    /// ## Returns
    /// A new temporal object.
    pub fn from_value_and_tstz_span_set<Tz: TimeZone>(
        value: i32,
        time_span_set: TsTzSpanSet,
    ) -> Self {
        Self::from_inner(unsafe {
            meos_sys::tintseqset_from_base_tstzspanset(value, time_span_set.inner())
        })
    }
}

impl TSequenceSet for TIntSeqSet {
    fn from_inner(inner: *const meos_sys::TSequenceSet) -> Self {
        Self { _inner: inner }
    }
}
impl TInt for TIntSeqSet {}

impl_temporal_for_tnumber!(TIntSeqSet, meos_sys::TSequenceSet, i32, Int);
impl_debug!(TIntSeqSet);
