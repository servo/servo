/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::hash_state::DefaultState;
use rand::Rng;
use std::hash::{Hash, Hasher, SipHasher};
use rand;
use std::slice::Iter;
use std::default::Default;


pub struct HashCache<K, V> {
    entries: HashMap<K, V, DefaultState<SipHasher>>,
}

impl<K, V> HashCache<K,V>
    where K: Clone + PartialEq + Eq + Hash,
          V: Clone,
{
    pub fn new() -> HashCache<K,V> {
        HashCache {
          entries: HashMap::with_hash_state(<DefaultState<SipHasher> as Default>::default()),
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.entries.insert(key, value);
    }

    pub fn find(&self, key: &K) -> Option<V> {
        match self.entries.get(key) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }

    pub fn find_or_create<F>(&mut self, key: &K, blk: F) -> V where F: Fn(&K) -> V {
        match self.entries.entry(key.clone()) {
            Occupied(occupied) => {
                (*occupied.get()).clone()
            }
            Vacant(vacant) => {
                (*vacant.insert(blk(key))).clone()
            }
        }
    }

    pub fn evict_all(&mut self) {
        self.entries.clear();
    }
}

pub struct LRUCache<K, V> {
    entries: Vec<(K, V)>,
    cache_size: usize,
}

impl<K: Clone + PartialEq, V: Clone> LRUCache<K,V> {
    pub fn new(size: usize) -> LRUCache<K, V> {
        LRUCache {
          entries: vec!(),
          cache_size: size,
        }
    }

    #[inline]
    pub fn touch(&mut self, pos: usize) -> V {
        let last_index = self.entries.len() - 1;
        if pos != last_index {
            let entry = self.entries.remove(pos);
            self.entries.push(entry);
        }
        self.entries[last_index].1.clone()
    }

    pub fn iter<'a>(&'a self) -> Iter<'a,(K,V)> {
        self.entries.iter()
    }

    pub fn insert(&mut self, key: K, val: V) {
        if self.entries.len() == self.cache_size {
            self.entries.remove(0);
        }
        self.entries.push((key, val));
    }

    pub fn find(&mut self, key: &K) -> Option<V> {
        match self.entries.iter().position(|&(ref k, _)| key == k) {
            Some(pos) => Some(self.touch(pos)),
            None      => None,
        }
    }

    pub fn find_or_create<F>(&mut self, key: &K, blk: F) -> V where F: Fn(&K) -> V {
        match self.entries.iter().position(|&(ref k, _)| *k == *key) {
            Some(pos) => self.touch(pos),
            None => {
                let val = blk(key);
                self.insert(key.clone(), val.clone());
                val
            }
        }
    }

    pub fn evict_all(&mut self) {
        self.entries.clear();
    }
}

pub struct SimpleHashCache<K,V> {
    entries: Vec<Option<(K,V)>>,
    k0: u64,
    k1: u64,
}

impl<K:Clone+Eq+Hash,V:Clone> SimpleHashCache<K,V> {
    pub fn new(cache_size: usize) -> SimpleHashCache<K,V> {
        let mut r = rand::thread_rng();
        SimpleHashCache {
            entries: vec![None; cache_size],
            k0: r.gen(),
            k1: r.gen(),
        }
    }

    #[inline]
    fn to_bucket(&self, h: usize) -> usize {
        h % self.entries.len()
    }

    #[inline]
    fn bucket_for_key<Q:Hash>(&self, key: &Q) -> usize {
        let mut hasher = SipHasher::new_with_keys(self.k0, self.k1);
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

    pub fn find_or_create<F>(&mut self, key: &K, blk: F) -> V where F: Fn(&K) -> V {
        match self.find(key) {
            Some(value) => return value,
            None => {}
        }
        let value = blk(key);
        self.insert((*key).clone(), value.clone());
        value
    }

    pub fn evict_all(&mut self) {
        for slot in &mut self.entries {
            *slot = None
        }
    }
}
