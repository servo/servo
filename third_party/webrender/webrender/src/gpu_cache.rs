/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Overview of the GPU cache.
//!
//! The main goal of the GPU cache is to allow on-demand
//! allocation and construction of GPU resources for the
//! vertex shaders to consume.
//!
//! Every item that wants to be stored in the GPU cache
//! should create a GpuCacheHandle that is used to refer
//! to a cached GPU resource. Creating a handle is a
//! cheap operation, that does *not* allocate room in the
//! cache.
//!
//! On any frame when that data is required, the caller
//! must request that handle, via ```request```. If the
//! data is not in the cache, the user provided closure
//! will be invoked to build the data.
//!
//! After ```end_frame``` has occurred, callers can
//! use the ```get_address``` API to get the allocated
//! address in the GPU cache of a given resource slot
//! for this frame.

use api::{DebugFlags, DocumentId, PremultipliedColorF};
#[cfg(test)]
use api::IdNamespace;
use api::units::TexelRect;
use euclid::{HomogeneousVector, Rect};
use crate::internal_types::{FastHashMap, FastHashSet};
use crate::profiler::GpuCacheProfileCounters;
use crate::render_backend::{FrameStamp, FrameId};
use crate::renderer::MAX_VERTEX_TEXTURE_WIDTH;
use std::{mem, u16, u32};
use std::num::NonZeroU32;
use std::ops::Add;
use std::time::{Duration, Instant};


/// At the time of this writing, Firefox uses about 15 GPU cache rows on
/// startup, and then gradually works its way up to the mid-30s with normal
/// browsing.
pub const GPU_CACHE_INITIAL_HEIGHT: i32 = 20;
const NEW_ROWS_PER_RESIZE: i32 = 10;

/// The number of frames an entry can go unused before being evicted.
const FRAMES_BEFORE_EVICTION: usize = 10;

/// The ratio of utilized blocks to total blocks for which we start the clock
/// on reclaiming memory.
const RECLAIM_THRESHOLD: f32 = 0.2;

/// The amount of time utilization must be below the above threshold before we
/// blow away the cache and rebuild it.
const RECLAIM_DELAY_S: u64 = 5;

#[derive(Debug, Copy, Clone, Eq, MallocSizeOf, PartialEq)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
struct Epoch(u32);

impl Epoch {
    fn next(&mut self) {
        *self = Epoch(self.0.wrapping_add(1));
    }
}

#[derive(Debug, Copy, Clone, MallocSizeOf)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
struct CacheLocation {
    block_index: BlockIndex,
    epoch: Epoch,
}

/// A single texel in RGBAF32 texture - 16 bytes.
#[derive(Copy, Clone, Debug, MallocSizeOf)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct GpuBlockData {
    data: [f32; 4],
}

impl GpuBlockData {
    pub const EMPTY: Self = GpuBlockData { data: [0.0; 4] };
}

/// Conversion helpers for GpuBlockData
impl From<PremultipliedColorF> for GpuBlockData {
    fn from(c: PremultipliedColorF) -> Self {
        GpuBlockData {
            data: [c.r, c.g, c.b, c.a],
        }
    }
}

impl From<[f32; 4]> for GpuBlockData {
    fn from(data: [f32; 4]) -> Self {
        GpuBlockData { data }
    }
}

impl<P> From<Rect<f32, P>> for GpuBlockData {
    fn from(r: Rect<f32, P>) -> Self {
        GpuBlockData {
            data: [
                r.origin.x,
                r.origin.y,
                r.size.width,
                r.size.height,
            ],
        }
    }
}

impl<P> From<HomogeneousVector<f32, P>> for GpuBlockData {
    fn from(v: HomogeneousVector<f32, P>) -> Self {
        GpuBlockData {
            data: [
                v.x,
                v.y,
                v.z,
                v.w,
            ],
        }
    }
}

impl From<TexelRect> for GpuBlockData {
    fn from(tr: TexelRect) -> Self {
        GpuBlockData {
            data: [tr.uv0.x, tr.uv0.y, tr.uv1.x, tr.uv1.y],
        }
    }
}


// Any data type that can be stored in the GPU cache should
// implement this trait.
pub trait ToGpuBlocks {
    // Request an arbitrary number of GPU data blocks.
    fn write_gpu_blocks(&self, _: GpuDataRequest);
}

// A handle to a GPU resource.
#[derive(Debug, Copy, Clone, MallocSizeOf)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct GpuCacheHandle {
    location: Option<CacheLocation>,
}

impl GpuCacheHandle {
    pub fn new() -> Self {
        GpuCacheHandle { location: None }
    }
}

// A unique address in the GPU cache. These are uploaded
// as part of the primitive instances, to allow the vertex
// shader to fetch the specific data.
#[derive(Copy, Debug, Clone, MallocSizeOf, Eq, PartialEq)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct GpuCacheAddress {
    pub u: u16,
    pub v: u16,
}

impl GpuCacheAddress {
    fn new(u: usize, v: usize) -> Self {
        GpuCacheAddress {
            u: u as u16,
            v: v as u16,
        }
    }

    pub const INVALID: GpuCacheAddress = GpuCacheAddress {
        u: u16::MAX,
        v: u16::MAX,
    };
}

impl Add<usize> for GpuCacheAddress {
    type Output = GpuCacheAddress;

    fn add(self, other: usize) -> GpuCacheAddress {
        GpuCacheAddress {
            u: self.u + other as u16,
            v: self.v,
        }
    }
}

// An entry in a free-list of blocks in the GPU cache.
#[derive(Debug, MallocSizeOf)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
struct Block {
    // The location in the cache of this block.
    address: GpuCacheAddress,
    // The current epoch (generation) of this block.
    epoch: Epoch,
    // Index of the next free block in the list it
    // belongs to (either a free-list or the
    // occupied list).
    next: Option<BlockIndex>,
    // The last frame this block was referenced.
    last_access_time: FrameId,
}

impl Block {
    fn new(
        address: GpuCacheAddress,
        next: Option<BlockIndex>,
        frame_id: FrameId,
        epoch: Epoch,
    ) -> Self {
        Block {
            address,
            next,
            last_access_time: frame_id,
            epoch,
        }
    }

    fn advance_epoch(&mut self, max_epoch: &mut Epoch) {
        self.epoch.next();
        if max_epoch.0 < self.epoch.0 {
            max_epoch.0 = self.epoch.0;
        }
    }

    /// Creates an invalid dummy block ID.
    pub const INVALID: Block = Block {
        address: GpuCacheAddress { u: 0, v: 0 },
        epoch: Epoch(0),
        next: None,
        last_access_time: FrameId::INVALID,
    };
}

/// Represents the index of a Block in the block array. We only create such
/// structs for blocks that represent the start of a chunk.
///
/// Because we use Option<BlockIndex> in a lot of places, we use a NonZeroU32
/// here and avoid ever using the index zero.
#[derive(Debug, Copy, Clone, MallocSizeOf)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
struct BlockIndex(NonZeroU32);

impl BlockIndex {
    fn new(idx: usize) -> Self {
        debug_assert!(idx <= u32::MAX as usize);
        BlockIndex(NonZeroU32::new(idx as u32).expect("Index zero forbidden"))
    }

    fn get(&self) -> usize {
        self.0.get() as usize
    }
}

// A row in the cache texture.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(MallocSizeOf)]
struct Row {
    // The fixed size of blocks that this row supports.
    // Each row becomes a slab allocator for a fixed block size.
    // This means no dealing with fragmentation within a cache
    // row as items are allocated and freed.
    block_count_per_item: usize,
}

impl Row {
    fn new(block_count_per_item: usize) -> Self {
        Row {
            block_count_per_item,
        }
    }
}

// A list of update operations that can be applied on the cache
// this frame. The list of updates is created by the render backend
// during frame construction. It's passed to the render thread
// where GL commands can be applied.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(MallocSizeOf)]
pub enum GpuCacheUpdate {
    Copy {
        block_index: usize,
        block_count: usize,
        address: GpuCacheAddress,
    },
}

/// Command to inform the debug display in the renderer when chunks are allocated
/// or freed.
#[derive(MallocSizeOf)]
pub enum GpuCacheDebugCmd {
    /// Describes an allocated chunk.
    Alloc(GpuCacheDebugChunk),
    /// Describes a freed chunk.
    Free(GpuCacheAddress),
}

#[derive(Clone, MallocSizeOf)]
pub struct GpuCacheDebugChunk {
    pub address: GpuCacheAddress,
    pub size: usize,
}

#[must_use]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(MallocSizeOf)]
pub struct GpuCacheUpdateList {
    /// The frame current update list was generated from.
    pub frame_id: FrameId,
    /// Whether the texture should be cleared before updates
    /// are applied.
    pub clear: bool,
    /// The current height of the texture. The render thread
    /// should resize the texture if required.
    pub height: i32,
    /// List of updates to apply.
    pub updates: Vec<GpuCacheUpdate>,
    /// A flat list of GPU blocks that are pending upload
    /// to GPU memory.
    pub blocks: Vec<GpuBlockData>,
    /// Whole state GPU block metadata for debugging.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub debug_commands: Vec<GpuCacheDebugCmd>,
}

// Holds the free lists of fixed size blocks. Mostly
// just serves to work around the borrow checker.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(MallocSizeOf)]
struct FreeBlockLists {
    free_list_1: Option<BlockIndex>,
    free_list_2: Option<BlockIndex>,
    free_list_4: Option<BlockIndex>,
    free_list_8: Option<BlockIndex>,
    free_list_16: Option<BlockIndex>,
    free_list_32: Option<BlockIndex>,
    free_list_64: Option<BlockIndex>,
    free_list_128: Option<BlockIndex>,
    free_list_256: Option<BlockIndex>,
    free_list_341: Option<BlockIndex>,
    free_list_512: Option<BlockIndex>,
    free_list_1024: Option<BlockIndex>,
}

impl FreeBlockLists {
    fn new() -> Self {
        FreeBlockLists {
            free_list_1: None,
            free_list_2: None,
            free_list_4: None,
            free_list_8: None,
            free_list_16: None,
            free_list_32: None,
            free_list_64: None,
            free_list_128: None,
            free_list_256: None,
            free_list_341: None,
            free_list_512: None,
            free_list_1024: None,
        }
    }

    fn get_actual_block_count_and_free_list(
        &mut self,
        block_count: usize,
    ) -> (usize, &mut Option<BlockIndex>) {
        // Find the appropriate free list to use based on the block size.
        //
        // Note that we cheat a bit with the 341 bucket, since it's not quite
        // a divisor of 1024, because purecss-francine allocates many 260-block
        // chunks, and there's no reason we shouldn't pack these three to a row.
        // This means the allocation statistics will under-report by one block
        // for each row using 341-block buckets, which is fine.
        debug_assert_eq!(MAX_VERTEX_TEXTURE_WIDTH, 1024, "Need to update bucketing");
        match block_count {
            0 => panic!("Can't allocate zero sized blocks!"),
            1 => (1, &mut self.free_list_1),
            2 => (2, &mut self.free_list_2),
            3..=4 => (4, &mut self.free_list_4),
            5..=8 => (8, &mut self.free_list_8),
            9..=16 => (16, &mut self.free_list_16),
            17..=32 => (32, &mut self.free_list_32),
            33..=64 => (64, &mut self.free_list_64),
            65..=128 => (128, &mut self.free_list_128),
            129..=256 => (256, &mut self.free_list_256),
            257..=341 => (341, &mut self.free_list_341),
            342..=512 => (512, &mut self.free_list_512),
            513..=1024 => (1024, &mut self.free_list_1024),
            _ => panic!("Can't allocate > MAX_VERTEX_TEXTURE_WIDTH per resource!"),
        }
    }
}

// CPU-side representation of the GPU resource cache texture.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(MallocSizeOf)]
struct Texture {
    // Current texture height
    height: i32,
    // All blocks that have been created for this texture
    blocks: Vec<Block>,
    // Metadata about each allocated row.
    rows: Vec<Row>,
    // The base Epoch for this texture.
    base_epoch: Epoch,
    // The maximum epoch reached. We track this along with the above so
    // that we can rebuild the Texture and avoid collisions with handles
    // allocated for the old texture.
    max_epoch: Epoch,
    // Free lists of available blocks for each supported
    // block size in the texture. These are intrusive
    // linked lists.
    free_lists: FreeBlockLists,
    // Linked list of currently occupied blocks. This
    // makes it faster to iterate blocks looking for
    // candidates to be evicted from the cache.
    occupied_list_heads: FastHashMap<DocumentId, BlockIndex>,
    // Pending blocks that have been written this frame
    // and will need to be sent to the GPU.
    pending_blocks: Vec<GpuBlockData>,
    // Pending update commands.
    updates: Vec<GpuCacheUpdate>,
    // Profile stats
    allocated_block_count: usize,
    // The stamp at which we first reached our threshold for reclaiming `GpuCache`
    // memory, or `None` if the threshold hasn't been reached.
    #[cfg_attr(feature = "serde", serde(skip))]
    reached_reclaim_threshold: Option<Instant>,
    // List of debug commands to be sent to the renderer when the GPU cache
    // debug display is enabled.
    #[cfg_attr(feature = "serde", serde(skip))]
    debug_commands: Vec<GpuCacheDebugCmd>,
    // The current debug flags for the system.
    debug_flags: DebugFlags,
}

impl Texture {
    fn new(base_epoch: Epoch, debug_flags: DebugFlags) -> Self {
        // Pre-fill the block array with one invalid block so that we never use
        // 0 for a BlockIndex. This lets us use NonZeroU32 for BlockIndex, which
        // saves memory.
        let blocks = vec![Block::INVALID];

        Texture {
            height: GPU_CACHE_INITIAL_HEIGHT,
            blocks,
            rows: Vec::new(),
            base_epoch,
            max_epoch: base_epoch,
            free_lists: FreeBlockLists::new(),
            pending_blocks: Vec::new(),
            updates: Vec::new(),
            occupied_list_heads: FastHashMap::default(),
            allocated_block_count: 0,
            reached_reclaim_threshold: None,
            debug_commands: Vec::new(),
            debug_flags,
        }
    }

    // Push new data into the cache. The ```pending_block_index``` field represents
    // where the data was pushed into the texture ```pending_blocks``` array.
    // Return the allocated address for this data.
    fn push_data(
        &mut self,
        pending_block_index: Option<usize>,
        block_count: usize,
        frame_stamp: FrameStamp
    ) -> CacheLocation {
        debug_assert!(frame_stamp.is_valid());
        // Find the appropriate free list to use based on the block size.
        let (alloc_size, free_list) = self.free_lists
            .get_actual_block_count_and_free_list(block_count);

        // See if we need a new row (if free-list has nothing available)
        if free_list.is_none() {
            if self.rows.len() as i32 == self.height {
                self.height += NEW_ROWS_PER_RESIZE;
            }

            // Create a new row.
            let items_per_row = MAX_VERTEX_TEXTURE_WIDTH / alloc_size;
            let row_index = self.rows.len();
            self.rows.push(Row::new(alloc_size));

            // Create a ```Block``` for each possible allocation address
            // in this row, and link it in to the free-list for this
            // block size.
            let mut prev_block_index = None;
            for i in 0 .. items_per_row {
                let address = GpuCacheAddress::new(i * alloc_size, row_index);
                let block_index = BlockIndex::new(self.blocks.len());
                let block = Block::new(address, prev_block_index, frame_stamp.frame_id(), self.base_epoch);
                self.blocks.push(block);
                prev_block_index = Some(block_index);
            }

            *free_list = prev_block_index;
        }

        // Given the code above, it's now guaranteed that there is a block
        // available in the appropriate free-list. Pull a block from the
        // head of the list.
        let free_block_index = free_list.take().unwrap();
        let block = &mut self.blocks[free_block_index.get()];
        *free_list = block.next;

        // Add the block to the occupied linked list.
        block.next = self.occupied_list_heads.get(&frame_stamp.document_id()).cloned();
        block.last_access_time = frame_stamp.frame_id();
        self.occupied_list_heads.insert(frame_stamp.document_id(), free_block_index);
        self.allocated_block_count += alloc_size;

        if let Some(pending_block_index) = pending_block_index {
            // Add this update to the pending list of blocks that need
            // to be updated on the GPU.
            self.updates.push(GpuCacheUpdate::Copy {
                block_index: pending_block_index,
                block_count,
                address: block.address,
            });
        }

        // If we're using the debug display, communicate the allocation to the
        // renderer thread. Note that we do this regardless of whether or not
        // pending_block_index is None (if it is, the renderer thread will fill
        // in the data via a deferred resolve, but the block is still considered
        // allocated).
        if self.debug_flags.contains(DebugFlags::GPU_CACHE_DBG) {
            self.debug_commands.push(GpuCacheDebugCmd::Alloc(GpuCacheDebugChunk {
                address: block.address,
                size: block_count,
            }));
        }

        CacheLocation {
            block_index: free_block_index,
            epoch: block.epoch,
        }
    }

    // Run through the list of occupied cache blocks and evict
    // any old blocks that haven't been referenced for a while.
    fn evict_old_blocks(&mut self, frame_stamp: FrameStamp) {
        debug_assert!(frame_stamp.is_valid());
        // Prune any old items from the list to make room.
        // Traverse the occupied linked list and see
        // which items have not been used for a long time.
        let mut current_block = self.occupied_list_heads.get(&frame_stamp.document_id()).map(|x| *x);
        let mut prev_block: Option<BlockIndex> = None;

        while let Some(index) = current_block {
            let (next_block, should_unlink) = {
                let block = &mut self.blocks[index.get()];

                let next_block = block.next;
                let mut should_unlink = false;

                // If this resource has not been used in the last
                // few frames, free it from the texture and mark
                // as empty.
                if block.last_access_time + FRAMES_BEFORE_EVICTION < frame_stamp.frame_id() {
                    should_unlink = true;

                    // Get the row metadata from the address.
                    let row = &mut self.rows[block.address.v as usize];

                    // Use the row metadata to determine which free-list
                    // this block belongs to.
                    let (_, free_list) = self.free_lists
                        .get_actual_block_count_and_free_list(row.block_count_per_item);

                    block.advance_epoch(&mut self.max_epoch);
                    block.next = *free_list;
                    *free_list = Some(index);

                    self.allocated_block_count -= row.block_count_per_item;

                    if self.debug_flags.contains(DebugFlags::GPU_CACHE_DBG) {
                        let cmd = GpuCacheDebugCmd::Free(block.address);
                        self.debug_commands.push(cmd);
                    }
                };

                (next_block, should_unlink)
            };

            // If the block was released, we will need to remove it
            // from the occupied linked list.
            if should_unlink {
                match prev_block {
                    Some(prev_block) => {
                        self.blocks[prev_block.get()].next = next_block;
                    }
                    None => {
                        match next_block {
                            Some(next_block) => {
                                self.occupied_list_heads.insert(frame_stamp.document_id(), next_block);
                            }
                            None => {
                                self.occupied_list_heads.remove(&frame_stamp.document_id());
                            }
                        }
                    }
                }
            } else {
                prev_block = current_block;
            }

            current_block = next_block;
        }
    }

    /// Returns the ratio of utilized blocks.
    fn utilization(&self) -> f32 {
        let total_blocks = self.rows.len() * MAX_VERTEX_TEXTURE_WIDTH;
        debug_assert!(total_blocks > 0);
        let ratio = self.allocated_block_count as f32 / total_blocks as f32;
        debug_assert!(0.0 <= ratio && ratio <= 1.0, "Bad ratio: {}", ratio);
        ratio
    }
}


/// A wrapper object for GPU data requests,
/// works as a container that can only grow.
#[must_use]
pub struct GpuDataRequest<'a> {
    handle: &'a mut GpuCacheHandle,
    frame_stamp: FrameStamp,
    start_index: usize,
    max_block_count: usize,
    texture: &'a mut Texture,
}

impl<'a> GpuDataRequest<'a> {
    pub fn push<B>(&mut self, block: B)
    where
        B: Into<GpuBlockData>,
    {
        self.texture.pending_blocks.push(block.into());
    }

    pub fn current_used_block_num(&self) -> usize {
        self.texture.pending_blocks.len() - self.start_index
    }
}

impl<'a> Drop for GpuDataRequest<'a> {
    fn drop(&mut self) {
        // Push the data to the texture pending updates list.
        let block_count = self.current_used_block_num();
        debug_assert!(block_count <= self.max_block_count);

        let location = self.texture
            .push_data(Some(self.start_index), block_count, self.frame_stamp);
        self.handle.location = Some(location);
    }
}


/// The main LRU cache interface.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(MallocSizeOf)]
pub struct GpuCache {
    /// Current FrameId.
    now: FrameStamp,
    /// CPU-side texture allocator.
    texture: Texture,
    /// Number of blocks requested this frame that don't
    /// need to be re-uploaded.
    saved_block_count: usize,
    /// The current debug flags for the system.
    debug_flags: DebugFlags,
    /// Whether there is a pending clear to send with the
    /// next update.
    pending_clear: bool,
    /// Indicates that prepare_for_frames has been called for this group of frames.
    /// Used for sanity checks.
    prepared_for_frames: bool,
    /// This indicates that we performed a cleanup operation which requires all
    /// documents to build a frame.
    requires_frame_build: bool,
    /// The set of documents which have had frames built in this update. Used for
    /// sanity checks.
    document_frames_to_build: FastHashSet<DocumentId>,
}

impl GpuCache {
    pub fn new() -> Self {
        let debug_flags = DebugFlags::empty();
        GpuCache {
            now: FrameStamp::INVALID,
            texture: Texture::new(Epoch(0), debug_flags),
            saved_block_count: 0,
            debug_flags,
            pending_clear: false,
            prepared_for_frames: false,
            requires_frame_build: false,
            document_frames_to_build: FastHashSet::default(),
        }
    }

    /// Creates a GpuCache and sets it up with a valid `FrameStamp`, which
    /// is useful for avoiding panics when instantiating the `GpuCache`
    /// directly from unit test code.
    #[cfg(test)]
    pub fn new_for_testing() -> Self {
        let mut cache = Self::new();
        let mut now = FrameStamp::first(DocumentId::new(IdNamespace(1), 1));
        now.advance();
        cache.prepared_for_frames = true;
        cache.begin_frame(now);
        cache
    }

    /// Drops everything in the GPU cache. Must not be called once gpu cache entries
    /// for the next frame have already been requested.
    pub fn clear(&mut self) {
        assert!(self.texture.updates.is_empty(), "Clearing with pending updates");
        let mut next_base_epoch = self.texture.max_epoch;
        next_base_epoch.next();
        self.texture = Texture::new(next_base_epoch, self.debug_flags);
        self.saved_block_count = 0;
        self.pending_clear = true;
        self.requires_frame_build = true;
    }

    pub fn requires_frame_build(&self) -> bool {
        self.requires_frame_build
    }

    pub fn prepare_for_frames(&mut self) {
        self.prepared_for_frames = true;
        if self.should_reclaim_memory() {
            self.clear();
            debug_assert!(self.document_frames_to_build.is_empty());
            for &document_id in self.texture.occupied_list_heads.keys() {
                self.document_frames_to_build.insert(document_id);
            }
        }
    }

    pub fn bookkeep_after_frames(&mut self) {
        assert!(self.document_frames_to_build.is_empty());
        assert!(self.prepared_for_frames);
        self.requires_frame_build = false;
        self.prepared_for_frames = false;
    }

    /// Begin a new frame.
    pub fn begin_frame(&mut self, stamp: FrameStamp) {
        debug_assert!(self.texture.pending_blocks.is_empty());
        assert!(self.prepared_for_frames);
        profile_scope!("begin_frame");
        self.now = stamp;
        self.texture.evict_old_blocks(self.now);
        self.saved_block_count = 0;
    }

    // Invalidate a (possibly) existing block in the cache.
    // This means the next call to request() for this location
    // will rebuild the data and upload it to the GPU.
    pub fn invalidate(&mut self, handle: &GpuCacheHandle) {
        if let Some(ref location) = handle.location {
            // don't invalidate blocks that are already re-assigned
            if let Some(block) = self.texture.blocks.get_mut(location.block_index.get()) {
                if block.epoch == location.epoch {
                    block.advance_epoch(&mut self.texture.max_epoch);
                }
            }
        }
    }

    /// Request a resource be added to the cache. If the resource
    /// is already in the cache, `None` will be returned.
    pub fn request<'a>(&'a mut self, handle: &'a mut GpuCacheHandle) -> Option<GpuDataRequest<'a>> {
        let mut max_block_count = MAX_VERTEX_TEXTURE_WIDTH;
        // Check if the allocation for this handle is still valid.
        if let Some(ref location) = handle.location {
            if let Some(block) = self.texture.blocks.get_mut(location.block_index.get()) {
                if block.epoch == location.epoch {
                    max_block_count = self.texture.rows[block.address.v as usize].block_count_per_item;
                    if block.last_access_time != self.now.frame_id() {
                        // Mark last access time to avoid evicting this block.
                        block.last_access_time = self.now.frame_id();
                        self.saved_block_count += max_block_count;
                    }
                    return None;
                }
            }
        }

        debug_assert!(self.now.is_valid());
        Some(GpuDataRequest {
            handle,
            frame_stamp: self.now,
            start_index: self.texture.pending_blocks.len(),
            texture: &mut self.texture,
            max_block_count,
        })
    }

    // Push an array of data blocks to be uploaded to the GPU
    // unconditionally for this frame. The cache handle will
    // assert if the caller tries to retrieve the address
    // of this handle on a subsequent frame. This is typically
    // used for uploading data that changes every frame, and
    // therefore makes no sense to try and cache.
    pub fn push_per_frame_blocks(&mut self, blocks: &[GpuBlockData]) -> GpuCacheHandle {
        let start_index = self.texture.pending_blocks.len();
        self.texture.pending_blocks.extend_from_slice(blocks);
        let location = self.texture
            .push_data(Some(start_index), blocks.len(), self.now);
        GpuCacheHandle {
            location: Some(location),
        }
    }

    // Reserve space in the cache for per-frame blocks that
    // will be resolved by the render thread via the
    // external image callback.
    pub fn push_deferred_per_frame_blocks(&mut self, block_count: usize) -> GpuCacheHandle {
        let location = self.texture.push_data(None, block_count, self.now);
        GpuCacheHandle {
            location: Some(location),
        }
    }

    /// End the frame. Return the list of updates to apply to the
    /// device specific cache texture.
    pub fn end_frame(
        &mut self,
        profile_counters: &mut GpuCacheProfileCounters,
    ) -> FrameStamp {
        profile_scope!("end_frame");
        profile_counters
            .allocated_rows
            .set(self.texture.rows.len());
        profile_counters
            .allocated_blocks
            .set(self.texture.allocated_block_count);
        profile_counters
            .saved_blocks
            .set(self.saved_block_count);

        let reached_threshold =
            self.texture.rows.len() > (GPU_CACHE_INITIAL_HEIGHT as usize) &&
            self.texture.utilization() < RECLAIM_THRESHOLD;
        if reached_threshold {
            self.texture.reached_reclaim_threshold.get_or_insert_with(Instant::now);
        } else {
            self.texture.reached_reclaim_threshold = None;
        }

        self.document_frames_to_build.remove(&self.now.document_id());
        self.now
    }

    /// Returns true if utilization has been low enough for long enough that we
    /// should blow the cache away and rebuild it.
    pub fn should_reclaim_memory(&self) -> bool {
        self.texture.reached_reclaim_threshold
            .map_or(false, |t| t.elapsed() > Duration::from_secs(RECLAIM_DELAY_S))
    }

    /// Extract the pending updates from the cache.
    pub fn extract_updates(&mut self) -> GpuCacheUpdateList {
        let clear = self.pending_clear;
        self.pending_clear = false;
        GpuCacheUpdateList {
            frame_id: self.now.frame_id(),
            clear,
            height: self.texture.height,
            debug_commands: mem::replace(&mut self.texture.debug_commands, Vec::new()),
            updates: mem::replace(&mut self.texture.updates, Vec::new()),
            blocks: mem::replace(&mut self.texture.pending_blocks, Vec::new()),
        }
    }

    /// Sets the current debug flags for the system.
    pub fn set_debug_flags(&mut self, flags: DebugFlags) {
        self.debug_flags = flags;
        self.texture.debug_flags = flags;
    }

    /// Get the actual GPU address in the texture for a given slot ID.
    /// It's assumed at this point that the given slot has been requested
    /// and built for this frame. Attempting to get the address for a
    /// freed or pending slot will panic!
    pub fn get_address(&self, id: &GpuCacheHandle) -> GpuCacheAddress {
        let location = id.location.expect("handle not requested or allocated!");
        let block = &self.texture.blocks[location.block_index.get()];
        debug_assert_eq!(block.epoch, location.epoch);
        debug_assert_eq!(block.last_access_time, self.now.frame_id());
        block.address
    }
}

#[test]
#[cfg(target_pointer_width = "64")]
fn test_struct_sizes() {
    use std::mem;
    // We can end up with a lot of blocks stored in the global vec, and keeping
    // them small helps reduce memory overhead.
    assert_eq!(mem::size_of::<Block>(), 24, "Block size changed");
}
