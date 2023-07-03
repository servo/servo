/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use api::{DirtyRect, ExternalImageType, ImageFormat};
use api::{DebugFlags, ImageDescriptor};
use api::units::*;
#[cfg(test)]
use api::{DocumentId, IdNamespace};
use crate::device::{TextureFilter, TextureFormatPair};
use crate::freelist::{FreeListHandle, WeakFreeListHandle};
use crate::gpu_cache::{GpuCache, GpuCacheHandle};
use crate::gpu_types::{ImageSource, UvRectKind};
use crate::internal_types::{
    CacheTextureId, LayerIndex, Swizzle, SwizzleSettings,
    TextureUpdateList, TextureUpdateSource, TextureSource,
    TextureCacheAllocInfo, TextureCacheUpdate,
};
use crate::lru_cache::LRUCache;
use crate::profiler::{ResourceProfileCounter, TextureCacheProfileCounters};
use crate::render_backend::FrameStamp;
use crate::resource_cache::{CacheItem, CachedImageData};
use smallvec::SmallVec;
use std::cell::Cell;
use std::cmp;
use std::mem;
use std::rc::Rc;

/// The size of each region/layer in shared cache texture arrays.
pub const TEXTURE_REGION_DIMENSIONS: i32 = 512;

const PICTURE_TEXTURE_ADD_SLICES: usize = 4;

/// The chosen image format for picture tiles.
const PICTURE_TILE_FORMAT: ImageFormat = ImageFormat::RGBA8;

/// The number of pixels in a region. Derived from the above.
const TEXTURE_REGION_PIXELS: usize =
    (TEXTURE_REGION_DIMENSIONS as usize) * (TEXTURE_REGION_DIMENSIONS as usize);

/// Items in the texture cache can either be standalone textures,
/// or a sub-rect inside the shared cache.
#[derive(Debug)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
enum EntryDetails {
    Standalone {
        /// Number of bytes this entry allocates
        size_in_bytes: usize,
    },
    Picture {
        // Index in the picture_textures array
        texture_index: usize,
        // Slice in the texture array
        layer_index: usize,
    },
    Cache {
        /// Origin within the texture layer where this item exists.
        origin: DeviceIntPoint,
        /// The layer index of the texture array.
        layer_index: usize,
    },
}

impl EntryDetails {
    fn describe(&self) -> (LayerIndex, DeviceIntPoint) {
        match *self {
            EntryDetails::Standalone { .. }  => (0, DeviceIntPoint::zero()),
            EntryDetails::Picture { layer_index, .. } => (layer_index, DeviceIntPoint::zero()),
            EntryDetails::Cache { origin, layer_index, .. } => (layer_index, origin),
        }
    }
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub enum CacheEntryMarker {}

// Stores information related to a single entry in the texture
// cache. This is stored for each item whether it's in the shared
// cache or a standalone texture.
#[derive(Debug)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
struct CacheEntry {
    /// Size the requested item, in device pixels.
    size: DeviceIntSize,
    /// Details specific to standalone or shared items.
    details: EntryDetails,
    /// Arbitrary user data associated with this item.
    user_data: [f32; 3],
    /// The last frame this item was requested for rendering.
    // TODO(gw): This stamp is only used for picture cache tiles, and some checks
    //           in the glyph cache eviction code. We could probably remove it
    //           entirely in future (or move to EntryDetails::Picture).
    last_access: FrameStamp,
    /// Handle to the resource rect in the GPU cache.
    uv_rect_handle: GpuCacheHandle,
    /// Image format of the data that the entry expects.
    input_format: ImageFormat,
    filter: TextureFilter,
    swizzle: Swizzle,
    /// The actual device texture ID this is part of.
    texture_id: CacheTextureId,
    /// Optional notice when the entry is evicted from the cache.
    eviction_notice: Option<EvictionNotice>,
    /// The type of UV rect this entry specifies.
    uv_rect_kind: UvRectKind,
}

impl CacheEntry {
    // Create a new entry for a standalone texture.
    fn new_standalone(
        texture_id: CacheTextureId,
        last_access: FrameStamp,
        params: &CacheAllocParams,
        swizzle: Swizzle,
        size_in_bytes: usize,
    ) -> Self {
        CacheEntry {
            size: params.descriptor.size,
            user_data: params.user_data,
            last_access,
            details: EntryDetails::Standalone {
                size_in_bytes,
            },
            texture_id,
            input_format: params.descriptor.format,
            filter: params.filter,
            swizzle,
            uv_rect_handle: GpuCacheHandle::new(),
            eviction_notice: None,
            uv_rect_kind: params.uv_rect_kind,
        }
    }

    // Update the GPU cache for this texture cache entry.
    // This ensures that the UV rect, and texture layer index
    // are up to date in the GPU cache for vertex shaders
    // to fetch from.
    fn update_gpu_cache(&mut self, gpu_cache: &mut GpuCache) {
        if let Some(mut request) = gpu_cache.request(&mut self.uv_rect_handle) {
            let (layer_index, origin) = self.details.describe();
            let image_source = ImageSource {
                p0: origin.to_f32(),
                p1: (origin + self.size).to_f32(),
                texture_layer: layer_index as f32,
                user_data: self.user_data,
                uv_rect_kind: self.uv_rect_kind,
            };
            image_source.write_gpu_blocks(&mut request);
        }
    }

    fn evict(&self) {
        if let Some(eviction_notice) = self.eviction_notice.as_ref() {
            eviction_notice.notify();
        }
    }

    fn alternative_input_format(&self) -> ImageFormat {
        match self.input_format {
            ImageFormat::RGBA8 => ImageFormat::BGRA8,
            ImageFormat::BGRA8 => ImageFormat::RGBA8,
            other => other,
        }
    }
}


/// A texture cache handle is a weak reference to a cache entry.
///
/// If the handle has not been inserted into the cache yet, or if the entry was
/// previously inserted and then evicted, lookup of the handle will fail, and
/// the cache handle needs to re-upload this item to the texture cache (see
/// request() below).
pub type TextureCacheHandle = WeakFreeListHandle<CacheEntryMarker>;

/// Describes the eviction policy for a given entry in the texture cache.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub enum Eviction {
    /// The entry will be evicted under the normal rules (which differ between
    /// standalone and shared entries).
    Auto,
    /// The entry will not be evicted until the policy is explicitly set to a
    /// different value.
    Manual,
}

// An eviction notice is a shared condition useful for detecting
// when a TextureCacheHandle gets evicted from the TextureCache.
// It is optionally installed to the TextureCache when an update()
// is scheduled. A single notice may be shared among any number of
// TextureCacheHandle updates. The notice may then be subsequently
// checked to see if any of the updates using it have been evicted.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct EvictionNotice {
    evicted: Rc<Cell<bool>>,
}

impl EvictionNotice {
    fn notify(&self) {
        self.evicted.set(true);
    }

    pub fn check(&self) -> bool {
        if self.evicted.get() {
            self.evicted.set(false);
            true
        } else {
            false
        }
    }
}

/// A set of lazily allocated, fixed size, texture arrays for each format the
/// texture cache supports.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
struct SharedTextures {
    array_color8_nearest: TextureArray,
    array_alpha8_linear: TextureArray,
    array_alpha16_linear: TextureArray,
    array_color8_linear: TextureArray,
}

impl SharedTextures {
    /// Mints a new set of shared textures.
    fn new(color_formats: TextureFormatPair<ImageFormat>) -> Self {
        Self {
            // Used primarily for cached shadow masks. There can be lots of
            // these on some pages like francine, but most pages don't use it
            // much.
            array_alpha8_linear: TextureArray::new(
                TextureFormatPair::from(ImageFormat::R8),
                TextureFilter::Linear,
                4,
            ),
            // Used for experimental hdr yuv texture support, but not used in
            // production Firefox.
            array_alpha16_linear: TextureArray::new(
                TextureFormatPair::from(ImageFormat::R16),
                TextureFilter::Linear,
                1,
            ),
            // The primary cache for images, glyphs, etc.
            array_color8_linear: TextureArray::new(
                color_formats.clone(),
                TextureFilter::Linear,
                16,
            ),
            // Used for image-rendering: crisp. This is mostly favicons, which
            // are small. Some other images use it too, but those tend to be
            // larger than 512x512 and thus don't use the shared cache anyway.
            array_color8_nearest: TextureArray::new(
                color_formats,
                TextureFilter::Nearest,
                1,
            ),
        }
    }

    /// Clears each texture in the set, with the given set of pending updates.
    fn clear(&mut self, updates: &mut TextureUpdateList) {
        self.array_alpha8_linear.clear(updates);
        self.array_alpha16_linear.clear(updates);
        self.array_color8_linear.clear(updates);
        self.array_color8_nearest.clear(updates);
    }

    /// Returns a mutable borrow for the shared texture array matching the parameters.
    fn select(
        &mut self, external_format: ImageFormat, filter: TextureFilter
    ) -> &mut TextureArray {
        match external_format {
            ImageFormat::R8 => {
                assert_eq!(filter, TextureFilter::Linear);
                &mut self.array_alpha8_linear
            }
            ImageFormat::R16 => {
                assert_eq!(filter, TextureFilter::Linear);
                &mut self.array_alpha16_linear
            }
            ImageFormat::RGBA8 |
            ImageFormat::BGRA8 => {
                match filter {
                    TextureFilter::Linear => &mut self.array_color8_linear,
                    TextureFilter::Nearest => &mut self.array_color8_nearest,
                    _ => panic!("Unexpexcted filter {:?}", filter),
                }
            }
            _ => panic!("Unexpected format {:?}", external_format),
        }
    }
}

/// The texture arrays used to hold picture cache tiles.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
struct PictureTextures {
    textures: Vec<WholeTextureArray>,
}

impl PictureTextures {
    fn new(
        initial_window_size: DeviceIntSize,
        picture_tile_sizes: &[DeviceIntSize],
        next_texture_id: &mut CacheTextureId,
        pending_updates: &mut TextureUpdateList,
    ) -> Self {
        let mut textures = Vec::new();
        for tile_size in picture_tile_sizes {
            // TODO(gw): The way initial size is used here may allocate a lot of memory once
            //           we are using multiple slice sizes. Do some measurements once we
            //           have multiple slices here and adjust the calculations as required.
            let num_x = (initial_window_size.width + tile_size.width - 1) / tile_size.width;
            let num_y = (initial_window_size.height + tile_size.height - 1) / tile_size.height;
            let mut slice_count = (num_x * num_y).max(1).min(16) as usize;
            if slice_count < 4 {
                // On some platforms we get bogus (1x1) initial window size. The first real frame will then
                // reallocate many more picture cache slices. Don't bother preallocating in that case.
                slice_count = 0;
            }

            if slice_count == 0 {
                continue;
            }

            let texture = WholeTextureArray {
                size: *tile_size,
                filter: TextureFilter::Nearest,
                format: PICTURE_TILE_FORMAT,
                texture_id: *next_texture_id,
                slices: vec![WholeTextureSlice { uv_rect_handle: None }; slice_count],
                has_depth: true,
            };

            next_texture_id.0 += 1;

            pending_updates.push_alloc(texture.texture_id, texture.to_info());

            textures.push(texture);
        }

        PictureTextures { textures }
    }

    fn get_or_allocate_tile(
        &mut self,
        tile_size: DeviceIntSize,
        now: FrameStamp,
        next_texture_id: &mut CacheTextureId,
        pending_updates: &mut TextureUpdateList,
    ) -> CacheEntry {
        let texture_index = self.textures
            .iter()
            .position(|texture| { texture.size == tile_size })
            .unwrap_or(self.textures.len());

        if texture_index == self.textures.len() {
            self.textures.push(WholeTextureArray {
                size: tile_size,
                filter: TextureFilter::Nearest,
                format: PICTURE_TILE_FORMAT,
                texture_id: *next_texture_id,
                slices: Vec::new(),
                has_depth: true,
            });
            next_texture_id.0 += 1;
        }

        let texture = &mut self.textures[texture_index];

        let layer_index = match texture.find_free() {
            Some(index) => index,
            None => {
                let was_empty = texture.slices.is_empty();
                let index = texture.grow(PICTURE_TEXTURE_ADD_SLICES);
                let info = texture.to_info();
                if was_empty {
                    pending_updates.push_alloc(texture.texture_id, info);
                } else {
                    pending_updates.push_realloc(texture.texture_id, info);
                }

                index
            },
        };

        texture.occupy(texture_index, layer_index, now)
    }

    fn get(&mut self, index: usize) -> &mut WholeTextureArray {
        &mut self.textures[index]
    }

    fn clear(&mut self, pending_updates: &mut TextureUpdateList) {
        for texture in &mut self.textures {
            if texture.slices.is_empty() {
                continue;
            }

            if let Some(texture_id) = texture.reset(PICTURE_TEXTURE_ADD_SLICES) {
                pending_updates.push_reset(texture_id, texture.to_info());
            }
        }
    }

    fn update_profile(&self, profile: &mut ResourceProfileCounter) {
        // For now, this profile counter just accumulates the slices and bytes
        // from all picture cache texture arrays.
        let mut picture_slices = 0;
        let mut picture_bytes = 0;
        for texture in &self.textures {
            picture_slices += texture.slices.len();
            picture_bytes += texture.size_in_bytes();
        }
        profile.set(picture_slices, picture_bytes);
    }

    #[cfg(feature = "replay")]
    fn tile_sizes(&self) -> Vec<DeviceIntSize> {
        self.textures.iter().map(|pt| pt.size).collect()
    }
}

/// Container struct for the various parameters used in cache allocation.
struct CacheAllocParams {
    descriptor: ImageDescriptor,
    filter: TextureFilter,
    user_data: [f32; 3],
    uv_rect_kind: UvRectKind,
}

/// General-purpose manager for images in GPU memory. This includes images,
/// rasterized glyphs, rasterized blobs, cached render tasks, etc.
///
/// The texture cache is owned and managed by the RenderBackend thread, and
/// produces a series of commands to manipulate the textures on the Renderer
/// thread. These commands are executed before any rendering is performed for
/// a given frame.
///
/// Entries in the texture cache are not guaranteed to live past the end of the
/// frame in which they are requested, and may be evicted. The API supports
/// querying whether an entry is still available.
///
/// The TextureCache is different from the GpuCache in that the former stores
/// images, whereas the latter stores data and parameters for use in the shaders.
/// This means that the texture cache can be visualized, which is a good way to
/// understand how it works. Enabling gfx.webrender.debug.texture-cache shows a
/// live view of its contents in Firefox.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct TextureCache {
    /// Set of texture arrays in different formats used for the shared cache.
    shared_textures: SharedTextures,

    /// A texture array per tile size for picture caching.
    picture_textures: PictureTextures,

    /// Maximum texture size supported by hardware.
    max_texture_size: i32,

    /// Maximum number of texture layers supported by hardware.
    max_texture_layers: usize,

    /// Settings on using texture unit swizzling.
    swizzle: Option<SwizzleSettings>,

    /// The current set of debug flags.
    debug_flags: DebugFlags,

    /// The next unused virtual texture ID. Monotonically increasing.
    next_id: CacheTextureId,

    /// A list of allocations and updates that need to be applied to the texture
    /// cache in the rendering thread this frame.
    #[cfg_attr(all(feature = "serde", any(feature = "capture", feature = "replay")), serde(skip))]
    pending_updates: TextureUpdateList,

    /// The current `FrameStamp`. Used for cache eviction policies.
    now: FrameStamp,

    /// List of picture cache entries. These are maintained separately from regular
    /// texture cache entries.
    picture_cache_handles: Vec<FreeListHandle<CacheEntryMarker>>,

    /// Cache of texture cache handles with automatic lifetime management, evicted
    /// in a least-recently-used order (except those entries with manual eviction enabled).
    lru_cache: LRUCache<CacheEntry, CacheEntryMarker>,

    /// A list of texture cache handles that have been set to explicitly have manual
    /// eviction policy enabled. The handles reference cache entries in the lru_cache
    /// above, but have opted in to manual lifetime management.
    manual_handles: Vec<FreeListHandle<CacheEntryMarker>>,

    /// Estimated memory usage of allocated entries in all of the shared textures. This
    /// is used to decide when to evict old items from the cache.
    shared_bytes_allocated: usize,

    /// Number of bytes allocated in standalone textures. Used as an input to deciding
    /// when to run texture cache eviction.
    standalone_bytes_allocated: usize,

    /// If the total bytes allocated in shared / standalone cache is less
    /// than this, then allow the cache to grow without forcing an eviction.
    // TODO(gw): In future, it's probably reasonable to make this higher again, perhaps 64-128 MB.
    eviction_threshold_bytes: usize,

    /// The maximum number of items that will be evicted per frame. This limit helps avoid jank
    /// on frames where we want to evict a large number of items. Instead, we'd prefer to drop
    /// the items incrementally over a number of frames, even if that means the total allocated
    /// size of the cache is above the desired threshold for a small number of frames.
    max_evictions_per_frame: usize,
}

impl TextureCache {
    pub fn new(
        max_texture_size: i32,
        mut max_texture_layers: usize,
        picture_tile_sizes: &[DeviceIntSize],
        initial_size: DeviceIntSize,
        color_formats: TextureFormatPair<ImageFormat>,
        swizzle: Option<SwizzleSettings>,
        eviction_threshold_bytes: usize,
        max_evictions_per_frame: usize,
    ) -> Self {
        // On MBP integrated Intel GPUs, texture arrays appear to be
        // implemented as a single texture of stacked layers, and that
        // texture appears to be subject to the texture size limit. As such,
        // allocating more than 32 512x512 regions results in a dimension
        // longer than 16k (the max texture size), causing incorrect behavior.
        //
        // So we clamp the number of layers on mac. This results in maximum
        // texture array size of 32MB, which isn't ideal but isn't terrible
        // either. OpenGL on mac is not long for this earth, so this may be
        // good enough until we have WebRender on gfx-rs (on Metal).
        //
        // On all platforms, we also clamp the number of textures per layer to 16
        // to avoid the cost of resizing large texture arrays (at the expense
        // of batching efficiency).
        //
        // Note that we could also define this more generally in terms of
        // |max_texture_size / TEXTURE_REGION_DIMENSION|, except:
        //   * max_texture_size is actually clamped beyond the device limit
        //     by Gecko to 8192, so we'd need to thread the raw device value
        //     here, and:
        //   * The bug we're working around is likely specific to a single
        //     driver family, and those drivers are also likely to share
        //     the same max texture size of 16k. If we do encounter a driver
        //     with the same bug but a lower max texture size, we might need
        //     to rethink our strategy anyway, since a limit below 32MB might
        //     start to introduce performance issues.
        max_texture_layers = max_texture_layers.min(16);

        let mut pending_updates = TextureUpdateList::new();

        // Shared texture cache controls swizzling on a per-entry basis, assuming that
        // the texture as a whole doesn't need to be swizzled (but only some entries do).
        // It would be possible to support this, but not needed at the moment.
        assert!(color_formats.internal != ImageFormat::BGRA8 ||
            swizzle.map_or(true, |s| s.bgra8_sampling_swizzle == Swizzle::default())
        );

        let mut next_texture_id = CacheTextureId(1);

        TextureCache {
            shared_textures: SharedTextures::new(color_formats),
            picture_textures: PictureTextures::new(
                initial_size,
                picture_tile_sizes,
                &mut next_texture_id,
                &mut pending_updates,
            ),
            max_texture_size,
            max_texture_layers,
            swizzle,
            debug_flags: DebugFlags::empty(),
            next_id: next_texture_id,
            pending_updates,
            now: FrameStamp::INVALID,
            lru_cache: LRUCache::new(),
            shared_bytes_allocated: 0,
            standalone_bytes_allocated: 0,
            picture_cache_handles: Vec::new(),
            manual_handles: Vec::new(),
            eviction_threshold_bytes,
            max_evictions_per_frame,
        }
    }

    /// Creates a TextureCache and sets it up with a valid `FrameStamp`, which
    /// is useful for avoiding panics when instantiating the `TextureCache`
    /// directly from unit test code.
    #[cfg(test)]
    pub fn new_for_testing(
        max_texture_size: i32,
        max_texture_layers: usize,
        image_format: ImageFormat,
    ) -> Self {
        let mut cache = Self::new(
            max_texture_size,
            max_texture_layers,
            &[],
            DeviceIntSize::zero(),
            TextureFormatPair::from(image_format),
            None,
            64 * 1024 * 1024,
            32,
        );
        let mut now = FrameStamp::first(DocumentId::new(IdNamespace(1), 1));
        now.advance();
        cache.begin_frame(now);
        cache
    }

    pub fn set_debug_flags(&mut self, flags: DebugFlags) {
        self.debug_flags = flags;
    }

    /// Clear all entries in the texture cache. This is a fairly drastic
    /// step that should only be called very rarely.
    pub fn clear_all(&mut self) {
        // Evict all manual eviction handles
        let manual_handles = mem::replace(
            &mut self.manual_handles,
            Vec::new(),
        );
        for handle in manual_handles {
            self.evict_impl(handle);
        }

        // Evict all picture cache handles
        let picture_handles = mem::replace(
            &mut self.picture_cache_handles,
            Vec::new(),
        );
        for handle in picture_handles {
            self.evict_impl(handle);
        }

        // Evict all auto (LRU) cache handles
        while let Some(entry) = self.lru_cache.pop_oldest() {
            entry.evict();
            self.free(&entry);
        }

        // Free the picture and shared textures
        self.picture_textures.clear(&mut self.pending_updates);
        self.shared_textures.clear(&mut self.pending_updates);
        self.pending_updates.note_clear();
    }

    /// Called at the beginning of each frame.
    pub fn begin_frame(&mut self, stamp: FrameStamp) {
        debug_assert!(!self.now.is_valid());
        profile_scope!("begin_frame");
        self.now = stamp;

        // Texture cache eviction is done at the start of the frame. This ensures that
        // we won't evict items that have been requested on this frame.
        self.evict_items_from_cache_if_required();
    }

    pub fn end_frame(&mut self, texture_cache_profile: &mut TextureCacheProfileCounters) {
        debug_assert!(self.now.is_valid());
        self.expire_old_picture_cache_tiles();

        // Release of empty shared textures is done at the end of the frame. That way, if the
        // eviction at the start of the frame frees up a texture, that is then subsequently
        // used during the frame, we avoid doing a free/alloc for it.
        self.shared_textures.array_alpha8_linear.release_empty_textures(&mut self.pending_updates);
        self.shared_textures.array_alpha16_linear.release_empty_textures(&mut self.pending_updates);
        self.shared_textures.array_color8_linear.release_empty_textures(&mut self.pending_updates);
        self.shared_textures.array_color8_nearest.release_empty_textures(&mut self.pending_updates);

        self.shared_textures.array_alpha8_linear
            .update_profile(&mut texture_cache_profile.pages_alpha8_linear);
        self.shared_textures.array_alpha16_linear
            .update_profile(&mut texture_cache_profile.pages_alpha16_linear);
        self.shared_textures.array_color8_linear
            .update_profile(&mut texture_cache_profile.pages_color8_linear);
        self.shared_textures.array_color8_nearest
            .update_profile(&mut texture_cache_profile.pages_color8_nearest);
        self.picture_textures
            .update_profile(&mut texture_cache_profile.pages_picture);
        texture_cache_profile.shared_bytes.set(self.shared_bytes_allocated);
        texture_cache_profile.standalone_bytes.set(self.standalone_bytes_allocated);

        self.now = FrameStamp::INVALID;
    }

    // Request an item in the texture cache. All images that will
    // be used on a frame *must* have request() called on their
    // handle, to update the last used timestamp and ensure
    // that resources are not flushed from the cache too early.
    //
    // Returns true if the image needs to be uploaded to the
    // texture cache (either never uploaded, or has been
    // evicted on a previous frame).
    pub fn request(&mut self, handle: &TextureCacheHandle, gpu_cache: &mut GpuCache) -> bool {
        match self.lru_cache.touch(handle) {
            // If an image is requested that is already in the cache,
            // refresh the GPU cache data associated with this item.
            Some(entry) => {
                entry.last_access = self.now;
                entry.update_gpu_cache(gpu_cache);
                false
            }
            None => true,
        }
    }

    // Returns true if the image needs to be uploaded to the
    // texture cache (either never uploaded, or has been
    // evicted on a previous frame).
    pub fn needs_upload(&self, handle: &TextureCacheHandle) -> bool {
        self.lru_cache.get_opt(handle).is_none()
    }

    pub fn max_texture_size(&self) -> i32 {
        self.max_texture_size
    }

    #[cfg(feature = "replay")]
    pub fn max_texture_layers(&self) -> usize {
        self.max_texture_layers
    }

    #[cfg(feature = "replay")]
    pub fn picture_tile_sizes(&self) -> Vec<DeviceIntSize> {
        self.picture_textures.tile_sizes()
    }

    #[cfg(feature = "replay")]
    pub fn color_formats(&self) -> TextureFormatPair<ImageFormat> {
        self.shared_textures.array_color8_linear.formats.clone()
    }

    #[cfg(feature = "replay")]
    pub fn swizzle_settings(&self) -> Option<SwizzleSettings> {
        self.swizzle
    }

    #[cfg(feature = "replay")]
    pub fn eviction_threshold_bytes(&self) -> usize {
        self.eviction_threshold_bytes
    }

    #[cfg(feature = "replay")]
    pub fn max_evictions_per_frame(&self) -> usize {
        self.max_evictions_per_frame
    }

    pub fn pending_updates(&mut self) -> TextureUpdateList {
        mem::replace(&mut self.pending_updates, TextureUpdateList::new())
    }

    // Update the data stored by a given texture cache handle.
    pub fn update(
        &mut self,
        handle: &mut TextureCacheHandle,
        descriptor: ImageDescriptor,
        filter: TextureFilter,
        data: Option<CachedImageData>,
        user_data: [f32; 3],
        mut dirty_rect: ImageDirtyRect,
        gpu_cache: &mut GpuCache,
        eviction_notice: Option<&EvictionNotice>,
        uv_rect_kind: UvRectKind,
        eviction: Eviction,
    ) {
        debug_assert!(self.now.is_valid());

        // Determine if we need to allocate texture cache memory
        // for this item. We need to reallocate if any of the following
        // is true:
        // - Never been in the cache
        // - Has been in the cache but was evicted.
        // - Exists in the cache but dimensions / format have changed.
        let realloc = match self.lru_cache.get_opt(handle) {
            Some(entry) => {
                entry.size != descriptor.size || (entry.input_format != descriptor.format &&
                    entry.alternative_input_format() != descriptor.format)
            }
            None => {
                // Not allocated, or was previously allocated but has been evicted.
                true
            }
        };

        if realloc {
            let params = CacheAllocParams { descriptor, filter, user_data, uv_rect_kind };
            self.allocate(&params, handle);

            // If we reallocated, we need to upload the whole item again.
            dirty_rect = DirtyRect::All;
        }

        // Update eviction policy (this is a no-op if it hasn't changed)
        if eviction == Eviction::Manual {
            if let Some(manual_handle) = self.lru_cache.set_manual_eviction(handle) {
                self.manual_handles.push(manual_handle);
            }
        }

        let entry = self.lru_cache.get_opt_mut(handle)
            .expect("BUG: handle must be valid now");

        // Install the new eviction notice for this update, if applicable.
        entry.eviction_notice = eviction_notice.cloned();
        entry.uv_rect_kind = uv_rect_kind;

        // Invalidate the contents of the resource rect in the GPU cache.
        // This ensures that the update_gpu_cache below will add
        // the new information to the GPU cache.
        //TODO: only invalidate if the parameters change?
        gpu_cache.invalidate(&entry.uv_rect_handle);

        // Upload the resource rect and texture array layer.
        entry.update_gpu_cache(gpu_cache);

        // Create an update command, which the render thread processes
        // to upload the new image data into the correct location
        // in GPU memory.
        if let Some(data) = data {
            // If the swizzling is supported, we always upload in the internal
            // texture format (thus avoiding the conversion by the driver).
            // Otherwise, pass the external format to the driver.
            let use_upload_format = self.swizzle.is_none();
            let (layer_index, origin) = entry.details.describe();
            let op = TextureCacheUpdate::new_update(
                data,
                &descriptor,
                origin,
                entry.size,
                layer_index as i32,
                use_upload_format,
                &dirty_rect,
            );
            self.pending_updates.push_update(entry.texture_id, op);
        }
    }

    // Check if a given texture handle has a valid allocation
    // in the texture cache.
    pub fn is_allocated(&self, handle: &TextureCacheHandle) -> bool {
        self.lru_cache.get_opt(handle).is_some()
    }

    // Check if a given texture handle was last used as recently
    // as the specified number of previous frames.
    pub fn is_recently_used(&self, handle: &TextureCacheHandle, margin: usize) -> bool {
        self.lru_cache.get_opt(handle).map_or(false, |entry| {
            entry.last_access.frame_id() + margin >= self.now.frame_id()
        })
    }

    // Return the allocated size of the texture handle's associated data,
    // or otherwise indicate the handle is invalid.
    pub fn get_allocated_size(&self, handle: &TextureCacheHandle) -> Option<usize> {
        self.lru_cache.get_opt(handle).map(|entry| {
            (entry.input_format.bytes_per_pixel() * entry.size.area()) as usize
        })
    }

    // Retrieve the details of an item in the cache. This is used
    // during batch creation to provide the resource rect address
    // to the shaders and texture ID to the batching logic.
    // This function will assert in debug modes if the caller
    // tries to get a handle that was not requested this frame.
    pub fn get(&self, handle: &TextureCacheHandle) -> CacheItem {
        let (texture_id, layer_index, uv_rect, swizzle, uv_rect_handle) = self.get_cache_location(handle);
        CacheItem {
            uv_rect_handle,
            texture_id: TextureSource::TextureCache(texture_id, swizzle),
            uv_rect,
            texture_layer: layer_index as i32,
        }
    }

    /// A more detailed version of get(). This allows access to the actual
    /// device rect of the cache allocation.
    ///
    /// Returns a tuple identifying the texture, the layer, the region,
    /// and its GPU handle.
    pub fn get_cache_location(
        &self,
        handle: &TextureCacheHandle,
    ) -> (CacheTextureId, LayerIndex, DeviceIntRect, Swizzle, GpuCacheHandle) {
        let entry = self.lru_cache
            .get_opt(handle)
            .expect("BUG: was dropped from cache or not updated!");
        debug_assert_eq!(entry.last_access, self.now);
        let (layer_index, origin) = entry.details.describe();
        (entry.texture_id,
         layer_index as usize,
         DeviceIntRect::new(origin, entry.size),
         entry.swizzle,
         entry.uv_rect_handle)
    }

    /// Internal helper function to evict a strong texture cache handle
    fn evict_impl(
        &mut self,
        handle: FreeListHandle<CacheEntryMarker>,
    ) {
        let entry = self.lru_cache.remove_manual_handle(handle);
        entry.evict();
        self.free(&entry);
    }

    /// Evict a texture cache handle that was previously set to be in manual
    /// eviction mode.
    pub fn evict_manual_handle(&mut self, handle: &TextureCacheHandle) {
        // Find the strong handle that matches this weak handle. If this
        // ever shows up in profiles, we can make it a hash (but the number
        // of manual eviction handles is typically small).
        let index = self.manual_handles.iter().position(|strong_handle| {
            strong_handle.matches(handle)
        });

        if let Some(index) = index {
            let handle = self.manual_handles.swap_remove(index);
            self.evict_impl(handle);
        }
    }

    /// Expire picture cache tiles that haven't been referenced in the last frame.
    /// The picture cache code manually keeps tiles alive by calling `request` on
    /// them if it wants to retain a tile that is currently not visible.
    fn expire_old_picture_cache_tiles(&mut self) {
        for i in (0 .. self.picture_cache_handles.len()).rev() {
            let evict = {
                let entry = self.lru_cache.get(&self.picture_cache_handles[i]);

                // Texture cache entries can be evicted at the start of
                // a frame, or at any time during the frame when a cache
                // allocation is occurring. This means that entries tagged
                // with eager eviction may get evicted before they have a
                // chance to be requested on the current frame. Instead,
                // advance the frame id of the entry by one before
                // comparison. This ensures that an eager entry will
                // not be evicted until it is not used for at least
                // one complete frame.
                let mut entry_frame_id = entry.last_access.frame_id();
                entry_frame_id.advance();

                entry_frame_id < self.now.frame_id()
            };

            if evict {
                let handle = self.picture_cache_handles.swap_remove(i);
                self.evict_impl(handle);
            }
        }
    }

    /// Evict old items from the shared and standalone caches, if we're over a
    /// threshold memory usage value
    fn evict_items_from_cache_if_required(&mut self) {
        let mut eviction_count = 0;

        // Keep evicting while memory is above the threshold, and we haven't
        // reached a maximum number of evictions this frame.
        while self.current_memory_estimate() > self.eviction_threshold_bytes && eviction_count < self.max_evictions_per_frame {
            match self.lru_cache.pop_oldest() {
                Some(entry) => {
                    entry.evict();
                    self.free(&entry);
                    eviction_count += 1;
                }
                None => {
                    // It's possible that we could fail to pop an item from the LRU list to evict, if every
                    // item in the cache is set to manual eviction mode. In this case, just break out of the
                    // loop as there's nothing we can do until the calling code manually evicts items to
                    // reduce the allocated cache size.
                    break;
                }
            }
        }
    }

    /// Return the total used bytes in standalone and shared textures. This is
    /// used to determine how many textures need to be evicted to keep texture
    /// cache memory usage under a reasonable limit. Note that this does not
    /// include memory allocated to picture cache tiles, which are considered
    /// separately for the purposes of texture cache eviction.
    fn current_memory_estimate(&self) -> usize {
        self.standalone_bytes_allocated + self.shared_bytes_allocated
    }

    // Free a cache entry from the standalone list or shared cache.
    fn free(&mut self, entry: &CacheEntry) {
        match entry.details {
            EntryDetails::Picture { texture_index, layer_index } => {
                let picture_texture = self.picture_textures.get(texture_index);
                picture_texture.slices[layer_index].uv_rect_handle = None;
                if self.debug_flags.contains(
                    DebugFlags::TEXTURE_CACHE_DBG |
                    DebugFlags::TEXTURE_CACHE_DBG_CLEAR_EVICTED)
                {
                    self.pending_updates.push_debug_clear(
                        entry.texture_id,
                        DeviceIntPoint::zero(),
                        picture_texture.size.width,
                        picture_texture.size.height,
                        layer_index,
                    );
                }
            }
            EntryDetails::Standalone { size_in_bytes, .. } => {
                self.standalone_bytes_allocated -= size_in_bytes;

                // This is a standalone texture allocation. Free it directly.
                self.pending_updates.push_free(entry.texture_id);
            }
            EntryDetails::Cache { origin, layer_index, .. } => {
                // Free the block in the given region.
                let texture_array = self.shared_textures.select(entry.input_format, entry.filter);
                let unit = texture_array.units
                    .iter_mut()
                    .find(|unit| unit.texture_id == entry.texture_id)
                    .expect("Unable to find the associated texture array unit");
                let region = &mut unit.regions[layer_index];

                self.shared_bytes_allocated -= region.slab_size.size_in_bytes(texture_array.formats.internal);

                if self.debug_flags.contains(
                    DebugFlags::TEXTURE_CACHE_DBG |
                    DebugFlags::TEXTURE_CACHE_DBG_CLEAR_EVICTED)
                {
                    self.pending_updates.push_debug_clear(
                        entry.texture_id,
                        origin,
                        region.slab_size.width,
                        region.slab_size.height,
                        layer_index,
                    );
                }
                region.free(origin, &mut unit.empty_regions);
            }
        }
    }

    /// Allocate a block from the shared cache.
    fn allocate_from_shared_cache(
        &mut self,
        params: &CacheAllocParams,
    ) -> CacheEntry {
        // Mutably borrow the correct texture.
        let texture_array = self.shared_textures.select(
            params.descriptor.format,
            params.filter,
        );
        let swizzle = if texture_array.formats.external == params.descriptor.format {
            Swizzle::default()
        } else {
            match self.swizzle {
                Some(_) => Swizzle::Bgra,
                None => Swizzle::default(),
            }
        };

        let max_texture_layers = self.max_texture_layers;
        let slab_size = SlabSize::new(params.descriptor.size);

        let mut info = TextureCacheAllocInfo {
            width: TEXTURE_REGION_DIMENSIONS,
            height: TEXTURE_REGION_DIMENSIONS,
            format: texture_array.formats.internal,
            filter: texture_array.filter,
            layer_count: 1,
            is_shared_cache: true,
            has_depth: false,
        };

        let unit_index = if let Some(index) = texture_array.units
            .iter()
            .position(|unit| unit.can_alloc(slab_size))
        {
            index
        } else if let Some(index) = texture_array.units
            .iter()
            .position(|unit| unit.regions.len() < max_texture_layers)
        {
            let unit = &mut texture_array.units[index];

            unit.push_regions(texture_array.layers_per_allocation);

            info.layer_count = unit.regions.len() as i32;
            self.pending_updates.push_realloc(unit.texture_id, info);

            index
        } else {
            let index = texture_array.units.len();
            texture_array.units.push(TextureArrayUnit {
                texture_id: self.next_id,
                regions: Vec::new(),
                empty_regions: 0,
            });

            let unit = &mut texture_array.units[index];

            unit.push_regions(texture_array.layers_per_allocation);

            info.layer_count = unit.regions.len() as i32;
            self.pending_updates.push_alloc(self.next_id, info);
            self.next_id.0 += 1;
            index
        };

        self.shared_bytes_allocated += slab_size.size_in_bytes(texture_array.formats.internal);

        // Do the allocation. This can fail and return None
        // if there are no free slots or regions available.
        texture_array.alloc(params, unit_index, self.now, swizzle)
    }

    // Returns true if the given image descriptor *may* be
    // placed in the shared texture cache.
    pub fn is_allowed_in_shared_cache(
        &self,
        filter: TextureFilter,
        descriptor: &ImageDescriptor,
    ) -> bool {
        let mut allowed_in_shared_cache = true;

        // Anything larger than TEXTURE_REGION_DIMENSIONS goes in a standalone texture.
        // TODO(gw): If we find pages that suffer from batch breaks in this
        //           case, add support for storing these in a standalone
        //           texture array.
        if descriptor.size.width > TEXTURE_REGION_DIMENSIONS ||
           descriptor.size.height > TEXTURE_REGION_DIMENSIONS
        {
            allowed_in_shared_cache = false;
        }

        // TODO(gw): For now, alpha formats of the texture cache can only be linearly sampled.
        //           Nearest sampling gets a standalone texture.
        //           This is probably rare enough that it can be fixed up later.
        if filter == TextureFilter::Nearest &&
           descriptor.format.bytes_per_pixel() <= 2
        {
            allowed_in_shared_cache = false;
        }

        allowed_in_shared_cache
    }

    /// Allocates a new standalone cache entry.
    fn allocate_standalone_entry(
        &mut self,
        params: &CacheAllocParams,
    ) -> CacheEntry {
        let texture_id = self.next_id;
        self.next_id.0 += 1;

        // Push a command to allocate device storage of the right size / format.
        let info = TextureCacheAllocInfo {
            width: params.descriptor.size.width,
            height: params.descriptor.size.height,
            format: params.descriptor.format,
            filter: params.filter,
            layer_count: 1,
            is_shared_cache: false,
            has_depth: false,
        };

        let size_in_bytes = (info.width * info.height * info.format.bytes_per_pixel()) as usize;
        self.standalone_bytes_allocated += size_in_bytes;

        self.pending_updates.push_alloc(texture_id, info);

        // Special handing for BGRA8 textures that may need to be swizzled.
        let swizzle = if params.descriptor.format == ImageFormat::BGRA8 {
            self.swizzle.map(|s| s.bgra8_sampling_swizzle)
        } else {
            None
        };

        CacheEntry::new_standalone(
            texture_id,
            self.now,
            params,
            swizzle.unwrap_or_default(),
            size_in_bytes,
        )
    }

    /// Allocates a cache entry appropriate for the given parameters.
    ///
    /// This allocates from the shared cache unless the parameters do not meet
    /// the shared cache requirements, in which case a standalone texture is
    /// used.
    fn allocate_cache_entry(
        &mut self,
        params: &CacheAllocParams,
    ) -> CacheEntry {
        assert!(!params.descriptor.size.is_empty());

        // If this image doesn't qualify to go in the shared (batching) cache,
        // allocate a standalone entry.
        if self.is_allowed_in_shared_cache(params.filter, &params.descriptor) {
            self.allocate_from_shared_cache(params)
        } else {
            self.allocate_standalone_entry(params)
        }
    }

    /// Allocates a cache entry for the given parameters, and updates the
    /// provided handle to point to the new entry.
    fn allocate(&mut self, params: &CacheAllocParams, handle: &mut TextureCacheHandle) {
        debug_assert!(self.now.is_valid());
        let new_cache_entry = self.allocate_cache_entry(params);

        // If the handle points to a valid cache entry, we want to replace the
        // cache entry with our newly updated location. We also need to ensure
        // that the storage (region or standalone) associated with the previous
        // entry here gets freed.
        //
        // If the handle is invalid, we need to insert the data, and append the
        // result to the corresponding vector.
        if let Some(old_entry) = self.lru_cache.replace_or_insert(handle, new_cache_entry) {
            old_entry.evict();
            self.free(&old_entry);
        }
    }

    // Update the data stored by a given texture cache handle for picture caching specifically.
    pub fn update_picture_cache(
        &mut self,
        tile_size: DeviceIntSize,
        handle: &mut TextureCacheHandle,
        gpu_cache: &mut GpuCache,
    ) {
        debug_assert!(self.now.is_valid());
        debug_assert!(tile_size.width > 0 && tile_size.height > 0);

        if self.lru_cache.get_opt(handle).is_none() {
            let cache_entry = self.picture_textures.get_or_allocate_tile(
                tile_size,
                self.now,
                &mut self.next_id,
                &mut self.pending_updates,
            );

            // Add the cache entry to the LRU cache, then mark it for manual eviction
            // so that the lifetime is controlled by the texture cache.

            *handle = self.lru_cache.push_new(cache_entry);

            let strong_handle = self.lru_cache
                .set_manual_eviction(handle)
                .expect("bug: handle must be valid here");
            self.picture_cache_handles.push(strong_handle);
        }

        // Upload the resource rect and texture array layer.
        self.lru_cache
            .get_opt_mut(handle)
            .expect("BUG: handle must be valid now")
            .update_gpu_cache(gpu_cache);
    }

    pub fn shared_alpha_expected_format(&self) -> ImageFormat {
        self.shared_textures.array_alpha8_linear.formats.external
    }

    pub fn shared_color_expected_format(&self) -> ImageFormat {
        self.shared_textures.array_color8_linear.formats.external
    }
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Copy, Clone, PartialEq)]
struct SlabSize {
    width: i32,
    height: i32,
}

impl SlabSize {
    fn new(size: DeviceIntSize) -> Self {
        let x_size = quantize_dimension(size.width);
        let y_size = quantize_dimension(size.height);

        assert!(x_size > 0 && x_size <= TEXTURE_REGION_DIMENSIONS);
        assert!(y_size > 0 && y_size <= TEXTURE_REGION_DIMENSIONS);

        let (width, height) = match (x_size, y_size) {
            // Special cased rectangular slab pages.
            (512, 0..=64) => (512, 64),
            (512, 128) => (512, 128),
            (512, 256) => (512, 256),
            (0..=64, 512) => (64, 512),
            (128, 512) => (128, 512),
            (256, 512) => (256, 512),

            // If none of those fit, use a square slab size.
            (x_size, y_size) => {
                let square_size = cmp::max(x_size, y_size);
                (square_size, square_size)
            }
        };

        SlabSize {
            width,
            height,
        }
    }

    fn size_in_bytes(&self, format: ImageFormat) -> usize {
        let bpp = format.bytes_per_pixel();
        (self.width * self.height * bpp) as usize
    }

    fn invalid() -> SlabSize {
        SlabSize {
            width: 0,
            height: 0,
        }
    }
}

// The x/y location within a texture region of an allocation.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
struct TextureLocation(u8, u8);

impl TextureLocation {
    fn new(x: i32, y: i32) -> Self {
        debug_assert!(x >= 0 && y >= 0 && x < 0x100 && y < 0x100);
        TextureLocation(x as u8, y as u8)
    }
}

/// A region corresponds to a layer in a shared cache texture.
///
/// All allocations within a region are of the same size.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
struct TextureRegion {
    layer_index: usize,
    slab_size: SlabSize,
    free_slots: Vec<TextureLocation>,
    total_slot_count: usize,
}

impl TextureRegion {
    fn new(layer_index: usize) -> Self {
        TextureRegion {
            layer_index,
            slab_size: SlabSize::invalid(),
            free_slots: Vec::new(),
            total_slot_count: 0,
        }
    }

    // Initialize a region to be an allocator for a specific slab size.
    fn init(&mut self, slab_size: SlabSize, empty_regions: &mut usize) {
        debug_assert!(self.slab_size == SlabSize::invalid());
        debug_assert!(self.free_slots.is_empty());

        self.slab_size = slab_size;
        let slots_per_x_axis = TEXTURE_REGION_DIMENSIONS / self.slab_size.width;
        let slots_per_y_axis = TEXTURE_REGION_DIMENSIONS / self.slab_size.height;

        // Add each block to a freelist.
        for y in 0 .. slots_per_y_axis {
            for x in 0 .. slots_per_x_axis {
                self.free_slots.push(TextureLocation::new(x, y));
            }
        }

        self.total_slot_count = self.free_slots.len();
        *empty_regions -= 1;
    }

    // Deinit a region, allowing it to become a region with
    // a different allocator size.
    fn deinit(&mut self, empty_regions: &mut usize) {
        self.slab_size = SlabSize::invalid();
        self.free_slots.clear();
        self.total_slot_count = 0;
        *empty_regions += 1;
    }

    fn is_empty(&self) -> bool {
        self.slab_size == SlabSize::invalid()
    }

    // Attempt to allocate a fixed size block from this region.
    fn alloc(&mut self) -> Option<DeviceIntPoint> {
        debug_assert!(self.slab_size != SlabSize::invalid());

        self.free_slots.pop().map(|location| {
            DeviceIntPoint::new(
                self.slab_size.width * location.0 as i32,
                self.slab_size.height * location.1 as i32,
            )
        })
    }

    // Free a block in this region.
    fn free(&mut self, point: DeviceIntPoint, empty_regions: &mut usize) {
        let x = point.x / self.slab_size.width;
        let y = point.y / self.slab_size.height;
        self.free_slots.push(TextureLocation::new(x, y));

        // If this region is completely unused, deinit it
        // so that it can become a different slab size
        // as required.
        if self.free_slots.len() == self.total_slot_count {
            self.deinit(empty_regions);
        }
    }
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
struct TextureArrayUnit {
    texture_id: CacheTextureId,
    regions: Vec<TextureRegion>,
    empty_regions: usize,
}

impl TextureArrayUnit {
    /// Adds a new empty region to the array.
    fn push_regions(&mut self, count: i32) {
        assert!(self.empty_regions <= self.regions.len());
        for _ in 0..count {
            let index = self.regions.len();
            self.regions.push(TextureRegion::new(index));
            self.empty_regions += 1;
        }
    }

    /// Returns true if we can allocate the given entry.
    fn can_alloc(&self, slab_size: SlabSize) -> bool {
        self.empty_regions != 0 || self.regions.iter().any(|region| {
            region.slab_size == slab_size && !region.free_slots.is_empty()
        })
    }

    fn is_empty(&self) -> bool {
        self.empty_regions == self.regions.len()
    }
}

/// A texture array contains a number of textures, each with a number of
/// layers, where each layer contains a region that can act as a slab allocator.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
struct TextureArray {
    filter: TextureFilter,
    formats: TextureFormatPair<ImageFormat>,
    units: SmallVec<[TextureArrayUnit; 1]>,
    layers_per_allocation: i32,
}

impl TextureArray {
    fn new(
        formats: TextureFormatPair<ImageFormat>,
        filter: TextureFilter,
        layers_per_allocation: i32,
    ) -> Self {
        TextureArray {
            formats,
            filter,
            units: SmallVec::new(),
            layers_per_allocation,
        }
    }

    /// Returns the number of GPU bytes consumed by this texture array.
    fn size_in_bytes(&self) -> usize {
        let bpp = self.formats.internal.bytes_per_pixel() as usize;
        let num_regions: usize = self.units.iter().map(|u| u.regions.len()).sum();
        num_regions * TEXTURE_REGION_PIXELS * bpp
    }

    fn clear(&mut self, updates: &mut TextureUpdateList) {
        for unit in self.units.drain(..) {
            updates.push_free(unit.texture_id);
        }
    }

    fn release_empty_textures(&mut self, updates: &mut TextureUpdateList) {
        self.units.retain(|unit| {
            if unit.is_empty() {
                updates.push_free(unit.texture_id);

                false
            } else {
                true
            }
        });
    }

    fn update_profile(&self, counter: &mut ResourceProfileCounter) {
        let num_regions: usize = self.units.iter().map(|u| u.regions.len()).sum();
        counter.set(num_regions, self.size_in_bytes());
    }

    /// Allocate space in this texture array.
    fn alloc(
        &mut self,
        params: &CacheAllocParams,
        unit_index: usize,
        now: FrameStamp,
        swizzle: Swizzle,
    ) -> CacheEntry {
        // Quantize the size of the allocation to select a region to
        // allocate from.
        let slab_size = SlabSize::new(params.descriptor.size);
        let unit = &mut self.units[unit_index];

        // TODO(gw): For simplicity, the initial implementation just
        //           has a single vec<> of regions. We could easily
        //           make this more efficient by storing a list of
        //           regions for each slab size specifically...

        // Keep track of the location of an empty region,
        // in case we need to select a new empty region
        // after the loop.
        let mut empty_region_index = None;
        let mut entry_details = None;

        // Run through the existing regions of this size, and see if
        // we can find a free block in any of them.
        for (i, region) in unit.regions.iter_mut().enumerate() {
            if region.is_empty() {
                empty_region_index = Some(i);
            } else if region.slab_size == slab_size {
                if let Some(location) = region.alloc() {
                    entry_details = Some(EntryDetails::Cache {
                        layer_index: region.layer_index,
                        origin: location,
                    });
                    break;
                }
            }
        }

        // Find a region of the right size and try to allocate from it.
        let details = match entry_details {
            Some(details) => details,
            None => {
                let region = &mut unit.regions[empty_region_index.unwrap()];
                region.init(slab_size, &mut unit.empty_regions);
                EntryDetails::Cache {
                    layer_index: region.layer_index,
                    origin: region.alloc().unwrap(),
                }
            }
        };

        CacheEntry {
            size: params.descriptor.size,
            user_data: params.user_data,
            last_access: now,
            details,
            uv_rect_handle: GpuCacheHandle::new(),
            input_format: params.descriptor.format,
            filter: self.filter,
            swizzle,
            texture_id: unit.texture_id,
            eviction_notice: None,
            uv_rect_kind: params.uv_rect_kind,
        }
    }
}


/// A tracking structure for each slice in `WholeTextureArray`.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Clone, Copy, Debug)]
struct WholeTextureSlice {
    uv_rect_handle: Option<GpuCacheHandle>,
}

/// A texture array that allocates whole slices and doesn't do any region tracking.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
struct WholeTextureArray {
    size: DeviceIntSize,
    filter: TextureFilter,
    format: ImageFormat,
    texture_id: CacheTextureId,
    slices: Vec<WholeTextureSlice>,
    has_depth: bool,
}

impl WholeTextureArray {
    fn to_info(&self) -> TextureCacheAllocInfo {
        TextureCacheAllocInfo {
            width: self.size.width,
            height: self.size.height,
            format: self.format,
            filter: self.filter,
            layer_count: self.slices.len() as i32,
            is_shared_cache: true, //TODO: reconsider
            has_depth: self.has_depth,
        }
    }

    /// Returns the number of GPU bytes consumed by this texture array.
    fn size_in_bytes(&self) -> usize {
        let bpp = self.format.bytes_per_pixel() as usize;
        self.slices.len() * (self.size.width * self.size.height) as usize * bpp
    }

    /// Find an free slice.
    fn find_free(&self) -> Option<LayerIndex> {
        self.slices.iter().position(|slice| slice.uv_rect_handle.is_none())
    }

    /// Grow the array by the specified number of slices
    fn grow(&mut self, count: usize) -> LayerIndex {
        let index = self.slices.len();
        for _ in 0 .. count {
            self.slices.push(WholeTextureSlice {
                uv_rect_handle: None,
            });
        }
        index
    }

    fn cache_entry_impl(
        &self,
        texture_index: usize,
        layer_index: usize,
        now: FrameStamp,
        uv_rect_handle: GpuCacheHandle,
        texture_id: CacheTextureId,
    ) -> CacheEntry {
        CacheEntry {
            size: self.size,
            user_data: [0.0; 3],
            last_access: now,
            details: EntryDetails::Picture {
                texture_index,
                layer_index,
            },
            uv_rect_handle,
            input_format: self.format,
            filter: self.filter,
            swizzle: Swizzle::default(),
            texture_id,
            eviction_notice: None,
            uv_rect_kind: UvRectKind::Rect,
        }
    }

    /// Occupy a specified slice by a cache entry.
    fn occupy(
        &mut self,
        texture_index: usize,
        layer_index: usize,
        now: FrameStamp,
    ) -> CacheEntry {
        let uv_rect_handle = GpuCacheHandle::new();
        assert!(self.slices[layer_index].uv_rect_handle.is_none());
        self.slices[layer_index].uv_rect_handle = Some(uv_rect_handle);
        self.cache_entry_impl(
            texture_index,
            layer_index,
            now,
            uv_rect_handle,
            self.texture_id,
        )
    }

    /// Reset the texture array to the specified number of slices, if it's larger.
    fn reset(
        &mut self, num_slices: usize
    ) -> Option<CacheTextureId> {
        if self.slices.len() <= num_slices {
            None
        } else {
            self.slices.truncate(num_slices);
            Some(self.texture_id)
        }
    }
}


impl TextureCacheUpdate {
    // Constructs a TextureCacheUpdate operation to be passed to the
    // rendering thread in order to do an upload to the right
    // location in the texture cache.
    fn new_update(
        data: CachedImageData,
        descriptor: &ImageDescriptor,
        origin: DeviceIntPoint,
        size: DeviceIntSize,
        layer_index: i32,
        use_upload_format: bool,
        dirty_rect: &ImageDirtyRect,
    ) -> TextureCacheUpdate {
        let source = match data {
            CachedImageData::Blob => {
                panic!("The vector image should have been rasterized.");
            }
            CachedImageData::External(ext_image) => match ext_image.image_type {
                ExternalImageType::TextureHandle(_) => {
                    panic!("External texture handle should not go through texture_cache.");
                }
                ExternalImageType::Buffer => TextureUpdateSource::External {
                    id: ext_image.id,
                    channel_index: ext_image.channel_index,
                },
            },
            CachedImageData::Raw(bytes) => {
                let finish = descriptor.offset +
                    descriptor.size.width * descriptor.format.bytes_per_pixel() +
                    (descriptor.size.height - 1) * descriptor.compute_stride();
                assert!(bytes.len() >= finish as usize);

                TextureUpdateSource::Bytes { data: bytes }
            }
        };
        let format_override = if use_upload_format {
            Some(descriptor.format)
        } else {
            None
        };

        match *dirty_rect {
            DirtyRect::Partial(dirty) => {
                // the dirty rectangle doesn't have to be within the area but has to intersect it, at least
                let stride = descriptor.compute_stride();
                let offset = descriptor.offset + dirty.origin.y * stride + dirty.origin.x * descriptor.format.bytes_per_pixel();

                TextureCacheUpdate {
                    rect: DeviceIntRect::new(
                        DeviceIntPoint::new(origin.x + dirty.origin.x, origin.y + dirty.origin.y),
                        DeviceIntSize::new(
                            dirty.size.width.min(size.width - dirty.origin.x),
                            dirty.size.height.min(size.height - dirty.origin.y),
                        ),
                    ),
                    source,
                    stride: Some(stride),
                    offset,
                    format_override,
                    layer_index,
                }
            }
            DirtyRect::All => {
                TextureCacheUpdate {
                    rect: DeviceIntRect::new(origin, size),
                    source,
                    stride: descriptor.stride,
                    offset: descriptor.offset,
                    format_override,
                    layer_index,
                }
            }
        }
    }
}

fn quantize_dimension(size: i32) -> i32 {
    match size {
        0 => unreachable!(),
        1..=16 => 16,
        17..=32 => 32,
        33..=64 => 64,
        65..=128 => 128,
        129..=256 => 256,
        257..=512 => 512,
        _ => panic!("Invalid dimensions for cache!"),
    }
}
