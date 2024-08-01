use super::temporal::Temporal;

pub trait TInstant: Temporal {
    fn from_inner(inner: *const meos_sys::TInstant) -> Self;
    fn inner(&self) -> *mut meos_sys::TInstant;
    fn value(&self) -> Self::TBase {
        self.start_value()
    }
}
