/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module contains the logic to obtain a primitive's source texture and uv rect.
//!
//! Currently this is a somewhat involved process because the code grew into having ad-hoc
//! ways to store this information depending on how the image data is produced. The goal
//! is for any textured primitive to be able to read from any source (texture cache, render
//! tasks, etc.) without primitive-specific code.

use crate::api::ExternalImageType;
use crate::api::units::*;
use crate::gpu_cache::GpuCache;
use crate::prim_store::DeferredResolve;
use crate::renderer::BLOCKS_PER_UV_RECT;
use crate::render_task_cache::RenderTaskCacheEntryHandle;
use crate::resource_cache::{ResourceCache, ImageRequest, CacheItem};
use crate::internal_types::{TextureSource, DeferredResolveIndex};

/// Resolve a resource cache's imagre request into a texture cache item.
pub fn resolve_image(
    request: ImageRequest,
    resource_cache: &ResourceCache,
    gpu_cache: &mut GpuCache,
    deferred_resolves: &mut Vec<DeferredResolve>,
) -> CacheItem {
    match resource_cache.get_image_properties(request.key) {
        Some(image_properties) => {
            // Check if an external image that needs to be resolved
            // by the render thread.
            match image_properties.external_image {
                Some(external_image) => {
                    // This is an external texture - we will add it to
                    // the deferred resolves list to be patched by
                    // the render thread...
                    let cache_handle = gpu_cache.push_deferred_per_frame_blocks(BLOCKS_PER_UV_RECT);

                    let deferred_resolve_index = DeferredResolveIndex(deferred_resolves.len() as u32);

                    let image_buffer_kind = match external_image.image_type {
                        ExternalImageType::TextureHandle(target) => {
                            target
                        }
                        ExternalImageType::Buffer => {
                            // The ExternalImageType::Buffer should be handled by resource_cache.
                            // It should go through the non-external case.
                            panic!("Unexpected non-texture handle type");
                        }
                    };

                    let cache_item = CacheItem {
                        texture_id: TextureSource::External(deferred_resolve_index, image_buffer_kind),
                        uv_rect_handle: cache_handle,
                        uv_rect: DeviceIntRect::new(
                            DeviceIntPoint::zero(),
                            image_properties.descriptor.size,
                        ),
                        user_data: [0.0; 4],
                    };

                    deferred_resolves.push(DeferredResolve {
                        image_properties,
                        address: gpu_cache.get_address(&cache_handle),
                        rendering: request.rendering,
                    });

                    cache_item
                }
                None => {
                    if let Ok(cache_item) = resource_cache.get_cached_image(request) {
                        cache_item
                    } else {
                        // There is no usable texture entry for the image key. Just return an invalid texture here.
                        CacheItem::invalid()
                    }
                }
            }
        }
        None => {
            CacheItem::invalid()
        }
    }
}

pub fn resolve_cached_render_task(
    handle: &RenderTaskCacheEntryHandle,
    resource_cache: &ResourceCache,
) -> CacheItem {
    let rt_cache_entry = resource_cache
        .get_cached_render_task(&handle);

    resource_cache.get_texture_cache_item(&rt_cache_entry.handle)
}
