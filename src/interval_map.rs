use std::{borrow::Borrow, collections::Bound, iter::FromIterator};

use super::bound::{BorrowPartialOrd2, EndBound, StartBound};
use super::interval::Interval;

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

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
pub enum MergedValue<T, U> {
    Left(T),
    Right(U),
    Both(T, U),
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
    ) -> Option<(Interval<K>, V)> {
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
                            y = iter.next();
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
    pub fn append(&mut self, other: &mut Self) {
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

    pub fn merge<V2: Clone>(
        &self,
        other: &IntervalMap<K, V2>,
    ) -> IntervalMap<K, MergedValue<V, V2>> {
        if other.is_empty() {
            let inner = self
                .iter()
                .map(|(i, v)| (i.clone(), MergedValue::Left(v.clone())))
                .collect();
            unsafe { IntervalMap::from_inner_unchecked(inner) }
        } else if self.is_empty() {
            let inner = other
                .iter()
                .map(|(i, v)| (i.clone(), MergedValue::Right(v.clone())))
                .collect();
            unsafe { IntervalMap::from_inner_unchecked(inner) }
        } else {
            let mut inner = Vec::with_capacity((self.len() + other.len()) * 2 - 1);
            let mut it1 = self.iter().map(|(i, v)| (i.clone(), v.clone()));
            let mut it2 = other.iter().map(|(i, v)| (i.clone(), v.clone()));
            let mut x1 = it1.next();
            let mut x2 = it2.next();
            loop {
                let x = (x1, x2);
                if let (Some((i1, v1)), Some((i2, v2))) = x {
                    if i1.end < i2.start {
                        inner.push((i1, MergedValue::Left(v1)));
                        x2 = Some((i2, v2));
                        x1 = it1.next();
                    } else if i2.end < i1.start {
                        inner.push((i2, MergedValue::Right(v2)));
                        x1 = Some((i1, v1));
                        x2 = it2.next();
                    } else if i1.start < i2.start {
                        inner.push((
                            Interval {
                                start: i1.start,
                                end: i2.start.clone().into(),
                            },
                            MergedValue::Left(v1.clone()),
                        ));
                        x1 = Some((
                            Interval {
                                start: i2.start.clone(),
                                end: i1.end,
                            },
                            v1,
                        ));
                        x2 = Some((i2, v2));
                    } else if i2.start < i1.start {
                        inner.push((
                            Interval {
                                start: i2.start,
                                end: i1.start.clone().into(),
                            },
                            MergedValue::Right(v2.clone()),
                        ));
                        x2 = Some((
                            Interval {
                                start: i1.start.clone(),
                                end: i2.end,
                            },
                            v2,
                        ));
                        x1 = Some((i1, v1));
                    } else if i1.end < i2.end {
                        inner.push((
                            Interval {
                                start: i1.start,
                                end: i1.end.clone(),
                            },
                            MergedValue::Both(v1, v2.clone()),
                        ));
                        x2 = Some((
                            Interval {
                                start: i1.end.into(),
                                end: i2.end,
                            },
                            v2,
                        ));
                        x1 = it1.next();
                    } else if i2.end < i1.end {
                        inner.push((
                            Interval {
                                start: i2.start,
                                end: i2.end.clone(),
                            },
                            MergedValue::Both(v1.clone(), v2),
                        ));
                        x1 = Some((
                            Interval {
                                start: i2.end.into(),
                                end: i1.end,
                            },
                            v1,
                        ));
                        x2 = it2.next();
                    } else {
                        inner.push((
                            Interval {
                                start: i1.start,
                                end: i1.end,
                            },
                            MergedValue::Both(v1, v2),
                        ));
                        x1 = it1.next();
                        x2 = it2.next();
                    }
                } else {
                    x1 = x.0;
                    x2 = x.1;
                    break;
                }
            }
            if let Some((i, v)) = x1 {
                inner.push((i, MergedValue::Left(v)))
            }
            if let Some((i, v)) = x2 {
                inner.push((i, MergedValue::Right(v)))
            }
            inner.extend(it1.map(|(i, v)| (i, MergedValue::Left(v))));
            inner.extend(it2.map(|(i, v)| (i, MergedValue::Right(v))));
            unsafe { IntervalMap::from_inner_unchecked(inner) }
        }
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

    pub fn invert(&mut self, val: V) {
        use std::mem::replace;
        use Bound::*;
        if self.is_empty() {
            self.sorted_vec.push(((..).into(), val));
            return;
        }
        let inner = &mut self.sorted_vec;
        let mut prev = Unbounded;
        let mut n = 0;
        for i in 0..inner.len() {
            if inner[i].0.start.0 == prev {
                prev = replace(&mut inner[i].0.end, EndBound(Unbounded)).0;
            } else if i == n {
                let interval = &mut inner[n].0;
                prev = StartBound::from(replace(
                    &mut interval.end,
                    replace(
                        &mut interval.start,
                        StartBound(replace(&mut prev, Unbounded)),
                    )
                    .into(),
                ))
                .0;
                inner[n].1 = val.clone();
                n += 1;
            } else {
                {
                    let (x, y) = inner.split_at_mut(i);
                    let x = &mut x[n].0;
                    let y = &mut y[0].0;
                    prev = StartBound::from(replace(
                        &mut y.end,
                        replace(
                            &mut x.end,
                            replace(
                                &mut y.start,
                                replace(&mut x.start, StartBound(replace(&mut prev, Unbounded))),
                            )
                            .into(),
                        ),
                    ))
                    .0;
                }
                inner[n].1 = val.clone();
                n += 1;
            }
        }
        inner.truncate(n);
        if prev != Unbounded {
            inner.push((Interval::new(prev, Unbounded), val));
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
    #[test]
    fn merge() {
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
        use MergedValue::*;
        assert_eq!(
            map1.merge(&map2),
            FromIterator::from_iter(vec![
                (10..20, Left(100)),
                (30..40, Left(200)),
                (45..50, Right(-100)),
                (50..55, Both(300, -100)),
                (55..60, Left(300)),
                (65..70, Right(-200)),
                (70..80, Both(400, -200)),
                (80..90, Right(-200)),
                (90..100, Both(500, -200)),
                (100..105, Right(-200)),
                (110..115, Left(600)),
                (115..120, Both(600, -300)),
                (120..125, Right(-300)),
                (130..135, Left(700)),
                (135..140, Both(700, -400)),
                (140..145, Right(-400)),
                (145..155, Right(-500)),
            ])
        )
    }

    #[test]
    fn invert() {
        use Bound::*;
        let mut map1 = IntervalMap::default();
        map1.insert(10..20, 100);
        map1.insert(30..40, 200);
        map1.insert(50..60, 300);
        map1.insert(70..80, 400);
        map1.insert(90..100, 500);
        map1.insert(110..120, 600);
        map1.insert(130..140, 700);
        map1.invert(42);
        let mut map2 = IntervalMap::default();
        map2.insert(..10, 42);
        map2.insert(Interval::new(Included(20), Excluded(30)), 42);
        map2.insert(Interval::new(Included(40), Excluded(50)), 42);
        map2.insert(Interval::new(Included(60), Excluded(70)), 42);
        map2.insert(Interval::new(Included(80), Excluded(90)), 42);
        map2.insert(Interval::new(Included(100), Excluded(110)), 42);
        map2.insert(Interval::new(Included(120), Excluded(130)), 42);
        map2.insert(Interval::new(Included(140), Unbounded), 42);
        assert_eq!(map1, map2)
    }

    #[test]
    fn append1() {
        let mut map1 = IntervalMap::default();
        map1.insert(.., 0);
        let mut map2 = IntervalMap::default();
        map2.insert(..5, 1);
        map2.insert(5..=5, 2);
        map1.append(&mut map2);
        let mut map3 = IntervalMap::default();
        map3.insert(.., 0);
        assert_eq!(map1, map3)
    }
}
