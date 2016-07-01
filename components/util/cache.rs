/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::default::Default;
use std::hash::{BuildHasherDefault, Hash, SipHasher};


#[derive(Debug)]
pub struct HashCache<K, V>
    where K: PartialEq + Eq + Hash,
          V: Clone,
{
    entries: HashMap<K, V, BuildHasherDefault<SipHasher>>,
}

impl<K, V> HashCache<K, V>
    where K: PartialEq + Eq + Hash,
          V: Clone,
{
    pub fn new() -> HashCache<K, V> {
        HashCache {
          entries: HashMap::with_hasher(Default::default()),
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

    pub fn find_or_create<F>(&mut self, key: K, mut blk: F) -> V where F: FnMut() -> V {
        match self.entries.entry(key) {
            Occupied(occupied) => {
                (*occupied.get()).clone()
            }
            Vacant(vacant) => {
                (*vacant.insert(blk())).clone()
            }
        }
    }

    pub fn evict_all(&mut self) {
        self.entries.clear();
    }
}
