use std::{
    ffi::{c_void, CStr, CString},
    fmt::Debug,
    hash::Hash,
    ptr,
    str::FromStr,
};

use chrono::{DateTime, TimeZone};

use super::tnumber::{impl_temporal_for_tnumber, TNumber};
use crate::{
    boxes::tbox::TBox,
    collections::{
        base::{
            collection::{impl_collection, Collection},
            span::Span,
            span_set::SpanSet,
        },
        datetime::{tstz_span::TsTzSpan, tstz_span_set::TsTzSpanSet},
        number::float_span_set::FloatSpanSet,
    },
    errors::ParseError,
    factory,
    temporal::{
        interpolation::TInterpolation,
        tbool::{TBoolInstant, TBoolSequence, TBoolSequenceSet},
        temporal::{
            impl_always_and_ever_value_equality_functions, impl_ordered_temporal_functions,
            impl_simple_traits_for_temporal, OrderedTemporal, Temporal,
        },
        tinstant::TInstant,
        tsequence::TSequence,
        tsequence_set::TSequenceSet,
    },
    utils::to_meos_timestamp,
    MeosEnum,
};

macro_rules! impl_debug {
    ($type:ty) => {
        impl Debug for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let out_str = unsafe { meos_sys::tfloat_out(self.inner(), 3) };
                let c_str = unsafe { CStr::from_ptr(out_str) };
                let str = c_str.to_str().map_err(|_| std::fmt::Error)?;
                let result = f.write_str(str);
                unsafe { libc::free(out_str as *mut c_void) };
                result
            }
        }
    };
}

#[derive(Debug)]
pub enum TFloat {
    Instant(TFloatInstant),
    Sequence(TFloatSequence),
    SequenceSet(TFloatSequenceSet),
}

impl MeosEnum for TFloat {
    fn from_instant(inner: *mut meos_sys::TInstant) -> Self {
        Self::Instant(TFloatInstant {
            _inner: ptr::NonNull::new(inner).expect("Null pointers not allowed"),
        })
    }

    fn from_sequence(inner: *mut meos_sys::TSequence) -> Self {
        Self::Sequence(TFloatSequence {
            _inner: ptr::NonNull::new(inner).expect("Null pointers not allowed"),
        })
    }

    fn from_sequence_set(inner: *mut meos_sys::TSequenceSet) -> Self {
        Self::SequenceSet(TFloatSequenceSet {
            _inner: ptr::NonNull::new(inner).expect("Null pointers not allowed"),
        })
    }

    fn from_mfjson(mfjson: &str) -> Self {
        let cstr = CString::new(mfjson).unwrap();
        factory::<Self>(unsafe { meos_sys::tfloat_from_mfjson(cstr.as_ptr()) })
    }

    fn inner(&self) -> *const meos_sys::Temporal {
        match self {
            TFloat::Instant(value) => value.inner(),
            TFloat::Sequence(value) => value.inner(),
            TFloat::SequenceSet(value) => value.inner(),
        }
    }
}

pub trait TFloatTrait:
    TNumber<Type = f64, TI = TFloatInstant, TS = TFloatSequence, TSS = TFloatSequenceSet, TBB = TBox>
{
    // ------------------------- Transformations -------------------------------

    /// Returns a new `TNumber` with the value dimension shifted by `shift` and scaled so the value dimension has width `width`.
    ///
    /// # Arguments
    /// * `shift` - Value to shift
    /// * `width` - Value representing the width of the new temporal number
    ///
    /// # Safety
    /// This function uses unsafe code to call the `meos_sys::tfloat_shift_scale_value` or
    /// `meos_sys::tfloat_shift_scale_value` functions.
    fn shift_scale_value(&self, shift: Option<Self::Type>, width: Option<Self::Type>) -> Self {
        let d = shift.unwrap_or_default();
        let w = width.unwrap_or_default();
        let modified = unsafe { meos_sys::tfloat_shift_scale_value(self.inner(), d, w) };
        Self::from_inner_as_temporal(modified)
    }
}

pub struct TFloatInstant {
    _inner: ptr::NonNull<meos_sys::TInstant>,
}

impl TInstant for TFloatInstant {
    fn from_inner(inner: *mut meos_sys::TInstant) -> Self {
        Self {
            _inner: ptr::NonNull::new(inner).expect("Null pointers not allowed"),
        }
    }

    fn inner_as_tinstant(&self) -> *const meos_sys::TInstant {
        self._inner.as_ptr()
    }

    fn from_value_and_timestamp<Tz: TimeZone>(value: Self::Type, timestamp: DateTime<Tz>) -> Self {
        Self::from_inner(unsafe { meos_sys::tfloatinst_make(value, to_meos_timestamp(&timestamp)) })
    }
}

impl TFloatTrait for TFloatInstant {}

impl_temporal_for_tnumber!(TFloatInstant, Instant, f64, Float);
impl_debug!(TFloatInstant);

impl<Tz: TimeZone> From<(f64, DateTime<Tz>)> for TFloatInstant {
    fn from((v, t): (f64, DateTime<Tz>)) -> Self {
        Self::from_value_and_timestamp(v, t)
    }
}

pub struct TFloatSequence {
    _inner: ptr::NonNull<meos_sys::TSequence>,
}

impl TFloatSequence {
    /// Creates a temporal object from a value and a TsTz span.
    ///
    /// ## Arguments
    /// * `value` - Base value.
    /// * `time_span` - Time object to use as the temporal dimension.
    ///
    /// ## Returns
    /// A new temporal object.
    pub fn from_value_and_tstz_span<Tz: TimeZone>(
        value: f64,
        time_span: TsTzSpan,
        interpolation: TInterpolation,
    ) -> Self {
        Self::from_inner(unsafe {
            meos_sys::tfloatseq_from_base_tstzspan(value, time_span.inner(), interpolation as u32)
        })
    }
}

impl TSequence for TFloatSequence {
    fn from_inner(inner: *mut meos_sys::TSequence) -> Self {
        Self {
            _inner: ptr::NonNull::new(inner).expect("Null pointers not allowed"),
        }
    }

    fn inner_mut_as_tsequence(&self) -> *mut meos_sys::TSequence {
        self._inner.as_ptr()
    }
}

impl TFloatTrait for TFloatSequence {}

impl_temporal_for_tnumber!(TFloatSequence, Sequence, f64, Float);
impl_debug!(TFloatSequence);

pub struct TFloatSequenceSet {
    _inner: ptr::NonNull<meos_sys::TSequenceSet>,
}

impl TFloatSequenceSet {
    /// Creates a temporal object from a base value and a TsTz span set.
    ///
    /// ## Arguments
    /// * `value` - Base value.
    /// * `time_span_set` - Time object to use as the temporal dimension.
    ///
    /// ## Returns
    /// A new temporal object.
    pub fn from_value_and_tstz_span_set<Tz: TimeZone>(
        value: f64,
        time_span_set: TsTzSpanSet,
        interpolation: TInterpolation,
    ) -> Self {
        Self::from_inner(unsafe {
            meos_sys::tfloatseqset_from_base_tstzspanset(
                value,
                time_span_set.inner(),
                interpolation as u32,
            )
        })
    }
}

impl TSequenceSet for TFloatSequenceSet {
    fn from_inner(inner: *mut meos_sys::TSequenceSet) -> Self {
        Self {
            _inner: ptr::NonNull::new(inner).expect("Null pointers not allowed"),
        }
    }
}
impl TFloatTrait for TFloatSequenceSet {}

impl_temporal_for_tnumber!(TFloatSequenceSet, SequenceSet, f64, Float);
impl_debug!(TFloatSequenceSet);

impl From<TFloatInstant> for TFloat {
    fn from(value: TFloatInstant) -> Self {
        TFloat::Instant(value)
    }
}

impl From<TFloatSequence> for TFloat {
    fn from(value: TFloatSequence) -> Self {
        TFloat::Sequence(value)
    }
}

impl From<TFloatSequenceSet> for TFloat {
    fn from(value: TFloatSequenceSet) -> Self {
        TFloat::SequenceSet(value)
    }
}

impl TryFrom<TFloat> for TFloatInstant {
    type Error = ParseError;
    fn try_from(value: TFloat) -> Result<Self, Self::Error> {
        if let TFloat::Instant(new_value) = value {
            Ok(new_value)
        } else {
            Err(ParseError)
        }
    }
}

impl TryFrom<TFloat> for TFloatSequence {
    type Error = ParseError;
    fn try_from(value: TFloat) -> Result<Self, Self::Error> {
        if let TFloat::Sequence(new_value) = value {
            Ok(new_value)
        } else {
            Err(ParseError)
        }
    }
}

impl TryFrom<TFloat> for TFloatSequenceSet {
    type Error = ParseError;
    fn try_from(value: TFloat) -> Result<Self, Self::Error> {
        if let TFloat::SequenceSet(new_value) = value {
            Ok(new_value)
        } else {
            Err(ParseError)
        }
    }
}
