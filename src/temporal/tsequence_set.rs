use super::{temporal::Temporal, tsequence::TSequence};

pub trait TSequenceSet: Temporal {
    /// ## Arguments
    /// * `values` - A slice of temporal sequences (`TSequence`) that represent the values of the temporal sequence set.
    /// * `normalize` - A boolean indicating whether to normalize the temporal sequence set.
    ///
    /// ## Returns
    /// Returns an instance of a type implementing the `TSequenceSet` trait.
    fn new(values: &[Self::TS], normalize: bool) -> Self {
        let mut t_list: Vec<_> = values.iter().map(TSequence::inner_as_tsequence).collect();
        TSequenceSet::from_inner(unsafe {
            meos_sys::tsequenceset_make(t_list.as_mut_ptr(), t_list.len() as i32, normalize)
        })
    }

    fn from_inner(inner: *mut meos_sys::TSequenceSet) -> Self;
}
