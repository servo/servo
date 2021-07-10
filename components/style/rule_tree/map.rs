/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![forbid(unsafe_code)]

use fxhash::FxHashMap;
use malloc_size_of::{MallocShallowSizeOf, MallocSizeOfOps};
use std::collections::hash_map;
use std::hash::Hash;
use std::mem;

pub(super) struct Map<K, V> {
    inner: MapInner<K, V>,
}

enum MapInner<K, V> {
    Empty,
    One(V),
    Map(Box<FxHashMap<K, V>>),
}

pub(super) struct MapIter<'a, K, V> {
    inner: MapIterInner<'a, K, V>,
}

enum MapIterInner<'a, K, V> {
    One(std::option::IntoIter<&'a V>),
    Map(std::collections::hash_map::Values<'a, K, V>),
}

pub(super) enum Entry<'a, K, V> {
    Occupied(&'a mut V),
    Vacant(VacantEntry<'a, K, V>),
}

pub(super) struct VacantEntry<'a, K, V> {
    inner: VacantEntryInner<'a, K, V>,
}

enum VacantEntryInner<'a, K, V> {
    One(&'a mut MapInner<K, V>),
    Map(hash_map::VacantEntry<'a, K, V>),
}

impl<K, V> Default for Map<K, V> {
    fn default() -> Self {
        Map {
            inner: MapInner::Empty,
        }
    }
}

impl<'a, K, V> IntoIterator for &'a Map<K, V> {
    type Item = &'a V;
    type IntoIter = MapIter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        MapIter {
            inner: match &self.inner {
                MapInner::Empty => MapIterInner::One(None.into_iter()),
                MapInner::One(one) => MapIterInner::One(Some(one).into_iter()),
                MapInner::Map(map) => MapIterInner::Map(map.values()),
            },
        }
    }
}

impl<'a, K, V> Iterator for MapIter<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            MapIterInner::One(one_iter) => one_iter.next(),
            MapIterInner::Map(map_iter) => map_iter.next(),
        }
    }
}

impl<K, V> Map<K, V>
where
    K: Eq + Hash,
{
    pub(super) fn is_empty(&self) -> bool {
        match &self.inner {
            MapInner::Empty => true,
            MapInner::One(_) => false,
            MapInner::Map(map) => map.is_empty(),
        }
    }

    #[cfg(debug_assertions)]
    pub(super) fn len(&self) -> usize {
        match &self.inner {
            MapInner::Empty => 0,
            MapInner::One(_) => 1,
            MapInner::Map(map) => map.len(),
        }
    }

    pub(super) fn get(&self, key: &K, key_from_value: impl FnOnce(&V) -> K) -> Option<&V> {
        match &self.inner {
            MapInner::One(one) if *key == key_from_value(one) => Some(one),
            MapInner::Map(map) => map.get(key),
            MapInner::Empty | MapInner::One(_) => None,
        }
    }

    pub(super) fn entry(
        &mut self,
        key: K,
        key_from_value: impl FnOnce(&V) -> K,
    ) -> Entry<'_, K, V> {
        match self.inner {
            ref mut inner @ MapInner::Empty => Entry::Vacant(VacantEntry {
                inner: VacantEntryInner::One(inner),
            }),
            MapInner::One(_) => {
                let one = match mem::replace(&mut self.inner, MapInner::Empty) {
                    MapInner::One(one) => one,
                    _ => unreachable!(),
                };
                // If this panics, the child `one` will be lost.
                let one_key = key_from_value(&one);
                // Same for the equality test.
                if key == one_key {
                    self.inner = MapInner::One(one);
                    let one = match &mut self.inner {
                        MapInner::One(one) => one,
                        _ => unreachable!(),
                    };
                    return Entry::Occupied(one);
                }
                self.inner = MapInner::Map(Box::new(FxHashMap::with_capacity_and_hasher(
                    2,
                    Default::default(),
                )));
                let map = match &mut self.inner {
                    MapInner::Map(map) => map,
                    _ => unreachable!(),
                };
                map.insert(one_key, one);
                match map.entry(key) {
                    hash_map::Entry::Vacant(entry) => Entry::Vacant(VacantEntry {
                        inner: VacantEntryInner::Map(entry),
                    }),
                    _ => unreachable!(),
                }
            },
            MapInner::Map(ref mut map) => match map.entry(key) {
                hash_map::Entry::Occupied(entry) => Entry::Occupied(entry.into_mut()),
                hash_map::Entry::Vacant(entry) => Entry::Vacant(VacantEntry {
                    inner: VacantEntryInner::Map(entry),
                }),
            },
        }
    }

    pub(super) fn remove(&mut self, key: &K, key_from_value: impl FnOnce(&V) -> K) -> Option<V> {
        match &mut self.inner {
            MapInner::One(one) if *key == key_from_value(one) => {
                match mem::replace(&mut self.inner, MapInner::Empty) {
                    MapInner::One(one) => Some(one),
                    _ => unreachable!(),
                }
            },
            MapInner::Map(map) => map.remove(key),
            MapInner::Empty | MapInner::One(_) => None,
        }
    }
}

impl<'a, K, V> VacantEntry<'a, K, V> {
    pub(super) fn insert(self, value: V) -> &'a mut V {
        match self.inner {
            VacantEntryInner::One(map) => {
                *map = MapInner::One(value);
                match map {
                    MapInner::One(one) => one,
                    _ => unreachable!(),
                }
            },
            VacantEntryInner::Map(entry) => entry.insert(value),
        }
    }
}

impl<K, V> MallocShallowSizeOf for Map<K, V>
where
    K: Eq + Hash,
{
    fn shallow_size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        match &self.inner {
            MapInner::Map(m) => {
                // We want to account for both the box and the hashmap.
                m.shallow_size_of(ops) + (**m).shallow_size_of(ops)
            },
            MapInner::One(_) | MapInner::Empty => 0,
        }
    }
}
