/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A Concurrent Generational Typed Arena.
//!
//! This arena is designed to support fast, concurrent allocations and lookups.
//! Allocations are represented by tokens, which include a generation number.
//! When the arena is reset (an operation only allowed in exclusive mode), the
//! generation of the arena is incremented and subsequent lookups for previously-
//! allocated tokens will fail.
//!
//! The arena is implemented with a two-tiered allocation hierarchy: `ChunkList`s
//! contain pointers to lazily-allocated `Chunk`s, which are arrays of `Item`s
//! (wrapped in `AtomicRefCell`s). `ChunkList`s also contain a pointer to the next
//! `ChunkList`. The first `ChunkList` is allocated inline in the `Arena` struct, and
//! subsequent `ChunkList`s are lazily allocated. A given `ChunkList` (indirectly)
//! contains CHUNKS_PER_LIST * ITEMS_PER_CHUNK items, and we try to make this
//! number large enough so that most workloads should need little to no pointer
//! chasing.
//!
//! The lockless coherency scheme is based on the following two properties:
//!   (1) Lazy allocation of buffers (`Chunk`s or `ChunkList`s) is the responsibility
//!       of whichever thread allocates the first item (i.e. performs the fetch_add
//!       that results in an index mapping to the first slot) in the new buffer.
//!       Other threads which concurrently allocate subsequent items will spin until
//!       the buffer appears.
//!   (2) Once a buffer is allocated, it can only be deallocated when the Arena is
//!       in exclusive mode, which means that no thread can simultaneously attempt
//!       to allocate or dereference tokens.

#![allow(unsafe_code)]

use atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};
use owning_handle::OwningHandle;
use owning_ref::{ArcRef, OwningRef};
use std::default::Default;
use std::mem;
use std::ptr;
use std::sync::{Arc, Weak};
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

/// Core arena structure. Callers always access the arena using an `ArenaArc`,
/// which provides |Sync| refcounted access with shared/exclusive borrow semantics.
pub struct Arena<Item: Default> {
    generation: u32,
    current_index: AtomicUsize,
    chunk_list: ChunkList<Item>,
    self_weak: ArenaWeak<Item>,
}

pub type ArenaArc<Item> = Arc<AtomicRefCell<Arena<Item>>>;
type ArenaWeak<Item> = Weak<AtomicRefCell<Arena<Item>>>;

impl<Item: Default> Arena<Item> {
    /// Creates a new arena.
    pub fn new() -> ArenaArc<Item> {
        let result = Arc::new(AtomicRefCell::new(Arena {
            generation: 1,
            current_index: AtomicUsize::new(0),
            chunk_list: ChunkList::new(),
            self_weak: Weak::new(),
        }));

        // Add a weak edge from the Arena to the containing Arc to facilitate
        // Token::bind().
        result.borrow_mut().self_weak = Arc::downgrade(&result);

        result
    }

    /// Allocates a `Token`.
    pub fn allocate(&self) -> Token<Item> {
        let data = self.allocate_raw();
        let arena = self.self_weak.upgrade().unwrap();
        unsafe { Token::create(data, arena).unwrap() }
    }

    /// Allocates a token directly as a `TokenData`, which saves a few borrow
    /// operations for callers that don't need to dereference the result.
    pub fn allocate_raw(&self) -> TokenData {
        let idx_usize = self.current_index.fetch_add(1, Ordering::Relaxed);
        if idx_usize >= (1 << 31) {
            panic!("Arena wrapped (can't handle more than 2^31 entries)");
        }
        let idx = idx_usize as u32;

        self.chunk_list.allocate(idx as usize);
        TokenData {
            generation: self.generation,
            index: idx,
        }
    }

    /// Not publicly exposed, invoked internally by `Token`. The caller must
    /// ensure that the `TokenData` does not represent an expired `Token`.
    fn lookup<'a>(&'a self, data: TokenData) -> &'a AtomicRefCell<Item> {
        debug_assert!(!data.is_expired(self));
        self.chunk_list.lookup(data.index as usize)
    }

    /// Resets the arena, deallocating all items and incrementing the generation.
    pub fn reset(&mut self) {
        if self.current_index.load(Ordering::Relaxed) == 0 {
            // No allocations were made this cycle, we can keep the old generation.
            return;
        }

        self.chunk_list = ChunkList::new();
        self.current_index.store(0, Ordering::Relaxed);
        self.generation = self.generation.checked_add(1).expect("generation overflow");
    }
}

// These parameters have not been measured, and could be tuned.
#[cfg(not(debug_assertions))]
const ITEMS_PER_CHUNK: usize = 64;
#[cfg(not(debug_assertions))]
const CHUNKS_PER_LIST: usize = 256;

// Make the chunks and lists smaller in debug builds to exercise the atomic
// allocation logic more often.
#[cfg(debug_assertions)]
const ITEMS_PER_CHUNK: usize = 4;
#[cfg(debug_assertions)]
const CHUNKS_PER_LIST: usize = 32;

// 16384 in release builds, 128 in debug builds.
pub const ITEMS_PER_LIST: usize = ITEMS_PER_CHUNK * CHUNKS_PER_LIST;

// I can't believe there isn't a better way to do this. :-(
macro_rules! make_array {
    ($Item: ty, $size: expr) => {
        unsafe {
            let mut arr: [$Item; $size] = mem::uninitialized();
            for (_, slot) in arr.iter_mut().enumerate() {
                let item = <$Item>::default();
                ptr::write(slot, item);
            }
            arr
        }
    }
}

struct Chunk<Item: Default> {
    pub items: [AtomicRefCell<Item>; ITEMS_PER_CHUNK],
}

impl<Item: Default> Chunk<Item> {
    fn new() -> Self {
        Chunk {
            items: make_array!(AtomicRefCell<Item>, ITEMS_PER_CHUNK)
        }
    }
}

struct ChunkList<Item: Default> {
    chunks: [AtomicPtr<Chunk<Item>>; CHUNKS_PER_LIST],
    next_list: AtomicPtr<ChunkList<Item>>,
}

impl<Item: Default> Drop for ChunkList<Item> {
    fn drop(&mut self) {
        // Drop any chunks.
        for chunk in self.chunks.iter() {
            let chunk_ptr = chunk.load(Ordering::Relaxed);
            if !chunk_ptr.is_null() {
                let _ = unsafe { Box::from_raw(chunk_ptr) };
            }
        }

        // Drop any subsequent lists.
        let next = self.next_list.load(Ordering::Relaxed);
        if !next.is_null() {
            let _ = unsafe { Box::from_raw(next) };
        }
    }
}

impl<Item: Default> ChunkList<Item> {
    fn new() -> Self {
        ChunkList {
            chunks: make_array!(AtomicPtr<Chunk<Item>>, CHUNKS_PER_LIST),
            next_list: AtomicPtr::new(ptr::null_mut()),
        }
    }

    fn allocate(&self, idx: usize) {
        if idx == ITEMS_PER_LIST {
            // Our item is the first item in the first chunk of the next list,
            // which means it's our job to synthesize that list.
            let new_list = Box::new(ChunkList::<Item>::new());
            debug_assert!(self.next_list.load(Ordering::Acquire).is_null());
            let new_list_ptr = Box::into_raw(new_list);
            self.next_list.store(new_list_ptr, Ordering::Release);
            unsafe { &*new_list_ptr }.allocate(0);
        } else if idx > ITEMS_PER_LIST {
            // Our item is in a subsequent list. Delegate to it, spinning if it
            // hasn't been created yet.
            loop {
                let list = self.next_list.load(Ordering::Acquire);
                if !list.is_null() {
                    unsafe { (*list).allocate(idx - ITEMS_PER_LIST) };
                    break;
                }
            }
        } else {
            // The item is in this list.
            let chunk_idx = idx / ITEMS_PER_CHUNK;
            let idx_in_chunk = idx % ITEMS_PER_CHUNK;
            if idx_in_chunk == 0 {
                // Our item is the first item in the next chunk.
                // That means it's our job to synthesize the next list.
                let chunk = Box::new(Chunk::<Item>::new());
                debug_assert!(self.chunks[chunk_idx].load(Ordering::Acquire).is_null());
                self.chunks[chunk_idx].store(Box::into_raw(chunk), Ordering::Release);
            } else {
                // The chunk should be allocated by someone else. Just make sure that
                // has happened, spinning as necessary.
                loop {
                    if !self.chunks[chunk_idx].load(Ordering::Acquire).is_null() {
                        break;
                    }
                }
            }
        }
    }

    // Note: We could probably get away with Relaxed loads here because
    // allocate() guarantees that the allocated data is observable on our thread,
    // and it seems unlikely that the caller would be able to send the Token to
    // another thread without some kind of SeqCst fence. But the benefit seems
    // pretty marginal (since x86 is Acquire/Release by default), and the prospect
    // of extremely rare ARM-only allocation races is pretty terrifying. So we just
    // use the same atomic semantics at all callsites for the AtomicPtrs.
    fn lookup(&self, idx: usize) -> &AtomicRefCell<Item> {
        if idx >= ITEMS_PER_LIST {
            // The item is in a subsequent list - traverse.
            let list = self.next_list.load(Ordering::Acquire);
            debug_assert!(!list.is_null());
            return unsafe { (*list).lookup(idx - ITEMS_PER_LIST) };
        }

        // The item is in this list.
        let chunk = idx / ITEMS_PER_CHUNK;
        let chunk_idx = idx % ITEMS_PER_CHUNK;
        let chunk_ptr = self.chunks[chunk].load(Ordering::Acquire);
        debug_assert!(!chunk_ptr.is_null());
        unsafe { &(*chunk_ptr).items[chunk_idx] }
    }
}

/// `TokenData` is the internal data storage for an address within an `Arena`.
/// It can be serialized to and from a `u64`, and used to construct a `Token`.
#[derive(Clone, Copy, Debug)]
pub struct TokenData {
    generation: u32,
    index: u32,
}

const LOW_MASK: u64 = 0xffffffff_u64;
const HIGH_MASK: u64 = LOW_MASK << 32;

impl TokenData {
    /// Serializes the `TokenData` into a u64. The resulting value will always have
    /// its low bit unset, which allows consumers to store one bit of additional
    /// information in the serialization if they wish.
    ///
    /// If needed, this mechanism could be extended to allow consumers to allocate
    /// an arbitrary number of user bits at the expense of reduced address space.
    pub fn serialize(&self) -> u64 {
        // Shift the generation into the high 32 bits.
        let shifted_generation = (self.generation as u64) << 32;

        // Shift the index left one bit to leave the bottom bit unused. This
        // allows the callers to use tokens as tagged pointers.
        debug_assert!(self.index & (1_u32 << 31) == 0);
        let shifted_index = (self.index << 1) as u64;

        shifted_generation | shifted_index
    }

    /// Deserializes a TokenData from a `u64`.
    pub fn deserialize(x: u64) -> Self {
        debug_assert!(x & 1 == 0, "Caller should strip the tag bit");
        TokenData {
            generation: ((x & HIGH_MASK) >> 32) as u32,
            index: (x & LOW_MASK) as u32,
        }
    }

    /// Returns whether this `TokenData` has expired. For the result to be
    /// meaningful, the `arena` argument must reference the same `Arena`
    /// from which this `TokenData` was allocated.
    pub fn is_expired<T: Default>(&self, arena: &Arena<T>) -> bool {
        let expired = self.generation != arena.generation;
        debug_assert!(!expired || self.generation < arena.generation);
        expired
    }

    /// Creates a `TokenData` that can represent a valid, expired token in any
    /// `Arena`.
    pub fn empty() -> Self {
        TokenData {
            generation: 0,
            index: 0,
        }
    }
}

type ArenaRef<Item> = ArcRef<AtomicRefCell<Arena<Item>>>;
type ArenaHandle<Item> = OwningHandle<ArenaRef<Item>, AtomicRef<'static, Arena<Item>>>;
type ItemRef<Item> = OwningRef<ArenaHandle<Item>, AtomicRefCell<Item>>;

/// A `Token` represents an allocated `Arena` item.
pub struct Token<Item: Default + 'static> {
    item: ItemRef<Item>,
    data: TokenData,
}

impl<Item: Default + 'static> Token<Item> {
    /// Immutably borrows the `Item` referenced by this `Token`.
    pub fn borrow(&self) -> AtomicRef<Item> {
        self.item.borrow()
    }

    /// Mutably borrows the `Item` referenced by this `Token`.
    pub fn borrow_mut(&self) -> AtomicRefMut<Item> {
        self.item.borrow_mut()
    }

    /// Access the `TokenData` backing this `Token`.
    pub fn data(&self) -> TokenData {
        self.data
    }

    /// Creates a `Token` from a `TokenData`. This is `unsafe` because the
    /// caller must ensure that the provided `TokenData` was obtained from the
    /// same `Arena` (with the exception of the `TokenData::empty()` sentinel,
    /// which can be used with any `Arena`).
    pub unsafe fn create(data: TokenData, arena: ArenaArc<Item>) -> Option<Self> {
        // Grab a handle to the arena.
        let arena_ref = ArenaRef::new(arena);
        let arena_handle = ArenaHandle::new(arena_ref, |x| x.as_ref().unwrap().borrow());

        // Check if the token is expired.
        if data.is_expired(&*arena_handle) {
            return None;
        }

        // Grab an owning handle to the item.
        let item_ref: ItemRef<Item> =
            OwningRef::new(arena_handle).map(|a| a.lookup(data));

        // Synthesize the token.
        Some(Token {
            item: item_ref,
            data: data,
        })
    }
}
