use std::{
    ffi::{c_void, CStr, CString},
    fmt::Debug,
    hash::Hash,
    str::FromStr,
};

use chrono::{DateTime, TimeZone};

use crate::{
    collections::{
        base::collection::{impl_collection, Collection},
        datetime::tstz_span::TsTzSpan,
    },
    errors::ParseError,
    temporal::{temporal::Temporal, tinstant::TInstant, tsequence::TSequence},
};

pub trait TInt: Temporal {
    // fn from_value_and_tstzspan(value: Self::Type, time_span: TsTzSpan) -> Self
    // where
    //     Self: TSequence,
    // {
    //     TSequence::new(value, datetime)
    // }
}

pub struct TIntInst {
    _inner: *const meos_sys::TInstant,
}

pub struct TIntSeq {
    _inner: *const meos_sys::TSequence,
}

pub struct TIntSeqSet {
    _inner: *const meos_sys::TSequenceSet,
}

impl Collection for TIntInst {
    impl_collection!(tnumber, i32);

    fn contains(&self, content: &Self::Type) -> bool {
        meos_sys::contains
    }
}

impl Clone for TIntInst {
    fn clone(&self) -> Self {
        Temporal::from_inner(unsafe { meos_sys::temporal_copy(Temporal::inner(self)) })
    }
}

impl FromStr for TIntInst {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        CString::new(s).map_err(|_| ParseError).map(|string| {
            let inner = unsafe { meos_sys::tint_in(string.as_ptr()) };
            Temporal::from_inner(inner)
        })
    }
}

impl Debug for TIntInst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out_str = unsafe { meos_sys::tint_out(Temporal::inner(self)) };
        let c_str = unsafe { CStr::from_ptr(out_str) };
        let str = c_str.to_str().map_err(|_| std::fmt::Error)?;
        let result = f.write_str(str);
        unsafe { libc::free(out_str as *mut c_void) };
        result
    }
}

impl PartialEq for TIntInst {
    fn eq(&self, other: &Self) -> bool {
        unsafe { meos_sys::temporal_eq(Temporal::inner(self), Temporal::inner(other)) }
    }
}

impl Temporal for TIntInst {
    type TI = TIntInst;

    type TS = TIntSeq;

    type TSS = TIntSeqSet;

    fn from_inner(inner: *const meos_sys::Temporal) -> Self {
        Self {
            _inner: inner as *const meos_sys::TInstant,
        }
    }

    fn from_base_time<Tz: TimeZone>(value: Self::Type, base: DateTime<Tz>) -> Self {
        todo!()
    }

    fn from_base_tstz_span<Tz: TimeZone>(value: Self::Type, base: TsTzSpan) -> Self {
        todo!()
    }

    fn from_base_tstz_span_set<Tz: TimeZone>(
        value: Self::Type,
        base: crate::collections::datetime::tstz_span_set::TsTzSpanSet,
    ) -> Self {
        todo!()
    }

    fn from_mfjson(mfjson: &str) -> Self {
        todo!()
    }

    fn inner(&self) -> *const meos_sys::Temporal {
        self._inner as *const meos_sys::Temporal
    }

    fn bounding_box(&self) -> impl crate::BoundingBox {
        todo!()
    }

    fn value_set(&self) -> std::collections::HashSet<Self::Type> {
        todo!()
    }

    fn values(&self) -> Vec<Self::Type> {
        todo!()
    }

    fn start_value(&self) -> Self::Type {
        todo!()
    }

    fn end_value(&self) -> Self::Type {
        todo!()
    }

    fn min_value(&self) -> Self::Type {
        todo!()
    }

    fn max_value(&self) -> Self::Type {
        todo!()
    }

    fn value_at_timestamp<Tz: TimeZone>(&self, timestamp: DateTime<Tz>) -> Self::Type {
        todo!()
    }
}

impl TInstant for TIntInst {
    fn new<Tz: TimeZone>(value: Self::Type, timestamp: DateTime<Tz>) -> Self {
        todo!()
    }

    fn from_inner(inner: *mut meos_sys::TInstant) -> Self {
        todo!()
    }

    fn inner(&self) -> *const meos_sys::TInstant {
        todo!()
    }
}

impl Hash for TIntInst {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let hash = unsafe { meos_sys::temporal_hash(Temporal::inner(self)) };
        state.write_u32(hash);

        state.finish();
    }
}

impl TInt for TIntInst {}
