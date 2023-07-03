/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


use api::{ImageDescriptor, ImageDescriptorFlags, DirtyRect};
use api::units::*;
use crate::border::BorderSegmentCacheKey;
use crate::box_shadow::{BoxShadowCacheKey};
use crate::device::TextureFilter;
use crate::freelist::{FreeList, FreeListHandle, WeakFreeListHandle};
use crate::gpu_cache::GpuCache;
use crate::internal_types::FastHashMap;
use crate::prim_store::image::ImageCacheKey;
use crate::prim_store::gradient::GradientCacheKey;
use crate::prim_store::line_dec::LineDecorationCacheKey;
use crate::resource_cache::CacheItem;
use std::{mem, usize, f32, i32};
use crate::texture_cache::{TextureCache, TextureCacheHandle, Eviction};
use crate::render_target::RenderTargetKind;
use crate::render_task::{RenderTask, RenderTaskLocation};
use crate::render_task_graph::{RenderTaskGraph, RenderTaskId};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub enum RenderTaskCacheKeyKind {
    BoxShadow(BoxShadowCacheKey),
    Image(ImageCacheKey),
    BorderSegment(BorderSegmentCacheKey),
    LineDecoration(LineDecorationCacheKey),
    Gradient(GradientCacheKey),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct RenderTaskCacheKey {
    pub size: DeviceIntSize,
    pub kind: RenderTaskCacheKeyKind,
}

#[derive(Debug)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct RenderTaskCacheEntry {
    user_data: Option<[f32; 3]>,
    is_opaque: bool,
    pub handle: TextureCacheHandle,
}

#[derive(Debug, MallocSizeOf)]
#[cfg_attr(feature = "capture", derive(Serialize))]
pub enum RenderTaskCacheMarker {}

// A cache of render tasks that are stored in the texture
// cache for usage across frames.
#[derive(Debug)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct RenderTaskCache {
    map: FastHashMap<RenderTaskCacheKey, FreeListHandle<RenderTaskCacheMarker>>,
    cache_entries: FreeList<RenderTaskCacheEntry, RenderTaskCacheMarker>,
}

pub type RenderTaskCacheEntryHandle = WeakFreeListHandle<RenderTaskCacheMarker>;

impl RenderTaskCache {
    pub fn new() -> Self {
        RenderTaskCache {
            map: FastHashMap::default(),
            cache_entries: FreeList::new(),
        }
    }

    pub fn clear(&mut self) {
        self.map.clear();
        self.cache_entries.clear();
    }

    pub fn begin_frame(
        &mut self,
        texture_cache: &mut TextureCache,
    ) {
        profile_scope!("begin_frame");
        // Drop any items from the cache that have been
        // evicted from the texture cache.
        //
        // This isn't actually necessary for the texture
        // cache to be able to evict old render tasks.
        // It will evict render tasks as required, since
        // the access time in the texture cache entry will
        // be stale if this task hasn't been requested
        // for a while.
        //
        // Nonetheless, we should remove stale entries
        // from here so that this hash map doesn't
        // grow indefinitely!
        let cache_entries = &mut self.cache_entries;

        self.map.retain(|_, handle| {
            let retain = texture_cache.is_allocated(
                &cache_entries.get(handle).handle,
            );
            if !retain {
                let handle = mem::replace(handle, FreeListHandle::invalid());
                cache_entries.free(handle);
            }
            retain
        });
    }

    fn alloc_render_task(
        render_task: &mut RenderTask,
        entry: &mut RenderTaskCacheEntry,
        gpu_cache: &mut GpuCache,
        texture_cache: &mut TextureCache,
    ) {
        // Find out what size to alloc in the texture cache.
        let size = match render_task.location {
            RenderTaskLocation::Fixed(..) |
            RenderTaskLocation::PictureCache { .. } |
            RenderTaskLocation::TextureCache { .. } => {
                panic!("BUG: dynamic task was expected");
            }
            RenderTaskLocation::Dynamic(_, size) => size,
        };

        // Select the right texture page to allocate from.
        let image_format = match render_task.target_kind() {
            RenderTargetKind::Color => texture_cache.shared_color_expected_format(),
            RenderTargetKind::Alpha => texture_cache.shared_alpha_expected_format(),
        };

        let flags = if entry.is_opaque {
            ImageDescriptorFlags::IS_OPAQUE
        } else {
            ImageDescriptorFlags::empty()
        };

        let descriptor = ImageDescriptor::new(
            size.width,
            size.height,
            image_format,
            flags,
        );

        // Allocate space in the texture cache, but don't supply
        // and CPU-side data to be uploaded.
        texture_cache.update(
            &mut entry.handle,
            descriptor,
            TextureFilter::Linear,
            None,
            entry.user_data.unwrap_or([0.0; 3]),
            DirtyRect::All,
            gpu_cache,
            None,
            render_task.uv_rect_kind(),
            Eviction::Auto,
        );

        // Get the allocation details in the texture cache, and store
        // this in the render task. The renderer will draw this
        // task into the appropriate layer and rect of the texture
        // cache on this frame.
        let (texture_id, texture_layer, uv_rect, _, _) =
            texture_cache.get_cache_location(&entry.handle);

        render_task.location = RenderTaskLocation::TextureCache {
            texture: texture_id,
            layer: texture_layer,
            rect: uv_rect.to_i32(),
        };
    }

    pub fn request_render_task<F>(
        &mut self,
        key: RenderTaskCacheKey,
        texture_cache: &mut TextureCache,
        gpu_cache: &mut GpuCache,
        render_tasks: &mut RenderTaskGraph,
        user_data: Option<[f32; 3]>,
        is_opaque: bool,
        f: F,
    ) -> Result<RenderTaskCacheEntryHandle, ()>
    where
        F: FnOnce(&mut RenderTaskGraph) -> Result<RenderTaskId, ()>,
    {
        // Get the texture cache handle for this cache key,
        // or create one.
        let cache_entries = &mut self.cache_entries;
        let entry_handle = self.map.entry(key).or_insert_with(|| {
            let entry = RenderTaskCacheEntry {
                handle: TextureCacheHandle::invalid(),
                user_data,
                is_opaque,
            };
            cache_entries.insert(entry)
        });
        let cache_entry = cache_entries.get_mut(entry_handle);

        // Check if this texture cache handle is valid.
        if texture_cache.request(&cache_entry.handle, gpu_cache) {
            // Invoke user closure to get render task chain
            // to draw this into the texture cache.
            let render_task_id = f(render_tasks)?;
            render_tasks.cacheable_render_tasks.push(render_task_id);

            cache_entry.user_data = user_data;
            cache_entry.is_opaque = is_opaque;

            RenderTaskCache::alloc_render_task(
                &mut render_tasks[render_task_id],
                cache_entry,
                gpu_cache,
                texture_cache,
            );
        }

        Ok(entry_handle.weak())
    }

    pub fn get_cache_entry(
        &self,
        handle: &RenderTaskCacheEntryHandle,
    ) -> &RenderTaskCacheEntry {
        self.cache_entries
            .get_opt(handle)
            .expect("bug: invalid render task cache handle")
    }

    #[allow(dead_code)]
    pub fn get_cache_item_for_render_task(&self,
                                          texture_cache: &TextureCache,
                                          key: &RenderTaskCacheKey)
                                          -> CacheItem {
        // Get the texture cache handle for this cache key.
        let handle = self.map.get(key).unwrap();
        let cache_entry = self.cache_entries.get(handle);
        texture_cache.get(&cache_entry.handle)
    }

    #[allow(dead_code)]
    pub fn get_allocated_size_for_render_task(&self,
                                              texture_cache: &TextureCache,
                                              key: &RenderTaskCacheKey)
                                              -> Option<usize> {
        let handle = self.map.get(key).unwrap();
        let cache_entry = self.cache_entries.get(handle);
        texture_cache.get_allocated_size(&cache_entry.handle)
    }
}

// TODO(gw): Rounding the content rect here to device pixels is not
// technically correct. Ideally we should ceil() here, and ensure that
// the extra part pixel in the case of fractional sizes is correctly
// handled. For now, just use rounding which passes the existing
// Gecko tests.
// Note: zero-square tasks are prohibited in WR task graph, so
// we ensure each dimension to be at least the length of 1 after rounding.
pub fn to_cache_size(size: DeviceSize) -> DeviceIntSize {
    DeviceIntSize::new(
        1.max(size.width.round() as i32),
        1.max(size.height.round() as i32),
    )
}
