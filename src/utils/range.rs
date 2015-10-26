use std::ops::{Range, RangeTo, RangeFrom, RangeFull};

/// Temporary re-implemenation of RangeArgument from stdlib while
/// waiting for it to become stable
pub trait RangeArgument<T> {
    /// Get the (possible) requested first element of a range.
    fn start(&self) -> Option<&T>;
    /// Get the (possible) requested last element of a range.
    fn end(&self) -> Option<&T>;
}
impl<T> RangeArgument<T> for RangeFull {
    fn start(&self) -> Option<&T> {
        None
    }
    fn end(&self) -> Option<&T> {
        None
    }
}
impl<T> RangeArgument<T> for RangeFrom<T> {
    fn start(&self) -> Option<&T> {
        Some(&self.start)
    }
    fn end(&self) -> Option<&T> {
        None
    }
}
impl<T> RangeArgument<T> for RangeTo<T> {
    fn start(&self) -> Option<&T> {
        None
    }
    fn end(&self) -> Option<&T> {
        Some(&self.end)
    }
}
impl<T> RangeArgument<T> for Range<T> {
    fn start(&self) -> Option<&T> {
        Some(&self.start)
    }
    fn end(&self) -> Option<&T> {
        Some(&self.end)
    }
}
