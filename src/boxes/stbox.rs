use std::{
    cmp,
    ffi::{c_void, CStr, CString},
    fmt::Debug,
    ptr,
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
    _inner: ptr::NonNull<meos_sys::STBox>,
}

impl MeosBox for STBox {
    fn from_wkb(wkb: &[u8]) -> Self {
        unsafe { Self::from_inner(meos_sys::stbox_from_wkb(wkb.as_ptr(), wkb.len())) }
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
        // Convert DateTime<Utc> to the expected timestamp format for MEOS
        let timestamptz = to_meos_timestamp(&time);
        unsafe { Self::from_inner(meos_sys::timestamptz_to_stbox(timestamptz)) }
    }

    fn tstzspan(&self) -> TsTzSpan {
        unsafe { TsTzSpan::from_inner(meos_sys::stbox_to_tstzspan(self.inner())) }
    }

    fn as_wkb(&self, variant: WKBVariant) -> &[u8] {
        unsafe {
            let mut size: usize = 0;
            let ptr = meos_sys::stbox_as_wkb(self.inner(), variant.into(), &mut size);
            std::slice::from_raw_parts(ptr, size)
        }
    }

    fn as_hexwkb(&self, variant: WKBVariant) -> &[u8] {
        unsafe {
            let mut size: usize = 0;
            let hexwkb_ptr = meos_sys::stbox_as_hexwkb(self.inner(), variant.into(), &mut size);

            CStr::from_ptr(hexwkb_ptr).to_bytes()
        }
    }

    // ------------------------- Accessors -------------------------------------

    fn has_x(&self) -> bool {
        unsafe { meos_sys::stbox_hasx(self.inner()) }
    }

    fn has_t(&self) -> bool {
        unsafe { meos_sys::stbox_hast(self.inner()) }
    }

    fn xmin(&self) -> Option<f64> {
        unsafe {
            let mut value = 0.0;
            let ptr: *mut f64 = ptr::addr_of_mut!(value);
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
            let ptr: *mut f64 = ptr::addr_of_mut!(value);
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
            let ptr: *mut i64 = ptr::addr_of_mut!(value);
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
            let ptr: *mut i64 = ptr::addr_of_mut!(value);
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
            let ptr: *mut bool = ptr::addr_of_mut!(is_inclusive);
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
            let ptr: *mut bool = ptr::addr_of_mut!(is_inclusive);
            if meos_sys::stbox_tmax_inc(self.inner(), ptr) {
                Some(is_inclusive)
            } else {
                None
            }
        }
    }

    // ------------------------- Transformation --------------------------------

    fn expand_time(&self, duration: TimeDelta) -> STBox {
        let interval = create_interval(duration);
        unsafe {
            Self::from_inner(meos_sys::stbox_expand_time(
                self.inner(),
                std::ptr::addr_of!(interval),
            ))
        }
    }

    fn shift_scale_time(&self, delta: Option<TimeDelta>, width: Option<TimeDelta>) -> STBox {
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

        let modified = unsafe { meos_sys::stbox_shift_scale_time(self.inner(), d, w) };
        STBox::from_inner(modified)
    }

    fn round(&self, max_decimals: i32) -> STBox {
        let result = unsafe { meos_sys::stbox_round(self.inner(), max_decimals) };
        STBox::from_inner(result)
    }

    // ------------------------- Set Operations --------------------------------

    fn union(&self, other: &STBox, strict: bool) -> Option<STBox> {
        let result = unsafe { meos_sys::union_stbox_stbox(self.inner(), other.inner(), strict) };
        if result.is_null() {
            None
        } else {
            Some(STBox::from_inner(result))
        }
    }

    fn intersection(&self, other: &STBox) -> Option<STBox> {
        let result = unsafe { meos_sys::intersection_stbox_stbox(self.inner(), other.inner()) };
        if result.is_null() {
            None
        } else {
            Some(STBox::from_inner(result))
        }
    }

    // ------------------------- Distance Operations --------------------------------

    fn nearest_approach_distance(&self, other: &STBox) -> f64 {
        unsafe { meos_sys::nad_stbox_stbox(self.inner(), other.inner()) }
    }
}

impl STBox {
    pub fn inner(&self) -> *const meos_sys::STBox {
        self._inner.as_ptr()
    }

    pub fn from_inner(inner: *mut meos_sys::STBox) -> Self {
        Self {
            _inner: ptr::NonNull::new(inner).expect("Null pointers not allowed"),
        }
    }

    #[cfg(feature = "geos")]
    pub fn from_geos(value: Geometry) -> Self {
        let v: Vec<u8> = value.to_wkb().unwrap().into();
        Self::from_wkb(&v)
    }

    // pub fn from_tpoint(temporal: TPoint) -> Self {
    //     unsafe {
    //         let inner = meos_sys::tpoint_to_stbox(temporal.inner());
    //         Self::from_inner(inner)
    //     }
    // }

    #[cfg(feature = "geos")]
    pub fn geos_geometry(&self) -> Option<Geometry> {
        // meos_sys::geo_as_ewkb(meos_sys::stbox_to_geo(box_))
        Geometry::new_from_wkb(self.as_wkb(WKBVariant::none())).ok()
    }

    // ------------------------- Transformation --------------------------------

    pub fn expand_space(&self, value: f64) -> STBox {
        unsafe { Self::from_inner(meos_sys::stbox_expand_space(self.inner(), value)) }
    }
}

impl Collection for STBox {
    impl_collection!(stbox, STBox);

    fn contains(&self, content: &Self::Type) -> bool {
        unsafe { meos_sys::contains_stbox_stbox(self.inner(), content.inner()) }
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
    /// # use meos::meos_initialize;
    /// use std::str::FromStr;
    /// # meos_initialize();
    /// let span1: STBox = STBox::from_str("STBOX ZT(((1.0,2.0,3.0),(4.0,5.0,6.0)),[2001-01-01, 2001-01-02])").unwrap();
    /// let span2: STBox = STBox::from_str("STBOX ZT(((1.0,2.0,3.0),(4.0,5.0,6.0)),[2001-01-01, 2001-01-02])").unwrap();
    /// assert_eq!(span1, span2);
    /// ```
    fn eq(&self, other: &Self) -> bool {
        unsafe { meos_sys::stbox_eq(self.inner(), other.inner()) }
    }
}

impl cmp::Eq for STBox {}

impl Debug for STBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out_str = unsafe { meos_sys::stbox_out(self.inner(), 3) };
        let c_str = unsafe { CStr::from_ptr(out_str) };
        let str = c_str.to_str().map_err(|_| std::fmt::Error)?;
        let result = f.write_str(str);
        unsafe { libc::free(out_str as *mut c_void) };
        result
    }
}

impl Clone for STBox {
    fn clone(&self) -> Self {
        unsafe { Self::from_inner(meos_sys::stbox_copy(self.inner())) }
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
    /// # use meos::meos_initialize;
    /// # meos_initialize();
    ///
    /// let stbox: STBox = "STBOX ZT(((1.0,2.0,3.0),(4.0,5.0,6.0)),[2001-01-01, 2001-01-02])".parse().expect("Failed to parse span");
    /// let temporal_span: TsTzSpan = stbox.tstzspan();
    /// assert_eq!(temporal_span, TsTzSpan::from_str("[2001-01-01, 2001-01-02]").unwrap());
    /// ```
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        CString::new(string).map_err(|_| ParseError).map(|string| {
            let inner = unsafe { meos_sys::stbox_in(string.as_ptr()) };
            Self::from_inner(inner)
        })
    }

    // ------------------------- Position Operations ---------------------------
}

impl From<&STBox> for TsTzSpan {
    fn from(stbox: &STBox) -> Self {
        unsafe { TsTzSpan::from_inner(meos_sys::stbox_to_tstzspan(stbox.inner())) }
    }
}
