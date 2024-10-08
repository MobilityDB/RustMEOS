pub mod interpolation;
pub mod number;
#[cfg(feature = "geos")]
pub mod point;
pub mod tbool;
#[allow(clippy::module_inception)]
pub mod temporal;
pub mod tinstant;
pub mod tsequence;
pub mod tsequence_set;
pub mod ttext;

/// Taken from https://json-c.github.io/json-c/json-c-0.10/doc/html/json__object_8h.html#a3294cb92765cdeb497cfd346644d1059
pub enum JSONCVariant {
    Plain,
    Spaced,
    Pretty,
}
