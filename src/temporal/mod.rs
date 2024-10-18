mod interpolation;
pub use interpolation::TInterpolation;

mod number;
pub use number::tfloat::*;
pub use number::tint::*;
pub use number::tnumber::TNumber;

#[cfg(feature = "geos")]
mod point;
pub use point::tgeogpoint::*;
pub use point::tgeompoint::*;
pub use point::tpoint::*;

mod tbool;
pub use tbool::*;

#[allow(clippy::module_inception)]
mod temporal;
pub use temporal::{OrderedTemporal, SimplifiableTemporal, Temporal};

mod tinstant;
pub use tinstant::TInstant;

mod tsequence;
pub use tsequence::TSequence;

mod tsequence_set;
pub use tsequence_set::TSequenceSet;

mod ttext;
pub use ttext::*;

/// Taken from <https://json-c.github.io/json-c/json-c-0.10/doc/html/json__object_8h.html#a3294cb92765cdeb497cfd346644d1059>
pub enum JSONCVariant {
    Plain,
    Spaced,
    Pretty,
}
