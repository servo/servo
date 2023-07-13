/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A generic backing store for caches.
//!
//! `FreeList` is a simple vector-backed data structure where each entry in the
//! vector contains an Option<T>. It maintains an index-based (rather than
//! pointer-based) free list to efficiently locate the next unused entry. If all
//! entries are occupied, insertion appends a new element to the vector.
//!
//! It also supports both strong and weak handle semantics. There is exactly one
//! (non-Clonable) strong handle per occupied entry, which must be passed by
//! value into `free()` to release an entry. Strong handles can produce an
//! unlimited number of (Clonable) weak handles, which are used to perform
//! lookups which may fail of the entry has been freed. A per-entry epoch ensures
//! that weak handle lookups properly fail even if the entry has been freed and
//! reused.
//!
//! TODO(gw): Add an occupied list head, for fast iteration of the occupied list
//! to implement retain() style functionality.

use std::{fmt, u32};
use std::marker::PhantomData;

#[derive(Debug, Copy, Clone, MallocSizeOf, PartialEq)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
struct Epoch(u32);

impl Epoch {
    /// Mints a new epoch.
    ///
    /// We start at 1 so that 0 is always an invalid epoch.
    fn new() -> Self {
        Epoch(1)
    }

    /// Returns an always-invalid epoch.
    fn invalid() -> Self {
        Epoch(0)
    }
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(MallocSizeOf)]
pub struct FreeListHandle<M> {
    index: u32,
    epoch: Epoch,
    _marker: PhantomData<M>,
}

/// More-compact textual representation for debug logging.
impl<M> fmt::Debug for FreeListHandle<M> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("StrongHandle")
            .field("index", &self.index)
            .field("epoch", &self.epoch.0)
            .finish()
    }
}

impl<M> FreeListHandle<M> {
    pub fn weak(&self) -> WeakFreeListHandle<M> {
        WeakFreeListHandle {
            index: self.index,
            epoch: self.epoch,
            _marker: PhantomData,
        }
    }

    pub fn invalid() -> Self {
        Self {
            index: 0,
            epoch: Epoch::invalid(),
            _marker: PhantomData,
        }
    }

    /// Returns true if this handle and the supplied weak handle reference
    /// the same underlying location in the freelist.
    pub fn matches(&self, weak_handle: &WeakFreeListHandle<M>) -> bool {
        self.index == weak_handle.index &&
        self.epoch == weak_handle.epoch
    }
}

impl<M> Clone for WeakFreeListHandle<M> {
    fn clone(&self) -> Self {
        WeakFreeListHandle {
            index: self.index,
            epoch: self.epoch,
            _marker: PhantomData,
        }
    }
}

impl<M> PartialEq for WeakFreeListHandle<M> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.epoch == other.epoch
    }
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(MallocSizeOf)]
pub struct WeakFreeListHandle<M> {
    index: u32,
    epoch: Epoch,
    _marker: PhantomData<M>,
}

/// More-compact textual representation for debug logging.
impl<M> fmt::Debug for WeakFreeListHandle<M> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("WeakHandle")
            .field("index", &self.index)
            .field("epoch", &self.epoch.0)
            .finish()
    }
}

impl<M> WeakFreeListHandle<M> {
    /// Returns an always-invalid handle.
    pub fn invalid() -> Self {
        Self {
            index: 0,
            epoch: Epoch::invalid(),
            _marker: PhantomData,
        }
    }
}

#[derive(Debug, MallocSizeOf)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
struct Slot<T> {
    next: Option<u32>,
    epoch: Epoch,
    value: Option<T>,
}

#[derive(Debug, MallocSizeOf)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct FreeList<T, M> {
    slots: Vec<Slot<T>>,
    free_list_head: Option<u32>,
    active_count: usize,
    _marker: PhantomData<M>,
}

impl<T, M> FreeList<T, M> {
    /// Mints a new `FreeList` with no entries.
    ///
    /// Triggers a 1-entry allocation.
    pub fn new() -> Self {
        // We guarantee that we never have zero entries by starting with one
        // free entry. This allows WeakFreeListHandle::invalid() to work
        // without adding any additional branches.
        let first_slot = Slot {
            next: None,
            epoch: Epoch::new(),
            value: None,
        };
        FreeList {
            slots: vec![first_slot],
            free_list_head: Some(0),
            active_count: 0,
            _marker: PhantomData,
        }
    }

    pub fn clear(&mut self) {
        self.slots.truncate(1);
        self.slots[0] = Slot {
            next: None,
            epoch: Epoch::new(),
            value: None,
        };
        self.free_list_head = Some(0);
        self.active_count = 0;
    }

    #[allow(dead_code)]
    pub fn get(&self, id: &FreeListHandle<M>) -> &T {
        self.slots[id.index as usize].value.as_ref().unwrap()
    }

    #[allow(dead_code)]
    pub fn get_mut(&mut self, id: &FreeListHandle<M>) -> &mut T {
        self.slots[id.index as usize].value.as_mut().unwrap()
    }

    pub fn get_opt(&self, id: &WeakFreeListHandle<M>) -> Option<&T> {
        let slot = &self.slots[id.index as usize];
        if slot.epoch == id.epoch {
            slot.value.as_ref()
        } else {
            None
        }
    }

    pub fn get_opt_mut(&mut self, id: &WeakFreeListHandle<M>) -> Option<&mut T> {
        let slot = &mut self.slots[id.index as usize];
        if slot.epoch == id.epoch {
            slot.value.as_mut()
        } else {
            None
        }
    }

    pub fn insert(&mut self, item: T) -> FreeListHandle<M> {
        self.active_count += 1;

        match self.free_list_head {
            Some(free_index) => {
                let slot = &mut self.slots[free_index as usize];

                // Remove from free list.
                self.free_list_head = slot.next;
                slot.next = None;
                slot.value = Some(item);

                FreeListHandle {
                    index: free_index,
                    epoch: slot.epoch,
                    _marker: PhantomData,
                }
            }
            None => {
                let index = self.slots.len() as u32;
                let epoch = Epoch::new();

                self.slots.push(Slot {
                    next: None,
                    epoch,
                    value: Some(item),
                });

                FreeListHandle {
                    index,
                    epoch,
                    _marker: PhantomData,
                }
            }
        }
    }

    pub fn free(&mut self, id: FreeListHandle<M>) -> T {
        self.active_count -= 1;
        let slot = &mut self.slots[id.index as usize];
        slot.next = self.free_list_head;
        slot.epoch = Epoch(slot.epoch.0 + 1);
        self.free_list_head = Some(id.index);
        slot.value.take().unwrap()
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.active_count
    }
}
