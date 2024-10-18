use std::{
    ffi::{c_void, CStr, CString},
    fmt::Debug,
    hash::Hash,
    ptr,
    str::FromStr,
};

use chrono::{DateTime, TimeZone};

use crate::{
    boxes::TBox,
    collections::{
        base::*,
        datetime::{TsTzSpan, TsTzSpanSet},
        number::IntSpanSet,
    },
    errors::ParseError,
    factory, impl_from_str,
    temporal::{
        interpolation::TInterpolation,
        tbool::{TBool, TBoolInstant, TBoolSequence, TBoolSequenceSet},
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

use super::tnumber::{impl_meos_enum, impl_temporal_for_tnumber, TNumber};

#[derive(Debug)]
pub enum TInt {
    Instant(TIntInstant),
    Sequence(TIntSequence),
    SequenceSet(TIntSequenceSet),
}

impl_from_str!(TInt);

impl MeosEnum for TInt {
    fn from_instant(inner: *mut meos_sys::TInstant) -> Self {
        Self::Instant(TIntInstant::from_inner(inner))
    }

    fn from_sequence(inner: *mut meos_sys::TSequence) -> Self {
        Self::Sequence(TIntSequence::from_inner(inner))
    }

    fn from_sequence_set(inner: *mut meos_sys::TSequenceSet) -> Self {
        Self::SequenceSet(TIntSequenceSet::from_inner(inner))
    }

    fn from_mfjson(mfjson: &str) -> Self {
        let cstr = CString::new(mfjson).unwrap();
        factory::<Self>(unsafe { meos_sys::tint_from_mfjson(cstr.as_ptr()) })
    }
}

pub trait TIntTrait:
    TNumber<
    Type = i32,
    TI = TIntInstant,
    TS = TIntSequence,
    TSS = TIntSequenceSet,
    TBB = TBox,
    Enum = TInt,
>
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

pub struct TIntInstant {
    _inner: ptr::NonNull<meos_sys::TInstant>,
}

impl TInstant for TIntInstant {
    fn from_inner(inner: *mut meos_sys::TInstant) -> Self {
        Self {
            _inner: ptr::NonNull::new(inner).expect("Null pointers not allowed"),
        }
    }

    fn inner_as_tinstant(&self) -> *const meos_sys::TInstant {
        self._inner.as_ptr()
    }

    fn from_value_and_timestamp<Tz: TimeZone>(value: Self::Type, timestamp: DateTime<Tz>) -> Self {
        Self::from_inner(unsafe { meos_sys::tintinst_make(value, to_meos_timestamp(&timestamp)) })
    }
}

impl<Tz: TimeZone> From<(i32, DateTime<Tz>)> for TIntInstant {
    fn from((v, t): (i32, DateTime<Tz>)) -> Self {
        Self::from_value_and_timestamp(v, t)
    }
}

impl TIntTrait for TIntInstant {}

impl_temporal_for_tnumber!(TIntInstant, Instant, i32, Int);
impl_debug!(TIntInstant);

pub struct TIntSequence {
    _inner: ptr::NonNull<meos_sys::TSequence>,
}
impl TIntSequence {
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

impl TSequence for TIntSequence {
    fn from_inner(inner: *mut meos_sys::TSequence) -> Self {
        Self {
            _inner: ptr::NonNull::new(inner).expect("Null pointers not allowed"),
        }
    }

    fn inner_mut_as_tsequence(&self) -> *mut meos_sys::TSequence {
        self._inner.as_ptr()
    }
}

impl FromIterator<TIntInstant> for TIntSequence {
    fn from_iter<T: IntoIterator<Item = TIntInstant>>(iter: T) -> Self {
        let vec: Vec<TIntInstant> = iter.into_iter().collect();
        Self::new(&vec, TInterpolation::Stepwise)
    }
}

impl<'a> FromIterator<&'a TIntInstant> for TIntSequence {
    fn from_iter<T: IntoIterator<Item = &'a TIntInstant>>(iter: T) -> Self {
        let vec: Vec<&TIntInstant> = iter.into_iter().collect();
        Self::new(&vec, TInterpolation::Stepwise)
    }
}

impl<Tz: TimeZone> FromIterator<(i32, DateTime<Tz>)> for TIntSequence {
    fn from_iter<T: IntoIterator<Item = (i32, DateTime<Tz>)>>(iter: T) -> Self {
        iter.into_iter().map(Into::<TIntInstant>::into).collect()
    }
}

impl TIntTrait for TIntSequence {}

impl_temporal_for_tnumber!(TIntSequence, Sequence, i32, Int);
impl_debug!(TIntSequence);

pub struct TIntSequenceSet {
    _inner: ptr::NonNull<meos_sys::TSequenceSet>,
}

impl TIntSequenceSet {
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

impl TSequenceSet for TIntSequenceSet {
    fn from_inner(inner: *mut meos_sys::TSequenceSet) -> Self {
        Self {
            _inner: ptr::NonNull::new(inner).expect("Null pointers not allowed"),
        }
    }
}

impl FromIterator<TIntSequence> for TIntSequenceSet {
    fn from_iter<T: IntoIterator<Item = TIntSequence>>(iter: T) -> Self {
        let vec: Vec<TIntSequence> = iter.into_iter().collect();
        let mut vec_ptr: Vec<_> = vec.iter().map(|t| t.inner_as_tsequence()).collect();

        let result = unsafe {
            meos_sys::tsequenceset_make(vec_ptr.as_mut_ptr(), vec_ptr.len() as i32, true)
        };
        TIntSequenceSet::from_inner(result)
    }
}

impl FromIterator<TInt> for TInt {
    fn from_iter<T: IntoIterator<Item = TInt>>(iter: T) -> Self {
        let mut iter = iter.into_iter();
        let first: TInt = iter.next().unwrap();
        let init_value = if let TInt::Instant(value) = first {
            TInt::from_sequence(
                value
                    .to_sequence(TInterpolation::Stepwise)
                    .inner_mut_as_tsequence(),
            )
        } else {
            first
        };

        iter.fold(init_value, |acc, item| match (acc, item) {
            (TInt::Sequence(acc_value), TInt::Sequence(item_value)) => {
                acc_value.append_sequence(item_value)
            }
            (TInt::Sequence(acc_value), TInt::Instant(item_value)) => {
                acc_value.append_instant(item_value, None, None)
            }
            (TInt::SequenceSet(acc_value), TInt::Instant(item_value)) => {
                acc_value.append_instant(item_value, None, None)
            }
            (TInt::SequenceSet(acc_value), TInt::Sequence(item_value)) => {
                acc_value.append_sequence(item_value)
            }
            (_, TInt::SequenceSet(_)) => unreachable!(),
            (TInt::Instant(_), _) => unreachable!(),
        })
    }
}

impl TIntTrait for TIntSequenceSet {}

impl_temporal_for_tnumber!(TIntSequenceSet, SequenceSet, i32, Int);
impl_debug!(TIntSequenceSet);

impl From<TIntInstant> for TInt {
    fn from(value: TIntInstant) -> Self {
        TInt::Instant(value)
    }
}

impl From<TIntSequence> for TInt {
    fn from(value: TIntSequence) -> Self {
        TInt::Sequence(value)
    }
}

impl From<TIntSequenceSet> for TInt {
    fn from(value: TIntSequenceSet) -> Self {
        TInt::SequenceSet(value)
    }
}

impl TryFrom<TInt> for TIntInstant {
    type Error = ParseError;
    fn try_from(value: TInt) -> Result<Self, Self::Error> {
        if let TInt::Instant(new_value) = value {
            Ok(new_value)
        } else {
            Err(ParseError)
        }
    }
}

impl TryFrom<TInt> for TIntSequence {
    type Error = ParseError;
    fn try_from(value: TInt) -> Result<Self, Self::Error> {
        if let TInt::Sequence(new_value) = value {
            Ok(new_value)
        } else {
            Err(ParseError)
        }
    }
}

impl TryFrom<TInt> for TIntSequenceSet {
    type Error = ParseError;
    fn try_from(value: TInt) -> Result<Self, Self::Error> {
        if let TInt::SequenceSet(new_value) = value {
            Ok(new_value)
        } else {
            Err(ParseError)
        }
    }
}

impl_meos_enum!(TInt, i32, Int);
