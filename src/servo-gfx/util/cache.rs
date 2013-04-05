/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub trait Cache<K: Copy + Eq, V: Copy> {
    fn new(size: uint) -> Self;
    fn insert(&mut self, key: &K, value: V);
    fn find(&self, key: &K) -> Option<V>;
    fn find_or_create(&mut self, key: &K, blk: &fn(&K) -> V) -> V;
    fn evict_all(&mut self);
}

pub struct MonoCache<K, V> {
    entry: Option<(K,V)>,
}

impl<K: Copy + Eq, V: Copy> Cache<K,V> for MonoCache<K,V> {
    fn new(_size: uint) -> MonoCache<K,V> {
        MonoCache { entry: None }
    }

    fn insert(&mut self, key: &K, value: V) {
        self.entry = Some((copy *key, value));
    }

    fn find(&self, key: &K) -> Option<V> {
        match self.entry {
            None => None,
            Some((ref k,v)) => if *k == *key { Some(v) } else { None }
        }
    }

    fn find_or_create(&mut self, key: &K, blk: &fn(&K) -> V) -> V {
        return match self.find(key) {
            None => { 
                let value = blk(key);
                self.entry = Some((copy *key, copy value));
                value
            },
            Some(v) => v
        };
    }
    fn evict_all(&mut self) {
        self.entry = None;
    }
}

#[test]
fn test_monocache() {
    // TODO: this is hideous because of Rust Issue #3902
    let cache = cache::new::<uint, @str, MonoCache<uint, @str>>(10);
    let one = @"one";
    let two = @"two";
    cache.insert(&1, one);

    assert!(cache.find(&1).is_some());
    assert!(cache.find(&2).is_none());
    cache.find_or_create(&2, |_v| { two });
    assert!(cache.find(&2).is_some());
    assert!(cache.find(&1).is_none());
}
