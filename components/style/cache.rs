/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A simple LRU cache.

#![deny(missing_docs)]

use std::collections::VecDeque;
use std::collections::vec_deque;

/// A LRU cache used to store a set of at most `n` elements at the same time.
///
/// The most-recently-used entry is at index zero.
pub struct LRUCache<K> {
    entries: VecDeque<K>,
    cache_size: usize,
}

/// A iterator over the items of the LRU cache.
pub type LRUCacheIterator<'a, K> = vec_deque::Iter<'a, K>;

/// A iterator over the mutable items of the LRU cache.
pub type LRUCacheMutIterator<'a, K> = vec_deque::IterMut<'a, K>;

impl<K: PartialEq> LRUCache<K> {
    /// Create a new LRU cache with `size` elements at most.
    pub fn new(size: usize) -> Self {
        LRUCache {
          entries: VecDeque::with_capacity(size),
          cache_size: size,
        }
    }

    /// Returns the number of elements in the cache.
    pub fn num_entries(&self) -> usize {
        self.entries.len()
    }

    #[inline]
    /// Touch a given entry, putting it first in the list.
    pub fn touch(&mut self, pos: usize) {
        let last_index = self.entries.len() - 1;
        if pos != last_index {
            let entry = self.entries.remove(pos).unwrap();
            self.entries.push_front(entry);
        }
    }

    /// Iterate over the contents of this cache, from more to less recently
    /// used.
    pub fn iter(&self) -> vec_deque::Iter<K> {
        self.entries.iter()
    }

    /// Iterate mutably over the contents of this cache.
    pub fn iter_mut(&mut self) -> vec_deque::IterMut<K> {
        self.entries.iter_mut()
    }

    /// Insert a given key in the cache.
    pub fn insert(&mut self, key: K) {
        if self.entries.len() == self.cache_size {
            self.entries.pop_back();
        }
        self.entries.push_front(key);
        debug_assert!(self.entries.len() <= self.cache_size);
    }

    /// Evict all elements from the cache.
    pub fn evict_all(&mut self) {
        self.entries.clear();
    }
}
