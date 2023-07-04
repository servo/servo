/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::freelist::{FreeList, FreeListHandle, WeakFreeListHandle};
use std::{mem, num};

/*
  This module implements a least recently used cache structure, which is
  used by the texture cache to manage the lifetime of items inside the
  texture cache. It has a few special pieces of functionality that the
  texture cache requires, but should be usable as a general LRU cache
  type if useful in other areas.

  The cache is implemented with two backing freelists. These allow
  random access to the underlying data, while being efficient in both
  memory access and allocation patterns.

  The first freelist stores the elements being cached (for example, the
  CacheEntry structure for the texture cache). These elements are stored
  in arbitrary order, reusing empty slots in the freelist where possible.

  The second freelist stores the LRU tracking information. Although the
  tracking elements are stored in arbitrary order inside a freelist for
  efficiency, they use next/prev links to represent a doubly-linked list,
  kept sorted in order of recent use. The next link is also used to store
  the current freelist within the array when the element is not occupied.
 */

/// Stores the data supplied by the user to be cached, and an index
/// into the LRU tracking freelist for this element.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
struct LRUCacheEntry<T> {
    /// The location of the LRU tracking element for this cache entry.
    /// This is None if the entry has manual eviction policy enabled.
    lru_index: Option<ItemIndex>,
    /// The cached data provided by the caller for this element.
    value: T,
}

/// The main public interface to the LRU cache
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct LRUCache<T, M> {
    /// A free list of cache entries, and indices into the LRU tracking list
    entries: FreeList<LRUCacheEntry<T>, M>,
    /// The LRU tracking list, allowing O(1) access to the oldest element
    lru: LRUTracker<FreeListHandle<M>>,
}

impl<T, M> LRUCache<T, M> {
    /// Construct a new LRU cache
    pub fn new() -> Self {
        LRUCache {
            entries: FreeList::new(),
            lru: LRUTracker::new(),
        }
    }

    /// Insert a new element into the cache. Returns a weak handle for callers to
    /// access the data, since the lifetime is managed by the LRU algorithm and it
    /// may be evicted at any time.
    pub fn push_new(
        &mut self,
        value: T,
    ) -> WeakFreeListHandle<M> {
        // It's a slightly awkward process to insert an element, since we don't know
        // the index of the LRU tracking element until we've got a handle for the
        // underlying cached data.

        // Insert the data provided by the caller
        let handle = self.entries.insert(LRUCacheEntry {
            lru_index: None,
            value,
        });

        // Get a weak handle to return to the caller
        let weak_handle = handle.weak();

        // Add an LRU tracking node that owns the strong handle, and store the location
        // of this inside the cache entry.
        let entry = self.entries.get_mut(&handle);
        entry.lru_index = Some(self.lru.push_new(handle));

        weak_handle
    }

    /// Get immutable access to the data at a given slot. Since this takes a strong
    /// handle, it's guaranteed to be valid.
    pub fn get(
        &self,
        handle: &FreeListHandle<M>,
    ) -> &T {
        &self.entries
            .get(handle)
            .value
    }

    /// Get immutable access to the data at a given slot. Since this takes a weak
    /// handle, it may have been evicted, so returns an Option.
    pub fn get_opt(
        &self,
        handle: &WeakFreeListHandle<M>,
    ) -> Option<&T> {
        self.entries
            .get_opt(handle)
            .map(|entry| {
                &entry.value
            })
    }

    /// Get mutable access to the data at a given slot. Since this takes a weak
    /// handle, it may have been evicted, so returns an Option.
    pub fn get_opt_mut(
        &mut self,
        handle: &WeakFreeListHandle<M>,
    ) -> Option<&mut T> {
        self.entries
            .get_opt_mut(handle)
            .map(|entry| {
                &mut entry.value
            })
    }

    /// Remove the oldest item from the cache. This is used to select elements to
    /// be evicted. If the cache is empty, or all elements in the cache have manual
    /// eviction enabled, this will return None
    pub fn pop_oldest(
        &mut self,
    ) -> Option<T> {
        self.lru
            .pop_front()
            .map(|handle| {
                let entry = self.entries.free(handle);
                // We should only find elements in this list with valid LRU location
                debug_assert!(entry.lru_index.is_some());
                entry.value
            })
    }

    /// This is a special case of `push_new`, which is a requirement for the texture
    /// cache. Sometimes, we want to replace the content of an existing handle if it
    /// exists, or insert a new element if the handle is invalid (for example, if an
    /// image is resized and it moves to a new location in the texture atlas). This
    /// method returns the old cache entry if it existed, so it can be freed by the caller.
    #[must_use]
    pub fn replace_or_insert(
        &mut self,
        handle: &mut WeakFreeListHandle<M>,
        data: T,
    ) -> Option<T> {
        match self.entries.get_opt_mut(handle) {
            Some(entry) => {
                Some(mem::replace(&mut entry.value, data))
            }
            None => {
                *handle = self.push_new(data);
                None
            }
        }
    }

    /// This is used by the calling code to signal that the element that this handle
    /// references has been used on this frame. Internally, it updates the links in
    /// the LRU tracking element to move this item to the end of the LRU list. Returns
    /// the underlying data in case the client wants to mutate it.
    pub fn touch(
        &mut self,
        handle: &WeakFreeListHandle<M>
    ) -> Option<&mut T> {
        let lru = &mut self.lru;

        self.entries
            .get_opt_mut(handle)
            .map(|entry| {
                // Only have a valid LRU index if eviction mode is auto
                if let Some(lru_index) = entry.lru_index {
                    lru.mark_used(lru_index);
                }

                &mut entry.value
            })
    }

    /// In some special cases, the caller may want to manually manage the
    /// lifetime of a resource. This method removes the LRU tracking information
    /// for an element, and returns the strong handle to the caller to manage.
    #[must_use]
    pub fn set_manual_eviction(
        &mut self,
        handle: &WeakFreeListHandle<M>,
    ) -> Option<FreeListHandle<M>> {
        let entry = self.entries
            .get_opt_mut(handle)
            .expect("bug: trying to set manual eviction on an invalid handle");

        // Remove the LRU tracking information from this element, if it exists.
        // (it may be None if manual eviction was already enabled for this element).
        entry.lru_index.take().map(|lru_index| {
            self.lru.remove(lru_index)
        })
    }

    /// Remove an element that is in manual eviction mode. This takes the caller
    /// managed strong handle, and removes this element from the freelist.
    pub fn remove_manual_handle(
        &mut self,
        handle: FreeListHandle<M>,
    ) -> T {
        let entry = self.entries.free(handle);
        debug_assert_eq!(entry.lru_index, None, "Must be manual eviction mode!");
        entry.value
    }

    /// Try to validate that the state of the cache is consistent
    #[cfg(test)]
    fn validate(&self) {
        self.lru.validate();
    }
}

/// Index of an LRU tracking element
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct ItemIndex(num::NonZeroU32);

impl ItemIndex {
    fn as_usize(&self) -> usize {
        self.0.get() as usize
    }
}

/// Stores a strong handle controlling the lifetime of the data in the LRU
/// cache, and a doubly-linked list node specifying where in the current LRU
/// order this element exists. These items are themselves backed by a freelist
/// to minimize heap allocations and improve cache access patterns.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug)]
struct Item<H> {
    prev: Option<ItemIndex>,
    next: Option<ItemIndex>,
    handle: Option<H>,
}

/// Internal implementation of the LRU tracking list
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
struct LRUTracker<H> {
    /// Current head of the list - this is the oldest item that will be evicted next.
    head: Option<ItemIndex>,
    /// Current tail of the list - this is the most recently used element.
    tail: Option<ItemIndex>,
    /// As tracking items are removed, they are stored in a freelist, to minimize heap allocations
    free_list_head: Option<ItemIndex>,
    /// The freelist that stores all the LRU tracking items
    items: Vec<Item<H>>,
}

impl<H> LRUTracker<H> where H: std::fmt::Debug {
    /// Construct a new LRU tracker
    fn new() -> Self {
        // Push a dummy entry in the vec that is never used. This ensures the NonZeroU32
        // property is respected, and we never create an ItemIndex(0).
        let items = vec![
            Item {
                prev: None,
                next: None,
                handle: None,
            },
        ];

        LRUTracker {
            head: None,
            tail: None,
            free_list_head: None,
            items,
        }
    }

    /// Internal function that takes an item index, and links it to the
    /// end of the tracker list (makes it the newest item).
    fn link_as_new_tail(
        &mut self,
        item_index: ItemIndex,
    ) {
        match (self.head, self.tail) {
            (Some(..), Some(tail)) => {
                // Both a head and a tail
                self.items[item_index.as_usize()].prev = Some(tail);
                self.items[item_index.as_usize()].next = None;

                self.items[tail.as_usize()].next = Some(item_index);
                self.tail = Some(item_index);
            }
            (None, None) => {
                // No head/tail, currently empty list
                self.items[item_index.as_usize()].prev = None;
                self.items[item_index.as_usize()].next = None;

                self.head = Some(item_index);
                self.tail = Some(item_index);
            }
            (Some(..), None) | (None, Some(..)) => {
                // Invalid state
                unreachable!();
            }
        }
    }

    /// Internal function that takes an LRU item index, and removes it from
    /// the current doubly linked list. Used during removal of items, and also
    /// when items are moved to the back of the list as they're touched.
    fn unlink(
        &mut self,
        item_index: ItemIndex,
    ) {
        let (next, prev) = {
            let item = &self.items[item_index.as_usize()];
            (item.next, item.prev)
        };

        match next {
            Some(next) => {
                self.items[next.as_usize()].prev = prev;
            }
            None => {
                debug_assert_eq!(self.tail, Some(item_index));
                self.tail = prev;
            }
        }

        match prev {
            Some(prev) => {
                self.items[prev.as_usize()].next = next;
            }
            None => {
                debug_assert_eq!(self.head, Some(item_index));
                self.head = next;
            }
        }
    }

    /// Push a new LRU tracking item on to the back of the list, marking
    /// it as the most recent item.
    fn push_new(
        &mut self,
        handle: H,
    ) -> ItemIndex {
        // See if there is a slot available in the current free list
        let item_index = match self.free_list_head {
            Some(index) => {
                // Reuse an existing slot
                let item = &mut self.items[index.as_usize()];

                assert!(item.handle.is_none());
                item.handle = Some(handle);

                self.free_list_head = item.next;

                index
            }
            None => {
                // No free slots available, push to the end of the array
                let index = ItemIndex(num::NonZeroU32::new(self.items.len() as u32).unwrap());

                self.items.push(Item {
                    prev: None,
                    next: None,
                    handle: Some(handle),
                });

                index
            }
        };

        // Now link this element into the LRU list
        self.link_as_new_tail(item_index);

        item_index
    }

    /// Remove the oldest element from the front of the LRU list. Returns None
    /// if the list is empty.
    fn pop_front(
        &mut self,
    ) -> Option<H> {
        let handle = match (self.head, self.tail) {
            (Some(head), Some(tail)) => {
                let item_index = head;

                // Head and tail are the same - removing the only element
                if head == tail {
                    self.head = None;
                    self.tail = None;
                } else {
                    // Update the head of the list, popping the first element off
                    let new_head = self.items[head.as_usize()].next.unwrap();
                    self.head = Some(new_head);
                    self.items[new_head.as_usize()].prev = None;
                }

                // Add this item to the freelist for later use
                self.items[item_index.as_usize()].next = self.free_list_head;
                self.free_list_head = Some(item_index);

                // Return the handle to the user
                Some(self.items[item_index.as_usize()].handle.take().unwrap())
            }
            (None, None) => {
                // List is empty
                None
            }
            (Some(..), None) | (None, Some(..)) => {
                // Invalid state
                unreachable!();
            }
        };

        handle
    }

    /// Manually remove an item from the LRU tracking list. This is used
    /// when an element switches from having its lifetime managed by the LRU
    /// algorithm to having a manual eviction policy.
    fn remove(
        &mut self,
        index: ItemIndex,
    ) -> H {
        // Remove from the LRU list
        self.unlink(index);

        let handle = self.items[index.as_usize()].handle.take().unwrap();

        // Add LRU item to the freelist for future use.
        self.items[index.as_usize()].next = self.free_list_head;
        self.free_list_head = Some(index);

        handle
    }

    /// Called to mark that an item was used on this frame. It unlinks the
    /// tracking item, and then re-links it to the back of the list.
    fn mark_used(
        &mut self,
        index: ItemIndex,
    ) {
        self.unlink(index);
        self.link_as_new_tail(index);
    }

    /// Try to validate that the state of the linked lists are consistent
    #[cfg(test)]
    fn validate(&self) {
        use std::collections::HashSet;

        // Must have a valid head/tail or be empty
        assert!((self.head.is_none() && self.tail.is_none()) || (self.head.is_some() && self.tail.is_some()));

        // If there is a head, the prev of the head must be none
        if let Some(head) = self.head {
            assert!(self.items[head.as_usize()].prev.is_none());
        }

        // If there is a tail, the next of the tail must be none
        if let Some(tail) = self.tail {
            assert!(self.items[tail.as_usize()].next.is_none());
        }

        // Collect all free and valid items, both in forwards and reverse order
        let mut free_items = Vec::new();
        let mut free_items_set = HashSet::new();
        let mut valid_items_front = Vec::new();
        let mut valid_items_front_set = HashSet::new();
        let mut valid_items_reverse = Vec::new();
        let mut valid_items_reverse_set = HashSet::new();

        let mut current = self.free_list_head;
        while let Some(index) = current {
            let item = &self.items[index.as_usize()];
            free_items.push(index);
            assert!(free_items_set.insert(index));
            current = item.next;
        }

        current = self.head;
        while let Some(index) = current {
            let item = &self.items[index.as_usize()];
            valid_items_front.push(index);
            assert!(valid_items_front_set.insert(index));
            current = item.next;
        }

        current = self.tail;
        while let Some(index) = current {
            let item = &self.items[index.as_usize()];
            valid_items_reverse.push(index);
            assert!(!valid_items_reverse_set.contains(&index));
            valid_items_reverse_set.insert(index);
            current = item.prev;
        }

        // Ensure set lengths match the vec lengths (should be enforced by the assert check during insert anyway)
        assert_eq!(valid_items_front.len(), valid_items_front_set.len());
        assert_eq!(valid_items_reverse.len(), valid_items_reverse_set.len());

        // Length of the array should equal free + valid items count + 1 (dummy entry)
        assert_eq!(free_items.len() + valid_items_front.len() + 1, self.items.len());

        // Should be same number of items whether iterating forwards or reverse
        assert_eq!(valid_items_front.len(), valid_items_reverse.len());

        // Ensure there are no items considered in the free list that are also in the valid list
        assert!(free_items_set.intersection(&valid_items_reverse_set).collect::<HashSet<_>>().is_empty());
        assert!(free_items_set.intersection(&valid_items_front_set).collect::<HashSet<_>>().is_empty());

        // Should be the same number of items regardless of iteration direction
        assert_eq!(valid_items_front_set.len(), valid_items_reverse_set.len());

        // Ensure that the ordering is exactly the same, regardless of iteration direction
        for (i0, i1) in valid_items_front.iter().zip(valid_items_reverse.iter().rev()) {
            assert_eq!(i0, i1);
        }
    }
}

#[test]
fn test_lru_tracker_push_pop() {
    // Push elements, pop them all off and ensure:
    // - Returned in oldest order
    // - pop_oldest returns None after last element popped
    struct CacheMarker;
    const NUM_ELEMENTS: usize = 50;

    let mut cache: LRUCache<usize, CacheMarker> = LRUCache::new();
    cache.validate();

    for i in 0 .. NUM_ELEMENTS {
        cache.push_new(i);
    }
    cache.validate();

    for i in 0 .. NUM_ELEMENTS {
        assert_eq!(cache.pop_oldest(), Some(i));
    }
    cache.validate();

    assert_eq!(cache.pop_oldest(), None);
}

#[test]
fn test_lru_tracker_push_touch_pop() {
    // Push elements, touch even handles, pop them all off and ensure:
    // - Returned in correct order
    // - pop_oldest returns None after last element popped
    struct CacheMarker;
    const NUM_ELEMENTS: usize = 50;

    let mut cache: LRUCache<usize, CacheMarker> = LRUCache::new();
    let mut handles = Vec::new();
    cache.validate();

    for i in 0 .. NUM_ELEMENTS {
        handles.push(cache.push_new(i));
    }
    cache.validate();

    for i in 0 .. NUM_ELEMENTS/2 {
        cache.touch(&handles[i*2]);
    }
    cache.validate();

    for i in 0 .. NUM_ELEMENTS/2 {
        assert_eq!(cache.pop_oldest(), Some(i*2+1));
    }
    cache.validate();
    for i in 0 .. NUM_ELEMENTS/2 {
        assert_eq!(cache.pop_oldest(), Some(i*2));
    }
    cache.validate();

    assert_eq!(cache.pop_oldest(), None);
}

#[test]
fn test_lru_tracker_push_get() {
    // Push elements, ensure:
    // - get access via weak handles works
    struct CacheMarker;
    const NUM_ELEMENTS: usize = 50;

    let mut cache: LRUCache<usize, CacheMarker> = LRUCache::new();
    let mut handles = Vec::new();
    cache.validate();

    for i in 0 .. NUM_ELEMENTS {
        handles.push(cache.push_new(i));
    }
    cache.validate();

    for i in 0 .. NUM_ELEMENTS/2 {
        assert!(cache.get_opt(&handles[i]) == Some(&i));
    }
    cache.validate();
}

#[test]
fn test_lru_tracker_push_replace_get() {
    // Push elements, replace contents, ensure:
    // - each element was replaced with new data correctly
    // - replace_or_insert works for invalid handles
    struct CacheMarker;
    const NUM_ELEMENTS: usize = 50;

    let mut cache: LRUCache<usize, CacheMarker> = LRUCache::new();
    let mut handles = Vec::new();
    cache.validate();

    for i in 0 .. NUM_ELEMENTS {
        handles.push(cache.push_new(i));
    }
    cache.validate();

    for i in 0 .. NUM_ELEMENTS {
        assert_eq!(cache.replace_or_insert(&mut handles[i], i * 2), Some(i));
    }
    cache.validate();

    for i in 0 .. NUM_ELEMENTS/2 {
        assert!(cache.get_opt(&handles[i]) == Some(&(i * 2)));
    }
    cache.validate();

    let mut empty_handle = WeakFreeListHandle::invalid();
    assert_eq!(cache.replace_or_insert(&mut empty_handle, 100), None);
    assert_eq!(cache.get_opt(&empty_handle), Some(&100));
}

#[test]
fn test_lru_tracker_manual_evict() {
    // Push elements, set even as manual eviction, ensure:
    // - correctly pop auto handles in correct order
    // - correctly remove manual handles, and have expected value
    struct CacheMarker;
    const NUM_ELEMENTS: usize = 50;

    let mut cache: LRUCache<usize, CacheMarker> = LRUCache::new();
    let mut handles = Vec::new();
    let mut manual_handles = Vec::new();
    cache.validate();

    for i in 0 .. NUM_ELEMENTS {
        handles.push(cache.push_new(i));
    }
    cache.validate();

    for i in 0 .. NUM_ELEMENTS/2 {
        manual_handles.push(cache.set_manual_eviction(&handles[i*2]).unwrap());
    }
    cache.validate();

    for i in 0 .. NUM_ELEMENTS/2 {
        assert!(cache.pop_oldest() == Some(i*2 + 1));
    }
    cache.validate();

    assert!(cache.pop_oldest().is_none());

    for (i, manual_handle) in manual_handles.drain(..).enumerate() {
        assert_eq!(*cache.get(&manual_handle), i*2);
        assert_eq!(cache.remove_manual_handle(manual_handle), i*2);
    }
}
