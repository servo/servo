/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::{BlobImageKey, ImageDescriptor, DirtyRect, TileSize, ResourceUpdate};
use crate::{BlobImageHandler, AsyncBlobImageRasterizer, BlobImageData, BlobImageParams};
use crate::{BlobImageRequest, BlobImageDescriptor, BlobImageResources, TransactionMsg};
use crate::{FontKey, FontTemplate, FontInstanceData, FontInstanceKey, AddFont};
use crate::image_tiling::*;
use crate::units::*;
use crate::font::SharedFontInstanceMap;
use crate::euclid::{point2, size2};

pub const DEFAULT_TILE_SIZE: TileSize = 512;

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
                    self.blob_image_templates.remove(&key);
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
                    self.fonts.templates.remove(&key);
                    if let Some(ref mut handler) = self.blob_image_handler {
                        handler.delete_font(key);
                    }
                }
                ResourceUpdate::DeleteFontInstance(key) => {
                    // We will delete from the shared font instance map in the resource cache
                    // after scene swap.

                    if let Some(ref mut r) = self.blob_image_handler {
                        r.delete_font_instance(key);
                    }
                }
                _ => {}
            }
        }

        let (rasterizer, requests) = self.create_blob_scene_builder_requests(&blobs_to_rasterize);
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

fn compute_valid_tiles_if_bounds_change(
    prev_rect: &DeviceIntRect,
    new_rect: &DeviceIntRect,
    tile_size: u16,
) -> Option<TileRange> {
    let intersection = match prev_rect.intersection(new_rect) {
        Some(rect) => rect,
        None => {
            return Some(TileRange::zero());
        }
    };

    let left = prev_rect.min_x() != new_rect.min_x();
    let right = prev_rect.max_x() != new_rect.max_x();
    let top = prev_rect.min_y() != new_rect.min_y();
    let bottom = prev_rect.max_y() != new_rect.max_y();

    if !left && !right && !top && !bottom {
        // Bounds have not changed.
        return None;
    }

    let tw = 1.0 / (tile_size as f32);
    let th = 1.0 / (tile_size as f32);

    let tiles = intersection
        .cast::<f32>()
        .scale(tw, th);

    let min_x = if left { f32::ceil(tiles.min_x()) } else { f32::floor(tiles.min_x()) };
    let min_y = if top { f32::ceil(tiles.min_y()) } else { f32::floor(tiles.min_y()) };
    let max_x = if right { f32::floor(tiles.max_x()) } else { f32::ceil(tiles.max_x()) };
    let max_y = if bottom { f32::floor(tiles.max_y()) } else { f32::ceil(tiles.max_y()) };

    Some(TileRange {
        origin: point2(min_x as i32, min_y as i32),
        size: size2((max_x - min_x) as i32, (max_y - min_y) as i32),
    })
}
