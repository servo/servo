//! This module contains shims around the stdlib HashMap
//! that add fallible methods
//!
//! These methods are a lie. They are not actually fallible. This is just to make
//! it smooth to switch between hashmap impls in a codebase.

use std::hash::{BuildHasher, Hash};
use std::collections::HashMap as StdMap;
use std::collections::HashSet as StdSet;
use std::ops::{Deref, DerefMut};

pub use std::collections::hash_map::{Entry, RandomState};

pub struct HashMap<K, V, S = RandomState>(StdMap<K, V, S>);

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

impl<K: Hash + Eq, V> HashMap<K, V, RandomState> {

    #[inline]
    pub fn new() -> HashMap<K, V, RandomState> {
        HashMap(StdMap::new())
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> HashMap<K, V, RandomState> {
        HashMap(StdMap::with_capacity(capacity))
    }

    #[inline]
    pub fn try_with_capacity(capacity: usize) -> Result<HashMap<K, V, RandomState>, ()> {
        Ok(HashMap(StdMap::with_capacity(capacity)))
    }
}


impl<K, V, S> HashMap<K, V, S>
    where K: Eq + Hash,
          S: BuildHasher
{
    #[inline]
    pub fn try_with_hasher(hash_builder: S) -> Result<HashMap<K, V, S>, ()> {
        Ok(HashMap(StdMap::with_hasher(hash_builder)))
    }

    #[inline]
    pub fn try_with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Result<HashMap<K, V, S>, ()> {
        Ok(HashMap(StdMap::with_capacity_and_hasher(capacity, hash_builder)))
    }

    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> HashMap<K, V, S> {
        HashMap(StdMap::with_capacity_and_hasher(capacity, hash_builder))
    }


    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), ()> {
        Ok(self.reserve(additional))
    }

    pub fn try_shrink_to_fit(&mut self) -> Result<(), ()> {
        Ok(self.shrink_to_fit())
    }

    pub fn try_entry(&mut self, key: K) -> Result<Entry<K, V>, ()> {
        Ok(self.entry(key))
    }

    #[inline]
    pub fn try_insert(&mut self, k: K, v: V) -> Result<Option<V>, ()> {
        Ok(self.insert(k, v))
    }
}

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
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), ()> {
        Ok(self.reserve(additional))
    }

    #[inline]
    pub fn try_shrink_to_fit(&mut self) -> Result<(), ()> {
        Ok(self.shrink_to_fit())
    }

    #[inline]
    pub fn try_insert(&mut self, value: T) -> Result<bool, ()> {
        Ok(self.insert(value))
    }
}
