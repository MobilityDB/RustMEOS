use std::{
    ffi::{c_void, CStr, CString},
    fmt::Debug,
    hash::Hash,
    ops::{BitAnd, BitOr},
    ptr,
    str::FromStr,
};

use chrono::{DateTime, TimeZone};

use crate::{
    collections::{
        base::{
            collection::{impl_collection, Collection},
            span::Span,
            span_set::SpanSet,
        },
        datetime::{tstz_span::TsTzSpan, tstz_span_set::TsTzSpanSet},
    },
    errors::ParseError,
    factory, impl_from_str,
    temporal::{
        temporal::{
            impl_always_and_ever_value_equality_functions, impl_simple_traits_for_temporal,
            Temporal,
        },
        tinstant::TInstant,
        tsequence::TSequence,
        tsequence_set::TSequenceSet,
    },
    utils::to_meos_timestamp,
    MeosEnum,
};

use super::interpolation::TInterpolation;

macro_rules! impl_debug {
    ($type:ty) => {
        impl Debug for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let out_str = unsafe { meos_sys::tbool_out(self.inner()) };
                let c_str = unsafe { CStr::from_ptr(out_str) };
                let str = c_str.to_str().map_err(|_| std::fmt::Error)?;
                let result = f.write_str(str);
                unsafe { libc::free(out_str as *mut c_void) };
                result
            }
        }
    };
}

macro_rules! impl_tbool_traits {
    ($type:ty, $temporal_type:ty) => {
        paste::paste! {
            impl Collection for $type {
                impl_collection!(tnumber, bool);

                fn contains(&self, content: &Self::Type) -> bool {
                    let result = unsafe { meos_sys::ever_eq_tbool_bool(self.inner(), *content) };
                    result == 1
                }
            }
            impl_simple_traits_for_temporal!($type, with_drop);
            impl_debug!($type);

            impl Temporal for $type {
                type TI = TBoolInstant;
                type TS = TBoolSequence;
                type TSS = TBoolSequenceSet;
                type TBB = TsTzSpan;
                type Enum = TBool;
                type TBoolType = $type;

                impl_always_and_ever_value_equality_functions!(bool);
                fn from_inner_as_temporal(inner: *mut meos_sys::Temporal) -> Self {
                    Self {
                        _inner: ptr::NonNull::new(inner as *mut $temporal_type).expect("Null pointers not allowed"),
                    }
                }

                fn inner(&self) -> *const meos_sys::Temporal {
                    self._inner.as_ptr() as *const meos_sys::Temporal
                }

                fn bounding_box(&self) -> Self::TBB {
                    self.timespan()
                }

                fn values(&self) -> Vec<Self::Type> {
                    let mut count = 0;
                    unsafe {
                        let values = meos_sys::tbool_values(self.inner(), ptr::addr_of_mut!(count));

                        Vec::from_raw_parts(values, count as usize, count as usize)
                    }
                }

                fn start_value(&self) -> Self::Type {
                    unsafe { meos_sys::tbool_start_value(self.inner()) }
                }

                fn end_value(&self) -> Self::Type {
                    unsafe { meos_sys::tbool_end_value(self.inner()) }
                }

                fn value_at_timestamp<Tz: TimeZone>(
                    &self,
                    timestamp: DateTime<Tz>,
                ) -> Option<Self::Type> {
                    let mut result = false;
                    unsafe {
                        let success = meos_sys::tbool_value_at_timestamptz(
                            self.inner(),
                            to_meos_timestamp(&timestamp),
                            true,
                            ptr::addr_of_mut!(result),
                        );
                        if success {
                            Some(result)
                        } else {
                            None
                        }
                    }
                }

                fn at_value(&self, value: &Self::Type) -> Option<Self::Enum> {
                    let result = unsafe { meos_sys::tbool_at_value(self.inner(), *value) };
                    if !result.is_null() {
                        Some(factory::<Self::Enum>(result))
                    } else {
                        None
                    }
                }
                /// Not implemented for `tbool` types
                fn at_values(&self, _: &[<Self as Collection>::Type]) -> Option<Self::Enum> { unimplemented!("Not implemented for `tbool` types") }

                fn minus_value(&self, value: Self::Type) -> Self::Enum {
                    factory::<Self::Enum>(unsafe {
                        meos_sys::tbool_minus_value(self.inner(), value)
                    })
                }
                /// Not implemented for `tbool` types
                fn minus_values(&self, _: &[<Self as Collection>::Type]) -> Self::Enum { unimplemented!("Not implemented for `tbool` types") }

                fn temporal_equal_value(&self, value: &Self::Type) -> Self {
                    Self::from_inner_as_temporal(unsafe {
                        meos_sys::teq_tbool_bool(self.inner(), *value)
                    })
                }

                fn temporal_not_equal_value(&self, value: &Self::Type) -> Self {
                    Self::from_inner_as_temporal(unsafe {
                        meos_sys::tne_tbool_bool(self.inner(), *value)
                    })
                }
            }
            impl BitAnd for $type {
                type Output = Self;

                fn bitand(self, rhs: Self) -> Self::Output {
                    self.temporal_and(&rhs)
                }
            }

            impl BitAnd<bool> for $type {
                type Output = Self;

                fn bitand(self, rhs: bool) -> Self::Output {
                    Self::from_inner_as_temporal(unsafe { meos_sys::tand_tbool_bool(self.inner(), rhs) })
                }
            }

            impl BitOr for $type {
                type Output = Self;

                fn bitor(self, rhs: Self) -> Self::Output {
                    self.temporal_or(&rhs)
                }
            }

            impl BitOr<bool> for $type {
                type Output = Self;

                fn bitor(self, rhs: bool) -> Self::Output {
                    Self::from_inner_as_temporal(unsafe { meos_sys::tor_tbool_bool(self.inner(), rhs) })
                }
            }

        }
    }
}

#[derive(Debug)]
pub enum TBool {
    Instant(TBoolInstant),
    Sequence(TBoolSequence),
    SequenceSet(TBoolSequenceSet),
}

impl_from_str!(TBool);

impl MeosEnum for TBool {
    fn from_instant(inner: *mut meos_sys::TInstant) -> Self {
        Self::Instant(TBoolInstant::from_inner(inner))
    }

    fn from_sequence(inner: *mut meos_sys::TSequence) -> Self {
        Self::Sequence(TBoolSequence::from_inner(inner))
    }

    fn from_sequence_set(inner: *mut meos_sys::TSequenceSet) -> Self {
        Self::SequenceSet(TBoolSequenceSet::from_inner(inner))
    }

    fn from_mfjson(mfjson: &str) -> Self {
        let cstr = CString::new(mfjson).unwrap();
        factory::<Self>(unsafe { meos_sys::tbool_from_mfjson(cstr.as_ptr()) })
    }
}

pub trait TBoolTrait:
    Temporal<
        Type = bool,
        TI = TBoolInstant,
        TS = TBoolSequence,
        TSS = TBoolSequenceSet,
        TBB = TsTzSpan,
    > + BitAnd
    + BitAnd<bool>
    + BitOr
    + BitOr<bool>
{
    fn temporal_or(&self, other: &Self) -> Self {
        Self::from_inner_as_temporal(unsafe {
            meos_sys::tor_tbool_tbool(self.inner(), other.inner())
        })
    }
    fn temporal_and(&self, other: &Self) -> Self {
        Self::from_inner_as_temporal(unsafe {
            meos_sys::tand_tbool_tbool(self.inner(), other.inner())
        })
    }
    fn temporal_not(&self) -> Self {
        Self::from_inner_as_temporal(unsafe { meos_sys::tnot_tbool(self.inner()) })
    }

    fn at_true(&self) -> Option<Self::Enum> {
        self.at_value(&true)
    }

    fn at_false(&self) -> Option<Self::Enum> {
        self.at_value(&false)
    }
}

pub struct TBoolInstant {
    _inner: ptr::NonNull<meos_sys::TInstant>,
}

impl TInstant for TBoolInstant {
    fn from_inner(inner: *mut meos_sys::TInstant) -> Self {
        Self {
            _inner: ptr::NonNull::new(inner).expect("Null pointers not allowed"),
        }
    }

    fn inner_as_tinstant(&self) -> *const meos_sys::TInstant {
        self._inner.as_ptr()
    }

    fn from_value_and_timestamp<Tz: TimeZone>(value: Self::Type, timestamp: DateTime<Tz>) -> Self {
        Self::from_inner(unsafe { meos_sys::tboolinst_make(value, to_meos_timestamp(&timestamp)) })
    }
}

impl TBoolTrait for TBoolInstant {}

impl_tbool_traits!(TBoolInstant, meos_sys::TInstant);

pub struct TBoolSequence {
    _inner: ptr::NonNull<meos_sys::TSequence>,
}
impl TBoolSequence {
    /// Creates a temporal object from a value and a TsTz span.
    ///
    /// ## Arguments
    /// * `value` - Base value.
    /// * `time_span` - Time object to use as the temporal dimension.
    ///
    /// ## Returns
    /// A new temporal object.
    pub fn from_value_and_tstz_span<Tz: TimeZone>(value: bool, time_span: TsTzSpan) -> Self {
        Self::from_inner(unsafe { meos_sys::tboolseq_from_base_tstzspan(value, time_span.inner()) })
    }
}

impl TSequence for TBoolSequence {
    fn from_inner(inner: *mut meos_sys::TSequence) -> Self {
        Self {
            _inner: ptr::NonNull::new(inner).expect("Null pointers not allowed"),
        }
    }

    fn inner_mut_as_tsequence(&self) -> *mut meos_sys::TSequence {
        self._inner.as_ptr()
    }
}

impl_tbool_traits!(TBoolSequence, meos_sys::TSequence);

impl TBoolTrait for TBoolSequence {}

impl FromIterator<TBoolInstant> for TBoolSequence {
    fn from_iter<T: IntoIterator<Item = TBoolInstant>>(iter: T) -> Self {
        let vec: Vec<TBoolInstant> = iter.into_iter().collect();
        Self::new(&vec, TInterpolation::Discrete)
    }
}

impl<'a> FromIterator<&'a TBoolInstant> for TBoolSequence {
    fn from_iter<T: IntoIterator<Item = &'a TBoolInstant>>(iter: T) -> Self {
        let vec: Vec<&TBoolInstant> = iter.into_iter().collect();
        Self::new(&vec, TInterpolation::Discrete)
    }
}

pub struct TBoolSequenceSet {
    _inner: ptr::NonNull<meos_sys::TSequenceSet>,
}

impl TBoolSequenceSet {
    /// Creates a temporal object from a base value and a TsTz span set.
    ///
    /// ## Arguments
    /// * `value` - Base value.
    /// * `time_span_set` - Time object to use as the temporal dimension.
    ///
    /// ## Returns
    /// A new temporal object.
    pub fn from_value_and_tstz_span_set<Tz: TimeZone>(
        value: bool,
        time_span_set: TsTzSpanSet,
    ) -> Self {
        Self::from_inner(unsafe {
            meos_sys::tboolseqset_from_base_tstzspanset(value, time_span_set.inner())
        })
    }
}

impl TSequenceSet for TBoolSequenceSet {
    fn from_inner(inner: *mut meos_sys::TSequenceSet) -> Self {
        Self {
            _inner: ptr::NonNull::new(inner).expect("Null pointers not allowed"),
        }
    }
}
impl TBoolTrait for TBoolSequenceSet {}

impl_tbool_traits!(TBoolSequenceSet, meos_sys::TSequenceSet);

impl From<TBoolInstant> for TBool {
    fn from(value: TBoolInstant) -> Self {
        TBool::Instant(value)
    }
}

impl From<TBoolSequence> for TBool {
    fn from(value: TBoolSequence) -> Self {
        TBool::Sequence(value)
    }
}

impl From<TBoolSequenceSet> for TBool {
    fn from(value: TBoolSequenceSet) -> Self {
        TBool::SequenceSet(value)
    }
}

impl TryFrom<TBool> for TBoolInstant {
    type Error = ParseError;
    fn try_from(value: TBool) -> Result<Self, Self::Error> {
        if let TBool::Instant(new_value) = value {
            Ok(new_value)
        } else {
            Err(ParseError)
        }
    }
}

impl TryFrom<TBool> for TBoolSequence {
    type Error = ParseError;
    fn try_from(value: TBool) -> Result<Self, Self::Error> {
        if let TBool::Sequence(new_value) = value {
            Ok(new_value)
        } else {
            Err(ParseError)
        }
    }
}

impl TryFrom<TBool> for TBoolSequenceSet {
    type Error = ParseError;
    fn try_from(value: TBool) -> Result<Self, Self::Error> {
        if let TBool::SequenceSet(new_value) = value {
            Ok(new_value)
        } else {
            Err(ParseError)
        }
    }
}

impl Collection for TBool {
    impl_collection!(tnumber, bool);

    fn contains(&self, content: &Self::Type) -> bool {
        let result = unsafe { meos_sys::ever_eq_tbool_bool(self.inner(), *content) };
        result == 1
    }
}
impl_simple_traits_for_temporal!(TBool);

impl Temporal for TBool {
    type TI = TBoolInstant;
    type TS = TBoolSequence;
    type TSS = TBoolSequenceSet;
    type TBB = TsTzSpan;
    type Enum = TBool;
    type TBoolType = TBool;

    impl_always_and_ever_value_equality_functions!(bool);
    fn from_inner_as_temporal(inner: *mut meos_sys::Temporal) -> Self {
        factory::<Self>(inner)
    }

    fn inner(&self) -> *const meos_sys::Temporal {
        match self {
            TBool::Instant(value) => value.inner(),
            TBool::Sequence(value) => value.inner(),
            TBool::SequenceSet(value) => value.inner(),
        }
    }

    fn bounding_box(&self) -> Self::TBB {
        self.timespan()
    }

    fn values(&self) -> Vec<Self::Type> {
        let mut count = 0;
        unsafe {
            let values = meos_sys::tbool_values(self.inner(), ptr::addr_of_mut!(count));

            Vec::from_raw_parts(values, count as usize, count as usize)
        }
    }

    fn start_value(&self) -> Self::Type {
        unsafe { meos_sys::tbool_start_value(self.inner()) }
    }

    fn end_value(&self) -> Self::Type {
        unsafe { meos_sys::tbool_end_value(self.inner()) }
    }

    fn value_at_timestamp<Tz: TimeZone>(&self, timestamp: DateTime<Tz>) -> Option<Self::Type> {
        let mut result = false;
        unsafe {
            let success = meos_sys::tbool_value_at_timestamptz(
                self.inner(),
                to_meos_timestamp(&timestamp),
                true,
                ptr::addr_of_mut!(result),
            );
            if success {
                Some(result)
            } else {
                None
            }
        }
    }

    fn at_value(&self, value: &Self::Type) -> Option<Self::Enum> {
        let result = unsafe { meos_sys::tbool_at_value(self.inner(), *value) };
        if !result.is_null() {
            Some(factory::<Self::Enum>(result))
        } else {
            None
        }
    }
    /// Not implemented for `tbool` types
    fn at_values(&self, _: &[Self::Type]) -> Option<Self::Enum> {
        unimplemented!("Not implemented for `tbool` types")
    }

    fn minus_value(&self, value: Self::Type) -> Self::Enum {
        factory::<Self::Enum>(unsafe { meos_sys::tbool_minus_value(self.inner(), value) })
    }
    /// Not implemented for `tbool` types
    fn minus_values(&self, _: &[Self::Type]) -> Self::Enum {
        unimplemented!("Not implemented for `tbool` types")
    }

    fn temporal_equal_value(&self, value: &Self::Type) -> Self {
        Self::from_inner_as_temporal(unsafe { meos_sys::teq_tbool_bool(self.inner(), *value) })
    }

    fn temporal_not_equal_value(&self, value: &Self::Type) -> Self {
        Self::from_inner_as_temporal(unsafe { meos_sys::tne_tbool_bool(self.inner(), *value) })
    }
}
impl BitAnd for TBool {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.temporal_and(&rhs)
    }
}

impl BitAnd<bool> for TBool {
    type Output = Self;

    fn bitand(self, rhs: bool) -> Self::Output {
        Self::from_inner_as_temporal(unsafe { meos_sys::tand_tbool_bool(self.inner(), rhs) })
    }
}

impl BitOr for TBool {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.temporal_or(&rhs)
    }
}

impl BitOr<bool> for TBool {
    type Output = Self;

    fn bitor(self, rhs: bool) -> Self::Output {
        Self::from_inner_as_temporal(unsafe { meos_sys::tor_tbool_bool(self.inner(), rhs) })
    }
}

impl TBoolTrait for TBool {}
