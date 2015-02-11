/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![old_impl_check]

use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::hash_state::DefaultState;
use rand::Rng;
use std::hash::{Hash, Hasher, SipHasher};
use std::iter::repeat;
use rand;
use std::slice::Iter;

#[cfg(test)]
use std::cell::Cell;

pub struct HashCache<K, V> {
    entries: HashMap<K, V, DefaultState<SipHasher>>,
}

impl<K, V> HashCache<K,V>
    where K: Clone + PartialEq + Eq + Hash<SipHasher>,
          V: Clone,
{
    pub fn new() -> HashCache<K,V> {
        HashCache {
          entries: HashMap::with_hash_state(DefaultState),
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

#[test]
fn test_hashcache() {
    let mut cache: HashCache<uint, Cell<&str>> = HashCache::new();

    cache.insert(1, Cell::new("one"));
    assert!(cache.find(&1).is_some());
    assert!(cache.find(&2).is_none());

    cache.find_or_create(&2, |_v| { Cell::new("two") });
    assert!(cache.find(&1).is_some());
    assert!(cache.find(&2).is_some());
}

pub struct LRUCache<K, V> {
    entries: Vec<(K, V)>,
    cache_size: uint,
}

impl<K: Clone + PartialEq, V: Clone> LRUCache<K,V> {
    pub fn new(size: uint) -> LRUCache<K, V> {
        LRUCache {
          entries: vec!(),
          cache_size: size,
        }
    }

    #[inline]
    pub fn touch(&mut self, pos: uint) -> V {
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

impl<K:Clone+Eq+Hash<SipHasher>,V:Clone> SimpleHashCache<K,V> {
    pub fn new(cache_size: uint) -> SimpleHashCache<K,V> {
        let mut r = rand::thread_rng();
        SimpleHashCache {
            entries: repeat(None).take(cache_size).collect(),
            k0: r.gen(),
            k1: r.gen(),
        }
    }

    #[inline]
    fn to_bucket(&self, h: uint) -> uint {
        h % self.entries.len()
    }

    #[inline]
    fn bucket_for_key<Q:Hash<SipHasher>>(&self, key: &Q) -> uint {
        let mut hasher = SipHasher::new_with_keys(self.k0, self.k1);
        key.hash(&mut hasher);
        self.to_bucket(hasher.finish() as uint)
    }

    pub fn insert(&mut self, key: K, value: V) {
        let bucket_index = self.bucket_for_key(&key);
        self.entries[bucket_index] = Some((key, value));
    }

    pub fn find<Q>(&self, key: &Q) -> Option<V> where Q: PartialEq<K> + Hash<SipHasher> + Eq {
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
        for slot in self.entries.iter_mut() {
            *slot = None
        }
    }
}

#[test]
fn test_lru_cache() {
    let one = Cell::new("one");
    let two = Cell::new("two");
    let three = Cell::new("three");
    let four = Cell::new("four");

    // Test normal insertion.
    let mut cache: LRUCache<uint,Cell<&str>> = LRUCache::new(2); // (_, _) (cache is empty)
    cache.insert(1, one);    // (1, _)
    cache.insert(2, two);    // (1, 2)
    cache.insert(3, three);  // (2, 3)

    assert!(cache.find(&1).is_none());  // (2, 3) (no change)
    assert!(cache.find(&3).is_some());  // (2, 3)
    assert!(cache.find(&2).is_some());  // (3, 2)

    // Test that LRU works (this insertion should replace 3, not 2).
    cache.insert(4, four); // (2, 4)

    assert!(cache.find(&1).is_none());  // (2, 4) (no change)
    assert!(cache.find(&2).is_some());  // (4, 2)
    assert!(cache.find(&3).is_none());  // (4, 2) (no change)
    assert!(cache.find(&4).is_some());  // (2, 4) (no change)

    // Test find_or_create.
    cache.find_or_create(&1, |_| { Cell::new("one") }); // (4, 1)

    assert!(cache.find(&1).is_some()); // (4, 1) (no change)
    assert!(cache.find(&2).is_none()); // (4, 1) (no change)
    assert!(cache.find(&3).is_none()); // (4, 1) (no change)
    assert!(cache.find(&4).is_some()); // (1, 4)
}
