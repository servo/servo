/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A simple LRU cache.

extern crate arrayvec;

use arrayvec::{Array, ArrayVec};

/// A LRU cache using a statically-sized array for storage.
///
/// The most-recently-used entry is at index `head`. The entries form a linked list, linked to each
/// other by indices within the `entries` array.  After an entry is added to the array, its index
/// never changes, so these links are never invalidated.
pub struct LRUCache<T, A: Array<Item=Entry<T>>> {
    entries: ArrayVec<A>,
    /// Index of the first entry. If the cache is empty, ignore this field.
    head: u16,
    /// Index of the last entry. If the cache is empty, ignore this field.
    tail: u16,
}

/// An opaque token used as an index into an LRUCache.
pub struct CacheIndex(u16);

/// An entry in an LRUCache.
pub struct Entry<T> {
    val: T,
    /// Index of the previous entry. If this entry is the head, ignore this field.
    prev: u16,
    /// Index of the next entry. If this entry is the tail, ignore this field.
    next: u16,
}

impl<T, A: Array<Item=Entry<T>>> Default for LRUCache<T, A> {
    fn default() -> Self {
        let cache = LRUCache {
            entries: ArrayVec::new(),
            head: 0,
            tail: 0,
        };
        assert!(cache.entries.capacity() < u16::max_value() as usize, "Capacity overflow");
        cache
    }
}

impl<T, A: Array<Item=Entry<T>>> LRUCache<T, A> {
    /// Returns the number of elements in the cache.
    pub fn num_entries(&self) -> usize {
        self.entries.len()
    }

    #[inline]
    /// Touch a given entry, putting it first in the list.
    pub fn touch(&mut self, idx: CacheIndex) {
        if idx.0 != self.head {
            self.remove(idx.0);
            self.push_front(idx.0);
        }
    }

    /// Returns the front entry in the list (most recently used).
    pub fn front(&self) -> Option<&T> {
        self.entries.get(self.head as usize).map(|e| &e.val)
    }

    /// Returns a mutable reference to the front entry in the list (most recently used).
    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.entries.get_mut(self.head as usize).map(|e| &mut e.val)
    }

    /// Iterate over the contents of this cache, from more to less recently
    /// used.
    pub fn iter(&self) -> LRUCacheIterator<T, A> {
        LRUCacheIterator {
            pos: self.head,
            done: self.entries.len() == 0,
            cache: self,
        }
    }

    /// Iterate mutably over the contents of this cache.
    pub fn iter_mut(&mut self) -> LRUCacheMutIterator<T, A> {
        LRUCacheMutIterator {
            pos: self.head,
            done: self.entries.len() == 0,
            cache: self,
        }
    }

    /// Performs a lookup on the cache with the given test routine. Touches
    /// the result on a hit.
    pub fn lookup<F, R>(&mut self, mut test_one: F) -> Option<R>
    where
        F: FnMut(&mut T) -> Option<R>
    {
        let mut result = None;
        for (i, candidate) in self.iter_mut() {
            if let Some(r) = test_one(candidate) {
                result = Some((i, r));
                break;
            }
        };

        match result {
            None => None,
            Some((i, r)) => {
                self.touch(i);
                let front = self.front_mut().unwrap();
                debug_assert!(test_one(front).is_some());
                Some(r)
            }
        }
    }

    /// Insert a given key in the cache.
    pub fn insert(&mut self, val: T) {
        let entry = Entry { val, prev: 0, next: 0 };

        // If the cache is full, replace the oldest entry. Otherwise, add an entry.
        let new_head = if self.entries.len() == self.entries.capacity() {
            let i = self.pop_back();
            self.entries[i as usize] = entry;
            i
        } else {
            self.entries.push(entry);
            self.entries.len() as u16 - 1
        };

        self.push_front(new_head);
    }

    /// Remove an from the linked list.
    ///
    /// Note: This only unlinks the entry from the list; it does not remove it from the array.
    fn remove(&mut self, i: u16) {
        let prev = self.entries[i as usize].prev;
        let next = self.entries[i as usize].next;

        if i == self.head {
            self.head = next;
        } else {
            self.entries[prev as usize].next = next;
        }

        if i == self.tail {
            self.tail = prev;
        } else {
            self.entries[next as usize].prev = prev;
        }
    }

    /// Insert a new entry at the head of the list.
    fn push_front(&mut self, i: u16) {
        if self.entries.len() == 1 {
            self.tail = i;
        } else {
            self.entries[i as usize].next = self.head;
            self.entries[self.head as usize].prev = i;
        }
        self.head = i;
    }

    /// Remove the last entry from the linked list. Returns the index of the removed entry.
    ///
    /// Note: This only unlinks the entry from the list; it does not remove it from the array.
    fn pop_back(&mut self) -> u16 {
        let old_tail = self.tail;
        let new_tail = self.entries[old_tail as usize].prev;
        self.tail = new_tail;
        old_tail
    }

    /// Evict all elements from the cache.
    pub fn evict_all(&mut self) {
        self.entries.clear();
    }
}

/// Immutable iterator over values in an LRUCache, from most-recently-used to least-recently-used.
pub struct LRUCacheIterator<'a, T: 'a, A: 'a + Array<Item=Entry<T>>> {
    cache: &'a LRUCache<T, A>,
    pos: u16,
    done: bool,
}

impl<'a, T, A> Iterator for LRUCacheIterator<'a, T, A>
where T: 'a,
      A: 'a + Array<Item=Entry<T>>
{
    type Item = (CacheIndex, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.done { return None }

        let entry = &self.cache.entries[self.pos as usize];

        let index = CacheIndex(self.pos);
        if self.pos == self.cache.tail {
            self.done = true;
        }
        self.pos = entry.next;

        Some((index, &entry.val))
    }
}

/// Mutable iterator over values in an LRUCache, from most-recently-used to least-recently-used.
pub struct LRUCacheMutIterator<'a, T: 'a, A: 'a + Array<Item=Entry<T>>> {
    cache: &'a mut LRUCache<T, A>,
    pos: u16,
    done: bool,
}

impl<'a, T, A> Iterator for LRUCacheMutIterator<'a, T, A>
where T: 'a,
      A: 'a + Array<Item=Entry<T>>
{
    type Item = (CacheIndex, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.done { return None }

        // Use a raw pointer because the compiler doesn't know that subsequent calls can't alias.
        let entry = unsafe {
            &mut *(&mut self.cache.entries[self.pos as usize] as *mut Entry<T>)
        };

        let index = CacheIndex(self.pos);
        if self.pos == self.cache.tail {
            self.done = true;
        }
        self.pos = entry.next;

        Some((index, &mut entry.val))
    }
}
