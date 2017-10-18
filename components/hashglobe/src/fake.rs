// Copyright 2014-2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module contains shims around the stdlib HashMap
//! that add fallible methods
//!
//! These methods are a lie. They are not actually fallible. This is just to make
//! it smooth to switch between hashmap impls in a codebase.

use std::collections::HashMap as StdMap;
use std::collections::HashSet as StdSet;
use std::fmt;
use std::hash::{BuildHasher, Hash};
use std::ops::{Deref, DerefMut};

pub use std::collections::hash_map::{Entry, RandomState, Iter as MapIter, IterMut as MapIterMut};
pub use std::collections::hash_set::{Iter as SetIter, IntoIter as SetIntoIter};

#[derive(Clone)]
pub struct HashMap<K, V, S = RandomState>(StdMap<K, V, S>);


use FailedAllocationError;

impl<K, V, S> Deref for HashMap<K, V, S> {
    type Target = StdMap<K, V, S>;
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
        Ok(HashMap(StdMap::with_hasher(hash_builder)))
    }

    #[inline]
    pub fn try_with_capacity_and_hasher(capacity: usize,
                                        hash_builder: S)
        -> Result<HashMap<K, V, S>, FailedAllocationError> {
        Ok(HashMap(StdMap::with_capacity_and_hasher(capacity, hash_builder)))
    }

    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> HashMap<K, V, S> {
        HashMap(StdMap::with_capacity_and_hasher(capacity, hash_builder))
    }


    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), FailedAllocationError> {
        Ok(self.reserve(additional))
    }

    pub fn try_shrink_to_fit(&mut self) -> Result<(), FailedAllocationError> {
        Ok(self.shrink_to_fit())
    }

    pub fn try_entry(&mut self, key: K) -> Result<Entry<K, V>, FailedAllocationError> {
        Ok(self.entry(key))
    }

    #[inline(always)]
    pub fn try_get_or_insert_with<F: FnOnce() -> V>(
        &mut self,
        key: K,
        default: F
    ) -> Result<&mut V, FailedAllocationError> {
        Ok(self.entry(key).or_insert_with(default))
    }

    #[inline]
    pub fn try_insert(&mut self, k: K, v: V) -> Result<Option<V>, FailedAllocationError> {
        Ok(self.insert(k, v))
    }

    #[inline(always)]
    pub fn begin_mutation(&mut self) {}
    #[inline(always)]
    pub fn end_mutation(&mut self) {}
}

#[derive(Clone)]
pub struct HashSet<T, S = RandomState>(StdSet<T, S>);


impl<T, S> Deref for HashSet<T, S> {
    type Target = StdSet<T, S>;
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
        HashSet(StdSet::new())
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> HashSet<T, RandomState> {
        HashSet(StdSet::with_capacity(capacity))
    }
}


impl<T, S> HashSet<T, S>
    where T: Eq + Hash,
          S: BuildHasher
{
    #[inline]
    pub fn with_hasher(hasher: S) -> HashSet<T, S> {
        HashSet(StdSet::with_hasher(hasher))
    }


    #[inline]
    pub fn with_capacity_and_hasher(capacity: usize, hasher: S) -> HashSet<T, S> {
        HashSet(StdSet::with_capacity_and_hasher(capacity, hasher))
    }

    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), FailedAllocationError> {
        Ok(self.reserve(additional))
    }

    #[inline]
    pub fn try_shrink_to_fit(&mut self) -> Result<(), FailedAllocationError> {
        Ok(self.shrink_to_fit())
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


