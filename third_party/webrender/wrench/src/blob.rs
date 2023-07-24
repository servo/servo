/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// A very basic BlobImageRasterizer that can only render a checkerboard pattern.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use webrender::api::*;
use webrender::api::units::{BlobDirtyRect, BlobToDeviceTranslation, TileOffset};
use webrender::api::units::DeviceIntRect;

// Serialize/deserialize the blob.

pub fn serialize_blob(color: ColorU) -> Arc<Vec<u8>> {
    Arc::new(vec![color.r, color.g, color.b, color.a])
}

fn deserialize_blob(blob: &[u8]) -> Result<ColorU, ()> {
    let mut iter = blob.iter();
    return match (iter.next(), iter.next(), iter.next(), iter.next()) {
        (Some(&r), Some(&g), Some(&b), Some(&a)) => Ok(ColorU::new(r, g, b, a)),
        (Some(&a), None, None, None) => Ok(ColorU::new(a, a, a, a)),
        _ => Err(()),
    };
}

// perform floor((x * a) / 255. + 0.5) see "Three wrongs make a right" for derivation
fn premul(x: u8, a: u8) -> u8 {
    let t = (x as u32) * (a as u32) + 128;
    ((t + (t >> 8)) >> 8) as u8
}

// This is the function that applies the deserialized drawing commands and generates
// actual image data.
fn render_blob(
    color: ColorU,
    descriptor: &BlobImageDescriptor,
    tile: TileOffset,
    _tile_size: TileSize,
    dirty_rect: &BlobDirtyRect,
) -> BlobImageResult {
    // Allocate storage for the result. Right now the resource cache expects the
    // tiles to have have no stride or offset.
    let buf_size = descriptor.rect.size.area() *
        descriptor.format.bytes_per_pixel();
    let mut texels = vec![0u8; (buf_size) as usize];

    // Generate a per-tile pattern to see it in the demo. For a real use case it would not
    // make sense for the rendered content to depend on its tile.
    let tile_checker = (tile.x % 2 == 0) != (tile.y % 2 == 0);

    let dirty_rect = dirty_rect.to_subrect_of(&descriptor.rect);

    // We want the dirty rect local to the tile rather than the whole image.
    let tx: BlobToDeviceTranslation = (-descriptor.rect.origin.to_vector()).into();

    let rasterized_rect = tx.transform_rect(&dirty_rect);

    for y in rasterized_rect.min_y() .. rasterized_rect.max_y() {
        for x in rasterized_rect.min_x() .. rasterized_rect.max_x() {
            // Apply the tile's offset. This is important: all drawing commands should be
            // translated by this offset to give correct results with tiled blob images.
            let x2 = x + descriptor.rect.origin.x;
            let y2 = y + descriptor.rect.origin.y;

            // Render a simple checkerboard pattern
            let checker = if (x2 % 20 >= 10) != (y2 % 20 >= 10) {
                1
            } else {
                0
            };
            // ..nested in the per-tile checkerboard pattern
            let tc = if tile_checker { 0 } else { (1 - checker) * 40 };

            match descriptor.format {
                ImageFormat::BGRA8 => {
                    let a = color.a * checker + tc;
                    let pixel_offset = ((y * descriptor.rect.size.width + x) * 4) as usize;
                    texels[pixel_offset + 0] = premul(color.b * checker + tc, a);
                    texels[pixel_offset + 1] = premul(color.g * checker + tc, a);
                    texels[pixel_offset + 2] = premul(color.r * checker + tc, a);
                    texels[pixel_offset + 3] = a;
                }
                ImageFormat::R8 => {
                    texels[(y * descriptor.rect.size.width + x) as usize] = color.a * checker + tc;
                }
                _ => {
                    return Err(BlobImageError::Other(
                        format!("Unsupported image format {:?}", descriptor.format),
                    ));
                }
            }
        }
    }

    Ok(RasterizedBlobImage {
        data: Arc::new(texels),
        rasterized_rect,
    })
}

/// See rawtest.rs. We use this to test that blob images are requested the right
/// amount of times.
pub struct BlobCallbacks {
    pub request: Box<dyn Fn(&[BlobImageParams]) + Send + 'static>,
}

impl BlobCallbacks {
    pub fn new() -> Self {
        BlobCallbacks { request: Box::new(|_|()) }
    }
}

pub struct CheckerboardRenderer {
    image_cmds: HashMap<BlobImageKey, (ColorU, TileSize)>,
    callbacks: Arc<Mutex<BlobCallbacks>>,
}

impl CheckerboardRenderer {
    pub fn new(callbacks: Arc<Mutex<BlobCallbacks>>) -> Self {
        CheckerboardRenderer {
            callbacks,
            image_cmds: HashMap::new(),
        }
    }
}

impl BlobImageHandler for CheckerboardRenderer {
    fn create_similar(&self) -> Box<dyn BlobImageHandler> {
        Box::new(CheckerboardRenderer::new(Arc::clone(&self.callbacks)))
    }

    fn add(&mut self, key: BlobImageKey, cmds: Arc<BlobImageData>,
           _visible_rect: &DeviceIntRect, tile_size: TileSize) {
        self.image_cmds
            .insert(key, (deserialize_blob(&cmds[..]).unwrap(), tile_size));
    }

    fn update(&mut self, key: BlobImageKey, cmds: Arc<BlobImageData>,
              _visible_rect: &DeviceIntRect, _dirty_rect: &BlobDirtyRect) {
        // Here, updating is just replacing the current version of the commands with
        // the new one (no incremental updates).
        self.image_cmds.get_mut(&key).unwrap().0 = deserialize_blob(&cmds[..]).unwrap();
    }

    fn delete(&mut self, key: BlobImageKey) {
        self.image_cmds.remove(&key);
    }

    fn delete_font(&mut self, _key: FontKey) {}

    fn delete_font_instance(&mut self, _key: FontInstanceKey) {}

    fn clear_namespace(&mut self, _namespace: IdNamespace) {}

    fn prepare_resources(
        &mut self,
        _services: &dyn BlobImageResources,
        requests: &[BlobImageParams],
    ) {
        if !requests.is_empty() {
            (self.callbacks.lock().unwrap().request)(&requests);
        }
    }

    fn create_blob_rasterizer(&mut self) -> Box<dyn AsyncBlobImageRasterizer> {
        Box::new(Rasterizer { image_cmds: self.image_cmds.clone() })
    }

    fn enable_multithreading(&mut self, _enable: bool) {}
}

struct Command {
    request: BlobImageRequest,
    color: ColorU,
    descriptor: BlobImageDescriptor,
    tile: TileOffset,
    tile_size: TileSize,
    dirty_rect: BlobDirtyRect,
}

struct Rasterizer {
    image_cmds: HashMap<BlobImageKey, (ColorU, TileSize)>,
}

impl AsyncBlobImageRasterizer for Rasterizer {
    fn rasterize(
        &mut self,
        requests: &[BlobImageParams],
        _low_priority: bool
    ) -> Vec<(BlobImageRequest, BlobImageResult)> {
        let requests: Vec<Command> = requests.into_iter().map(
            |item| {
                let (color, tile_size) = self.image_cmds[&item.request.key];

                Command {
                    request: item.request,
                    color,
                    tile_size,
                    tile: item.request.tile,
                    descriptor: item.descriptor,
                    dirty_rect: item.dirty_rect,
                }
            }
        ).collect();

        requests.iter().map(|cmd| {
            (cmd.request, render_blob(cmd.color, &cmd.descriptor, cmd.tile, cmd.tile_size, &cmd.dirty_rect))
        }).collect()
    }
}
