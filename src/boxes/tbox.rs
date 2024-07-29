use std::{
    cmp,
    ffi::{c_void, CStr, CString},
    fmt::Debug,
    ptr,
};

use chrono::{DateTime, TimeDelta, TimeZone, Utc};

use crate::{
    collections::{
        base::{
            collection::{impl_collection, Collection},
            span::Span,
        },
        datetime::{tstz_span::TsTzSpan, MICROSECONDS_UNTIL_2000},
        number::{float_span::FloatSpan, int_span::IntSpan, number_span::NumberSpan},
    },
    errors::ParseError,
    utils::create_interval,
    WKBVariant,
};

use super::r#box::Box as MeosBox;

pub struct TBox {
    _inner: *mut meos_sys::TBox,
}

impl MeosBox for TBox {
    fn from_wkb(wkb: &[u8]) -> Self {
        unsafe { Self::from_inner(meos_sys::tbox_from_wkb(wkb.as_ptr(), wkb.len())) }
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
        // Convert DateTime<Utc> to the expected timestamp format for MEOS
        let timestamptz = time.timestamp_micros() - MICROSECONDS_UNTIL_2000;
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
            let ptr = meos_sys::tbox_as_wkb(self._inner, variant.into(), &mut size);
            std::slice::from_raw_parts(ptr, size)
        }
    }

    fn as_hexwkb(&self, variant: WKBVariant) -> &[u8] {
        unsafe {
            let mut size: usize = 0;
            let ptr = meos_sys::tbox_as_hexwkb(self._inner, variant.into(), &mut size);
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
            let ptr: *mut f64 = ptr::addr_of_mut!(value);
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
            let ptr: *mut f64 = ptr::addr_of_mut!(value);
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
            let ptr: *mut i64 = ptr::addr_of_mut!(value);
            if meos_sys::tbox_tmin(self.inner(), ptr) {
                DateTime::from_timestamp_micros(value + MICROSECONDS_UNTIL_2000)
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
            let ptr: *mut i64 = ptr::addr_of_mut!(value);
            if meos_sys::tbox_tmax(self.inner(), ptr) {
                DateTime::from_timestamp_micros(value + MICROSECONDS_UNTIL_2000)
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
            let ptr: *mut bool = ptr::addr_of_mut!(is_inclusive);
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
            let ptr: *mut bool = ptr::addr_of_mut!(is_inclusive);
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
            Self::from_inner(meos_sys::tbox_expand_time(
                self.inner(),
                std::ptr::addr_of!(interval),
            ))
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

    // ------------------------- Distance Operations --------------------------------
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
    fn shift_scale_time(&self, delta: Option<TimeDelta>, width: Option<TimeDelta>) -> TBox {
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

        let modified = unsafe { meos_sys::tbox_shift_scale_time(self._inner, d, w) };
        TBox::from_inner(modified)
    }

    fn union(&self, other: &Self, strict: bool) -> Option<Self> {
        let result = unsafe { meos_sys::union_tbox_tbox(self.inner(), other.inner(), strict) };
        if result.is_null() {
            None
        } else {
            Some(Self::from_inner(result))
        }
    }

    fn intersection(&self, other: &Self) -> Option<Self> {
        let result = unsafe { meos_sys::intersection_tbox_tbox(self.inner(), other.inner()) };
        if result.is_null() {
            None
        } else {
            Some(Self::from_inner(result))
        }
    }
}

impl TBox {
    fn inner(&self) -> *mut meos_sys::TBox {
        self._inner
    }

    fn from_inner(inner: *mut meos_sys::TBox) -> Self {
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

    // pub fn from_tnumber(temporal: TNumber) -> Self {
    //     unsafe {
    //         let inner = tnumber_to_meos_sys::tbox(temporal.inner);
    //         Self::from_inner(inner)
    //     }
    // }

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
        unsafe { FloatSpan::from_inner(meos_sys::tbox_to_floatspan(self._inner)) }
    }

    // ------------------------- Accessors -------------------------------------

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
            let ptr: *mut bool = ptr::addr_of_mut!(is_inclusive);
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
            let ptr: *mut bool = ptr::addr_of_mut!(is_inclusive);
            if meos_sys::tbox_xmax_inc(self.inner(), ptr) {
                Some(is_inclusive)
            } else {
                None
            }
        }
    }

    // ------------------------- Transformation --------------------------------
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
        unsafe { Self::from_inner(meos_sys::tbox_expand_float(self.inner(), value)) }
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
    pub fn shift_scale_value(&self, delta: Option<f64>, width: Option<f64>) -> TBox {
        let d = delta.unwrap_or_default();
        let w = width.unwrap_or_default();
        let modified = unsafe {
            meos_sys::tbox_shift_scale_float(self._inner, d, w, delta.is_some(), width.is_some())
        };
        TBox::from_inner(modified)
    }
}

impl Collection for TBox {
    impl_collection!(tbox, ());

    fn contains(&self, content: &Self::Type) -> bool {
        unsafe { meos_sys::contains_tbox_tnumber(self.inner(), std::ptr::null()) }
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
        CString::new(string).map_err(|_| ParseError).map(|string| {
            let inner = unsafe { meos_sys::tbox_in(string.as_ptr()) };
            Self::from_inner(inner)
        })
    }
}
