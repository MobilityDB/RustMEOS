use super::temporal::Temporal;

pub trait TSequenceSet: Temporal {
    fn from_inner(inner: *const meos_sys::TSequenceSet) -> Self;
}
