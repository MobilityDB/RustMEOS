use super::temporal::Temporal;

pub trait TSequence: Temporal {
    fn from_inner(inner: *const meos_sys::TSequence) -> Self;
    fn inner(&self) -> *mut meos_sys::TSequence;
}
