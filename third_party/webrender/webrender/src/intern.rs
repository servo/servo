/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The interning module provides a generic data structure
//! interning container. It is similar in concept to a
//! traditional string interning container, but it is
//! specialized to the WR thread model.
//!
//! There is an Interner structure, that lives in the
//! scene builder thread, and a DataStore structure
//! that lives in the frame builder thread.
//!
//! Hashing, interning and handle creation is done by
//! the interner structure during scene building.
//!
//! Delta changes for the interner are pushed during
//! a transaction to the frame builder. The frame builder
//! is then able to access the content of the interned
//! handles quickly, via array indexing.
//!
//! Epoch tracking ensures that the garbage collection
//! step which the interner uses to remove items is
//! only invoked on items that the frame builder thread
//! is no longer referencing.
//!
//! Items in the data store are stored in a traditional
//! free-list structure, for content access and memory
//! usage efficiency.
//!
//! The epoch is incremented each time a scene is
//! built. The most recently used scene epoch is
//! stored inside each handle. This is then used for
//! cache invalidation.

use crate::internal_types::FastHashMap;
use malloc_size_of::MallocSizeOf;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use std::{ops, u64};
use crate::util::VecHelper;
use crate::profiler::TransactionProfile;

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, Copy, Clone, Hash, MallocSizeOf, PartialEq, Eq)]
struct Epoch(u32);

/// A list of updates to be applied to the data store,
/// provided by the interning structure.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(MallocSizeOf)]
pub struct UpdateList<S> {
    /// Items to insert.
    pub insertions: Vec<Insertion<S>>,

    /// Items to remove.
    pub removals: Vec<Removal>,
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(MallocSizeOf)]
pub struct Insertion<S> {
    pub index: usize,
    pub uid: ItemUid,
    pub value: S,
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(MallocSizeOf)]
pub struct Removal {
    pub index: usize,
    pub uid: ItemUid,
}

impl<S> UpdateList<S> {
    fn new() -> UpdateList<S> {
        UpdateList {
            insertions: Vec::new(),
            removals: Vec::new(),
        }
    }

    fn take_and_preallocate(&mut self) -> UpdateList<S> {
        UpdateList {
            insertions: self.insertions.take_and_preallocate(),
            removals: self.removals.take_and_preallocate(),
        }
    }
}

/// A globally, unique identifier
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, Copy, Clone, Eq, Hash, MallocSizeOf, PartialEq)]
pub struct ItemUid {
    uid: u64,
}

impl ItemUid {
    // Intended for debug usage only
    pub fn get_uid(&self) -> u64 {
        self.uid
    }
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, Hash, MallocSizeOf, PartialEq, Eq)]
pub struct Handle<I> {
    index: u32,
    epoch: Epoch,
    _marker: PhantomData<I>,
}

impl<I> Clone for Handle<I> {
    fn clone(&self) -> Self {
        Handle {
            index: self.index,
            epoch: self.epoch,
            _marker: self._marker,
        }
    }
}

impl<I> Copy for Handle<I> {}

impl<I> Handle<I> {
    pub fn uid(&self) -> ItemUid {
        ItemUid {
            // The index in the freelist + the epoch it was interned generates a stable
            // unique id for an interned element.
            uid: ((self.index as u64) << 32) | self.epoch.0 as u64
        }
    }
}

pub trait InternDebug {
    fn on_interned(&self, _uid: ItemUid) {}
}

/// The data store lives in the frame builder thread. It
/// contains a free-list of items for fast access.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(MallocSizeOf)]
pub struct DataStore<I: Internable> {
    items: Vec<Option<I::StoreData>>,
}

impl<I: Internable> Default for DataStore<I> {
    fn default() -> Self {
        DataStore {
            items: Vec::new(),
        }
    }
}

impl<I: Internable> DataStore<I> {
    /// Apply any updates from the scene builder thread to
    /// this data store.
    pub fn apply_updates(
        &mut self,
        update_list: UpdateList<I::Key>,
        profile: &mut TransactionProfile,
    ) {
        for insertion in update_list.insertions {
            self.items
                .entry(insertion.index)
                .set(Some(insertion.value.into()));
        }

        for removal in update_list.removals {
            self.items[removal.index] = None;
        }

        profile.set(I::PROFILE_COUNTER, self.items.len());
    }
}

/// Retrieve an item from the store via handle
impl<I: Internable> ops::Index<Handle<I>> for DataStore<I> {
    type Output = I::StoreData;
    fn index(&self, handle: Handle<I>) -> &I::StoreData {
        self.items[handle.index as usize].as_ref().expect("Bad datastore lookup")
    }
}

/// Retrieve a mutable item from the store via handle
/// Retrieve an item from the store via handle
impl<I: Internable> ops::IndexMut<Handle<I>> for DataStore<I> {
    fn index_mut(&mut self, handle: Handle<I>) -> &mut I::StoreData {
        self.items[handle.index as usize].as_mut().expect("Bad datastore lookup")
    }
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(MallocSizeOf)]
struct ItemDetails<I> {
    /// Frame that this element was first interned
    interned_epoch: Epoch,
    /// Last frame this element was referenced (used to GC intern items)
    last_used_epoch: Epoch,
    /// Index into the freelist this item is located
    index: usize,
    /// Type marker for create_handle method
    _marker: PhantomData<I>,
}

impl<I> ItemDetails<I> {
    /// Construct a stable handle value from the item details
    fn create_handle(&self) -> Handle<I> {
        Handle {
            index: self.index as u32,
            epoch: self.interned_epoch,
            _marker: PhantomData,
        }
    }
}

/// The main interning data structure. This lives in the
/// scene builder thread, and handles hashing and interning
/// unique data structures. It also manages a free-list for
/// the items in the data store, which is synchronized via
/// an update list of additions / removals.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(MallocSizeOf)]
pub struct Interner<I: Internable> {
    /// Uniquely map an interning key to a handle
    map: FastHashMap<I::Key, ItemDetails<I>>,
    /// List of free slots in the data store for re-use.
    free_list: Vec<usize>,
    /// Pending list of updates that need to be applied.
    update_list: UpdateList<I::Key>,
    /// The current epoch for the interner.
    current_epoch: Epoch,
    /// The information associated with each interned
    /// item that can be accessed by the interner.
    local_data: Vec<I::InternData>,
}

impl<I: Internable> Default for Interner<I> {
    fn default() -> Self {
        Interner {
            map: FastHashMap::default(),
            free_list: Vec::new(),
            update_list: UpdateList::new(),
            current_epoch: Epoch(1),
            local_data: Vec::new(),
        }
    }
}

impl<I: Internable> Interner<I> {
    /// Intern a data structure, and return a handle to
    /// that data. The handle can then be stored in the
    /// frame builder, and safely accessed via the data
    /// store that lives in the frame builder thread.
    /// The provided closure is invoked to build the
    /// local data about an interned structure if the
    /// key isn't already interned.
    pub fn intern<F>(
        &mut self,
        data: &I::Key,
        fun: F,
    ) -> Handle<I> where F: FnOnce() -> I::InternData {
        // Use get_mut rather than entry here to avoid
        // cloning the (sometimes large) key in the common
        // case, where the data already exists in the interner.
        if let Some(details) = self.map.get_mut(data) {
            // Update the last referenced frame for this element
            details.last_used_epoch = self.current_epoch;
            // Return a stable handle value for dependency checking
            return details.create_handle();
        }

        // We need to intern a new data item. First, find out
        // if there is a spare slot in the free-list that we
        // can use. Otherwise, append to the end of the list.
        let index = match self.free_list.pop() {
            Some(index) => index,
            None => self.local_data.len(),
        };

        // Generate a handle for access via the data store.
        let handle = Handle {
            index: index as u32,
            epoch: self.current_epoch,
            _marker: PhantomData,
        };

        let uid = handle.uid();

        // Add a pending update to insert the new data.
        self.update_list.insertions.push(Insertion {
            index,
            uid,
            value: data.clone(),
        });

        #[cfg(debug_assertions)]
        data.on_interned(uid);

        // Store this handle so the next time it is
        // interned, it gets re-used.
        self.map.insert(data.clone(), ItemDetails {
            interned_epoch: self.current_epoch,
            last_used_epoch: self.current_epoch,
            index,
            _marker: PhantomData,
        });

        // Create the local data for this item that is
        // being interned.
        self.local_data.entry(index).set(fun());

        handle
    }

    /// Retrieve the pending list of updates for an interner
    /// that need to be applied to the data store. Also run
    /// a GC step that removes old entries.
    pub fn end_frame_and_get_pending_updates(&mut self) -> UpdateList<I::Key> {
        let mut update_list = self.update_list.take_and_preallocate();

        let free_list = &mut self.free_list;
        let current_epoch = self.current_epoch.0;

        // First, run a GC step. Walk through the handles, and
        // if we find any that haven't been used for some time,
        // remove them. If this ever shows up in profiles, we
        // can make the GC step partial (scan only part of the
        // map each frame). It also might make sense in the
        // future to adjust how long items remain in the cache
        // based on the current size of the list.
        self.map.retain(|_, details| {
            if details.last_used_epoch.0 + 10 < current_epoch {
                // To expire an item:
                //  - Add index to the free-list for re-use.
                //  - Add an update to the data store to invalidate this slot.
                //  - Remove from the hash map.
                free_list.push(details.index);
                update_list.removals.push(Removal {
                    index: details.index,
                    uid: details.create_handle().uid(),
                });
                return false;
            }

            true
        });

        // Begin the next epoch
        self.current_epoch = Epoch(self.current_epoch.0 + 1);

        update_list
    }
}

/// Retrieve the local data for an item from the interner via handle
impl<I: Internable> ops::Index<Handle<I>> for Interner<I> {
    type Output = I::InternData;
    fn index(&self, handle: Handle<I>) -> &I::InternData {
        &self.local_data[handle.index as usize]
    }
}

/// Meta-macro to enumerate the various interner identifiers and types.
///
/// IMPORTANT: Keep this synchronized with the list in mozilla-central located at
/// gfx/webrender_bindings/webrender_ffi.h
///
/// Note that this could be a lot less verbose if concat_idents! were stable. :-(
#[macro_export]
macro_rules! enumerate_interners {
    ($macro_name: ident) => {
        $macro_name! {
            clip: ClipIntern,
            prim: PrimitiveKeyKind,
            normal_border: NormalBorderPrim,
            image_border: ImageBorder,
            image: Image,
            yuv_image: YuvImage,
            line_decoration: LineDecoration,
            linear_grad: LinearGradient,
            radial_grad: RadialGradient,
            conic_grad: ConicGradient,
            picture: Picture,
            text_run: TextRun,
            filter_data: FilterDataIntern,
            backdrop: Backdrop,
            polygon: PolygonIntern,
        }
    }
}

macro_rules! declare_interning_memory_report {
    ( $( $name:ident: $ty:ident, )+ ) => {
        ///
        #[repr(C)]
        #[derive(AddAssign, Clone, Debug, Default)]
        pub struct InternerSubReport {
            $(
                ///
                pub $name: usize,
            )+
        }
    }
}

enumerate_interners!(declare_interning_memory_report);

/// Memory report for interning-related data structures.
/// cbindgen:derive-eq=false
/// cbindgen:derive-ostream=false
#[repr(C)]
#[derive(Clone, Debug, Default)]
pub struct InterningMemoryReport {
    ///
    pub interners: InternerSubReport,
    ///
    pub data_stores: InternerSubReport,
}

impl ::std::ops::AddAssign for InterningMemoryReport {
    fn add_assign(&mut self, other: InterningMemoryReport) {
        self.interners += other.interners;
        self.data_stores += other.data_stores;
    }
}

// The trick to make trait bounds configurable by features.
mod dummy {
    #[cfg(not(feature = "capture"))]
    pub trait Serialize {}
    #[cfg(not(feature = "capture"))]
    impl<T> Serialize for T {}
    #[cfg(not(feature = "replay"))]
    pub trait Deserialize<'a> {}
    #[cfg(not(feature = "replay"))]
    impl<'a, T> Deserialize<'a> for T {}
}
#[cfg(feature = "capture")]
use serde::Serialize as InternSerialize;
#[cfg(not(feature = "capture"))]
use self::dummy::Serialize as InternSerialize;
#[cfg(feature = "replay")]
use serde::Deserialize as InternDeserialize;
#[cfg(not(feature = "replay"))]
use self::dummy::Deserialize as InternDeserialize;

/// Implement `Internable` for a type that wants to participate in interning.
pub trait Internable: MallocSizeOf {
    type Key: Eq + Hash + Clone + Debug + MallocSizeOf + InternDebug + InternSerialize + for<'a> InternDeserialize<'a>;
    type StoreData: From<Self::Key> + MallocSizeOf + InternSerialize + for<'a> InternDeserialize<'a>;
    type InternData: MallocSizeOf + InternSerialize + for<'a> InternDeserialize<'a>;

    // Profile counter indices, see the list in profiler.rs
    const PROFILE_COUNTER: usize;
}
