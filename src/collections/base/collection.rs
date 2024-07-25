use std::{fmt::Debug, hash::Hash, str::FromStr};

pub trait Collection: PartialEq + Debug + FromStr + Hash + Clone {
    type Type: Clone;
    // Topological Operations

    /// Returns whether `self` is contained in `container`.
    ///
    /// ## Arguments
    ///
    /// * `container` - The container to compare with.
    ///
    /// ## Returns
    ///
    /// * `true` if contained, `false` otherwise.
    ///
    /// ## Safety
    ///
    /// Calls the underlying `meos_sys::contained_span_span` function.
    fn is_contained_in(&self, container: &Self) -> bool;

    /// Determines if the collection contains the specified item.
    ///
    /// # Arguments
    ///
    /// * `content` - The item to check for containment within the collection.
    ///
    /// # Returns
    ///
    /// * `true` if the collection contains the specified item, `false` otherwise.
    fn contains(&self, content: &Self::Type) -> bool;

    /// Returns whether `self` overlaps `other`. That is, both share at least an element.
    ///
    /// ## Arguments
    ///
    /// * `other` - The object to compare with.
    ///
    /// ## Returns
    ///
    /// * `true` if overlaps, `false` otherwise.
    ///
    /// ## Safety
    ///
    /// Calls the underlying `meos_sys::overlaps_span_span` function.
    fn overlaps(&self, other: &Self) -> bool;

    // Position Operations

    /// Returns whether `self` is strictly before `other`. That is, `self` ends before `other` starts.
    ///
    /// ## Arguments
    ///
    /// * `other` - The object to compare with.
    ///
    /// ## Returns
    ///
    /// * `true` if before, `false` otherwise.
    ///
    /// ## Safety
    ///
    /// Calls the underlying `meos_sys::left_span_span` function.
    fn is_left(&self, other: &Self) -> bool;

    /// Returns whether `self` is before `other` allowing overlap. That is, `self` ends before `other` ends (or at the same time).
    ///
    /// ## Arguments
    ///
    /// * `other` - The object to compare with.
    ///
    /// ## Returns
    ///
    /// * `true` if before, `false` otherwise.
    ///
    /// ## Safety
    ///
    /// Calls the underlying `meos_sys::overleft_span_span` function.
    fn is_over_or_left(&self, other: &Self) -> bool;

    /// Returns whether `self` is after `other` allowing overlap. That is, `self` starts after `other` starts (or at the same time).
    ///
    /// ## Arguments
    ///
    /// * `other` - The object to compare with.
    ///
    /// ## Returns
    ///
    /// * `true` if overlapping or after, `false` otherwise.
    ///
    /// ## Safety
    ///
    /// Calls the underlying `meos_sys::overright_span_span` function.
    fn is_over_or_right(&self, other: &Self) -> bool;

    /// Returns whether `self` is strictly after `other`. That is, `self` starts after `other` ends.
    ///
    /// ## Arguments
    ///
    /// * `other` - The object to compare with.
    ///
    /// ## Returns
    ///
    /// * `true` if after, `false` otherwise.
    ///
    /// ## Safety
    ///
    /// Calls the underlying `meos_sys::right_span_span` function.
    fn is_right(&self, other: &Self) -> bool;
}

// Rust doesn't support yet generating multiple blanket implementations for the same type: see https://stackoverflow.com/questions/73782573/why-do-blanket-implementations-for-two-different-traits-conflict.
// therefore, to avoid implementing manually per each specific instance of span, set, etc. we use this macro to generate some methods automatically.
// We can therefore just generate all of the collection functions with the kind of container (span, set, ect.), and the kind of element (int, float, etc.)

// Parameters:
//  $type: The type of the container: spanset, span, or set
//  $subtype: The type of what is contained: float, int, geo, etc.
//  $subtype_type: The actual type in Rust of the subtype: f64, i32, etc.
macro_rules! impl_collection {
    ($type:ident, $subtype:ident, $subtype_type:ty) => {
        type Type = $subtype_type;
        paste::paste! {
            fn is_contained_in(&self, container: &Self) -> bool {
                unsafe { meos_sys::[<contained _ $type _ $type>](self.inner(), container.inner()) }
            }

            fn overlaps(&self, other: &Self) -> bool {
                unsafe { meos_sys::[<overlaps _ $type _ $type>](self.inner(), other.inner()) }
            }

            fn is_left(&self, other: &Self) -> bool {
                unsafe { meos_sys::[<left _ $type _ $type>](self.inner(), other.inner()) }
            }

            fn is_over_or_left(&self, other: &Self) -> bool {
                unsafe { meos_sys::[<overleft _ $type _ $type>](self.inner(), other.inner()) }
            }

            fn is_over_or_right(&self, other: &Self) -> bool {
                unsafe { meos_sys::[<overright _ $type _ $type>](self.inner(), other.inner()) }
            }

            fn is_right(&self, other: &Self) -> bool {
                unsafe { meos_sys::[<right _ $type _ $type>](self.inner(), other.inner()) }
            }
        }
    };
}

pub(crate) use impl_collection;
