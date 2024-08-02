use chrono::TimeZone;

use crate::collections::datetime::tstz_span::TsTzSpan;

use super::{interpolation::TInterpolation, temporal::Temporal, tinstant::TInstant};

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
    fn new<Tz: TimeZone>(values: &[Self::TI], interpolation: TInterpolation) -> Self {
        let mut t_list: Vec<_> = values.iter().map(<Self::TI as TInstant>::inner).collect();
        // The default for discrete instances or instantaneous sequences is an inclusive upper bound
        let upper_inclusive =
            matches!(interpolation, TInterpolation::Discrete) || values.len() == 1;
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
    fn from_value_and_tstzspan(value: Self::Type, time_span: TsTzSpan) -> Self;
    fn from_inner(inner: *const meos_sys::TSequence) -> Self;
    fn inner(&self) -> *const meos_sys::TSequence;

    fn is_lower_inclusive(&self) -> bool {
        unsafe { meos_sys::temporal_lower_inc(Temporal::inner(self)) == 1 }
    }
    fn is_upper_inclusive(&self) -> bool {
        unsafe { meos_sys::temporal_upper_inc(Temporal::inner(self)) == 1 }
    }
}
