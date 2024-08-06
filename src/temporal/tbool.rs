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
};

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
            impl_simple_traits_for_temporal!($type, tbool);

            impl Temporal for $type {
                type TI = TBoolInst;
                type TS = TBoolSeq;
                type TSS = TBoolSeqSet;
                type TBB = TsTzSpan;


                impl_always_and_ever_value_equality_functions!(bool);
                fn from_inner_as_temporal(inner: *const meos_sys::Temporal) -> Self {
                    Self {
                        _inner: inner as *const $temporal_type,
                    }
                }

                fn from_mfjson(mfjson: &str) -> Self {
                    let cstr = CString::new(mfjson).unwrap();
                    Self::from_inner_as_temporal(unsafe { meos_sys::tbool_from_mfjson(cstr.as_ptr()) })
                }

                fn inner(&self) -> *const meos_sys::Temporal {
                    self._inner as *const meos_sys::Temporal
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

                fn at_value(&self, value: &Self::Type) -> Option<Self> {
                    let result = unsafe { meos_sys::tbool_at_value(self.inner(), *value) };
                    if !result.is_null() {
                        Some(Self::from_inner_as_temporal(result))
                    } else {
                        None
                    }
                }
                /// Not implemented for `tbool` types
                fn at_values(&self, _: &[<Self as Collection>::Type]) -> Option<Self> { unimplemented!("Not implemented for `tbool` types") }

                fn minus_value(&self, value: Self::Type) -> Self {
                    Self::from_inner_as_temporal(unsafe {
                        meos_sys::tbool_minus_value(self.inner(), value)
                    })
                }
                /// Not implemented for `tbool` types
                fn minus_values(&self, _: &[<Self as Collection>::Type]) -> Self { unimplemented!("Not implemented for `tbool` types") }
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

pub trait TBool:
    Temporal<Type = bool, TI = TBoolInst, TS = TBoolSeq, TSS = TBoolSeqSet, TBB = TsTzSpan>
    + BitAnd
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

    fn at_true(&self) -> Option<Self> {
        self.at_value(&true)
    }

    fn at_false(&self) -> Option<Self> {
        self.at_value(&false)
    }
}

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

pub struct TBoolInst {
    _inner: *const meos_sys::TInstant,
}

impl TInstant for TBoolInst {
    fn from_inner(inner: *mut meos_sys::TInstant) -> Self {
        Self { _inner: inner }
    }

    fn inner_as_tinstant(&self) -> *const meos_sys::TInstant {
        self._inner
    }

    fn from_value_and_timestamp<Tz: TimeZone>(value: Self::Type, timestamp: DateTime<Tz>) -> Self {
        Self::from_inner(unsafe { meos_sys::tboolinst_make(value, to_meos_timestamp(&timestamp)) })
    }
}

impl TBool for TBoolInst {}

impl_tbool_traits!(TBoolInst, meos_sys::TInstant);
impl_debug!(TBoolInst);

pub struct TBoolSeq {
    _inner: *const meos_sys::TSequence,
}
impl TBoolSeq {
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

impl TSequence for TBoolSeq {
    fn from_inner(inner: *const meos_sys::TSequence) -> Self {
        Self { _inner: inner }
    }

    fn inner_as_tsequence(&self) -> *const meos_sys::TSequence {
        self._inner
    }
}

impl_tbool_traits!(TBoolSeq, meos_sys::TSequence);
impl_debug!(TBoolSeq);

impl TBool for TBoolSeq {}

pub struct TBoolSeqSet {
    _inner: *const meos_sys::TSequenceSet,
}

impl TBoolSeqSet {
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

impl TSequenceSet for TBoolSeqSet {
    fn from_inner(inner: *const meos_sys::TSequenceSet) -> Self {
        Self { _inner: inner }
    }
}
impl TBool for TBoolSeqSet {}

impl_tbool_traits!(TBoolSeqSet, meos_sys::TSequenceSet);
impl_debug!(TBoolSeqSet);
