use std::{
    cmp,
    ffi::{c_void, CStr, CString},
    fmt::Debug,
    ptr,
    str::FromStr,
};

use chrono::{DateTime, TimeDelta, Utc};
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
    utils::create_interval,
    WKBVariant,
};

pub struct STBox {
    _inner: *mut meos_sys::STBox,
}

impl STBox {
    fn inner(&self) -> *mut meos_sys::STBox {
        self._inner
    }

    fn from_inner(inner: *mut meos_sys::STBox) -> Self {
        Self { _inner: inner }
    }

    pub fn from_wkb(wkb: &[u8]) -> Self {
        unsafe { Self::from_inner(meos_sys::stbox_from_wkb(wkb.as_ptr(), wkb.len())) }
    }

    pub fn from_hexwkb(hexwkb: &[u8]) -> Self {
        let c_hexwkb = CString::new(hexwkb).unwrap();
        unsafe {
            let inner = meos_sys::stbox_from_hexwkb(c_hexwkb.as_ptr());
            Self::from_inner(inner)
        }
    }

    pub fn from_geos(value: Geometry) -> Self {
        let v: Vec<u8> = value.to_wkb().unwrap().into();
        Self::from_wkb(&v)
    }

    pub fn from_time(time: DateTime<Utc>) -> Self {
        // Convert DateTime<Utc> to the expected timestamp format for MEOS
        let timestamptz = time.timestamp();
        unsafe { Self::from_inner(meos_sys::timestamptz_to_stbox(timestamptz)) }
    }

    // pub fn from_tnumber(temporal: TNumber) -> Self {
    //     unsafe {
    //         let inner = tnumber_to_meos_sys::stbox(temporal.inner);
    //         Self::from_inner(inner)
    //     }
    // }

    pub fn geos_geometry(&self) -> Geometry {
        Geometry::new_from_wkb(&self.as_wkb(WKBVariant::none())).unwrap()
    }

    pub fn to_tstzspan(&self) -> TsTzSpan {
        unsafe { TsTzSpan::from_inner(meos_sys::stbox_to_tstzspan(self._inner)) }
    }

    pub fn as_wkb(&self, variant: WKBVariant) -> Vec<u8> {
        unsafe {
            let mut size: usize = 0;
            let ptr = meos_sys::stbox_as_wkb(self._inner, variant.into(), &mut size);
            Vec::from_raw_parts(ptr as *mut u8, size, size)
        }
    }

    pub fn as_hexwkb(&self, variant: WKBVariant) -> String {
        unsafe {
            let hexwkb_ptr =
                meos_sys::stbox_as_hexwkb(self.inner(), variant.into(), std::ptr::null_mut());
            CStr::from_ptr(hexwkb_ptr).to_str().unwrap().to_owned()
        }
    }

    // ------------------------- Accessors -------------------------------------

    pub fn has_x(&self) -> bool {
        unsafe { meos_sys::stbox_hasx(self.inner()) }
    }

    pub fn has_t(&self) -> bool {
        unsafe { meos_sys::stbox_hast(self.inner()) }
    }

    pub fn xmin(&self) -> Option<f64> {
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

    pub fn xmax(&self) -> Option<f64> {
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

    pub fn tmin(&self) -> Option<DateTime<Utc>> {
        unsafe {
            let mut value: i64 = 0;
            let ptr: *mut i64 = ptr::addr_of_mut!(value);
            if meos_sys::stbox_tmin(self.inner(), ptr) {
                DateTime::from_timestamp_micros(value)
            } else {
                None
            }
        }
    }

    pub fn tmax(&self) -> Option<DateTime<Utc>> {
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

    pub fn tmin_is_inclusive(&self) -> Option<bool> {
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

    pub fn tmax_is_inclusive(&self) -> Option<bool> {
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

    pub fn expand_space(&self, value: f64) -> STBox {
        unsafe { Self::from_inner(meos_sys::stbox_expand_space(self.inner(), value)) }
    }

    pub fn expand_time(&self, duration: TimeDelta) -> STBox {
        let interval = create_interval(duration);
        unsafe {
            Self::from_inner(meos_sys::stbox_expand_time(
                self.inner(),
                std::ptr::addr_of!(interval),
            ))
        }
    }

    pub fn shift_scale_time(&self, delta: Option<TimeDelta>, width: Option<TimeDelta>) -> STBox {
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

        let modified = unsafe { meos_sys::stbox_shift_scale_time(self._inner, d, w) };
        STBox::from_inner(modified)
    }

    pub fn round(&self, max_decimals: i32) -> STBox {
        let result = unsafe { meos_sys::stbox_round(self.inner(), max_decimals) };
        STBox::from_inner(result)
    }

    // ------------------------- Set Operations --------------------------------

    pub fn union(&self, other: &STBox, strict: bool) -> STBox {
        let result = unsafe { meos_sys::union_stbox_stbox(self.inner(), other.inner(), strict) };
        STBox::from_inner(result)
    }

    pub fn intersection(&self, other: &STBox) -> Option<STBox> {
        let result = unsafe { meos_sys::intersection_stbox_stbox(self.inner(), other.inner()) };
        if result.is_null() {
            None
        } else {
            Some(STBox::from_inner(result))
        }
    }

    // ------------------------- Distance Operations --------------------------------

    pub fn nearest_approach_distance(&self, other: &STBox) -> f64 {
        unsafe { meos_sys::nad_stbox_stbox(self.inner(), other.inner()) }
    }
}

impl Collection for STBox {
    impl_collection!(stbox, ());

    fn contains(&self, content: &Self::Type) -> bool {
        unsafe { meos_sys::contains_stbox_tpoint(self.inner(), std::ptr::null()) }
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
    /// # use meos::collections::number::int_span::IntSpan;
    /// # use meos::collections::datetime::tstz_span::TsTzSpan;
    /// use std::str::FromStr;
    /// # use meos::init;
    /// # init();
    ///
    /// let stbox: STBox = "STBOX ZT(((1.0,2.0,3.0),(4.0,5.0,6.0)),[2001-01-01, 2001-01-02])".parse().expect("Failed to parse span");
    /// let value_span: IntSpan = (&stbox).into();
    /// let temporal_span: TsTzSpan = (&stbox).into();
    /// assert_eq!(value_span, (0..10).into());
    /// assert_eq!(temporal_span, TsTzSpan::from_str("[2020-06-01, 2020-06-05]").unwrap());
    /// ```
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        CString::new(string).map_err(|_| ParseError).map(|string| {
            let inner = unsafe { meos_sys::stbox_in(string.as_ptr()) };
            Self::from_inner(inner)
        })
    }

    // ------------------------- Position Operations ---------------------------
}

impl From<String> for STBox {
    /// Converts a `String` into a `STBox`.
    ///
    /// ## Arguments
    /// * `value` - A `String` containing the representation of a `STBox`.
    ///
    /// ## Returns
    /// * A `STBox` instance.
    ///
    /// ## Panics
    /// * Panics if the string cannot be parsed into a `STBox`.
    ///
    /// ## Example
    /// ```
    /// # use meos::boxes::stbox::STBox;
    /// # use meos::collections::base::span::Span;
    /// # use meos::collections::datetime::tstz_span::TsTzSpan;
    /// # use meos::collections::number::int_span::IntSpan;
    /// # use std::string::String;
    /// # use meos::init;
    /// use std::str::FromStr;
    ///
    /// # init();
    ///
    /// let stbox: STBox = String::from("TBOXINT XT([0, 10),[2020-06-01, 2020-06-05])").into();
    /// let value_span: IntSpan = (&stbox).into();
    /// let temporal_span: TsTzSpan = (&stbox).into();
    /// assert_eq!(value_span, (0..10).into());
    /// assert_eq!(temporal_span, TsTzSpan::from_str("[2020-06-01, 2020-06-05]").unwrap());
    /// ```
    fn from(value: String) -> Self {
        STBox::from_str(&value).expect("Failed to parse the stbox")
    }
}

impl From<&STBox> for TsTzSpan {
    fn from(stbox: &STBox) -> Self {
        unsafe { TsTzSpan::from_inner(meos_sys::stbox_to_tstzspan(stbox.inner())) }
    }
}
