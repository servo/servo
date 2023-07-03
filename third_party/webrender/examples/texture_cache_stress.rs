/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate gleam;
extern crate glutin;
extern crate webrender;
extern crate winit;

#[path = "common/boilerplate.rs"]
mod boilerplate;

use crate::boilerplate::{Example, HandyDandyRectBuilder};
use gleam::gl;
use std::mem;
use webrender::api::*;
use webrender::api::units::*;


struct ImageGenerator {
    patterns: [[u8; 3]; 6],
    next_pattern: usize,
    current_image: Vec<u8>,
}

impl ImageGenerator {
    fn new() -> Self {
        ImageGenerator {
            next_pattern: 0,
            patterns: [
                [1, 0, 0],
                [0, 1, 0],
                [0, 0, 1],
                [1, 1, 0],
                [0, 1, 1],
                [1, 0, 1],
            ],
            current_image: Vec::new(),
        }
    }

    fn generate_image(&mut self, size: i32) {
        let pattern = &self.patterns[self.next_pattern];
        self.current_image.clear();
        for y in 0 .. size {
            for x in 0 .. size {
                let lum = 255 * (1 - (((x & 8) == 0) ^ ((y & 8) == 0)) as u8);
                self.current_image.extend_from_slice(&[
                    lum * pattern[0],
                    lum * pattern[1],
                    lum * pattern[2],
                    0xff,
                ]);
            }
        }

        self.next_pattern = (self.next_pattern + 1) % self.patterns.len();
    }

    fn take(&mut self) -> Vec<u8> {
        mem::replace(&mut self.current_image, Vec::new())
    }
}

impl ExternalImageHandler for ImageGenerator {
    fn lock(
        &mut self,
        _key: ExternalImageId,
        channel_index: u8,
        _rendering: ImageRendering
    ) -> ExternalImage {
        self.generate_image(channel_index as i32);
        ExternalImage {
            uv: TexelRect::new(0.0, 0.0, 1.0, 1.0),
            source: ExternalImageSource::RawData(&self.current_image),
        }
    }
    fn unlock(&mut self, _key: ExternalImageId, _channel_index: u8) {}
}

struct App {
    stress_keys: Vec<ImageKey>,
    image_key: Option<ImageKey>,
    image_generator: ImageGenerator,
    swap_keys: Vec<ImageKey>,
    swap_index: usize,
}

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
        let bounds = (0, 0).to(512, 512);
        let space_and_clip = SpaceAndClipInfo::root_scroll(pipeline_id);

        builder.push_simple_stacking_context(
            bounds.origin,
            space_and_clip.spatial_id,
            PrimitiveFlags::IS_BACKFACE_VISIBLE,
        );

        let x0 = 50.0;
        let y0 = 50.0;
        let image_size = LayoutSize::new(4.0, 4.0);

        if self.swap_keys.is_empty() {
            let key0 = api.generate_image_key();
            let key1 = api.generate_image_key();

            self.image_generator.generate_image(128);
            txn.add_image(
                key0,
                ImageDescriptor::new(128, 128, ImageFormat::BGRA8, ImageDescriptorFlags::IS_OPAQUE),
                ImageData::new(self.image_generator.take()),
                None,
            );

            self.image_generator.generate_image(128);
            txn.add_image(
                key1,
                ImageDescriptor::new(128, 128, ImageFormat::BGRA8, ImageDescriptorFlags::IS_OPAQUE),
                ImageData::new(self.image_generator.take()),
                None,
            );

            self.swap_keys.push(key0);
            self.swap_keys.push(key1);
        }

        for (i, key) in self.stress_keys.iter().enumerate() {
            let x = (i % 128) as f32;
            let y = (i / 128) as f32;
            let info = CommonItemProperties::new(
                LayoutRect::new(
                    LayoutPoint::new(x0 + image_size.width * x, y0 + image_size.height * y),
                    image_size,
                ),
                space_and_clip,
            );

            builder.push_image(
                &info,
                bounds,
                ImageRendering::Auto,
                AlphaType::PremultipliedAlpha,
                *key,
                ColorF::WHITE,
            );
        }

        if let Some(image_key) = self.image_key {
            let image_size = LayoutSize::new(100.0, 100.0);
            let info = CommonItemProperties::new(
                LayoutRect::new(LayoutPoint::new(100.0, 100.0), image_size),
                space_and_clip,
            );
            builder.push_image(
                &info,
                bounds,
                ImageRendering::Auto,
                AlphaType::PremultipliedAlpha,
                image_key,
                ColorF::WHITE,
            );
        }

        let swap_key = self.swap_keys[self.swap_index];
        let image_size = LayoutSize::new(64.0, 64.0);
        let info = CommonItemProperties::new(
            LayoutRect::new(LayoutPoint::new(100.0, 400.0), image_size),
            space_and_clip,
        );
        builder.push_image(
            &info,
            bounds,
            ImageRendering::Auto,
            AlphaType::PremultipliedAlpha,
            swap_key,
            ColorF::WHITE,
        );
        self.swap_index = 1 - self.swap_index;

        builder.pop_stacking_context();
    }

    fn on_event(
        &mut self,
        event: winit::WindowEvent,
        api: &mut RenderApi,
        document_id: DocumentId,
    ) -> bool {
        match event {
            winit::WindowEvent::KeyboardInput {
                input: winit::KeyboardInput {
                    state: winit::ElementState::Pressed,
                    virtual_keycode: Some(key),
                    ..
                },
                ..
            } => {
                let mut txn = Transaction::new();

                match key {
                    winit::VirtualKeyCode::S => {
                        self.stress_keys.clear();

                        for _ in 0 .. 16 {
                            for _ in 0 .. 16 {
                                let size = 4;

                                let image_key = api.generate_image_key();

                                self.image_generator.generate_image(size);

                                txn.add_image(
                                    image_key,
                                    ImageDescriptor::new(
                                        size,
                                        size,
                                        ImageFormat::BGRA8,
                                        ImageDescriptorFlags::IS_OPAQUE,
                                    ),
                                    ImageData::new(self.image_generator.take()),
                                    None,
                                );

                                self.stress_keys.push(image_key);
                            }
                        }
                    }
                    winit::VirtualKeyCode::D => if let Some(image_key) = self.image_key.take() {
                        txn.delete_image(image_key);
                    },
                    winit::VirtualKeyCode::U => if let Some(image_key) = self.image_key {
                        let size = 128;
                        self.image_generator.generate_image(size);

                        txn.update_image(
                            image_key,
                            ImageDescriptor::new(size, size, ImageFormat::BGRA8, ImageDescriptorFlags::IS_OPAQUE),
                            ImageData::new(self.image_generator.take()),
                            &DirtyRect::All,
                        );
                    },
                    winit::VirtualKeyCode::E => {
                        if let Some(image_key) = self.image_key.take() {
                            txn.delete_image(image_key);
                        }

                        let size = 32;
                        let image_key = api.generate_image_key();

                        let image_data = ExternalImageData {
                            id: ExternalImageId(0),
                            channel_index: size as u8,
                            image_type: ExternalImageType::Buffer,
                        };

                        txn.add_image(
                            image_key,
                            ImageDescriptor::new(size, size, ImageFormat::BGRA8, ImageDescriptorFlags::IS_OPAQUE),
                            ImageData::External(image_data),
                            None,
                        );

                        self.image_key = Some(image_key);
                    }
                    winit::VirtualKeyCode::R => {
                        if let Some(image_key) = self.image_key.take() {
                            txn.delete_image(image_key);
                        }

                        let image_key = api.generate_image_key();
                        let size = 32;
                        self.image_generator.generate_image(size);

                        txn.add_image(
                            image_key,
                            ImageDescriptor::new(size, size, ImageFormat::BGRA8, ImageDescriptorFlags::IS_OPAQUE),
                            ImageData::new(self.image_generator.take()),
                            None,
                        );

                        self.image_key = Some(image_key);
                    }
                    _ => {}
                }

                api.send_transaction(document_id, txn);
                return true;
            }
            _ => {}
        }

        false
    }

    fn get_image_handlers(
        &mut self,
        _gl: &dyn gl::Gl,
    ) -> (Option<Box<dyn ExternalImageHandler>>,
          Option<Box<dyn OutputImageHandler>>) {
        (Some(Box::new(ImageGenerator::new())), None)
    }
}

fn main() {
    let mut app = App {
        image_key: None,
        stress_keys: Vec::new(),
        image_generator: ImageGenerator::new(),
        swap_keys: Vec::new(),
        swap_index: 0,
    };
    boilerplate::main_wrapper(&mut app, None);
}
