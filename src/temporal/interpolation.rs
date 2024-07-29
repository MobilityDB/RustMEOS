use std::fmt;
use std::str::FromStr;

use crate::errors::ParseError;

/// Enum representing the different types of interpolation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TInterpolation {
    None,
    Discrete,
    Stepwise,
    Linear,
}

// Implementing `FromStr` for easier parsing from strings.
impl FromStr for TInterpolation {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" => Ok(TInterpolation::None),
            "discrete" => Ok(TInterpolation::Discrete),
            "linear" => Ok(TInterpolation::Linear),
            "stepwise" | "step" => Ok(TInterpolation::Stepwise),
            _ => Err(ParseError),
        }
    }
}

// Implementing `fmt::Display` for easy printing.
impl fmt::Display for TInterpolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
