/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::hashmap::HashMap;

pub trait Cache<K: Eq, V: Clone> {
    fn insert(&mut self, key: K, value: V);
    fn find(&mut self, key: &K) -> Option<V>;
    fn find_or_create(&mut self, key: &K, blk: |&K| -> V) -> V;
    fn evict_all(&mut self);
}

pub struct MonoCache<K, V> {
    entry: Option<(K,V)>,
}

impl<K: Clone + Eq, V: Clone> MonoCache<K,V> {
    pub fn new(_size: uint) -> MonoCache<K,V> {
        MonoCache { entry: None }
    }
}

impl<K: Clone + Eq, V: Clone> Cache<K,V> for MonoCache<K,V> {
    fn insert(&mut self, key: K, value: V) {
        self.entry = Some((key, value));
    }

    fn find(&mut self, key: &K) -> Option<V> {
        match self.entry {
            None => None,
            Some((ref k, ref v)) => if *k == *key { Some(v.clone()) } else { None }
        }
    }

    fn find_or_create(&mut self, key: &K, blk: |&K| -> V) -> V {
        match self.find(key) {
            Some(value) => value,
            None => {
                let value = blk(key);
                self.entry = Some((key.clone(), value.clone()));
                value
            }
        }
    }

    fn evict_all(&mut self) {
        self.entry = None;
    }
}

#[test]
fn test_monocache() {
    let mut cache = MonoCache::new(10);
    let one = @"one";
    let two = @"two";
    cache.insert(1, one);

    assert!(cache.find(&1).is_some());
    assert!(cache.find(&2).is_none());
    cache.find_or_create(&2, |_v| { two });
    assert!(cache.find(&2).is_some());
    assert!(cache.find(&1).is_none());
}

pub struct HashCache<K, V> {
    entries: HashMap<K, V>,
}

impl<K: Clone + Eq + Hash, V: Clone> HashCache<K,V> {
    pub fn new() -> HashCache<K, V> {
        HashCache {
          entries: HashMap::new(),
        }
    }
}

impl<K: Clone + Eq + Hash, V: Clone> Cache<K,V> for HashCache<K,V> {
    fn insert(&mut self, key: K, value: V) {
        self.entries.insert(key, value);
    }

    fn find(&mut self, key: &K) -> Option<V> {
        match self.entries.find(key) {
            Some(v) => Some(v.clone()),
            None    => None,
        }
    }

    fn find_or_create(&mut self, key: &K, blk: |&K| -> V) -> V {
        self.entries.find_or_insert_with(key.clone(), blk).clone()
    }

    fn evict_all(&mut self) {
        self.entries.clear();
    }
}

#[test]
fn test_hashcache() {
    let mut cache = HashCache::new();
    let one = @"one";
    let two = @"two";

    cache.insert(1, one);
    assert!(cache.find(&1).is_some());
    assert!(cache.find(&2).is_none());

    cache.find_or_create(&2, |_v| { two });
    assert!(cache.find(&1).is_some());
    assert!(cache.find(&2).is_some());
}

pub struct LRUCache<K, V> {
    entries: ~[(K, V)],
    cache_size: uint,
}

impl<K: Clone + Eq, V: Clone> LRUCache<K,V> {
    pub fn new(size: uint) -> LRUCache<K, V> {
        LRUCache {
          entries: ~[],
          cache_size: size,
        }
    }

    pub fn touch(&mut self, pos: uint) -> V {
        let last_index = self.entries.len() - 1;
        if pos != last_index {
            let entry = self.entries.remove(pos);
            self.entries.push(entry);
        }
        self.entries[last_index].second_ref().clone()
    }
}

impl<K: Clone + Eq, V: Clone> Cache<K,V> for LRUCache<K,V> {
    fn insert(&mut self, key: K, val: V) {
        if self.entries.len() == self.cache_size {
            self.entries.remove(0);
        }
        self.entries.push((key, val));
    }

    fn find(&mut self, key: &K) -> Option<V> {
        match self.entries.iter().position(|&(ref k, _)| *k == *key) {
            Some(pos) => Some(self.touch(pos)),
            None      => None,
        }
    }

    fn find_or_create(&mut self, key: &K, blk: |&K| -> V) -> V {
        match self.entries.iter().position(|&(ref k, _)| *k == *key) {
            Some(pos) => self.touch(pos),
            None => {
                let val = blk(key);
                self.insert(key.clone(), val.clone());
                val
            }
        }
    }

    fn evict_all(&mut self) {
        self.entries.clear();
    }
}

#[test]
fn test_lru_cache() {
    let one = @"one";
    let two = @"two";
    let three = @"three";
    let four = @"four";

    // Test normal insertion.
    let mut cache = LRUCache::new(2); // (_, _) (cache is empty)
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
    cache.find_or_create(&1, |_| { one }); // (4, 1)

    assert!(cache.find(&1).is_some()); // (4, 1) (no change)
    assert!(cache.find(&2).is_none()); // (4, 1) (no change)
    assert!(cache.find(&3).is_none()); // (4, 1) (no change)
    assert!(cache.find(&4).is_some()); // (1, 4)
}
