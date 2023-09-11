/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::api::{BlobImageKey, ImageDescriptor, DirtyRect, TileSize};
use crate::api::{BlobImageHandler, AsyncBlobImageRasterizer, BlobImageData, BlobImageParams};
use crate::api::{BlobImageRequest, BlobImageDescriptor, BlobImageResources};
use crate::api::{FontKey, FontTemplate, FontInstanceData, FontInstanceKey};
use crate::api::SharedFontInstanceMap;
use crate::api::units::*;
use crate::render_api::{ResourceUpdate, TransactionMsg, AddFont};
use crate::image_tiling::*;
use crate::profiler;

use std::collections::HashMap;
use std::sync::Arc;

/// We use this to generate the async blob rendering requests.
struct BlobImageTemplate {
    descriptor: ImageDescriptor,
    tile_size: TileSize,
    dirty_rect: BlobDirtyRect,
    /// See ImageResource::visible_rect.
    visible_rect: DeviceIntRect,
    // If the active rect of the blob changes, this represents the
    // range of tiles that remain valid. This must be taken into
    // account in addition to the valid rect when submitting blob
    // rasterization requests.
    // `None` means the bounds have not changed (tiles are still valid).
    // `Some(TileRange::zero())` means all of the tiles are invalid.
    valid_tiles_after_bounds_change: Option<TileRange>,
}

struct FontResources {
    templates: HashMap<FontKey, FontTemplate>,
    instances: SharedFontInstanceMap,
}

pub struct ApiResources {
    blob_image_templates: HashMap<BlobImageKey, BlobImageTemplate>,
    pub blob_image_handler: Option<Box<dyn BlobImageHandler>>,
    fonts: FontResources,
}

impl BlobImageResources for FontResources {
    fn get_font_data(&self, key: FontKey) -> &FontTemplate {
        self.templates.get(&key).unwrap()
    }
    fn get_font_instance_data(&self, key: FontInstanceKey) -> Option<FontInstanceData> {
        self.instances.get_font_instance_data(key)
    }
}

impl ApiResources {
    pub fn new(
        blob_image_handler: Option<Box<dyn BlobImageHandler>>,
        instances: SharedFontInstanceMap,
    ) -> Self {
        ApiResources {
            blob_image_templates: HashMap::new(),
            blob_image_handler,
            fonts: FontResources {
                templates: HashMap::new(),
                instances,
            }
        }
    }

    pub fn get_shared_font_instances(&self) -> SharedFontInstanceMap {
        self.fonts.instances.clone()
    }

    pub fn update(&mut self, transaction: &mut TransactionMsg) {
        let mut blobs_to_rasterize = Vec::new();
        for update in &transaction.resource_updates {
            match *update {
                ResourceUpdate::AddBlobImage(ref img) => {
                    self.blob_image_handler
                        .as_mut()
                        .unwrap()
                        .add(img.key, Arc::clone(&img.data), &img.visible_rect, img.tile_size);

                    self.blob_image_templates.insert(
                        img.key,
                        BlobImageTemplate {
                            descriptor: img.descriptor,
                            tile_size: img.tile_size,
                            dirty_rect: DirtyRect::All,
                            valid_tiles_after_bounds_change: None,
                            visible_rect: img.visible_rect,
                        },
                    );
                    blobs_to_rasterize.push(img.key);
                }
                ResourceUpdate::UpdateBlobImage(ref img) => {
                    debug_assert_eq!(img.visible_rect.size, img.descriptor.size);
                    self.update_blob_image(
                        img.key,
                        Some(&img.descriptor),
                        Some(&img.dirty_rect),
                        Some(Arc::clone(&img.data)),
                        &img.visible_rect,
                    );
                    blobs_to_rasterize.push(img.key);
                }
                ResourceUpdate::DeleteBlobImage(key) => {
                    transaction.use_scene_builder_thread = true;
                    self.blob_image_templates.remove(&key);
                    if let Some(ref mut handler) = self.blob_image_handler {
                        handler.delete(key);
                    }
                }
                ResourceUpdate::SetBlobImageVisibleArea(ref key, ref area) => {
                    self.update_blob_image(*key, None, None, None, area);
                    blobs_to_rasterize.push(*key);
                }
                ResourceUpdate::AddFont(ref font) => {
                    match font {
                        AddFont::Raw(key, bytes, index) => {
                            self.fonts.templates.insert(
                                *key,
                                FontTemplate::Raw(Arc::clone(bytes), *index),
                            );
                        }
                        AddFont::Native(key, native_font_handle) => {
                            self.fonts.templates.insert(
                                *key,
                                FontTemplate::Native(native_font_handle.clone()),
                            );
                        }
                    }
                }
                ResourceUpdate::AddFontInstance(ref instance) => {
                    // TODO(nical): Don't clone these.
                    self.fonts.instances.add_font_instance(
                        instance.key,
                        instance.font_key,
                        instance.glyph_size,
                        instance.options.clone(),
                        instance.platform_options.clone(),
                        instance.variations.clone(),
                    );
                }
                ResourceUpdate::DeleteFont(key) => {
                    transaction.use_scene_builder_thread = true;
                    self.fonts.templates.remove(&key);
                    if let Some(ref mut handler) = self.blob_image_handler {
                        handler.delete_font(key);
                    }
                }
                ResourceUpdate::DeleteFontInstance(key) => {
                    transaction.use_scene_builder_thread = true;
                    // We will delete from the shared font instance map in the resource cache
                    // after scene swap.

                    if let Some(ref mut r) = self.blob_image_handler {
                        r.delete_font_instance(key);
                    }
                }
                ResourceUpdate::DeleteImage(..) => {
                    transaction.use_scene_builder_thread = true;
                }
                _ => {}
            }
        }

        let (rasterizer, requests) = self.create_blob_scene_builder_requests(&blobs_to_rasterize);
        transaction.profile.set(profiler::RASTERIZED_BLOBS, blobs_to_rasterize.len());
        transaction.profile.set(profiler::RASTERIZED_BLOB_TILES, requests.len());
        transaction.use_scene_builder_thread |= !requests.is_empty();
        transaction.use_scene_builder_thread |= !transaction.scene_ops.is_empty();
        transaction.blob_rasterizer = rasterizer;
        transaction.blob_requests = requests;
    }

    pub fn enable_multithreading(&mut self, enable: bool) {
        if let Some(ref mut handler) = self.blob_image_handler {
            handler.enable_multithreading(enable);
        }
    }

    fn update_blob_image(
        &mut self,
        key: BlobImageKey,
        descriptor: Option<&ImageDescriptor>,
        dirty_rect: Option<&BlobDirtyRect>,
        data: Option<Arc<BlobImageData>>,
        visible_rect: &DeviceIntRect,
    ) {
        if let Some(data) = data {
            let dirty_rect = dirty_rect.unwrap();
            self.blob_image_handler.as_mut().unwrap().update(key, data, visible_rect, dirty_rect);
        }

        let image = self.blob_image_templates
            .get_mut(&key)
            .expect("Attempt to update non-existent blob image");

        let mut valid_tiles_after_bounds_change = compute_valid_tiles_if_bounds_change(
            &image.visible_rect,
            visible_rect,
            image.tile_size,
        );

        match (image.valid_tiles_after_bounds_change, valid_tiles_after_bounds_change) {
            (Some(old), Some(ref mut new)) => {
                *new = new.intersection(&old).unwrap_or_else(TileRange::zero);
            }
            (Some(old), None) => {
                valid_tiles_after_bounds_change = Some(old);
            }
            _ => {}
        }

        let blob_size = visible_rect.size;

        if let Some(descriptor) = descriptor {
            image.descriptor = *descriptor;
        } else {
            // make sure the descriptor size matches the visible rect.
            // This might not be necessary but let's stay on the safe side.
            image.descriptor.size = blob_size;
        }

        if let Some(dirty_rect) = dirty_rect {
            image.dirty_rect = image.dirty_rect.union(dirty_rect);
        }

        image.valid_tiles_after_bounds_change = valid_tiles_after_bounds_change;
        image.visible_rect = *visible_rect;
    }

    pub fn create_blob_scene_builder_requests(
        &mut self,
        keys: &[BlobImageKey]
    ) -> (Option<Box<dyn AsyncBlobImageRasterizer>>, Vec<BlobImageParams>) {
        if self.blob_image_handler.is_none() || keys.is_empty() {
            return (None, Vec::new());
        }

        let mut blob_request_params = Vec::new();
        for key in keys {
            let template = self.blob_image_templates.get_mut(key).unwrap();

            // If we know that only a portion of the blob image is in the viewport,
            // only request these visible tiles since blob images can be huge.
            let tiles = compute_tile_range(
                &template.visible_rect,
                template.tile_size,
            );

            // Don't request tiles that weren't invalidated.
            let dirty_tiles = match template.dirty_rect {
                DirtyRect::Partial(dirty_rect) => {
                    compute_tile_range(
                        &dirty_rect.cast_unit(),
                        template.tile_size,
                    )
                }
                DirtyRect::All => tiles,
            };

            for_each_tile_in_range(&tiles, |tile| {
                let still_valid = template.valid_tiles_after_bounds_change
                    .map(|valid_tiles| valid_tiles.contains(tile))
                    .unwrap_or(true);

                if still_valid && !dirty_tiles.contains(tile) {
                    return;
                }

                let descriptor = BlobImageDescriptor {
                    rect: compute_tile_rect(
                        &template.visible_rect,
                        template.tile_size,
                        tile,
                    ).cast_unit(),
                    format: template.descriptor.format,
                };

                assert!(descriptor.rect.size.width > 0 && descriptor.rect.size.height > 0);
                blob_request_params.push(
                    BlobImageParams {
                        request: BlobImageRequest { key: *key, tile },
                        descriptor,
                        dirty_rect: DirtyRect::All,
                    }
                );
            });

            template.dirty_rect = DirtyRect::empty();
            template.valid_tiles_after_bounds_change = None;
        }

        let handler = self.blob_image_handler.as_mut().unwrap();
        handler.prepare_resources(&self.fonts, &blob_request_params);
        (Some(handler.create_blob_rasterizer()), blob_request_params)
    }
}
