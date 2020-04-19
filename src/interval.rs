use std::{
    borrow::Borrow,
    cmp::Ordering,
    collections::Bound,
    ops::{Range, RangeBounds, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive},
};

use super::bound::{BorrowPartialOrd, EndBound, StartBound};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Interval<T> {
    pub start: StartBound<T>,
    pub end: EndBound<T>,
}

impl<T> Interval<T> {
    pub const fn new(start: Bound<T>, end: Bound<T>) -> Self {
        Self {
            start: StartBound(start),
            end: EndBound(end),
        }
    }

    pub fn is_empty(&self) -> bool
    where
        T: PartialOrd,
    {
        !(self.start < self.end)
    }

    pub(super) fn remove(self, other: &Self) -> [Option<Interval<T>>; 2]
    where
        T: PartialOrd + Clone,
    {
        if self.end < other.start {
            [Some(self), None]
        } else if self.start > other.end {
            [None, Some(self)]
        } else {
            [
                if self.start < other.start {
                    Some(Interval {
                        start: self.start,
                        end: other.start.clone().into(),
                    })
                } else {
                    None
                },
                if self.end > other.end {
                    Some(Interval {
                        start: other.end.clone().into(),
                        end: self.end.into(),
                    })
                } else {
                    None
                },
            ]
        }
    }

    fn borrow_contains<Q>(&self, other: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: ?Sized + PartialOrd,
    {
        matches!(self.partial_cmp(other), Some(Ordering::Equal))
    }
}

impl<T> RangeBounds<T> for Interval<T> {
    fn start_bound(&self) -> Bound<&T> {
        use Bound::*;
        match self.start.0 {
            Excluded(ref v) => Excluded(v),
            Included(ref v) => Included(v),
            Unbounded => Unbounded,
        }
    }
    fn end_bound(&self) -> Bound<&T> {
        use Bound::*;
        match self.end.0 {
            Excluded(ref v) => Excluded(v),
            Included(ref v) => Included(v),
            Unbounded => Unbounded,
        }
    }
}

impl<T, Q> PartialEq<Q> for Interval<T>
where
    T: Borrow<Q>,
    Q: ?Sized + PartialOrd,
{
    fn eq(&self, other: &Q) -> bool {
        self.borrow_contains(other)
    }
}

impl<T, Q> PartialOrd<Q> for Interval<T>
where
    T: Borrow<Q>,
    Q: ?Sized + PartialOrd,
{
    fn partial_cmp(&self, other: &Q) -> Option<Ordering> {
        match self.start.borrow_partial_cmp(other) {
            Some(Ordering::Less) | Some(Ordering::Equal) => {
                match self.end.borrow_partial_cmp(other) {
                    Some(Ordering::Greater) | Some(Ordering::Equal) => Some(Ordering::Equal),
                    x => x,
                }
            }
            x => x,
        }
    }
}

impl<T> From<Range<T>> for Interval<T> {
    fn from(r: Range<T>) -> Self {
        Self {
            start: StartBound(Bound::Included(r.start)),
            end: EndBound(Bound::Excluded(r.end)),
        }
    }
}

impl<T> From<RangeInclusive<T>> for Interval<T> {
    fn from(r: RangeInclusive<T>) -> Self {
        let (start, end) = r.into_inner();
        Self {
            start: StartBound(Bound::Included(start)),
            end: EndBound(Bound::Included(end)),
        }
    }
}

impl<T> From<RangeFrom<T>> for Interval<T> {
    fn from(r: RangeFrom<T>) -> Self {
        Self {
            start: StartBound(Bound::Included(r.start)),
            end: EndBound(Bound::Unbounded),
        }
    }
}

impl<T> From<RangeTo<T>> for Interval<T> {
    fn from(r: RangeTo<T>) -> Self {
        Self {
            start: StartBound(Bound::Unbounded),
            end: EndBound(Bound::Excluded(r.end)),
        }
    }
}

impl<T> From<RangeToInclusive<T>> for Interval<T> {
    fn from(r: RangeToInclusive<T>) -> Self {
        Self {
            start: StartBound(Bound::Unbounded),
            end: EndBound(Bound::Included(r.end)),
        }
    }
}

impl<T> From<RangeFull> for Interval<T> {
    fn from(_: RangeFull) -> Self {
        Self {
            start: StartBound(Bound::Unbounded),
            end: EndBound(Bound::Unbounded),
        }
    }
}

#[cfg(feature = "proc-macro")]
use {
    proc_macro2::TokenStream,
    quote::{quote, ToTokens},
};

#[cfg(feature = "proc-macro")]
impl<T: ToTokens> ToTokens for Interval<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let s = self.start.to_token_stream();
        let e = self.end.to_token_stream();
        *tokens = quote!(interval_map::Interval {
            start: #s,
            end: #e,
        });
    }
}

#[cfg(all(test, feature = "proc-macro"))]
mod tests {
    use super::*;
    #[test]
    fn to_tokens() {
        let i: Interval<_> = (0..1).into();
        assert_eq!(
            i.to_token_stream().to_string(),
            quote!(interval_map::Interval {
                start: interval_map::bound::StartBound(std::collections::Bound::Included(0i32)),
                end: interval_map::bound::EndBound(std::collections::Bound::Excluded(1i32)),
            })
            .to_string()
        );
    }
}
