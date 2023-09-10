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
use crate::picture::{SurfaceIndex, SurfaceInfo};
use crate::prim_store::image::ImageCacheKey;
use crate::prim_store::gradient::{
    FastLinearGradientCacheKey, LinearGradientCacheKey, RadialGradientCacheKey,
    ConicGradientCacheKey,
};
use crate::prim_store::line_dec::LineDecorationCacheKey;
use crate::resource_cache::CacheItem;
use std::{mem, usize, f32, i32};
use crate::texture_cache::{TextureCache, TextureCacheHandle, Eviction, TargetShader};
use crate::render_target::RenderTargetKind;
use crate::render_task::{RenderTask, StaticRenderTaskSurface, RenderTaskLocation, RenderTaskKind, CachedTask};
use crate::render_task_graph::{RenderTaskGraphBuilder, RenderTaskId};
use crate::frame_builder::add_child_render_task;
use euclid::Scale;

const MAX_CACHE_TASK_SIZE: f32 = 4096.0;

/// Describes a parent dependency for a render task. Render tasks
/// may depend on a surface (e.g. when a surface uses a cached border)
/// or an arbitrary render task (e.g. when a clip mask uses a blurred
/// box-shadow input).
pub enum RenderTaskParent {
    /// Parent is a surface
    Surface(SurfaceIndex),
    /// Parent is a render task
    RenderTask(RenderTaskId),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub enum RenderTaskCacheKeyKind {
    BoxShadow(BoxShadowCacheKey),
    Image(ImageCacheKey),
    BorderSegment(BorderSegmentCacheKey),
    LineDecoration(LineDecorationCacheKey),
    FastLinearGradient(FastLinearGradientCacheKey),
    LinearGradient(LinearGradientCacheKey),
    RadialGradient(RadialGradientCacheKey),
    ConicGradient(ConicGradientCacheKey),
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
    user_data: Option<[f32; 4]>,
    target_kind: RenderTargetKind,
    is_opaque: bool,
    frame_id: u64,
    pub handle: TextureCacheHandle,
    /// If a render task was generated for this cache entry on _this_ frame,
    /// we need to track the task id here. This allows us to hook it up as
    /// a dependency of any parent tasks that make a reqiest from the render
    /// task cache.
    pub render_task_id: Option<RenderTaskId>,
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
    frame_id: u64,
}

pub type RenderTaskCacheEntryHandle = WeakFreeListHandle<RenderTaskCacheMarker>;

impl RenderTaskCache {
    pub fn new() -> Self {
        RenderTaskCache {
            map: FastHashMap::default(),
            cache_entries: FreeList::new(),
            frame_id: 0,
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
        self.frame_id += 1;
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
        let frame_id = self.frame_id;

        self.map.retain(|_, handle| {
            let mut retain = texture_cache.is_allocated(
                &cache_entries.get(handle).handle,
            );
            if retain {
                let entry = cache_entries.get_mut(&handle);
                if frame_id > entry.frame_id + 10 {
                    texture_cache.evict_handle(&entry.handle);
                    retain = false;
                }
            }

            if !retain {
                let handle = mem::replace(handle, FreeListHandle::invalid());
                cache_entries.free(handle);
            }

            retain
        });

        // Clear out the render task ID of any remaining cache entries that were drawn
        // on the previous frame, so we don't accidentally hook up stale dependencies
        // when building the frame graph.
        for (_, handle) in &self.map {
            let entry = self.cache_entries.get_mut(handle);
            entry.render_task_id = None;
        }
    }

    fn alloc_render_task(
        render_task: &mut RenderTask,
        entry: &mut RenderTaskCacheEntry,
        gpu_cache: &mut GpuCache,
        texture_cache: &mut TextureCache,
    ) {
        // Find out what size to alloc in the texture cache.
        let size = render_task.location.size();
        let target_kind = render_task.target_kind();

        // Select the right texture page to allocate from.
        let image_format = match target_kind {
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
            entry.user_data.unwrap_or([0.0; 4]),
            DirtyRect::All,
            gpu_cache,
            None,
            render_task.uv_rect_kind(),
            Eviction::Auto,
            TargetShader::Default,
        );

        // Get the allocation details in the texture cache, and store
        // this in the render task. The renderer will draw this task
        // into the appropriate rect of the texture cache on this frame.
        let (texture_id, uv_rect, _, _, _) =
            texture_cache.get_cache_location(&entry.handle);

        let surface = StaticRenderTaskSurface::TextureCache {
            texture: texture_id,
            target_kind,
        };

        render_task.location = RenderTaskLocation::Static {
            surface,
            rect: uv_rect.to_i32(),
        };
    }

    pub fn request_render_task<F>(
        &mut self,
        key: RenderTaskCacheKey,
        texture_cache: &mut TextureCache,
        gpu_cache: &mut GpuCache,
        rg_builder: &mut RenderTaskGraphBuilder,
        user_data: Option<[f32; 4]>,
        is_opaque: bool,
        parent: RenderTaskParent,
        surfaces: &[SurfaceInfo],
        f: F,
    ) -> Result<RenderTaskId, ()>
    where
        F: FnOnce(&mut RenderTaskGraphBuilder) -> Result<RenderTaskId, ()>,
    {
        let frame_id = self.frame_id;
        let size = key.size;
        // Get the texture cache handle for this cache key,
        // or create one.
        let cache_entries = &mut self.cache_entries;
        let entry_handle = self.map.entry(key).or_insert_with(|| {
            let entry = RenderTaskCacheEntry {
                handle: TextureCacheHandle::invalid(),
                user_data,
                target_kind: RenderTargetKind::Color, // will be set below.
                is_opaque,
                frame_id,
                render_task_id: None,
            };
            cache_entries.insert(entry)
        });
        let cache_entry = cache_entries.get_mut(entry_handle);
        cache_entry.frame_id = self.frame_id;

        // Check if this texture cache handle is valid.
        if texture_cache.request(&cache_entry.handle, gpu_cache) {
            // Invoke user closure to get render task chain
            // to draw this into the texture cache.
            let render_task_id = f(rg_builder)?;

            cache_entry.user_data = user_data;
            cache_entry.is_opaque = is_opaque;
            cache_entry.render_task_id = Some(render_task_id);

            let render_task = rg_builder.get_task_mut(render_task_id);

            render_task.mark_cached(entry_handle.weak());
            cache_entry.target_kind = render_task.kind.target_kind();

            RenderTaskCache::alloc_render_task(
                render_task,
                cache_entry,
                gpu_cache,
                texture_cache,
            );
        }

        // If this render task cache is being drawn this frame, ensure we hook up the
        // render task for it as a dependency of any render task that uses this as
        // an input source.
        if let Some(render_task_id) = cache_entry.render_task_id {
            match parent {
                RenderTaskParent::Surface(surface_index) => {
                    // If parent is a surface, use helper fn to add this dependency,
                    // which correctly takes account of the render task configuration
                    // of the surface.
                    add_child_render_task(
                        surface_index,
                        render_task_id,
                        surfaces,
                        rg_builder
                    );
                }
                RenderTaskParent::RenderTask(parent_render_task_id) => {
                    // For render tasks, just add it as a direct dependency on the
                    // task graph builder.
                    rg_builder.add_dependency(
                        parent_render_task_id,
                        render_task_id,
                    );
                }
            }

            return Ok(render_task_id);
        }

        let target_kind = cache_entry.target_kind;
        let mut task = RenderTask::new(
            RenderTaskLocation::CacheRequest { size, },
            RenderTaskKind::Cached(CachedTask {
                target_kind,
            }),
        );
        task.mark_cached(entry_handle.weak());
        let render_task_id = rg_builder.add().init(task);

        Ok(render_task_id)
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
pub fn to_cache_size(size: LayoutSize, device_pixel_scale: &mut Scale<f32, LayoutPixel, DevicePixel>) -> DeviceIntSize {
    let mut device_size = (size * *device_pixel_scale).round();

    if device_size.width > MAX_CACHE_TASK_SIZE || device_size.height > MAX_CACHE_TASK_SIZE {
        let scale = MAX_CACHE_TASK_SIZE / f32::max(device_size.width, device_size.height);
        *device_pixel_scale = *device_pixel_scale * Scale::new(scale);
        device_size = (size * *device_pixel_scale).round();
    }

    DeviceIntSize::new(
        1.max(device_size.width as i32),
        1.max(device_size.height as i32),
    )
}
