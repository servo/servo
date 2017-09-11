/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A simple LRU cache.

#![deny(missing_docs)]

extern crate arraydeque;
use self::arraydeque::Array;
use self::arraydeque::ArrayDeque;

/// A LRU cache used to store a set of at most `n` elements at the same time.
///
/// The most-recently-used entry is at index zero.
pub struct LRUCache <K: Array>{
    entries: ArrayDeque<K>,
}

/// A iterator over the items of the LRU cache.
pub type LRUCacheIterator<'a, K> = arraydeque::Iter<'a, K>;

/// A iterator over the mutable items of the LRU cache.
pub type LRUCacheMutIterator<'a, K> = arraydeque::IterMut<'a, K>;

impl<K: Array> LRUCache<K> {
    /// Create a new LRU cache with `size` elements at most.
    pub fn new() -> Self {
        LRUCache {
          entries: ArrayDeque::new(),
        }
    }

    /// Returns the number of elements in the cache.
    pub fn num_entries(&self) -> usize {
        self.entries.len()
    }

    #[inline]
    /// Touch a given entry, putting it first in the list.
    pub fn touch(&mut self, pos: usize) {
        if pos != 0 {
            let entry = self.entries.remove(pos).unwrap();
            self.entries.push_front(entry);
        }
    }

    /// Returns the front entry in the list (most recently used).
    pub fn front(&self) -> Option<&K::Item> {
        self.entries.get(0)
    }

    /// Returns a mutable reference to the front entry in the list (most recently used).
    pub fn front_mut(&mut self) -> Option<&mut K::Item> {
        self.entries.get_mut(0)
    }

    /// Iterate over the contents of this cache, from more to less recently
    /// used.
    pub fn iter(&self) -> arraydeque::Iter<K::Item> {
        self.entries.iter()
    }

    /// Iterate mutably over the contents of this cache.
    pub fn iter_mut(&mut self) -> arraydeque::IterMut<K::Item> {
        self.entries.iter_mut()
    }

    /// Insert a given key in the cache.
    pub fn insert(&mut self, key: K::Item) {
        if self.entries.len() == self.entries.capacity() {
            self.entries.pop_back();
        }
        self.entries.push_front(key);
        debug_assert!(self.entries.len() <= self.entries.capacity());
    }

    /// Evict all elements from the cache.
    pub fn evict_all(&mut self) {
        self.entries.clear();
    }
}
