/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module contains shims around the ordermap crate that add fallible
//! methods.
//!
//! These methods are a lie. They are not actually fallible. This is just to
//! make it smooth to switch between hashmap impls the codebase.

use ordermap::{OrderMap, OrderSet};
use std::fmt;
use std::hash::{BuildHasher, Hash};
use std::ops::{Deref, DerefMut};

pub use std::collections::hash_map::RandomState;
pub use ordermap::{Entry, Iter as MapIter, IterMut as MapIterMut};
pub use ordermap::set::{Iter as SetIter, IntoIter as SetIntoIter};

#[derive(Clone)]
pub struct HashMap<K, V, S = RandomState>(OrderMap<K, V, S>);


use FailedAllocationError;

impl<K, V, S> Deref for HashMap<K, V, S> {
    type Target = OrderMap<K, V, S>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K, V, S> DerefMut for HashMap<K, V, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<K, V, S> HashMap<K, V, S>
    where K: Eq + Hash,
          S: BuildHasher
{
    #[inline]
    pub fn try_with_hasher(hash_builder: S) -> Result<HashMap<K, V, S>, FailedAllocationError> {
        Ok(HashMap(OrderMap::with_hasher(hash_builder)))
    }

    #[inline]
    pub fn try_with_capacity_and_hasher(capacity: usize,
                                        hash_builder: S)
        -> Result<HashMap<K, V, S>, FailedAllocationError> {
        Ok(HashMap(OrderMap::with_capacity_and_hasher(capacity, hash_builder)))
    }

    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> HashMap<K, V, S> {
        HashMap(OrderMap::with_capacity_and_hasher(capacity, hash_builder))
    }


    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), FailedAllocationError> {
        Ok(self.reserve(additional))
    }

    #[inline]
    pub fn try_entry(&mut self, key: K) -> Result<Entry<K, V, S>, FailedAllocationError> {
        Ok(self.entry(key))
    }

    #[inline]
    pub fn try_insert(&mut self, k: K, v: V) -> Result<Option<V>, FailedAllocationError> {
        Ok(self.insert(k, v))
    }
}

#[derive(Clone)]
pub struct HashSet<T, S = RandomState>(OrderSet<T, S>);


impl<T, S> Deref for HashSet<T, S> {
    type Target = OrderSet<T, S>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, S> DerefMut for HashSet<T, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Hash + Eq> HashSet<T, RandomState> {
    #[inline]
    pub fn new() -> HashSet<T, RandomState> {
        HashSet(OrderSet::new())
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> HashSet<T, RandomState> {
        HashSet(OrderSet::with_capacity(capacity))
    }
}


impl<T, S> HashSet<T, S>
    where T: Eq + Hash,
          S: BuildHasher
{
    #[inline]
    pub fn with_hasher(hasher: S) -> HashSet<T, S> {
        HashSet(OrderSet::with_hasher(hasher))
    }


    #[inline]
    pub fn with_capacity_and_hasher(capacity: usize, hasher: S) -> HashSet<T, S> {
        HashSet(OrderSet::with_capacity_and_hasher(capacity, hasher))
    }

    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), FailedAllocationError> {
        Ok(self.reserve(additional))
    }

    #[inline]
    pub fn try_insert(&mut self, value: T) -> Result<bool, FailedAllocationError> {
        Ok(self.insert(value))
    }
}

// Pass through trait impls
// We can't derive these since the bounds are not obvious to the derive macro

impl<K: Hash + Eq, V, S: BuildHasher + Default> Default for HashMap<K, V, S> {
    fn default() -> Self {
        HashMap(Default::default())
    }
}

impl<K, V, S> fmt::Debug for HashMap<K, V, S>
    where K: Eq + Hash + fmt::Debug,
          V: fmt::Debug,
          S: BuildHasher {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<K, V, S> PartialEq for HashMap<K, V, S>
    where K: Eq + Hash,
          V: PartialEq,
          S: BuildHasher
{
    fn eq(&self, other: &HashMap<K, V, S>) -> bool {
        self.0.eq(&other.0)
    }
}

impl<K, V, S> Eq for HashMap<K, V, S>
    where K: Eq + Hash,
          V: Eq,
          S: BuildHasher
{
}

impl<'a, K, V, S> IntoIterator for &'a HashMap<K, V, S>
    where K: Eq + Hash,
          S: BuildHasher
{
    type Item = (&'a K, &'a V);
    type IntoIter = MapIter<'a, K, V>;

    fn into_iter(self) -> MapIter<'a, K, V> {
        self.0.iter()
    }
}

impl<'a, K, V, S> IntoIterator for &'a mut HashMap<K, V, S>
    where K: Eq + Hash,
          S: BuildHasher
{
    type Item = (&'a K, &'a mut V);
    type IntoIter = MapIterMut<'a, K, V>;

    fn into_iter(self) -> MapIterMut<'a, K, V> {
        self.0.iter_mut()
    }
}

impl<T: Eq + Hash, S: BuildHasher + Default> Default for HashSet<T, S> {
    fn default() -> Self {
        HashSet(Default::default())
    }
}

impl<T, S> fmt::Debug for HashSet<T, S>
    where T: Eq + Hash + fmt::Debug,
          S: BuildHasher
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T, S> PartialEq for HashSet<T, S>
    where T: Eq + Hash,
          S: BuildHasher
{
    fn eq(&self, other: &HashSet<T, S>) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T, S> Eq for HashSet<T, S>
    where T: Eq + Hash,
          S: BuildHasher
{
}

impl<'a, T, S> IntoIterator for &'a HashSet<T, S>
    where T: Eq + Hash,
          S: BuildHasher
{
    type Item = &'a T;
    type IntoIter = SetIter<'a, T>;

    fn into_iter(self) -> SetIter<'a, T> {
        self.0.iter()
    }
}

impl<T, S> IntoIterator for HashSet<T, S>
    where T: Eq + Hash,
          S: BuildHasher
{
    type Item = T;
    type IntoIter = SetIntoIter<T>;


    fn into_iter(self) -> SetIntoIter<T> {
        self.0.into_iter()
    }
}


