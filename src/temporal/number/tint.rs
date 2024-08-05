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
            impl_always_and_ever_value_functions, impl_simple_types_for_temporal, Temporal,
        },
        tinstant::TInstant,
        tsequence::TSequence,
        tsequence_set::TSequenceSet,
    },
    utils::to_meos_timestamp,
};

use super::tnumber::TNumber;

macro_rules! impl_temporal {
    ($type:ty, $meos_type:ty, $base_type:ty, $generic_type_name:ident) => {
        paste::paste! {
            impl Collection for $type {
                impl_collection!(tnumber, $base_type);

                fn contains(&self, content: &Self::Type) -> bool {
                    IntSpanSet::from_inner(unsafe { meos_sys::tnumber_valuespans(self.inner()) })
                        .contains(content)
                }
            }
            impl_simple_types_for_temporal!($type, $generic_type_name);

            impl Debug for $type {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let out_str = unsafe { meos_sys::[<$generic_type_name _out>](self.inner()) };
                    let c_str = unsafe { CStr::from_ptr(out_str) };
                    let str = c_str.to_str().map_err(|_| std::fmt::Error)?;
                    let result = f.write_str(str);
                    unsafe { libc::free(out_str as *mut c_void) };
                    result
                }
            }

            impl TNumber for $type {
                fn nearest_approach_distance(&self, other: &Self) -> $base_type {
                    unsafe { meos_sys::[<nad_ $generic_type_name _ $generic_type_name>](self.inner(), other.inner()) }
                }
            }

            impl Temporal for $type {
                type TI = TIntInst;

                type TS = TIntSeq;

                type TSS = TIntSeqSet;

                type TBB = TBox;

                fn from_inner_as_temporal(inner: *const meos_sys::Temporal) -> Self {
                    Self {
                        _inner: inner as *const $meos_type,
                    }
                }

                fn from_mfjson(mfjson: &str) -> Self {
                    let cstr = CString::new(mfjson).unwrap();
                    Self::from_inner_as_temporal(unsafe { meos_sys::[<$generic_type_name _from_mfjson>](cstr.as_ptr()) })
                }

                fn inner(&self) -> *const meos_sys::Temporal {
                    self._inner as *const meos_sys::Temporal
                }

                fn bounding_box(&self) -> Self::TBB {
                    TNumber::bounding_box(self)
                }

                fn values(&self) -> Vec<Self::Type> {
                    let mut count = 0;
                    unsafe {
                        let values = meos_sys::[<$generic_type_name _values>](self.inner(), ptr::addr_of_mut!(count));

                        Vec::from_raw_parts(values, count as usize, count as usize)
                    }
                }

                fn start_value(&self) -> Self::Type {
                    unsafe { meos_sys::[<$generic_type_name _start_value>](self.inner()) }
                }

                fn end_value(&self) -> Self::Type {
                    unsafe { meos_sys::[<$generic_type_name _end_value>](self.inner()) }
                }

                fn min_value(&self) -> Self::Type {
                    unsafe { meos_sys::[<$generic_type_name _min_value>](self.inner()) }
                }

                fn max_value(&self) -> Self::Type {
                    unsafe { meos_sys::[<$generic_type_name _max_value>](self.inner()) }
                }

                fn value_at_timestamp<Tz: TimeZone>(
                    &self,
                    timestamp: DateTime<Tz>,
                ) -> Option<Self::Type> {
                    let mut result = 0;
                    unsafe {
                        let success = meos_sys::[<$generic_type_name _value_at_timestamptz>](
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
                    let result = unsafe { meos_sys::[<$generic_type_name _at_value>](self.inner(), *value) };
                    if result != ptr::null_mut() {
                        Some(Self::from_inner_as_temporal(result))
                    } else {
                        None
                    }
                }

                fn at_values(&self, values: &[Self::Type]) -> Option<Self> {
                    unsafe {
                        let set = meos_sys::intset_make(values.as_ptr(), values.len() as i32);
                        let result = meos_sys::temporal_at_values(self.inner(), set);
                        if result != ptr::null_mut() {
                            Some(Self::from_inner_as_temporal(result))
                        } else {
                            None
                        }
                    }
                }

                fn minus_value(&self, value: Self::Type) -> Self {
                    Self::from_inner_as_temporal(unsafe {
                        meos_sys::[<$generic_type_name _minus_value>](self.inner(), value)
                    })
                }

                fn minus_values(&self, values: &[Self::Type]) -> Self {
                    Self::from_inner_as_temporal(unsafe {
                        let set = meos_sys::intset_make(values.as_ptr(), values.len() as i32);
                        meos_sys::temporal_minus_values(self.inner(), set)
                    })
                }

                impl_always_and_ever_value_functions!(int);
            }
        }
    }
}

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

impl_temporal!(TIntInst, meos_sys::TInstant, i32, tint);

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

impl_temporal!(TIntSeq, meos_sys::TSequence, i32, tint);

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

impl_temporal!(TIntSeqSet, meos_sys::TSequenceSet, i32, tint);
