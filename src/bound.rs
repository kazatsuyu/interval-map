use std::{borrow::Borrow, cmp::Ordering, collections::Bound};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct StartBound<T>(pub(super) Bound<T>);

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct EndBound<T>(pub(super) Bound<T>);

pub(super) trait BorrowPartialOrd<T: ?Sized> {
    fn borrow_partial_cmp(&self, other: &T) -> Option<Ordering>;
}

pub(super) trait BorrowPartialOrd2<T: ?Sized> {
    fn borrow_partial_cmp_2(&self, other: &T) -> Option<Ordering>;
}

#[doc(hidden)]
macro __impl($self:ident, $other:ident, $x:ident, $y:ident) {
    impl<T> From<$other<T>> for $self<T> {
        fn from(o: $other<T>) -> Self {
            use Bound::*;
            match o.0 {
                Included(v) => Self(Excluded(v)),
                Excluded(v) => Self(Included(v)),
                Unbounded => panic!(),
            }
        }
    }

    impl<T> Ord for $self<T>
    where
        T: Ord,
    {
        fn cmp(&self, other: &Self) -> Ordering {
            self.partial_cmp(other).unwrap()
        }
    }

    impl<T> PartialEq<T> for $self<T>
    where
        T: PartialEq,
    {
        fn eq(&self, other: &T) -> bool {
            use Bound::*;
            match self.0 {
                Included(ref v) => v == other,
                _ => false,
            }
        }
    }

    impl<T> PartialOrd for $self<T>
    where
        T: PartialOrd,
    {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            self.borrow_partial_cmp_2(other)
        }
    }

    impl<T> PartialEq<$other<T>> for $self<T>
    where
        T: PartialEq,
    {
        fn eq(&self, other: &$other<T>) -> bool {
            use Bound::*;
            match (&self.0, &other.0) {
                (&Included(ref x), &Included(ref y)) => x == y,
                _ => false,
            }
        }
    }

    impl<T> PartialOrd<$other<T>> for $self<T>
    where
        T: PartialOrd,
    {
        fn partial_cmp(&self, other: &$other<T>) -> Option<Ordering> {
            self.borrow_partial_cmp_2(other)
        }
    }

    impl<T> PartialOrd<T> for $self<T>
    where
        T: PartialOrd,
    {
        fn partial_cmp(&self, other: &T) -> Option<Ordering> {
            self.borrow_partial_cmp(other)
        }
    }

    impl<T, Q> BorrowPartialOrd<Q> for $self<T>
    where
        T: Borrow<Q>,
        Q: ?Sized + PartialOrd,
    {
        fn borrow_partial_cmp(&self, other: &Q) -> Option<Ordering> {
            use Bound::*;
            match self.0 {
                Included(ref v) => v.borrow().partial_cmp(other),
                Excluded(ref v) => match v.borrow().partial_cmp(other) {
                    Some(Ordering::Equal) => Some(Ordering::Less),
                    x => x,
                },
                _ => Some(Ordering::Greater),
            }
        }
    }

    impl<T, Q> BorrowPartialOrd2<$self<Q>> for $self<T>
    where
        T: Borrow<Q>,
        Q: PartialOrd,
    {
        fn borrow_partial_cmp_2(&self, other: &$self<Q>) -> Option<Ordering> {
            use Bound::*;
            match (&self.0, &other.0) {
                (&Excluded(ref x), &Excluded(ref y)) | (&Included(ref x), &Included(ref y)) => {
                    x.borrow().partial_cmp(y)
                }
                (&Excluded(ref x), &Included(ref y)) => match x.borrow().partial_cmp(y) {
                    Some(Ordering::Equal) => Some(Ordering::$y),
                    x => x,
                },
                (&Included(ref x), &Excluded(ref y)) => match x.borrow().partial_cmp(y) {
                    Some(Ordering::Equal) => Some(Ordering::$x),
                    x => x,
                },
                (&Unbounded, &Unbounded) => Some(Ordering::Equal),
                (&Unbounded, _) => Some(Ordering::$x),
                (_, &Unbounded) => Some(Ordering::$y),
            }
        }
    }

    impl<T, Q> BorrowPartialOrd2<$other<Q>> for $self<T>
    where
        T: Borrow<Q>,
        Q: PartialOrd,
    {
        fn borrow_partial_cmp_2(&self, other: &$other<Q>) -> Option<Ordering> {
            use Bound::*;
            match (&self.0, &other.0) {
                (&Included(ref x), &Included(ref y)) => x.borrow().partial_cmp(y),
                (&Excluded(ref x), &Excluded(ref y))
                | (&Excluded(ref x), &Included(ref y))
                | (&Included(ref x), &Excluded(ref y)) => match x.borrow().partial_cmp(y) {
                    Some(Ordering::Equal) => Some(Ordering::$y),
                    x => x,
                },
                (&Unbounded, _) | (_, &Unbounded) => Some(Ordering::$x),
            }
        }
    }
}

__impl!(StartBound, EndBound, Less, Greater);
__impl!(EndBound, StartBound, Greater, Less);
