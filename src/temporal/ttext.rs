use std::{
    ffi::{c_void, CStr, CString},
    fmt::Debug,
    hash::Hash,
    ptr,
    str::FromStr,
};

use chrono::{DateTime, TimeZone};

use crate::{
    collections::{
        base::{collection::Collection, span::Span, span_set::SpanSet},
        datetime::{tstz_span::TsTzSpan, tstz_span_set::TsTzSpanSet},
    },
    errors::ParseError,
    factory,
    temporal::{
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

fn from_ctext(ctext: *mut meos_sys::text) -> String {
    unsafe {
        let cstr = meos_sys::text2cstring(ctext);
        let string = CStr::from_ptr(cstr).to_str().unwrap();
        let result = string.to_owned();

        libc::free(cstr as *mut _);

        result
    }
}

fn to_ctext(string: &str) -> *mut meos_sys::text {
    let cstr = CString::new(string).unwrap();
    unsafe { meos_sys::cstring2text(cstr.as_ptr()) }
}

macro_rules! impl_ttext_traits {
    ($type:ty, $temporal_type:ident) => {
        paste::paste! {
            impl Collection for $type {
                type Type = String;

                fn contains(&self, content: &Self::Type) -> bool {
                    let result = unsafe { meos_sys::ever_eq_ttext_text(self.inner(), to_ctext(content)) };
                    result == 1
                }

                fn is_contained_in(&self, _: &Self) -> bool {
                    unimplemented!("Not implemented for `ttext` types")
                }

                fn overlaps(&self, _: &Self) -> bool {
                    unimplemented!("Not implemented for `ttext` types")
                }

                fn is_left(&self, _: &Self) -> bool {
                    unimplemented!("Not implemented for `ttext` types")
                }

                fn is_over_or_left(&self, _: &Self) -> bool {
                    unimplemented!("Not implemented for `ttext` types")
                }

                fn is_over_or_right(&self, _: &Self) -> bool {
                    unimplemented!("Not implemented for `ttext` types")
                }

                fn is_right(&self, _: &Self) -> bool {
                    unimplemented!("Not implemented for `ttext` types")
                }

                fn is_adjacent(&self, _: &Self) -> bool {
                    unimplemented!("Not implemented for `ttext` types")
                }
            }

            impl_simple_traits_for_temporal!($type, ttext);

            impl OrderedTemporal for $type {
                fn min_value(&self) -> Self::Type {
                    from_ctext(unsafe { meos_sys::ttext_min_value(self.inner()) })
                }

                fn max_value(&self) -> Self::Type {
                    from_ctext(unsafe { meos_sys::ttext_max_value(self.inner()) })
                }

                impl_ordered_temporal_functions!(text, to_ctext);
            }

            impl Temporal for $type {
                type TI = TTextInstant;
                type TS = TTextSequence;
                type TSS = TTextSequenceSet;
                type TBB = TsTzSpan;
                type Enum = TText;
                type TBoolType = [<TBool $temporal_type>];

                impl_always_and_ever_value_equality_functions!(text, to_ctext);
                fn from_inner_as_temporal(inner: *const meos_sys::Temporal) -> Self {
                    Self {
                        _inner: inner as *const meos_sys::[<T $temporal_type>],
                    }
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
                        let values = meos_sys::ttext_values(self.inner(), ptr::addr_of_mut!(count));

                        Vec::from_raw_parts(values, count as usize, count as usize).into_iter().map(from_ctext).collect()
                    }
                }

                fn start_value(&self) -> Self::Type {
                    from_ctext(unsafe { meos_sys::ttext_start_value(self.inner()) })
                }

                fn end_value(&self) -> Self::Type {
                    from_ctext(unsafe { meos_sys::ttext_end_value(self.inner()) })
                }

                fn value_at_timestamp<Tz: TimeZone>(
                    &self,
                    timestamp: DateTime<Tz>,
                ) -> Option<Self::Type> {
                    let mut result = to_ctext("");
                    unsafe {
                        let success = meos_sys::ttext_value_at_timestamptz(
                            self.inner(),
                            to_meos_timestamp(&timestamp),
                            true,
                            ptr::addr_of_mut!(result),
                        );
                        if success {
                            Some(from_ctext(result))
                        } else {
                            None
                        }
                    }
                }

                fn at_value(&self, value: &Self::Type) -> Option<Self::Enum> {
                    let result = unsafe { meos_sys::ttext_at_value(self.inner(), to_ctext(value)) };
                    if !result.is_null() {
                        Some(factory::<Self::Enum>(result))
                    } else {
                        None
                    }
                }
                fn at_values(&self, values: &[Self::Type]) -> Option<Self::Enum> {
                    unsafe {
                        let ctexts: Vec<_> = values.into_iter().map(|text| to_ctext(&text)).collect();
                        let set = meos_sys::textset_make(ctexts.as_ptr() as *mut *const _, values.len() as i32);
                        let result = meos_sys::temporal_at_values(self.inner(), set);
                        if !result.is_null() {
                            Some(factory::<Self::Enum>(result))
                        } else {
                            None
                        }
                    }
                }

                fn minus_value(&self, value: Self::Type) -> Self::Enum {
                    factory::<Self::Enum>(unsafe {
                        meos_sys::ttext_minus_value(self.inner(), to_ctext(&value))
                    })
                }

                fn minus_values(&self, values: &[Self::Type]) -> Self::Enum {
                    factory::<Self::Enum>(unsafe {
                        let ctexts: Vec<_> = values.into_iter().map(|text| to_ctext(&text)).collect();
                        let set = meos_sys::textset_make(ctexts.as_ptr() as *mut *const _, values.len() as i32);
                        meos_sys::temporal_minus_values(self.inner(), set)
                    })
                }

                fn temporal_equal_value(&self, value: &Self::Type) -> Self::TBoolType {
                    Self::TBoolType::from_inner_as_temporal(unsafe {
                        meos_sys::teq_ttext_text(self.inner(), to_ctext(value))
                    })
                }

                fn temporal_not_equal_value(&self, value: &Self::Type) -> Self::TBoolType {
                    Self::TBoolType::from_inner_as_temporal(unsafe {
                        meos_sys::tne_ttext_text(self.inner(), to_ctext(value))
                    })
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum TText {
    Instant(TTextInstant),
    Sequence(TTextSequence),
    SequenceSet(TTextSequenceSet),
}

impl MeosEnum for TText {
    fn from_instant(inner: *const meos_sys::TInstant) -> Self {
        Self::Instant(TTextInstant::from_inner(inner))
    }

    fn from_sequence(inner: *const meos_sys::TSequence) -> Self {
        Self::Sequence(TTextSequence::from_inner(inner))
    }

    fn from_sequence_set(inner: *const meos_sys::TSequenceSet) -> Self {
        Self::SequenceSet(TTextSequenceSet::from_inner(inner))
    }

    fn from_mfjson(mfjson: &str) -> Self {
        let cstr = CString::new(mfjson).unwrap();
        factory::<Self>(unsafe { meos_sys::ttext_from_mfjson(cstr.as_ptr()) })
    }

    fn inner(&self) -> *const meos_sys::Temporal {
        match self {
            TText::Instant(value) => value.inner(),
            TText::Sequence(value) => value.inner(),
            TText::SequenceSet(value) => value.inner(),
        }
    }
}

pub trait TTextTrait:
    Temporal<
    Type = String,
    TI = TTextInstant,
    TS = TTextSequence,
    TSS = TTextSequenceSet,
    TBB = TsTzSpan,
>
{
    fn concatenate_str(&self, string: &str) -> Self {
        Self::from_inner_as_temporal(unsafe {
            meos_sys::textcat_ttext_text(self.inner(), to_ctext(string))
        })
    }

    fn lowercase(&self) -> Self {
        Self::from_inner_as_temporal(unsafe { meos_sys::ttext_lower(self.inner()) })
    }

    fn uppercase(&self) -> Self {
        Self::from_inner_as_temporal(unsafe { meos_sys::ttext_upper(self.inner()) })
    }
}

macro_rules! impl_debug {
    ($type:ty) => {
        impl Debug for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let out_str = unsafe { meos_sys::ttext_out(self.inner()) };
                let c_str = unsafe { CStr::from_ptr(out_str) };
                let str = c_str.to_str().map_err(|_| std::fmt::Error)?;
                let result = f.write_str(str);
                unsafe { libc::free(out_str as *mut c_void) };
                result
            }
        }
    };
}

pub struct TTextInstant {
    _inner: *const meos_sys::TInstant,
}

impl TInstant for TTextInstant {
    fn from_inner(inner: *const meos_sys::TInstant) -> Self {
        Self { _inner: inner }
    }

    fn inner_as_tinstant(&self) -> *const meos_sys::TInstant {
        self._inner
    }

    fn from_value_and_timestamp<Tz: TimeZone>(value: Self::Type, timestamp: DateTime<Tz>) -> Self {
        Self::from_inner(unsafe {
            meos_sys::ttextinst_make(to_ctext(&value), to_meos_timestamp(&timestamp))
        })
    }
}

impl TTextTrait for TTextInstant {}

impl_ttext_traits!(TTextInstant, Instant);
impl_debug!(TTextInstant);

pub struct TTextSequence {
    _inner: *const meos_sys::TSequence,
}
impl TTextSequence {
    /// Creates a temporal object from a value and a TsTz span.
    ///
    /// ## Arguments
    /// * `value` - Base value.
    /// * `time_span` - Time object to use as the temporal dimension.
    ///
    /// ## Returns
    /// A new temporal object.
    pub fn from_value_and_tstz_span<Tz: TimeZone>(value: String, time_span: TsTzSpan) -> Self {
        Self::from_inner(unsafe {
            meos_sys::ttextseq_from_base_tstzspan(to_ctext(&value), time_span.inner())
        })
    }
}

impl TSequence for TTextSequence {
    fn from_inner(inner: *const meos_sys::TSequence) -> Self {
        Self { _inner: inner }
    }

    fn inner_as_tsequence(&self) -> *const meos_sys::TSequence {
        self._inner
    }
}

impl TTextTrait for TTextSequence {}

impl_ttext_traits!(TTextSequence, Sequence);
impl_debug!(TTextSequence);

pub struct TTextSequenceSet {
    _inner: *const meos_sys::TSequenceSet,
}

impl TTextSequenceSet {
    /// Creates a temporal object from a base value and a TsTz span set.
    ///
    /// ## Arguments
    /// * `value` - Base value.
    /// * `time_span_set` - Time object to use as the temporal dimension.
    ///
    /// ## Returns
    /// A new temporal object.
    pub fn from_value_and_tstz_span_set<Tz: TimeZone>(
        value: String,
        time_span_set: TsTzSpanSet,
    ) -> Self {
        Self::from_inner(unsafe {
            meos_sys::ttextseqset_from_base_tstzspanset(to_ctext(&value), time_span_set.inner())
        })
    }
}

impl TSequenceSet for TTextSequenceSet {
    fn from_inner(inner: *const meos_sys::TSequenceSet) -> Self {
        Self { _inner: inner }
    }
}
impl TTextTrait for TTextSequenceSet {}

impl_ttext_traits!(TTextSequenceSet, SequenceSet);
impl_debug!(TTextSequenceSet);

impl From<TTextInstant> for TText {
    fn from(value: TTextInstant) -> Self {
        TText::Instant(value)
    }
}

impl From<TTextSequence> for TText {
    fn from(value: TTextSequence) -> Self {
        TText::Sequence(value)
    }
}

impl From<TTextSequenceSet> for TText {
    fn from(value: TTextSequenceSet) -> Self {
        TText::SequenceSet(value)
    }
}

impl TryFrom<TText> for TTextInstant {
    type Error = ParseError;
    fn try_from(value: TText) -> Result<Self, Self::Error> {
        if let TText::Instant(new_value) = value {
            Ok(new_value)
        } else {
            Err(ParseError)
        }
    }
}

impl TryFrom<TText> for TTextSequence {
    type Error = ParseError;
    fn try_from(value: TText) -> Result<Self, Self::Error> {
        if let TText::Sequence(new_value) = value {
            Ok(new_value)
        } else {
            Err(ParseError)
        }
    }
}

impl TryFrom<TText> for TTextSequenceSet {
    type Error = ParseError;
    fn try_from(value: TText) -> Result<Self, Self::Error> {
        if let TText::SequenceSet(new_value) = value {
            Ok(new_value)
        } else {
            Err(ParseError)
        }
    }
}
