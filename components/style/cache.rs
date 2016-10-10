/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Two simple cache data structures.

use std::collections::hash_map::RandomState;
use std::hash::{Hash, Hasher, BuildHasher};
use std::slice::{Iter, IterMut};

pub struct LRUCache<K, V> {
    entries: Vec<(K, V)>,
    cache_size: usize,
}

impl<K: PartialEq, V: Clone> LRUCache<K, V> {
    pub fn new(size: usize) -> LRUCache<K, V> {
        LRUCache {
          entries: vec![],
          cache_size: size,
        }
    }

    #[inline]
    pub fn touch(&mut self, pos: usize) -> &V {
        let last_index = self.entries.len() - 1;
        if pos != last_index {
            let entry = self.entries.remove(pos);
            self.entries.push(entry);
        }
        &self.entries[last_index].1
    }

    pub fn iter(&self) -> Iter<(K, V)> {
        self.entries.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<(K, V)> {
        self.entries.iter_mut()
    }

    pub fn insert(&mut self, key: K, val: V) {
        if self.entries.len() == self.cache_size {
            self.entries.remove(0);
        }
        self.entries.push((key, val));
    }

    pub fn find(&mut self, key: &K) -> Option<V> {
        match self.entries.iter().position(|&(ref k, _)| key == k) {
            Some(pos) => Some(self.touch(pos).clone()),
            None      => None,
        }
    }

    pub fn find_or_create<F>(&mut self, key: K, mut blk: F) -> V where F: FnMut() -> V {
        match self.entries.iter().position(|&(ref k, _)| *k == key) {
            Some(pos) => self.touch(pos).clone(),
            None => {
                let val = blk();
                self.insert(key, val.clone());
                val
            }
        }
    }

    pub fn evict_all(&mut self) {
        self.entries.clear();
    }
}

pub struct SimpleHashCache<K, V> {
    entries: Vec<Option<(K, V)>>,
    random: RandomState,
}

impl<K: Clone + Eq + Hash, V: Clone> SimpleHashCache<K, V> {
    pub fn new(cache_size: usize) -> SimpleHashCache<K, V> {
        SimpleHashCache {
            entries: vec![None; cache_size],
            random: RandomState::new(),
        }
    }

    #[inline]
    fn to_bucket(&self, h: usize) -> usize {
        h % self.entries.len()
    }

    #[inline]
    fn bucket_for_key<Q: Hash>(&self, key: &Q) -> usize {
        let mut hasher = self.random.build_hasher();
        key.hash(&mut hasher);
        self.to_bucket(hasher.finish() as usize)
    }

    pub fn insert(&mut self, key: K, value: V) {
        let bucket_index = self.bucket_for_key(&key);
        self.entries[bucket_index] = Some((key, value));
    }

    pub fn find<Q>(&self, key: &Q) -> Option<V> where Q: PartialEq<K> + Hash + Eq {
        let bucket_index = self.bucket_for_key(key);
        match self.entries[bucket_index] {
            Some((ref existing_key, ref value)) if key == existing_key => Some((*value).clone()),
            _ => None,
        }
    }

    pub fn find_or_create<F>(&mut self, key: K, mut blk: F) -> V where F: FnMut() -> V {
        if let Some(value) = self.find(&key) {
            return value;
        }
        let value = blk();
        self.insert(key, value.clone());
        value
    }

    pub fn evict_all(&mut self) {
        for slot in &mut self.entries {
            *slot = None
        }
    }
}
