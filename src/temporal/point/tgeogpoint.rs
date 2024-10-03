use crate::impl_from_str;
use crate::temporal::interpolation::TInterpolation;
use crate::temporal::{tinstant::TInstant, tsequence::TSequence};
use core::fmt;
use std::{
    ffi::{c_void, CString},
    hash::Hash,
    mem, ptr,
    str::FromStr,
};

use crate::temporal::tsequence_set::TSequenceSet;
use crate::{
    boxes::stbox::STBox,
    collections::base::collection::{impl_collection, Collection},
    errors::ParseError,
    factory,
    temporal::{
        tbool::*,
        temporal::{
            impl_always_and_ever_value_equality_functions, impl_simple_traits_for_temporal,
            SimplifiableTemporal, Temporal,
        },
    },
    utils::to_meos_timestamp,
    MeosEnum,
};
use chrono::{DateTime, TimeZone};
use geos::Geometry;

use super::tpoint::{
    create_set_of_geometries, geometry_to_gserialized, gserialized_to_geometry, impl_tpoint_traits,
    TPointTrait,
};

pub struct TGeogPointInstant {
    _inner: ptr::NonNull<meos_sys::TInstant>,
}

impl_tpoint_traits!(TGeogPointInstant, Instant, true, Geog);

impl TInstant for TGeogPointInstant {
    fn from_inner(inner: *mut meos_sys::TInstant) -> Self {
        Self {
            _inner: ptr::NonNull::new(inner).expect("Null pointers not allowed"),
        }
    }

    fn inner_as_tinstant(&self) -> *const meos_sys::TInstant {
        self._inner.as_ptr()
    }

    fn from_value_and_timestamp<Tz: TimeZone>(value: Self::Type, timestamp: DateTime<Tz>) -> Self {
        Self::from_inner(unsafe {
            meos_sys::tpointinst_make(
                geometry_to_gserialized(&value),
                to_meos_timestamp(&timestamp),
            )
        })
    }
}

impl TPointTrait<true> for TGeogPointInstant {}

pub struct TGeogPointSequence {
    _inner: ptr::NonNull<meos_sys::TSequence>,
}
impl TGeogPointSequence {
    /// Returns the azimuth of the temporal point between the start and end locations.
    ///
    /// ## Returns
    ///
    /// A `f64` indicating the direction of the temporal point.
    ///
    /// ## MEOS Functions
    ///
    /// tpoint_direction
    pub fn direction(&self) -> f64 {
        let mut result = 0.;
        let _ = unsafe { meos_sys::tpoint_direction(self.inner(), ptr::addr_of_mut!(result)) };
        result
    }
}

impl_tpoint_traits!(TGeogPointSequence, Sequence, true, Geog);

impl TSequence for TGeogPointSequence {
    fn from_inner(inner: *mut meos_sys::TSequence) -> Self {
        Self {
            _inner: ptr::NonNull::new(inner).expect("Null pointers not allowed"),
        }
    }

    fn inner_mut_as_tsequence(&self) -> *mut meos_sys::TSequence {
        self._inner.as_ptr()
    }
}

impl TPointTrait<true> for TGeogPointSequence {}

pub struct TGeogPointSequenceSet {
    _inner: ptr::NonNull<meos_sys::TSequenceSet>,
}
impl TGeogPointSequenceSet {
    /// Returns the azimuth of the temporal point between the start and end locations.
    ///
    /// ## Returns
    ///
    /// A `f64` indicating the direction of the temporal point.
    ///
    /// ## MEOS Functions
    ///
    /// tpoint_direction
    pub fn direction(&self) -> f64 {
        let mut result = 0.;
        let _ = unsafe { meos_sys::tpoint_direction(self.inner(), ptr::addr_of_mut!(result)) };
        result
    }
}

impl_tpoint_traits!(TGeogPointSequenceSet, SequenceSet, true, Geog);

impl TSequenceSet for TGeogPointSequenceSet {
    fn from_inner(inner: *mut meos_sys::TSequenceSet) -> Self {
        Self {
            _inner: ptr::NonNull::new(inner).expect("Null pointers not allowed"),
        }
    }
}

impl TPointTrait<true> for TGeogPointSequenceSet {}

#[derive(Debug)]
pub enum TGeogPoint {
    Instant(TGeogPointInstant),
    Sequence(TGeogPointSequence),
    SequenceSet(TGeogPointSequenceSet),
}

impl_from_str!(TGeogPoint);

impl TPointTrait<true> for TGeogPoint {}

impl MeosEnum for TGeogPoint {
    fn from_instant(inner: *mut meos_sys::TInstant) -> Self {
        Self::Instant(TGeogPointInstant::from_inner(inner))
    }

    fn from_sequence(inner: *mut meos_sys::TSequence) -> Self {
        Self::Sequence(TGeogPointSequence::from_inner(inner))
    }

    fn from_sequence_set(inner: *mut meos_sys::TSequenceSet) -> Self {
        Self::SequenceSet(TGeogPointSequenceSet::from_inner(inner))
    }

    fn from_mfjson(mfjson: &str) -> Self {
        let cstr = CString::new(mfjson).unwrap();
        factory::<Self>(unsafe { meos_sys::tint_from_mfjson(cstr.as_ptr()) })
    }
}

impl FromIterator<TGeogPointInstant> for TGeogPointSequence {
    fn from_iter<T: IntoIterator<Item = TGeogPointInstant>>(iter: T) -> Self {
        let vec: Vec<TGeogPointInstant> = iter.into_iter().collect();
        Self::new(&vec, TInterpolation::Linear)
    }
}

impl<'a> FromIterator<&'a TGeogPointInstant> for TGeogPointSequence {
    fn from_iter<T: IntoIterator<Item = &'a TGeogPointInstant>>(iter: T) -> Self {
        let vec: Vec<&TGeogPointInstant> = iter.into_iter().collect();
        Self::new(&vec, TInterpolation::Linear)
    }
}

impl Collection for TGeogPoint {
    impl_collection!(tpoint, Geometry);
    fn contains(&self, element: &Self::Type) -> bool {
        unsafe {
            meos_sys::contains_tpoint_stbox(
                self.inner(),
                meos_sys::geo_to_stbox(geometry_to_gserialized(element)),
            )
        }
    }
}

impl_simple_traits_for_temporal!(TGeogPoint);
impl SimplifiableTemporal for TGeogPoint {}

impl Temporal for TGeogPoint {
    type TI = TGeogPointInstant;
    type TS = TGeogPointSequence;
    type TSS = TGeogPointSequenceSet;
    type TBB = STBox;
    type Enum = TGeogPoint;
    type TBoolType = TBool;

    impl_always_and_ever_value_equality_functions!(point, geometry_to_gserialized);
    fn from_inner_as_temporal(inner: *mut meos_sys::Temporal) -> Self {
        factory::<Self>(inner)
    }

    fn inner(&self) -> *const meos_sys::Temporal {
        match self {
            TGeogPoint::Instant(value) => value.inner(),
            TGeogPoint::Sequence(value) => value.inner(),
            TGeogPoint::SequenceSet(value) => value.inner(),
        }
    }

    fn bounding_box(&self) -> Self::TBB {
        STBox::from_inner(unsafe { meos_sys::tpoint_to_stbox(self.inner()) })
    }

    fn values(&self) -> Vec<Self::Type> {
        let mut count = 0;
        unsafe {
            let values = meos_sys::tpoint_values(self.inner(), ptr::addr_of_mut!(count));

            Vec::from_raw_parts(values, count as usize, count as usize)
                .into_iter()
                .map(gserialized_to_geometry)
                .map(Result::unwrap)
                .collect()
        }
    }

    fn start_value(&self) -> Self::Type {
        gserialized_to_geometry(unsafe { meos_sys::tpoint_start_value(self.inner()) }).unwrap()
    }

    fn end_value(&self) -> Self::Type {
        gserialized_to_geometry(unsafe { meos_sys::tpoint_end_value(self.inner()) }).unwrap()
    }

    fn value_at_timestamp<Tz: TimeZone>(&self, timestamp: DateTime<Tz>) -> Option<Self::Type> {
        let mut result: mem::MaybeUninit<*mut meos_sys::GSERIALIZED> = mem::MaybeUninit::uninit();
        unsafe {
            let success = meos_sys::tpoint_value_at_timestamptz(
                self.inner(),
                to_meos_timestamp(&timestamp),
                true,
                result.as_mut_ptr(),
            );
            if success {
                Some(gserialized_to_geometry(result.assume_init()).unwrap())
            } else {
                None
            }
        }
    }

    fn at_value(&self, value: &Self::Type) -> Option<Self::Enum> {
        let result =
            unsafe { meos_sys::tpoint_at_value(self.inner(), geometry_to_gserialized(value)) };
        if !result.is_null() {
            Some(factory::<Self::Enum>(result))
        } else {
            None
        }
    }
    fn at_values(&self, values: &[Self::Type]) -> Option<Self::Enum> {
        unsafe {
            let set = create_set_of_geometries(values);
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
            meos_sys::tpoint_minus_value(self.inner(), geometry_to_gserialized(&value))
        })
    }

    fn minus_values(&self, values: &[Self::Type]) -> Self::Enum {
        factory::<Self::Enum>(unsafe {
            let set = create_set_of_geometries(values);
            meos_sys::temporal_minus_values(self.inner(), set)
        })
    }

    fn temporal_equal_value(&self, value: &Self::Type) -> Self::TBoolType {
        Self::TBoolType::from_inner_as_temporal(unsafe {
            meos_sys::teq_tpoint_point(self.inner(), geometry_to_gserialized(value))
        })
    }

    fn temporal_not_equal_value(&self, value: &Self::Type) -> Self::TBoolType {
        Self::TBoolType::from_inner_as_temporal(unsafe {
            meos_sys::tne_tpoint_point(self.inner(), geometry_to_gserialized(value))
        })
    }
}
