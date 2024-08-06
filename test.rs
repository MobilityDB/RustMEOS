#![feature(prelude_import)]
#![allow(refining_impl_trait)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use std::{
    ffi::{CStr, CString},
    sync::Once,
};
use bitmask_enum::bitmask;
use boxes::r#box::Box as MeosBox;
use collections::base::collection::Collection;
pub use meos_sys;
pub mod boxes {
    pub mod r#box {
        use chrono::{DateTime, TimeDelta, TimeZone, Utc};
        use crate::{
            collections::{base::collection::Collection, datetime::tstz_span::TsTzSpan},
            WKBVariant,
        };
        pub trait Box: Collection {
            fn from_wkb(wkb: &[u8]) -> Self;
            fn from_hexwkb(hexwkb: &[u8]) -> Self;
            fn from_time<Tz: TimeZone>(time: DateTime<Tz>) -> Self;
            fn from_temporal_span(span: TsTzSpan) -> Self;
            fn tstzspan(&self) -> TsTzSpan;
            fn as_wkb(&self, variant: WKBVariant) -> &[u8];
            fn as_hexwkb(&self, variant: WKBVariant) -> &[u8];
            fn round(&self, max_decimals: i32) -> Self;
            fn expand_time(&self, other: TimeDelta) -> Self;
            fn is_tmin_inclusive(&self) -> Option<bool>;
            fn is_tmax_inclusive(&self) -> Option<bool>;
            fn shift_scale_time(
                &self,
                delta: Option<TimeDelta>,
                width: Option<TimeDelta>,
            ) -> Self;
            fn intersection(&self, other: &Self) -> Option<Self>;
            fn union(&self, other: &Self, strict: bool) -> Option<Self>;
            fn nearest_approach_distance(&self, other: &Self) -> f64;
            fn has_x(&self) -> bool;
            fn has_t(&self) -> bool;
            fn xmin(&self) -> Option<f64>;
            fn xmax(&self) -> Option<f64>;
            fn tmin(&self) -> Option<DateTime<Utc>>;
            fn tmax(&self) -> Option<DateTime<Utc>>;
        }
    }
    pub mod stbox {
        use std::{
            cmp, ffi::{c_void, CStr, CString},
            fmt::Debug, ptr,
        };
        use chrono::{DateTime, TimeDelta, TimeZone, Utc};
        #[cfg(feature = "geos")]
        use geos::{Geom, Geometry};
        use crate::{
            collections::{
                base::{
                    collection::{impl_collection, Collection},
                    span::Span,
                },
                datetime::tstz_span::TsTzSpan,
            },
            errors::ParseError,
            utils::{create_interval, from_meos_timestamp, to_meos_timestamp},
            WKBVariant,
        };
        use super::r#box::Box as MeosBox;
        pub struct STBox {
            _inner: *const meos_sys::STBox,
        }
        impl MeosBox for STBox {
            fn from_wkb(wkb: &[u8]) -> Self {
                unsafe {
                    Self::from_inner(meos_sys::stbox_from_wkb(wkb.as_ptr(), wkb.len()))
                }
            }
            fn from_hexwkb(hexwkb: &[u8]) -> Self {
                let c_hexwkb = CString::new(hexwkb).unwrap();
                unsafe {
                    let inner = meos_sys::stbox_from_hexwkb(c_hexwkb.as_ptr());
                    Self::from_inner(inner)
                }
            }
            fn from_temporal_span(span: TsTzSpan) -> Self {
                unsafe { Self::from_inner(meos_sys::tstzspan_to_stbox(span.inner())) }
            }
            /// Creates an `STBox` from a `DateTime` value.
            ///
            /// ## Arguments
            /// * `time` - A `DateTime<Tz>` instance representing a timestamp.
            ///
            /// ## Returns
            /// An `STBox` instance.
            ///
            /// ## Example
            /// ```
            /// # use meos::boxes::stbox::STBox;
            /// use meos::boxes::r#box::Box;
            /// use chrono::{Utc, TimeZone};
            /// let datetime = Utc.with_ymd_and_hms(2020, 5, 15, 12, 0, 0).unwrap();
            /// let stbox = STBox::from_time(datetime);
            /// assert_eq!(stbox.tmin().unwrap(), datetime);
            /// ```
            fn from_time<Tz: TimeZone>(time: DateTime<Tz>) -> Self {
                let timestamptz = to_meos_timestamp(&time);
                unsafe { Self::from_inner(meos_sys::timestamptz_to_stbox(timestamptz)) }
            }
            fn tstzspan(&self) -> TsTzSpan {
                unsafe { TsTzSpan::from_inner(meos_sys::stbox_to_tstzspan(self._inner)) }
            }
            fn as_wkb(&self, variant: WKBVariant) -> &[u8] {
                unsafe {
                    let mut size: usize = 0;
                    let ptr = meos_sys::stbox_as_wkb(
                        self._inner,
                        variant.into(),
                        &mut size,
                    );
                    std::slice::from_raw_parts(ptr, size)
                }
            }
            fn as_hexwkb(&self, variant: WKBVariant) -> &[u8] {
                unsafe {
                    let mut size: usize = 0;
                    let hexwkb_ptr = meos_sys::stbox_as_hexwkb(
                        self.inner(),
                        variant.into(),
                        &mut size,
                    );
                    CStr::from_ptr(hexwkb_ptr).to_bytes()
                }
            }
            fn has_x(&self) -> bool {
                unsafe { meos_sys::stbox_hasx(self.inner()) }
            }
            fn has_t(&self) -> bool {
                unsafe { meos_sys::stbox_hast(self.inner()) }
            }
            fn xmin(&self) -> Option<f64> {
                unsafe {
                    let mut value = 0.0;
                    let ptr: *mut f64 = &raw mut value;
                    if meos_sys::stbox_xmin(self.inner(), ptr) {
                        Some(value)
                    } else {
                        None
                    }
                }
            }
            fn xmax(&self) -> Option<f64> {
                unsafe {
                    let mut value = 0.0;
                    let ptr: *mut f64 = &raw mut value;
                    if meos_sys::stbox_xmax(self.inner(), ptr) {
                        Some(value)
                    } else {
                        None
                    }
                }
            }
            fn tmin(&self) -> Option<DateTime<Utc>> {
                unsafe {
                    let mut value: i64 = 0;
                    let ptr: *mut i64 = &raw mut value;
                    if meos_sys::stbox_tmin(self.inner(), ptr) {
                        Some(from_meos_timestamp(value))
                    } else {
                        None
                    }
                }
            }
            fn tmax(&self) -> Option<DateTime<Utc>> {
                unsafe {
                    let mut value: i64 = 0;
                    let ptr: *mut i64 = &raw mut value;
                    if meos_sys::stbox_tmax(self.inner(), ptr) {
                        DateTime::from_timestamp_micros(value)
                    } else {
                        None
                    }
                }
            }
            fn is_tmin_inclusive(&self) -> Option<bool> {
                unsafe {
                    let mut is_inclusive = false;
                    let ptr: *mut bool = &raw mut is_inclusive;
                    if meos_sys::stbox_tmin_inc(self.inner(), ptr) {
                        Some(is_inclusive)
                    } else {
                        None
                    }
                }
            }
            fn is_tmax_inclusive(&self) -> Option<bool> {
                unsafe {
                    let mut is_inclusive = false;
                    let ptr: *mut bool = &raw mut is_inclusive;
                    if meos_sys::stbox_tmax_inc(self.inner(), ptr) {
                        Some(is_inclusive)
                    } else {
                        None
                    }
                }
            }
            fn expand_time(&self, duration: TimeDelta) -> STBox {
                let interval = create_interval(duration);
                unsafe {
                    Self::from_inner(
                        meos_sys::stbox_expand_time(self.inner(), &raw const interval),
                    )
                }
            }
            fn shift_scale_time(
                &self,
                delta: Option<TimeDelta>,
                width: Option<TimeDelta>,
            ) -> STBox {
                let d = {
                    if let Some(d) = delta {
                        &*Box::new(create_interval(d)) as *const meos_sys::Interval
                    } else {
                        std::ptr::null()
                    }
                };
                let w = {
                    if let Some(w) = width {
                        &*Box::new(create_interval(w)) as *const meos_sys::Interval
                    } else {
                        std::ptr::null()
                    }
                };
                let modified = unsafe {
                    meos_sys::stbox_shift_scale_time(self._inner, d, w)
                };
                STBox::from_inner(modified)
            }
            fn round(&self, max_decimals: i32) -> STBox {
                let result = unsafe {
                    meos_sys::stbox_round(self.inner(), max_decimals)
                };
                STBox::from_inner(result)
            }
            fn union(&self, other: &STBox, strict: bool) -> Option<STBox> {
                let result = unsafe {
                    meos_sys::union_stbox_stbox(self.inner(), other.inner(), strict)
                };
                if result.is_null() { None } else { Some(STBox::from_inner(result)) }
            }
            fn intersection(&self, other: &STBox) -> Option<STBox> {
                let result = unsafe {
                    meos_sys::intersection_stbox_stbox(self.inner(), other.inner())
                };
                if result.is_null() { None } else { Some(STBox::from_inner(result)) }
            }
            fn nearest_approach_distance(&self, other: &STBox) -> f64 {
                unsafe { meos_sys::nad_stbox_stbox(self.inner(), other.inner()) }
            }
        }
        impl STBox {
            fn inner(&self) -> *const meos_sys::STBox {
                self._inner
            }
            fn from_inner(inner: *const meos_sys::STBox) -> Self {
                Self { _inner: inner }
            }
            #[cfg(feature = "geos")]
            pub fn from_geos(value: Geometry) -> Self {
                let v: Vec<u8> = value.to_wkb().unwrap().into();
                Self::from_wkb(&v)
            }
            #[cfg(feature = "geos")]
            pub fn geos_geometry(&self) -> Geometry {
                Geometry::new_from_wkb(self.as_wkb(WKBVariant::none())).unwrap()
            }
            pub fn expand_space(&self, value: f64) -> STBox {
                unsafe {
                    Self::from_inner(meos_sys::stbox_expand_space(self.inner(), value))
                }
            }
        }
        impl Collection for STBox {
            type Type = ();
            fn is_contained_in(&self, container: &Self) -> bool {
                unsafe {
                    meos_sys::contained_stbox_stbox(self.inner(), container.inner())
                }
            }
            fn overlaps(&self, other: &Self) -> bool {
                unsafe { meos_sys::overlaps_stbox_stbox(self.inner(), other.inner()) }
            }
            fn is_left(&self, other: &Self) -> bool {
                unsafe { meos_sys::left_stbox_stbox(self.inner(), other.inner()) }
            }
            fn is_over_or_left(&self, other: &Self) -> bool {
                unsafe { meos_sys::overleft_stbox_stbox(self.inner(), other.inner()) }
            }
            fn is_over_or_right(&self, other: &Self) -> bool {
                unsafe { meos_sys::overright_stbox_stbox(self.inner(), other.inner()) }
            }
            fn is_right(&self, other: &Self) -> bool {
                unsafe { meos_sys::right_stbox_stbox(self.inner(), other.inner()) }
            }
            fn is_adjacent(&self, other: &Self) -> bool {
                unsafe { meos_sys::adjacent_stbox_stbox(self.inner(), other.inner()) }
            }
            fn contains(&self, content: &Self::Type) -> bool {
                unsafe {
                    meos_sys::contains_stbox_tpoint(self.inner(), std::ptr::null())
                }
            }
        }
        impl cmp::PartialEq for STBox {
            /// Checks if two `STBox` instances are equal.
            ///
            /// # Arguments
            /// * `other` - Another `STBox` instance.
            ///
            /// ## Returns
            /// * `true` if the spans are equal, `false` otherwise.
            ///
            /// ## Example
            /// ```
            /// # use meos::boxes::stbox::STBox;
            /// # use meos::init;
            /// use std::str::FromStr;
            /// # init();
            /// let span1: STBox = STBox::from_str("STBOX ZT(((1.0,2.0,3.0),(4.0,5.0,6.0)),[2001-01-01, 2001-01-02])").unwrap();
            /// let span2: STBox = STBox::from_str("STBOX ZT(((1.0,2.0,3.0),(4.0,5.0,6.0)),[2001-01-01, 2001-01-02])").unwrap();
            /// assert_eq!(span1, span2);
            /// ```
            fn eq(&self, other: &Self) -> bool {
                unsafe { meos_sys::stbox_eq(self._inner, other._inner) }
            }
        }
        impl cmp::Eq for STBox {}
        impl Debug for STBox {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let out_str = unsafe { meos_sys::stbox_out(self._inner, 3) };
                let c_str = unsafe { CStr::from_ptr(out_str) };
                let str = c_str.to_str().map_err(|_| std::fmt::Error)?;
                let result = f.write_str(str);
                unsafe { libc::free(out_str as *mut c_void) };
                result
            }
        }
        impl Clone for STBox {
            fn clone(&self) -> Self {
                unsafe { Self::from_inner(meos_sys::stbox_copy(self._inner)) }
            }
        }
        impl std::str::FromStr for STBox {
            type Err = ParseError;
            /// Parses a `STBox` from a string representation.
            ///
            /// ## Arguments
            /// * `string` - A string slice containing the representation.
            ///
            /// ## Returns
            /// * A `STBox` instance.
            ///
            /// ## Errors
            /// * Returns `ParseSpanError` if the string cannot be parsed.
            ///
            /// ## Example
            /// ```
            /// # use meos::boxes::stbox::STBox;
            /// # use meos::collections::base::span::Span;
            /// # use meos::collections::datetime::tstz_span::TsTzSpan;
            /// use meos::boxes::r#box::Box;
            /// use std::str::FromStr;
            /// # use meos::init;
            /// # init();
            ///
            /// let stbox: STBox = "STBOX ZT(((1.0,2.0,3.0),(4.0,5.0,6.0)),[2001-01-01, 2001-01-02])".parse().expect("Failed to parse span");
            /// let temporal_span: TsTzSpan = stbox.tstzspan();
            /// assert_eq!(temporal_span, TsTzSpan::from_str("[2001-01-01, 2001-01-02]").unwrap());
            /// ```
            fn from_str(string: &str) -> Result<Self, Self::Err> {
                CString::new(string)
                    .map_err(|_| ParseError)
                    .map(|string| {
                        let inner = unsafe { meos_sys::stbox_in(string.as_ptr()) };
                        Self::from_inner(inner)
                    })
            }
        }
        impl From<&STBox> for TsTzSpan {
            fn from(stbox: &STBox) -> Self {
                unsafe {
                    TsTzSpan::from_inner(meos_sys::stbox_to_tstzspan(stbox.inner()))
                }
            }
        }
    }
    pub mod tbox {
        use std::{
            cmp, ffi::{c_void, CStr, CString},
            fmt::Debug, ptr,
        };
        use chrono::{DateTime, TimeDelta, TimeZone, Utc};
        use crate::{
            collections::{
                base::{
                    collection::{impl_collection, Collection},
                    span::Span,
                },
                datetime::tstz_span::TsTzSpan,
                number::{
                    float_span::FloatSpan, int_span::IntSpan, number_span::NumberSpan,
                },
            },
            errors::ParseError,
            utils::{create_interval, from_meos_timestamp, to_meos_timestamp},
            WKBVariant,
        };
        use super::r#box::Box as MeosBox;
        pub struct TBox {
            _inner: *const meos_sys::TBox,
        }
        impl MeosBox for TBox {
            fn from_wkb(wkb: &[u8]) -> Self {
                unsafe {
                    Self::from_inner(meos_sys::tbox_from_wkb(wkb.as_ptr(), wkb.len()))
                }
            }
            fn from_hexwkb(hexwkb: &[u8]) -> Self {
                let c_hexwkb = CString::new(hexwkb).unwrap();
                unsafe {
                    let inner = meos_sys::tbox_from_hexwkb(c_hexwkb.as_ptr());
                    Self::from_inner(inner)
                }
            }
            fn from_temporal_span(value: TsTzSpan) -> Self {
                unsafe { Self::from_inner(meos_sys::span_to_tbox(value.inner())) }
            }
            /// Creates a new `TBox` instance from a `DateTime<Tz>` object.
            /// Using the value as the lower and upper bounds of the temporal span
            ///
            /// ## Arguments
            /// * `time` - A `DateTime<Utc>` instance.
            ///
            /// ## Returns
            /// A new `TBox` instance representing the timestamp.
            ///
            /// ## Example
            /// ```
            /// # use meos::boxes::tbox::TBox;
            /// use meos::boxes::r#box::Box;
            /// use chrono::{Utc, TimeZone};
            /// let datetime = Utc.with_ymd_and_hms(2020, 5, 15, 12, 0, 0).unwrap();
            /// let tbox = TBox::from_time(datetime);
            /// assert_eq!(tbox.tmin().unwrap(), datetime);
            /// ```
            fn from_time<Tz: TimeZone>(time: DateTime<Tz>) -> Self {
                let timestamptz = to_meos_timestamp(&time);
                unsafe { Self::from_inner(meos_sys::timestamptz_to_tbox(timestamptz)) }
            }
            /// Converts the `TBox` into a `TsTzSpan` representing the timestamp with time zone span.
            ///
            /// ## Returns
            /// A `TsTzSpan` instance.
            ///
            /// ## Example
            /// ```
            /// # use meos::boxes::tbox::TBox;
            /// use meos::boxes::r#box::Box;
            /// # use meos::init;
            /// use chrono::{Utc, TimeZone, TimeDelta};
            /// # init();
            /// let datetime = Utc.with_ymd_and_hms(2020, 5, 15, 12, 0, 0).unwrap();
            /// let tbox = TBox::from_time(datetime);
            /// let tstzspan = tbox.tstzspan();
            /// assert_eq!(tstzspan, (datetime..=datetime).into()); // Assuming `TsTzSpan` has a `to_string` method
            /// ```
            fn tstzspan(&self) -> TsTzSpan {
                unsafe { TsTzSpan::from_inner(meos_sys::tbox_to_tstzspan(self._inner)) }
            }
            fn as_wkb(&self, variant: WKBVariant) -> &[u8] {
                unsafe {
                    let mut size: usize = 0;
                    let ptr = meos_sys::tbox_as_wkb(
                        self._inner,
                        variant.into(),
                        &mut size,
                    );
                    std::slice::from_raw_parts(ptr, size)
                }
            }
            fn as_hexwkb(&self, variant: WKBVariant) -> &[u8] {
                unsafe {
                    let mut size: usize = 0;
                    let ptr = meos_sys::tbox_as_hexwkb(
                        self._inner,
                        variant.into(),
                        &mut size,
                    );
                    CStr::from_ptr(ptr).to_bytes()
                }
            }
            /// Checks if the `TBox` has a valid X dimension.
            ///
            /// ## Returns
            /// A boolean indicating the presence of an X dimension.
            ///
            /// ## Example
            /// ```
            /// # use meos::boxes::tbox::TBox;
            /// use meos::boxes::r#box::Box;
            /// let tbox = TBox::from_int(5);
            /// assert!(tbox.has_x());
            /// ```
            fn has_x(&self) -> bool {
                unsafe { meos_sys::tbox_hasx(self.inner()) }
            }
            /// Checks if the `TBox` has a valid T (time) dimension.
            ///
            /// ## Returns
            /// A boolean indicating the presence of a T dimension.
            ///
            /// ## Example
            /// ```
            /// # use meos::boxes::tbox::TBox;
            /// use meos::boxes::r#box::Box;
            /// use chrono::{Utc, TimeZone};
            /// let datetime = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
            /// let tbox = TBox::from_time(datetime);
            /// assert!(tbox.has_t());
            /// ```
            fn has_t(&self) -> bool {
                unsafe { meos_sys::tbox_hast(self.inner()) }
            }
            /// Retrieves the minimum value of the X dimension in the `TBox`.
            ///
            /// ## Returns
            /// An optional `f64` representing the minimum X value, or `None` if not available.
            ///
            /// ## Example
            /// ```
            /// # use meos::boxes::tbox::TBox;
            /// use meos::boxes::r#box::Box;
            /// let tbox = TBox::from_float(3.14);
            /// assert_eq!(tbox.xmin(), Some(3.14));
            /// ```
            fn xmin(&self) -> Option<f64> {
                unsafe {
                    let mut value = 0.0;
                    let ptr: *mut f64 = &raw mut value;
                    if meos_sys::tbox_xmin(self.inner(), ptr) {
                        Some(value)
                    } else {
                        None
                    }
                }
            }
            /// Retrieves the maximum value of the X dimension in the `TBox`.
            ///
            /// ## Returns
            /// An optional `f64` representing the maximum X value, or `None` if not available.
            ///
            /// ## Example
            /// ```
            /// # use meos::boxes::tbox::TBox;
            /// use meos::boxes::r#box::Box;
            /// let tbox = TBox::from_float(3.14);
            /// assert_eq!(tbox.xmax(), Some(3.14));
            /// ```
            fn xmax(&self) -> Option<f64> {
                unsafe {
                    let mut value = 0.0;
                    let ptr: *mut f64 = &raw mut value;
                    if meos_sys::tbox_xmax(self.inner(), ptr) {
                        Some(value)
                    } else {
                        None
                    }
                }
            }
            /// Retrieves the minimum value of the T (time) dimension in the `TBox`.
            ///
            /// ## Returns
            /// An optional `DateTime<Utc>` representing the minimum time value, or `None` if not available.
            ///
            /// ## Example
            /// ```
            /// # use meos::boxes::tbox::TBox;
            /// use meos::boxes::r#box::Box;
            /// use chrono::{Utc, TimeZone};
            /// let datetime = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
            /// let tbox = TBox::from_time(datetime);
            /// assert_eq!(tbox.tmin(), Some(datetime));
            /// ```
            fn tmin(&self) -> Option<DateTime<Utc>> {
                unsafe {
                    let mut value: i64 = 0;
                    let ptr: *mut i64 = &raw mut value;
                    if meos_sys::tbox_tmin(self.inner(), ptr) {
                        Some(from_meos_timestamp(value))
                    } else {
                        None
                    }
                }
            }
            /// Retrieves the maximum value of the T (time) dimension in the `TBox`.
            ///
            /// ## Returns
            /// An optional `DateTime<Utc>` representing the maximum time value, or `None` if not available.
            ///
            /// ## Example
            /// ```
            /// # use meos::boxes::tbox::TBox;
            /// use meos::boxes::r#box::Box;
            /// use chrono::{Utc, TimeZone};
            /// let datetime = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
            /// let tbox = TBox::from_time(datetime);
            /// assert_eq!(tbox.tmax(), Some(datetime));
            /// ```
            fn tmax(&self) -> Option<DateTime<Utc>> {
                unsafe {
                    let mut value: i64 = 0;
                    let ptr: *mut i64 = &raw mut value;
                    if meos_sys::tbox_tmax(self.inner(), ptr) {
                        Some(from_meos_timestamp(value))
                    } else {
                        None
                    }
                }
            }
            /// Checks if the minimum T (time) value is inclusive in the `TBox`.
            ///
            /// ## Returns
            /// An optional `bool` indicating if the minimum T value is inclusive, or `None` if not available.
            ///
            /// ## Example
            /// ```
            /// # use meos::boxes::tbox::TBox;
            /// use meos::boxes::r#box::Box;
            /// let tbox = TBox::from_int(5);
            /// assert_eq!(tbox.is_tmin_inclusive(), None); // No temporal dimension
            /// ```
            fn is_tmin_inclusive(&self) -> Option<bool> {
                unsafe {
                    let mut is_inclusive = false;
                    let ptr: *mut bool = &raw mut is_inclusive;
                    if meos_sys::tbox_tmin_inc(self.inner(), ptr) {
                        Some(is_inclusive)
                    } else {
                        None
                    }
                }
            }
            /// Checks if the maximum T (time) value is inclusive in the `TBox`.
            ///
            /// ## Returns
            /// An optional `bool` indicating if the maximum T value is inclusive, or `None` if not available.
            ///
            /// ## Example
            /// ```
            /// # use meos::boxes::tbox::TBox;
            /// use meos::boxes::r#box::Box;
            /// use chrono::{Utc, TimeZone};
            /// let datetime = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
            /// let tbox = TBox::from_time(datetime);
            /// assert_eq!(tbox.is_tmax_inclusive(), Some(true));
            /// ```
            fn is_tmax_inclusive(&self) -> Option<bool> {
                unsafe {
                    let mut is_inclusive = false;
                    let ptr: *mut bool = &raw mut is_inclusive;
                    if meos_sys::tbox_tmax_inc(self.inner(), ptr) {
                        Some(is_inclusive)
                    } else {
                        None
                    }
                }
            }
            /// Expands the `TBox` by a specified duration on the T (time) dimension.
            ///
            /// ## Arguments
            /// * `duration` - The duration by which to expand the T dimension.
            ///
            /// ## Returns
            /// A new `TBox` instance with expanded bounds.
            ///
            /// ## Example
            /// ```
            /// # use meos::boxes::tbox::TBox;
            /// use meos::boxes::r#box::Box;
            /// use chrono::{Utc, TimeZone, TimeDelta};
            /// let datetime = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
            /// let tbox = TBox::from_time(datetime);
            /// let expanded_tbox = tbox.expand_time(TimeDelta::days(1));
            ///
            /// assert!(expanded_tbox.tmin().unwrap() < datetime);
            /// assert!(expanded_tbox.tmax().unwrap() > datetime);
            /// ```
            fn expand_time(&self, duration: TimeDelta) -> TBox {
                let interval = create_interval(duration);
                unsafe {
                    Self::from_inner(
                        meos_sys::tbox_expand_time(self.inner(), &raw const interval),
                    )
                }
            }
            /// Rounds the numerical values in the `TBox` to a specified number of decimal places.
            ///
            /// ## Arguments
            /// * `max_decimals` - The maximum number of decimal places.
            ///
            /// ## Returns
            /// A new `TBox` instance with rounded values.
            ///
            /// ## Example
            /// ```
            /// # use meos::boxes::tbox::TBox;
            /// use meos::boxes::r#box::Box;
            /// let tbox = TBox::from_float(3.141592);
            /// let rounded_tbox = tbox.round(2);
            /// assert_eq!(rounded_tbox.xmin(), Some(3.14));
            /// ```
            fn round(&self, max_decimals: i32) -> TBox {
                let result = unsafe { meos_sys::tbox_round(self.inner(), max_decimals) };
                TBox::from_inner(result)
            }
            /// Calculates the nearest approach distance between two `TBox` instances.
            ///
            /// ## Arguments
            /// * `other` - The other `TBox` to measure the distance to.
            ///
            /// ## Returns
            /// A `f64` representing the nearest approach distance.
            ///
            /// ## Example
            /// ```
            /// # use meos::boxes::tbox::TBox;
            /// use meos::boxes::r#box::Box;
            /// let tbox1 = TBox::from_float(1.0);
            /// let tbox2 = TBox::from_float(5.0);
            /// let distance = tbox1.nearest_approach_distance(&tbox2);
            /// assert_eq!(distance, 4.0);
            /// ```
            fn nearest_approach_distance(&self, other: &TBox) -> f64 {
                unsafe { meos_sys::nad_tboxfloat_tboxfloat(self.inner(), other.inner()) }
            }
            /// Shifts and scales the T (time) dimension of the `TBox`.
            ///
            /// ## Arguments
            /// * `delta` - The duration to shift by.
            /// * `width` - The new width for the T dimension.
            ///
            /// ## Returns
            /// A new `TBox` instance with shifted and scaled bounds.
            ///
            /// ## Example
            /// ```
            /// # use meos::boxes::tbox::TBox;
            /// use meos::boxes::r#box::Box;
            /// # use meos::collections::datetime::tstz_span::TsTzSpan;
            /// use chrono::{Utc, TimeZone, TimeDelta};
            /// let datetime1 = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
            /// let datetime2 = Utc.with_ymd_and_hms(2020, 2, 1, 0, 0, 0).unwrap();
            /// let tspan: TsTzSpan = (datetime1..datetime2).into();
            /// let tbox = TBox::from_temporal_span(tspan);
            /// let shifted_scaled_tbox = tbox.shift_scale_time(Some(TimeDelta::days(1)), Some(TimeDelta::days(2)));
            ///
            /// assert_eq!(shifted_scaled_tbox.tmin().unwrap(), datetime1 + TimeDelta::days(2));
            /// assert_eq!(shifted_scaled_tbox.tmax().unwrap(), Utc.with_ymd_and_hms(2020, 1, 7, 0, 0, 0).unwrap());
            /// ```
            fn shift_scale_time(
                &self,
                delta: Option<TimeDelta>,
                width: Option<TimeDelta>,
            ) -> TBox {
                let d = {
                    if let Some(d) = delta {
                        &*Box::new(create_interval(d)) as *const meos_sys::Interval
                    } else {
                        std::ptr::null()
                    }
                };
                let w = {
                    if let Some(w) = width {
                        &*Box::new(create_interval(w)) as *const meos_sys::Interval
                    } else {
                        std::ptr::null()
                    }
                };
                let modified = unsafe {
                    meos_sys::tbox_shift_scale_time(self._inner, d, w)
                };
                TBox::from_inner(modified)
            }
            fn union(&self, other: &Self, strict: bool) -> Option<Self> {
                let result = unsafe {
                    meos_sys::union_tbox_tbox(self.inner(), other.inner(), strict)
                };
                if result.is_null() { None } else { Some(Self::from_inner(result)) }
            }
            fn intersection(&self, other: &Self) -> Option<Self> {
                let result = unsafe {
                    meos_sys::intersection_tbox_tbox(self.inner(), other.inner())
                };
                if result.is_null() { None } else { Some(Self::from_inner(result)) }
            }
        }
        impl TBox {
            fn inner(&self) -> *const meos_sys::TBox {
                self._inner
            }
            pub fn from_inner(inner: *const meos_sys::TBox) -> Self {
                Self { _inner: inner }
            }
            /// Creates a new `TBox` instance from an integer value.
            /// Using the value as both lower and upper bounds of the value span
            ///
            /// ## Arguments
            /// * `value` - An integer to convert to `TBox`.
            ///
            /// ## Returns
            /// A new `TBox` instance representing the integer.
            ///
            /// ## Example
            /// ```
            /// # use meos::boxes::tbox::TBox;
            /// use meos::boxes::r#box::Box;
            /// let tbox = TBox::from_int(42);
            /// assert_eq!(tbox.xmin().unwrap(), 42.0);
            /// ```
            pub fn from_int(value: i32) -> Self {
                unsafe { Self::from_inner(meos_sys::int_to_tbox(value)) }
            }
            /// Creates a new `TBox` instance from an float value.
            /// Using the value as both lower and upper bounds of the value span
            ///
            /// ## Arguments
            /// * `value` - An float to convert to `TBox`.
            ///
            /// ## Returns
            /// A new `TBox` instance representing the float.
            ///
            /// ## Example
            /// ```
            /// # use meos::boxes::tbox::TBox;
            /// use meos::boxes::r#box::Box;
            /// let tbox = TBox::from_float(42.0);
            /// assert_eq!(tbox.xmin().unwrap(), 42.0);
            /// ```
            pub fn from_float(value: f64) -> Self {
                unsafe { Self::from_inner(meos_sys::float_to_tbox(value)) }
            }
            /// Creates a new `TBox` instance from an float value.
            /// Using the value as both lower and upper bounds of the value span
            ///
            /// ## Arguments
            /// * `value` - An float to convert to `TBox`.
            ///
            /// ## Returns
            /// A new `TBox` instance representing the float.
            ///
            /// ## Example
            /// ```
            /// # use meos::boxes::tbox::TBox;
            /// use meos::boxes::r#box::Box;
            /// # use meos::collections::base::span::Span;
            /// # use meos::collections::number::float_span::FloatSpan;
            ///
            /// let span: FloatSpan = (42.0..50.0).into();
            /// let tbox = TBox::from_numeric_span(span);
            /// assert_eq!(tbox.xmin().unwrap(), 42.0);
            /// assert_eq!(tbox.xmax().unwrap(), 50.0);
            /// ```
            pub fn from_numeric_span(value: impl NumberSpan) -> Self {
                unsafe { Self::from_inner(meos_sys::span_to_tbox(value.inner())) }
            }
            /// Converts the `TBox` into an `IntSpan` representing the integer span.
            ///
            /// ## Returns
            /// An `IntSpan` instance.
            ///
            /// ## Example
            /// ```
            /// # use meos::boxes::tbox::TBox;
            /// use meos::boxes::r#box::Box;
            /// # use meos::collections::number::int_span::IntSpan;
            ///
            /// let tbox = TBox::from_int(5);
            /// let intspan = tbox.intspan();
            /// assert_eq!(intspan, (5..6).into());
            /// ```
            pub fn intspan(&self) -> IntSpan {
                unsafe { IntSpan::from_inner(meos_sys::tbox_to_intspan(self._inner)) }
            }
            /// Converts the `TBox` into an `FloatSpan` representing the float span.
            ///
            /// ## Returns
            /// An `FloatSpan` instance.
            ///
            /// ## Example
            /// ```
            /// # use meos::boxes::tbox::TBox;
            /// use meos::boxes::r#box::Box;
            /// # use meos::collections::number::float_span::FloatSpan;
            ///
            /// let tbox = TBox::from_float(5.0);
            /// let floatspan = tbox.floatspan();
            /// assert_eq!(floatspan, (5.0..=5.0).into());
            /// ```
            pub fn floatspan(&self) -> FloatSpan {
                unsafe {
                    FloatSpan::from_inner(meos_sys::tbox_to_floatspan(self._inner))
                }
            }
            /// Checks if the minimum X value is inclusive in the `TBox`.
            ///
            /// ## Returns
            /// An optional `bool` indicating if the minimum X value is inclusive, or `None` if not available.
            ///
            /// ## Example
            /// ```
            /// # use meos::boxes::tbox::TBox;
            /// use meos::boxes::r#box::Box;
            /// let tbox = TBox::from_float(3.14);
            /// assert_eq!(tbox.xmin_is_inclusive(), Some(true));
            /// ```
            pub fn xmin_is_inclusive(&self) -> Option<bool> {
                unsafe {
                    let mut is_inclusive = false;
                    let ptr: *mut bool = &raw mut is_inclusive;
                    if meos_sys::tbox_xmin_inc(self.inner(), ptr) {
                        Some(is_inclusive)
                    } else {
                        None
                    }
                }
            }
            /// Checks if the maximum X value is inclusive in the `TBox`.
            ///
            /// ## Returns
            /// An optional `bool` indicating if the maximum X value is inclusive, or `None` if not available.
            ///
            /// ## Example
            /// ```
            /// # use meos::boxes::tbox::TBox;
            /// use meos::boxes::r#box::Box;
            /// let tbox = TBox::from_float(3.14);
            /// assert_eq!(tbox.xmax_is_inclusive(), Some(true)); // Assuming inclusivity is true by default
            /// ```
            pub fn xmax_is_inclusive(&self) -> Option<bool> {
                unsafe {
                    let mut is_inclusive = false;
                    let ptr: *mut bool = &raw mut is_inclusive;
                    if meos_sys::tbox_xmax_inc(self.inner(), ptr) {
                        Some(is_inclusive)
                    } else {
                        None
                    }
                }
            }
            /// Expands the `TBox` by a specified value on the X dimension.
            ///
            /// ## Arguments
            /// * `value` - The value by which to expand the X dimension.
            ///
            /// ## Returns
            /// A new `TBox` instance with expanded bounds.
            ///
            /// ## Example
            /// ```
            /// # use meos::boxes::tbox::TBox;
            /// use meos::boxes::r#box::Box;
            /// let tbox = TBox::from_float(3.0);
            /// let expanded_tbox = tbox.expand_value(2.0);
            /// assert_eq!(expanded_tbox.xmin(), Some(1.0));
            /// assert_eq!(expanded_tbox.xmax(), Some(5.0));
            /// ```
            pub fn expand_value(&self, value: f64) -> TBox {
                unsafe {
                    Self::from_inner(meos_sys::tbox_expand_float(self.inner(), value))
                }
            }
            /// Shifts and scales the X dimension of the `TBox`.
            ///
            /// ## Arguments
            /// * `delta` - The value to shift by.
            /// * `width` - The new width for the X dimension.
            ///
            /// ## Returns
            /// A new `TBox` instance with shifted and scaled bounds.
            ///
            /// ## Example
            /// ```
            /// # use meos::boxes::tbox::TBox;
            /// use meos::boxes::r#box::Box;
            /// # use std::str::FromStr;
            /// # use meos::init;
            /// # init();
            /// let tbox = TBox::from_str("TBOXFLOAT XT([0, 10),[2020-06-01, 2020-06-05])").unwrap();
            /// let shifted_scaled_tbox = tbox.shift_scale_value(Some(2.0), Some(4.0));
            ///
            /// assert_eq!(shifted_scaled_tbox.xmin(), Some(2.0)); // Example calculation
            /// assert_eq!(shifted_scaled_tbox.xmax(), Some(6.0)); // Example calculation
            /// ```
            pub fn shift_scale_value(
                &self,
                delta: Option<f64>,
                width: Option<f64>,
            ) -> TBox {
                let d = delta.unwrap_or_default();
                let w = width.unwrap_or_default();
                let modified = unsafe {
                    meos_sys::tbox_shift_scale_float(
                        self._inner,
                        d,
                        w,
                        delta.is_some(),
                        width.is_some(),
                    )
                };
                TBox::from_inner(modified)
            }
        }
        impl Collection for TBox {
            type Type = ();
            fn is_contained_in(&self, container: &Self) -> bool {
                unsafe { meos_sys::contained_tbox_tbox(self.inner(), container.inner()) }
            }
            fn overlaps(&self, other: &Self) -> bool {
                unsafe { meos_sys::overlaps_tbox_tbox(self.inner(), other.inner()) }
            }
            fn is_left(&self, other: &Self) -> bool {
                unsafe { meos_sys::left_tbox_tbox(self.inner(), other.inner()) }
            }
            fn is_over_or_left(&self, other: &Self) -> bool {
                unsafe { meos_sys::overleft_tbox_tbox(self.inner(), other.inner()) }
            }
            fn is_over_or_right(&self, other: &Self) -> bool {
                unsafe { meos_sys::overright_tbox_tbox(self.inner(), other.inner()) }
            }
            fn is_right(&self, other: &Self) -> bool {
                unsafe { meos_sys::right_tbox_tbox(self.inner(), other.inner()) }
            }
            fn is_adjacent(&self, other: &Self) -> bool {
                unsafe { meos_sys::adjacent_tbox_tbox(self.inner(), other.inner()) }
            }
            fn contains(&self, content: &Self::Type) -> bool {
                unsafe {
                    meos_sys::contains_tbox_tnumber(self.inner(), std::ptr::null())
                }
            }
        }
        impl cmp::PartialEq for TBox {
            /// Checks if two `TBox` instances are equal.
            ///
            /// # Arguments
            /// * `other` - Another `TBox` instance.
            ///
            /// ## Returns
            /// * `true` if the spans are equal, `false` otherwise.
            ///
            /// ## Example
            /// ```
            /// # use meos::boxes::tbox::TBox;
            /// use meos::boxes::r#box::Box;
            /// # use meos::init;
            /// use std::str::FromStr;
            /// # init();
            /// let span1: TBox = TBox::from_str("TBOXFLOAT XT([0, 10),[2020-06-01, 2020-06-05])").unwrap();
            /// let span2: TBox = TBox::from_str("TBOXFLOAT XT([0, 10),[2020-06-01, 2020-06-05])").unwrap();
            /// assert_eq!(span1, span2);
            /// ```
            fn eq(&self, other: &Self) -> bool {
                unsafe { meos_sys::tbox_eq(self._inner, other._inner) }
            }
        }
        impl cmp::Eq for TBox {}
        impl Debug for TBox {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let out_str = unsafe { meos_sys::tbox_out(self._inner, 3) };
                let c_str = unsafe { CStr::from_ptr(out_str) };
                let str = c_str.to_str().map_err(|_| std::fmt::Error)?;
                let result = f.write_str(str);
                unsafe { libc::free(out_str as *mut c_void) };
                result
            }
        }
        impl Clone for TBox {
            fn clone(&self) -> Self {
                unsafe { Self::from_inner(meos_sys::tbox_copy(self._inner)) }
            }
        }
        impl std::str::FromStr for TBox {
            type Err = ParseError;
            /// Parses a `TBox` from a string representation.
            ///
            /// ## Arguments
            /// * `string` - A string slice containing the representation.
            ///
            /// ## Returns
            /// * A `TBox` instance.
            ///
            /// ## Errors
            /// * Returns `ParseSpanError` if the string cannot be parsed.
            ///
            /// ## Example
            /// ```
            /// # use meos::boxes::tbox::TBox;
            /// use meos::boxes::r#box::Box;
            /// # use meos::collections::base::span::Span;
            /// # use meos::collections::number::int_span::IntSpan;
            /// # use meos::collections::datetime::tstz_span::TsTzSpan;
            /// use std::str::FromStr;
            /// # use meos::init;
            /// # init();
            ///
            /// let tbox: TBox = "TBOXINT XT([0, 10),[2020-06-01, 2020-06-05])".parse().expect("Failed to parse span");
            /// let value_span: IntSpan = tbox.intspan();
            /// let temporal_span: TsTzSpan = tbox.tstzspan();
            /// assert_eq!(value_span, (0..10).into());
            /// assert_eq!(temporal_span, TsTzSpan::from_str("[2020-06-01, 2020-06-05]").unwrap());
            /// ```
            fn from_str(string: &str) -> Result<Self, Self::Err> {
                CString::new(string)
                    .map_err(|_| ParseError)
                    .map(|string| {
                        let inner = unsafe { meos_sys::tbox_in(string.as_ptr()) };
                        Self::from_inner(inner)
                    })
            }
        }
    }
}
pub mod collections {
    pub mod base {
        pub mod collection {
            use std::{fmt::Debug, str::FromStr};
            pub trait Collection: PartialEq + Debug + FromStr + Clone {
                type Type;
                /// Returns whether `self` is contained in `container`.
                ///
                /// ## Arguments
                ///
                /// * `container` - The container to compare with.
                ///
                /// ## Returns
                ///
                /// * `true` if contained, `false` otherwise.
                fn is_contained_in(&self, container: &Self) -> bool;
                /// Determines if the collection contains the specified item.
                ///
                /// # Arguments
                ///
                /// * `content` - The item to check for containment within the collection.
                ///
                /// # Returns
                ///
                /// * `true` if the collection contains the specified item, `false` otherwise.
                fn contains(&self, content: &Self::Type) -> bool;
                /// Returns whether `self` overlaps `other`. That is, both share at least an element.
                ///
                /// ## Arguments
                ///
                /// * `other` - The object to compare with.
                ///
                /// ## Returns
                ///
                /// * `true` if overlaps, `false` otherwise.
                fn overlaps(&self, other: &Self) -> bool;
                /// Returns whether `self` is strictly before `other`. That is, `self` ends before `other` starts.
                ///
                /// ## Arguments
                ///
                /// * `other` - The object to compare with.
                ///
                /// ## Returns
                ///
                /// * `true` if before, `false` otherwise.
                fn is_left(&self, other: &Self) -> bool;
                /// Returns whether `self` is before `other` allowing overlap. That is, `self` ends before `other` ends (or at the same time).
                ///
                /// ## Arguments
                ///
                /// * `other` - The object to compare with.
                ///
                /// ## Returns
                ///
                /// * `true` if before, `false` otherwise.
                fn is_over_or_left(&self, other: &Self) -> bool;
                /// Returns whether `self` is after `other` allowing overlap. That is, `self` starts after `other` starts (or at the same time).
                ///
                /// ## Arguments
                ///
                /// * `other` - The object to compare with.
                ///
                /// ## Returns
                ///
                /// * `true` if overlapping or after, `false` otherwise.
                fn is_over_or_right(&self, other: &Self) -> bool;
                /// Returns whether `self` is strictly after `other`. That is, `self` starts after `other` ends.
                ///
                /// ## Arguments
                ///
                /// * `other` - The object to compare with.
                ///
                /// ## Returns
                ///
                /// * `true` if after, `false` otherwise.
                fn is_right(&self, other: &Self) -> bool;
                /// Returns whether `self` is adjacent to `other`. That is, `self` starts just after `other` ends.
                ///
                /// ## Arguments
                ///
                /// * `other` - The object to compare with.
                ///
                /// ## Returns
                ///
                /// * `true` if adjacent, `false` otherwise.
                fn is_adjacent(&self, other: &Self) -> bool;
            }
            pub(crate) use impl_collection;
        }
        pub mod span {
            use std::ffi::{CStr, CString};
            use crate::WKBVariant;
            use super::{collection::Collection, span_set::SpanSet};
            pub trait Span: Collection {
                /// Type used to represent subsets (duration, widths, etc.)
                type SubsetType;
                fn inner(&self) -> *const meos_sys::Span;
                /// Creates a new `Span` from a WKB representation.
                ///
                /// # Arguments
                /// * `hexwkb` - A string slice containing the WKB representation.
                ///
                /// ## Returns
                /// * A new `Span` instance.
                fn from_wkb(wkb: &[u8]) -> Self {
                    let span = unsafe {
                        meos_sys::span_from_wkb(wkb.as_ptr(), wkb.len())
                    };
                    Self::from_inner(span)
                }
                /// Creates a new `Span` from a hexadecimal WKB representation.
                ///
                /// # Arguments
                /// * `hexwkb` - A string slice containing the hexadecimal WKB representation.
                ///
                /// ## Returns
                /// * A new `Span` instance.
                fn from_hexwkb(hexwkb: &[u8]) -> Self {
                    let c_string = CString::new(hexwkb).expect("Cannot create CString");
                    let span = unsafe { meos_sys::span_from_hexwkb(c_string.as_ptr()) };
                    Self::from_inner(span)
                }
                fn from_inner(inner: *const meos_sys::Span) -> Self;
                fn as_wkb(&self, variant: WKBVariant) -> &[u8] {
                    unsafe {
                        let mut size = 0;
                        let wkb = meos_sys::span_as_wkb(
                            self.inner(),
                            variant.into(),
                            &mut size as *mut _,
                        );
                        std::slice::from_raw_parts(wkb, size)
                    }
                }
                fn as_hexwkb(&self, variant: WKBVariant) -> &[u8] {
                    unsafe {
                        let mut size: usize = 0;
                        let hexwkb_ptr = meos_sys::span_as_hexwkb(
                            self.inner(),
                            variant.into(),
                            &mut size,
                        );
                        CStr::from_ptr(hexwkb_ptr).to_bytes()
                    }
                }
                fn lower(&self) -> Self::Type;
                fn upper(&self) -> Self::Type;
                fn distance_to_value(&self, value: &Self::Type) -> Self::SubsetType;
                fn distance_to_span(&self, other: &Self) -> Self::SubsetType;
                /// Checks if the lower bound of the span is inclusive.
                ///
                /// ## Returns
                /// * `true` if the lower bound is inclusive, `false` otherwise.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::number::float_span::FloatSpan;
                /// # use meos::collections::base::span::Span;
                /// # use std::str::FromStr;
                ///
                /// let span: FloatSpan = FloatSpan::from_str("[23.9, 78.8]").unwrap();
                /// assert!(span.is_lower_inclusive());
                ///
                /// let span: FloatSpan = FloatSpan::from_str("(23.9, 78.8]").unwrap();
                /// assert!(!span.is_lower_inclusive());
                /// ```
                fn is_lower_inclusive(&self) -> bool {
                    unsafe { meos_sys::span_lower_inc(self.inner()) }
                }
                /// Checks if the upper bound of the span is inclusive.
                ///
                /// ## Returns
                /// * `true` if the upper bound is inclusive, `false` otherwise.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::number::float_span::FloatSpan;
                /// # use meos::collections::base::span::Span;
                /// # use std::str::FromStr;
                ///
                /// let span: FloatSpan = (23.9..=78.8).into();
                /// assert!(span.is_upper_inclusive());
                ///
                /// let span: FloatSpan = (23.9..78.8).into();
                /// assert!(!span.is_upper_inclusive());
                /// ```
                fn is_upper_inclusive(&self) -> bool {
                    unsafe { meos_sys::span_upper_inc(self.inner()) }
                }
                /// Return a new `Span` with the lower and upper bounds shifted by `delta`.
                fn shift(&self, delta: Self::SubsetType) -> Self;
                /// Return a new `Span` with the lower and upper bounds scaled so that the width is `width`.
                fn scale(&self, width: Self::SubsetType) -> Self;
                /// Return a new `Span` with the lower and upper bounds shifted by `delta` and scaled so that the width is `width`.
                fn shift_scale(
                    &self,
                    delta: Option<Self::SubsetType>,
                    width: Option<Self::SubsetType>,
                ) -> Self;
                fn to_spanset<T: SpanSet<Type = Self::Type>>(&self) -> T {
                    unsafe { T::from_inner(meos_sys::span_to_spanset(self.inner())) }
                }
                fn intersection(&self, other: &Self) -> Option<Self> {
                    let result = unsafe {
                        meos_sys::intersection_span_span(self.inner(), other.inner())
                    };
                    if !result.is_null() { Some(Self::from_inner(result)) } else { None }
                }
                fn union<T: SpanSet<Type = Self::Type>>(
                    &self,
                    other: &Self,
                ) -> Option<T> {
                    let result = unsafe {
                        meos_sys::union_span_span(self.inner(), other.inner())
                    };
                    if !result.is_null() { Some(T::from_inner(result)) } else { None }
                }
            }
        }
        pub mod span_set {
            use std::{
                ffi::{CStr, CString},
                ptr,
            };
            use crate::WKBVariant;
            use super::{collection::Collection, span::Span};
            pub trait SpanSet: Collection + FromIterator<Self::SpanType> {
                type SpanType: Span;
                /// Type used to represent subsets (duration, widths, etc.)
                type SubsetType;
                fn inner(&self) -> *const meos_sys::SpanSet;
                /// Creates a new `Span` from a WKB representation.
                ///
                /// ## Arguments
                /// * `hexwkb` - A string slice containing the WKB representation.
                ///
                /// ## Returns
                /// * A new `Span` instance.
                fn from_wkb(wkb: &[u8]) -> Self {
                    let span = unsafe {
                        meos_sys::spanset_from_wkb(wkb.as_ptr(), wkb.len())
                    };
                    Self::from_inner(span)
                }
                /// Creates a new `Span` from a hexadecimal WKB representation.
                ///
                /// ## Arguments
                /// * `hexwkb` - A string slice containing the hexadecimal WKB representation.
                ///
                /// ## Returns
                /// * A new `Span` instance.
                fn from_hexwkb(hexwkb: &[u8]) -> Self {
                    let c_string = CString::new(hexwkb).expect("Cannot create CString");
                    let span = unsafe {
                        meos_sys::spanset_from_hexwkb(c_string.as_ptr())
                    };
                    Self::from_inner(span)
                }
                fn copy(&self) -> Self {
                    let inner = unsafe { meos_sys::spanset_copy(self.inner()) };
                    Self::from_inner(inner)
                }
                fn from_inner(inner: *const meos_sys::SpanSet) -> Self;
                fn as_wkb(&self, variant: WKBVariant) -> &[u8] {
                    unsafe {
                        let mut size = 0;
                        let wkb = meos_sys::spanset_as_wkb(
                            self.inner(),
                            variant.into(),
                            &raw mut size,
                        );
                        std::slice::from_raw_parts(wkb, size)
                    }
                }
                fn as_hexwkb(&self, variant: WKBVariant) -> &[u8] {
                    unsafe {
                        let mut size = 0;
                        let wkb = meos_sys::spanset_as_hexwkb(
                            self.inner(),
                            variant.into(),
                            &raw mut size,
                        );
                        CStr::from_ptr(wkb).to_bytes()
                    }
                }
                fn num_spans(&self) -> i32 {
                    unsafe { meos_sys::spanset_num_spans(self.inner()) }
                }
                fn start_span(&self) -> Self::SpanType {
                    let span = unsafe { meos_sys::spanset_start_span(self.inner()) };
                    Span::from_inner(span)
                }
                fn end_span(&self) -> Self::SpanType {
                    let span = unsafe { meos_sys::spanset_end_span(self.inner()) };
                    Span::from_inner(span)
                }
                fn span_n(&self, n: i32) -> Self::SpanType {
                    let span = unsafe { meos_sys::spanset_span_n(self.inner(), n) };
                    Span::from_inner(span)
                }
                fn spans(&self) -> Vec<Self::SpanType> {
                    let spans = unsafe { meos_sys::spanset_spanarr(self.inner()) };
                    let size = self.num_spans() as usize;
                    unsafe {
                        Vec::from_raw_parts(spans, size, size)
                            .iter()
                            .map(|&span| Span::from_inner(span))
                            .collect()
                    }
                }
                fn width(&self, ignore_gaps: bool) -> Self::Type;
                /// Return a new `SpanSet` with the lower and upper bounds shifted by `delta`.
                fn shift(&self, delta: Self::SubsetType) -> Self;
                /// Return a new `SpanSet` with the lower and upper bounds scaled so that the width is `width`.
                fn scale(&self, width: Self::SubsetType) -> Self;
                /// Return a new `SpanSet` with the lower and upper bounds shifted by `delta` and scaled so that the width is `width`.
                fn shift_scale(
                    &self,
                    delta: Option<Self::SubsetType>,
                    width: Option<Self::SubsetType>,
                ) -> Self;
                fn intersection(&self, other: &Self) -> Option<Self> {
                    let result = unsafe {
                        meos_sys::intersection_spanset_spanset(
                            self.inner(),
                            other.inner(),
                        )
                    };
                    if !result.is_null() { Some(Self::from_inner(result)) } else { None }
                }
                fn union(&self, other: &Self) -> Option<Self> {
                    let result = unsafe {
                        meos_sys::union_spanset_spanset(self.inner(), other.inner())
                    };
                    if !result.is_null() { Some(Self::from_inner(result)) } else { None }
                }
                fn hash(&self) -> u32 {
                    unsafe { meos_sys::spanset_hash(self.inner()) }
                }
                fn distance_to_value(&self, value: &Self::Type) -> Self::SubsetType;
                fn distance_to_span_set(&self, other: &Self) -> Self::SubsetType;
                fn distance_to_span(&self, span: &Self::SpanType) -> Self::SubsetType;
            }
            pub(crate) use impl_iterator;
        }
    }
    pub mod datetime {
        use chrono::Days;
        pub mod date_span {
            use std::{
                cmp, ffi::{c_void, CStr, CString},
                fmt::Debug, hash::Hash, ops::{BitAnd, Range, RangeInclusive},
            };
            use chrono::{Datelike, NaiveDate, TimeDelta};
            use collection::{impl_collection, Collection};
            use span::Span;
            use crate::{
                collections::{base::*, datetime::DAYS_UNTIL_2000},
                errors::ParseError, utils::from_interval,
            };
            pub struct DateSpan {
                _inner: *const meos_sys::Span,
            }
            impl Drop for DateSpan {
                fn drop(&mut self) {
                    unsafe {
                        libc::free(self._inner as *mut c_void);
                    }
                }
            }
            impl Collection for DateSpan {
                type Type = NaiveDate;
                fn is_contained_in(&self, container: &Self) -> bool {
                    unsafe {
                        meos_sys::contained_span_span(self.inner(), container.inner())
                    }
                }
                fn overlaps(&self, other: &Self) -> bool {
                    unsafe { meos_sys::overlaps_span_span(self.inner(), other.inner()) }
                }
                fn is_left(&self, other: &Self) -> bool {
                    unsafe { meos_sys::left_span_span(self.inner(), other.inner()) }
                }
                fn is_over_or_left(&self, other: &Self) -> bool {
                    unsafe { meos_sys::overleft_span_span(self.inner(), other.inner()) }
                }
                fn is_over_or_right(&self, other: &Self) -> bool {
                    unsafe { meos_sys::overright_span_span(self.inner(), other.inner()) }
                }
                fn is_right(&self, other: &Self) -> bool {
                    unsafe { meos_sys::right_span_span(self.inner(), other.inner()) }
                }
                fn is_adjacent(&self, other: &Self) -> bool {
                    unsafe { meos_sys::adjacent_span_span(self.inner(), other.inner()) }
                }
                fn contains(&self, content: &NaiveDate) -> bool {
                    unsafe {
                        meos_sys::contains_span_date(
                            self.inner(),
                            content.num_days_from_ce(),
                        )
                    }
                }
            }
            impl span::Span for DateSpan {
                type SubsetType = TimeDelta;
                fn inner(&self) -> *const meos_sys::Span {
                    self._inner
                }
                /// Creates a new `DateSpan` from an inner podateer to a `meos_sys::Span`.
                ///
                /// # Arguments
                /// * `inner` - A podateer to the inner `meos_sys::Span`.
                ///
                /// ## Returns
                /// * A new `DateSpan` instance.
                fn from_inner(inner: *const meos_sys::Span) -> Self {
                    Self { _inner: inner }
                }
                /// Returns the lower bound of the span.
                ///
                /// ## Returns
                /// * The lower bound as a `NaiveDate`.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::datetime::date_span::DateSpan;
                /// # use meos::collections::base::span::Span;
                /// # use chrono::naive::NaiveDate;
                ///
                /// let from_ymd_opt = |y, m, d| NaiveDate::from_ymd_opt(y, m, d).unwrap();
                ///
                /// let span: DateSpan = (from_ymd_opt(2023, 1, 1)..from_ymd_opt(2023, 1, 15)).into();
                /// let lower = span.lower();
                /// assert_eq!(lower, from_ymd_opt(2023, 1, 1));
                /// ```
                fn lower(&self) -> Self::Type {
                    let num_of_days = unsafe { meos_sys::datespan_lower(self.inner()) };
                    NaiveDate::from_num_days_from_ce_opt(num_of_days)
                        .expect("Wrong date returned from meos")
                        .checked_add_days(DAYS_UNTIL_2000)
                        .unwrap()
                }
                /// Returns the upper bound of the span.
                ///
                /// ## Returns
                /// * The upper bound as a `NaiveDate`.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::datetime::date_span::DateSpan;
                /// # use meos::collections::base::span::Span;
                /// # use chrono::naive::NaiveDate;
                ///
                /// let from_ymd_opt = |y, m, d| NaiveDate::from_ymd_opt(y, m, d).unwrap();
                ///
                /// let span: DateSpan = (from_ymd_opt(2023, 1, 1)..from_ymd_opt(2023, 1, 15)).into();
                /// let upper = span.upper();
                /// assert_eq!(upper, from_ymd_opt(2023, 1, 15));
                /// ```
                fn upper(&self) -> Self::Type {
                    let num_of_days = unsafe { meos_sys::datespan_upper(self.inner()) };
                    NaiveDate::from_num_days_from_ce_opt(num_of_days)
                        .expect("Wrong date returned from meos")
                        .checked_add_days(DAYS_UNTIL_2000)
                        .unwrap()
                }
                /// Return a new `DateSpan` with the lower and upper bounds shifted by `delta`.
                ///
                /// # Arguments
                /// * `delta` - The value to shift by, as a `NaiveDate`.
                ///
                /// # Returns
                /// A new `DateSpan` instance.
                ///
                /// # Example
                /// ```
                /// # use meos::collections::datetime::date_span::DateSpan;
                /// # use meos::collections::base::span::Span;
                /// use chrono::naive::NaiveDate;
                /// use chrono::TimeDelta;
                ///
                /// let from_ymd_opt = |y, m, d| NaiveDate::from_ymd_opt(y, m, d).unwrap();
                ///
                /// let span: DateSpan = (from_ymd_opt(2023, 1, 1)..from_ymd_opt(2023, 1, 15)).into();
                /// let shifted_span = span.shift(TimeDelta::days(5));
                /// let expected_span: DateSpan = (from_ymd_opt(2023, 1, 6)..from_ymd_opt(2023, 1, 20)).into();
                /// assert_eq!(shifted_span, expected_span);
                /// ```
                fn shift(&self, delta: TimeDelta) -> DateSpan {
                    self.shift_scale(Some(delta), None)
                }
                /// Return a new `DateSpan` with the lower and upper bounds scaled so that the width is `width`.
                ///
                /// # Arguments
                /// * `width` - The new width, as a `NaiveDate`.
                ///
                /// # Returns
                /// A new `DateSpan` instance.
                ///
                /// # Example
                /// ```
                /// # use meos::collections::datetime::date_span::DateSpan;
                /// # use meos::collections::base::span::Span;
                /// use chrono::naive::NaiveDate;
                /// use chrono::TimeDelta;
                ///
                /// let from_ymd_opt = |y, m, d| NaiveDate::from_ymd_opt(y, m, d).unwrap();
                ///
                /// let span: DateSpan = (from_ymd_opt(2023, 1, 1)..from_ymd_opt(2023, 1, 15)).into();
                /// let scaled_span = span.scale(TimeDelta::days(5));
                /// let expected_span: DateSpan = (from_ymd_opt(2023, 1, 1)..from_ymd_opt(2023, 1, 07)).into();
                /// assert_eq!(scaled_span, expected_span);
                /// ```
                fn scale(&self, width: TimeDelta) -> DateSpan {
                    self.shift_scale(None, Some(width))
                }
                /// Return a new `DateSpan` with the lower and upper bounds shifted by `delta` and scaled so that the width is `width`.
                ///
                /// # Arguments
                /// * `delta` - The value to shift by, as a `NaiveDate`.
                /// * `width` - The new width, as a `NaiveDate`.
                ///
                /// # Returns
                /// A new `DateSpan` instance.
                ///
                /// # Example
                /// ```
                /// # use meos::collections::datetime::date_span::DateSpan;
                /// # use meos::collections::base::span::Span;
                /// use chrono::naive::NaiveDate;
                /// use chrono::TimeDelta;
                ///
                /// let from_ymd_opt = |y, m, d| NaiveDate::from_ymd_opt(y, m, d).unwrap();
                ///
                /// let span: DateSpan = (from_ymd_opt(2023, 1, 1)..from_ymd_opt(2023, 1, 15)).into();
                /// let shifted_scaled_span = span.shift_scale(Some(TimeDelta::days(5)), Some(TimeDelta::days(10)));
                /// let expected_span: DateSpan = (from_ymd_opt(2023, 1, 6)..from_ymd_opt(2023, 1, 17)).into();
                /// assert_eq!(shifted_scaled_span, expected_span);
                /// ```
                fn shift_scale(
                    &self,
                    delta: Option<TimeDelta>,
                    width: Option<TimeDelta>,
                ) -> DateSpan {
                    let d = delta
                        .unwrap_or_default()
                        .num_days()
                        .try_into()
                        .expect("Number too big");
                    let w = width
                        .unwrap_or_default()
                        .num_days()
                        .try_into()
                        .expect("Number too big");
                    let modified = unsafe {
                        meos_sys::datespan_shift_scale(
                            self._inner,
                            d,
                            w,
                            delta.is_some(),
                            width.is_some(),
                        )
                    };
                    DateSpan::from_inner(modified)
                }
                /// Calculates the distance between this `DateSpan` and a specific timestamp (`value`).
                ///
                /// ## Arguments
                /// * `value` - Anvalue `DateSpan` to calculate the distance to.
                ///
                /// ## Returns
                /// A `TimeDelta` representing the distance in seconds between the two spans.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::datetime::date_span::DateSpan;
                /// # use meos::init;
                /// use std::str::FromStr;
                /// # use meos::collections::base::span::Span;
                /// use chrono::TimeDelta;
                /// # init();
                /// let span1 = DateSpan::from_str("[2019-09-08, 2019-09-10]").unwrap();
                /// let span2 = DateSpan::from_str("[2019-09-12, 2019-09-14]").unwrap();
                /// let distance = span1.distance_to_span(&span2);
                /// assert_eq!(distance, TimeDelta::days(2));
                /// ```
                fn distance_to_value(&self, other: &Self::Type) -> TimeDelta {
                    unsafe {
                        TimeDelta::days(
                            meos_sys::distance_span_date(
                                    self.inner(),
                                    other
                                        .checked_sub_days(DAYS_UNTIL_2000)
                                        .unwrap()
                                        .num_days_from_ce(),
                                )
                                .into(),
                        )
                    }
                }
                /// Calculates the distance between this `DateSpan` and another `DateSpan`.
                ///
                /// ## Arguments
                /// * `other` - Another `DateSpan` to calculate the distance to.
                ///
                /// ## Returns
                /// A `TimeDelta` representing the distance in seconds between the two spans.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::datetime::date_span::DateSpan;
                /// # use meos::collections::base::span::Span;
                /// # use chrono::{TimeDelta, TimeZone, Utc};
                /// # use meos::init;
                /// use std::str::FromStr;
                /// # init();
                /// let span_set1 = DateSpan::from_str("[2019-09-08, 2019-09-10]").unwrap();
                /// let span_set2 = DateSpan::from_str("[2018-08-07, 2018-08-17]").unwrap();
                /// let distance = span_set1.distance_to_span(&span_set2);
                /// assert_eq!(distance, TimeDelta::days(387));
                /// ```
                fn distance_to_span(&self, other: &Self) -> TimeDelta {
                    unsafe {
                        TimeDelta::days(
                            meos_sys::distance_datespan_datespan(
                                    self.inner(),
                                    other.inner(),
                                )
                                .into(),
                        )
                    }
                }
            }
            impl DateSpan {
                pub fn duration(&self) -> TimeDelta {
                    from_interval(unsafe {
                        meos_sys::datespan_duration(self._inner).read()
                    })
                }
            }
            impl Clone for DateSpan {
                fn clone(&self) -> Self {
                    unsafe { Self::from_inner(meos_sys::span_copy(self._inner)) }
                }
            }
            impl Hash for DateSpan {
                fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                    let hash = unsafe { meos_sys::span_hash(self._inner) };
                    state.write_u32(hash);
                    state.finish();
                }
            }
            impl std::str::FromStr for DateSpan {
                type Err = ParseError;
                /// Parses a `DateSpan` from a string representation.
                ///
                /// ## Arguments
                /// * `string` - A string slice containing the representation.
                ///
                /// ## Returns
                /// * A `DateSpan` instance.
                ///
                /// ## Errors
                /// * Returns `ParseSpanError` if the string cannot be parsed.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::datetime::date_span::DateSpan;
                /// # use meos::collections::base::span::Span;
                /// # use std::str::FromStr;
                /// # use meos::init;
                /// use chrono::NaiveDate;
                /// # init();
                /// let from_ymd_opt = |y, m, d| NaiveDate::from_ymd_opt(y, m, d).unwrap();
                ///
                /// let span: DateSpan = "(2019-09-08, 2019-09-10)".parse().expect("Failed to parse span");
                /// assert_eq!(span.lower(), from_ymd_opt(2019, 9, 9));
                /// assert_eq!(span.upper(), from_ymd_opt(2019, 9, 10));
                /// ```
                fn from_str(string: &str) -> Result<Self, Self::Err> {
                    CString::new(string)
                        .map_err(|_| ParseError)
                        .map(|string| {
                            let inner = unsafe {
                                meos_sys::datespan_in(string.as_ptr())
                            };
                            Self::from_inner(inner)
                        })
                }
            }
            impl cmp::PartialEq for DateSpan {
                /// Checks if two `DateSpan` instances are equal.
                ///
                /// # Arguments
                /// * `other` - Another `DateSpan` instance.
                ///
                /// ## Returns
                /// * `true` if the spans are equal, `false` otherwise.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::datetime::date_span::DateSpan;
                /// # use meos::collections::base::span::Span;
                /// use chrono::naive::NaiveDate;
                ///
                /// let from_ymd_opt = |y, m, d| NaiveDate::from_ymd_opt(y, m, d).unwrap();
                ///
                /// let span1: DateSpan = (from_ymd_opt(1, 1, 1)..from_ymd_opt(2, 2, 2)).into();
                /// let span2: DateSpan = (from_ymd_opt(1, 1, 1)..from_ymd_opt(2, 2, 2)).into();
                /// assert_eq!(span1, span2);
                /// ```
                fn eq(&self, other: &Self) -> bool {
                    unsafe { meos_sys::span_eq(self._inner, other._inner) }
                }
            }
            impl cmp::Eq for DateSpan {}
            impl From<Range<NaiveDate>> for DateSpan {
                fn from(Range { start, end }: Range<NaiveDate>) -> Self {
                    let inner = unsafe {
                        meos_sys::datespan_make(
                            start
                                .checked_sub_days(DAYS_UNTIL_2000)
                                .unwrap()
                                .num_days_from_ce(),
                            end
                                .checked_sub_days(DAYS_UNTIL_2000)
                                .unwrap()
                                .num_days_from_ce(),
                            true,
                            false,
                        )
                    };
                    Self::from_inner(inner)
                }
            }
            impl From<RangeInclusive<NaiveDate>> for DateSpan {
                fn from(range: RangeInclusive<NaiveDate>) -> Self {
                    let inner = unsafe {
                        meos_sys::datespan_make(
                            range
                                .start()
                                .checked_sub_days(DAYS_UNTIL_2000)
                                .unwrap()
                                .num_days_from_ce(),
                            range
                                .end()
                                .checked_sub_days(DAYS_UNTIL_2000)
                                .unwrap()
                                .num_days_from_ce(),
                            true,
                            true,
                        )
                    };
                    Self::from_inner(inner)
                }
            }
            impl Debug for DateSpan {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let out_str = unsafe { meos_sys::datespan_out(self._inner) };
                    let c_str = unsafe { CStr::from_ptr(out_str) };
                    let str = c_str.to_str().map_err(|_| std::fmt::Error)?;
                    let result = f.write_str(str);
                    unsafe { libc::free(out_str as *mut c_void) };
                    result
                }
            }
            impl BitAnd for DateSpan {
                type Output = Option<DateSpan>;
                /// Computes the dateersection of two `DateSpan` instances.
                ///
                /// # Arguments
                /// * `other` - Another `DateSpan` instance.
                ///
                /// ## Returns
                /// * An `Option<DateSpan>` containing the dateersection, or `None` if there is no dateersection.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::datetime::date_span::DateSpan;
                /// # use meos::collections::base::span::Span;
                /// # use std::str::FromStr;
                /// use chrono::naive::NaiveDate;
                ///
                /// let from_ymd_opt = |y, m, d| NaiveDate::from_ymd_opt(y, m, d).unwrap();
                ///
                /// let span1: DateSpan = (from_ymd_opt(1, 1, 1)..from_ymd_opt(1, 1, 11)).into();
                /// let span2: DateSpan = (from_ymd_opt(1, 1, 9)..from_ymd_opt(2, 1, 11)).into();
                /// let date_intersection = (span1 & span2).unwrap();
                ///
                /// assert_eq!(date_intersection, (from_ymd_opt(1, 1, 9)..from_ymd_opt(1, 1, 11)).into())
                /// ```
                fn bitand(self, other: Self) -> Self::Output {
                    let result = unsafe {
                        meos_sys::intersection_span_span(self._inner, other._inner)
                    };
                    if !result.is_null() {
                        Some(DateSpan::from_inner(result))
                    } else {
                        None
                    }
                }
            }
            impl PartialOrd for DateSpan {
                fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
                    let cmp = unsafe { meos_sys::span_cmp(self._inner, other._inner) };
                    match cmp {
                        -1 => Some(cmp::Ordering::Less),
                        0 => Some(cmp::Ordering::Equal),
                        1 => Some(cmp::Ordering::Greater),
                        _ => None,
                    }
                }
            }
            impl Ord for DateSpan {
                fn cmp(&self, other: &Self) -> cmp::Ordering {
                    self.partial_cmp(other)
                        .expect(
                            "Unreachable since for non-null and same types spans, we only return -1, 0, or 1",
                        )
                }
            }
        }
        pub mod date_span_set {
            use std::ffi::{c_void, CStr, CString};
            use chrono::Datelike;
            use chrono::NaiveDate;
            use chrono::TimeDelta;
            use collection::{impl_collection, Collection};
            use span::Span;
            use span_set::impl_iterator;
            use std::fmt::Debug;
            use std::hash::Hash;
            use std::ops::{BitAnd, BitOr};
            use crate::collections::base::span_set::SpanSet;
            use crate::collections::base::*;
            use crate::errors::ParseError;
            use super::date_span::DateSpan;
            use super::DAYS_UNTIL_2000;
            pub struct DateSpanSet {
                _inner: *const meos_sys::SpanSet,
            }
            impl Drop for DateSpanSet {
                fn drop(&mut self) {
                    unsafe {
                        libc::free(self._inner as *mut c_void);
                    }
                }
            }
            impl Collection for DateSpanSet {
                type Type = NaiveDate;
                fn is_contained_in(&self, container: &Self) -> bool {
                    unsafe {
                        meos_sys::contained_spanset_spanset(
                            self.inner(),
                            container.inner(),
                        )
                    }
                }
                fn overlaps(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::overlaps_spanset_spanset(self.inner(), other.inner())
                    }
                }
                fn is_left(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::left_spanset_spanset(self.inner(), other.inner())
                    }
                }
                fn is_over_or_left(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::overleft_spanset_spanset(self.inner(), other.inner())
                    }
                }
                fn is_over_or_right(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::overright_spanset_spanset(self.inner(), other.inner())
                    }
                }
                fn is_right(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::right_spanset_spanset(self.inner(), other.inner())
                    }
                }
                fn is_adjacent(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::adjacent_spanset_spanset(self.inner(), other.inner())
                    }
                }
                fn contains(&self, content: &NaiveDate) -> bool {
                    unsafe {
                        meos_sys::contains_spanset_date(
                            self.inner(),
                            content.num_days_from_ce(),
                        )
                    }
                }
            }
            impl span_set::SpanSet for DateSpanSet {
                type SpanType = DateSpan;
                type SubsetType = TimeDelta;
                fn inner(&self) -> *const meos_sys::SpanSet {
                    self._inner
                }
                fn from_inner(inner: *const meos_sys::SpanSet) -> Self
                where
                    Self: Sized,
                {
                    Self { _inner: inner }
                }
                fn width(&self, _ignore_gaps: bool) -> Self::Type {
                    {
                        ::core::panicking::panic_fmt(
                            format_args!(
                                "not implemented: {0}",
                                format_args!("Not implemented for date"),
                            ),
                        );
                    }
                }
                /// Return a new `DateSpanSet` with the lower and upper bounds shifted by `delta`.
                ///
                /// ## Arguments
                /// * `delta` - The value to shift by.
                ///
                /// ## Returns
                /// A new `DateSpanSet` instance.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::datetime::date_span_set::DateSpanSet;
                /// # use meos::init;
                /// # use std::str::FromStr;
                /// # use meos::collections::base::span_set::SpanSet;
                /// use chrono::TimeDelta;
                /// # init();
                /// let span_set = DateSpanSet::from_str("{[2019-09-08, 2019-09-10], [2019-09-16, 2019-09-20]}").unwrap();
                /// let shifted_span_set = span_set.shift(TimeDelta::days(5));
                ///
                /// let expected_shifted_span_set =
                ///     DateSpanSet::from_str("{[2019-09-13, 2019-09-16), [2019-09-21, 2019-09-26)}").unwrap();
                /// assert_eq!(shifted_span_set, expected_shifted_span_set);
                /// ```
                fn shift(&self, delta: TimeDelta) -> DateSpanSet {
                    self.shift_scale(Some(delta), None)
                }
                /// Return a new `DateSpanSet` with the lower and upper bounds scaled so that the width is `width`.
                ///
                /// ## Arguments
                /// * `width` - The new width.
                ///
                /// ## Returns
                /// A new `DateSpanSet` instance.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::datetime::date_span_set::DateSpanSet;
                /// # use meos::init;
                /// # use std::str::FromStr;
                /// # use meos::collections::base::span_set::SpanSet;
                /// use chrono::TimeDelta;
                /// # init();
                /// let span_set = DateSpanSet::from_str("{[2019-09-08, 2019-09-10], [2019-09-13, 2019-09-15]}").unwrap();
                /// let scaled_span_set = span_set.scale(TimeDelta::days(5));
                ///
                /// let expected_scaled_span_set =
                ///     DateSpanSet::from_str("{[2019-09-08, 2019-09-10), [2019-09-11, 2019-09-14)}").unwrap();
                /// assert_eq!(scaled_span_set, expected_scaled_span_set);
                /// ```
                fn scale(&self, width: TimeDelta) -> DateSpanSet {
                    self.shift_scale(None, Some(width))
                }
                /// Return a new `DateSpanSet` with the lower and upper bounds shifted by `delta` and scaled so that the width is `width`.
                ///
                /// ## Arguments
                /// * `delta` - The value to shift by.
                /// * `width` - The new width.
                ///
                /// ## Returns
                /// A new `DateSpanSet` instance.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::datetime::date_span_set::DateSpanSet;
                /// # use meos::init;
                /// # use std::str::FromStr;
                /// # use meos::collections::base::span_set::SpanSet;
                /// use chrono::TimeDelta;
                /// # init();
                /// let span_set = DateSpanSet::from_str("{[2019-09-08, 2019-09-10], [2019-09-11, 2019-09-12]}").unwrap();
                /// let shifted_scaled_span_set = span_set.shift_scale(Some(TimeDelta::days(5)), Some(TimeDelta::days(10)));
                ///
                /// let expected_shifted_scaled_span_set =
                ///     DateSpanSet::from_str("{[2019-09-13, 2019-09-24)}").unwrap();
                /// assert_eq!(shifted_scaled_span_set, expected_shifted_scaled_span_set);
                /// ```
                fn shift_scale(
                    &self,
                    delta: Option<TimeDelta>,
                    width: Option<TimeDelta>,
                ) -> DateSpanSet {
                    let d = delta
                        .unwrap_or_default()
                        .num_days()
                        .try_into()
                        .expect("Number too big");
                    let w = width
                        .unwrap_or_default()
                        .num_days()
                        .try_into()
                        .expect("Number too big");
                    let modified = unsafe {
                        meos_sys::datespanset_shift_scale(
                            self._inner,
                            d,
                            w,
                            delta.is_some(),
                            width.is_some(),
                        )
                    };
                    DateSpanSet::from_inner(modified)
                }
                /// Calculates the distance between this `DateSpanSet` and a specific timestamp (`value`).
                ///
                /// ## Arguments
                /// * `value` - A timestamp represented by `TimeDelta` from the Unix epoch.
                ///
                /// ## Returns
                /// A `TimeDelta` representing the distance in seconds between this `DateSpanSet` and the given timestamp.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::datetime::date_span_set::DateSpanSet;
                /// # use meos::collections::base::span_set::SpanSet;
                /// # use chrono::{TimeDelta, TimeZone, NaiveDate};
                /// # use meos::init;
                /// use std::str::FromStr;
                /// # init();
                /// let span_set = DateSpanSet::from_str("{[2019-09-08, 2019-09-10], [2019-09-11, 2019-09-12]}").unwrap();
                /// let timestamp = NaiveDate::from_ymd_opt(2019, 9, 5).unwrap();
                /// let distance = span_set.distance_to_value(&timestamp);
                /// assert_eq!(distance, TimeDelta::days(3));
                /// ```
                fn distance_to_value(&self, other: &Self::Type) -> TimeDelta {
                    unsafe {
                        TimeDelta::days(
                            meos_sys::distance_spanset_date(
                                    self.inner(),
                                    other
                                        .checked_sub_days(DAYS_UNTIL_2000)
                                        .unwrap()
                                        .num_days_from_ce(),
                                )
                                .into(),
                        )
                    }
                }
                /// Calculates the distance between this `DateSpanSet` and another `DateSpanSet`.
                ///
                /// ## Arguments
                /// * `other` - Another `DateSpanSet` to calculate the distance to.
                ///
                /// ## Returns
                /// A `TimeDelta` representing the distance in seconds between the two span sets.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::datetime::date_span_set::DateSpanSet;
                /// # use meos::collections::base::span_set::SpanSet;
                /// # use chrono::{TimeDelta, TimeZone, Utc};
                /// # use meos::init;
                /// use std::str::FromStr;
                /// # init();
                /// let span_set1 = DateSpanSet::from_str("{[2019-09-08, 2019-09-10], [2019-09-11, 2019-09-12]}").unwrap();
                /// let span_set2 = DateSpanSet::from_str("{[2018-08-07, 2018-08-17], [2018-10-17, 2018-10-20]}").unwrap();
                /// let distance = span_set1.distance_to_span_set(&span_set2);
                /// assert_eq!(distance, TimeDelta::days(323));
                /// ```
                fn distance_to_span_set(&self, other: &Self) -> TimeDelta {
                    unsafe {
                        TimeDelta::days(
                            meos_sys::distance_datespanset_datespanset(
                                    self.inner(),
                                    other.inner(),
                                )
                                .into(),
                        )
                    }
                }
                /// Calculates the distance between this `DateSpanSet` and a `DateSpan`.
                ///
                /// ## Arguments
                /// * `other` - A `DateSpan` to calculate the distance to.
                ///
                /// ## Returns
                /// A `TimeDelta` representing the distance in seconds between the span set and the span.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::datetime::date_span_set::DateSpanSet;
                /// # use meos::collections::base::span_set::SpanSet;
                /// # use meos::collections::datetime::date_span::DateSpan;
                /// # use meos::collections::base::span::Span;
                /// # use chrono::{TimeDelta, TimeZone, Utc};
                /// # use meos::init;
                /// use std::str::FromStr;
                /// # init();
                /// let span_set = DateSpanSet::from_str("{[2019-09-08, 2019-09-10], [2019-09-11, 2019-09-12]}").unwrap();
                /// let span = DateSpan::from_str("[2018-08-07, 2018-08-17]").unwrap();
                /// let distance = span_set.distance_to_span(&span);
                /// assert_eq!(distance, TimeDelta::days(387));
                /// ```
                fn distance_to_span(&self, span: &Self::SpanType) -> TimeDelta {
                    unsafe {
                        TimeDelta::days(
                            meos_sys::distance_datespanset_datespan(
                                    self.inner(),
                                    span.inner(),
                                )
                                .into(),
                        )
                    }
                }
            }
            impl Clone for DateSpanSet {
                fn clone(&self) -> DateSpanSet {
                    self.copy()
                }
            }
            impl IntoIterator for DateSpanSet {
                type Item = <DateSpanSet as SpanSet>::SpanType;
                type IntoIter = std::vec::IntoIter<Self::Item>;
                fn into_iter(self) -> Self::IntoIter {
                    self.spans().into_iter()
                }
            }
            impl FromIterator<<DateSpanSet as SpanSet>::SpanType> for DateSpanSet {
                fn from_iter<T: IntoIterator<Item = <DateSpanSet as SpanSet>::SpanType>>(
                    iter: T,
                ) -> Self {
                    iter.into_iter().collect()
                }
            }
            impl<'a> FromIterator<&'a <DateSpanSet as SpanSet>::SpanType>
            for DateSpanSet {
                fn from_iter<
                    T: IntoIterator<Item = &'a <DateSpanSet as SpanSet>::SpanType>,
                >(iter: T) -> Self {
                    let mut iter = iter.into_iter();
                    let first = iter.next().unwrap();
                    iter.fold(
                        first.to_spanset(),
                        |acc, item| { (acc | item.to_spanset()).unwrap() },
                    )
                }
            }
            impl Hash for DateSpanSet {
                fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                    let hash = unsafe { meos_sys::spanset_hash(self._inner) };
                    state.write_u32(hash);
                    state.finish();
                }
            }
            impl std::str::FromStr for DateSpanSet {
                type Err = ParseError;
                fn from_str(string: &str) -> Result<Self, Self::Err> {
                    CString::new(string)
                        .map_err(|_| ParseError)
                        .map(|string| {
                            let inner = unsafe {
                                meos_sys::datespanset_in(string.as_ptr())
                            };
                            Self::from_inner(inner)
                        })
                }
            }
            impl std::cmp::PartialEq for DateSpanSet {
                fn eq(&self, other: &Self) -> bool {
                    unsafe { meos_sys::spanset_eq(self._inner, other._inner) }
                }
            }
            impl Debug for DateSpanSet {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let out_str = unsafe { meos_sys::datespanset_out(self._inner) };
                    let c_str = unsafe { CStr::from_ptr(out_str) };
                    let str = c_str.to_str().map_err(|_| std::fmt::Error)?;
                    let result = f.write_str(str);
                    unsafe { libc::free(out_str as *mut c_void) };
                    result
                }
            }
            impl BitAnd<DateSpanSet> for DateSpanSet {
                type Output = Option<DateSpanSet>;
                /// Computes the dateersection of two `DateSpanSet`s.
                ///
                /// ## Arguments
                ///
                /// * `other` - Another `DateSpanSet` to dateersect with.
                ///
                /// ## Returns
                ///
                /// * `Some(DateSpanSet)` - A new `DateSpanSet` containing the dateersection, if it exists.
                /// * `None` - If the dateersection is empty.
                ///
                /// ## Example
                ///
                /// ```
                /// # use meos::collections::datetime::date_span_set::DateSpanSet;
                /// # use meos::init;
                /// # use std::str::FromStr;
                /// # use meos::collections::base::span_set::SpanSet;
                /// # init();
                /// let span_set1 = DateSpanSet::from_str("{[2019-09-08, 2019-09-10], [2019-09-15, 2019-09-20]}").unwrap();
                /// let span_set2 = DateSpanSet::from_str("{[2019-09-15, 2019-09-30], [2019-11-11, 2019-11-12]}").unwrap();
                ///
                /// let expected_result = DateSpanSet::from_str("{[2019-09-15, 2019-09-21)}").unwrap();
                /// assert_eq!((span_set1 & span_set2).unwrap(), expected_result);
                /// ```
                fn bitand(self, other: DateSpanSet) -> Self::Output {
                    self.intersection(&other)
                }
            }
            impl BitOr for DateSpanSet {
                type Output = Option<DateSpanSet>;
                /// Computes the union of two `DateSpanSet`s.
                ///
                /// ## Arguments
                ///
                /// * `other` - Another `DateSpanSet` to union with.
                ///
                /// ## Returns
                ///
                /// * `Some(DateSpanSet)` - A new `DateSpanSet` containing the union.
                /// * `None` - If the union is empty.
                ///
                /// ## Example
                ///
                /// ```
                /// # use meos::collections::datetime::date_span_set::DateSpanSet;
                /// # use meos::init;
                /// # use std::str::FromStr;
                /// # use meos::collections::base::span_set::SpanSet;
                /// # init();
                /// let span_set1 = DateSpanSet::from_str("{[2019-09-08, 2019-09-10], [2019-09-15, 2019-09-20]}").unwrap();
                /// let span_set2 = DateSpanSet::from_str("{[2019-09-15, 2019-09-30], [2019-11-11, 2019-11-12]}").unwrap();
                ///
                /// let expected_result = DateSpanSet::from_str("{[2019-09-08, 2019-09-11), [2019-09-15, 2019-10-01), [2019-11-11, 2019-11-13)}").unwrap();
                /// assert_eq!((span_set1 | span_set2).unwrap(), expected_result)
                /// ```
                fn bitor(self, other: Self) -> Self::Output {
                    self.union(&other)
                }
            }
        }
        pub mod tstz_span {
            use std::{
                cmp, ffi::{c_void, CStr, CString},
                fmt::Debug, hash::Hash, ops::{BitAnd, Range, RangeInclusive},
            };
            use chrono::{DateTime, Datelike, TimeDelta, TimeZone, Utc};
            use collection::{impl_collection, Collection};
            use span::Span;
            use crate::{
                collections::base::*, errors::ParseError,
                utils::{
                    create_interval, from_interval, from_meos_timestamp,
                    to_meos_timestamp,
                },
                BoundingBox,
            };
            pub struct TsTzSpan {
                _inner: *const meos_sys::Span,
            }
            impl Drop for TsTzSpan {
                fn drop(&mut self) {
                    unsafe {
                        libc::free(self._inner as *mut c_void);
                    }
                }
            }
            impl Collection for TsTzSpan {
                type Type = DateTime<Utc>;
                fn is_contained_in(&self, container: &Self) -> bool {
                    unsafe {
                        meos_sys::contained_span_span(self.inner(), container.inner())
                    }
                }
                fn overlaps(&self, other: &Self) -> bool {
                    unsafe { meos_sys::overlaps_span_span(self.inner(), other.inner()) }
                }
                fn is_left(&self, other: &Self) -> bool {
                    unsafe { meos_sys::left_span_span(self.inner(), other.inner()) }
                }
                fn is_over_or_left(&self, other: &Self) -> bool {
                    unsafe { meos_sys::overleft_span_span(self.inner(), other.inner()) }
                }
                fn is_over_or_right(&self, other: &Self) -> bool {
                    unsafe { meos_sys::overright_span_span(self.inner(), other.inner()) }
                }
                fn is_right(&self, other: &Self) -> bool {
                    unsafe { meos_sys::right_span_span(self.inner(), other.inner()) }
                }
                fn is_adjacent(&self, other: &Self) -> bool {
                    unsafe { meos_sys::adjacent_span_span(self.inner(), other.inner()) }
                }
                fn contains(&self, content: &DateTime<Utc>) -> bool {
                    unsafe {
                        meos_sys::contains_span_date(
                            self.inner(),
                            content.num_days_from_ce(),
                        )
                    }
                }
            }
            impl span::Span for TsTzSpan {
                type SubsetType = TimeDelta;
                fn inner(&self) -> *const meos_sys::Span {
                    self._inner
                }
                /// Creates a new `TsTzSpan` from an inner podateer to a `meos_sys::Span`.
                ///
                /// # Arguments
                /// * `inner` - A podateer to the inner `meos_sys::Span`.
                ///
                /// ## Returns
                /// * A new `TsTzSpan` instance.
                fn from_inner(inner: *const meos_sys::Span) -> Self {
                    Self { _inner: inner }
                }
                /// Returns the lower bound of the span.
                ///
                /// ## Returns
                /// * The lower bound as a `DateTime`.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::datetime::tstz_span::TsTzSpan;
                /// # use meos::collections::base::span::Span;
                /// # use chrono::naive::NaiveDate;
                ///
                /// let from_ymd_opt = |y, m, d| NaiveDate::from_ymd_opt(y, m, d)
                ///                                 .unwrap().and_hms_opt(0, 0, 0)
                ///                                 .unwrap().and_utc();
                ///
                /// let span: TsTzSpan = (from_ymd_opt(2023, 1, 1)..from_ymd_opt(2023, 1, 15)).into();
                /// let lower = span.lower();
                /// assert_eq!(lower, from_ymd_opt(2023, 1, 1));
                /// ```
                fn lower(&self) -> Self::Type {
                    let timestamp = unsafe { meos_sys::tstzspan_lower(self.inner()) };
                    from_meos_timestamp(timestamp)
                }
                /// Returns the upper bound of the span.
                ///
                /// ## Returns
                /// * The upper bound as a `DateTime`.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::datetime::tstz_span::TsTzSpan;
                /// # use meos::collections::base::span::Span;
                /// # use chrono::naive::NaiveDate;
                ///
                /// let from_ymd_opt = |y, m, d| NaiveDate::from_ymd_opt(y, m, d)
                ///                                 .unwrap().and_hms_opt(0, 0, 0)
                ///                                 .unwrap().and_utc();
                ///
                /// let span: TsTzSpan = (from_ymd_opt(2023, 1, 1)..from_ymd_opt(2023, 1, 15)).into();
                /// let upper = span.upper();
                /// assert_eq!(upper, from_ymd_opt(2023, 1, 15));
                /// ```
                fn upper(&self) -> Self::Type {
                    let timestamp = unsafe { meos_sys::tstzspan_upper(self.inner()) };
                    from_meos_timestamp(timestamp)
                }
                /// Return a new `TsTzSpan` with the lower and upper bounds shifted by `delta`.
                ///
                /// # Arguments
                /// * `delta` - The value to shift by, as a `DateTime`.
                ///
                /// # Returns
                /// A new `TsTzSpan` instance.
                ///
                /// # Example
                /// ```
                /// # use meos::collections::datetime::tstz_span::TsTzSpan;
                /// # use meos::collections::base::span::Span;
                /// # use meos::init;
                /// use chrono::naive::NaiveDate;
                /// use chrono::TimeDelta;
                /// # init();
                ///
                /// let from_ymd_opt = |y, m, d| NaiveDate::from_ymd_opt(y, m, d)
                ///                                 .unwrap().and_hms_opt(0, 0, 0)
                ///                                 .unwrap().and_utc();
                ///
                /// let span: TsTzSpan = (from_ymd_opt(2023, 1, 1)..from_ymd_opt(2023, 1, 15)).into();
                /// let shifted_span = span.shift(TimeDelta::weeks(8));
                /// let expected_span: TsTzSpan = (from_ymd_opt(2023, 4, 23)..from_ymd_opt(2023, 5, 7)).into();
                /// assert_eq!(shifted_span, expected_span);
                /// ```
                fn shift(&self, delta: TimeDelta) -> TsTzSpan {
                    self.shift_scale(Some(delta), None)
                }
                /// Return a new `TsTzSpan` with the lower and upper bounds scaled so that the width is `width`.
                ///
                /// # Arguments
                /// * `width` - The new width, as a `DateTime`.
                ///
                /// # Returns
                /// A new `TsTzSpan` instance.
                ///
                /// # Example
                /// ```
                /// # use meos::collections::datetime::tstz_span::TsTzSpan;
                /// # use meos::collections::base::span::Span;
                /// # use meos::init;
                /// use chrono::naive::NaiveDate;
                /// use chrono::TimeDelta;
                /// # init();
                ///
                /// let from_ymd_opt = |y, m, d| NaiveDate::from_ymd_opt(y, m, d)
                ///                                 .unwrap().and_hms_opt(0, 0, 0)
                ///                                 .unwrap().and_utc();
                ///
                /// let span: TsTzSpan = (from_ymd_opt(2023, 1, 1)..from_ymd_opt(2023, 1, 15)).into();
                /// let scaled_span = span.scale(TimeDelta::weeks(4));
                /// let expected_span: TsTzSpan = (from_ymd_opt(2023, 1, 1)..from_ymd_opt(2023, 2, 26)).into();
                /// assert_eq!(scaled_span, expected_span);
                /// ```
                fn scale(&self, width: TimeDelta) -> TsTzSpan {
                    self.shift_scale(None, Some(width))
                }
                /// Return a new `TsTzSpan` with the lower and upper bounds shifted by `delta` and scaled so that the width is `width`.
                ///
                /// # Arguments
                /// * `delta` - The value to shift by, as a `DateTime`.
                /// * `width` - The new width, as a `DateTime`.
                ///
                /// # Returns
                /// A new `TsTzSpan` instance.
                ///
                /// # Example
                /// ```
                /// # use meos::collections::datetime::tstz_span::TsTzSpan;
                /// # use meos::collections::base::span::Span;
                /// use chrono::naive::NaiveDate;
                /// use chrono::TimeDelta;
                /// # use meos::init;
                /// # init();
                /// let from_ymd_opt = |y, m, d| NaiveDate::from_ymd_opt(y, m, d)
                ///                                 .unwrap().and_hms_opt(0, 0, 0)
                ///                                 .unwrap().and_utc();
                ///
                /// let span: TsTzSpan = (from_ymd_opt(2023, 1, 1)..from_ymd_opt(2023, 1, 15)).into();
                /// let shifted_scaled_span = span.shift_scale(Some(TimeDelta::weeks(4)), Some(TimeDelta::weeks(4)));
                /// let expected_span: TsTzSpan = (from_ymd_opt(2023, 2, 26)..from_ymd_opt(2023, 4, 23)).into();
                /// assert_eq!(shifted_scaled_span, expected_span);
                /// ```
                fn shift_scale(
                    &self,
                    delta: Option<TimeDelta>,
                    width: Option<TimeDelta>,
                ) -> TsTzSpan {
                    let d = {
                        if let Some(d) = delta {
                            &*Box::new(create_interval(d)) as *const meos_sys::Interval
                        } else {
                            std::ptr::null()
                        }
                    };
                    let w = {
                        if let Some(w) = width {
                            &*Box::new(create_interval(w)) as *const meos_sys::Interval
                        } else {
                            std::ptr::null()
                        }
                    };
                    let modified = unsafe {
                        meos_sys::tstzspan_shift_scale(self._inner, d, w)
                    };
                    TsTzSpan::from_inner(modified)
                }
                /// Calculates the distance between this `TsTzSpan` and a specific timestamp (`value`).
                ///
                /// ## Arguments
                /// * `value` - Anvalue `TsTzSpan` to calculate the distance to.
                ///
                /// ## Returns
                /// A `TimeDelta` representing the distance in seconds between the two spans.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::datetime::tstz_span::TsTzSpan;
                /// # use meos::init;
                /// use std::str::FromStr;
                /// # use meos::collections::base::span::Span;
                /// use chrono::TimeDelta;
                /// # init();
                /// let span1 = TsTzSpan::from_str("[2019-09-08 00:00:00+00, 2019-09-10 00:00:00+00]").unwrap();
                /// let span2 = TsTzSpan::from_str("[2019-09-12 00:00:00+00, 2019-09-14 00:00:00+00]").unwrap();
                /// let distance = span1.distance_to_span(&span2);
                /// assert_eq!(distance, TimeDelta::days(2));
                /// ```
                fn distance_to_value(&self, value: &Self::Type) -> TimeDelta {
                    unsafe {
                        TimeDelta::seconds(
                            meos_sys::distance_span_timestamptz(
                                self.inner(),
                                to_meos_timestamp(value),
                            ) as i64,
                        )
                    }
                }
                /// Calculates the distance between this `TsTzSpan` and another `TsTzSpan`.
                ///
                /// ## Arguments
                /// * `other` - Another `TsTzSpan` to calculate the distance to.
                ///
                /// ## Returns
                /// A `TimeDelta` representing the distance in seconds between the two spans.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::datetime::tstz_span::TsTzSpan;
                /// # use meos::collections::base::span::Span;
                /// # use chrono::{TimeDelta, TimeZone, Utc};
                /// # use meos::init;
                /// use std::str::FromStr;
                /// # init();
                /// let span_set1 = TsTzSpan::from_str("[2019-09-08 00:00:00+00, 2019-09-10 00:00:00+00]").unwrap();
                /// let span_set2 = TsTzSpan::from_str("[2018-08-07 00:00:00+00, 2018-08-17 00:00:00+00]").unwrap();
                /// let distance = span_set1.distance_to_span(&span_set2);
                /// assert_eq!(distance, TimeDelta::days(387));
                /// ```
                fn distance_to_span(&self, other: &Self) -> TimeDelta {
                    unsafe {
                        TimeDelta::seconds(
                            meos_sys::distance_tstzspan_tstzspan(
                                self.inner(),
                                other.inner(),
                            ) as i64,
                        )
                    }
                }
            }
            impl TsTzSpan {
                pub fn duration(&self) -> TimeDelta {
                    from_interval(unsafe {
                        meos_sys::tstzspan_duration(self._inner).read()
                    })
                }
            }
            impl BoundingBox for TsTzSpan {}
            impl Clone for TsTzSpan {
                fn clone(&self) -> Self {
                    unsafe { Self::from_inner(meos_sys::span_copy(self._inner)) }
                }
            }
            impl Hash for TsTzSpan {
                fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                    let hash = unsafe { meos_sys::span_hash(self._inner) };
                    state.write_u32(hash);
                    state.finish();
                }
            }
            impl std::str::FromStr for TsTzSpan {
                type Err = ParseError;
                /// Parses a `TsTzSpan` from a string representation.
                ///
                /// ## Arguments
                /// * `string` - A string slice containing the representation.
                ///
                /// ## Returns
                /// * A `TsTzSpan` instance.
                ///
                /// ## Errors
                /// * Returns `ParseSpanError` if the string cannot be parsed.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::datetime::tstz_span::TsTzSpan;
                /// # use meos::collections::base::span::Span;
                /// # use std::str::FromStr;
                /// # use meos::init;
                /// use chrono::NaiveDate;
                /// # init();
                /// let from_ymd_opt = |y, m, d| NaiveDate::from_ymd_opt(y, m, d)
                ///                                 .unwrap().and_hms_opt(0, 0, 0)
                ///                                 .unwrap().and_utc();
                ///
                /// let span: TsTzSpan = "(2019-09-08, 2019-09-10)".parse().expect("Failed to parse span");
                /// assert_eq!(span.lower(), from_ymd_opt(2019, 9, 8));
                /// assert_eq!(span.upper(), from_ymd_opt(2019, 9, 10));
                /// ```
                fn from_str(string: &str) -> Result<Self, Self::Err> {
                    CString::new(string)
                        .map_err(|_| ParseError)
                        .map(|string| {
                            let inner = unsafe {
                                meos_sys::tstzspan_in(string.as_ptr())
                            };
                            Self::from_inner(inner)
                        })
                }
            }
            impl cmp::PartialEq for TsTzSpan {
                /// Checks if two `TsTzSpan` instances are equal.
                ///
                /// # Arguments
                /// * `other` - Another `TsTzSpan` instance.
                ///
                /// ## Returns
                /// * `true` if the spans are equal, `false` otherwise.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::datetime::tstz_span::TsTzSpan;
                /// # use meos::collections::base::span::Span;
                /// use chrono::naive::NaiveDate;
                ///
                /// let from_ymd_opt = |y, m, d| NaiveDate::from_ymd_opt(y, m, d)
                ///                                 .unwrap().and_hms_opt(0, 0, 0)
                ///                                 .unwrap().and_utc();
                ///
                /// let span1: TsTzSpan = (from_ymd_opt(1, 1, 1)..from_ymd_opt(2, 2, 2)).into();
                /// let span2: TsTzSpan = (from_ymd_opt(1, 1, 1)..from_ymd_opt(2, 2, 2)).into();
                /// assert_eq!(span1, span2);
                /// ```
                fn eq(&self, other: &Self) -> bool {
                    unsafe { meos_sys::span_eq(self._inner, other._inner) }
                }
            }
            impl cmp::Eq for TsTzSpan {}
            impl<Tz: TimeZone> From<Range<DateTime<Tz>>> for TsTzSpan {
                fn from(Range { start, end }: Range<DateTime<Tz>>) -> Self {
                    let inner = unsafe {
                        meos_sys::tstzspan_make(
                            to_meos_timestamp(&start),
                            to_meos_timestamp(&end),
                            true,
                            false,
                        )
                    };
                    Self::from_inner(inner)
                }
            }
            impl<Tz: TimeZone> From<RangeInclusive<DateTime<Tz>>> for TsTzSpan {
                fn from(range: RangeInclusive<DateTime<Tz>>) -> Self {
                    let inner = unsafe {
                        meos_sys::tstzspan_make(
                            to_meos_timestamp(range.start()),
                            to_meos_timestamp(range.end()),
                            true,
                            true,
                        )
                    };
                    Self::from_inner(inner)
                }
            }
            impl Debug for TsTzSpan {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let out_str = unsafe { meos_sys::tstzspan_out(self._inner) };
                    let c_str = unsafe { CStr::from_ptr(out_str) };
                    let str = c_str.to_str().map_err(|_| std::fmt::Error)?;
                    let result = f.write_str(str);
                    unsafe { libc::free(out_str as *mut c_void) };
                    result
                }
            }
            impl BitAnd for TsTzSpan {
                type Output = Option<TsTzSpan>;
                /// Computes the dateersection of two `TsTzSpan` instances.
                ///
                /// # Arguments
                /// * `other` - Another `TsTzSpan` instance.
                ///
                /// ## Returns
                /// * An `Option<TsTzSpan>` containing the dateersection, or `None` if there is no dateersection.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::datetime::tstz_span::TsTzSpan;
                /// # use meos::collections::base::span::Span;
                /// # use std::str::FromStr;
                /// use chrono::naive::NaiveDate;
                ///
                /// let from_ymd_opt = |y, m, d| NaiveDate::from_ymd_opt(y, m, d)
                ///                                 .unwrap().and_hms_opt(0, 0, 0)
                ///                                 .unwrap().and_utc();
                ///
                /// let span1: TsTzSpan = (from_ymd_opt(1, 1, 1)..from_ymd_opt(1, 1, 11)).into();
                /// let span2: TsTzSpan = (from_ymd_opt(1, 1, 9)..from_ymd_opt(2, 1, 11)).into();
                /// let date_intersection = (span1 & span2).unwrap();
                ///
                /// assert_eq!(date_intersection, (from_ymd_opt(1, 1, 9)..from_ymd_opt(1, 1, 11)).into())
                /// ```
                fn bitand(self, other: Self) -> Self::Output {
                    let result = unsafe {
                        meos_sys::intersection_span_span(self._inner, other._inner)
                    };
                    if !result.is_null() {
                        Some(TsTzSpan::from_inner(result))
                    } else {
                        None
                    }
                }
            }
            impl PartialOrd for TsTzSpan {
                fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
                    let cmp = unsafe { meos_sys::span_cmp(self._inner, other._inner) };
                    match cmp {
                        -1 => Some(cmp::Ordering::Less),
                        0 => Some(cmp::Ordering::Equal),
                        1 => Some(cmp::Ordering::Greater),
                        _ => None,
                    }
                }
            }
            impl Ord for TsTzSpan {
                fn cmp(&self, other: &Self) -> cmp::Ordering {
                    self.partial_cmp(other)
                        .expect(
                            "Unreachable since for non-null and same types spans, we only return -1, 0, or 1",
                        )
                }
            }
        }
        pub mod tstz_span_set {
            use std::ffi::{c_void, CStr, CString};
            use chrono::DateTime;
            use chrono::Datelike;
            use chrono::TimeDelta;
            use chrono::Utc;
            use collection::{impl_collection, Collection};
            use span::Span;
            use span_set::impl_iterator;
            use std::fmt::Debug;
            use std::hash::Hash;
            use std::ops::{BitAnd, BitOr};
            use crate::collections::base::span_set::SpanSet;
            use crate::collections::base::*;
            use crate::errors::ParseError;
            use crate::utils::to_meos_timestamp;
            use super::tstz_span::TsTzSpan;
            use crate::utils::create_interval;
            pub struct TsTzSpanSet {
                _inner: *const meos_sys::SpanSet,
            }
            impl Drop for TsTzSpanSet {
                fn drop(&mut self) {
                    unsafe {
                        libc::free(self._inner as *mut c_void);
                    }
                }
            }
            impl Collection for TsTzSpanSet {
                type Type = DateTime<Utc>;
                fn is_contained_in(&self, container: &Self) -> bool {
                    unsafe {
                        meos_sys::contained_spanset_spanset(
                            self.inner(),
                            container.inner(),
                        )
                    }
                }
                fn overlaps(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::overlaps_spanset_spanset(self.inner(), other.inner())
                    }
                }
                fn is_left(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::left_spanset_spanset(self.inner(), other.inner())
                    }
                }
                fn is_over_or_left(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::overleft_spanset_spanset(self.inner(), other.inner())
                    }
                }
                fn is_over_or_right(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::overright_spanset_spanset(self.inner(), other.inner())
                    }
                }
                fn is_right(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::right_spanset_spanset(self.inner(), other.inner())
                    }
                }
                fn is_adjacent(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::adjacent_spanset_spanset(self.inner(), other.inner())
                    }
                }
                fn contains(&self, content: &DateTime<Utc>) -> bool {
                    unsafe {
                        meos_sys::contains_spanset_date(
                            self.inner(),
                            content.num_days_from_ce(),
                        )
                    }
                }
            }
            impl span_set::SpanSet for TsTzSpanSet {
                type SpanType = TsTzSpan;
                type SubsetType = TimeDelta;
                fn inner(&self) -> *const meos_sys::SpanSet {
                    self._inner
                }
                fn from_inner(inner: *const meos_sys::SpanSet) -> Self
                where
                    Self: Sized,
                {
                    Self { _inner: inner }
                }
                fn width(&self, _ignore_gaps: bool) -> Self::Type {
                    {
                        ::core::panicking::panic_fmt(
                            format_args!(
                                "not implemented: {0}",
                                format_args!("Not implemented for date"),
                            ),
                        );
                    }
                }
                /// Return a new `TsTzSpanSet` with the lower and upper bounds shifted by `delta`.
                ///
                /// ## Arguments
                /// * `delta` - The value to shift by.
                ///
                /// ## Returns
                /// A new `TsTzSpanSet` instance.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::datetime::tstz_span_set::TsTzSpanSet;
                /// # use meos::init;
                /// # use std::str::FromStr;
                /// # use meos::collections::base::span_set::SpanSet;
                /// use chrono::TimeDelta;
                /// # init();
                /// let span_set = TsTzSpanSet::from_str("{[2019-09-08 00:00:00+00, 2019-09-10 00:00:00+00], [2019-09-16 00:00:00+00, 2019-09-20 00:00:00+00]}").unwrap();
                /// let shifted_span_set = span_set.shift(TimeDelta::days(5));
                ///
                /// let expected_shifted_span_set =
                ///     TsTzSpanSet::from_str("{[2019-09-18 00:00:00+00, 2019-09-20 00:00:00+00], [2019-09-26 00:00:00+00, 2019-09-30 00:00:00+00]}").unwrap();
                /// assert_eq!(shifted_span_set, expected_shifted_span_set);
                /// ```
                fn shift(&self, delta: TimeDelta) -> TsTzSpanSet {
                    self.shift_scale(Some(delta), None)
                }
                /// Return a new `TsTzSpanSet` with the lower and upper bounds scaled so that the width is `width`.
                ///
                /// ## Arguments
                /// * `width` - The new width.
                ///
                /// ## Returns
                /// A new `TsTzSpanSet` instance.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::datetime::tstz_span_set::TsTzSpanSet;
                /// # use meos::init;
                /// # use std::str::FromStr;
                /// # use meos::collections::base::span_set::SpanSet;
                /// use chrono::TimeDelta;
                /// # init();
                /// let span_set = TsTzSpanSet::from_str("{[2019-09-08 00:00:00+00, 2019-09-10 00:00:00+00], [2019-09-13 00:00:00+00, 2019-09-15 00:00:00+00]}").unwrap();
                /// let scaled_span_set = span_set.scale(TimeDelta::days(5));
                ///
                /// let expected_scaled_span_set =
                ///     TsTzSpanSet::from_str("{[2019-09-08 00:00:00+00, 2019-09-10 20:34:17.142857+00], [2019-09-15 03:25:42.857142+00, 2019-09-18 00:00:00+00]}").unwrap();
                /// assert_eq!(scaled_span_set, expected_scaled_span_set);
                /// ```
                fn scale(&self, width: TimeDelta) -> TsTzSpanSet {
                    self.shift_scale(None, Some(width))
                }
                /// Return a new `TsTzSpanSet` with the lower and upper bounds shifted by `delta` and scaled so that the width is `width`.
                ///
                /// ## Arguments
                /// * `delta` - The value to shift by.
                /// * `width` - The new width.
                ///
                /// ## Returns
                /// A new `TsTzSpanSet` instance.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::datetime::tstz_span_set::TsTzSpanSet;
                /// # use meos::init;
                /// # use std::str::FromStr;
                /// # use meos::collections::base::span_set::SpanSet;
                /// use chrono::TimeDelta;
                /// # init();
                /// let span_set = TsTzSpanSet::from_str("{[2019-09-08 00:00:00+00, 2019-09-10 00:00:00+00], [2019-09-11 00:00:00+00, 2019-09-12 00:00:00+00]}").unwrap();
                /// let shifted_scaled_span_set = span_set.shift_scale(Some(TimeDelta::days(5)), Some(TimeDelta::days(10)));
                ///
                /// let expected_shifted_scaled_span_set =
                ///     TsTzSpanSet::from_str("{[2019-09-18 00:00:00+00, 2019-09-28 00:00:00+00], [2019-10-03 00:00:00+00, 2019-10-08 00:00:00+00]}").unwrap();
                /// assert_eq!(shifted_scaled_span_set, expected_shifted_scaled_span_set);
                /// ```
                fn shift_scale(
                    &self,
                    delta: Option<TimeDelta>,
                    width: Option<TimeDelta>,
                ) -> TsTzSpanSet {
                    let d = {
                        if let Some(d) = delta {
                            &*Box::new(create_interval(d)) as *const meos_sys::Interval
                        } else {
                            std::ptr::null()
                        }
                    };
                    let w = {
                        if let Some(w) = width {
                            &*Box::new(create_interval(w)) as *const meos_sys::Interval
                        } else {
                            std::ptr::null()
                        }
                    };
                    let modified = unsafe {
                        meos_sys::tstzspanset_shift_scale(self._inner, d, w)
                    };
                    TsTzSpanSet::from_inner(modified)
                }
                /// Calculates the distance between this `TsTzSpanSet` and a specific timestamp (`value`).
                ///
                /// ## Arguments
                /// * `value` - A timestamp represented by `TimeDelta` from the Unix epoch.
                ///
                /// ## Returns
                /// A `TimeDelta` representing the distance in seconds between this `TsTzSpanSet` and the given timestamp.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::datetime::tstz_span_set::TsTzSpanSet;
                /// # use meos::collections::base::span_set::SpanSet;
                /// # use chrono::{TimeDelta, TimeZone, Utc};
                /// # use meos::init;
                /// use std::str::FromStr;
                /// # init();
                /// let span_set = TsTzSpanSet::from_str("{[2019-09-08 00:00:00+00, 2019-09-10 00:00:00+00], [2019-09-11 00:00:00+00, 2019-09-12 00:00:00+00]}").unwrap();
                /// let timestamp = Utc.with_ymd_and_hms(2019, 9, 5, 0, 0, 0).unwrap();
                /// let distance = span_set.distance_to_value(&timestamp);
                /// assert_eq!(distance, TimeDelta::days(3));
                /// ```
                fn distance_to_value(&self, value: &DateTime<Utc>) -> TimeDelta {
                    unsafe {
                        TimeDelta::seconds(
                            meos_sys::distance_spanset_timestamptz(
                                self.inner(),
                                to_meos_timestamp(value),
                            ) as i64,
                        )
                    }
                }
                /// Calculates the distance between this `TsTzSpanSet` and another `TsTzSpanSet`.
                ///
                /// ## Arguments
                /// * `other` - Another `TsTzSpanSet` to calculate the distance to.
                ///
                /// ## Returns
                /// A `TimeDelta` representing the distance in seconds between the two span sets.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::datetime::tstz_span_set::TsTzSpanSet;
                /// # use crate::meos::collections::base::span_set::SpanSet;
                /// # use chrono::{TimeDelta, TimeZone, Utc};
                /// # use meos::init;
                /// use std::str::FromStr;
                /// # init();
                /// let span_set1 = TsTzSpanSet::from_str("{[2019-09-08 00:00:00+00, 2019-09-10 00:00:00+00], [2019-09-11 00:00:00+00, 2019-09-12 00:00:00+00]}").unwrap();
                /// let span_set2 = TsTzSpanSet::from_str("{[2018-08-07 00:00:00+00, 2018-08-17 00:00:00+00], [2018-10-17 00:00:00+00, 2018-10-20 00:00:00+00]}").unwrap();
                /// let distance = span_set1.distance_to_span_set(&span_set2);
                /// assert_eq!(distance, TimeDelta::days(323));
                /// ```
                fn distance_to_span_set(&self, other: &Self) -> TimeDelta {
                    unsafe {
                        TimeDelta::seconds(
                            meos_sys::distance_tstzspanset_tstzspanset(
                                self.inner(),
                                other.inner(),
                            ) as i64,
                        )
                    }
                }
                /// Calculates the distance between this `TsTzSpanSet` and a `TsTzSpan`.
                ///
                /// ## Arguments
                /// * `other` - A `TsTzSpan` to calculate the distance to.
                ///
                /// ## Returns
                /// A `TimeDelta` representing the distance in seconds between the span set and the span.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::datetime::tstz_span_set::TsTzSpanSet;
                /// # use meos::collections::base::span_set::SpanSet;
                /// # use meos::collections::datetime::tstz_span::TsTzSpan;
                /// # use meos::collections::base::span::Span;
                /// # use chrono::{TimeDelta, TimeZone, Utc};
                /// # use meos::init;
                /// use std::str::FromStr;
                /// # init();
                /// let span_set = TsTzSpanSet::from_str("{[2019-09-08 00:00:00+00, 2019-09-10 00:00:00+00], [2019-09-11 00:00:00+00, 2019-09-12 00:00:00+00]}").unwrap();
                /// let span = TsTzSpan::from_str("[2018-08-07 00:00:00+00, 2018-08-17 00:00:00+00]").unwrap();
                /// let distance = span_set.distance_to_span(&span);
                /// assert_eq!(distance, TimeDelta::days(387));
                /// ```
                fn distance_to_span(&self, span: &Self::SpanType) -> Self::SubsetType {
                    unsafe {
                        TimeDelta::seconds(
                            meos_sys::distance_tstzspanset_tstzspan(
                                self.inner(),
                                span.inner(),
                            ) as i64,
                        )
                    }
                }
            }
            impl Clone for TsTzSpanSet {
                fn clone(&self) -> TsTzSpanSet {
                    self.copy()
                }
            }
            impl IntoIterator for TsTzSpanSet {
                type Item = <TsTzSpanSet as SpanSet>::SpanType;
                type IntoIter = std::vec::IntoIter<Self::Item>;
                fn into_iter(self) -> Self::IntoIter {
                    self.spans().into_iter()
                }
            }
            impl FromIterator<<TsTzSpanSet as SpanSet>::SpanType> for TsTzSpanSet {
                fn from_iter<T: IntoIterator<Item = <TsTzSpanSet as SpanSet>::SpanType>>(
                    iter: T,
                ) -> Self {
                    iter.into_iter().collect()
                }
            }
            impl<'a> FromIterator<&'a <TsTzSpanSet as SpanSet>::SpanType>
            for TsTzSpanSet {
                fn from_iter<
                    T: IntoIterator<Item = &'a <TsTzSpanSet as SpanSet>::SpanType>,
                >(iter: T) -> Self {
                    let mut iter = iter.into_iter();
                    let first = iter.next().unwrap();
                    iter.fold(
                        first.to_spanset(),
                        |acc, item| { (acc | item.to_spanset()).unwrap() },
                    )
                }
            }
            impl Hash for TsTzSpanSet {
                fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                    let hash = unsafe { meos_sys::spanset_hash(self._inner) };
                    state.write_u32(hash);
                    state.finish();
                }
            }
            impl std::str::FromStr for TsTzSpanSet {
                type Err = ParseError;
                fn from_str(string: &str) -> Result<Self, Self::Err> {
                    CString::new(string)
                        .map_err(|_| ParseError)
                        .map(|string| {
                            let inner = unsafe {
                                meos_sys::tstzspanset_in(string.as_ptr())
                            };
                            Self::from_inner(inner)
                        })
                }
            }
            impl std::cmp::PartialEq for TsTzSpanSet {
                fn eq(&self, other: &Self) -> bool {
                    unsafe { meos_sys::spanset_eq(self._inner, other._inner) }
                }
            }
            impl Debug for TsTzSpanSet {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let out_str = unsafe { meos_sys::tstzspanset_out(self._inner) };
                    let c_str = unsafe { CStr::from_ptr(out_str) };
                    let str = c_str.to_str().map_err(|_| std::fmt::Error)?;
                    let result = f.write_str(str);
                    unsafe { libc::free(out_str as *mut c_void) };
                    result
                }
            }
            impl BitAnd<TsTzSpanSet> for TsTzSpanSet {
                type Output = Option<TsTzSpanSet>;
                /// Computes the dateersection of two `TsTzSpanSet`s.
                ///
                /// ## Arguments
                ///
                /// * `other` - Another `TsTzSpanSet` to dateersect with.
                ///
                /// ## Returns
                ///
                /// * `Some(TsTzSpanSet)` - A new `TsTzSpanSet` containing the dateersection, if it exists.
                /// * `None` - If the dateersection is empty.
                ///
                /// ## Example
                ///
                /// ```
                /// # use meos::collections::datetime::tstz_span_set::TsTzSpanSet;
                /// # use meos::init;
                /// # use std::str::FromStr;
                /// # use meos::collections::base::span_set::SpanSet;
                /// # init();
                /// let span_set1 = TsTzSpanSet::from_str("{[2019-09-08 00:00:00+00, 2019-09-10 00:00:00+00], [2019-09-15 00:00:00+00, 2019-09-20 00:00:00+00]}").unwrap();
                /// let span_set2 = TsTzSpanSet::from_str("{[2019-09-15 00:00:00+00, 2019-09-30 00:00:00+00], [2019-11-11 00:00:00+00, 2019-11-12 00:00:00+00]}").unwrap();
                ///
                /// let expected_result = TsTzSpanSet::from_str("{[2019-09-15 00:00:00+00, 2019-09-20 00:00:00+00]}").unwrap();
                /// assert_eq!((span_set1 & span_set2).unwrap(), expected_result);
                /// ```
                fn bitand(self, other: TsTzSpanSet) -> Self::Output {
                    self.intersection(&other)
                }
            }
            impl BitOr for TsTzSpanSet {
                type Output = Option<TsTzSpanSet>;
                /// Computes the union of two `TsTzSpanSet`s.
                ///
                /// ## Arguments
                ///
                /// * `other` - Another `TsTzSpanSet` to union with.
                ///
                /// ## Returns
                ///
                /// * `Some(TsTzSpanSet)` - A new `TsTzSpanSet` containing the union.
                /// * `None` - If the union is empty.
                ///
                /// ## Example
                ///
                /// ```
                /// # use meos::collections::datetime::tstz_span_set::TsTzSpanSet;
                /// # use meos::init;
                /// # use std::str::FromStr;
                /// # use meos::collections::base::span_set::SpanSet;
                /// # init();
                /// let span_set1 = TsTzSpanSet::from_str("{[2019-09-08 00:00:00+00, 2019-09-10 00:00:00+00], [2019-09-15 00:00:00+00, 2019-09-20 00:00:00+00]}").unwrap();
                /// let span_set2 = TsTzSpanSet::from_str("{[2019-09-15 00:00:00+00, 2019-09-30 00:00:00+00], [2019-11-11 00:00:00+00, 2019-11-12 00:00:00+00]}").unwrap();
                ///
                /// let expected_result = TsTzSpanSet::from_str("{[2019-09-08 00:00:00+00, 2019-09-10 00:00:00+00], [2019-09-15 00:00:00+00, 2019-09-30 00:00:00+00], [2019-11-11 00:00:00+00, 2019-11-12 00:00:00+00]}").unwrap();
                /// assert_eq!((span_set1 | span_set2).unwrap(), expected_result)
                /// ```
                fn bitor(self, other: Self) -> Self::Output {
                    self.union(&other)
                }
            }
        }
        /// Needed since MEOS uses as a baseline date 2000-01-01
        pub(crate) const DAYS_UNTIL_2000: Days = Days::new(730_120);
        pub(crate) const MICROSECONDS_UNTIL_2000: i64 = 946684800000000;
    }
    pub mod geo {}
    pub mod number {
        pub mod number_span {
            use crate::collections::base::span::Span;
            pub(crate) trait NumberSpan: Span {}
        }
        pub mod number_span_set {
            use crate::collections::base::span_set::SpanSet;
            pub(crate) trait NumberSpanSet: SpanSet {}
        }
        pub mod float_span {
            use std::{
                cmp, ffi::{c_void, CStr, CString},
                fmt::Debug, hash::Hash, ops::{BitAnd, Range, RangeInclusive},
            };
            use collection::{impl_collection, Collection};
            use span::Span;
            use crate::{collections::base::*, errors::ParseError};
            use super::number_span::NumberSpan;
            pub struct FloatSpan {
                _inner: *const meos_sys::Span,
            }
            impl Drop for FloatSpan {
                fn drop(&mut self) {
                    unsafe {
                        libc::free(self._inner as *mut c_void);
                    }
                }
            }
            impl Collection for FloatSpan {
                type Type = f64;
                fn is_contained_in(&self, container: &Self) -> bool {
                    unsafe {
                        meos_sys::contained_span_span(self.inner(), container.inner())
                    }
                }
                fn overlaps(&self, other: &Self) -> bool {
                    unsafe { meos_sys::overlaps_span_span(self.inner(), other.inner()) }
                }
                fn is_left(&self, other: &Self) -> bool {
                    unsafe { meos_sys::left_span_span(self.inner(), other.inner()) }
                }
                fn is_over_or_left(&self, other: &Self) -> bool {
                    unsafe { meos_sys::overleft_span_span(self.inner(), other.inner()) }
                }
                fn is_over_or_right(&self, other: &Self) -> bool {
                    unsafe { meos_sys::overright_span_span(self.inner(), other.inner()) }
                }
                fn is_right(&self, other: &Self) -> bool {
                    unsafe { meos_sys::right_span_span(self.inner(), other.inner()) }
                }
                fn is_adjacent(&self, other: &Self) -> bool {
                    unsafe { meos_sys::adjacent_span_span(self.inner(), other.inner()) }
                }
                fn contains(&self, content: &f64) -> bool {
                    unsafe { meos_sys::contains_span_float(self.inner(), *content) }
                }
            }
            impl span::Span for FloatSpan {
                type SubsetType = Self::Type;
                fn inner(&self) -> *const meos_sys::Span {
                    self._inner
                }
                /// Creates a new `FloatSpan` from an inner pointer to a `meos_sys::Span`.
                ///
                /// # Arguments
                /// * `inner` - A pointer to the inner `meos_sys::Span`.
                ///
                /// ## Returns
                /// * A new `FloatSpan` instance.
                fn from_inner(inner: *const meos_sys::Span) -> Self {
                    Self { _inner: inner }
                }
                /// Returns the lower bound of the span.
                ///
                /// ## Returns
                /// * The lower bound as a `f64`.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::number::float_span::FloatSpan;
                /// # use meos::collections::base::span::Span;
                ///
                /// let span: FloatSpan = (12.9..67.8).into();
                /// let lower = span.lower();
                /// ```
                fn lower(&self) -> Self::Type {
                    unsafe { meos_sys::floatspan_lower(self.inner()) }
                }
                /// Returns the upper bound of the span.
                ///
                /// ## Returns
                /// * The upper bound as a `f64`.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::number::float_span::FloatSpan;
                /// # use meos::collections::base::span::Span;
                ///
                /// let span: FloatSpan = (12.9..67.8).into();;
                ///
                /// assert_eq!(span.upper(), 67.8)
                /// ```
                fn upper(&self) -> Self::Type {
                    unsafe { meos_sys::floatspan_upper(self.inner()) }
                }
                /// Return a new `FloatSpan` with the lower and upper bounds shifted by `delta`.
                ///
                /// # Arguments
                /// * `delta` - The value to shift by.
                ///
                /// # Returns
                /// A new `FloatSpan` instance.
                ///
                /// # Example
                /// ```
                /// # use meos::collections::number::float_span::FloatSpan;
                /// # use meos::collections::base::span::Span;
                ///
                /// let span: FloatSpan = (12.9..67.8).into();
                /// let shifted_span = span.shift(5.0);
                ///
                /// assert_eq!(shifted_span, (17.9..72.8).into())
                /// ```
                fn shift(&self, delta: f64) -> FloatSpan {
                    self.shift_scale(Some(delta), None)
                }
                /// Return a new `FloatSpan` with the lower and upper bounds scaled so that the width is `width`.
                ///
                /// # Arguments
                /// * `width` - The new width.
                ///
                /// # Returns
                /// A new `FloatSpan` instance.
                ///
                /// # Example
                /// ```
                /// # use meos::collections::number::float_span::FloatSpan;
                /// # use meos::collections::base::span::Span;
                ///
                /// let span: FloatSpan = (12.9..67.8).into();
                /// let scaled_span = span.scale(10.0);
                ///
                /// assert_eq!(scaled_span, (12.9..22.9).into())
                /// ```
                fn scale(&self, width: f64) -> FloatSpan {
                    self.shift_scale(None, Some(width))
                }
                /// Return a new `FloatSpan` with the lower and upper bounds shifted by `delta` and scaled so that the width is `width`.
                ///
                /// # Arguments
                /// * `delta` - The value to shift by.
                /// * `width` - The new width.
                ///
                /// # Returns
                /// A new `FloatSpan` instance.
                ///
                /// # Example
                /// ```
                /// # use meos::collections::number::float_span::FloatSpan;
                /// # use meos::collections::base::span::Span;
                ///
                /// let span: FloatSpan = (12.9..67.8).into();
                /// let shifted_scaled_span = span.shift_scale(Some(5.0), Some(10.0));
                ///
                /// assert_eq!(shifted_scaled_span, (17.9..27.9).into())
                /// ```
                fn shift_scale(
                    &self,
                    delta: Option<f64>,
                    width: Option<f64>,
                ) -> FloatSpan {
                    let d = delta.unwrap_or(0.0);
                    let w = width.unwrap_or(0.0);
                    let modified = unsafe {
                        meos_sys::floatspan_shift_scale(
                            self._inner,
                            d,
                            w,
                            delta.is_some(),
                            width.is_some(),
                        )
                    };
                    FloatSpan::from_inner(modified)
                }
                fn distance_to_value(&self, other: &Self::Type) -> f64 {
                    unsafe { meos_sys::distance_span_float(self.inner(), *other) }
                }
                fn distance_to_span(&self, other: &Self) -> f64 {
                    unsafe {
                        meos_sys::distance_floatspan_floatspan(
                            self.inner(),
                            other.inner(),
                        )
                    }
                }
            }
            impl NumberSpan for FloatSpan {}
            impl Clone for FloatSpan {
                fn clone(&self) -> Self {
                    unsafe { Self::from_inner(meos_sys::span_copy(self._inner)) }
                }
            }
            impl Hash for FloatSpan {
                fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                    let hash = unsafe { meos_sys::span_hash(self._inner) };
                    state.write_u32(hash);
                    state.finish();
                }
            }
            impl std::str::FromStr for FloatSpan {
                type Err = ParseError;
                /// Parses a `FloatSpan` from a string representation.
                ///
                /// ## Arguments
                /// * `string` - A string slice containing the representation.
                ///
                /// ## Returns
                /// * A `FloatSpan` instance.
                ///
                /// ## Errors
                /// * Returns `ParseSpanError` if the string cannot be parsed.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::number::float_span::FloatSpan;
                /// # use meos::collections::base::span::Span;
                /// # use std::str::FromStr;
                ///
                /// let span: FloatSpan = "(12.9, 67.8)".parse().expect("Failed to parse span");
                /// assert_eq!(span.lower(), 12.9);
                /// assert_eq!(span.upper(), 67.8);
                /// ```
                fn from_str(string: &str) -> Result<Self, Self::Err> {
                    CString::new(string)
                        .map_err(|_| ParseError)
                        .map(|string| {
                            let inner = unsafe {
                                meos_sys::floatspan_in(string.as_ptr())
                            };
                            Self::from_inner(inner)
                        })
                }
            }
            impl cmp::PartialEq for FloatSpan {
                /// Checks if two `FloatSpan` instances are equal.
                ///
                /// # Arguments
                /// * `other` - Another `FloatSpan` instance.
                ///
                /// ## Returns
                /// * `true` if the spans are equal, `false` otherwise.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::number::float_span::FloatSpan;
                /// # use meos::collections::base::span::Span;
                /// # use std::str::FromStr;
                ///
                /// let span1: FloatSpan = (12.9..67.8).into();
                /// let span2: FloatSpan = (12.9..67.8).into();
                /// assert_eq!(span1, span2);
                /// ```
                fn eq(&self, other: &Self) -> bool {
                    unsafe { meos_sys::span_eq(self._inner, other._inner) }
                }
            }
            impl cmp::Eq for FloatSpan {}
            impl From<Range<f64>> for FloatSpan {
                fn from(Range { start, end }: Range<f64>) -> Self {
                    let inner = unsafe {
                        meos_sys::floatspan_make(start, end, true, false)
                    };
                    Self::from_inner(inner)
                }
            }
            impl From<Range<f32>> for FloatSpan {
                fn from(Range { start, end }: Range<f32>) -> Self {
                    let inner = unsafe {
                        meos_sys::floatspan_make(start as f64, end as f64, true, false)
                    };
                    Self::from_inner(inner)
                }
            }
            impl From<RangeInclusive<f64>> for FloatSpan {
                fn from(range: RangeInclusive<f64>) -> Self {
                    let inner = unsafe {
                        meos_sys::floatspan_make(
                            *range.start(),
                            *range.end(),
                            true,
                            true,
                        )
                    };
                    Self::from_inner(inner)
                }
            }
            impl From<RangeInclusive<f32>> for FloatSpan {
                fn from(range: RangeInclusive<f32>) -> Self {
                    let inner = unsafe {
                        meos_sys::floatspan_make(
                            *range.start() as f64,
                            *range.end() as f64,
                            true,
                            true,
                        )
                    };
                    Self::from_inner(inner)
                }
            }
            impl Debug for FloatSpan {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let out_str = unsafe { meos_sys::floatspan_out(self._inner, 3) };
                    let c_str = unsafe { CStr::from_ptr(out_str) };
                    let str = c_str.to_str().map_err(|_| std::fmt::Error)?;
                    let result = f.write_str(str);
                    unsafe { libc::free(out_str as *mut c_void) };
                    result
                }
            }
            impl BitAnd for FloatSpan {
                type Output = Option<FloatSpan>;
                /// Computes the intersection of two `FloatSpan` instances.
                ///
                /// # Arguments
                /// * `other` - Another `FloatSpan` instance.
                ///
                /// ## Returns
                /// * An `Option<FloatSpan>` containing the intersection, or `None` if there is no intersection.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::number::float_span::FloatSpan;
                /// # use meos::collections::base::span::Span;
                /// # use std::str::FromStr;
                ///
                /// let span1: FloatSpan = (12.9..67.8).into();
                /// let span2: FloatSpan = (50.0..80.0).into();
                /// let intersection = (span1 & span2).unwrap();
                ///
                /// assert_eq!(intersection, (50.0..67.8).into())
                /// ```
                fn bitand(self, other: FloatSpan) -> Self::Output {
                    self.intersection(&other)
                }
            }
            impl PartialOrd for FloatSpan {
                fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
                    let cmp = unsafe { meos_sys::span_cmp(self._inner, other._inner) };
                    match cmp {
                        -1 => Some(cmp::Ordering::Less),
                        0 => Some(cmp::Ordering::Equal),
                        1 => Some(cmp::Ordering::Greater),
                        _ => None,
                    }
                }
            }
            impl Ord for FloatSpan {
                fn cmp(&self, other: &Self) -> cmp::Ordering {
                    self.partial_cmp(other)
                        .expect(
                            "Unreachable since for non-null and same types spans, we only return -1, 0, or 1",
                        )
                }
            }
        }
        pub mod float_span_set {
            use std::ffi::{c_void, CStr, CString};
            use std::fmt::Debug;
            use std::hash::Hash;
            use std::ops::{BitAnd, BitOr};
            use collection::{impl_collection, Collection};
            use span_set::impl_iterator;
            use crate::collections::base::span::Span;
            use crate::collections::base::span_set::SpanSet;
            use crate::collections::base::*;
            use crate::errors::ParseError;
            use super::float_span::FloatSpan;
            use super::number_span_set::NumberSpanSet;
            pub struct FloatSpanSet {
                _inner: *const meos_sys::SpanSet,
            }
            impl Drop for FloatSpanSet {
                fn drop(&mut self) {
                    unsafe {
                        libc::free(self._inner as *mut c_void);
                    }
                }
            }
            impl Collection for FloatSpanSet {
                type Type = f64;
                fn is_contained_in(&self, container: &Self) -> bool {
                    unsafe {
                        meos_sys::contained_spanset_spanset(
                            self.inner(),
                            container.inner(),
                        )
                    }
                }
                fn overlaps(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::overlaps_spanset_spanset(self.inner(), other.inner())
                    }
                }
                fn is_left(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::left_spanset_spanset(self.inner(), other.inner())
                    }
                }
                fn is_over_or_left(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::overleft_spanset_spanset(self.inner(), other.inner())
                    }
                }
                fn is_over_or_right(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::overright_spanset_spanset(self.inner(), other.inner())
                    }
                }
                fn is_right(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::right_spanset_spanset(self.inner(), other.inner())
                    }
                }
                fn is_adjacent(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::adjacent_spanset_spanset(self.inner(), other.inner())
                    }
                }
                fn contains(&self, content: &f64) -> bool {
                    unsafe { meos_sys::contains_spanset_float(self.inner(), *content) }
                }
            }
            impl span_set::SpanSet for FloatSpanSet {
                type SpanType = FloatSpan;
                type SubsetType = <Self as Collection>::Type;
                fn inner(&self) -> *const meos_sys::SpanSet {
                    self._inner
                }
                fn from_inner(inner: *const meos_sys::SpanSet) -> Self
                where
                    Self: Sized,
                {
                    Self { _inner: inner }
                }
                fn width(&self, ignore_gaps: bool) -> Self::Type {
                    unsafe { meos_sys::floatspanset_width(self.inner(), ignore_gaps) }
                }
                /// Return a new `FloatSpanSet` with the lower and upper bounds shifted by `delta`.
                ///
                /// ## Arguments
                /// * `delta` - The value to shift by.
                ///
                /// ## Returns
                /// A new `FloatSpanSet` instance.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::number::float_span_set::FloatSpanSet;
                /// # use std::str::FromStr;
                /// # use meos::collections::base::span_set::SpanSet;
                ///
                /// let span = FloatSpanSet::from_str("{[17.5, 18.5), [19.5, 20.5)}").unwrap();
                /// let shifted_span = span.shift(5.0);
                ///
                /// let expected_shifted_span =
                ///     FloatSpanSet::from_str("{[22.5, 23.5), [24.5, 25.5)}").unwrap();
                /// assert_eq!(shifted_span, expected_shifted_span);
                /// ```
                fn shift(&self, delta: f64) -> FloatSpanSet {
                    self.shift_scale(Some(delta), None)
                }
                /// Return a new `FloatSpanSet` with the lower and upper bounds scaled so that the width is `width`.
                ///
                /// ## Arguments
                /// * `width` - The new width.
                ///
                /// ## Returns
                /// A new `FloatSpanSet` instance.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::number::float_span_set::FloatSpanSet;
                /// # use std::str::FromStr;
                /// # use meos::collections::base::span_set::SpanSet;
                ///
                /// let span = FloatSpanSet::from_str("{[17.5, 18.5), [19.5, 20.5)}").unwrap();
                /// let scaled_span = span.scale(2.0);
                ///
                /// let expected_scaled_span =
                ///     FloatSpanSet::from_str("{[17.5, 18.1666666666666666666666), [18.833333333333333333333, 19.5)}").unwrap();
                /// assert_eq!(scaled_span, expected_scaled_span);
                /// ```
                fn scale(&self, width: f64) -> FloatSpanSet {
                    self.shift_scale(None, Some(width))
                }
                /// Return a new `FloatSpanSet` with the lower and upper bounds shifted by `delta` and scaled so that the width is `width`.
                ///
                /// ## Arguments
                /// * `delta` - The value to shift by.
                /// * `width` - The new width.
                ///
                /// ## Returns
                /// A new `FloatSpanSet` instance.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::number::float_span_set::FloatSpanSet;
                /// # use std::str::FromStr;
                /// # use meos::collections::base::span_set::SpanSet;
                ///
                /// let span = FloatSpanSet::from_str("{[17.5, 18.5), [19.5, 20.5)}").unwrap();
                /// let shifted_scaled_span = span.shift_scale(Some(5.0), Some(2.5));
                ///
                /// let expected_shifted_scaled_span =
                ///     FloatSpanSet::from_str("{[22.5, 23.3333333333333333333), [24.16666666666666666, 25)}").unwrap();
                /// assert_eq!(shifted_scaled_span, expected_shifted_scaled_span);
                /// ```
                fn shift_scale(
                    &self,
                    delta: Option<f64>,
                    width: Option<f64>,
                ) -> FloatSpanSet {
                    let d = delta.unwrap_or(0.0);
                    let w = width.unwrap_or(0.0);
                    let modified = unsafe {
                        meos_sys::floatspanset_shift_scale(
                            self._inner,
                            d,
                            w,
                            delta.is_some(),
                            width.is_some(),
                        )
                    };
                    FloatSpanSet::from_inner(modified)
                }
                /// Calculates the distance between this `FloatSpanSet` and an integer (`value`).
                ///
                /// ## Arguments
                /// * `value` - An f64 to calculate the distance to.
                ///
                /// ## Returns
                /// An `f64` representing the distance between the span set and the value.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::number::float_span_set::FloatSpanSet;
                /// # use meos::collections::base::span_set::SpanSet;
                /// let span_set: FloatSpanSet = [(2019.0..2023.5).into(), (2029.0..2030.5).into()].iter().collect();
                /// let distance = span_set.distance_to_value(&2032.5);
                /// assert_eq!(distance, 2.0);
                /// ```
                fn distance_to_value(&self, other: &Self::Type) -> f64 {
                    unsafe { meos_sys::distance_spanset_float(self.inner(), *other) }
                }
                /// Calculates the distance between this `FloatSpanSet` and another `FloatSpanSet`.
                ///
                /// ## Arguments
                /// * `other` - An `FloatSpanSet` to calculate the distance to.
                ///
                /// ## Returns
                /// An `f64` representing the distance between the two spansets.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::number::float_span_set::FloatSpanSet;
                /// # use meos::collections::base::span_set::SpanSet;
                /// # use meos::collections::base::span::Span;
                ///
                /// let span_set1: FloatSpanSet = [(2019.0..2023.5).into(), (2029.0..2030.5).into()].iter().collect();
                /// let span_set2: FloatSpanSet = [(2049.0..2050.5).into(), (2059.0..2600.5).into()].iter().collect();
                /// let distance = span_set1.distance_to_span_set(&span_set2);
                ///
                /// assert_eq!(distance, 18.5);
                fn distance_to_span_set(&self, other: &Self) -> f64 {
                    unsafe {
                        meos_sys::distance_floatspanset_floatspanset(
                            self.inner(),
                            other.inner(),
                        )
                    }
                }
                /// Calculates the distance between this `FloatSpanSet` and a `FloatSpan`.
                ///
                /// ## Arguments
                /// * `other` - A `FloatSpan` to calculate the distance to.
                ///
                /// ## Returns
                /// A `TimeDelta` representing the distance in seconds between the span set and the span.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::number::float_span_set::FloatSpanSet;
                /// # use meos::collections::base::span_set::SpanSet;
                /// # use meos::collections::datetime::date_span::DateSpan;
                /// # use meos::collections::base::span::Span;
                /// # use meos::collections::number::float_span::FloatSpan;
                ///
                /// let span_set: FloatSpanSet = [(2019.0..2023.5).into(), (2029.0..2030.5).into()].iter().collect();
                /// let span: FloatSpan = (2009.0..2013.5).into();
                /// let distance = span_set.distance_to_span(&span);
                /// assert_eq!(distance, 5.5);
                /// ```
                fn distance_to_span(&self, span: &Self::SpanType) -> Self::SubsetType {
                    unsafe {
                        meos_sys::distance_floatspanset_floatspan(
                            self.inner(),
                            span.inner(),
                        )
                    }
                }
            }
            impl NumberSpanSet for FloatSpanSet {}
            impl Clone for FloatSpanSet {
                fn clone(&self) -> FloatSpanSet {
                    self.copy()
                }
            }
            impl IntoIterator for FloatSpanSet {
                type Item = <FloatSpanSet as SpanSet>::SpanType;
                type IntoIter = std::vec::IntoIter<Self::Item>;
                fn into_iter(self) -> Self::IntoIter {
                    self.spans().into_iter()
                }
            }
            impl FromIterator<<FloatSpanSet as SpanSet>::SpanType> for FloatSpanSet {
                fn from_iter<
                    T: IntoIterator<Item = <FloatSpanSet as SpanSet>::SpanType>,
                >(iter: T) -> Self {
                    iter.into_iter().collect()
                }
            }
            impl<'a> FromIterator<&'a <FloatSpanSet as SpanSet>::SpanType>
            for FloatSpanSet {
                fn from_iter<
                    T: IntoIterator<Item = &'a <FloatSpanSet as SpanSet>::SpanType>,
                >(iter: T) -> Self {
                    let mut iter = iter.into_iter();
                    let first = iter.next().unwrap();
                    iter.fold(
                        first.to_spanset(),
                        |acc, item| { (acc | item.to_spanset()).unwrap() },
                    )
                }
            }
            impl Hash for FloatSpanSet {
                fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                    let hash = unsafe { meos_sys::spanset_hash(self._inner) };
                    state.write_u32(hash);
                    state.finish();
                }
            }
            impl std::str::FromStr for FloatSpanSet {
                type Err = ParseError;
                fn from_str(string: &str) -> Result<Self, Self::Err> {
                    CString::new(string)
                        .map_err(|_| ParseError)
                        .map(|string| {
                            let inner = unsafe {
                                meos_sys::floatspanset_in(string.as_ptr())
                            };
                            Self::from_inner(inner)
                        })
                }
            }
            impl std::cmp::PartialEq for FloatSpanSet {
                fn eq(&self, other: &Self) -> bool {
                    unsafe { meos_sys::spanset_eq(self._inner, other._inner) }
                }
            }
            impl Debug for FloatSpanSet {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let out_str = unsafe { meos_sys::floatspanset_out(self._inner, 3) };
                    let c_str = unsafe { CStr::from_ptr(out_str) };
                    let str = c_str.to_str().map_err(|_| std::fmt::Error)?;
                    let result = f.write_str(str);
                    unsafe { libc::free(out_str as *mut c_void) };
                    result
                }
            }
            impl BitAnd<FloatSpanSet> for FloatSpanSet {
                type Output = Option<FloatSpanSet>;
                /// Computes the intersection of two `FloatSpanSet`s.
                ///
                /// ## Arguments
                ///
                /// * `other` - Another `FloatSpanSet` to intersect with.
                ///
                /// ## Returns
                ///
                /// * `Some(FloatSpanSet)` - A new `FloatSpanSet` containing the intersection, if it exists.
                /// * `None` - If the intersection is empty.
                ///
                /// ## Example
                ///
                /// ```
                /// # use meos::collections::number::float_span_set::FloatSpanSet;
                /// # use std::str::FromStr;
                /// # use meos::collections::base::span_set::SpanSet;
                ///
                /// let span_set1 = FloatSpanSet::from_str("{[17.5, 18.5), [19.5, 20.5)}").unwrap();
                /// let span_set2 = FloatSpanSet::from_str("{[19.5, 23.5), [45.5, 67.5)}").unwrap();
                ///
                /// let expected_result = FloatSpanSet::from_str("{[19.5, 20.5)}").unwrap();
                /// assert_eq!((span_set1 & span_set2).unwrap(), expected_result);
                /// ```
                fn bitand(self, other: FloatSpanSet) -> Self::Output {
                    self.intersection(&other)
                }
            }
            impl BitOr for FloatSpanSet {
                type Output = Option<FloatSpanSet>;
                /// Computes the union of two `FloatSpanSet`s.
                ///
                /// ## Arguments
                ///
                /// * `other` - Another `FloatSpanSet` to union with.
                ///
                /// ## Returns
                ///
                /// * `Some(FloatSpanSet)` - A new `FloatSpanSet` containing the union.
                /// * `None` - If the union is empty.
                ///
                /// ## Example
                ///
                /// ```
                /// # use meos::collections::number::float_span_set::FloatSpanSet;
                /// # use std::str::FromStr;
                /// # use meos::collections::base::span_set::SpanSet;
                ///
                /// let span_set1 = FloatSpanSet::from_str("{[17.5, 18.5), [19.5, 20.5)}").unwrap();
                /// let span_set2 = FloatSpanSet::from_str("{[19.5, 23.5), [45.5, 67.5)}").unwrap();
                ///
                /// let expected_result = FloatSpanSet::from_str("{[17.5, 18.5), [19.5, 23.5), [45.5, 67.5)}").unwrap();
                /// assert_eq!((span_set1 | span_set2).unwrap(), expected_result)
                /// ```
                fn bitor(self, other: Self) -> Self::Output {
                    self.union(&other)
                }
            }
        }
        pub mod int_span {
            use std::{
                cmp, ffi::{c_void, CStr, CString},
                fmt::Debug, hash::Hash, ops::{BitAnd, Range, RangeInclusive},
            };
            use collection::{impl_collection, Collection};
            use span::Span;
            use crate::{collections::base::*, errors::ParseError};
            use super::number_span::NumberSpan;
            pub struct IntSpan {
                _inner: *const meos_sys::Span,
            }
            impl Drop for IntSpan {
                fn drop(&mut self) {
                    unsafe {
                        libc::free(self._inner as *mut c_void);
                    }
                }
            }
            impl Collection for IntSpan {
                type Type = i32;
                fn is_contained_in(&self, container: &Self) -> bool {
                    unsafe {
                        meos_sys::contained_span_span(self.inner(), container.inner())
                    }
                }
                fn overlaps(&self, other: &Self) -> bool {
                    unsafe { meos_sys::overlaps_span_span(self.inner(), other.inner()) }
                }
                fn is_left(&self, other: &Self) -> bool {
                    unsafe { meos_sys::left_span_span(self.inner(), other.inner()) }
                }
                fn is_over_or_left(&self, other: &Self) -> bool {
                    unsafe { meos_sys::overleft_span_span(self.inner(), other.inner()) }
                }
                fn is_over_or_right(&self, other: &Self) -> bool {
                    unsafe { meos_sys::overright_span_span(self.inner(), other.inner()) }
                }
                fn is_right(&self, other: &Self) -> bool {
                    unsafe { meos_sys::right_span_span(self.inner(), other.inner()) }
                }
                fn is_adjacent(&self, other: &Self) -> bool {
                    unsafe { meos_sys::adjacent_span_span(self.inner(), other.inner()) }
                }
                fn contains(&self, content: &i32) -> bool {
                    unsafe { meos_sys::contains_span_int(self.inner(), *content) }
                }
            }
            impl span::Span for IntSpan {
                type SubsetType = Self::Type;
                fn inner(&self) -> *const meos_sys::Span {
                    self._inner
                }
                /// Creates a new `IntSpan` from an inner pointer to a `meos_sys::Span`.
                ///
                /// # Arguments
                /// * `inner` - A pointer to the inner `meos_sys::Span`.
                ///
                /// ## Returns
                /// * A new `IntSpan` instance.
                fn from_inner(inner: *const meos_sys::Span) -> Self {
                    Self { _inner: inner }
                }
                /// Returns the lower bound of the span.
                ///
                /// ## Returns
                /// * The lower bound as a `i32`.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::number::int_span::IntSpan;
                /// # use meos::collections::base::span::Span;
                ///
                /// let span: IntSpan = (12..67).into();
                /// let lower = span.lower();
                /// ```
                fn lower(&self) -> Self::Type {
                    unsafe { meos_sys::intspan_lower(self.inner()) }
                }
                /// Returns the upper bound of the span.
                ///
                /// ## Returns
                /// * The upper bound as a `i32`.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::number::int_span::IntSpan;
                /// # use meos::collections::base::span::Span;
                ///
                /// let span: IntSpan = (12..67).into();;
                ///
                /// assert_eq!(span.upper(), 67)
                /// ```
                fn upper(&self) -> Self::Type {
                    unsafe { meos_sys::intspan_upper(self.inner()) }
                }
                /// Return a new `IntSpan` with the lower and upper bounds shifted by `delta`.
                ///
                /// # Arguments
                /// * `delta` - The value to shift by.
                ///
                /// # Returns
                /// A new `IntSpan` instance.
                ///
                /// # Example
                /// ```
                /// # use meos::collections::number::int_span::IntSpan;
                /// # use meos::collections::base::span::Span;
                ///
                /// let span: IntSpan = (12..67).into();
                /// let shifted_span = span.shift(5);
                ///
                /// assert_eq!(shifted_span, (17..72).into())
                /// ```
                fn shift(&self, delta: i32) -> IntSpan {
                    self.shift_scale(Some(delta), None)
                }
                /// Return a new `IntSpan` with the lower and upper bounds scaled so that the width is `width`.
                ///
                /// # Arguments
                /// * `width` - The new width.
                ///
                /// # Returns
                /// A new `IntSpan` instance.
                ///
                /// # Example
                /// ```
                /// # use meos::collections::number::int_span::IntSpan;
                /// # use meos::collections::base::span::Span;
                ///
                /// let span: IntSpan = (12..67).into();
                /// let scaled_span = span.scale(10);
                ///
                /// assert_eq!(scaled_span, (12..23).into())
                /// ```
                fn scale(&self, width: i32) -> IntSpan {
                    self.shift_scale(None, Some(width))
                }
                /// Return a new `IntSpan` with the lower and upper bounds shifted by `delta` and scaled so that the width is `width`.
                ///
                /// # Arguments
                /// * `delta` - The value to shift by.
                /// * `width` - The new width.
                ///
                /// # Returns
                /// A new `IntSpan` instance.
                ///
                /// # Example
                /// ```
                /// # use meos::collections::number::int_span::IntSpan;
                /// # use meos::collections::base::span::Span;
                ///
                /// let span: IntSpan = (12..67).into();
                /// let shifted_scaled_span = span.shift_scale(Some(5), Some(10));
                ///
                /// assert_eq!(shifted_scaled_span, (17..28).into())
                /// ```
                fn shift_scale(
                    &self,
                    delta: Option<i32>,
                    width: Option<i32>,
                ) -> IntSpan {
                    let d = delta.unwrap_or(0);
                    let w = width.unwrap_or(0);
                    let modified = unsafe {
                        meos_sys::intspan_shift_scale(
                            self._inner,
                            d,
                            w,
                            delta.is_some(),
                            width.is_some(),
                        )
                    };
                    IntSpan::from_inner(modified)
                }
                /// Calculates the distance between this `IntSpan` and an int.
                ///
                /// ## Arguments
                /// * `value` - An `i32` to calculate the distance to.
                ///
                /// ## Returns
                /// An `i32` representing the distance between the span and the value.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::number::int_span::IntSpan;
                /// # use meos::collections::base::span::Span;
                ///
                /// let span: IntSpan = (12..67).into();
                /// let distance = span.distance_to_value(&8);
                ///
                /// assert_eq!(distance, 4);
                /// ```
                fn distance_to_value(&self, value: &i32) -> i32 {
                    unsafe { meos_sys::distance_span_int(self.inner(), *value) }
                }
                /// Calculates the distance between this `IntSpan` and another `IntSpan`.
                ///
                /// ## Arguments
                /// * `other` - An `IntSpan` to calculate the distance to.
                ///
                /// ## Returns
                /// An `i32` representing the distance between the two spans.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::number::int_span::IntSpan;
                /// # use meos::collections::base::span::Span;
                ///
                /// let span1: IntSpan = (12..67).into();
                /// let span2: IntSpan = (10..11).into();
                /// let distance = span1.distance_to_span(&span2);
                ///
                /// assert_eq!(distance, 2);
                /// ```
                fn distance_to_span(&self, other: &Self) -> i32 {
                    unsafe {
                        meos_sys::distance_intspan_intspan(self.inner(), other.inner())
                    }
                }
            }
            impl NumberSpan for IntSpan {}
            impl Clone for IntSpan {
                fn clone(&self) -> Self {
                    unsafe { Self::from_inner(meos_sys::span_copy(self._inner)) }
                }
            }
            impl Hash for IntSpan {
                fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                    let hash = unsafe { meos_sys::span_hash(self._inner) };
                    state.write_u32(hash);
                    state.finish();
                }
            }
            impl std::str::FromStr for IntSpan {
                type Err = ParseError;
                /// Parses a `IntSpan` from a string representation.
                ///
                /// ## Arguments
                /// * `string` - A string slice containing the representation.
                ///
                /// ## Returns
                /// * A `IntSpan` instance.
                ///
                /// ## Errors
                /// * Returns `ParseSpanError` if the string cannot be parsed.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::number::int_span::IntSpan;
                /// # use meos::collections::base::span::Span;
                /// # use std::str::FromStr;
                ///
                /// let span: IntSpan = "(12, 67)".parse().expect("Failed to parse span");
                /// assert_eq!(span.lower(), 13);
                /// assert_eq!(span.upper(), 67);
                /// ```
                fn from_str(string: &str) -> Result<Self, Self::Err> {
                    CString::new(string)
                        .map_err(|_| ParseError)
                        .map(|string| {
                            let inner = unsafe { meos_sys::intspan_in(string.as_ptr()) };
                            Self::from_inner(inner)
                        })
                }
            }
            impl cmp::PartialEq for IntSpan {
                /// Checks if two `IntSpan` instances are equal.
                ///
                /// # Arguments
                /// * `other` - Another `IntSpan` instance.
                ///
                /// ## Returns
                /// * `true` if the spans are equal, `false` otherwise.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::number::int_span::IntSpan;
                /// # use meos::collections::base::span::Span;
                /// # use std::str::FromStr;
                ///
                /// let span1: IntSpan = (12..67).into();
                /// let span2: IntSpan = (12..67).into();
                /// assert_eq!(span1, span2);
                /// ```
                fn eq(&self, other: &Self) -> bool {
                    unsafe { meos_sys::span_eq(self._inner, other._inner) }
                }
            }
            impl cmp::Eq for IntSpan {}
            impl From<Range<i32>> for IntSpan {
                fn from(Range { start, end }: Range<i32>) -> Self {
                    let inner = unsafe {
                        meos_sys::intspan_make(start, end, true, false)
                    };
                    Self::from_inner(inner)
                }
            }
            impl From<RangeInclusive<i32>> for IntSpan {
                fn from(range: RangeInclusive<i32>) -> Self {
                    let inner = unsafe {
                        meos_sys::intspan_make(*range.start(), *range.end(), true, true)
                    };
                    Self::from_inner(inner)
                }
            }
            impl From<RangeInclusive<f32>> for IntSpan {
                fn from(range: RangeInclusive<f32>) -> Self {
                    let inner = unsafe {
                        meos_sys::intspan_make(
                            *range.start() as i32,
                            *range.end() as i32,
                            true,
                            true,
                        )
                    };
                    Self::from_inner(inner)
                }
            }
            impl Debug for IntSpan {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let out_str = unsafe { meos_sys::intspan_out(self._inner) };
                    let c_str = unsafe { CStr::from_ptr(out_str) };
                    let str = c_str.to_str().map_err(|_| std::fmt::Error)?;
                    let result = f.write_str(str);
                    unsafe { libc::free(out_str as *mut c_void) };
                    result
                }
            }
            impl BitAnd for IntSpan {
                type Output = Option<IntSpan>;
                /// Computes the intersection of two `IntSpan` instances.
                ///
                /// # Arguments
                /// * `other` - Another `IntSpan` instance.
                ///
                /// ## Returns
                /// * An `Option<IntSpan>` containing the intersection, or `None` if there is no intersection.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::number::int_span::IntSpan;
                /// # use meos::collections::base::span::Span;
                /// # use std::str::FromStr;
                ///
                /// let span1: IntSpan = (12..67).into();
                /// let span2: IntSpan = (50..90).into();
                /// let intersection = (span1 & span2).unwrap();
                ///
                /// assert_eq!(intersection, (50..67).into())
                /// ```
                fn bitand(self, other: Self) -> Self::Output {
                    self.intersection(&other)
                }
            }
            impl PartialOrd for IntSpan {
                fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
                    let cmp = unsafe { meos_sys::span_cmp(self._inner, other._inner) };
                    match cmp {
                        -1 => Some(cmp::Ordering::Less),
                        0 => Some(cmp::Ordering::Equal),
                        1 => Some(cmp::Ordering::Greater),
                        _ => None,
                    }
                }
            }
            impl Ord for IntSpan {
                fn cmp(&self, other: &Self) -> cmp::Ordering {
                    self.partial_cmp(other)
                        .expect(
                            "Unreachable since for non-null and same types spans, we only return -1, 0, or 1",
                        )
                }
            }
        }
        pub mod int_span_set {
            use std::ffi::{c_void, CStr, CString};
            use std::fmt::Debug;
            use std::hash::Hash;
            use std::ops::{BitAnd, BitOr};
            use collection::{impl_collection, Collection};
            use span::Span;
            use span_set::impl_iterator;
            use crate::collections::base::span_set::SpanSet;
            use crate::collections::base::*;
            use crate::errors::ParseError;
            use super::int_span::IntSpan;
            use super::number_span_set::NumberSpanSet;
            pub struct IntSpanSet {
                _inner: *const meos_sys::SpanSet,
            }
            impl Drop for IntSpanSet {
                fn drop(&mut self) {
                    unsafe {
                        libc::free(self._inner as *mut c_void);
                    }
                }
            }
            impl Collection for IntSpanSet {
                type Type = i32;
                fn is_contained_in(&self, container: &Self) -> bool {
                    unsafe {
                        meos_sys::contained_spanset_spanset(
                            self.inner(),
                            container.inner(),
                        )
                    }
                }
                fn overlaps(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::overlaps_spanset_spanset(self.inner(), other.inner())
                    }
                }
                fn is_left(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::left_spanset_spanset(self.inner(), other.inner())
                    }
                }
                fn is_over_or_left(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::overleft_spanset_spanset(self.inner(), other.inner())
                    }
                }
                fn is_over_or_right(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::overright_spanset_spanset(self.inner(), other.inner())
                    }
                }
                fn is_right(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::right_spanset_spanset(self.inner(), other.inner())
                    }
                }
                fn is_adjacent(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::adjacent_spanset_spanset(self.inner(), other.inner())
                    }
                }
                fn contains(&self, content: &i32) -> bool {
                    unsafe { meos_sys::contains_spanset_int(self.inner(), *content) }
                }
            }
            impl span_set::SpanSet for IntSpanSet {
                type SpanType = IntSpan;
                type SubsetType = <Self as Collection>::Type;
                fn inner(&self) -> *const meos_sys::SpanSet {
                    self._inner
                }
                fn from_inner(inner: *const meos_sys::SpanSet) -> Self
                where
                    Self: Sized,
                {
                    Self { _inner: inner }
                }
                fn width(&self, ignore_gaps: bool) -> Self::Type {
                    unsafe { meos_sys::intspanset_width(self.inner(), ignore_gaps) }
                }
                /// Return a new `IntSpanSet` with the lower and upper bounds shifted by `delta`.
                ///
                /// ## Arguments
                /// * `delta` - The value to shift by.
                ///
                /// ## Returns
                /// A new `IntSpanSet` instance.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::number::int_span_set::IntSpanSet;
                /// # use std::str::FromStr;
                /// # use meos::collections::base::span_set::SpanSet;
                ///
                /// let span = IntSpanSet::from_str("{[17, 18), [19, 20)}").unwrap();
                /// let shifted_span = span.shift(5);
                ///
                /// let expected_shifted_span =
                ///     IntSpanSet::from_str("{[22, 23), [24, 25)}").unwrap();
                /// assert_eq!(shifted_span, expected_shifted_span);
                /// ```
                fn shift(&self, delta: i32) -> IntSpanSet {
                    self.shift_scale(Some(delta), None)
                }
                /// Return a new `IntSpanSet` with the lower and upper bounds scaled so that the width is `width`.
                ///
                /// ## Arguments
                /// * `width` - The new width.
                ///
                /// ## Returns
                /// A new `IntSpanSet` instance.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::number::int_span_set::IntSpanSet;
                /// # use std::str::FromStr;
                /// # use meos::collections::base::span_set::SpanSet;
                ///
                /// let span = IntSpanSet::from_str("{[17, 18), [19, 23)}").unwrap();
                /// let scaled_span = span.scale(5);
                ///
                /// let expected_scaled_span =
                ///     IntSpanSet::from_str("{[17, 18), [19, 23)}").unwrap();
                /// assert_eq!(scaled_span, expected_scaled_span);
                /// ```
                fn scale(&self, width: i32) -> IntSpanSet {
                    self.shift_scale(None, Some(width))
                }
                /// Return a new `IntSpanSet` with the lower and upper bounds shifted by `delta` and scaled so that the width is `width`.
                ///
                /// ## Arguments
                /// * `delta` - The value to shift by.
                /// * `width` - The new width.
                ///
                /// ## Returns
                /// A new `IntSpanSet` instance.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::number::int_span_set::IntSpanSet;
                /// # use std::str::FromStr;
                /// # use meos::collections::base::span_set::SpanSet;
                ///
                /// let span = IntSpanSet::from_str("{[17, 18), [19, 20)}").unwrap();
                /// let shifted_scaled_span = span.shift_scale(Some(5), Some(2));
                ///
                /// let expected_shifted_scaled_span =
                ///     IntSpanSet::from_str("{[22, 23), [24, 25)}").unwrap();
                /// assert_eq!(shifted_scaled_span, expected_shifted_scaled_span);
                /// ```
                fn shift_scale(
                    &self,
                    delta: Option<i32>,
                    width: Option<i32>,
                ) -> IntSpanSet {
                    let d = delta.unwrap_or(0);
                    let w = width.unwrap_or(0);
                    let modified = unsafe {
                        meos_sys::intspanset_shift_scale(
                            self._inner,
                            d,
                            w,
                            delta.is_some(),
                            width.is_some(),
                        )
                    };
                    IntSpanSet::from_inner(modified)
                }
                /// Calculates the distance between this `IntSpanSet` and an integer (`value`).
                ///
                /// ## Arguments
                /// * `value` - An i32 to calculate the distance to.
                ///
                /// ## Returns
                /// An `i32` representing the distance between the span set and the value.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::number::int_span_set::IntSpanSet;
                /// # use meos::collections::base::span_set::SpanSet;
                /// let span_set: IntSpanSet = [(2019..2023).into(), (2029..2030).into()].iter().collect();
                /// let distance = span_set.distance_to_value(&2032);
                /// assert_eq!(distance, 3);
                /// ```
                fn distance_to_value(&self, value: &Self::Type) -> i32 {
                    unsafe { meos_sys::distance_spanset_int(self.inner(), *value) }
                }
                /// Calculates the distance between this `IntSpanSet` and another `IntSpanSet`.
                ///
                /// ## Arguments
                /// * `other` - An `IntSpanSet` to calculate the distance to.
                ///
                /// ## Returns
                /// An `i32` representing the distance between the two spansets.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::number::int_span_set::IntSpanSet;
                /// # use meos::collections::base::span_set::SpanSet;
                /// # use meos::collections::base::span::Span;
                ///
                /// let span_set1: IntSpanSet = [(2019..2023).into(), (2029..2030).into()].iter().collect();
                /// let span_set2: IntSpanSet = [(2049..2050).into(), (2059..2600).into()].iter().collect();
                /// let distance = span_set1.distance_to_span_set(&span_set2);
                ///
                /// assert_eq!(distance, 20);
                /// ```
                fn distance_to_span_set(&self, other: &Self) -> i32 {
                    unsafe {
                        meos_sys::distance_intspanset_intspanset(
                            self.inner(),
                            other.inner(),
                        )
                    }
                }
                /// Calculates the distance between this `IntSpanSet` and a `IntSpan`.
                ///
                /// ## Arguments
                /// * `other` - A `IntSpan` to calculate the distance to.
                ///
                /// ## Returns
                /// A `TimeDelta` representing the distance in seconds between the span set and the span.
                ///
                /// ## Example
                /// ```
                /// # use meos::collections::number::int_span_set::IntSpanSet;
                /// # use meos::collections::base::span_set::SpanSet;
                /// # use meos::collections::base::span::Span;
                /// # use meos::collections::number::int_span::IntSpan;
                ///
                /// let span_set: IntSpanSet = [(2019..2023).into(), (2029..2030).into()].iter().collect();
                /// let span: IntSpan = (2009..2010).into();
                /// let distance = span_set.distance_to_span(&span);
                /// assert_eq!(distance, 10);
                /// ```
                fn distance_to_span(&self, span: &Self::SpanType) -> Self::SubsetType {
                    unsafe {
                        meos_sys::distance_intspanset_intspan(self.inner(), span.inner())
                    }
                }
            }
            impl NumberSpanSet for IntSpanSet {}
            impl Clone for IntSpanSet {
                fn clone(&self) -> IntSpanSet {
                    self.copy()
                }
            }
            impl IntoIterator for IntSpanSet {
                type Item = <IntSpanSet as SpanSet>::SpanType;
                type IntoIter = std::vec::IntoIter<Self::Item>;
                fn into_iter(self) -> Self::IntoIter {
                    self.spans().into_iter()
                }
            }
            impl FromIterator<<IntSpanSet as SpanSet>::SpanType> for IntSpanSet {
                fn from_iter<T: IntoIterator<Item = <IntSpanSet as SpanSet>::SpanType>>(
                    iter: T,
                ) -> Self {
                    iter.into_iter().collect()
                }
            }
            impl<'a> FromIterator<&'a <IntSpanSet as SpanSet>::SpanType> for IntSpanSet {
                fn from_iter<
                    T: IntoIterator<Item = &'a <IntSpanSet as SpanSet>::SpanType>,
                >(iter: T) -> Self {
                    let mut iter = iter.into_iter();
                    let first = iter.next().unwrap();
                    iter.fold(
                        first.to_spanset(),
                        |acc, item| { (acc | item.to_spanset()).unwrap() },
                    )
                }
            }
            impl Hash for IntSpanSet {
                fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                    let hash = unsafe { meos_sys::spanset_hash(self._inner) };
                    state.write_u32(hash);
                    state.finish();
                }
            }
            impl std::str::FromStr for IntSpanSet {
                type Err = ParseError;
                fn from_str(string: &str) -> Result<Self, Self::Err> {
                    CString::new(string)
                        .map_err(|_| ParseError)
                        .map(|string| {
                            let inner = unsafe {
                                meos_sys::intspanset_in(string.as_ptr())
                            };
                            Self::from_inner(inner)
                        })
                }
            }
            impl std::cmp::PartialEq for IntSpanSet {
                fn eq(&self, other: &Self) -> bool {
                    unsafe { meos_sys::spanset_eq(self._inner, other._inner) }
                }
            }
            impl Debug for IntSpanSet {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let out_str = unsafe { meos_sys::intspanset_out(self._inner) };
                    let c_str = unsafe { CStr::from_ptr(out_str) };
                    let str = c_str.to_str().map_err(|_| std::fmt::Error)?;
                    let result = f.write_str(str);
                    unsafe { libc::free(out_str as *mut c_void) };
                    result
                }
            }
            impl BitAnd<IntSpanSet> for IntSpanSet {
                type Output = Option<IntSpanSet>;
                /// Computes the intersection of two `IntSpanSet`s.
                ///
                /// ## Arguments
                ///
                /// * `other` - Another `IntSpanSet` to intersect with.
                ///
                /// ## Returns
                ///
                /// * `Some(IntSpanSet)` - A new `IntSpanSet` containing the intersection, if it exists.
                /// * `None` - If the intersection is empty.
                ///
                /// ## Example
                ///
                /// ```
                /// # use meos::collections::number::int_span_set::IntSpanSet;
                /// # use std::str::FromStr;
                /// # use meos::collections::base::span_set::SpanSet;
                ///
                /// let span_set1 = IntSpanSet::from_str("{[17, 18), [19, 20)}").unwrap();
                /// let span_set2 = IntSpanSet::from_str("{[19, 23), [45, 67)}").unwrap();
                ///
                /// let expected_result = IntSpanSet::from_str("{[19, 20)}").unwrap();
                /// assert_eq!((span_set1 & span_set2).unwrap(), expected_result);
                /// ```
                fn bitand(self, other: IntSpanSet) -> Self::Output {
                    self.intersection(&other)
                }
            }
            impl BitOr for IntSpanSet {
                type Output = Option<IntSpanSet>;
                /// Computes the union of two `IntSpanSet`s.
                ///
                /// ## Arguments
                ///
                /// * `other` - Another `IntSpanSet` to union with.
                ///
                /// ## Returns
                ///
                /// * `Some(IntSpanSet)` - A new `IntSpanSet` containing the union.
                /// * `None` - If the union is empty.
                ///
                /// ## Example
                ///
                /// ```
                /// # use meos::collections::number::int_span_set::IntSpanSet;
                /// # use std::str::FromStr;
                /// # use meos::collections::base::span_set::SpanSet;
                ///
                /// let span_set1 = IntSpanSet::from_str("{[17, 18), [19, 20)}").unwrap();
                /// let span_set2 = IntSpanSet::from_str("{[19, 23), [45, 67)}").unwrap();
                ///
                /// let expected_result = IntSpanSet::from_str("{[17, 18), [19, 23), [45, 67)}").unwrap();
                /// assert_eq!((span_set1 | span_set2).unwrap(), expected_result)
                /// ```
                fn bitor(self, other: Self) -> Self::Output {
                    self.union(&other)
                }
            }
        }
    }
}
pub mod errors {
    pub struct ParseError;
    #[automatically_derived]
    impl ::core::fmt::Debug for ParseError {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(f, "ParseError")
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for ParseError {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for ParseError {
        #[inline]
        fn eq(&self, other: &ParseError) -> bool {
            true
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for ParseError {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
}
pub mod temporal {
    pub mod interpolation {
        use std::fmt;
        use std::str::FromStr;
        use crate::errors::ParseError;
        /// Enum representing the different types of interpolation.
        pub enum TInterpolation {
            None = meos_sys::interpType_INTERP_NONE as isize,
            Discrete = meos_sys::interpType_DISCRETE as isize,
            Stepwise = meos_sys::interpType_STEP as isize,
            Linear = meos_sys::interpType_LINEAR as isize,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for TInterpolation {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(
                    f,
                    match self {
                        TInterpolation::None => "None",
                        TInterpolation::Discrete => "Discrete",
                        TInterpolation::Stepwise => "Stepwise",
                        TInterpolation::Linear => "Linear",
                    },
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for TInterpolation {
            #[inline]
            fn clone(&self) -> TInterpolation {
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for TInterpolation {}
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for TInterpolation {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for TInterpolation {
            #[inline]
            fn eq(&self, other: &TInterpolation) -> bool {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                __self_discr == __arg1_discr
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Eq for TInterpolation {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_receiver_is_total_eq(&self) -> () {}
        }
        #[automatically_derived]
        impl ::core::hash::Hash for TInterpolation {
            #[inline]
            fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                ::core::hash::Hash::hash(&__self_discr, state)
            }
        }
        impl FromStr for TInterpolation {
            type Err = ParseError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.to_lowercase().as_str() {
                    "none" => Ok(TInterpolation::None),
                    "discrete" => Ok(TInterpolation::Discrete),
                    "linear" => Ok(TInterpolation::Linear),
                    "stepwise" | "step" => Ok(TInterpolation::Stepwise),
                    _ => Err(ParseError),
                }
            }
        }
        impl fmt::Display for TInterpolation {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_fmt(format_args!("{0:?}", self))
            }
        }
    }
    pub mod number {
        pub mod tfloat {
            use std::{
                ffi::{c_void, CStr, CString},
                fmt::Debug, hash::Hash, ptr, str::FromStr,
            };
            use chrono::{DateTime, TimeZone};
            use crate::{
                boxes::tbox::TBox,
                collections::{
                    base::{
                        collection::{impl_collection, Collection},
                        span::Span, span_set::SpanSet,
                    },
                    datetime::{tstz_span::TsTzSpan, tstz_span_set::TsTzSpanSet},
                    number::float_span_set::FloatSpanSet,
                },
                errors::ParseError,
                temporal::{
                    interpolation::TInterpolation,
                    temporal::{impl_simple_traits_for_temporal, Temporal},
                    tinstant::TInstant, tsequence::TSequence, tsequence_set::TSequenceSet,
                },
                utils::to_meos_timestamp,
            };
            use super::tnumber::{impl_temporal_for_tnumber, TNumber};
            pub trait TFloat: Temporal<
                    Type = f64,
                    TI = TFloatInst,
                    TS = TFloatSeq,
                    TSS = TFloatSeqSet,
                    TBB = TBox,
                > {
                /// Returns a new `TNumber` with the value dimension shifted by `shift` and scaled so the value dimension has width `width`.
                ///
                /// # Arguments
                /// * `shift` - Value to shift
                /// * `width` - Value representing the width of the new temporal number
                ///
                /// # Safety
                /// This function uses unsafe code to call the `meos_sys::tfloat_shift_scale_value` or
                /// `meos_sys::tfloat_shift_scale_value` functions.
                fn shift_scale_value(
                    &self,
                    shift: Option<Self::Type>,
                    width: Option<Self::Type>,
                ) -> Self {
                    let d = shift.unwrap_or_default();
                    let w = width.unwrap_or_default();
                    let modified = unsafe {
                        meos_sys::tfloat_shift_scale_value(self.inner(), d, w)
                    };
                    Self::from_inner_as_temporal(modified)
                }
            }
            pub struct TFloatInst {
                _inner: *const meos_sys::TInstant,
            }
            impl TInstant for TFloatInst {
                fn from_inner(inner: *mut meos_sys::TInstant) -> Self {
                    Self { _inner: inner }
                }
                fn inner_as_tinstant(&self) -> *const meos_sys::TInstant {
                    self._inner
                }
                fn from_value_and_timestamp<Tz: TimeZone>(
                    value: Self::Type,
                    timestamp: DateTime<Tz>,
                ) -> Self {
                    Self::from_inner(unsafe {
                        meos_sys::tfloatinst_make(value, to_meos_timestamp(&timestamp))
                    })
                }
            }
            impl TFloat for TFloatInst {}
            impl Collection for TFloatInst {
                type Type = f64;
                fn is_contained_in(&self, container: &Self) -> bool {
                    unsafe {
                        meos_sys::contained_tnumber_tnumber(
                            self.inner(),
                            container.inner(),
                        )
                    }
                }
                fn overlaps(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::overlaps_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn is_left(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::left_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn is_over_or_left(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::overleft_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn is_over_or_right(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::overright_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn is_right(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::right_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn is_adjacent(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::adjacent_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn contains(&self, content: &Self::Type) -> bool {
                    FloatSpanSet::from_inner(unsafe {
                            meos_sys::tnumber_valuespans(self.inner())
                        })
                        .contains(content)
                }
            }
            impl Clone for TFloatInst {
                fn clone(&self) -> Self {
                    Temporal::from_inner_as_temporal(unsafe {
                        meos_sys::temporal_copy(self.inner())
                    })
                }
            }
            impl FromStr for TFloatInst {
                type Err = ParseError;
                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    CString::new(s)
                        .map_err(|_| ParseError)
                        .map(|string| {
                            let inner = unsafe { meos_sys::tfloat_in(string.as_ptr()) };
                            Self::from_inner_as_temporal(inner)
                        })
                }
            }
            impl PartialEq for TFloatInst {
                fn eq(&self, other: &Self) -> bool {
                    unsafe { meos_sys::temporal_eq(self.inner(), other.inner()) }
                }
            }
            impl Hash for TFloatInst {
                fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                    let hash = unsafe { meos_sys::temporal_hash(self.inner()) };
                    state.write_u32(hash);
                    state.finish();
                }
            }
            impl TNumber for TFloatInst {
                fn nearest_approach_distance(&self, other: &Self) -> Self::Type {
                    unsafe { meos_sys::nad_tfloat_tfloat(self.inner(), other.inner()) }
                }
            }
            impl Temporal for TFloatInst {
                type TI = TFloatInst;
                type TS = TFloatSeq;
                type TSS = TFloatSeqSet;
                type TBB = TBox;
                fn from_inner_as_temporal(inner: *const meos_sys::Temporal) -> Self {
                    Self {
                        _inner: inner as *const meos_sys::TInstant,
                    }
                }
                fn from_mfjson(mfjson: &str) -> Self {
                    let cstr = CString::new(mfjson).unwrap();
                    Self::from_inner_as_temporal(unsafe {
                        meos_sys::tfloat_from_mfjson(cstr.as_ptr())
                    })
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
                        let values = meos_sys::tfloat_values(
                            self.inner(),
                            &raw mut count,
                        );
                        Vec::from_raw_parts(values, count as usize, count as usize)
                    }
                }
                fn start_value(&self) -> Self::Type {
                    unsafe { meos_sys::tfloat_start_value(self.inner()) }
                }
                fn end_value(&self) -> Self::Type {
                    unsafe { meos_sys::tfloat_end_value(self.inner()) }
                }
                fn min_value(&self) -> Self::Type {
                    unsafe { meos_sys::tfloat_min_value(self.inner()) }
                }
                fn max_value(&self) -> Self::Type {
                    unsafe { meos_sys::tfloat_max_value(self.inner()) }
                }
                fn value_at_timestamp<Tz: TimeZone>(
                    &self,
                    timestamp: DateTime<Tz>,
                ) -> Option<Self::Type> {
                    let mut result = 0.into();
                    unsafe {
                        let success = meos_sys::tfloat_value_at_timestamptz(
                            self.inner(),
                            to_meos_timestamp(&timestamp),
                            true,
                            &raw mut result,
                        );
                        if success { Some(result) } else { None }
                    }
                }
                fn at_value(&self, value: &Self::Type) -> Option<Self> {
                    let result = unsafe {
                        meos_sys::tfloat_at_value(self.inner(), *value)
                    };
                    if result != ptr::null_mut() {
                        Some(Self::from_inner_as_temporal(result))
                    } else {
                        None
                    }
                }
                fn at_values(&self, values: &[Self::Type]) -> Option<Self> {
                    unsafe {
                        let set = meos_sys::floatset_make(
                            values.as_ptr(),
                            values.len() as i32,
                        );
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
                        meos_sys::tfloat_minus_value(self.inner(), value)
                    })
                }
                fn minus_values(&self, values: &[Self::Type]) -> Self {
                    Self::from_inner_as_temporal(unsafe {
                        let set = meos_sys::floatset_make(
                            values.as_ptr(),
                            values.len() as i32,
                        );
                        meos_sys::temporal_minus_values(self.inner(), set)
                    })
                }
            }
            impl Debug for TFloatInst {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let out_str = unsafe { meos_sys::tfloat_out(self.inner(), 3) };
                    let c_str = unsafe { CStr::from_ptr(out_str) };
                    let str = c_str.to_str().map_err(|_| std::fmt::Error)?;
                    let result = f.write_str(str);
                    unsafe { libc::free(out_str as *mut c_void) };
                    result
                }
            }
            pub struct TFloatSeq {
                _inner: *const meos_sys::TSequence,
            }
            impl TFloatSeq {
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
                        meos_sys::tfloatseq_from_base_tstzspan(
                            value,
                            time_span.inner(),
                            interpolation as u32,
                        )
                    })
                }
            }
            impl TSequence for TFloatSeq {
                fn from_inner(inner: *const meos_sys::TSequence) -> Self {
                    Self { _inner: inner }
                }
                fn inner_as_tsequence(&self) -> *const meos_sys::TSequence {
                    self._inner
                }
            }
            impl TFloat for TFloatSeq {}
            impl Collection for TFloatSeq {
                type Type = f64;
                fn is_contained_in(&self, container: &Self) -> bool {
                    unsafe {
                        meos_sys::contained_tnumber_tnumber(
                            self.inner(),
                            container.inner(),
                        )
                    }
                }
                fn overlaps(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::overlaps_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn is_left(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::left_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn is_over_or_left(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::overleft_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn is_over_or_right(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::overright_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn is_right(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::right_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn is_adjacent(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::adjacent_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn contains(&self, content: &Self::Type) -> bool {
                    FloatSpanSet::from_inner(unsafe {
                            meos_sys::tnumber_valuespans(self.inner())
                        })
                        .contains(content)
                }
            }
            impl Clone for TFloatSeq {
                fn clone(&self) -> Self {
                    Temporal::from_inner_as_temporal(unsafe {
                        meos_sys::temporal_copy(self.inner())
                    })
                }
            }
            impl FromStr for TFloatSeq {
                type Err = ParseError;
                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    CString::new(s)
                        .map_err(|_| ParseError)
                        .map(|string| {
                            let inner = unsafe { meos_sys::tfloat_in(string.as_ptr()) };
                            Self::from_inner_as_temporal(inner)
                        })
                }
            }
            impl PartialEq for TFloatSeq {
                fn eq(&self, other: &Self) -> bool {
                    unsafe { meos_sys::temporal_eq(self.inner(), other.inner()) }
                }
            }
            impl Hash for TFloatSeq {
                fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                    let hash = unsafe { meos_sys::temporal_hash(self.inner()) };
                    state.write_u32(hash);
                    state.finish();
                }
            }
            impl TNumber for TFloatSeq {
                fn nearest_approach_distance(&self, other: &Self) -> Self::Type {
                    unsafe { meos_sys::nad_tfloat_tfloat(self.inner(), other.inner()) }
                }
            }
            impl Temporal for TFloatSeq {
                type TI = TFloatInst;
                type TS = TFloatSeq;
                type TSS = TFloatSeqSet;
                type TBB = TBox;
                fn from_inner_as_temporal(inner: *const meos_sys::Temporal) -> Self {
                    Self {
                        _inner: inner as *const meos_sys::TSequence,
                    }
                }
                fn from_mfjson(mfjson: &str) -> Self {
                    let cstr = CString::new(mfjson).unwrap();
                    Self::from_inner_as_temporal(unsafe {
                        meos_sys::tfloat_from_mfjson(cstr.as_ptr())
                    })
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
                        let values = meos_sys::tfloat_values(
                            self.inner(),
                            &raw mut count,
                        );
                        Vec::from_raw_parts(values, count as usize, count as usize)
                    }
                }
                fn start_value(&self) -> Self::Type {
                    unsafe { meos_sys::tfloat_start_value(self.inner()) }
                }
                fn end_value(&self) -> Self::Type {
                    unsafe { meos_sys::tfloat_end_value(self.inner()) }
                }
                fn min_value(&self) -> Self::Type {
                    unsafe { meos_sys::tfloat_min_value(self.inner()) }
                }
                fn max_value(&self) -> Self::Type {
                    unsafe { meos_sys::tfloat_max_value(self.inner()) }
                }
                fn value_at_timestamp<Tz: TimeZone>(
                    &self,
                    timestamp: DateTime<Tz>,
                ) -> Option<Self::Type> {
                    let mut result = 0.into();
                    unsafe {
                        let success = meos_sys::tfloat_value_at_timestamptz(
                            self.inner(),
                            to_meos_timestamp(&timestamp),
                            true,
                            &raw mut result,
                        );
                        if success { Some(result) } else { None }
                    }
                }
                fn at_value(&self, value: &Self::Type) -> Option<Self> {
                    let result = unsafe {
                        meos_sys::tfloat_at_value(self.inner(), *value)
                    };
                    if result != ptr::null_mut() {
                        Some(Self::from_inner_as_temporal(result))
                    } else {
                        None
                    }
                }
                fn at_values(&self, values: &[Self::Type]) -> Option<Self> {
                    unsafe {
                        let set = meos_sys::floatset_make(
                            values.as_ptr(),
                            values.len() as i32,
                        );
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
                        meos_sys::tfloat_minus_value(self.inner(), value)
                    })
                }
                fn minus_values(&self, values: &[Self::Type]) -> Self {
                    Self::from_inner_as_temporal(unsafe {
                        let set = meos_sys::floatset_make(
                            values.as_ptr(),
                            values.len() as i32,
                        );
                        meos_sys::temporal_minus_values(self.inner(), set)
                    })
                }
            }
            impl Debug for TFloatSeq {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let out_str = unsafe { meos_sys::tfloat_out(self.inner(), 3) };
                    let c_str = unsafe { CStr::from_ptr(out_str) };
                    let str = c_str.to_str().map_err(|_| std::fmt::Error)?;
                    let result = f.write_str(str);
                    unsafe { libc::free(out_str as *mut c_void) };
                    result
                }
            }
            pub struct TFloatSeqSet {
                _inner: *const meos_sys::TSequenceSet,
            }
            impl TFloatSeqSet {
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
            impl TSequenceSet for TFloatSeqSet {
                fn from_inner(inner: *const meos_sys::TSequenceSet) -> Self {
                    Self { _inner: inner }
                }
            }
            impl TFloat for TFloatSeqSet {}
            impl Collection for TFloatSeqSet {
                type Type = f64;
                fn is_contained_in(&self, container: &Self) -> bool {
                    unsafe {
                        meos_sys::contained_tnumber_tnumber(
                            self.inner(),
                            container.inner(),
                        )
                    }
                }
                fn overlaps(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::overlaps_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn is_left(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::left_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn is_over_or_left(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::overleft_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn is_over_or_right(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::overright_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn is_right(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::right_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn is_adjacent(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::adjacent_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn contains(&self, content: &Self::Type) -> bool {
                    FloatSpanSet::from_inner(unsafe {
                            meos_sys::tnumber_valuespans(self.inner())
                        })
                        .contains(content)
                }
            }
            impl Clone for TFloatSeqSet {
                fn clone(&self) -> Self {
                    Temporal::from_inner_as_temporal(unsafe {
                        meos_sys::temporal_copy(self.inner())
                    })
                }
            }
            impl FromStr for TFloatSeqSet {
                type Err = ParseError;
                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    CString::new(s)
                        .map_err(|_| ParseError)
                        .map(|string| {
                            let inner = unsafe { meos_sys::tfloat_in(string.as_ptr()) };
                            Self::from_inner_as_temporal(inner)
                        })
                }
            }
            impl PartialEq for TFloatSeqSet {
                fn eq(&self, other: &Self) -> bool {
                    unsafe { meos_sys::temporal_eq(self.inner(), other.inner()) }
                }
            }
            impl Hash for TFloatSeqSet {
                fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                    let hash = unsafe { meos_sys::temporal_hash(self.inner()) };
                    state.write_u32(hash);
                    state.finish();
                }
            }
            impl TNumber for TFloatSeqSet {
                fn nearest_approach_distance(&self, other: &Self) -> Self::Type {
                    unsafe { meos_sys::nad_tfloat_tfloat(self.inner(), other.inner()) }
                }
            }
            impl Temporal for TFloatSeqSet {
                type TI = TFloatInst;
                type TS = TFloatSeq;
                type TSS = TFloatSeqSet;
                type TBB = TBox;
                fn from_inner_as_temporal(inner: *const meos_sys::Temporal) -> Self {
                    Self {
                        _inner: inner as *const meos_sys::TSequenceSet,
                    }
                }
                fn from_mfjson(mfjson: &str) -> Self {
                    let cstr = CString::new(mfjson).unwrap();
                    Self::from_inner_as_temporal(unsafe {
                        meos_sys::tfloat_from_mfjson(cstr.as_ptr())
                    })
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
                        let values = meos_sys::tfloat_values(
                            self.inner(),
                            &raw mut count,
                        );
                        Vec::from_raw_parts(values, count as usize, count as usize)
                    }
                }
                fn start_value(&self) -> Self::Type {
                    unsafe { meos_sys::tfloat_start_value(self.inner()) }
                }
                fn end_value(&self) -> Self::Type {
                    unsafe { meos_sys::tfloat_end_value(self.inner()) }
                }
                fn min_value(&self) -> Self::Type {
                    unsafe { meos_sys::tfloat_min_value(self.inner()) }
                }
                fn max_value(&self) -> Self::Type {
                    unsafe { meos_sys::tfloat_max_value(self.inner()) }
                }
                fn value_at_timestamp<Tz: TimeZone>(
                    &self,
                    timestamp: DateTime<Tz>,
                ) -> Option<Self::Type> {
                    let mut result = 0.into();
                    unsafe {
                        let success = meos_sys::tfloat_value_at_timestamptz(
                            self.inner(),
                            to_meos_timestamp(&timestamp),
                            true,
                            &raw mut result,
                        );
                        if success { Some(result) } else { None }
                    }
                }
                fn at_value(&self, value: &Self::Type) -> Option<Self> {
                    let result = unsafe {
                        meos_sys::tfloat_at_value(self.inner(), *value)
                    };
                    if result != ptr::null_mut() {
                        Some(Self::from_inner_as_temporal(result))
                    } else {
                        None
                    }
                }
                fn at_values(&self, values: &[Self::Type]) -> Option<Self> {
                    unsafe {
                        let set = meos_sys::floatset_make(
                            values.as_ptr(),
                            values.len() as i32,
                        );
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
                        meos_sys::tfloat_minus_value(self.inner(), value)
                    })
                }
                fn minus_values(&self, values: &[Self::Type]) -> Self {
                    Self::from_inner_as_temporal(unsafe {
                        let set = meos_sys::floatset_make(
                            values.as_ptr(),
                            values.len() as i32,
                        );
                        meos_sys::temporal_minus_values(self.inner(), set)
                    })
                }
            }
            impl Debug for TFloatSeqSet {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let out_str = unsafe { meos_sys::tfloat_out(self.inner(), 3) };
                    let c_str = unsafe { CStr::from_ptr(out_str) };
                    let str = c_str.to_str().map_err(|_| std::fmt::Error)?;
                    let result = f.write_str(str);
                    unsafe { libc::free(out_str as *mut c_void) };
                    result
                }
            }
        }
        pub mod tint {
            use std::{
                ffi::{c_void, CStr, CString},
                fmt::Debug, hash::Hash, ptr, str::FromStr,
            };
            use chrono::{DateTime, TimeZone};
            use crate::{
                boxes::tbox::TBox,
                collections::{
                    base::{
                        collection::{impl_collection, Collection},
                        span::Span, span_set::SpanSet,
                    },
                    datetime::{tstz_span::TsTzSpan, tstz_span_set::TsTzSpanSet},
                    number::int_span_set::IntSpanSet,
                },
                errors::ParseError,
                temporal::{
                    temporal::{impl_simple_traits_for_temporal, Temporal},
                    tinstant::TInstant, tsequence::TSequence, tsequence_set::TSequenceSet,
                },
                utils::to_meos_timestamp,
            };
            use super::tnumber::{impl_temporal_for_tnumber, TNumber};
            pub trait TInt: Temporal<
                    Type = i32,
                    TI = TIntInst,
                    TS = TIntSeq,
                    TSS = TIntSeqSet,
                    TBB = TBox,
                > {
                /// Returns a new `TNumber` with the value dimension shifted by `shift` and scaled so the value dimension has width `width`.
                ///
                /// # Arguments
                /// * `shift` - Value to shift
                /// * `width` - Value representing the width of the new temporal number
                ///
                /// # Safety
                /// This function uses unsafe code to call the `meos_sys::tint_shift_scale_value` or
                /// `meos_sys::tfloat_shift_scale_value` functions.
                fn shift_scale_value(
                    &self,
                    shift: Option<Self::Type>,
                    width: Option<Self::Type>,
                ) -> Self {
                    let d = shift.unwrap_or_default();
                    let w = width.unwrap_or_default();
                    let modified = unsafe {
                        meos_sys::tint_shift_scale_value(self.inner(), d, w)
                    };
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
                fn from_value_and_timestamp<Tz: TimeZone>(
                    value: Self::Type,
                    timestamp: DateTime<Tz>,
                ) -> Self {
                    Self::from_inner(unsafe {
                        meos_sys::tintinst_make(value, to_meos_timestamp(&timestamp))
                    })
                }
            }
            impl TInt for TIntInst {}
            impl Collection for TIntInst {
                type Type = i32;
                fn is_contained_in(&self, container: &Self) -> bool {
                    unsafe {
                        meos_sys::contained_tnumber_tnumber(
                            self.inner(),
                            container.inner(),
                        )
                    }
                }
                fn overlaps(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::overlaps_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn is_left(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::left_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn is_over_or_left(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::overleft_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn is_over_or_right(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::overright_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn is_right(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::right_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn is_adjacent(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::adjacent_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn contains(&self, content: &Self::Type) -> bool {
                    IntSpanSet::from_inner(unsafe {
                            meos_sys::tnumber_valuespans(self.inner())
                        })
                        .contains(content)
                }
            }
            impl Clone for TIntInst {
                fn clone(&self) -> Self {
                    Temporal::from_inner_as_temporal(unsafe {
                        meos_sys::temporal_copy(self.inner())
                    })
                }
            }
            impl FromStr for TIntInst {
                type Err = ParseError;
                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    CString::new(s)
                        .map_err(|_| ParseError)
                        .map(|string| {
                            let inner = unsafe { meos_sys::tint_in(string.as_ptr()) };
                            Self::from_inner_as_temporal(inner)
                        })
                }
            }
            impl PartialEq for TIntInst {
                fn eq(&self, other: &Self) -> bool {
                    unsafe { meos_sys::temporal_eq(self.inner(), other.inner()) }
                }
            }
            impl Hash for TIntInst {
                fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                    let hash = unsafe { meos_sys::temporal_hash(self.inner()) };
                    state.write_u32(hash);
                    state.finish();
                }
            }
            impl TNumber for TIntInst {
                fn nearest_approach_distance(&self, other: &Self) -> Self::Type {
                    unsafe { meos_sys::nad_tint_tint(self.inner(), other.inner()) }
                }
            }
            impl Temporal for TIntInst {
                type TI = TIntInst;
                type TS = TIntSeq;
                type TSS = TIntSeqSet;
                type TBB = TBox;
                fn from_inner_as_temporal(inner: *const meos_sys::Temporal) -> Self {
                    Self {
                        _inner: inner as *const meos_sys::TInstant,
                    }
                }
                fn from_mfjson(mfjson: &str) -> Self {
                    let cstr = CString::new(mfjson).unwrap();
                    Self::from_inner_as_temporal(unsafe {
                        meos_sys::tint_from_mfjson(cstr.as_ptr())
                    })
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
                        let values = meos_sys::tint_values(self.inner(), &raw mut count);
                        Vec::from_raw_parts(values, count as usize, count as usize)
                    }
                }
                fn start_value(&self) -> Self::Type {
                    unsafe { meos_sys::tint_start_value(self.inner()) }
                }
                fn end_value(&self) -> Self::Type {
                    unsafe { meos_sys::tint_end_value(self.inner()) }
                }
                fn min_value(&self) -> Self::Type {
                    unsafe { meos_sys::tint_min_value(self.inner()) }
                }
                fn max_value(&self) -> Self::Type {
                    unsafe { meos_sys::tint_max_value(self.inner()) }
                }
                fn value_at_timestamp<Tz: TimeZone>(
                    &self,
                    timestamp: DateTime<Tz>,
                ) -> Option<Self::Type> {
                    let mut result = 0.into();
                    unsafe {
                        let success = meos_sys::tint_value_at_timestamptz(
                            self.inner(),
                            to_meos_timestamp(&timestamp),
                            true,
                            &raw mut result,
                        );
                        if success { Some(result) } else { None }
                    }
                }
                fn at_value(&self, value: &Self::Type) -> Option<Self> {
                    let result = unsafe {
                        meos_sys::tint_at_value(self.inner(), *value)
                    };
                    if result != ptr::null_mut() {
                        Some(Self::from_inner_as_temporal(result))
                    } else {
                        None
                    }
                }
                fn at_values(&self, values: &[Self::Type]) -> Option<Self> {
                    unsafe {
                        let set = meos_sys::intset_make(
                            values.as_ptr(),
                            values.len() as i32,
                        );
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
                        meos_sys::tint_minus_value(self.inner(), value)
                    })
                }
                fn minus_values(&self, values: &[Self::Type]) -> Self {
                    Self::from_inner_as_temporal(unsafe {
                        let set = meos_sys::intset_make(
                            values.as_ptr(),
                            values.len() as i32,
                        );
                        meos_sys::temporal_minus_values(self.inner(), set)
                    })
                }
            }
            impl Debug for TIntInst {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let out_str = unsafe { meos_sys::tint_out(self.inner()) };
                    let c_str = unsafe { CStr::from_ptr(out_str) };
                    let str = c_str.to_str().map_err(|_| std::fmt::Error)?;
                    let result = f.write_str(str);
                    unsafe { libc::free(out_str as *mut c_void) };
                    result
                }
            }
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
                pub fn from_value_and_tstz_span<Tz: TimeZone>(
                    value: i32,
                    time_span: TsTzSpan,
                ) -> Self {
                    Self::from_inner(unsafe {
                        meos_sys::tintseq_from_base_tstzspan(value, time_span.inner())
                    })
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
            impl Collection for TIntSeq {
                type Type = i32;
                fn is_contained_in(&self, container: &Self) -> bool {
                    unsafe {
                        meos_sys::contained_tnumber_tnumber(
                            self.inner(),
                            container.inner(),
                        )
                    }
                }
                fn overlaps(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::overlaps_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn is_left(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::left_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn is_over_or_left(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::overleft_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn is_over_or_right(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::overright_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn is_right(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::right_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn is_adjacent(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::adjacent_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn contains(&self, content: &Self::Type) -> bool {
                    IntSpanSet::from_inner(unsafe {
                            meos_sys::tnumber_valuespans(self.inner())
                        })
                        .contains(content)
                }
            }
            impl Clone for TIntSeq {
                fn clone(&self) -> Self {
                    Temporal::from_inner_as_temporal(unsafe {
                        meos_sys::temporal_copy(self.inner())
                    })
                }
            }
            impl FromStr for TIntSeq {
                type Err = ParseError;
                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    CString::new(s)
                        .map_err(|_| ParseError)
                        .map(|string| {
                            let inner = unsafe { meos_sys::tint_in(string.as_ptr()) };
                            Self::from_inner_as_temporal(inner)
                        })
                }
            }
            impl PartialEq for TIntSeq {
                fn eq(&self, other: &Self) -> bool {
                    unsafe { meos_sys::temporal_eq(self.inner(), other.inner()) }
                }
            }
            impl Hash for TIntSeq {
                fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                    let hash = unsafe { meos_sys::temporal_hash(self.inner()) };
                    state.write_u32(hash);
                    state.finish();
                }
            }
            impl TNumber for TIntSeq {
                fn nearest_approach_distance(&self, other: &Self) -> Self::Type {
                    unsafe { meos_sys::nad_tint_tint(self.inner(), other.inner()) }
                }
            }
            impl Temporal for TIntSeq {
                type TI = TIntInst;
                type TS = TIntSeq;
                type TSS = TIntSeqSet;
                type TBB = TBox;
                fn from_inner_as_temporal(inner: *const meos_sys::Temporal) -> Self {
                    Self {
                        _inner: inner as *const meos_sys::TSequence,
                    }
                }
                fn from_mfjson(mfjson: &str) -> Self {
                    let cstr = CString::new(mfjson).unwrap();
                    Self::from_inner_as_temporal(unsafe {
                        meos_sys::tint_from_mfjson(cstr.as_ptr())
                    })
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
                        let values = meos_sys::tint_values(self.inner(), &raw mut count);
                        Vec::from_raw_parts(values, count as usize, count as usize)
                    }
                }
                fn start_value(&self) -> Self::Type {
                    unsafe { meos_sys::tint_start_value(self.inner()) }
                }
                fn end_value(&self) -> Self::Type {
                    unsafe { meos_sys::tint_end_value(self.inner()) }
                }
                fn min_value(&self) -> Self::Type {
                    unsafe { meos_sys::tint_min_value(self.inner()) }
                }
                fn max_value(&self) -> Self::Type {
                    unsafe { meos_sys::tint_max_value(self.inner()) }
                }
                fn value_at_timestamp<Tz: TimeZone>(
                    &self,
                    timestamp: DateTime<Tz>,
                ) -> Option<Self::Type> {
                    let mut result = 0.into();
                    unsafe {
                        let success = meos_sys::tint_value_at_timestamptz(
                            self.inner(),
                            to_meos_timestamp(&timestamp),
                            true,
                            &raw mut result,
                        );
                        if success { Some(result) } else { None }
                    }
                }
                fn at_value(&self, value: &Self::Type) -> Option<Self> {
                    let result = unsafe {
                        meos_sys::tint_at_value(self.inner(), *value)
                    };
                    if result != ptr::null_mut() {
                        Some(Self::from_inner_as_temporal(result))
                    } else {
                        None
                    }
                }
                fn at_values(&self, values: &[Self::Type]) -> Option<Self> {
                    unsafe {
                        let set = meos_sys::intset_make(
                            values.as_ptr(),
                            values.len() as i32,
                        );
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
                        meos_sys::tint_minus_value(self.inner(), value)
                    })
                }
                fn minus_values(&self, values: &[Self::Type]) -> Self {
                    Self::from_inner_as_temporal(unsafe {
                        let set = meos_sys::intset_make(
                            values.as_ptr(),
                            values.len() as i32,
                        );
                        meos_sys::temporal_minus_values(self.inner(), set)
                    })
                }
            }
            impl Debug for TIntSeq {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let out_str = unsafe { meos_sys::tint_out(self.inner()) };
                    let c_str = unsafe { CStr::from_ptr(out_str) };
                    let str = c_str.to_str().map_err(|_| std::fmt::Error)?;
                    let result = f.write_str(str);
                    unsafe { libc::free(out_str as *mut c_void) };
                    result
                }
            }
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
                        meos_sys::tintseqset_from_base_tstzspanset(
                            value,
                            time_span_set.inner(),
                        )
                    })
                }
            }
            impl TSequenceSet for TIntSeqSet {
                fn from_inner(inner: *const meos_sys::TSequenceSet) -> Self {
                    Self { _inner: inner }
                }
            }
            impl TInt for TIntSeqSet {}
            impl Collection for TIntSeqSet {
                type Type = i32;
                fn is_contained_in(&self, container: &Self) -> bool {
                    unsafe {
                        meos_sys::contained_tnumber_tnumber(
                            self.inner(),
                            container.inner(),
                        )
                    }
                }
                fn overlaps(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::overlaps_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn is_left(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::left_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn is_over_or_left(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::overleft_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn is_over_or_right(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::overright_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn is_right(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::right_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn is_adjacent(&self, other: &Self) -> bool {
                    unsafe {
                        meos_sys::adjacent_tnumber_tnumber(self.inner(), other.inner())
                    }
                }
                fn contains(&self, content: &Self::Type) -> bool {
                    IntSpanSet::from_inner(unsafe {
                            meos_sys::tnumber_valuespans(self.inner())
                        })
                        .contains(content)
                }
            }
            impl Clone for TIntSeqSet {
                fn clone(&self) -> Self {
                    Temporal::from_inner_as_temporal(unsafe {
                        meos_sys::temporal_copy(self.inner())
                    })
                }
            }
            impl FromStr for TIntSeqSet {
                type Err = ParseError;
                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    CString::new(s)
                        .map_err(|_| ParseError)
                        .map(|string| {
                            let inner = unsafe { meos_sys::tint_in(string.as_ptr()) };
                            Self::from_inner_as_temporal(inner)
                        })
                }
            }
            impl PartialEq for TIntSeqSet {
                fn eq(&self, other: &Self) -> bool {
                    unsafe { meos_sys::temporal_eq(self.inner(), other.inner()) }
                }
            }
            impl Hash for TIntSeqSet {
                fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                    let hash = unsafe { meos_sys::temporal_hash(self.inner()) };
                    state.write_u32(hash);
                    state.finish();
                }
            }
            impl TNumber for TIntSeqSet {
                fn nearest_approach_distance(&self, other: &Self) -> Self::Type {
                    unsafe { meos_sys::nad_tint_tint(self.inner(), other.inner()) }
                }
            }
            impl Temporal for TIntSeqSet {
                type TI = TIntInst;
                type TS = TIntSeq;
                type TSS = TIntSeqSet;
                type TBB = TBox;
                fn from_inner_as_temporal(inner: *const meos_sys::Temporal) -> Self {
                    Self {
                        _inner: inner as *const meos_sys::TSequenceSet,
                    }
                }
                fn from_mfjson(mfjson: &str) -> Self {
                    let cstr = CString::new(mfjson).unwrap();
                    Self::from_inner_as_temporal(unsafe {
                        meos_sys::tint_from_mfjson(cstr.as_ptr())
                    })
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
                        let values = meos_sys::tint_values(self.inner(), &raw mut count);
                        Vec::from_raw_parts(values, count as usize, count as usize)
                    }
                }
                fn start_value(&self) -> Self::Type {
                    unsafe { meos_sys::tint_start_value(self.inner()) }
                }
                fn end_value(&self) -> Self::Type {
                    unsafe { meos_sys::tint_end_value(self.inner()) }
                }
                fn min_value(&self) -> Self::Type {
                    unsafe { meos_sys::tint_min_value(self.inner()) }
                }
                fn max_value(&self) -> Self::Type {
                    unsafe { meos_sys::tint_max_value(self.inner()) }
                }
                fn value_at_timestamp<Tz: TimeZone>(
                    &self,
                    timestamp: DateTime<Tz>,
                ) -> Option<Self::Type> {
                    let mut result = 0.into();
                    unsafe {
                        let success = meos_sys::tint_value_at_timestamptz(
                            self.inner(),
                            to_meos_timestamp(&timestamp),
                            true,
                            &raw mut result,
                        );
                        if success { Some(result) } else { None }
                    }
                }
                fn at_value(&self, value: &Self::Type) -> Option<Self> {
                    let result = unsafe {
                        meos_sys::tint_at_value(self.inner(), *value)
                    };
                    if result != ptr::null_mut() {
                        Some(Self::from_inner_as_temporal(result))
                    } else {
                        None
                    }
                }
                fn at_values(&self, values: &[Self::Type]) -> Option<Self> {
                    unsafe {
                        let set = meos_sys::intset_make(
                            values.as_ptr(),
                            values.len() as i32,
                        );
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
                        meos_sys::tint_minus_value(self.inner(), value)
                    })
                }
                fn minus_values(&self, values: &[Self::Type]) -> Self {
                    Self::from_inner_as_temporal(unsafe {
                        let set = meos_sys::intset_make(
                            values.as_ptr(),
                            values.len() as i32,
                        );
                        meos_sys::temporal_minus_values(self.inner(), set)
                    })
                }
            }
            impl Debug for TIntSeqSet {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let out_str = unsafe { meos_sys::tint_out(self.inner()) };
                    let c_str = unsafe { CStr::from_ptr(out_str) };
                    let str = c_str.to_str().map_err(|_| std::fmt::Error)?;
                    let result = f.write_str(str);
                    unsafe { libc::free(out_str as *mut c_void) };
                    result
                }
            }
        }
        pub mod tnumber {
            use crate::{
                boxes::tbox::TBox,
                collections::number::{
                    number_span::NumberSpan, number_span_set::NumberSpanSet,
                },
                temporal::temporal::{
                    impl_always_and_ever_value_functions_with_ordering, Temporal,
                },
            };
            pub trait TNumber: Temporal<TBB = TBox> {
                /// Returns the bounding box of `self`.
                ///
                /// # Returns
                /// The bounding box of `self`.
                ///
                /// # Safety
                /// This function uses unsafe code to call the `meos_sys::tbox_tnumber` function.
                fn bounding_box(&self) -> TBox {
                    unsafe { TBox::from_inner(meos_sys::tnumber_to_tbox(self.inner())) }
                }
                /// Returns the integral of `self`.
                ///
                /// # Returns
                /// The integral of `self`.
                ///
                /// # Safety
                /// This function uses unsafe code to call the `meos_sys::tnumber_integral` function.
                fn integral(&self) -> f64 {
                    unsafe { meos_sys::tnumber_integral(self.inner()) }
                }
                /// Returns the time-weighted average of `self`.
                ///
                /// # Returns
                /// The time-weighted average of `self`.
                ///
                /// # Safety
                /// This function uses unsafe code to call the `meos_sys::tnumber_twavg` function.
                fn time_weighted_average(&self) -> f64 {
                    unsafe { meos_sys::tnumber_twavg(self.inner()) }
                }
                /// Returns a new temporal object with the values of `self` where it's not in `span`
                ///
                /// ## Arguments
                /// * `span` - A `IntSpan` or `FloatSpan` to substract the values from
                fn minus_span(&self, span: impl NumberSpan) -> Self {
                    Self::from_inner_as_temporal(unsafe {
                        meos_sys::tnumber_minus_span(self.inner(), span.inner())
                    })
                }
                /// Returns a new temporal object with the values of `self` where it's not in `span_set`
                ///
                /// ## Arguments
                /// * `span_set` - A `IntSpanSet` or `FloatSpanSet` to substract the values from
                fn minus_span_set(&self, span_set: impl NumberSpanSet) -> Self {
                    Self::from_inner_as_temporal(unsafe {
                        meos_sys::tnumber_minus_spanset(self.inner(), span_set.inner())
                    })
                }
                /// Adds the value(s) of `other` to the value(s) of `self`.
                ///
                /// # Arguments
                /// * `other` - A temporal number to add to the value(s) of `self`.
                fn add(&self, other: &Self) -> Self {
                    Self::from_inner_as_temporal(unsafe {
                        meos_sys::add_tnumber_tnumber(self.inner(), other.inner())
                    })
                }
                /// Substract the value(s) of `other` to the value(s) of `self`.
                ///
                /// # Arguments
                /// * `other` - A temporal number to substract to the value(s) of `self`.
                fn substract(&self, other: &Self) -> Self {
                    Self::from_inner_as_temporal(unsafe {
                        meos_sys::sub_tnumber_tnumber(self.inner(), other.inner())
                    })
                }
                /// Multiplicate the value(s) of `other` by the value(s) of `self`.
                ///
                /// # Arguments
                /// * `other` - A temporal number to multiplicate by the value(s) of `self`.
                fn multiplicate(&self, other: &Self) -> Self {
                    Self::from_inner_as_temporal(unsafe {
                        meos_sys::mult_tnumber_tnumber(self.inner(), other.inner())
                    })
                }
                /// Divide the value(s) of `other` by the value(s) of `self`.
                ///
                /// # Arguments
                /// * `other` - A temporal number to divide by the value(s) of `self`.
                fn divide(&self, other: &Self) -> Self {
                    Self::from_inner_as_temporal(unsafe {
                        meos_sys::div_tnumber_tnumber(self.inner(), other.inner())
                    })
                }
                /// Returns the absolute value of `self`.
                ///
                /// # Returns
                /// The absolute value of `self`.
                fn abs(&self) -> Self {
                    Self::from_inner_as_temporal(unsafe {
                        meos_sys::tnumber_abs(self.inner())
                    })
                }
                /// Returns the change in value between successive pairs of `self`.
                ///
                /// # Returns
                /// The change in value between successive pairs of `self`.
                fn delta_value(&self) -> Self {
                    Self::from_inner_as_temporal(unsafe {
                        meos_sys::tnumber_delta_value(self.inner())
                    })
                }
                /// Returns the distance between `self` and `other`.
                ///
                /// # Arguments
                /// * `other` - A temporal number to compute the distance to.
                fn distance(&self, other: &Self) -> Self {
                    Self::from_inner_as_temporal(unsafe {
                        meos_sys::distance_tnumber_tnumber(self.inner(), other.inner())
                    })
                }
                /// Returns the nearest approach distance between `self` and `other`.
                ///
                /// # Arguments
                /// * `other` - A temporal number to compute the nearest approach distance to.
                fn nearest_approach_distance(&self, other: &Self) -> Self::Type;
            }
            pub(crate) use impl_temporal_for_tnumber;
        }
    }
    pub mod temporal {
        use std::{
            ffi::{c_void, CStr, CString},
            hash::Hash, ptr,
        };
        use crate::{
            collections::{
                base::{collection::Collection, span::Span, span_set::SpanSet},
                datetime::{tstz_span::TsTzSpan, tstz_span_set::TsTzSpanSet},
            },
            utils::{
                create_interval, from_interval, from_meos_timestamp, to_meos_timestamp,
            },
            BoundingBox, WKBVariant,
        };
        use chrono::{DateTime, TimeDelta, TimeZone, Utc};
        use super::{
            interpolation::TInterpolation, tinstant::TInstant, tsequence::TSequence,
            tsequence_set::TSequenceSet, JSONCVariant,
        };
        pub trait Temporal: Collection + Hash {
            type TI: TInstant;
            type TS: TSequence;
            type TSS: TSequenceSet;
            type TBB: BoundingBox;
            fn from_inner_as_temporal(inner: *const meos_sys::Temporal) -> Self;
            /// Creates a temporal object from an MF-JSON string.
            ///
            /// ## Arguments
            /// * `mfjson` - The MF-JSON string.
            ///
            /// ## Returns
            /// A temporal object.
            fn from_mfjson(mfjson: &str) -> Self;
            /// Creates a temporal object from Well-Known Binary (WKB) bytes.
            ///
            /// ## Arguments
            /// * `wkb` - The WKB bytes.
            ///
            /// ## Returns
            /// A temporal object.
            fn from_wkb(wkb: &[u8]) -> Self {
                unsafe {
                    Self::from_inner_as_temporal(
                        meos_sys::temporal_from_wkb(wkb.as_ptr(), wkb.len()),
                    )
                }
            }
            /// Creates a temporal object from a hex-encoded WKB string.
            ///
            /// ## Arguments
            /// * `hexwkb` - The hex-encoded WKB string.
            ///
            /// ## Returns
            /// A temporal object.
            fn from_hexwkb(hexwkb: &[u8]) -> Self {
                let c_hexwkb = CString::new(hexwkb).unwrap();
                unsafe {
                    let inner = meos_sys::temporal_from_hexwkb(c_hexwkb.as_ptr());
                    Self::from_inner_as_temporal(inner)
                }
            }
            /// Creates a temporal object by merging multiple temporal objects.
            ///
            /// ## Arguments
            /// * `temporals` - The temporal objects to merge.
            ///
            /// ## Returns
            /// A merged temporal object.
            fn from_merge(temporals: &[Self]) -> Self {
                let mut t_list: Vec<_> = temporals.iter().map(Self::inner).collect();
                Self::from_inner_as_temporal(unsafe {
                    meos_sys::temporal_merge_array(
                        t_list.as_mut_ptr(),
                        temporals.len() as i32,
                    )
                })
            }
            fn inner(&self) -> *const meos_sys::Temporal;
            /// Returns the temporal object as an MF-JSON string.
            ///
            /// ## Arguments
            /// * `with_bbox` - Whether to include the bounding box in the output.
            /// * `flags` - The flags to use for the output.
            /// * `precision` - The precision to use for the output.
            /// * `srs` - The spatial reference system (SRS) to use for the output.
            ///
            /// ## Returns
            /// The temporal object as an MF-JSON string.
            fn as_mfjson(
                &self,
                with_bbox: bool,
                variant: JSONCVariant,
                precision: i32,
                srs: &str,
            ) -> String {
                let srs = CString::new(srs).unwrap();
                let out_str = unsafe {
                    meos_sys::temporal_as_mfjson(
                        self.inner(),
                        with_bbox,
                        variant as i32,
                        precision,
                        srs.as_ptr(),
                    )
                };
                let c_str = unsafe { CStr::from_ptr(out_str) };
                let str = c_str.to_str().unwrap().to_owned();
                unsafe { libc::free(out_str as *mut c_void) };
                str
            }
            /// Returns the temporal object as Well-Known Binary (WKB) bytes.
            ///
            /// ## Returns
            /// The temporal object as WKB bytes.
            fn as_wkb(&self, variant: WKBVariant) -> &[u8] {
                unsafe {
                    let mut size: usize = 0;
                    let ptr = meos_sys::temporal_as_wkb(
                        self.inner(),
                        variant.into(),
                        &mut size,
                    );
                    std::slice::from_raw_parts(ptr, size)
                }
            }
            /// Returns the temporal object as a hex-encoded WKB string.
            ///
            /// ## Returns
            /// The temporal object as a hex-encoded WKB bytes.
            fn as_hexwkb(&self, variant: WKBVariant) -> &[u8] {
                unsafe {
                    let mut size: usize = 0;
                    let hexwkb_ptr = meos_sys::temporal_as_hexwkb(
                        self.inner(),
                        variant.into(),
                        &mut size,
                    );
                    CStr::from_ptr(hexwkb_ptr).to_bytes()
                }
            }
            /// Returns the bounding box of the temporal object.
            ///
            /// ## Returns
            /// The bounding box of the temporal object.
            fn bounding_box(&self) -> Self::TBB;
            /// Returns the interpolation method of the temporal object.
            ///
            /// ## Returns
            /// The interpolation method.
            fn interpolation(&self) -> TInterpolation {
                let string = unsafe {
                    CStr::from_ptr(meos_sys::temporal_interp(self.inner()))
                };
                string.to_str().unwrap().parse().unwrap()
            }
            /// Returns the set of unique values in the temporal object.
            ///
            /// ## Returns
            /// A set of unique values.
            /// Returns the list of values taken by the temporal object.
            ///
            /// ## Returns
            /// A list of values.
            fn values(&self) -> Vec<Self::Type>;
            /// Returns the starting value of the temporal object.
            ///
            /// ## Returns
            /// The starting value.
            fn start_value(&self) -> Self::Type;
            /// Returns the ending value of the temporal object.
            ///
            /// ## Returns
            /// The ending value.
            fn end_value(&self) -> Self::Type;
            /// Returns the minimum value of the temporal object.
            ///
            /// ## Returns
            /// The minimum value.
            fn min_value(&self) -> Self::Type;
            /// Returns the maximum value of the temporal object.
            ///
            /// ## Returns
            /// The maximum value.
            fn max_value(&self) -> Self::Type;
            /// Returns the value of the temporal object at a specific timestamp.
            ///
            /// ## Arguments
            /// * `timestamp` - The timestamp.
            ///
            /// ## Returns
            /// The value at the given timestamp.
            fn value_at_timestamp<Tz: TimeZone>(
                &self,
                timestamp: DateTime<Tz>,
            ) -> Option<Self::Type>;
            /// Returns the time span on which the temporal object is defined.
            ///
            /// ## Returns
            /// The time span.
            fn time(&self) -> TsTzSpanSet {
                TsTzSpanSet::from_inner(unsafe { meos_sys::temporal_time(self.inner()) })
            }
            /// Returns the time span on which the temporal object is defined.
            ///
            /// ## Returns
            /// The time span.
            fn timespan(&self) -> TsTzSpan {
                unsafe {
                    TsTzSpan::from_inner(meos_sys::temporal_to_tstzspan(self.inner()))
                }
            }
            /// Returns the duration of the temporal object.
            ///
            /// ## Arguments
            /// * `ignore_gaps` - Whether to ignore gaps in the temporal value.
            ///
            /// ## Returns
            /// The duration of the temporal object.
            fn duration(&self, ignore_gaps: bool) -> TimeDelta {
                from_interval(unsafe {
                    meos_sys::temporal_duration(self.inner(), ignore_gaps).read()
                })
            }
            /// Returns the number of instants in the temporal object.
            ///
            /// ## Returns
            /// The number of instants.
            fn num_instants(&self) -> i32 {
                unsafe { meos_sys::temporal_num_instants(self.inner()) }
            }
            /// Returns the first instant in the temporal object.
            ///
            /// ## Returns
            /// The first instant.
            fn start_instant(&self) -> Self::TI {
                <Self::TI as TInstant>::from_inner(unsafe {
                    meos_sys::temporal_start_instant(self.inner())
                })
            }
            /// Returns the last instant in the temporal object.
            ///
            /// ## Returns
            /// The last instant.
            fn end_instant(&self) -> Self::TI {
                <Self::TI as TInstant>::from_inner(unsafe {
                    meos_sys::temporal_end_instant(self.inner())
                })
            }
            /// Returns the instant with the minimum value in the temporal object.
            ///
            /// ## Returns
            /// The instant with the minimum value.
            fn min_instant(&self) -> Self::TI {
                <Self::TI as TInstant>::from_inner(unsafe {
                    meos_sys::temporal_min_instant(self.inner())
                })
            }
            /// Returns the instant with the maximum value in the temporal object.
            ///
            /// ## Returns
            /// The instant with the maximum value.
            fn max_instant(&self) -> Self::TI {
                <Self::TI as TInstant>::from_inner(unsafe {
                    meos_sys::temporal_max_instant(self.inner())
                })
            }
            /// Returns the n-th instant in the temporal object.
            ///
            /// ## Arguments
            /// * `n` - The index (0-based).
            ///
            /// ## Return
            /// The n-th instant if exists, None otherwise.
            fn instant_n(&self, n: i32) -> Option<Self::TI> {
                let result = unsafe { meos_sys::temporal_instant_n(self.inner(), n) };
                if !result.is_null() {
                    Some(<Self::TI as TInstant>::from_inner(result))
                } else {
                    None
                }
            }
            /// Returns the list of instants in the temporal object.
            ///
            /// ## Returns
            /// A list of instants.
            fn instants(&self) -> Vec<Self::TI> {
                let mut count = 0;
                unsafe {
                    let instants = meos_sys::temporal_instants(
                        self.inner(),
                        &raw mut count,
                    );
                    Vec::from_raw_parts(instants, count as usize, count as usize)
                        .iter()
                        .map(|&instant| <Self::TI as TInstant>::from_inner(instant))
                        .collect()
                }
            }
            /// Returns the number of timestamps in the temporal object.
            ///
            /// ## Returns
            /// The number of timestamps.
            fn num_timestamps(&self) -> i32 {
                unsafe { meos_sys::temporal_num_timestamps(self.inner()) }
            }
            /// Returns the first timestamp in the temporal object.
            ///
            /// ## Returns
            /// The first timestamp.
            fn start_timestamp(&self) -> DateTime<Utc> {
                from_meos_timestamp(unsafe {
                    meos_sys::temporal_start_timestamptz(self.inner())
                })
            }
            /// Returns the last timestamp in the temporal object.
            ///
            /// ## Returns
            /// The last timestamp.
            fn end_timestamp(&self) -> DateTime<Utc> {
                from_meos_timestamp(unsafe {
                    meos_sys::temporal_end_timestamptz(self.inner())
                })
            }
            /// Returns the n-th timestamp in the temporal object.
            ///
            /// ## Arguments
            /// * `n` - The index (0-based).
            ///
            /// ## Returns
            /// The n-th timestamp if exists, None otherwise.
            fn timestamp_n(&self, n: i32) -> Option<DateTime<Utc>> {
                let mut timestamp = 0;
                unsafe {
                    let success = meos_sys::temporal_timestamptz_n(
                        self.inner(),
                        n,
                        &raw mut timestamp,
                    );
                    if success { Some(from_meos_timestamp(timestamp)) } else { None }
                }
            }
            /// Returns the list of timestamps in the temporal object.
            ///
            /// ## Returns
            /// A list of timestamps.
            fn timestamps(&self) -> Vec<DateTime<Utc>> {
                let mut count = 0;
                let timestamps = unsafe {
                    meos_sys::temporal_timestamps(self.inner(), &raw mut count)
                };
                unsafe {
                    Vec::from_raw_parts(timestamps, count as usize, count as usize)
                        .iter()
                        .map(|&timestamp| from_meos_timestamp(timestamp))
                        .collect()
                }
            }
            /// Returns the list of segments in the temporal object.
            ///
            /// ## Returns
            /// A list of segments.
            ///
            /// MEOS Functions:
            ///    `temporal_segments`
            fn segments(&self) -> Vec<Self::TS> {
                let mut count = 0;
                let segments = unsafe {
                    meos_sys::temporal_segments(self.inner(), &raw mut count)
                };
                unsafe {
                    Vec::from_raw_parts(segments, count as usize, count as usize)
                        .iter()
                        .map(|&segment| <Self::TS as TSequence>::from_inner(segment))
                        .collect()
                }
            }
            /// Returns a new `Temporal` object with the given interpolation.
            ///
            /// MEOS Functions:
            ///     `temporal_set_interpolation`
            fn set_interpolation(&self, interpolation: TInterpolation) -> Self {
                Self::from_inner_as_temporal(unsafe {
                    meos_sys::temporal_set_interp(self.inner(), interpolation as u32)
                })
            }
            /// Returns a new `Temporal` with the temporal dimension shifted by `delta`.
            ///
            /// ## Arguments
            /// * `delta` - TimeDelta to shift the temporal dimension.
            ///
            /// MEOS Functions:
            ///     `temporal_shift_time`
            fn shift_time(&self, delta: TimeDelta) -> Self {
                self.shift_scale_time(Some(delta), None)
            }
            /// Returns a new `Temporal` scaled so the temporal dimension has duration `duration`.
            ///
            /// ## Arguments
            /// * `duration` - TimeDelta representing the new temporal duration.
            ///
            /// MEOS Functions:
            ///     `temporal_scale_time`
            fn scale_time(&self, duration: TimeDelta) -> Self {
                self.shift_scale_time(None, Some(duration))
            }
            /// Returns a new `Temporal` with the time dimension shifted and scaled.
            ///
            /// ## Arguments
            /// * `shift` - TimeDelta to shift the time dimension.
            /// * `duration` - TimeDelta representing the new temporal duration.
            ///
            /// MEOS Functions:
            ///     `temporal_shift_scale_time`
            fn shift_scale_time(
                &self,
                shift: Option<TimeDelta>,
                duration: Option<TimeDelta>,
            ) -> Self {
                let d = {
                    if let Some(d) = shift {
                        &*Box::new(create_interval(d)) as *const meos_sys::Interval
                    } else {
                        std::ptr::null()
                    }
                };
                let w = {
                    if let Some(w) = duration {
                        &*Box::new(create_interval(w)) as *const meos_sys::Interval
                    } else {
                        std::ptr::null()
                    }
                };
                let modified = unsafe {
                    meos_sys::temporal_shift_scale_time(self.inner(), d, w)
                };
                Self::from_inner_as_temporal(modified)
            }
            /// Returns a new `Temporal` downsampled with respect to `duration`.
            ///
            /// ## Arguments
            /// * `duration` - TimeDelta of the temporal tiles.
            /// * `start` - Start time of the temporal tiles.
            /// * `interpolation`- Interpolation of the resulting temporal object.
            ///
            /// MEOS Functions:
            ///     `temporal_tsample`
            fn temporal_sample<Tz: TimeZone>(
                self,
                duration: TimeDelta,
                start: DateTime<Tz>,
                interpolation: TInterpolation,
            ) -> Self {
                let interval = create_interval(duration);
                Self::from_inner_as_temporal(unsafe {
                    meos_sys::temporal_tsample(
                        self.inner(),
                        &raw const interval,
                        to_meos_timestamp(&start),
                        interpolation as u32,
                    )
                })
            }
            /// Returns a new `Temporal` with precision reduced to `duration`.
            ///
            /// ## Arguments
            /// * `duration` - TimeDelta of the temporal tiles.
            /// * `start` - Start time of the temporal tiles.
            ///
            /// MEOS Functions:
            ///     `temporal_tprecision`
            fn temporal_precision<Tz: TimeZone>(
                self,
                duration: TimeDelta,
                start: DateTime<Tz>,
            ) -> Self {
                let interval = create_interval(duration);
                Self::from_inner_as_temporal(unsafe {
                    meos_sys::temporal_tprecision(
                        self.inner(),
                        &raw const interval,
                        to_meos_timestamp(&start),
                    )
                })
            }
            /// Converts `self` into a `TInstant`.
            ///
            /// MEOS Functions:
            ///     `temporal_to_tinstant`
            fn to_instant(&self) -> Self::TI {
                TInstant::from_inner(unsafe {
                    meos_sys::temporal_to_tinstant(self.inner())
                })
            }
            /// Converts `self` into a `TSequence`.
            ///
            /// ## Arguments
            /// * `interpolation` - The interpolation type for the sequence.
            ///
            /// MEOS Functions:
            ///     `temporal_to_sequence`
            fn to_sequence(&self, interpolation: TInterpolation) -> Self::TS {
                let c_str = CString::new(interpolation.to_string()).unwrap();
                TSequence::from_inner(unsafe {
                    meos_sys::temporal_to_tsequence(
                        self.inner(),
                        c_str.as_ptr() as *mut _,
                    )
                })
            }
            /// Converts `self` into a `TSequenceSet`.
            ///
            /// ## Arguments
            /// * `interpolation` - The interpolation type for the sequence set.
            ///
            /// MEOS Functions:
            ///     `temporal_to_tsequenceset`
            fn to_sequence_set(&self, interpolation: TInterpolation) -> Self::TSS {
                let c_str = CString::new(interpolation.to_string()).unwrap();
                TSequenceSet::from_inner(unsafe {
                    meos_sys::temporal_to_tsequenceset(
                        self.inner(),
                        c_str.as_ptr() as *mut _,
                    )
                })
            }
            /// Appends `instant` to `self`.
            ///
            /// ## Arguments
            /// * `instant` - Instant to append.
            /// * `max_dist` - Maximum distance for defining a gap.
            /// * `max_time` - Maximum time for defining a gap.
            ///
            /// MEOS Functions:
            ///     `temporal_append_tinstant`
            fn append_instant(
                self,
                instant: Self::TI,
                max_dist: Option<f64>,
                max_time: Option<TimeDelta>,
            ) -> Self {
                let mut max_time = create_interval(max_time.unwrap_or_default());
                Self::from_inner_as_temporal(unsafe {
                    meos_sys::temporal_append_tinstant(
                        self.inner() as *mut _,
                        TInstant::inner_as_tinstant(&instant),
                        max_dist.unwrap_or_default(),
                        &raw mut max_time,
                        false,
                    )
                })
            }
            /// Appends `sequence` to `self`.
            ///
            /// ## Arguments
            /// * `sequence` - Sequence to append.
            ///
            /// MEOS Functions:
            ///     `temporal_append_tsequence`
            fn append_sequence(&self, sequence: Self::TS) -> Self {
                Self::from_inner_as_temporal(unsafe {
                    meos_sys::temporal_append_tsequence(
                        self.inner() as *mut _,
                        sequence.inner_as_tsequence(),
                        false,
                    )
                })
            }
            /// Merges `self` with `other`.
            ///
            /// ## Arguments
            /// * `other` - Another temporal object
            ///
            /// MEOS Functions:
            ///     `temporal_merge`
            fn merge_other(&self, other: Self) -> Self {
                Self::from_inner_as_temporal(unsafe {
                    meos_sys::temporal_merge(self.inner(), other.inner())
                })
            }
            /// Inserts `other` into `self`.
            ///
            /// ## Arguments
            /// * `other` - Temporal object to insert.
            /// * `connect` - Whether to connect inserted elements with existing ones.
            ///
            /// MEOS Functions:
            ///     `temporal_insert`
            fn insert(&self, other: Self, connect: bool) -> Self {
                Self::from_inner_as_temporal(unsafe {
                    meos_sys::temporal_insert(self.inner(), other.inner(), connect)
                })
            }
            /// Updates `self` with `other`.
            ///
            /// ## Arguments
            /// * `other` - Temporal object to update with.
            /// * `connect` - Whether to connect updated elements with existing ones.
            ///
            /// MEOS Functions:
            ///     `temporal_update`
            fn update(&self, other: Self, connect: bool) -> Self {
                Self::from_inner_as_temporal(unsafe {
                    meos_sys::temporal_update(self.inner(), other.inner(), connect)
                })
            }
            /// Deletes elements from `self` at `other`.
            ///
            /// ## Arguments
            /// * `other` - Time object specifying the elements to delete.
            /// * `connect` - Whether to connect the potential gaps generated by the deletions.
            ///
            /// MEOS Functions:
            ///     `temporal_delete`
            fn delete_at_timestamp<Tz: TimeZone>(
                &self,
                other: DateTime<Tz>,
                connect: bool,
            ) -> Self {
                Self::from_inner_as_temporal(unsafe {
                    meos_sys::temporal_delete_timestamptz(
                        self.inner(),
                        to_meos_timestamp(&other),
                        connect,
                    )
                })
            }
            /// Deletes elements from `self` at `time_span`.
            ///
            /// ## Arguments
            /// * `time_span` - Time span object specifying the elements to delete.
            /// * `connect` - Whether to connect the potential gaps generated by the deletions.
            fn delete_at_tstz_span(&self, time_span: TsTzSpan, connect: bool) -> Self {
                Self::from_inner_as_temporal(unsafe {
                    meos_sys::temporal_delete_tstzspan(
                        self.inner(),
                        time_span.inner(),
                        connect,
                    )
                })
            }
            /// Deletes elements from `self` at `time_span_set`.
            ///
            /// ## Arguments
            /// * `time_span_set` - Time span set object specifying the elements to delete.
            /// * `connect` - Whether to connect the potential gaps generated by the deletions.
            fn delete_at_tstz_span_set(
                &self,
                time_span_set: TsTzSpanSet,
                connect: bool,
            ) -> Self {
                Self::from_inner_as_temporal(unsafe {
                    meos_sys::temporal_delete_tstzspanset(
                        self.inner(),
                        time_span_set.inner(),
                        connect,
                    )
                })
            }
            /// Returns a new temporal object with values restricted to the time `other`.
            ///
            /// ## Arguments
            /// * `other` - A timestamp to restrict the values to.
            ///
            /// MEOS Functions:
            ///     `temporal_at_temporal_at_timestamptz`
            fn at_timestamp<Tz: TimeZone>(&self, other: DateTime<Tz>) -> Self {
                unsafe {
                    Self::from_inner_as_temporal(
                        meos_sys::temporal_at_timestamptz(
                            self.inner(),
                            to_meos_timestamp(&other),
                        ),
                    )
                }
            }
            /// Returns a new temporal object with values restricted to the time `time_span`.
            ///
            /// ## Arguments
            /// * `time_span` - A time span to restrict the values to.
            ///
            /// MEOS Functions:
            ///     `temporal_at_tstzspan`
            fn at_tstz_span(&self, time_span: TsTzSpan) -> Self {
                Self::from_inner_as_temporal(unsafe {
                    meos_sys::temporal_at_tstzspan(self.inner(), time_span.inner())
                })
            }
            /// Returns a new temporal object with values restricted to the time `time_span_set`.
            ///
            /// ## Arguments
            /// * `time_span_set` - A time span set to restrict the values to.
            ///
            /// MEOS Functions:
            ///     `temporal_at_tstzspanset`
            fn at_tstz_span_set(&self, time_span_set: TsTzSpanSet) -> Self {
                Self::from_inner_as_temporal(unsafe {
                    meos_sys::temporal_at_tstzspanset(
                        self.inner(),
                        time_span_set.inner(),
                    )
                })
            }
            /// Returns a new temporal object containing the times `self` is at its minimum value.
            ///
            /// MEOS Functions:
            ///     `temporal_at_min`
            fn at_min(&self) -> Self {
                Self::from_inner_as_temporal(unsafe {
                    meos_sys::temporal_at_min(self.inner())
                })
            }
            /// Returns a new temporal object containing the times `self` is at its maximum value.
            ///
            /// MEOS Functions:
            ///     `temporal_at_max`
            fn at_max(&self) -> Self {
                Self::from_inner_as_temporal(unsafe {
                    meos_sys::temporal_at_max(self.inner())
                })
            }
            /// Returns a new temporal object containing the times `self` is at `value`.
            ///
            /// MEOS Functions:
            ///     `temporal_at_value`
            fn at_value(&self, value: &Self::Type) -> Option<Self>;
            /// Returns a new temporal object containing the times `self` is in any of the values of `values`.
            ///
            /// MEOS Functions:
            ///     `temporal_at_values`
            fn at_values(&self, values: &[Self::Type]) -> Option<Self>;
            /// Returns a new temporal object with values at `timestamp` removed.
            ///
            /// ## Arguments
            /// * `timestamp` - A timestamp specifying the values to remove.
            ///
            /// MEOS Functions:
            ///     `temporal_minus_*`
            fn minus_timestamp<Tz: TimeZone>(&self, timestamp: DateTime<Tz>) -> Self {
                Self::from_inner_as_temporal(unsafe {
                    meos_sys::temporal_minus_timestamptz(
                        self.inner(),
                        to_meos_timestamp(&timestamp),
                    )
                })
            }
            /// Returns a new temporal object with values at any of the values of `timestamps` removed.
            ///
            /// ## Arguments
            /// * `timestamps` - A timestamp specifying the values to remove.
            ///
            /// MEOS Functions:
            ///     `temporal_minus_*`
            fn minus_timestamp_set<Tz: TimeZone>(
                &self,
                timestamps: &[DateTime<Tz>],
            ) -> Self {
                let timestamps: Vec<_> = timestamps
                    .iter()
                    .map(to_meos_timestamp)
                    .collect();
                let set = unsafe {
                    meos_sys::tstzset_make(timestamps.as_ptr(), timestamps.len() as i32)
                };
                Self::from_inner_as_temporal(unsafe {
                    meos_sys::temporal_minus_tstzset(self.inner(), set)
                })
            }
            /// Returns a new temporal object with values at `time_span` removed.
            ///
            /// ## Arguments
            /// * `time_span` - A time span specifying the values to remove.
            fn minus_tstz_span(&self, time_span: TsTzSpan) -> Self {
                Self::from_inner_as_temporal(unsafe {
                    meos_sys::temporal_minus_tstzspan(self.inner(), time_span.inner())
                })
            }
            /// Returns a new temporal object with values at `time_span_set` removed.
            ///
            /// ## Arguments
            /// * `time_span_set` - A time span set specifying the values to remove.
            fn minus_tstz_span_set(&self, time_span_set: TsTzSpanSet) -> Self {
                Self::from_inner_as_temporal(unsafe {
                    meos_sys::temporal_minus_tstzspanset(
                        self.inner(),
                        time_span_set.inner(),
                    )
                })
            }
            /// Returns a new temporal object containing the times `self` is not at its minimum value.
            ///
            /// MEOS Functions:
            ///     `temporal_minus_min`
            fn minus_min(&self) -> Self {
                Self::from_inner_as_temporal(unsafe {
                    meos_sys::temporal_minus_min(self.inner())
                })
            }
            /// Returns a new temporal object containing the times `self` is not at its maximum value.
            ///
            /// MEOS Functions:
            ///     `temporal_minus_max`
            fn minus_max(&self) -> Self {
                Self::from_inner_as_temporal(unsafe {
                    meos_sys::temporal_minus_max(self.inner())
                })
            }
            /// Returns a new temporal object containing the times `self` is not at `value`.
            ///
            /// MEOS Functions:
            ///     `temporal_minus_value`
            fn minus_value(&self, value: Self::Type) -> Self;
            /// Returns a new temporal object containing the times `self` is not at `values`.
            ///
            /// MEOS Functions:
            ///     `temporal_minus_values`
            fn minus_values(&self, values: &[Self::Type]) -> Self;
            /// Checks if the bounding box of `self` is adjacent to the bounding box of `other`.
            ///
            /// ## Arguments
            /// * `other` - A time or temporal object to compare.
            ///
            /// See also:
            ///     `Collection.is_adjacent`
            fn is_adjacent(&self, other: Self) -> bool {
                self.bounding_box().is_adjacent(&other.bounding_box())
            }
            /// Checks if the bounding timespan of `self` is temporally adjacent to the bounding timespan of `other`.
            ///
            /// ## Arguments
            /// * `other` - A time or temporal object to compare.
            ///
            /// See also:
            ///     `Collection.is_adjacent`
            fn is_temporally_adjacent(&self, other: Self) -> bool {
                self.timespan().is_adjacent(&other.timespan())
            }
            /// Checks if the bounding-box of `self` is contained in the bounding-box of `container`.
            ///
            /// ## Arguments
            /// * `container` - A time or temporal object to compare.
            ///
            /// See also:
            ///     `Collection.is_contained_in`
            fn is_contained_in(&self, other: Self) -> bool {
                self.bounding_box().is_contained_in(&other.bounding_box())
            }
            /// Checks if the bounding timespan of `self` is contained in the bounding timespan of `container`.
            ///
            /// ## Arguments
            /// * `container` - A time or temporal object to compare.
            ///
            /// See also:
            ///     `Collection.is_contained_in`
            fn is_temporally_contained_in(&self, other: Self) -> bool {
                self.timespan().is_contained_in(&other.timespan())
            }
            /// Checks if the bounding timespan of `self` contains the bounding timespan of `other`.
            ///
            /// ## Arguments
            /// * `other` - A time or temporal object to compare.
            fn contains(&self, other: Self) -> bool {
                other.bounding_box().is_contained_in(&self.bounding_box())
            }
            /// Checks if the bounding timespan of `self` temporally contains the bounding timespan of `other`.
            ///
            /// ## Arguments
            /// * `other` - A time or temporal object to compare.
            fn temporally_contains(&self, other: Self) -> bool {
                other.timespan().is_contained_in(&self.timespan())
            }
            /// Checks if the bounding timespan of `self` overlaps with the bounding timespan of `other`.
            ///
            /// ## Arguments
            /// * `other` - A time or temporal object to compare.
            ///
            /// See also:
            ///     `Collection.overlaps`
            fn overlaps(&self, other: Self) -> bool {
                self.bounding_box().overlaps(&other.bounding_box())
            }
            /// Checks if the bounding timespan of `self` temporally overlaps with the bounding timespan of `other`.
            ///
            /// ## Arguments
            /// * `other` - A time or temporal object to compare.
            ///
            /// See also:
            ///     `TsTzSpan.overlaps`
            fn temporally_overlaps(&self, other: Self) -> bool {
                self.timespan().overlaps(&other.timespan())
            }
            /// Returns whether `self` is before `other`.
            ///
            /// ## Arguments
            /// * `other` - A time or temporal object to compare.
            ///
            /// ## Returns
            /// True if `self` is before `other`, False otherwise.
            ///
            /// See also:
            ///     `TsTzSpan.is_left`
            fn is_before(&self, other: Self) -> bool {
                self.timespan().is_left(&other.timespan())
            }
            /// Returns whether `self` is before `other` allowing overlap.
            ///
            /// ## Arguments
            /// * `other` - A time or temporal object to compare.
            ///
            /// ## Returns
            /// True if `self` is before `other` allowing overlap, False otherwise.
            ///
            /// See also:
            ///     `TsTzSpan.is_over_or_left`
            fn is_over_or_before(&self, other: Self) -> bool {
                self.timespan().is_over_or_left(&other.timespan())
            }
            /// Returns whether `self` is after `other`.
            ///
            /// ## Arguments
            /// * `other` - A time or temporal object to compare.
            ///
            /// ## Returns
            /// True if `self` is after `other`, False otherwise.
            ///
            /// See also:
            ///     `TsTzSpan.is_right`
            fn is_after(&self, other: Self) -> bool {
                self.timespan().is_right(&other.timespan())
            }
            /// Returns whether `self` is after `other` allowing overlap.
            ///
            /// ## Arguments
            /// * `other` - A time or temporal object to compare.
            ///
            /// ## Returns
            /// True if `self` is after `other` allowing overlap, False otherwise.
            ///
            /// See also:
            ///     `TsTzSpan.is_over_or_right`
            fn is_over_or_after(&self, other: Self) -> bool {
                self.timespan().is_over_or_right(&other.timespan())
            }
            /// Returns the Frechet distance between `self` and `other`.
            ///
            /// ## Arguments
            /// * `other` - A temporal object to compare.
            ///
            /// ## Returns
            /// A float with the Frechet distance.
            ///
            /// MEOS Functions:
            ///     `temporal_frechet_distance`
            fn frechet_distance(&self, other: Self) -> f64 {
                unsafe {
                    meos_sys::temporal_frechet_distance(self.inner(), other.inner())
                }
            }
            /// Returns the Dynamic Time Warp distance between `self` and `other`.
            ///
            /// ## Arguments
            /// * `other` - A temporal object to compare.
            ///
            /// ## Returns
            /// A float with the Dynamic Time Warp distance.
            ///
            /// MEOS Functions:
            ///     `temporal_dyntimewarp_distance`
            fn dyntimewarp_distance(&self, other: Self) -> f64 {
                unsafe {
                    meos_sys::temporal_dyntimewarp_distance(self.inner(), other.inner())
                }
            }
            /// Returns the Hausdorff distance between `self` and `other`.
            ///
            /// ## Arguments
            /// * `other` - A temporal object to compare.
            ///
            /// ## Returns
            /// A float with the Hausdorff distance.
            ///
            /// MEOS Functions:
            ///     `temporal_hausdorff_distance`
            fn hausdorff_distance(&self, other: Self) -> f64 {
                unsafe {
                    meos_sys::temporal_hausdorff_distance(self.inner(), other.inner())
                }
            }
            /// Splits the temporal object into multiple pieces based on the given duration.
            ///
            /// ## Arguments
            /// * `duration` - Duration of each temporal tile.
            /// * `start` - Start time for the tiles.
            ///
            /// ## Returns
            /// A list of temporal objects representing the split tiles.
            ///
            /// MEOS Functions:
            ///     `temporal_time_split`
            fn time_split<Tz: TimeZone>(
                &self,
                duration: TimeDelta,
                start: DateTime<Tz>,
            ) -> Vec<Self> {
                let duration = create_interval(duration);
                let start = to_meos_timestamp(&start);
                let mut count = 0;
                let _buckets = Vec::new().as_mut_ptr();
                unsafe {
                    let temps = meos_sys::temporal_time_split(
                        self.inner(),
                        &raw const duration,
                        start,
                        _buckets,
                        &raw mut count,
                    );
                    Vec::from_raw_parts(temps, count as usize, count as usize)
                        .iter()
                        .map(|&t| Temporal::from_inner_as_temporal(t))
                        .collect()
                }
            }
            /// Splits the temporal object into `n` equal-duration parts.
            ///
            /// ## Arguments
            /// * `n` - Number of parts to split into.
            ///
            /// ## Returns
            /// A list of temporal objects representing the split parts.
            ///
            /// MEOS Functions:
            ///     `temporal_time_split`
            fn time_split_n(&self, n: usize) -> Vec<Self> {
                let start = self.start_timestamp();
                let duration = (self.end_timestamp() - start) / n as i32;
                self.time_split(duration, start)
            }
            /// Extracts the subsequences where the object stays within a certain distance for a specified duration.
            ///
            /// ## Arguments
            /// * `max_distance` - Maximum distance of a stop.
            /// * `min_duration` - Minimum duration of a stop.
            ///
            /// ## Returns
            /// A sequence set of stops.
            ///
            /// MEOS Functions:
            ///     `temporal_stops`
            fn stops(&self, max_distance: f64, min_duration: TimeDelta) -> Self::TSS {
                let interval = create_interval(min_duration);
                unsafe {
                    <Self::TSS as TSequenceSet>::from_inner(
                        meos_sys::temporal_stops(
                            self.inner(),
                            max_distance,
                            &raw const interval,
                        ),
                    )
                }
            }
            /// Returns whether the values of `self` are always less than `other`.
            ///
            /// # Arguments
            ///
            /// * `other` - Another temporal instance to compare against.
            ///
            /// # Returns
            ///
            /// `true` if the values of `self` are always less than `other`, `false` otherwise.
            fn always_less(&self, other: &Self) -> Option<bool> {
                let result = unsafe {
                    meos_sys::always_lt_temporal_temporal(self.inner(), other.inner())
                };
                if result != -1 { Some(result == 1) } else { None }
            }
            /// Returns whether the values of `self` are always less than or equal to `other`.
            ///
            /// # Arguments
            ///
            /// * `other` - Another temporal instance to compare against.
            ///
            /// # Returns
            ///
            /// `true` if the values of `self` are always less than or equal to `other`, `false` otherwise.
            fn always_less_or_equal(&self, other: &Self) -> Option<bool> {
                let result = unsafe {
                    meos_sys::always_le_temporal_temporal(self.inner(), other.inner())
                };
                if result != -1 { Some(result == 1) } else { None }
            }
            /// Returns whether the values of `self` are always equal to `other`.
            ///
            /// # Arguments
            ///
            /// * `other` - Another temporal instance to compare against.
            ///
            /// # Returns
            ///
            /// `true` if the values of `self` are always equal to `other`, `false` otherwise.
            fn always_equal(&self, other: &Self) -> Option<bool> {
                let result = unsafe {
                    meos_sys::always_eq_temporal_temporal(self.inner(), other.inner())
                };
                if result != -1 { Some(result == 1) } else { None }
            }
            /// Returns whether the values of `self` are always not equal to `other`.
            ///
            /// # Arguments
            ///
            /// * `other` - Another temporal instance to compare against.
            ///
            /// # Returns
            ///
            /// `true` if the values of `self` are always not equal to `other`, `false` otherwise.
            fn always_not_equal(&self, other: &Self) -> Option<bool> {
                let result = unsafe {
                    meos_sys::always_ne_temporal_temporal(self.inner(), other.inner())
                };
                if result != -1 { Some(result == 1) } else { None }
            }
            /// Returns whether the values of `self` are always greater than or equal to `other`.
            ///
            /// # Arguments
            ///
            /// * `other` - Another temporal instance to compare against.
            ///
            /// # Returns
            ///
            /// `true` if the values of `self` are always greater than or equal to `other`, `false` otherwise.
            fn always_greater_or_equal(&self, other: &Self) -> Option<bool> {
                let result = unsafe {
                    meos_sys::always_ge_temporal_temporal(self.inner(), other.inner())
                };
                if result != -1 { Some(result == 1) } else { None }
            }
            /// Returns whether the values of `self` are always greater than `other`.
            ///
            /// # Arguments
            ///
            /// * `other` - Another temporal instance to compare against.
            ///
            /// # Returns
            ///
            /// `true` if the values of `self` are always greater than `other`, `false` otherwise.
            fn always_greater(&self, other: &Self) -> Option<bool> {
                let result = unsafe {
                    meos_sys::always_gt_temporal_temporal(self.inner(), other.inner())
                };
                if result != -1 { Some(result == 1) } else { None };
                if result != -1 { Some(result == 1) } else { None }
            }
            /// Returns whether the values of `self` are ever less than `other`.
            ///
            /// # Arguments
            ///
            /// * `other` - Another temporal instance to compare against.
            ///
            /// # Returns
            ///
            /// `true` if the values of `self` are ever less than `other`, `false` otherwise.
            fn ever_less(&self, other: &Self) -> Option<bool> {
                let result = unsafe {
                    meos_sys::ever_lt_temporal_temporal(self.inner(), other.inner())
                };
                if result != -1 { Some(result == 1) } else { None }
            }
            /// Returns whether the values of `self` are ever less than or equal to `other`.
            ///
            /// # Arguments
            ///
            /// * `other` - Another temporal instance to compare against.
            ///
            /// # Returns
            ///
            /// `true` if the values of `self` are ever less than or equal to `other`, `false` otherwise.
            fn ever_less_or_equal(&self, other: &Self) -> Option<bool> {
                let result = unsafe {
                    meos_sys::ever_le_temporal_temporal(self.inner(), other.inner())
                };
                if result != -1 { Some(result == 1) } else { None }
            }
            /// Returns whether the values of `self` are ever equal to `other`.
            ///
            /// # Arguments
            ///
            /// * `other` - Another temporal instance to compare against.
            ///
            /// # Returns
            ///
            /// `true` if the values of `self` are ever equal to `other`, `false` otherwise.
            fn ever_equal(&self, other: &Self) -> Option<bool> {
                let result = unsafe {
                    meos_sys::ever_eq_temporal_temporal(self.inner(), other.inner())
                };
                if result != -1 { Some(result == 1) } else { None }
            }
            /// Returns whether the values of `self` are ever not equal to `other`.
            ///
            /// # Arguments
            ///
            /// * `other` - Another temporal instance to compare against.
            ///
            /// # Returns
            ///
            /// `true` if the values of `self` are ever not equal to `other`, `false` otherwise.
            fn ever_not_equal(&self, other: &Self) -> Option<bool> {
                let result = unsafe {
                    meos_sys::ever_ne_temporal_temporal(self.inner(), other.inner())
                };
                if result != -1 { Some(result == 1) } else { None }
            }
            /// Returns whether the values of `self` are ever greater than or equal to `other`.
            ///
            /// # Arguments
            ///
            /// * `other` - Another temporal instance to compare against.
            ///
            /// # Returns
            ///
            /// `true` if the values of `self` are ever greater than or equal to `other`, `false` otherwise.
            fn ever_greater_or_equal(&self, other: &Self) -> Option<bool> {
                let result = unsafe {
                    meos_sys::ever_ge_temporal_temporal(self.inner(), other.inner())
                };
                if result != -1 { Some(result == 1) } else { None }
            }
            /// Returns whether the values of `self` are ever greater than `other`.
            ///
            /// # Arguments
            ///
            /// * `other` - Another temporal instance to compare against.
            ///
            /// # Returns
            ///
            /// `true` if the values of `self` are ever greater than `other`, `false` otherwise.
            fn ever_greater(&self, other: &Self) -> Option<bool> {
                let result = unsafe {
                    meos_sys::ever_gt_temporal_temporal(self.inner(), other.inner())
                };
                if result != -1 { Some(result == 1) } else { None }
            }
            /// Returns whether the values of `self` are always less than `value`.
            ///
            /// # Arguments
            ///
            /// * `value` - Value to compare against.
            ///
            /// # Returns
            ///
            /// `true` if the values of `self` are always less than `value`, `false` otherwise.
            fn always_less_than_value(&self, value: Self::Type) -> Option<bool>;
            /// Returns whether the values of `self` are always less than or equal to `value`.
            ///
            /// # Arguments
            ///
            /// * `value` - Value to compare against.
            ///
            /// # Returns
            ///
            /// `true` if the values of `self` are always less than or equal to `value`, `false` otherwise.
            fn always_less_or_equal_than_value(&self, value: Self::Type) -> Option<bool>;
            /// Returns whether the values of `self` are always equal to `value`.
            ///
            /// # Arguments
            ///
            /// * `value` - Value to compare against.
            ///
            /// # Returns
            ///
            /// `true` if the values of `self` are always equal to `value`, `false` otherwise.
            fn always_equal_than_value(&self, value: Self::Type) -> Option<bool>;
            /// Returns whether the values of `self` are always not equal to `value`.
            ///
            /// # Arguments
            ///
            /// * `value` - Value to compare against.
            ///
            /// # Returns
            ///
            /// `true` if the values of `self` are always not equal to `value`, `false` otherwise.
            fn always_not_equal_than_value(&self, value: Self::Type) -> Option<bool>;
            /// Returns whether the values of `self` are always greater than or equal to `value`.
            ///
            /// # Arguments
            ///
            /// * `value` - Value to compare against.
            ///
            /// # Returns
            ///
            /// `true` if the values of `self` are always greater than or equal to `value`, `false` otherwise.
            fn always_greater_or_equal_than_value(
                &self,
                value: Self::Type,
            ) -> Option<bool>;
            /// Returns whether the values of `self` are always greater than `value`.
            ///
            /// # Arguments
            ///
            /// * `value` - Value to compare against.
            ///
            /// # Returns
            ///
            /// `true` if the values of `self` are always greater than `value`, `false` otherwise.
            fn always_greater_than_value(&self, value: Self::Type) -> Option<bool>;
            /// Returns whether the values of `self` are ever less than `value`.
            ///
            /// # Arguments
            ///
            /// * `value` - Value to compare against.
            ///
            /// # Returns
            ///
            /// `true` if the values of `self` are ever less than `value`, `false` otherwise.
            fn ever_less_than_value(&self, value: Self::Type) -> Option<bool>;
            /// Returns whether the values of `self` are ever less than or equal to `value`.
            ///
            /// # Arguments
            ///
            /// * `value` - Value to compare against.
            ///
            /// # Returns
            ///
            /// `true` if the values of `self` are ever less than or equal to `value`, `false` otherwise.
            fn ever_less_or_equal_than_value(&self, value: Self::Type) -> Option<bool>;
            /// Returns whether the values of `self` are ever equal to `value`.
            ///
            /// # Arguments
            ///
            /// * `value` - Value to compare against.
            ///
            /// # Returns
            ///
            /// `true` if the values of `self` are ever equal to `value`, `false` otherwise.
            fn ever_equal_than_value(&self, value: Self::Type) -> Option<bool>;
            /// Returns whether the values of `self` are ever not equal to `value`.
            ///
            /// # Arguments
            ///
            /// * `value` - Value to compare against.
            ///
            /// # Returns
            ///
            /// `true` if the values of `self` are ever not equal to `value`, `false` otherwise.
            fn ever_not_equal_than_value(&self, value: Self::Type) -> Option<bool>;
            /// Returns whether the values of `self` are ever greater than or equal to `value`.
            ///
            /// # Arguments
            ///
            /// * `value` - Value to compare against.
            ///
            /// # Returns
            ///
            /// `true` if the values of `self` are ever greater than or equal to `value`, `false` otherwise.
            fn ever_greater_or_equal_than_value(
                &self,
                value: Self::Type,
            ) -> Option<bool>;
            /// Returns whether the values of `self` are ever greater than `value`.
            ///
            /// # Arguments
            ///
            /// * `value` - Value to compare against.
            ///
            /// # Returns
            ///
            /// `true` if the values of `self` are ever greater than `value`, `false` otherwise.
            fn ever_greater_than_value(&self, value: Self::Type) -> Option<bool>;
        }
        pub(crate) use impl_simple_traits_for_temporal;
        pub(crate) use impl_always_and_ever_value_equality_functions;
        pub(crate) use impl_always_and_ever_value_functions_with_ordering;
    }
    pub mod tinstant {
        use chrono::{DateTime, TimeZone, Utc};
        use super::temporal::Temporal;
        pub trait TInstant: Temporal {
            fn from_inner(inner: *mut meos_sys::TInstant) -> Self;
            fn inner_as_tinstant(&self) -> *const meos_sys::TInstant;
            /// ## Returns
            /// The value of the temporal instant.
            fn value(&self) -> Self::Type {
                self.start_value()
            }
            /// Creates a temporal object from a value and a time object.
            ///
            /// ## Arguments
            /// * `value` - Base value.
            /// * `timestamp` - Time object to use as the temporal dimension.
            ///
            /// ## Returns
            /// A new temporal object.
            fn from_value_and_timestamp<Tz: TimeZone>(
                value: Self::Type,
                timestamp: DateTime<Tz>,
            ) -> Self;
            /// Returns the timestamp of the temporal instant.
            ///
            /// ## Returns
            /// A `chrono::DateTime` object.
            fn timestamp(&self) -> DateTime<Utc> {
                self.start_timestamp()
            }
        }
    }
    pub mod tsequence {
        use chrono::TimeZone;
        use super::{
            interpolation::TInterpolation, temporal::Temporal, tinstant::TInstant,
        };
        pub trait TSequence: Temporal {
            /// ## Arguments
            /// * `values` - A slice of temporal instants (`TInstant`) that represent the values of the temporal sequence.
            /// * `interpolation` - The interpolation method to use for the temporal sequence.
            ///
            /// ## Returns
            /// Returns an instance of a type implementing the `TSequence` trait.
            ///
            /// ## Note
            /// We assume that the lower bound will be inclusive and
            /// the upper one exclusive (except for Discrete interpolations and instantaneous sequences, where it's inclusive), if you find yourself needing another variant, report it.
            fn new<Tz: TimeZone>(
                values: &[Self::TI],
                interpolation: TInterpolation,
            ) -> Self {
                let mut t_list: Vec<_> = values
                    .iter()
                    .map(Self::TI::inner_as_tinstant)
                    .collect();
                let upper_inclusive = match interpolation {
                    TInterpolation::Discrete => true,
                    _ => false,
                } || values.len() == 1;
                TSequence::from_inner(unsafe {
                    meos_sys::tsequence_make(
                        t_list.as_mut_ptr(),
                        t_list.len() as i32,
                        true,
                        upper_inclusive,
                        interpolation as u32,
                        true,
                    )
                })
            }
            fn from_inner(inner: *const meos_sys::TSequence) -> Self;
            fn inner_as_tsequence(&self) -> *const meos_sys::TSequence;
            fn is_lower_inclusive(&self) -> bool {
                unsafe { meos_sys::temporal_lower_inc(self.inner()) }
            }
            fn is_upper_inclusive(&self) -> bool {
                unsafe { meos_sys::temporal_upper_inc(self.inner()) }
            }
        }
    }
    pub mod tsequence_set {
        use super::{temporal::Temporal, tsequence::TSequence};
        pub trait TSequenceSet: Temporal {
            /// ## Arguments
            /// * `values` - A slice of temporal sequences (`TSequence`) that represent the values of the temporal sequence set.
            /// * `normalize` - A boolean indicating whether to normalize the temporal sequence set.
            ///
            /// ## Returns
            /// Returns an instance of a type implementing the `TSequenceSet` trait.
            fn new(values: &[Self::TS], normalize: bool) -> Self {
                let mut t_list: Vec<_> = values
                    .iter()
                    .map(TSequence::inner_as_tsequence)
                    .collect();
                TSequenceSet::from_inner(unsafe {
                    meos_sys::tsequenceset_make(
                        t_list.as_mut_ptr(),
                        t_list.len() as i32,
                        normalize,
                    )
                })
            }
            fn from_inner(inner: *const meos_sys::TSequenceSet) -> Self;
        }
    }
    /// Taken from https://json-c.github.io/json-c/json-c-0.10/doc/html/json__object_8h.html#a3294cb92765cdeb497cfd346644d1059
    pub enum JSONCVariant {
        Plain,
        Spaced,
        Pretty,
    }
}
pub(crate) mod utils {
    use chrono::{DateTime, TimeZone, Utc};
    use crate::collections::datetime::MICROSECONDS_UNTIL_2000;
    pub(crate) fn create_interval(t: chrono::TimeDelta) -> meos_sys::Interval {
        let time_in_microseconds = t.num_microseconds().unwrap_or(0);
        let total_days = t.num_days() as i32;
        meos_sys::Interval {
            time: time_in_microseconds,
            day: total_days,
            month: 0,
        }
    }
    pub(crate) fn from_interval(interval: meos_sys::Interval) -> chrono::TimeDelta {
        let time_in_microseconds = interval.time;
        let days = interval.day as i64;
        let months = interval.month as i64;
        chrono::TimeDelta::microseconds(time_in_microseconds)
            + chrono::TimeDelta::days(days + months * 30)
    }
    pub(crate) fn to_meos_timestamp<Tz: TimeZone>(dt: &DateTime<Tz>) -> i64 {
        dt.timestamp_micros() - MICROSECONDS_UNTIL_2000
    }
    pub(crate) fn from_meos_timestamp(
        timestamp: meos_sys::TimestampTz,
    ) -> DateTime<Utc> {
        DateTime::from_timestamp_micros(timestamp + MICROSECONDS_UNTIL_2000)
            .expect("Failed to parse DateTime")
    }
}
static START: Once = Once::new();
extern "C" fn finalize() {
    unsafe {
        meos_sys::meos_finalize();
    }
}
pub trait BoundingBox: Collection {}
impl<T> BoundingBox for T
where
    T: MeosBox,
{}
unsafe extern "C" fn error_handler(
    _error_level: i32,
    _error_code: i32,
    message: *const i8,
) {
    let message = CStr::from_ptr(message).to_str().unwrap();
    {
        #[cold]
        #[track_caller]
        #[inline(never)]
        #[rustc_const_panic_str]
        #[rustc_do_not_const_check]
        const fn panic_cold_display<T: ::core::fmt::Display>(arg: &T) -> ! {
            ::core::panicking::panic_display(arg)
        }
        panic_cold_display(&message);
    };
}
pub fn init() {
    START
        .call_once(|| unsafe {
            let ptr = CString::new("UTC").unwrap();
            meos_sys::meos_initialize(ptr.as_ptr(), Some(error_handler));
            libc::atexit(finalize);
        });
}
#[repr(transparent)]
pub struct WKBVariant {
    bits: u8,
}
#[automatically_derived]
impl ::core::clone::Clone for WKBVariant {
    #[inline]
    fn clone(&self) -> WKBVariant {
        let _: ::core::clone::AssertParamIsClone<u8>;
        *self
    }
}
#[automatically_derived]
impl ::core::marker::Copy for WKBVariant {}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for WKBVariant {}
#[automatically_derived]
impl ::core::cmp::PartialEq for WKBVariant {
    #[inline]
    fn eq(&self, other: &WKBVariant) -> bool {
        self.bits == other.bits
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for WKBVariant {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {
        let _: ::core::cmp::AssertParamIsEq<u8>;
    }
}
#[automatically_derived]
impl ::core::cmp::PartialOrd for WKBVariant {
    #[inline]
    fn partial_cmp(
        &self,
        other: &WKBVariant,
    ) -> ::core::option::Option<::core::cmp::Ordering> {
        ::core::cmp::PartialOrd::partial_cmp(&self.bits, &other.bits)
    }
}
#[automatically_derived]
impl ::core::cmp::Ord for WKBVariant {
    #[inline]
    fn cmp(&self, other: &WKBVariant) -> ::core::cmp::Ordering {
        ::core::cmp::Ord::cmp(&self.bits, &other.bits)
    }
}
#[automatically_derived]
impl ::core::hash::Hash for WKBVariant {
    #[inline]
    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
        ::core::hash::Hash::hash(&self.bits, state)
    }
}
#[allow(non_upper_case_globals)]
impl WKBVariant {
    /// Little endian encoding
    pub const NDR: WKBVariant = Self {
        bits: meos_sys::WKB_NDR as u8,
    };
    /// Big endian encoding
    pub const XDR: WKBVariant = Self {
        bits: meos_sys::WKB_XDR as u8,
    };
    /// Extended variant
    pub const Extended: WKBVariant = Self {
        bits: meos_sys::WKB_EXTENDED as u8,
    };
    /// Returns the underlying bits of the bitmask.
    #[inline]
    pub const fn bits(&self) -> u8 {
        self.bits
    }
    /// Returns a bitmask that contains all values.
    ///
    /// This will include bits that do not have any flags.
    /// Use `::all_flags()` if you only want to use flags.
    #[inline]
    pub const fn all_bits() -> Self {
        Self { bits: !0 }
    }
    /// Returns a bitmask that contains all flags.
    #[inline]
    pub const fn all_flags() -> Self {
        Self {
            bits: Self::NDR.bits | Self::XDR.bits | Self::Extended.bits | 0,
        }
    }
    /// Returns `true` if the bitmask contains all values.
    ///
    /// This will check for `bits == !0`,
    /// use `.is_all_flags()` if you only want to check for all flags
    #[inline]
    pub const fn is_all_bits(&self) -> bool {
        self.bits == !0
    }
    /// Returns `true` if the bitmask contains all flags.
    ///
    /// This will fail if any unused bit is set,
    /// consider using `.truncate()` first.
    #[inline]
    pub const fn is_all_flags(&self) -> bool {
        self.bits == Self::all_flags().bits
    }
    /// Returns a bitmask that contains all values.
    ///
    /// This will include bits that do not have any flags.
    /// Use `::all_flags()` if you only want to use flags.
    #[inline]
    #[deprecated(note = "Please use the `::all_bits()` constructor")]
    pub const fn all() -> Self {
        Self::all_bits()
    }
    /// Returns `true` if the bitmask contains all values.
    ///
    /// This will check for `bits == !0`,
    /// use `.is_all_flags()` if you only want to check for all flags
    #[inline]
    #[deprecated(note = "Please use the `.is_all_bits()` method")]
    pub const fn is_all(&self) -> bool {
        self.is_all_bits()
    }
    /// Returns a bitmask that contains all flags.
    #[inline]
    #[deprecated(note = "Please use the `::all_flags()` constructor")]
    pub const fn full() -> Self {
        Self::all_flags()
    }
    /// Returns `true` if the bitmask contains all flags.
    ///
    /// This will fail if any unused bit is set,
    /// consider using `.truncate()` first.
    #[inline]
    #[deprecated(note = "Please use the `.is_all_flags()` method")]
    pub const fn is_full(&self) -> bool {
        self.is_all_flags()
    }
    /// Returns a bitmask that does not contain any values.
    #[inline]
    pub const fn none() -> Self {
        Self { bits: 0 }
    }
    /// Returns `true` if the bitmask does not contain any values.
    #[inline]
    pub const fn is_none(&self) -> bool {
        self.bits == 0
    }
    /// Returns a bitmask that only has bits corresponding to flags
    #[inline]
    pub const fn truncate(&self) -> Self {
        Self {
            bits: self.bits & Self::all_flags().bits,
        }
    }
    /// Returns `true` if `self` intersects with any value in `other`,
    /// or if `other` does not contain any values.
    ///
    /// This is equivalent to `(self & other) != 0 || other == 0`.
    #[inline]
    pub const fn intersects(&self, other: Self) -> bool {
        (self.bits & other.bits) != 0 || other.bits == 0
    }
    /// Returns `true` if `self` contains all values of `other`.
    ///
    /// This is equivalent to  `(self & other) == other`.
    #[inline]
    pub const fn contains(&self, other: Self) -> bool {
        (self.bits & other.bits) == other.bits
    }
    /// Returns the bitwise NOT of the bitmask.
    #[inline]
    pub const fn not(self) -> Self {
        Self { bits: !self.bits }
    }
    /// Returns the bitwise AND of the bitmask.
    #[inline]
    pub const fn and(self, other: Self) -> Self {
        Self {
            bits: self.bits & other.bits,
        }
    }
    /// Returns the bitwise OR of the bitmask.
    #[inline]
    pub const fn or(self, other: Self) -> Self {
        Self {
            bits: self.bits | other.bits,
        }
    }
    /// Returns the bitwise XOR of the bitmask.
    #[inline]
    pub const fn xor(self, other: Self) -> Self {
        Self {
            bits: self.bits ^ other.bits,
        }
    }
}
impl core::ops::Not for WKBVariant {
    type Output = Self;
    #[inline]
    fn not(self) -> Self::Output {
        Self {
            bits: core::ops::Not::not(self.bits),
        }
    }
}
impl core::ops::BitAnd for WKBVariant {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            bits: core::ops::BitAnd::bitand(self.bits, rhs.bits),
        }
    }
}
impl core::ops::BitAndAssign for WKBVariant {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        core::ops::BitAndAssign::bitand_assign(&mut self.bits, rhs.bits)
    }
}
impl core::ops::BitOr for WKBVariant {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            bits: core::ops::BitOr::bitor(self.bits, rhs.bits),
        }
    }
}
impl core::ops::BitOrAssign for WKBVariant {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        core::ops::BitOrAssign::bitor_assign(&mut self.bits, rhs.bits)
    }
}
impl core::ops::BitXor for WKBVariant {
    type Output = Self;
    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self {
            bits: core::ops::BitXor::bitxor(self.bits, rhs.bits),
        }
    }
}
impl core::ops::BitXorAssign for WKBVariant {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        core::ops::BitXorAssign::bitxor_assign(&mut self.bits, rhs.bits)
    }
}
impl From<u8> for WKBVariant {
    #[inline]
    fn from(val: u8) -> Self {
        Self { bits: val }
    }
}
impl From<WKBVariant> for u8 {
    #[inline]
    fn from(val: WKBVariant) -> u8 {
        val.bits
    }
}
impl PartialEq<u8> for WKBVariant {
    #[inline]
    fn eq(&self, other: &u8) -> bool {
        self.bits == *other
    }
}
impl core::fmt::Debug for WKBVariant {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("WKBVariant").field("bits", &self.bits).finish()
    }
}
impl core::fmt::Binary for WKBVariant {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Binary::fmt(&self.bits, f)
    }
}
impl core::fmt::LowerHex for WKBVariant {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::LowerHex::fmt(&self.bits, f)
    }
}
impl core::fmt::UpperHex for WKBVariant {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::UpperHex::fmt(&self.bits, f)
    }
}
impl core::fmt::Octal for WKBVariant {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Octal::fmt(&self.bits, f)
    }
}
