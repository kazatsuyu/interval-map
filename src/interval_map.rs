use std::{
    borrow::Borrow,
    cmp::Ordering,
    collections::Bound,
    iter::FromIterator,
    ops::{Range, RangeBounds, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive},
};

use super::bound::{BorrowPartialOrd, BorrowPartialOrd2, EndBound, StartBound};

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

    fn remove(self, other: &Self) -> [Option<Interval<T>>; 2]
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

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct IntervalMap<K, V> {
    sorted_vec: Vec<(Interval<K>, V)>,
}

impl<K, V> IntervalMap<K, V> {
    pub fn new() -> Self {
        Self { sorted_vec: vec![] }
    }
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            sorted_vec: Vec::with_capacity(capacity),
        }
    }
    pub fn capacity(&self) -> usize {
        self.sorted_vec.capacity()
    }
    pub fn keys(&self) -> Keys<K, V> {
        Keys(self.iter())
    }
    pub fn values(&self) -> Values<K, V> {
        Values(self.iter())
    }
    pub fn values_mut(&mut self) -> ValuesMut<K, V> {
        ValuesMut(self.iter_mut())
    }
    pub fn iter(&self) -> Iter<K, V> {
        Iter(self.sorted_vec.iter())
    }
    pub fn iter_mut(&mut self) -> IterMut<K, V> {
        IterMut(self.sorted_vec.iter_mut())
    }
    pub fn len(&self) -> usize {
        self.sorted_vec.len()
    }
    pub fn is_empty(&self) -> bool {
        self.sorted_vec.is_empty()
    }
    pub fn drain(&mut self) -> Drain<K, V> {
        Drain(self.sorted_vec.drain(..))
    }
    pub fn clear(&mut self) {
        self.sorted_vec.clear()
    }
    pub fn reserve(&mut self, additional: usize) {
        self.sorted_vec.reserve(additional)
    }
    #[cfg(feature = "try_reserve")]
    pub fn try_reserve(
        &mut self,
        additional: usize,
    ) -> Result<(), std::collections::TryReserveError> {
        self.sorted_vec.try_reserve(additional)
    }
    pub fn shrink_to_fit(&mut self) {
        self.sorted_vec.shrink_to_fit()
    }
    #[cfg(feature = "shrink_to")]
    pub fn shrink_to(
        &mut self,
        min_capacity: usize,
    ) -> Result<(), std::collections::TryReserveError> {
        self.sorted_vec.shrink_to(min_capacity)
    }
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: ?Sized + Ord,
    {
        self.sorted_vec
            .binary_search_by(|(interval, _)| interval.partial_cmp(key).unwrap())
            .map(|i| Some(&self.sorted_vec[i].1))
            .unwrap_or(None)
    }
    pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&Interval<K>, &V)>
    where
        K: Borrow<Q>,
        Q: ?Sized + Ord,
    {
        self.sorted_vec
            .binary_search_by(|(interval, _)| interval.partial_cmp(key).unwrap())
            .map(|i| Some((&self.sorted_vec[i].0, &self.sorted_vec[i].1)))
            .unwrap_or(None)
    }
    #[cfg(feature = "map_first_last")]
    pub fn first_key_value(&self) -> Option<(&Interval<K>, &V)>
    where
        K: Ord,
    {
        self.sorted_vec.first().map(|(i, v)| (i, v))
    }
    #[cfg(feature = "map_first_last")]
    pub fn first_entry<T>(&mut self) -> Option<OccupiedEntry<K, V>> {
        if !self.sorted_vec.is_empty() {
            Some(OccupiedEntry(0, &mut self.sorted_vec))
        } else {
            None
        }
    }
    #[cfg(feature = "map_first_last")]
    pub fn last_key_value(&self) -> Option<(&Interval<K>, &V)>
    where
        K: Ord,
    {
        self.sorted_vec.last().map(|(i, v)| (i, v))
    }
    #[cfg(feature = "map_first_last")]
    pub fn last_entry<T>(&mut self) -> Option<OccupiedEntry<K, V>> {
        if !self.sorted_vec.is_empty() {
            Some(OccupiedEntry(
                self.sorted_vec.len() - 1,
                &mut self.sorted_vec,
            ))
        } else {
            None
        }
    }
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.sorted_vec
            .binary_search_by(|(interval, _)| interval.partial_cmp(key).unwrap())
            .is_ok()
    }
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.sorted_vec
            .binary_search_by(|(interval, _)| interval.partial_cmp(key).unwrap())
            .map(move |i| Some(&mut self.sorted_vec[i].1))
            .unwrap_or(None)
    }
    pub fn range<T, R>(&self, r: R) -> Iter<K, V>
    where
        K: Borrow<T>,
        R: Into<Interval<T>>,
        T: Ord,
    {
        let i = r.into();
        match [
            self.sorted_vec.binary_search_by(|(interval, _)| {
                interval.start.borrow_partial_cmp_2(&i.start).unwrap()
            }),
            self.sorted_vec.binary_search_by(|(interval, _)| {
                interval.end.borrow_partial_cmp_2(&i.end).unwrap()
            }),
        ] {
            [Ok(s), Ok(e)] | [Err(s), Ok(e)] => Iter(self.sorted_vec[s..=e].iter()),
            [Ok(s), Err(e)] | [Err(s), Err(e)] => Iter(self.sorted_vec[s..e].iter()),
        }
    }

    pub fn range_mut<T, R>(&mut self, r: R) -> IterMut<K, V>
    where
        K: Borrow<T>,
        R: Into<Interval<T>>,
        T: Ord,
    {
        let i = r.into();
        match [
            self.sorted_vec.binary_search_by(|(interval, _)| {
                interval.start.borrow_partial_cmp_2(&i.start).unwrap()
            }),
            self.sorted_vec.binary_search_by(|(interval, _)| {
                interval.end.borrow_partial_cmp_2(&i.end).unwrap()
            }),
        ] {
            [Ok(s), Ok(e)] | [Err(s), Ok(e)] => IterMut(self.sorted_vec[s..=e].iter_mut()),
            [Ok(s), Err(e)] | [Err(s), Err(e)] => IterMut(self.sorted_vec[s..e].iter_mut()),
        }
    }

    pub fn entry(&mut self, key: K) -> Entry<K, V>
    where
        K: Ord + Clone,
    {
        match self
            .sorted_vec
            .binary_search_by(|(interval, _)| interval.partial_cmp(&key).unwrap())
        {
            Ok(i) => Entry::Occupied(OccupiedEntry(i, &mut self.sorted_vec)),
            Err(i) => Entry::Vacant(VacantEntry(
                i,
                (key.clone()..=key).into(),
                &mut self.sorted_vec,
            )),
        }
    }

    pub fn inner(&self) -> &[(Interval<K>, V)] {
        &self.sorted_vec
    }

    pub fn into_inner(self) -> Vec<(Interval<K>, V)> {
        self.sorted_vec
    }

    pub unsafe fn from_inner_unchecked(inner: Vec<(Interval<K>, V)>) -> Self {
        Self { sorted_vec: inner }
    }
}

impl<K, V> IntervalMap<K, V>
where
    K: Ord + Clone,
    V: Clone,
{
    fn insert_impl(&mut self, interval: Interval<K>, val: V) {
        if interval.is_empty() {
            return;
        }
        if let Some((i, v)) = self.sorted_vec.pop() {
            match interval.remove(&i) {
                [Some(left), Some(right)] => {
                    self.insert_impl(left, val.clone());
                    self.sorted_vec.push((i, v));
                    self.sorted_vec.push((right, val));
                }
                [Some(left), None] => {
                    self.insert_impl(left, val);
                    self.sorted_vec.push((i, v));
                }
                [None, Some(right)] => {
                    self.sorted_vec.push((i, v));
                    self.sorted_vec.push((right, val));
                }
                [None, None] => {
                    self.sorted_vec.push((i, v));
                }
            }
        } else {
            self.sorted_vec.push((interval, val));
        }
    }
    pub fn insert<T: Into<Interval<K>>>(&mut self, key: T, val: V) {
        self.insert_impl(key.into(), val)
    }

    fn overwrite_impl(&mut self, interval: Interval<K>, val: V) {
        if interval.is_empty() {
            return;
        }
        if let Some((i, v)) = self.sorted_vec.pop() {
            match i.remove(&interval) {
                [Some(left), Some(right)] => {
                    self.sorted_vec.push((left, v.clone()));
                    self.sorted_vec.push((interval, val));
                    self.sorted_vec.push((right, v));
                }
                [Some(left), None] => {
                    self.sorted_vec.push((left, v.clone()));
                    self.sorted_vec.push((interval, val));
                }
                [None, Some(right)] => {
                    self.overwrite_impl(interval, val);
                    self.sorted_vec.push((right, v));
                }
                [None, None] => {
                    self.overwrite_impl(interval, val);
                }
            }
        } else {
            self.sorted_vec.push((interval, val));
        }
    }
    pub fn overwrite<T: Into<Interval<K>>>(&mut self, key: T, val: V) {
        self.overwrite_impl(key.into(), val)
    }

    fn remove_impl(&mut self, interval: Interval<K>) {
        if interval.is_empty() {
            return;
        }
        if let Some((i, v)) = self.sorted_vec.pop() {
            match i.remove(&interval) {
                [Some(left), Some(right)] => {
                    self.sorted_vec.push((left, v.clone()));
                    self.sorted_vec.push((right, v));
                }
                [Some(left), None] => {
                    self.sorted_vec.push((left, v.clone()));
                }
                [None, Some(right)] => {
                    self.remove_impl(interval);
                    self.sorted_vec.push((right, v));
                }
                [None, None] => {
                    self.remove_impl(interval);
                }
            }
        }
    }
    pub fn remove<T: Into<Interval<K>>>(&mut self, key: T) {
        self.remove_impl(key.into())
    }
    fn append_impl(
        &mut self,
        iter: &mut Drain<K, V>,
        x: Option<(Interval<K>, V)>,
        mut y: Option<(Interval<K>, V)>,
    ) -> Option<(Interval<K>, V)>
    where
        K: std::fmt::Debug,
    {
        if let Some((i, v)) = x {
            loop {
                if let Some((i2, v2)) = y {
                    match i2.remove(&i) {
                        [Some(left), Some(right)] => {
                            self.insert_impl(left, v2.clone());
                            self.sorted_vec.push((i, v));
                            break Some((right, v2));
                        }
                        [Some(left), None] => {
                            let x = self.sorted_vec.pop();
                            if let Some((i3, v3)) = self.append_impl(iter, x, Some((left, v2))) {
                                if i3.end < i.start {
                                    self.sorted_vec.push((i3, v3));
                                    y = iter.next();
                                } else {
                                    y = Some((i3, v3));
                                }
                            } else {
                                y = iter.next();
                            }
                        }
                        [None, Some(right)] => {
                            self.sorted_vec.push((i, v));
                            break Some((right, v2));
                        }
                        [None, None] => {
                            self.sorted_vec.push((i, v));
                            break iter.next();
                        }
                    }
                } else {
                    self.sorted_vec.push((i, v));
                    break None;
                }
            }
        } else {
            y
        }
    }
    pub fn append(&mut self, other: &mut Self)
    where
        K: std::fmt::Debug,
    {
        if self.is_empty() {
            std::mem::swap(self, other);
            return;
        }
        let mut iter = other.drain();
        let x = self.sorted_vec.pop();
        let y = iter.next();
        if let Some(v) = self.append_impl(&mut iter, x, y) {
            self.sorted_vec.push(v);
        }
        self.sorted_vec.extend(iter);
    }
    pub fn split_off<Q>(&mut self, key: K) -> Self {
        match self
            .sorted_vec
            .binary_search_by(|(interval, _)| interval.partial_cmp(&key).unwrap())
        {
            Ok(i) => {
                let v = &mut self.sorted_vec[i];
                if v.0.start == key {
                    Self {
                        sorted_vec: self.sorted_vec.split_off(i),
                    }
                } else {
                    let mut sorted_vec = self.sorted_vec.split_off(i);
                    self.sorted_vec.push((
                        Interval {
                            start: std::mem::replace(
                                &mut sorted_vec[0].0.start,
                                StartBound(Bound::Included(key.clone())),
                            ),
                            end: EndBound(Bound::Excluded(key)),
                        },
                        sorted_vec[0].1.clone(),
                    ));
                    Self { sorted_vec }
                }
            }
            Err(i) => Self {
                sorted_vec: self.sorted_vec.split_off(i),
            },
        }
    }
}

impl<K, V> Default for IntervalMap<K, V>
where
    K: Ord + Clone,
    V: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

pub struct Keys<'a, K: 'a, V: 'a>(Iter<'a, K, V>);

impl<'a, K: 'a, V: 'a> Iterator for Keys<'a, K, V> {
    type Item = &'a Interval<K>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(i, _)| i)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

pub struct Values<'a, K: 'a, V: 'a>(Iter<'a, K, V>);

impl<'a, K: 'a, V: 'a> Iterator for Values<'a, K, V> {
    type Item = &'a V;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, v)| v)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

pub struct ValuesMut<'a, K: 'a, V: 'a>(IterMut<'a, K, V>);

impl<'a, K: 'a, V: 'a> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, v)| v)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

pub struct Iter<'a, K: 'a, V: 'a>(std::slice::Iter<'a, (Interval<K>, V)>);

impl<'a, K: 'a, V: 'a> Iterator for Iter<'a, K, V> {
    type Item = (&'a Interval<K>, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(i, v)| (i, v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

pub struct IterMut<'a, K: 'a, V: 'a>(std::slice::IterMut<'a, (Interval<K>, V)>);

impl<'a, K: 'a, V: 'a> Iterator for IterMut<'a, K, V> {
    type Item = (&'a Interval<K>, &'a mut V);
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(i, v)| (i as &'a Interval<K>, v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

pub struct Drain<'a, K: 'a, V: 'a>(std::vec::Drain<'a, (Interval<K>, V)>);

impl<'a, K: 'a, V: 'a> Iterator for Drain<'a, K, V> {
    type Item = (Interval<K>, V);
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

#[derive(Debug)]
pub enum Entry<'a, K: 'a, V: 'a> {
    Vacant(VacantEntry<'a, K, V>),
    Occupied(OccupiedEntry<'a, K, V>),
}

impl<'a, K, V> Entry<'a, K, V>
where
    K: Ord,
{
    pub fn or_insert(self, default: V) -> &'a mut V {
        use Entry::*;
        match self {
            Vacant(v) => v.insert(default),
            Occupied(v) => v.into_mut(),
        }
    }
    pub fn or_insert_with<F>(self, default: F) -> &'a mut V
    where
        F: FnOnce() -> V,
    {
        use Entry::*;
        match self {
            Vacant(v) => v.insert(default()),
            Occupied(v) => v.into_mut(),
        }
    }
    pub fn key(&self) -> &Interval<K> {
        use Entry::*;
        match *self {
            Vacant(ref v) => v.key(),
            Occupied(ref v) => v.key(),
        }
    }
    pub fn and_modify<F>(self, f: F) -> Entry<'a, K, V>
    where
        F: FnOnce(&mut V),
    {
        use Entry::*;
        match self {
            Occupied(mut v) => {
                f(v.get_mut());
                Occupied(v)
            }
            v => v,
        }
    }
    pub fn or_default(self) -> &'a mut V
    where
        K: Ord,
        V: Default,
    {
        use Entry::*;
        match self {
            Vacant(v) => v.insert(Default::default()),
            Occupied(v) => v.into_mut(),
        }
    }
}

#[derive(Debug)]
pub struct VacantEntry<'a, K: 'a, V: 'a>(
    usize,
    Interval<K>,
    &'a mut std::vec::Vec<(Interval<K>, V)>,
);

impl<'a, K, V> VacantEntry<'a, K, V>
where
    K: Ord,
{
    pub fn key(&self) -> &Interval<K> {
        &self.1
    }
    pub fn into_key(self) -> Interval<K> {
        self.1
    }
    pub fn insert(self, value: V) -> &'a mut V {
        self.2.insert(self.0, (self.1, value));
        &mut self.2[self.0].1
    }
}

#[derive(Debug)]
pub struct OccupiedEntry<'a, K: 'a, V: 'a>(usize, &'a mut std::vec::Vec<(Interval<K>, V)>);
impl<'a, K, V> OccupiedEntry<'a, K, V>
where
    K: Ord,
{
    pub fn key(&self) -> &Interval<K> {
        &self.1[self.0].0
    }
    pub fn remove_entry(self) -> (Interval<K>, V) {
        self.1.remove(self.0)
    }
    pub fn get(&self) -> &V {
        &self.1[self.0].1
    }
    pub fn get_mut(&mut self) -> &mut V {
        &mut self.1[self.0].1
    }
    pub fn into_mut(self) -> &'a mut V {
        &mut self.1[self.0].1
    }
    pub fn insert(&mut self, value: V) -> V {
        std::mem::replace(self.get_mut(), value)
    }
    pub fn remove(self) -> V {
        self.remove_entry().1
    }
}

impl<'a, K, V> Extend<(&'a Interval<K>, &'a V)> for IntervalMap<K, V>
where
    K: 'a + Ord + Clone,
    V: 'a + Clone,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (&'a Interval<K>, &'a V)>,
    {
        for (i, v) in iter.into_iter() {
            self.insert_impl(i.clone(), v.clone())
        }
    }
}

impl<K, V> Extend<(Interval<K>, V)> for IntervalMap<K, V>
where
    K: Ord + Clone,
    V: Clone,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (Interval<K>, V)>,
    {
        for (i, v) in iter.into_iter() {
            self.insert_impl(i, v)
        }
    }
}

impl<K, I, V> FromIterator<(I, V)> for IntervalMap<K, V>
where
    K: Ord + Clone,
    I: Into<Interval<K>>,
    V: Clone,
{
    fn from_iter<T: IntoIterator<Item = (I, V)>>(iter: T) -> Self {
        let mut map = Self::new();
        for (i, v) in iter {
            map.insert(i.into(), v)
        }
        map
    }
}

impl<K, Q, V> std::ops::Index<&Q> for IntervalMap<K, V>
where
    K: Borrow<Q>,
    Q: Ord + ?Sized,
{
    type Output = V;
    fn index(&self, key: &Q) -> &V {
        self.get(key).unwrap()
    }
}

impl<K, V> IntoIterator for IntervalMap<K, V> {
    type Item = (Interval<K>, V);
    type IntoIter = IntoIter<K, V>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.sorted_vec.into_iter())
    }
}

pub struct IntoIter<K, V>(std::vec::IntoIter<(Interval<K>, V)>);

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (Interval<K>, V);
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, K, V> IntoIterator for &'a IntervalMap<K, V> {
    type Item = (&'a Interval<K>, &'a V);
    type IntoIter = Iter<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V> IntoIterator for &'a mut IntervalMap<K, V> {
    type Item = (&'a Interval<K>, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn interval_map() {
        let mut map = IntervalMap::default();
        map.insert(10..20, 100);
        map.insert(30..40, 200);
        map.insert(0..=25, 300);
        map.insert(60.., 400);
        map.overwrite(70..80, 500);
        assert_eq!(map.get(&std::i32::MIN), None);
        assert_eq!(map.get(&-1), None);
        assert_eq!(map.get(&0), Some(&300));
        assert_eq!(map.get(&9), Some(&300));
        assert_eq!(map.get(&10), Some(&100));
        assert_eq!(map.get(&19), Some(&100));
        assert_eq!(map.get(&20), Some(&300));
        assert_eq!(map.get(&25), Some(&300));
        assert_eq!(map.get(&26), None);
        assert_eq!(map.get(&29), None);
        assert_eq!(map.get(&30), Some(&200));
        assert_eq!(map.get(&39), Some(&200));
        assert_eq!(map.get(&40), None);
        assert_eq!(map.get(&60), Some(&400));
        assert_eq!(map.get(&69), Some(&400));
        assert_eq!(map.get(&70), Some(&500));
        assert_eq!(map.get(&79), Some(&500));
        assert_eq!(map.get(&80), Some(&400));
        assert_eq!(map.get(&std::i32::MAX), Some(&400));
    }
    #[test]
    fn append() {
        let mut map1 = IntervalMap::default();
        map1.insert(10..20, 100);
        map1.insert(30..40, 200);
        map1.insert(50..60, 300);
        map1.insert(70..80, 400);
        map1.insert(90..100, 500);
        map1.insert(110..120, 600);
        map1.insert(130..140, 700);
        let mut map2 = IntervalMap::default();
        map2.insert(45..55, -100);
        map2.insert(65..105, -200);
        map2.insert(115..125, -300);
        map2.insert(135..145, -400);
        map2.insert(145..155, -500);
        map1.append(&mut map2);
        assert_eq!(
            map1,
            FromIterator::from_iter(vec![
                (10..20, 100),
                (30..40, 200),
                (45..50, -100),
                (50..60, 300),
                (65..70, -200),
                (70..80, 400),
                (80..90, -200),
                (90..100, 500),
                (100..105, -200),
                (110..120, 600),
                (115..125, -300),
                (130..140, 700),
                (135..145, -400),
                (145..155, -500),
            ])
        )
    }
}
