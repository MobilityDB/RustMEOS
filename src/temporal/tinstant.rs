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
    fn from_value_and_timestamp<Tz: TimeZone>(value: Self::Type, timestamp: DateTime<Tz>) -> Self;

    /// Returns the timestamp of the temporal instant.
    ///
    /// ## Returns
    /// A `chrono::DateTime` object.
    fn timestamp(&self) -> DateTime<Utc> {
        self.start_timestamp()
    }
}
