/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A map that preserves order for the keys, and that is easily indexable.
//!
//! This is used for CSS custom properties.
//!
//! TODO(emilio): Look for something like this in crates.io, or publish it?

use precomputed_hash::PrecomputedHash;
use selector_map::PrecomputedDiagnosticHashMap;
use std::borrow::Borrow;
use std::hash::Hash;

/// A map that preserves order for the keys, and that is easily indexable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderedMap<K, V>
where
    K: PrecomputedHash + Hash + Eq + Clone,
{
    /// Key index.
    index: Vec<K>,
    /// Key-value map.
    values: PrecomputedDiagnosticHashMap<K, V>,
}

impl<K, V> OrderedMap<K, V>
where
    K: Eq + PrecomputedHash + Hash + Clone,
{
    /// Creates a new ordered map.
    pub fn new() -> Self {
        OrderedMap {
            index: Vec::new(),
            values: PrecomputedDiagnosticHashMap::default(),
        }
    }

    /// Returns a slice with the keys of this map.
    pub fn keys(&self) -> &[K] {
        &self.index
    }

    /// Creates a new ordered map, with a given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        OrderedMap {
            index: Vec::with_capacity(capacity),
            values: PrecomputedDiagnosticHashMap::with_capacity_and_hasher(capacity, Default::default()),
        }
    }

    /// Insert a new key-value pair.
    pub fn insert(&mut self, key: K, value: V) {
        if !self.values.contains_key(&key) {
            self.index.push(key.clone());
        }
        self.values.insert(key, value);
    }

    /// Get a value given its key.
    pub fn get(&self, key: &K) -> Option<&V> {
        let value = self.values.get(key);
        debug_assert_eq!(value.is_some(), self.index.contains(key));
        value
    }

    /// Get the key located at the given index.
    pub fn get_key_at(&self, index: u32) -> Option<&K> {
        self.index.get(index as usize)
    }

    /// Get an ordered map iterator.
    pub fn iter<'a>(&'a self) -> OrderedMapIterator<'a, K, V> {
        OrderedMapIterator {
            inner: self,
            pos: 0,
        }
    }

    /// Get the count of items in the map.
    pub fn len(&self) -> usize {
        debug_assert_eq!(self.values.len(), self.index.len());
        self.values.len()
    }

    /// Remove an item given its key.
    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: PrecomputedHash + Hash + Eq,
    {
        let index = match self.index.iter().position(|k| k.borrow() == key) {
            Some(p) => p,
            None => return None,
        };
        self.index.remove(index);
        self.values.remove(key)
    }
}

/// An iterator for OrderedMap.
///
/// The iteration order is determined by the order that the values are
/// added to the key-value map.
pub struct OrderedMapIterator<'a, K, V>
where
    K: 'a + Eq + PrecomputedHash + Hash + Clone, V: 'a,
{
    /// The OrderedMap itself.
    inner: &'a OrderedMap<K, V>,
    /// The position of the iterator.
    pos: usize,
}

impl<'a, K, V> Iterator for OrderedMapIterator<'a, K, V>
where
    K: Eq + PrecomputedHash + Hash + Clone,
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        let key = match self.inner.index.get(self.pos) {
            Some(k) => k,
            None => return None,
        };

        self.pos += 1;
        let value = &self.inner.values[key];

        Some((key, value))
    }
}
