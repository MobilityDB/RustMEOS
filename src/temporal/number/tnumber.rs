use crate::{
    boxes::tbox::TBox,
    collections::number::{number_span::NumberSpan, number_span_set::NumberSpanSet},
    temporal::temporal::Temporal,
};

pub trait TNumber: Temporal<TBB = TBox> {
    // ------------------------- Accessors -------------------------------------
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

    // ------------------------- Restrictions ----------------------------------
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

    // ------------------------- Operations ------------------------------------
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

    // ------------------------- Aggregations ----------------------------------
    /// Returns the absolute value of `self`.
    ///
    /// # Returns
    /// The absolute value of `self`.
    fn abs(&self) -> Self {
        Self::from_inner_as_temporal(unsafe { meos_sys::tnumber_abs(self.inner()) })
    }

    /// Returns the change in value between successive pairs of `self`.
    ///
    /// # Returns
    /// The change in value between successive pairs of `self`.
    fn delta_value(&self) -> Self {
        Self::from_inner_as_temporal(unsafe { meos_sys::tnumber_delta_value(self.inner()) })
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
