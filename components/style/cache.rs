/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A simple LRU cache.

#![deny(missing_docs)]

use std::slice::{Iter, IterMut};

/// A LRU cache used to store a set of at most `n` elements at the same time.
///
/// Currently used for the style sharing candidate cache.
pub struct LRUCache<K, V> {
    entries: Vec<(K, V)>,
    cache_size: usize,
}

impl<K: PartialEq, V: Clone> LRUCache<K, V> {
    /// Create a new LRU cache with `size` elements at most.
    pub fn new(size: usize) -> LRUCache<K, V> {
        LRUCache {
          entries: vec![],
          cache_size: size,
        }
    }

    #[inline]
    /// Touch a given position, and put it in the last item on the list.
    pub fn touch(&mut self, pos: usize) -> &V {
        let last_index = self.entries.len() - 1;
        if pos != last_index {
            let entry = self.entries.remove(pos);
            self.entries.push(entry);
        }
        &self.entries[last_index].1
    }

    /// Iterate over the contents of this cache.
    pub fn iter(&self) -> Iter<(K, V)> {
        self.entries.iter()
    }

    /// Iterate mutably over the contents of this cache.
    pub fn iter_mut(&mut self) -> IterMut<(K, V)> {
        self.entries.iter_mut()
    }

    /// Insert a given key and value in the cache.
    pub fn insert(&mut self, key: K, val: V) {
        if self.entries.len() == self.cache_size {
            self.entries.remove(0);
        }
        self.entries.push((key, val));
    }

    /// Try to find a key in the cache.
    pub fn find(&mut self, key: &K) -> Option<V> {
        match self.entries.iter().position(|&(ref k, _)| key == k) {
            Some(pos) => Some(self.touch(pos).clone()),
            None      => None,
        }
    }

    /// Try to find a given key, or create a given item with that key executing
    /// `blk`.
    pub fn find_or_create<F>(&mut self, key: K, mut blk: F) -> V
        where F: FnMut() -> V,
    {
        match self.entries.iter().position(|&(ref k, _)| *k == key) {
            Some(pos) => self.touch(pos).clone(),
            None => {
                let val = blk();
                self.insert(key, val.clone());
                val
            }
        }
    }

    /// Evict all elements from the cache.
    pub fn evict_all(&mut self) {
        self.entries.clear();
    }
}
