/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use api::{BlobImageResources, BlobImageRequest, RasterizedBlobImage, ImageFormat};
use api::{DebugFlags, FontInstanceKey, FontKey, FontTemplate, GlyphIndex};
use api::{ExternalImageData, ExternalImageType, ExternalImageId, BlobImageResult, FontInstanceData};
use api::{DirtyRect, GlyphDimensions, IdNamespace, DEFAULT_TILE_SIZE};
use api::{ImageData, ImageDescriptor, ImageKey, ImageRendering, TileSize};
use api::{BlobImageKey, VoidPtrToSizeFn};
use api::{SharedFontInstanceMap, BaseFontInstance};
use api::units::*;
use crate::{render_api::{ClearCache, AddFont, ResourceUpdate, MemoryReport}, util::WeakTable};
use crate::image_tiling::{compute_tile_size, compute_tile_range};
#[cfg(feature = "capture")]
use crate::capture::ExternalCaptureImage;
#[cfg(feature = "replay")]
use crate::capture::PlainExternalImage;
#[cfg(any(feature = "replay", feature = "png", feature="capture"))]
use crate::capture::CaptureConfig;
use crate::composite::{NativeSurfaceId, NativeSurfaceOperation, NativeTileId, NativeSurfaceOperationDetails};
use crate::device::TextureFilter;
use crate::glyph_cache::GlyphCache;
use crate::glyph_cache::GlyphCacheEntry;
use crate::glyph_rasterizer::{GLYPH_FLASHING, FontInstance, GlyphFormat, GlyphKey, GlyphRasterizer};
use crate::gpu_cache::{GpuCache, GpuCacheAddress, GpuCacheHandle};
use crate::gpu_types::UvRectKind;
use crate::internal_types::{CacheTextureId, FastHashMap, FastHashSet, TextureSource, ResourceUpdateList};
use crate::picture::SurfaceInfo;
use crate::profiler::{self, TransactionProfile, bytes_to_mb};
use crate::render_backend::{FrameId, FrameStamp};
use crate::render_task_graph::{RenderTaskId, RenderTaskGraphBuilder};
use crate::render_task_cache::{RenderTaskCache, RenderTaskCacheKey, RenderTaskParent};
use crate::render_task_cache::{RenderTaskCacheEntry, RenderTaskCacheEntryHandle};
use euclid::point2;
use smallvec::SmallVec;
use std::collections::hash_map::Entry::{self, Occupied, Vacant};
use std::collections::hash_map::{Iter, IterMut};
use std::collections::VecDeque;
#[cfg(any(feature = "capture", feature = "replay"))]
use std::collections::HashMap;
use std::{cmp, mem};
use std::fmt::Debug;
use std::hash::Hash;
use std::os::raw::c_void;
#[cfg(any(feature = "capture", feature = "replay"))]
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::u32;
use crate::texture_cache::{TextureCache, TextureCacheHandle, Eviction, TargetShader};

// Counter for generating unique native surface ids
static NEXT_NATIVE_SURFACE_ID: AtomicUsize = AtomicUsize::new(0);

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct GlyphFetchResult {
    pub index_in_text_run: i32,
    pub uv_rect_address: GpuCacheAddress,
    pub offset: DevicePoint,
    pub size: DeviceIntSize,
    pub scale: f32,
}

// These coordinates are always in texels.
// They are converted to normalized ST
// values in the vertex shader. The reason
// for this is that the texture may change
// dimensions (e.g. the pages in a texture
// atlas can grow). When this happens, by
// storing the coordinates as texel values
// we don't need to go through and update
// various CPU-side structures.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct CacheItem {
    pub texture_id: TextureSource,
    pub uv_rect_handle: GpuCacheHandle,
    pub uv_rect: DeviceIntRect,
    pub user_data: [f32; 4],
}

impl CacheItem {
    pub fn invalid() -> Self {
        CacheItem {
            texture_id: TextureSource::Invalid,
            uv_rect_handle: GpuCacheHandle::new(),
            uv_rect: DeviceIntRect::zero(),
            user_data: [0.0; 4],
        }
    }

    pub fn is_valid(&self) -> bool {
        self.texture_id != TextureSource::Invalid
    }
}

/// Represents the backing store of an image in the cache.
/// This storage can take several forms.
#[derive(Clone, Debug)]
pub enum CachedImageData {
    /// A simple series of bytes, provided by the embedding and owned by WebRender.
    /// The format is stored out-of-band, currently in ImageDescriptor.
    Raw(Arc<Vec<u8>>),
    /// An series of commands that can be rasterized into an image via an
    /// embedding-provided callback.
    ///
    /// The commands are stored elsewhere and this variant is used as a placeholder.
    Blob,
    /// An image owned by the embedding, and referenced by WebRender. This may
    /// take the form of a texture or a heap-allocated buffer.
    External(ExternalImageData),
}

impl From<ImageData> for CachedImageData {
    fn from(img_data: ImageData) -> Self {
        match img_data {
            ImageData::Raw(data) => CachedImageData::Raw(data),
            ImageData::External(data) => CachedImageData::External(data),
        }
    }
}

impl CachedImageData {
    /// Returns true if this represents a blob.
    #[inline]
    pub fn is_blob(&self) -> bool {
        match *self {
            CachedImageData::Blob => true,
            _ => false,
        }
    }

    /// Returns true if this variant of CachedImageData should go through the texture
    /// cache.
    #[inline]
    pub fn uses_texture_cache(&self) -> bool {
        match *self {
            CachedImageData::External(ref ext_data) => match ext_data.image_type {
                ExternalImageType::TextureHandle(_) => false,
                ExternalImageType::Buffer => true,
            },
            CachedImageData::Blob => true,
            CachedImageData::Raw(_) => true,
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct ImageProperties {
    pub descriptor: ImageDescriptor,
    pub external_image: Option<ExternalImageData>,
    pub tiling: Option<TileSize>,
    // Potentially a subset of the image's total rectangle. This rectangle is what
    // we map to the (layout space) display item bounds.
    pub visible_rect: DeviceIntRect,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum State {
    Idle,
    AddResources,
    QueryResources,
}

/// Post scene building state.
type RasterizedBlob = FastHashMap<TileOffset, RasterizedBlobImage>;

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ImageGeneration(pub u32);

impl ImageGeneration {
    pub const INVALID: ImageGeneration = ImageGeneration(u32::MAX);
}

struct ImageResource {
    data: CachedImageData,
    descriptor: ImageDescriptor,
    tiling: Option<TileSize>,
    /// This is used to express images that are virtually very large
    /// but with only a visible sub-set that is valid at a given time.
    visible_rect: DeviceIntRect,
    generation: ImageGeneration,
}

#[derive(Clone, Debug)]
pub struct ImageTiling {
    pub image_size: DeviceIntSize,
    pub tile_size: TileSize,
}

#[derive(Default)]
struct ImageTemplates {
    images: FastHashMap<ImageKey, ImageResource>,
}

impl ImageTemplates {
    fn insert(&mut self, key: ImageKey, resource: ImageResource) {
        self.images.insert(key, resource);
    }

    fn remove(&mut self, key: ImageKey) -> Option<ImageResource> {
        self.images.remove(&key)
    }

    fn get(&self, key: ImageKey) -> Option<&ImageResource> {
        self.images.get(&key)
    }

    fn get_mut(&mut self, key: ImageKey) -> Option<&mut ImageResource> {
        self.images.get_mut(&key)
    }
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
struct CachedImageInfo {
    texture_cache_handle: TextureCacheHandle,
    dirty_rect: ImageDirtyRect,
    manual_eviction: bool,
}

impl CachedImageInfo {
    fn mark_unused(&mut self, texture_cache: &mut TextureCache) {
        texture_cache.evict_handle(&self.texture_cache_handle);
        self.manual_eviction = false;
    }
}

#[cfg(debug_assertions)]
impl Drop for CachedImageInfo {
    fn drop(&mut self) {
        debug_assert!(!self.manual_eviction, "Manual eviction requires cleanup");
    }
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct ResourceClassCache<K: Hash + Eq, V, U: Default> {
    resources: FastHashMap<K, V>,
    pub user_data: U,
}

impl<K, V, U> ResourceClassCache<K, V, U>
where
    K: Clone + Hash + Eq + Debug,
    U: Default,
{
    pub fn new() -> Self {
        ResourceClassCache {
            resources: FastHashMap::default(),
            user_data: Default::default(),
        }
    }

    pub fn get(&self, key: &K) -> &V {
        self.resources.get(key)
            .expect("Didn't find a cached resource with that ID!")
    }

    pub fn try_get(&self, key: &K) -> Option<&V> {
        self.resources.get(key)
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.resources.insert(key, value);
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.resources.remove(key)
    }

    pub fn get_mut(&mut self, key: &K) -> &mut V {
        self.resources.get_mut(key)
            .expect("Didn't find a cached resource with that ID!")
    }

    pub fn try_get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.resources.get_mut(key)
    }

    pub fn entry(&mut self, key: K) -> Entry<K, V> {
        self.resources.entry(key)
    }

    pub fn iter(&self) -> Iter<K, V> {
        self.resources.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<K, V> {
        self.resources.iter_mut()
    }

    pub fn is_empty(&mut self) -> bool {
        self.resources.is_empty()
    }

    pub fn clear(&mut self) {
        self.resources.clear();
    }

    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&K, &mut V) -> bool,
    {
        self.resources.retain(f);
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
struct CachedImageKey {
    pub rendering: ImageRendering,
    pub tile: Option<TileOffset>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct ImageRequest {
    pub key: ImageKey,
    pub rendering: ImageRendering,
    pub tile: Option<TileOffset>,
}

impl ImageRequest {
    pub fn with_tile(&self, offset: TileOffset) -> Self {
        ImageRequest {
            key: self.key,
            rendering: self.rendering,
            tile: Some(offset),
        }
    }

    pub fn is_untiled_auto(&self) -> bool {
        self.tile.is_none() && self.rendering == ImageRendering::Auto
    }
}

impl Into<BlobImageRequest> for ImageRequest {
    fn into(self) -> BlobImageRequest {
        BlobImageRequest {
            key: BlobImageKey(self.key),
            tile: self.tile.unwrap(),
        }
    }
}

impl Into<CachedImageKey> for ImageRequest {
    fn into(self) -> CachedImageKey {
        CachedImageKey {
            rendering: self.rendering,
            tile: self.tile,
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "capture", derive(Clone, Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub enum ImageCacheError {
    OverLimitSize,
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
enum ImageResult {
    UntiledAuto(CachedImageInfo),
    Multi(ResourceClassCache<CachedImageKey, CachedImageInfo, ()>),
    Err(ImageCacheError),
}

impl ImageResult {
    /// Releases any texture cache entries held alive by this ImageResult.
    fn drop_from_cache(&mut self, texture_cache: &mut TextureCache) {
        match *self {
            ImageResult::UntiledAuto(ref mut entry) => {
                entry.mark_unused(texture_cache);
            },
            ImageResult::Multi(ref mut entries) => {
                for entry in entries.resources.values_mut() {
                    entry.mark_unused(texture_cache);
                }
            },
            ImageResult::Err(_) => {},
        }
    }
}

type ImageCache = ResourceClassCache<ImageKey, ImageResult, ()>;

struct Resources {
    font_templates: FastHashMap<FontKey, FontTemplate>,
    font_instances: SharedFontInstanceMap,
    image_templates: ImageTemplates,
    // We keep a set of Weak references to the fonts so that we're able to include them in memory
    // reports even if only the OS is holding on to the Vec<u8>. PtrWeakHashSet will periodically
    // drop any references that have gone dead.
    weak_fonts: WeakTable
}

impl BlobImageResources for Resources {
    fn get_font_data(&self, key: FontKey) -> &FontTemplate {
        self.font_templates.get(&key).unwrap()
    }
    fn get_font_instance_data(&self, key: FontInstanceKey) -> Option<FontInstanceData> {
        self.font_instances.get_font_instance_data(key)
    }
}

// We only use this to report glyph dimensions to the user of the API, so using
// the font instance key should be enough. If we start using it to cache dimensions
// for internal font instances we should change the hash key accordingly.
pub type GlyphDimensionsCache = FastHashMap<(FontInstanceKey, GlyphIndex), Option<GlyphDimensions>>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlobImageRasterizerEpoch(usize);

/// Internal information about allocated render targets in the pool
struct RenderTarget {
    size: DeviceIntSize,
    format: ImageFormat,
    texture_id: CacheTextureId,
    /// If true, this is currently leant out, and not available to other passes
    is_active: bool,
    last_frame_used: FrameId,
}

impl RenderTarget {
    fn size_in_bytes(&self) -> usize {
        let bpp = self.format.bytes_per_pixel() as usize;
        (self.size.width * self.size.height) as usize * bpp
    }

    /// Returns true if this texture was used within `threshold` frames of
    /// the current frame.
    pub fn used_recently(&self, current_frame_id: FrameId, threshold: usize) -> bool {
        self.last_frame_used + threshold >= current_frame_id
    }
}

/// High-level container for resources managed by the `RenderBackend`.
///
/// This includes a variety of things, including images, fonts, and glyphs,
/// which may be stored as memory buffers, GPU textures, or handles to resources
/// managed by the OS or other parts of WebRender.
pub struct ResourceCache {
    cached_glyphs: GlyphCache,
    cached_images: ImageCache,
    cached_render_tasks: RenderTaskCache,

    resources: Resources,
    state: State,
    current_frame_id: FrameId,

    #[cfg(feature = "capture")]
    /// Used for capture sequences. If the resource cache is updated, then we
    /// mark it as dirty. When the next frame is captured in the sequence, we
    /// dump the state of the resource cache.
    capture_dirty: bool,

    pub texture_cache: TextureCache,

    /// TODO(gw): We should expire (parts of) this cache semi-regularly!
    cached_glyph_dimensions: GlyphDimensionsCache,
    glyph_rasterizer: GlyphRasterizer,

    /// The set of images that aren't present or valid in the texture cache,
    /// and need to be rasterized and/or uploaded this frame. This includes
    /// both blobs and regular images.
    pending_image_requests: FastHashSet<ImageRequest>,

    rasterized_blob_images: FastHashMap<BlobImageKey, RasterizedBlob>,

    /// A log of the last three frames worth of deleted image keys kept
    /// for debugging purposes.
    deleted_blob_keys: VecDeque<Vec<BlobImageKey>>,

    /// A list of queued compositor surface updates to apply next frame.
    pending_native_surface_updates: Vec<NativeSurfaceOperation>,

    image_templates_memory: usize,
    font_templates_memory: usize,

    /// A pool of render targets for use by the render task graph
    render_target_pool: Vec<RenderTarget>,
}

impl ResourceCache {
    pub fn new(
        texture_cache: TextureCache,
        glyph_rasterizer: GlyphRasterizer,
        cached_glyphs: GlyphCache,
        font_instances: SharedFontInstanceMap,
    ) -> Self {
        ResourceCache {
            cached_glyphs,
            cached_images: ResourceClassCache::new(),
            cached_render_tasks: RenderTaskCache::new(),
            resources: Resources {
                font_instances,
                font_templates: FastHashMap::default(),
                image_templates: ImageTemplates::default(),
                weak_fonts: WeakTable::new(),
            },
            cached_glyph_dimensions: FastHashMap::default(),
            texture_cache,
            state: State::Idle,
            current_frame_id: FrameId::INVALID,
            pending_image_requests: FastHashSet::default(),
            glyph_rasterizer,
            rasterized_blob_images: FastHashMap::default(),
            // We want to keep three frames worth of delete blob keys
            deleted_blob_keys: vec![Vec::new(), Vec::new(), Vec::new()].into(),
            pending_native_surface_updates: Vec::new(),
            #[cfg(feature = "capture")]
            capture_dirty: true,
            image_templates_memory: 0,
            font_templates_memory: 0,
            render_target_pool: Vec::new(),
        }
    }

    /// Construct a resource cache for use in unit tests.
    #[cfg(test)]
    pub fn new_for_testing() -> Self {
        use rayon::ThreadPoolBuilder;

        let texture_cache = TextureCache::new_for_testing(
            4096,
            ImageFormat::RGBA8,
        );
        let workers = Arc::new(ThreadPoolBuilder::new().build().unwrap());
        let glyph_rasterizer = GlyphRasterizer::new(workers, true).unwrap();
        let cached_glyphs = GlyphCache::new();
        let font_instances = SharedFontInstanceMap::new();

        ResourceCache::new(
            texture_cache,
            glyph_rasterizer,
            cached_glyphs,
            font_instances,
        )
    }

    pub fn max_texture_size(&self) -> i32 {
        self.texture_cache.max_texture_size()
    }

    /// Maximum texture size before we consider it preferrable to break the texture
    /// into tiles.
    pub fn tiling_threshold(&self) -> i32 {
        self.texture_cache.tiling_threshold()
    }

    pub fn enable_multithreading(&mut self, enable: bool) {
        self.glyph_rasterizer.enable_multithreading(enable);
    }

    fn should_tile(limit: i32, descriptor: &ImageDescriptor, data: &CachedImageData) -> bool {
        let size_check = descriptor.size.width > limit || descriptor.size.height > limit;
        match *data {
            CachedImageData::Raw(_) | CachedImageData::Blob => size_check,
            CachedImageData::External(info) => {
                // External handles already represent existing textures so it does
                // not make sense to tile them into smaller ones.
                info.image_type == ExternalImageType::Buffer && size_check
            }
        }
    }

    // Request the texture cache item for a cacheable render
    // task. If the item is already cached, the texture cache
    // handle will be returned. Otherwise, the user supplied
    // closure will be invoked to generate the render task
    // chain that is required to draw this task.
    pub fn request_render_task<F>(
        &mut self,
        key: RenderTaskCacheKey,
        gpu_cache: &mut GpuCache,
        rg_builder: &mut RenderTaskGraphBuilder,
        user_data: Option<[f32; 4]>,
        is_opaque: bool,
        parent: RenderTaskParent,
        surfaces: &[SurfaceInfo],
        f: F,
    ) -> RenderTaskId
    where
        F: FnOnce(&mut RenderTaskGraphBuilder) -> RenderTaskId,
    {
        self.cached_render_tasks.request_render_task(
            key,
            &mut self.texture_cache,
            gpu_cache,
            rg_builder,
            user_data,
            is_opaque,
            parent,
            surfaces,
            |render_graph| Ok(f(render_graph))
        ).expect("Failed to request a render task from the resource cache!")
    }

    pub fn post_scene_building_update(
        &mut self,
        updates: Vec<ResourceUpdate>,
        profile: &mut TransactionProfile,
    ) {
        // TODO, there is potential for optimization here, by processing updates in
        // bulk rather than one by one (for example by sorting allocations by size or
        // in a way that reduces fragmentation in the atlas).
        #[cfg(feature = "capture")]
        match updates.is_empty() {
            false => self.capture_dirty = true,
            _ => {},
        }

        for update in updates {
            match update {
                ResourceUpdate::AddImage(img) => {
                    if let ImageData::Raw(ref bytes) = img.data {
                        self.image_templates_memory += bytes.len();
                        profile.set(profiler::IMAGE_TEMPLATES_MEM, bytes_to_mb(self.image_templates_memory));
                    }
                    self.add_image_template(
                        img.key,
                        img.descriptor,
                        img.data.into(),
                        &img.descriptor.size.into(),
                        img.tiling,
                    );
                    profile.set(profiler::IMAGE_TEMPLATES, self.resources.image_templates.images.len());
                }
                ResourceUpdate::UpdateImage(img) => {
                    self.update_image_template(img.key, img.descriptor, img.data.into(), &img.dirty_rect);
                }
                ResourceUpdate::AddBlobImage(img) => {
                    self.add_image_template(
                        img.key.as_image(),
                        img.descriptor,
                        CachedImageData::Blob,
                        &img.visible_rect,
                        Some(img.tile_size),
                    );
                }
                ResourceUpdate::UpdateBlobImage(img) => {
                    self.update_image_template(
                        img.key.as_image(),
                        img.descriptor,
                        CachedImageData::Blob,
                        &to_image_dirty_rect(
                            &img.dirty_rect
                        ),
                    );
                    self.discard_tiles_outside_visible_area(img.key, &img.visible_rect); // TODO: remove?
                    self.set_image_visible_rect(img.key.as_image(), &img.visible_rect);
                }
                ResourceUpdate::DeleteImage(img) => {
                    self.delete_image_template(img);
                    profile.set(profiler::IMAGE_TEMPLATES, self.resources.image_templates.images.len());
                    profile.set(profiler::IMAGE_TEMPLATES_MEM, bytes_to_mb(self.image_templates_memory));
                }
                ResourceUpdate::DeleteBlobImage(img) => {
                    self.delete_image_template(img.as_image());
                }
                ResourceUpdate::DeleteFont(font) => {
                    self.delete_font_template(font);
                    profile.set(profiler::FONT_TEMPLATES, self.resources.font_templates.len());
                    profile.set(profiler::FONT_TEMPLATES_MEM, bytes_to_mb(self.font_templates_memory));
                }
                ResourceUpdate::DeleteFontInstance(font) => {
                    self.delete_font_instance(font);
                }
                ResourceUpdate::SetBlobImageVisibleArea(key, area) => {
                    self.discard_tiles_outside_visible_area(key, &area);
                    self.set_image_visible_rect(key.as_image(), &area);
                }
                ResourceUpdate::AddFont(font) => {
                    match font {
                        AddFont::Raw(id, bytes, index) => {
                            self.font_templates_memory += bytes.len();
                            profile.set(profiler::FONT_TEMPLATES_MEM, bytes_to_mb(self.font_templates_memory));
                            self.add_font_template(id, FontTemplate::Raw(bytes, index));
                        }
                        AddFont::Native(id, native_font_handle) => {
                            self.add_font_template(id, FontTemplate::Native(native_font_handle));
                        }
                    }
                    profile.set(profiler::FONT_TEMPLATES, self.resources.font_templates.len());
                }
                ResourceUpdate::AddFontInstance(..) => {
                    // Already added in ApiResources.
                }
            }
        }
    }

    pub fn add_rasterized_blob_images(
        &mut self,
        images: Vec<(BlobImageRequest, BlobImageResult)>,
        profile: &mut TransactionProfile,
    ) {
        for (request, result) in images {
            let data = match result {
                Ok(data) => data,
                Err(..) => {
                    warn!("Failed to rasterize a blob image");
                    continue;
                }
            };

            profile.add(profiler::RASTERIZED_BLOBS_PX, data.rasterized_rect.area());

            // First make sure we have an entry for this key (using a placeholder
            // if need be).
            let tiles = self.rasterized_blob_images.entry(request.key).or_insert_with(
                || { RasterizedBlob::default() }
            );

            tiles.insert(request.tile, data);

            match self.cached_images.try_get_mut(&request.key.as_image()) {
                Some(&mut ImageResult::Multi(ref mut entries)) => {
                    let cached_key = CachedImageKey {
                        rendering: ImageRendering::Auto, // TODO(nical)
                        tile: Some(request.tile),
                    };
                    if let Some(entry) = entries.try_get_mut(&cached_key) {
                        entry.dirty_rect = DirtyRect::All;
                    }
                }
                _ => {}
            }
        }
    }

    pub fn add_font_template(&mut self, font_key: FontKey, template: FontTemplate) {
        // Push the new font to the font renderer, and also store
        // it locally for glyph metric requests.
        if let FontTemplate::Raw(ref font, _) = template {
            self.resources.weak_fonts.insert(Arc::downgrade(font));
        }
        self.glyph_rasterizer.add_font(font_key, template.clone());
        self.resources.font_templates.insert(font_key, template);
    }

    pub fn delete_font_template(&mut self, font_key: FontKey) {
        self.glyph_rasterizer.delete_font(font_key);
        if let Some(FontTemplate::Raw(data, _)) = self.resources.font_templates.remove(&font_key) {
            self.font_templates_memory -= data.len();
        }
        self.cached_glyphs
            .clear_fonts(|font| font.font_key == font_key);
    }

    pub fn delete_font_instance(&mut self, instance_key: FontInstanceKey) {
        self.resources.font_instances.delete_font_instance(instance_key);
    }

    pub fn get_font_instances(&self) -> SharedFontInstanceMap {
        self.resources.font_instances.clone()
    }

    pub fn get_font_instance(&self, instance_key: FontInstanceKey) -> Option<Arc<BaseFontInstance>> {
        self.resources.font_instances.get_font_instance(instance_key)
    }

    pub fn add_image_template(
        &mut self,
        image_key: ImageKey,
        descriptor: ImageDescriptor,
        data: CachedImageData,
        visible_rect: &DeviceIntRect,
        mut tiling: Option<TileSize>,
    ) {
        if tiling.is_none() && Self::should_tile(self.tiling_threshold(), &descriptor, &data) {
            // We aren't going to be able to upload a texture this big, so tile it, even
            // if tiling was not requested.
            tiling = Some(DEFAULT_TILE_SIZE);
        }

        let resource = ImageResource {
            descriptor,
            data,
            tiling,
            visible_rect: *visible_rect,
            generation: ImageGeneration(0),
        };

        self.resources.image_templates.insert(image_key, resource);
    }

    pub fn update_image_template(
        &mut self,
        image_key: ImageKey,
        descriptor: ImageDescriptor,
        data: CachedImageData,
        dirty_rect: &ImageDirtyRect,
    ) {
        let tiling_threshold = self.tiling_threshold();
        let image = match self.resources.image_templates.get_mut(image_key) {
            Some(res) => res,
            None => panic!("Attempt to update non-existent image"),
        };

        let mut tiling = image.tiling;
        if tiling.is_none() && Self::should_tile(tiling_threshold, &descriptor, &data) {
            tiling = Some(DEFAULT_TILE_SIZE);
        }

        // Each cache entry stores its own copy of the image's dirty rect. This allows them to be
        // updated independently.
        match self.cached_images.try_get_mut(&image_key) {
            Some(&mut ImageResult::UntiledAuto(ref mut entry)) => {
                entry.dirty_rect = entry.dirty_rect.union(dirty_rect);
            }
            Some(&mut ImageResult::Multi(ref mut entries)) => {
                for (key, entry) in entries.iter_mut() {
                    // We want the dirty rect relative to the tile and not the whole image.
                    let local_dirty_rect = match (tiling, key.tile) {
                        (Some(tile_size), Some(tile)) => {
                            dirty_rect.map(|mut rect|{
                                let tile_offset = DeviceIntPoint::new(
                                    tile.x as i32,
                                    tile.y as i32,
                                ) * tile_size as i32;
                                rect.origin -= tile_offset.to_vector();

                                let tile_rect = compute_tile_size(
                                    &descriptor.size.into(),
                                    tile_size,
                                    tile,
                                ).into();

                                rect.intersection(&tile_rect).unwrap_or_else(DeviceIntRect::zero)
                            })
                        }
                        (None, Some(..)) => DirtyRect::All,
                        _ => *dirty_rect,
                    };
                    entry.dirty_rect = entry.dirty_rect.union(&local_dirty_rect);
                }
            }
            _ => {}
        }

        if image.descriptor.format != descriptor.format {
            // could be a stronger warning/error?
            trace!("Format change {:?} -> {:?}", image.descriptor.format, descriptor.format);
        }
        *image = ImageResource {
            descriptor,
            data,
            tiling,
            visible_rect: descriptor.size.into(),
            generation: ImageGeneration(image.generation.0 + 1),
        };
    }

    pub fn delete_image_template(&mut self, image_key: ImageKey) {
        // Remove the template.
        let value = self.resources.image_templates.remove(image_key);

        // Release the corresponding texture cache entry, if any.
        if let Some(mut cached) = self.cached_images.remove(&image_key) {
            cached.drop_from_cache(&mut self.texture_cache);
        }

        match value {
            Some(image) => if image.data.is_blob() {
                if let CachedImageData::Raw(data) = image.data {
                    self.image_templates_memory -= data.len();
                }

                let blob_key = BlobImageKey(image_key);
                self.deleted_blob_keys.back_mut().unwrap().push(blob_key);
                self.rasterized_blob_images.remove(&blob_key);
            },
            None => {
                warn!("Delete the non-exist key");
                debug!("key={:?}", image_key);
            }
        }
    }

    /// Return the current generation of an image template
    pub fn get_image_generation(&self, key: ImageKey) -> ImageGeneration {
        self.resources
            .image_templates
            .get(key)
            .map_or(ImageGeneration::INVALID, |template| template.generation)
    }

    /// Requests an image to ensure that it will be in the texture cache this frame.
    ///
    /// returns the size in device pixel of the image or tile.
    pub fn request_image(
        &mut self,
        request: ImageRequest,
        gpu_cache: &mut GpuCache,
    ) -> DeviceIntSize {
        debug_assert_eq!(self.state, State::AddResources);

        let template = match self.resources.image_templates.get(request.key) {
            Some(template) => template,
            None => {
                warn!("ERROR: Trying to render deleted / non-existent key");
                debug!("key={:?}", request.key);
                return DeviceIntSize::zero();
            }
        };

        let size = match request.tile {
            Some(tile) => compute_tile_size(&template.visible_rect, template.tiling.unwrap(), tile),
            None => template.descriptor.size,
        };

        // Images that don't use the texture cache can early out.
        if !template.data.uses_texture_cache() {
            return size;
        }

        let side_size =
            template.tiling.map_or(cmp::max(template.descriptor.size.width, template.descriptor.size.height),
                                   |tile_size| tile_size as i32);
        if side_size > self.texture_cache.max_texture_size() {
            // The image or tiling size is too big for hardware texture size.
            warn!("Dropping image, image:(w:{},h:{}, tile:{}) is too big for hardware!",
                  template.descriptor.size.width, template.descriptor.size.height, template.tiling.unwrap_or(0));
            self.cached_images.insert(request.key, ImageResult::Err(ImageCacheError::OverLimitSize));
            return DeviceIntSize::zero();
        }

        let storage = match self.cached_images.entry(request.key) {
            Occupied(e) => {
                // We might have an existing untiled entry, and need to insert
                // a second entry. In such cases we need to move the old entry
                // out first, replacing it with a dummy entry, and then creating
                // the tiled/multi-entry variant.
                let entry = e.into_mut();
                if !request.is_untiled_auto() {
                    let untiled_entry = match entry {
                        &mut ImageResult::UntiledAuto(ref mut entry) => {
                            Some(mem::replace(entry, CachedImageInfo {
                                texture_cache_handle: TextureCacheHandle::invalid(),
                                dirty_rect: DirtyRect::All,
                                manual_eviction: false,
                            }))
                        }
                        _ => None
                    };

                    if let Some(untiled_entry) = untiled_entry {
                        let mut entries = ResourceClassCache::new();
                        let untiled_key = CachedImageKey {
                            rendering: ImageRendering::Auto,
                            tile: None,
                        };
                        entries.insert(untiled_key, untiled_entry);
                        *entry = ImageResult::Multi(entries);
                    }
                }
                entry
            }
            Vacant(entry) => {
                entry.insert(if request.is_untiled_auto() {
                    ImageResult::UntiledAuto(CachedImageInfo {
                        texture_cache_handle: TextureCacheHandle::invalid(),
                        dirty_rect: DirtyRect::All,
                        manual_eviction: false,
                    })
                } else {
                    ImageResult::Multi(ResourceClassCache::new())
                })
            }
        };

        // If this image exists in the texture cache, *and* the dirty rect
        // in the cache is empty, then it is valid to use as-is.
        let entry = match *storage {
            ImageResult::UntiledAuto(ref mut entry) => entry,
            ImageResult::Multi(ref mut entries) => {
                entries.entry(request.into())
                    .or_insert(CachedImageInfo {
                        texture_cache_handle: TextureCacheHandle::invalid(),
                        dirty_rect: DirtyRect::All,
                        manual_eviction: false,
                    })
            },
            ImageResult::Err(_) => panic!("Errors should already have been handled"),
        };

        let needs_upload = self.texture_cache.request(&entry.texture_cache_handle, gpu_cache);

        if !needs_upload && entry.dirty_rect.is_empty() {
            return size;
        }

        if !self.pending_image_requests.insert(request) {
            return size;
        }

        if template.data.is_blob() {
            let request: BlobImageRequest = request.into();
            let missing = match self.rasterized_blob_images.get(&request.key) {
                Some(tiles) => !tiles.contains_key(&request.tile),
                _ => true,
            };

            assert!(!missing);
        }

        size
    }

    fn discard_tiles_outside_visible_area(
        &mut self,
        key: BlobImageKey,
        area: &DeviceIntRect
    ) {
        let tile_size = match self.resources.image_templates.get(key.as_image()) {
            Some(template) => template.tiling.unwrap(),
            None => {
                //println!("Missing image template (key={:?})!", key);
                return;
            }
        };

        let tiles = match self.rasterized_blob_images.get_mut(&key) {
            Some(tiles) => tiles,
            _ => { return; }
        };

        let tile_range = compute_tile_range(
            &area,
            tile_size,
        );

        tiles.retain(|tile, _| { tile_range.contains(*tile) });

        let texture_cache = &mut self.texture_cache;
        match self.cached_images.try_get_mut(&key.as_image()) {
            Some(&mut ImageResult::Multi(ref mut entries)) => {
                entries.retain(|key, entry| {
                    if key.tile.is_none() || tile_range.contains(key.tile.unwrap()) {
                        return true;
                    }
                    entry.mark_unused(texture_cache);
                    return false;
                });
            }
            _ => {}
        }
    }

    fn set_image_visible_rect(&mut self, key: ImageKey, rect: &DeviceIntRect) {
        if let Some(image) = self.resources.image_templates.get_mut(key) {
            image.visible_rect = *rect;
            image.descriptor.size = rect.size;
        }
    }

    pub fn request_glyphs(
        &mut self,
        mut font: FontInstance,
        glyph_keys: &[GlyphKey],
        gpu_cache: &mut GpuCache,
    ) {
        debug_assert_eq!(self.state, State::AddResources);

        self.glyph_rasterizer.prepare_font(&mut font);
        self.glyph_rasterizer.request_glyphs(
            &mut self.cached_glyphs,
            font,
            glyph_keys,
            &mut self.texture_cache,
            gpu_cache,
        );
    }

    pub fn pending_updates(&mut self) -> ResourceUpdateList {
        ResourceUpdateList {
            texture_updates: self.texture_cache.pending_updates(),
            native_surface_updates: mem::replace(&mut self.pending_native_surface_updates, Vec::new()),
        }
    }

    pub fn fetch_glyphs<F>(
        &self,
        mut font: FontInstance,
        glyph_keys: &[GlyphKey],
        fetch_buffer: &mut Vec<GlyphFetchResult>,
        gpu_cache: &mut GpuCache,
        mut f: F,
    ) where
        F: FnMut(TextureSource, GlyphFormat, &[GlyphFetchResult]),
    {
        debug_assert_eq!(self.state, State::QueryResources);

        self.glyph_rasterizer.prepare_font(&mut font);
        let glyph_key_cache = self.cached_glyphs.get_glyph_key_cache_for_font(&font);

        let mut current_texture_id = TextureSource::Invalid;
        let mut current_glyph_format = GlyphFormat::Subpixel;
        debug_assert!(fetch_buffer.is_empty());

        for (loop_index, key) in glyph_keys.iter().enumerate() {
            let (cache_item, glyph_format) = match *glyph_key_cache.get(key) {
                GlyphCacheEntry::Cached(ref glyph) => {
                    (self.texture_cache.get(&glyph.texture_cache_handle), glyph.format)
                }
                GlyphCacheEntry::Blank | GlyphCacheEntry::Pending => continue,
            };
            if current_texture_id != cache_item.texture_id ||
                current_glyph_format != glyph_format {
                if !fetch_buffer.is_empty() {
                    f(current_texture_id, current_glyph_format, fetch_buffer);
                    fetch_buffer.clear();
                }
                current_texture_id = cache_item.texture_id;
                current_glyph_format = glyph_format;
            }
            fetch_buffer.push(GlyphFetchResult {
                index_in_text_run: loop_index as i32,
                uv_rect_address: gpu_cache.get_address(&cache_item.uv_rect_handle),
                offset: DevicePoint::new(cache_item.user_data[0], cache_item.user_data[1]),
                size: cache_item.uv_rect.size,
                scale: cache_item.user_data[2],
            });
        }

        if !fetch_buffer.is_empty() {
            f(current_texture_id, current_glyph_format, fetch_buffer);
            fetch_buffer.clear();
        }
    }

    pub fn get_glyph_dimensions(
        &mut self,
        font: &FontInstance,
        glyph_index: GlyphIndex,
    ) -> Option<GlyphDimensions> {
        match self.cached_glyph_dimensions.entry((font.instance_key, glyph_index)) {
            Occupied(entry) => *entry.get(),
            Vacant(entry) => *entry.insert(
                self.glyph_rasterizer
                    .get_glyph_dimensions(font, glyph_index),
            ),
        }
    }

    pub fn get_glyph_index(&mut self, font_key: FontKey, ch: char) -> Option<u32> {
        self.glyph_rasterizer.get_glyph_index(font_key, ch)
    }

    #[inline]
    pub fn get_cached_image(&self, request: ImageRequest) -> Result<CacheItem, ()> {
        debug_assert_eq!(self.state, State::QueryResources);
        let image_info = self.get_image_info(request)?;
        Ok(self.get_texture_cache_item(&image_info.texture_cache_handle))
    }

    pub fn get_cached_render_task(
        &self,
        handle: &RenderTaskCacheEntryHandle,
    ) -> &RenderTaskCacheEntry {
        self.cached_render_tasks.get_cache_entry(handle)
    }

    #[inline]
    fn get_image_info(&self, request: ImageRequest) -> Result<&CachedImageInfo, ()> {
        // TODO(Jerry): add a debug option to visualize the corresponding area for
        // the Err() case of CacheItem.
        match *self.cached_images.get(&request.key) {
            ImageResult::UntiledAuto(ref image_info) => Ok(image_info),
            ImageResult::Multi(ref entries) => Ok(entries.get(&request.into())),
            ImageResult::Err(_) => Err(()),
        }
    }

    #[inline]
    pub fn get_texture_cache_item(&self, handle: &TextureCacheHandle) -> CacheItem {
        self.texture_cache.get(handle)
    }

    pub fn get_image_properties(&self, image_key: ImageKey) -> Option<ImageProperties> {
        let image_template = &self.resources.image_templates.get(image_key);

        image_template.map(|image_template| {
            let external_image = match image_template.data {
                CachedImageData::External(ext_image) => match ext_image.image_type {
                    ExternalImageType::TextureHandle(_) => Some(ext_image),
                    // external buffer uses resource_cache.
                    ExternalImageType::Buffer => None,
                },
                // raw and blob image are all using resource_cache.
                CachedImageData::Raw(..) | CachedImageData::Blob => None,
            };

            ImageProperties {
                descriptor: image_template.descriptor,
                external_image,
                tiling: image_template.tiling,
                visible_rect: image_template.visible_rect,
            }
        })
    }

    pub fn begin_frame(&mut self, stamp: FrameStamp, profile: &mut TransactionProfile) {
        profile_scope!("begin_frame");
        debug_assert_eq!(self.state, State::Idle);
        self.state = State::AddResources;
        self.texture_cache.begin_frame(stamp, profile);
        self.cached_glyphs.begin_frame(
            stamp,
            &mut self.texture_cache,
            &mut self.glyph_rasterizer,
        );
        self.cached_render_tasks.begin_frame(&mut self.texture_cache);
        self.current_frame_id = stamp.frame_id();

        // pop the old frame and push a new one
        self.deleted_blob_keys.pop_front();
        self.deleted_blob_keys.push_back(Vec::new());
    }

    pub fn block_until_all_resources_added(
        &mut self,
        gpu_cache: &mut GpuCache,
        profile: &mut TransactionProfile,
    ) {
        profile_scope!("block_until_all_resources_added");

        debug_assert_eq!(self.state, State::AddResources);
        self.state = State::QueryResources;

        self.glyph_rasterizer.resolve_glyphs(
            &mut self.cached_glyphs,
            &mut self.texture_cache,
            gpu_cache,
            profile,
        );

        // Apply any updates of new / updated images (incl. blobs) to the texture cache.
        self.update_texture_cache(gpu_cache);
    }

    fn update_texture_cache(&mut self, gpu_cache: &mut GpuCache) {
        profile_scope!("update_texture_cache");
        for request in self.pending_image_requests.drain() {
            let image_template = self.resources.image_templates.get_mut(request.key).unwrap();
            debug_assert!(image_template.data.uses_texture_cache());

            let mut updates: SmallVec<[(CachedImageData, Option<DeviceIntRect>); 1]> = SmallVec::new();

            match image_template.data {
                CachedImageData::Raw(..) | CachedImageData::External(..) => {
                    // Safe to clone here since the Raw image data is an
                    // Arc, and the external image data is small.
                    updates.push((image_template.data.clone(), None));
                }
                CachedImageData::Blob => {
                    let blob_image = self.rasterized_blob_images.get_mut(&BlobImageKey(request.key)).unwrap();
                    let img = &blob_image[&request.tile.unwrap()];
                    updates.push((
                        CachedImageData::Raw(Arc::clone(&img.data)),
                        Some(img.rasterized_rect)
                    ));
                }
            };

            for (image_data, blob_rasterized_rect) in updates {
                let entry = match *self.cached_images.get_mut(&request.key) {
                    ImageResult::UntiledAuto(ref mut entry) => entry,
                    ImageResult::Multi(ref mut entries) => entries.get_mut(&request.into()),
                    ImageResult::Err(_) => panic!("Update requested for invalid entry")
                };

                let mut descriptor = image_template.descriptor.clone();
                let mut dirty_rect = entry.dirty_rect.replace_with_empty();

                if let Some(tile) = request.tile {
                    let tile_size = image_template.tiling.unwrap();
                    let clipped_tile_size = compute_tile_size(&image_template.visible_rect, tile_size, tile);
                    // The tiled image could be stored on the CPU as one large image or be
                    // already broken up into tiles. This affects the way we compute the stride
                    // and offset.
                    let tiled_on_cpu = image_template.data.is_blob();
                    if !tiled_on_cpu {
                        // we don't expect to have partial tiles at the top and left of non-blob
                        // images.
                        debug_assert_eq!(image_template.visible_rect.origin, point2(0, 0));
                        let bpp = descriptor.format.bytes_per_pixel();
                        let stride = descriptor.compute_stride();
                        descriptor.stride = Some(stride);
                        descriptor.offset +=
                            tile.y as i32 * tile_size as i32 * stride +
                            tile.x as i32 * tile_size as i32 * bpp;
                    }

                    descriptor.size = clipped_tile_size;
                }

                // If we are uploading the dirty region of a blob image we might have several
                // rects to upload so we use each of these rasterized rects rather than the
                // overall dirty rect of the image.
                if let Some(rect) = blob_rasterized_rect {
                    dirty_rect = DirtyRect::Partial(rect);
                }

                let filter = match request.rendering {
                    ImageRendering::Pixelated => {
                        TextureFilter::Nearest
                    }
                    ImageRendering::Auto | ImageRendering::CrispEdges => {
                        // If the texture uses linear filtering, enable mipmaps and
                        // trilinear filtering, for better image quality. We only
                        // support this for now on textures that are not placed
                        // into the shared cache. This accounts for any image
                        // that is > 512 in either dimension, so it should cover
                        // the most important use cases. We may want to support
                        // mip-maps on shared cache items in the future.
                        if descriptor.allow_mipmaps() &&
                           descriptor.size.width > 512 &&
                           descriptor.size.height > 512 &&
                           !self.texture_cache.is_allowed_in_shared_cache(
                            TextureFilter::Linear,
                            &descriptor,
                        ) {
                            TextureFilter::Trilinear
                        } else {
                            TextureFilter::Linear
                        }
                    }
                };

                let eviction = if image_template.data.is_blob() {
                    entry.manual_eviction = true;
                    Eviction::Manual
                } else {
                    Eviction::Auto
                };

                //Note: at this point, the dirty rectangle is local to the descriptor space
                self.texture_cache.update(
                    &mut entry.texture_cache_handle,
                    descriptor,
                    filter,
                    Some(image_data),
                    [0.0; 4],
                    dirty_rect,
                    gpu_cache,
                    None,
                    UvRectKind::Rect,
                    eviction,
                    TargetShader::Default,
                );
            }
        }
    }

    /// Queue up allocation of a new OS native compositor surface with the
    /// specified tile size.
    pub fn create_compositor_surface(
        &mut self,
        virtual_offset: DeviceIntPoint,
        tile_size: DeviceIntSize,
        is_opaque: bool,
    ) -> NativeSurfaceId {
        let id = NativeSurfaceId(NEXT_NATIVE_SURFACE_ID.fetch_add(1, Ordering::Relaxed) as u64);

        self.pending_native_surface_updates.push(
            NativeSurfaceOperation {
                details: NativeSurfaceOperationDetails::CreateSurface {
                    id,
                    virtual_offset,
                    tile_size,
                    is_opaque,
                },
            }
        );

        id
    }

    pub fn create_compositor_external_surface(
        &mut self,
        is_opaque: bool,
    ) -> NativeSurfaceId {
        let id = NativeSurfaceId(NEXT_NATIVE_SURFACE_ID.fetch_add(1, Ordering::Relaxed) as u64);

        self.pending_native_surface_updates.push(
            NativeSurfaceOperation {
                details: NativeSurfaceOperationDetails::CreateExternalSurface {
                    id,
                    is_opaque,
                },
            }
        );

        id
    }

    /// Queue up destruction of an existing native OS surface. This is used when
    /// a picture cache surface is dropped or resized.
    pub fn destroy_compositor_surface(
        &mut self,
        id: NativeSurfaceId,
    ) {
        self.pending_native_surface_updates.push(
            NativeSurfaceOperation {
                details: NativeSurfaceOperationDetails::DestroySurface {
                    id,
                }
            }
        );
    }

    /// Queue construction of a native compositor tile on a given surface.
    pub fn create_compositor_tile(
        &mut self,
        id: NativeTileId,
    ) {
        self.pending_native_surface_updates.push(
            NativeSurfaceOperation {
                details: NativeSurfaceOperationDetails::CreateTile {
                    id,
                },
            }
        );
    }

    /// Queue destruction of a native compositor tile.
    pub fn destroy_compositor_tile(
        &mut self,
        id: NativeTileId,
    ) {
        self.pending_native_surface_updates.push(
            NativeSurfaceOperation {
                details: NativeSurfaceOperationDetails::DestroyTile {
                    id,
                },
            }
        );
    }

    pub fn attach_compositor_external_image(
        &mut self,
        id: NativeSurfaceId,
        external_image: ExternalImageId,
    ) {
        self.pending_native_surface_updates.push(
            NativeSurfaceOperation {
                details: NativeSurfaceOperationDetails::AttachExternalImage {
                    id,
                    external_image,
                },
            }
        );
    }


    pub fn end_frame(&mut self, profile: &mut TransactionProfile) {
        debug_assert_eq!(self.state, State::QueryResources);
        profile_scope!("end_frame");
        self.state = State::Idle;

        // GC the render target pool, if it's currently > 64 MB in size.
        //
        // We use a simple scheme whereby we drop any texture that hasn't been used
        // in the last 60 frames, until we are below the size threshold. This should
        // generally prevent any sustained build-up of unused textures, unless we don't
        // generate frames for a long period. This can happen when the window is
        // minimized, and we probably want to flush all the WebRender caches in that case [1].
        // There is also a second "red line" memory threshold which prevents
        // memory exhaustion if many render targets are allocated within a small
        // number of frames. For now this is set at 320 MB (10x the normal memory threshold).
        //
        // [1] https://bugzilla.mozilla.org/show_bug.cgi?id=1494099
        self.gc_render_targets(
            64 * 1024 * 1024,
            32 * 1024 * 1024 * 10,
            60,
        );

        self.texture_cache.end_frame(profile);
    }

    pub fn set_debug_flags(&mut self, flags: DebugFlags) {
        GLYPH_FLASHING.store(flags.contains(DebugFlags::GLYPH_FLASHING), std::sync::atomic::Ordering::Relaxed);
        self.texture_cache.set_debug_flags(flags);
    }

    pub fn clear(&mut self, what: ClearCache) {
        if what.contains(ClearCache::IMAGES) {
            for (_key, mut cached) in self.cached_images.resources.drain() {
                cached.drop_from_cache(&mut self.texture_cache);
            }
        }
        if what.contains(ClearCache::GLYPHS) {
            self.cached_glyphs.clear();
        }
        if what.contains(ClearCache::GLYPH_DIMENSIONS) {
            self.cached_glyph_dimensions.clear();
        }
        if what.contains(ClearCache::RENDER_TASKS) {
            self.cached_render_tasks.clear();
        }
        if what.contains(ClearCache::TEXTURE_CACHE) {
            self.texture_cache.clear_all();
        }
        if what.contains(ClearCache::RENDER_TARGETS) {
            self.clear_render_target_pool();
        }
    }

    pub fn clear_namespace(&mut self, namespace: IdNamespace) {
        self.clear_images(|k| k.0 == namespace);

        self.resources.font_instances.clear_namespace(namespace);

        for &key in self.resources.font_templates.keys().filter(|key| key.0 == namespace) {
            self.glyph_rasterizer.delete_font(key);
        }
        self.resources
            .font_templates
            .retain(|key, _| key.0 != namespace);
        self.cached_glyphs
            .clear_fonts(|font| font.font_key.0 == namespace);
    }

    /// Reports the CPU heap usage of this ResourceCache.
    ///
    /// NB: It would be much better to use the derive(MallocSizeOf) machinery
    /// here, but the Arcs complicate things. The two ways to handle that would
    /// be to either (a) Implement MallocSizeOf manually for the things that own
    /// them and manually avoid double-counting, or (b) Use the "seen this pointer
    /// yet" machinery from the proper malloc_size_of crate. We can do this if/when
    /// more accurate memory reporting on these resources becomes a priority.
    pub fn report_memory(&self, op: VoidPtrToSizeFn) -> MemoryReport {
        let mut report = MemoryReport::default();

        let mut seen_fonts = std::collections::HashSet::new();
        // Measure fonts. We only need the templates here, because the instances
        // don't have big buffers.
        for (_, font) in self.resources.font_templates.iter() {
            if let FontTemplate::Raw(ref raw, _) = font {
                report.fonts += unsafe { op(raw.as_ptr() as *const c_void) };
                seen_fonts.insert(raw.as_ptr());
            }
        }

        for font in self.resources.weak_fonts.iter() {
            if !seen_fonts.contains(&font.as_ptr()) { 
                report.weak_fonts += unsafe { op(font.as_ptr() as *const c_void) };
            }
        }

        // Measure images.
        for (_, image) in self.resources.image_templates.images.iter() {
            report.images += match image.data {
                CachedImageData::Raw(ref v) => unsafe { op(v.as_ptr() as *const c_void) },
                CachedImageData::Blob | CachedImageData::External(..) => 0,
            }
        }

        // Mesure rasterized blobs.
        // TODO(gw): Temporarily disabled while we roll back a crash. We can re-enable
        //           these when that crash is fixed.
        /*
        for (_, image) in self.rasterized_blob_images.iter() {
            let mut accumulate = |b: &RasterizedBlobImage| {
                report.rasterized_blobs += unsafe { op(b.data.as_ptr() as *const c_void) };
            };
            match image {
                RasterizedBlob::Tiled(map) => map.values().for_each(&mut accumulate),
                RasterizedBlob::NonTiled(vec) => vec.iter().for_each(&mut accumulate),
            };
        }
        */

        report
    }

    /// Properly deletes all images matching the predicate.
    fn clear_images<F: Fn(&ImageKey) -> bool>(&mut self, f: F) {
        let keys = self.resources.image_templates.images.keys().filter(|k| f(*k))
            .cloned().collect::<SmallVec<[ImageKey; 16]>>();

        for key in keys {
            self.delete_image_template(key);
        }

        #[cfg(features="leak_checks")]
        let check_leaks = true;
        #[cfg(not(features="leak_checks"))]
        let check_leaks = false;

        if check_leaks {
            let blob_f = |key: &BlobImageKey| { f(&key.as_image()) };
            assert!(!self.resources.image_templates.images.keys().any(&f));
            assert!(!self.cached_images.resources.keys().any(&f));
            assert!(!self.rasterized_blob_images.keys().any(&blob_f));
        }
    }

    /// Get a render target from the pool, or allocate a new one if none are
    /// currently available that match the requested parameters.
    pub fn get_or_create_render_target_from_pool(
        &mut self,
        size: DeviceIntSize,
        format: ImageFormat,
    ) -> CacheTextureId {
        for target in &mut self.render_target_pool {
            if target.size == size &&
               target.format == format &&
               !target.is_active {
                // Found a target that's not currently in use which matches. Update
                // the last_frame_used for GC purposes.
                target.is_active = true;
                target.last_frame_used = self.current_frame_id;
                return target.texture_id;
            }
        }

        // Need to create a new render target and add it to the pool

        let texture_id = self.texture_cache.alloc_render_target(
            size,
            format,
        );

        self.render_target_pool.push(RenderTarget {
            size,
            format,
            texture_id,
            is_active: true,
            last_frame_used: self.current_frame_id,
        });

        texture_id
    }

    /// Return a render target to the pool.
    pub fn return_render_target_to_pool(
        &mut self,
        id: CacheTextureId,
    ) {
        let target = self.render_target_pool
            .iter_mut()
            .find(|t| t.texture_id == id)
            .expect("bug: invalid render target id");

        assert!(target.is_active);
        target.is_active = false;
    }

    /// Clear all current render targets (e.g. on memory pressure)
    fn clear_render_target_pool(
        &mut self,
    ) {
        for target in self.render_target_pool.drain(..) {
            debug_assert!(!target.is_active);
            self.texture_cache.free_render_target(target.texture_id);
        }
    }

    /// Garbage collect and remove old render targets from the pool that haven't
    /// been used for some time.
    fn gc_render_targets(
        &mut self,
        total_bytes_threshold: usize,
        total_bytes_red_line_threshold: usize,
        frames_threshold: usize,
    ) {
        // Get the total GPU memory size used by the current render target pool
        let mut rt_pool_size_in_bytes: usize = self.render_target_pool
            .iter()
            .map(|t| t.size_in_bytes())
            .sum();

        // If the total size of the pool is less than the threshold, don't bother
        // trying to GC any targets
        if rt_pool_size_in_bytes <= total_bytes_threshold {
            return;
        }

        // Sort the current pool by age, so that we remove oldest textures first
        self.render_target_pool.sort_by_key(|t| t.last_frame_used);

        // We can't just use retain() because `RenderTarget` requires manual cleanup.
        let mut retained_targets = SmallVec::<[RenderTarget; 8]>::new();

        for target in self.render_target_pool.drain(..) {
            assert!(!target.is_active);

            // Drop oldest textures until we are under the allowed size threshold.
            // However, if it's been used in very recently, it is always kept around,
            // which ensures we don't thrash texture allocations on pages that do
            // require a very large render target pool and are regularly changing.
            let above_red_line = rt_pool_size_in_bytes > total_bytes_red_line_threshold;
            let above_threshold = rt_pool_size_in_bytes > total_bytes_threshold;
            let used_recently = target.used_recently(self.current_frame_id, frames_threshold);
            let used_this_frame = target.last_frame_used == self.current_frame_id;

            if !used_this_frame && (above_red_line || (above_threshold && !used_recently)) {
                rt_pool_size_in_bytes -= target.size_in_bytes();
                self.texture_cache.free_render_target(target.texture_id);
            } else {
                retained_targets.push(target);
            }
        }

        self.render_target_pool.extend(retained_targets);
    }

    #[cfg(test)]
    pub fn validate_surfaces(
        &self,
        expected_surfaces: &[(i32, i32, ImageFormat)],
    ) {
        assert_eq!(expected_surfaces.len(), self.render_target_pool.len());

        for (expected, surface) in expected_surfaces.iter().zip(self.render_target_pool.iter()) {
            assert_eq!(DeviceIntSize::new(expected.0, expected.1), surface.size);
            assert_eq!(expected.2, surface.format);
        }
    }
}

impl Drop for ResourceCache {
    fn drop(&mut self) {
        self.clear_images(|_| true);
    }
}

#[cfg(any(feature = "capture", feature = "replay"))]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
struct PlainFontTemplate {
    data: String,
    index: u32,
}

#[cfg(any(feature = "capture", feature = "replay"))]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
struct PlainImageTemplate {
    data: String,
    descriptor: ImageDescriptor,
    tiling: Option<TileSize>,
    generation: ImageGeneration,
}

#[cfg(any(feature = "capture", feature = "replay"))]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct PlainResources {
    font_templates: FastHashMap<FontKey, PlainFontTemplate>,
    font_instances: HashMap<FontInstanceKey, Arc<BaseFontInstance>>,
    image_templates: FastHashMap<ImageKey, PlainImageTemplate>,
}

#[cfg(feature = "capture")]
#[derive(Serialize)]
pub struct PlainCacheRef<'a> {
    current_frame_id: FrameId,
    glyphs: &'a GlyphCache,
    glyph_dimensions: &'a GlyphDimensionsCache,
    images: &'a ImageCache,
    render_tasks: &'a RenderTaskCache,
    textures: &'a TextureCache,
}

#[cfg(feature = "replay")]
#[derive(Deserialize)]
pub struct PlainCacheOwn {
    current_frame_id: FrameId,
    glyphs: GlyphCache,
    glyph_dimensions: GlyphDimensionsCache,
    images: ImageCache,
    render_tasks: RenderTaskCache,
    textures: TextureCache,
}

#[cfg(feature = "replay")]
const NATIVE_FONT: &'static [u8] = include_bytes!("../res/Proggy.ttf");

// This currently only casts the unit but will soon apply an offset
fn to_image_dirty_rect(blob_dirty_rect: &BlobDirtyRect) -> ImageDirtyRect {
    match *blob_dirty_rect {
        DirtyRect::Partial(rect) => DirtyRect::Partial(
            DeviceIntRect {
                origin: DeviceIntPoint::new(rect.origin.x, rect.origin.y),
                size: DeviceIntSize::new(rect.size.width, rect.size.height),
            }
        ),
        DirtyRect::All => DirtyRect::All,
    }
}

impl ResourceCache {
    #[cfg(feature = "capture")]
    pub fn save_capture(
        &mut self, root: &PathBuf
    ) -> (PlainResources, Vec<ExternalCaptureImage>) {
        use std::fs;
        use std::io::Write;

        info!("saving resource cache");
        let res = &self.resources;
        let path_fonts = root.join("fonts");
        if !path_fonts.is_dir() {
            fs::create_dir(&path_fonts).unwrap();
        }
        let path_images = root.join("images");
        if !path_images.is_dir() {
            fs::create_dir(&path_images).unwrap();
        }
        let path_blobs = root.join("blobs");
        if !path_blobs.is_dir() {
            fs::create_dir(&path_blobs).unwrap();
        }
        let path_externals = root.join("externals");
        if !path_externals.is_dir() {
            fs::create_dir(&path_externals).unwrap();
        }

        info!("\tfont templates");
        let mut font_paths = FastHashMap::default();
        for template in res.font_templates.values() {
            let data: &[u8] = match *template {
                FontTemplate::Raw(ref arc, _) => arc,
                FontTemplate::Native(_) => continue,
            };
            let font_id = res.font_templates.len() + 1;
            let entry = match font_paths.entry(data.as_ptr()) {
                Entry::Occupied(_) => continue,
                Entry::Vacant(e) => e,
            };
            let file_name = format!("{}.raw", font_id);
            let short_path = format!("fonts/{}", file_name);
            fs::File::create(path_fonts.join(file_name))
                .expect(&format!("Unable to create {}", short_path))
                .write_all(data)
                .unwrap();
            entry.insert(short_path);
        }

        info!("\timage templates");
        let mut image_paths = FastHashMap::default();
        let mut other_paths = FastHashMap::default();
        let mut num_blobs = 0;
        let mut external_images = Vec::new();
        for (&key, template) in res.image_templates.images.iter() {
            let desc = &template.descriptor;
            match template.data {
                CachedImageData::Raw(ref arc) => {
                    let image_id = image_paths.len() + 1;
                    let entry = match image_paths.entry(arc.as_ptr()) {
                        Entry::Occupied(_) => continue,
                        Entry::Vacant(e) => e,
                    };

                    #[cfg(feature = "png")]
                    CaptureConfig::save_png(
                        root.join(format!("images/{}.png", image_id)),
                        desc.size,
                        desc.format,
                        desc.stride,
                        &arc,
                    );
                    let file_name = format!("{}.raw", image_id);
                    let short_path = format!("images/{}", file_name);
                    fs::File::create(path_images.join(file_name))
                        .expect(&format!("Unable to create {}", short_path))
                        .write_all(&*arc)
                        .unwrap();
                    entry.insert(short_path);
                }
                CachedImageData::Blob => {
                    warn!("Tiled blob images aren't supported yet");
                    let result = RasterizedBlobImage {
                        rasterized_rect: desc.size.into(),
                        data: Arc::new(vec![0; desc.compute_total_size() as usize])
                    };

                    assert_eq!(result.rasterized_rect.size, desc.size);
                    assert_eq!(result.data.len(), desc.compute_total_size() as usize);

                    num_blobs += 1;
                    #[cfg(feature = "png")]
                    CaptureConfig::save_png(
                        root.join(format!("blobs/{}.png", num_blobs)),
                        desc.size,
                        desc.format,
                        desc.stride,
                        &result.data,
                    );
                    let file_name = format!("{}.raw", num_blobs);
                    let short_path = format!("blobs/{}", file_name);
                    let full_path = path_blobs.clone().join(&file_name);
                    fs::File::create(full_path)
                        .expect(&format!("Unable to create {}", short_path))
                        .write_all(&result.data)
                        .unwrap();
                    other_paths.insert(key, short_path);
                }
                CachedImageData::External(ref ext) => {
                    let short_path = format!("externals/{}", external_images.len() + 1);
                    other_paths.insert(key, short_path.clone());
                    external_images.push(ExternalCaptureImage {
                        short_path,
                        descriptor: desc.clone(),
                        external: ext.clone(),
                    });
                }
            }
        }

        let resources = PlainResources {
            font_templates: res.font_templates
                .iter()
                .map(|(key, template)| {
                    (*key, match *template {
                        FontTemplate::Raw(ref arc, index) => {
                            PlainFontTemplate {
                                data: font_paths[&arc.as_ptr()].clone(),
                                index,
                            }
                        }
                        #[cfg(not(target_os = "macos"))]
                        FontTemplate::Native(ref native) => {
                            PlainFontTemplate {
                                data: native.path.to_string_lossy().to_string(),
                                index: native.index,
                            }
                        }
                        #[cfg(target_os = "macos")]
                        FontTemplate::Native(ref native) => {
                            PlainFontTemplate {
                                data: native.0.postscript_name().to_string(),
                                index: 0,
                            }
                        }
                    })
                })
                .collect(),
            font_instances: res.font_instances.clone_map(),
            image_templates: res.image_templates.images
                .iter()
                .map(|(key, template)| {
                    (*key, PlainImageTemplate {
                        data: match template.data {
                            CachedImageData::Raw(ref arc) => image_paths[&arc.as_ptr()].clone(),
                            _ => other_paths[key].clone(),
                        },
                        descriptor: template.descriptor.clone(),
                        tiling: template.tiling,
                        generation: template.generation,
                    })
                })
                .collect(),
        };

        (resources, external_images)
    }

    #[cfg(feature = "capture")]
    pub fn save_caches(&self, _root: &PathBuf) -> PlainCacheRef {
        PlainCacheRef {
            current_frame_id: self.current_frame_id,
            glyphs: &self.cached_glyphs,
            glyph_dimensions: &self.cached_glyph_dimensions,
            images: &self.cached_images,
            render_tasks: &self.cached_render_tasks,
            textures: &self.texture_cache,
        }
    }

    #[cfg(feature = "replay")]
    pub fn load_capture(
        &mut self,
        resources: PlainResources,
        caches: Option<PlainCacheOwn>,
        config: &CaptureConfig,
    ) -> Vec<PlainExternalImage> {
        use std::{fs, path::Path};
        use crate::texture_cache::TextureCacheConfig;

        info!("loading resource cache");
        //TODO: instead of filling the local path to Arc<data> map as we process
        // each of the resource types, we could go through all of the local paths
        // and fill out the map as the first step.
        let mut raw_map = FastHashMap::<String, Arc<Vec<u8>>>::default();

        self.clear(ClearCache::all());
        self.clear_images(|_| true);

        match caches {
            Some(cached) => {
                self.current_frame_id = cached.current_frame_id;
                self.cached_glyphs = cached.glyphs;
                self.cached_glyph_dimensions = cached.glyph_dimensions;
                self.cached_images = cached.images;
                self.cached_render_tasks = cached.render_tasks;
                self.texture_cache = cached.textures;
            }
            None => {
                self.current_frame_id = FrameId::INVALID;
                self.texture_cache = TextureCache::new(
                    self.texture_cache.max_texture_size(),
                    self.texture_cache.tiling_threshold(),
                    self.texture_cache.default_picture_tile_size(),
                    self.texture_cache.color_formats(),
                    self.texture_cache.swizzle_settings(),
                    &TextureCacheConfig::DEFAULT,
                );
            }
        }

        self.glyph_rasterizer.reset();
        let res = &mut self.resources;
        res.font_templates.clear();
        res.font_instances.set(resources.font_instances);
        res.image_templates.images.clear();

        info!("\tfont templates...");
        let root = config.resource_root();
        let native_font_replacement = Arc::new(NATIVE_FONT.to_vec());
        for (key, plain_template) in resources.font_templates {
            let arc = match raw_map.entry(plain_template.data) {
                Entry::Occupied(e) => {
                    e.get().clone()
                }
                Entry::Vacant(e) => {
                    let file_path = if Path::new(e.key()).is_absolute() {
                        PathBuf::from(e.key())
                    } else {
                        root.join(e.key())
                    };
                    let arc = match fs::read(file_path) {
                        Ok(buffer) => Arc::new(buffer),
                        Err(err) => {
                            error!("Unable to open font template {:?}: {:?}", e.key(), err);
                            Arc::clone(&native_font_replacement)
                        }
                    };
                    e.insert(arc).clone()
                }
            };

            let template = FontTemplate::Raw(arc, plain_template.index);
            self.glyph_rasterizer.add_font(key, template.clone());
            res.font_templates.insert(key, template);
        }

        info!("\timage templates...");
        let mut external_images = Vec::new();
        for (key, template) in resources.image_templates {
            let data = match config.deserialize_for_resource::<PlainExternalImage, _>(&template.data) {
                Some(plain) => {
                    let ext_data = plain.external;
                    external_images.push(plain);
                    CachedImageData::External(ext_data)
                }
                None => {
                    let arc = match raw_map.entry(template.data) {
                        Entry::Occupied(e) => {
                            e.get().clone()
                        }
                        Entry::Vacant(e) => {
                            let buffer = fs::read(root.join(e.key()))
                                .expect(&format!("Unable to open {}", e.key()));
                            e.insert(Arc::new(buffer))
                                .clone()
                        }
                    };
                    CachedImageData::Raw(arc)
                }
            };

            res.image_templates.images.insert(key, ImageResource {
                data,
                descriptor: template.descriptor,
                tiling: template.tiling,
                visible_rect: template.descriptor.size.into(),
                generation: template.generation,
            });
        }

        external_images
    }

    #[cfg(feature = "capture")]
    pub fn save_capture_sequence(&mut self, config: &mut CaptureConfig) -> Vec<ExternalCaptureImage> {
        if self.capture_dirty {
            self.capture_dirty = false;
            config.prepare_resource();
            let (resources, deferred) = self.save_capture(&config.resource_root());
            config.serialize_for_resource(&resources, "plain-resources.ron");
            deferred
        } else {
            Vec::new()
        }
    }
}
