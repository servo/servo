/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate gleam;
extern crate glutin;
extern crate rayon;
extern crate webrender;
extern crate winit;

#[path = "common/boilerplate.rs"]
mod boilerplate;

use crate::boilerplate::{Example, HandyDandyRectBuilder};
use rayon::{ThreadPool, ThreadPoolBuilder};
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use webrender::api::{self, DisplayListBuilder, DocumentId, PipelineId, PrimitiveFlags, RenderApi, Transaction};
use webrender::api::{ColorF, CommonItemProperties, SpaceAndClipInfo, ImageDescriptorFlags};
use webrender::api::units::*;
use webrender::euclid::size2;

// This example shows how to implement a very basic BlobImageHandler that can only render
// a checkerboard pattern.

// The deserialized command list internally used by this example is just a color.
type ImageRenderingCommands = api::ColorU;

// Serialize/deserialize the blob.
// For real usecases you should probably use serde rather than doing it by hand.

fn serialize_blob(color: api::ColorU) -> Arc<Vec<u8>> {
    Arc::new(vec![color.r, color.g, color.b, color.a])
}

fn deserialize_blob(blob: &[u8]) -> Result<ImageRenderingCommands, ()> {
    let mut iter = blob.iter();
    return match (iter.next(), iter.next(), iter.next(), iter.next()) {
        (Some(&r), Some(&g), Some(&b), Some(&a)) => Ok(api::ColorU::new(r, g, b, a)),
        (Some(&a), None, None, None) => Ok(api::ColorU::new(a, a, a, a)),
        _ => Err(()),
    };
}

// This is the function that applies the deserialized drawing commands and generates
// actual image data.
fn render_blob(
    commands: Arc<ImageRenderingCommands>,
    descriptor: &api::BlobImageDescriptor,
    tile: TileOffset,
) -> api::BlobImageResult {
    let color = *commands;

    // Note: This implementation ignores the dirty rect which isn't incorrect
    // but is a missed optimization.

    // Allocate storage for the result. Right now the resource cache expects the
    // tiles to have have no stride or offset.
    let bpp = 4;
    let mut texels = Vec::with_capacity((descriptor.rect.size.area() * bpp) as usize);

    // Generate a per-tile pattern to see it in the demo. For a real use case it would not
    // make sense for the rendered content to depend on its tile.
    let tile_checker = (tile.x % 2 == 0) != (tile.y % 2 == 0);

    let [w, h] = descriptor.rect.size.to_array();
    let offset = descriptor.rect.origin;

    for y in 0..h {
        for x in 0..w {
            // Apply the tile's offset. This is important: all drawing commands should be
            // translated by this offset to give correct results with tiled blob images.
            let x2 = x + offset.x;
            let y2 = y + offset.y;

            // Render a simple checkerboard pattern
            let checker = if (x2 % 20 >= 10) != (y2 % 20 >= 10) {
                1
            } else {
                0
            };
            // ..nested in the per-tile checkerboard pattern
            let tc = if tile_checker { 0 } else { (1 - checker) * 40 };

            match descriptor.format {
                api::ImageFormat::BGRA8 => {
                    texels.push(color.b * checker + tc);
                    texels.push(color.g * checker + tc);
                    texels.push(color.r * checker + tc);
                    texels.push(color.a * checker + tc);
                }
                api::ImageFormat::R8 => {
                    texels.push(color.a * checker + tc);
                }
                _ => {
                    return Err(api::BlobImageError::Other(
                        format!("Unsupported image format"),
                    ));
                }
            }
        }
    }

    Ok(api::RasterizedBlobImage {
        data: Arc::new(texels),
        rasterized_rect: size2(w, h).into(),
    })
}

struct CheckerboardRenderer {
    // We are going to defer the rendering work to worker threads.
    // Using a pre-built Arc<ThreadPool> rather than creating our own threads
    // makes it possible to share the same thread pool as the glyph renderer (if we
    // want to).
    workers: Arc<ThreadPool>,

    // The deserialized drawing commands.
    // In this example we store them in Arcs. This isn't necessary since in this simplified
    // case the command list is a simple 32 bits value and would be cheap to clone before sending
    // to the workers. But in a more realistic scenario the commands would typically be bigger
    // and more expensive to clone, so let's pretend it is also the case here.
    image_cmds: HashMap<api::BlobImageKey, Arc<ImageRenderingCommands>>,
}

impl CheckerboardRenderer {
    fn new(workers: Arc<ThreadPool>) -> Self {
        CheckerboardRenderer {
            image_cmds: HashMap::new(),
            workers,
        }
    }
}

impl api::BlobImageHandler for CheckerboardRenderer {
    fn create_similar(&self) -> Box<dyn api::BlobImageHandler> {
        Box::new(CheckerboardRenderer::new(Arc::clone(&self.workers)))
    }

    fn add(&mut self, key: api::BlobImageKey, cmds: Arc<api::BlobImageData>,
           _visible_rect: &DeviceIntRect, _: api::TileSize) {
        self.image_cmds
            .insert(key, Arc::new(deserialize_blob(&cmds[..]).unwrap()));
    }

    fn update(&mut self, key: api::BlobImageKey, cmds: Arc<api::BlobImageData>,
              _visible_rect: &DeviceIntRect, _dirty_rect: &BlobDirtyRect) {
        // Here, updating is just replacing the current version of the commands with
        // the new one (no incremental updates).
        self.image_cmds
            .insert(key, Arc::new(deserialize_blob(&cmds[..]).unwrap()));
    }

    fn delete(&mut self, key: api::BlobImageKey) {
        self.image_cmds.remove(&key);
    }

    fn prepare_resources(
        &mut self,
        _services: &dyn api::BlobImageResources,
        _requests: &[api::BlobImageParams],
    ) {}

    fn enable_multithreading(&mut self, _: bool) {}
    fn delete_font(&mut self, _font: api::FontKey) {}
    fn delete_font_instance(&mut self, _instance: api::FontInstanceKey) {}
    fn clear_namespace(&mut self, _namespace: api::IdNamespace) {}
    fn create_blob_rasterizer(&mut self) -> Box<dyn api::AsyncBlobImageRasterizer> {
        Box::new(Rasterizer {
            workers: Arc::clone(&self.workers),
            image_cmds: self.image_cmds.clone(),
        })
    }
}

struct Rasterizer {
    workers: Arc<ThreadPool>,
    image_cmds: HashMap<api::BlobImageKey, Arc<ImageRenderingCommands>>,
}

impl api::AsyncBlobImageRasterizer for Rasterizer {
    fn rasterize(
        &mut self,
        requests: &[api::BlobImageParams],
        _low_priority: bool
    ) -> Vec<(api::BlobImageRequest, api::BlobImageResult)> {
        let requests: Vec<(&api::BlobImageParams, Arc<ImageRenderingCommands>)> = requests.into_iter().map(|params| {
            (params, Arc::clone(&self.image_cmds[&params.request.key]))
        }).collect();

        self.workers.install(|| {
            requests.into_par_iter().map(|(params, commands)| {
                (params.request, render_blob(commands, &params.descriptor, params.request.tile))
            }).collect()
        })
    }
}

struct App {}

impl Example for App {
    fn render(
        &mut self,
        api: &mut RenderApi,
        builder: &mut DisplayListBuilder,
        txn: &mut Transaction,
        _device_size: DeviceIntSize,
        pipeline_id: PipelineId,
        _document_id: DocumentId,
    ) {
        let space_and_clip = SpaceAndClipInfo::root_scroll(pipeline_id);

        builder.push_simple_stacking_context(
            LayoutPoint::zero(),
            space_and_clip.spatial_id,
            PrimitiveFlags::IS_BACKFACE_VISIBLE,
        );

        let size1 = DeviceIntSize::new(500, 500);
        let blob_img1 = api.generate_blob_image_key();
        txn.add_blob_image(
            blob_img1,
            api::ImageDescriptor::new(
                size1.width,
                size1.height,
                api::ImageFormat::BGRA8,
                ImageDescriptorFlags::IS_OPAQUE,
            ),
            serialize_blob(api::ColorU::new(50, 50, 150, 255)),
            size1.into(),
            Some(128),
        );
        let bounds = (30, 30).by(size1.width, size1.height);
        builder.push_image(
            &CommonItemProperties::new(bounds, space_and_clip),
            bounds,
            api::ImageRendering::Auto,
            api::AlphaType::PremultipliedAlpha,
            blob_img1.as_image(),
            ColorF::WHITE,
        );

        let size2 = DeviceIntSize::new(256, 256);
        let blob_img2 = api.generate_blob_image_key();
        txn.add_blob_image(
            blob_img2,
            api::ImageDescriptor::new(
                size2.width,
                size2.height,
                api::ImageFormat::BGRA8,
                ImageDescriptorFlags::IS_OPAQUE,
            ),
            serialize_blob(api::ColorU::new(50, 150, 50, 255)),
            size2.into(),
            None,
        );
        let bounds = (600, 600).by(size2.width, size2.height);
        builder.push_image(
            &CommonItemProperties::new(bounds, space_and_clip),
            bounds,
            api::ImageRendering::Auto,
            api::AlphaType::PremultipliedAlpha,
            blob_img2.as_image(),
            ColorF::WHITE,
        );

        builder.pop_stacking_context();
    }
}

fn main() {
    let workers =
        ThreadPoolBuilder::new().thread_name(|idx| format!("WebRender:Worker#{}", idx))
                                .build();

    let workers = Arc::new(workers.unwrap());

    let opts = webrender::RendererOptions {
        workers: Some(Arc::clone(&workers)),
        // Register our blob renderer, so that WebRender integrates it in the resource cache..
        // Share the same pool of worker threads between WebRender and our blob renderer.
        blob_image_handler: Some(Box::new(CheckerboardRenderer::new(Arc::clone(&workers)))),
        ..Default::default()
    };

    let mut app = App {};

    boilerplate::main_wrapper(&mut app, Some(opts));
}
