/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A simple LRU cache.

#![deny(missing_docs)]

use std::slice::{Iter, IterMut};

/// A LRU cache used to store a set of at most `n` elements at the same time.
///
/// Currently used for the style sharing candidate cache.
pub struct LRUCache<K> {
    entries: Vec<K>,
    cache_size: usize,
}

impl<K: PartialEq> LRUCache<K> {
    /// Create a new LRU cache with `size` elements at most.
    pub fn new(size: usize) -> Self {
        LRUCache {
          entries: vec![],
          cache_size: size,
        }
    }

    #[inline]
    /// Touch a given position, and put it in the last item on the list.
    pub fn touch(&mut self, pos: usize) {
        let last_index = self.entries.len() - 1;
        if pos != last_index {
            let entry = self.entries.remove(pos);
            self.entries.push(entry);
        }
    }

    /// Iterate over the contents of this cache.
    pub fn iter(&self) -> Iter<K> {
        self.entries.iter()
    }

    /// Iterate mutably over the contents of this cache.
    pub fn iter_mut(&mut self) -> IterMut<K> {
        self.entries.iter_mut()
    }

    /// Insert a given key in the cache.
    pub fn insert(&mut self, key: K) {
        if self.entries.len() == self.cache_size {
            self.entries.remove(0);
        }
        self.entries.push(key);
    }

    /// Evict all elements from the cache.
    pub fn evict_all(&mut self) {
        self.entries.clear();
    }
}
