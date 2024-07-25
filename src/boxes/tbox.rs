use std::{
    ffi::{CStr, CString},
    ptr,
    str::FromStr,
};

use chrono::{DateTime, Utc};

use crate::{
    collections::{
        base::span::Span,
        datetime::tstz_span::TsTzSpan,
        number::{float_span::FloatSpan, int_span::IntSpan},
    },
    errors::ParseError,
};

pub struct TBox {
    _inner: *mut meos_sys::TBox,
}

impl TBox {
    fn inner(&self) -> *mut meos_sys::TBox {
        self._inner
    }

    fn from_inner(inner: *mut meos_sys::TBox) -> Self {
        Self { _inner: inner }
    }

    pub fn from_wkb(wkb: &[u8]) -> Self {
        unsafe { Self::from_inner(meos_sys::tbox_from_wkb(wkb.as_ptr(), wkb.len())) }
    }

    pub fn from_hexwkb(hexwkb: &str) -> Self {
        let c_hexwkb = CString::new(hexwkb).unwrap();
        unsafe {
            let inner = meos_sys::tbox_from_hexwkb(c_hexwkb.as_ptr());
            Self::from_inner(inner)
        }
    }

    pub fn from_int(value: i32) -> Self {
        unsafe { Self::from_inner(meos_sys::int_to_tbox(value)) }
    }

    pub fn from_float(value: f64) -> Self {
        unsafe { Self::from_inner(meos_sys::float_to_tbox(value)) }
    }

    pub fn from_span(value: impl Span) -> Self {
        unsafe { Self::from_inner(meos_sys::span_to_tbox(value.inner())) }
    }

    pub fn from_time(time: DateTime<Utc>) -> Self {
        // Convert DateTime<Utc> to the expected timestamp format for MEOS
        let timestamptz = time.timestamp();
        unsafe { Self::from_inner(meos_sys::timestamptz_to_tbox(timestamptz)) }
    }

    // pub fn from_tnumber(temporal: TNumber) -> Self {
    //     unsafe {
    //         let inner = tnumber_to_meos_sys::tbox(temporal.inner);
    //         Self::from_inner(inner)
    //     }
    // }

    pub fn to_intspan(&self) -> FloatSpan {
        unsafe { FloatSpan::from_inner(meos_sys::tbox_to_intspan(self._inner)) }
    }

    pub fn to_floatspan(&self) -> FloatSpan {
        unsafe { FloatSpan::from_inner(meos_sys::tbox_to_floatspan(self._inner)) }
    }

    pub fn to_tstzspan(&self) -> TsTzSpan {
        unsafe { TsTzSpan::from_inner(meos_sys::tbox_to_tstzspan(self._inner)) }
    }

    pub fn as_wkb(&self) -> Vec<u8> {
        unsafe {
            let mut size: usize = 0;
            let ptr = meos_sys::tbox_as_wkb(self._inner, 4, &mut size);
            Vec::from_raw_parts(ptr as *mut u8, size, size)
        }
    }

    pub fn as_hexwkb(&self) -> String {
        unsafe {
            let hexwkb_ptr = meos_sys::tbox_as_hexwkb(self.inner(), 1, std::ptr::null_mut());
            CStr::from_ptr(hexwkb_ptr).to_str().unwrap().to_owned()
        }
    }

    // ------------------------- Accessors -------------------------------------

    pub fn has_x(&self) -> bool {
        unsafe { meos_sys::tbox_hasx(self.inner()) }
    }

    pub fn has_t(&self) -> bool {
        unsafe { meos_sys::tbox_hast(self.inner()) }
    }

    pub fn xmin(&self) -> Option<f64> {
        unsafe {
            let mut value = 0.0;
            let ptr: *mut f64 = ptr::addr_of_mut!(value);
            if meos_sys::tbox_xmin(self.inner(), ptr) {
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
            if meos_sys::tbox_xmax(self.inner(), ptr) {
                Some(value)
            } else {
                None
            }
        }
    }

    pub fn xmin_inc(&self) -> bool {
        unsafe {
            let mut is_inclusive = false;
            let ptr: *mut bool = ptr::addr_of_mut!(is_inclusive);
            meos_sys::tbox_xmin_inc(self.inner(), ptr);
            is_inclusive
        }
    }

    pub fn xmax_inc(&self) -> bool {
        unsafe {
            let mut is_inclusive = false;
            let ptr: *mut bool = ptr::addr_of_mut!(is_inclusive);
            meos_sys::tbox_xmax_inc(self.inner(), ptr);
            is_inclusive
        }
    }

    pub fn tmin(&self) -> Option<DateTime<Utc>> {
        unsafe {
            let mut value: i64 = 0;
            let ptr: *mut i64 = ptr::addr_of_mut!(value);
            if meos_sys::tbox_tmin(self.inner(), ptr) {
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
            if meos_sys::tbox_tmax(self.inner(), ptr) {
                DateTime::from_timestamp_micros(value)
            } else {
                None
            }
        }
    }

    pub fn tmin_inc(&self) -> bool {
        unsafe {
            let mut is_inclusive = false;
            let ptr: *mut bool = ptr::addr_of_mut!(is_inclusive);
            meos_sys::tbox_tmin_inc(self.inner(), ptr);
            is_inclusive
        }
    }

    pub fn tmax_inc(&self) -> bool {
        unsafe {
            let mut is_inclusive = false;
            let ptr: *mut bool = ptr::addr_of_mut!(is_inclusive);
            meos_sys::tbox_tmax_inc(self.inner(), ptr);
            is_inclusive
        }
    }

    // ------------------------- Transformation --------------------------------

    pub fn expand(&self, other: TBox) -> TBox {
        let result = match other {
            TBox::IntValue(val) => unsafe { meos_sys::tbox_expand_int(self.inner(), val) },
            TBox::FloatValue(val) => unsafe { meos_sys::tbox_expand_float(self.inner(), val) },
            TBox::Duration(val) => unsafe { meos_sys::tbox_expand_time(self.inner(), val) },
        };
        TBox::new(result)
    }

    pub fn shift_value(&self, delta: f64) -> TBox {
        let result = unsafe { meos_sys::span_shift_scale(self.inner(), delta, 1.0) };
        TBox::new(result)
    }

    pub fn shift_time(&self, delta: Duration) -> TBox {
        let result =
            unsafe { meos_sys::tstzspan_shift_scale(self.inner(), delta.as_secs() as i64, 1.0) };
        TBox::new(result)
    }

    pub fn scale_value(&self, width: f64) -> TBox {
        let result = unsafe { meos_sys::span_shift_scale(self.inner(), 0.0, width) };
        TBox::new(result)
    }

    pub fn scale_time(&self, duration: Duration) -> TBox {
        let result =
            unsafe { meos_sys::tstzspan_shift_scale(self.inner(), 0, duration.as_secs() as i64) };
        TBox::new(result)
    }

    pub fn shift_scale_value(&self, shift: Option<f64>, width: Option<f64>) -> TBox {
        let (shift, width) = (shift.unwrap_or(0.0), width.unwrap_or(0.0));
        let result = unsafe { meos_sys::span_shift_scale(self.inner(), shift, width) };
        TBox::new(result)
    }

    pub fn shift_scale_time(&self, shift: Option<Duration>, duration: Option<Duration>) -> TBox {
        let (shift, duration) = (
            shift.unwrap_or_else(|| Duration::new(0, 0)),
            duration.unwrap_or_else(|| Duration::new(0, 0)),
        );
        let result = unsafe {
            meos_sys::tstzspan_shift_scale(
                self.inner(),
                shift.as_secs() as i64,
                duration.as_secs() as i64,
            )
        };
        TBox::new(result)
    }

    pub fn round(&self, max_decimals: i32) -> TBox {
        let result = unsafe { meos_sys::tbox_round(self.inner(), max_decimals) };
        TBox::new(result)
    }

    // ------------------------- Set Operations --------------------------------

    pub fn union(&self, other: &TBox, strict: bool) -> TBox {
        let result = unsafe { meos_sys::union_tbox_tbox(self.inner(), other.inner(), strict) };
        TBox::new(result)
    }

    pub fn intersection(&self, other: &TBox) -> Option<TBox> {
        let result = unsafe { meos_sys::intersection_tbox_tbox(self.inner(), other.inner()) };
        if result.is_null() {
            None
        } else {
            Some(TBox::new(result))
        }
    }

    // ------------------------- Topological Operations ------------------------

    pub fn is_adjacent(&self, other: &TBox) -> bool {
        unsafe { meos_sys::adjacent_tbox_tbox(self.inner(), other.inner()) }
    }

    pub fn is_contained_in(&self, container: &TBox) -> bool {
        unsafe { meos_sys::contained_tbox_tbox(self.inner(), container.inner()) }
    }

    pub fn contains(&self, content: &TBox) -> bool {
        unsafe { meos_sys::contains_tbox_tbox(self.inner(), content.inner()) }
    }

    pub fn overlaps(&self, other: &TBox) -> bool {
        unsafe { meos_sys::overlaps_tbox_tbox(self.inner(), other.inner()) }
    }

    pub fn is_same(&self, other: &TBox) -> bool {
        unsafe { meos_sys::same_tbox_tbox(self.inner(), other.inner()) }
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
    /// # use meos::collections::base::span::Span;
    /// # use meos::collections::number::int_span::IntSpan;
    /// # use meos::collections::datetime::tstz_span::TsTzSpan;
    /// use std::str::FromStr;
    /// # use meos::init;
    /// # init();
    ///
    /// let tbox: TBox = "TBOXINT XT([0, 10),[2020-06-01, 2020-06-05])".parse().expect("Failed to parse span");
    /// let value_span: IntSpan = (&tbox).into();
    /// let temporal_span: TsTzSpan = (&tbox).into();
    /// assert_eq!(value_span, (0..10).into());
    /// assert_eq!(temporal_span, TsTzSpan::from_str("[2020-06-01, 2020-06-05]").unwrap());
    /// ```
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        CString::new(string).map_err(|_| ParseError).map(|string| {
            let inner = unsafe { meos_sys::tbox_in(string.as_ptr()) };
            Self::from_inner(inner)
        })
    }

    // ------------------------- Position Operations ---------------------------
}

impl From<String> for TBox {
    /// Converts a `String` into a `TBox`.
    ///
    /// ## Arguments
    /// * `value` - A `String` containing the representation of a `TBox`.
    ///
    /// ## Returns
    /// * A `TBox` instance.
    ///
    /// ## Panics
    /// * Panics if the string cannot be parsed into a `TBox`.
    ///
    /// ## Example
    /// ```
    /// # use meos::boxes::tbox::TBox;
    /// # use meos::collections::base::span::Span;
    /// # use meos::collections::datetime::tstz_span::TsTzSpan;
    /// # use meos::collections::number::int_span::IntSpan;
    /// # use std::string::String;
    /// # use meos::init;
    /// use std::str::FromStr;
    ///
    /// # init();
    ///
    /// let tbox: TBox = String::from("TBOXINT XT([0, 10),[2020-06-01, 2020-06-05])").into();
    /// let value_span: IntSpan = (&tbox).into();
    /// let temporal_span: TsTzSpan = (&tbox).into();
    /// assert_eq!(value_span, (0..10).into());
    /// assert_eq!(temporal_span, TsTzSpan::from_str("[2020-06-01, 2020-06-05]").unwrap());
    /// ```
    fn from(value: String) -> Self {
        TBox::from_str(&value).expect("Failed to parse the tbox")
    }
}

impl From<&TBox> for IntSpan {
    fn from(tbox: &TBox) -> Self {
        unsafe { IntSpan::from_inner(meos_sys::tbox_to_intspan(tbox.inner())) }
    }
}

impl From<&TBox> for FloatSpan {
    fn from(tbox: &TBox) -> Self {
        unsafe { FloatSpan::from_inner(meos_sys::tbox_to_floatspan(tbox.inner())) }
    }
}

impl From<&TBox> for TsTzSpan {
    fn from(tbox: &TBox) -> Self {
        unsafe { TsTzSpan::from_inner(meos_sys::tbox_to_tstzspan(tbox.inner())) }
    }
}
