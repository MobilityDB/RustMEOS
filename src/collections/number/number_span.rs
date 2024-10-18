use crate::collections::base::Span;

/// You shouldn't probably implement this trait yourself, it's just to run some functions in both IntSpan and FloatSpan
pub trait NumberSpan: Span {}
