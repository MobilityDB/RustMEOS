mod collection;
pub(crate) use collection::impl_collection;
pub use collection::Collection;

mod span;
pub use span::Span;

mod span_set;
pub(crate) use span_set::impl_iterator;
pub use span_set::SpanSet;
