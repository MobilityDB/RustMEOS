use chrono::{DateTime, TimeZone, Utc};

use super::temporal::Temporal;

pub trait TInstant: Temporal {
    fn new<Tz: TimeZone>(value: Self::Type, timestamp: DateTime<Tz>) -> Self;
    fn from_inner(inner: *mut meos_sys::TInstant) -> Self;
    fn inner(&self) -> *const meos_sys::TInstant;

    /// ## Returns
    /// The value of the temporal instant.
    fn value(&self) -> Self::Type {
        self.start_value()
    }

    /// Returns the timestamp of the temporal instant.
    ///
    /// ## Returns
    /// A `chrono::DateTime` object.
    fn timestamp(&self) -> DateTime<Utc> {
        self.start_timestamp()
    }
}
