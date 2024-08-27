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
    geometry_to_gserialized, gserialized_to_geometry, impl_tpoint_traits, TPointTrait,
};

pub struct TGeomPointInstant {
    _inner: ptr::NonNull<meos_sys::TInstant>,
}

impl_tpoint_traits!(TGeomPointInstant, Instant, false, Geom);

impl TInstant for TGeomPointInstant {
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

impl TPointTrait<false> for TGeomPointInstant {}

pub struct TGeomPointSequence {
    _inner: ptr::NonNull<meos_sys::TSequence>,
}
impl TGeomPointSequence {
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

impl_tpoint_traits!(TGeomPointSequence, Sequence, false, Geom);

impl TSequence for TGeomPointSequence {
    fn from_inner(inner: *mut meos_sys::TSequence) -> Self {
        Self {
            _inner: ptr::NonNull::new(inner).expect("Null pointers not allowed"),
        }
    }

    fn inner_mut_as_tsequence(&self) -> *mut meos_sys::TSequence {
        self._inner.as_ptr()
    }
}

impl TPointTrait<false> for TGeomPointSequence {}

pub struct TGeomPointSequenceSet {
    _inner: ptr::NonNull<meos_sys::TSequenceSet>,
}
impl TGeomPointSequenceSet {
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

impl_tpoint_traits!(TGeomPointSequenceSet, SequenceSet, false, Geom);

impl TSequenceSet for TGeomPointSequenceSet {
    fn from_inner(inner: *mut meos_sys::TSequenceSet) -> Self {
        Self {
            _inner: ptr::NonNull::new(inner).expect("Null pointers not allowed"),
        }
    }
}

impl TPointTrait<false> for TGeomPointSequenceSet {}

#[derive(Debug)]
pub enum TGeomPoint {
    Instant(TGeomPointInstant),
    Sequence(TGeomPointSequence),
    SequenceSet(TGeomPointSequenceSet),
}

impl_from_str!(TGeomPoint);

impl MeosEnum for TGeomPoint {
    fn from_instant(inner: *mut meos_sys::TInstant) -> Self {
        Self::Instant(TGeomPointInstant::from_inner(inner))
    }

    fn from_sequence(inner: *mut meos_sys::TSequence) -> Self {
        Self::Sequence(TGeomPointSequence::from_inner(inner))
    }

    fn from_sequence_set(inner: *mut meos_sys::TSequenceSet) -> Self {
        Self::SequenceSet(TGeomPointSequenceSet::from_inner(inner))
    }

    fn from_mfjson(mfjson: &str) -> Self {
        let cstr = CString::new(mfjson).unwrap();
        factory::<Self>(unsafe { meos_sys::tint_from_mfjson(cstr.as_ptr()) })
    }
}

impl FromIterator<TGeomPointInstant> for TGeomPointSequence {
    fn from_iter<T: IntoIterator<Item = TGeomPointInstant>>(iter: T) -> Self {
        let vec: Vec<TGeomPointInstant> = iter.into_iter().collect();
        Self::new(&vec, TInterpolation::Linear)
    }
}

impl<'a> FromIterator<&'a TGeomPointInstant> for TGeomPointSequence {
    fn from_iter<T: IntoIterator<Item = &'a TGeomPointInstant>>(iter: T) -> Self {
        let vec: Vec<&TGeomPointInstant> = iter.into_iter().collect();
        Self::new(&vec, TInterpolation::Linear)
    }
}

impl Collection for TGeomPoint {
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

impl_simple_traits_for_temporal!(TGeomPoint);
impl SimplifiableTemporal for TGeomPoint {}

impl Temporal for TGeomPoint {
    type TI = TGeomPointInstant;
    type TS = TGeomPointSequence;
    type TSS = TGeomPointSequenceSet;
    type TBB = STBox;
    type Enum = TGeomPoint;
    type TBoolType = TBool;

    impl_always_and_ever_value_equality_functions!(point, geometry_to_gserialized);
    fn from_inner_as_temporal(inner: *mut meos_sys::Temporal) -> Self {
        factory::<Self>(inner)
    }

    fn inner(&self) -> *const meos_sys::Temporal {
        match self {
            TGeomPoint::Instant(value) => value.inner(),
            TGeomPoint::Sequence(value) => value.inner(),
            TGeomPoint::SequenceSet(value) => value.inner(),
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
            let cgeos: Vec<_> = values
                .into_iter()
                .map(|geo| geometry_to_gserialized(&geo))
                .collect();
            let set = meos_sys::geoset_make(cgeos.as_ptr() as *mut *const _, values.len() as i32);
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
            let cgeos: Vec<_> = values
                .into_iter()
                .map(|geo| geometry_to_gserialized(&geo))
                .collect();
            let set = meos_sys::geoset_make(cgeos.as_ptr() as *mut *const _, values.len() as i32);
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
