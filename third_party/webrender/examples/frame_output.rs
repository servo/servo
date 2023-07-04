/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate euclid;
extern crate gleam;
extern crate glutin;
extern crate webrender;
extern crate winit;

#[path = "common/boilerplate.rs"]
mod boilerplate;

use crate::boilerplate::{Example, HandyDandyRectBuilder};
use euclid::Scale;
use gleam::gl;
use webrender::api::*;
use webrender::api::units::*;


// This example demonstrates using the frame output feature to copy
// the output of a WR framebuffer to a custom texture.

#[derive(Debug)]
struct Document {
    id: DocumentId,
    pipeline_id: PipelineId,
    content_rect: LayoutRect,
    color: ColorF,
}


struct App {
    external_image_key: Option<ImageKey>,
    output_document: Option<Document>
}

struct OutputHandler {
    texture_id: gl::GLuint
}

struct ExternalHandler {
    texture_id: gl::GLuint
}

impl OutputImageHandler for OutputHandler {
    fn lock(&mut self, _id: PipelineId) -> Option<(u32, FramebufferIntSize)> {
        Some((self.texture_id, FramebufferIntSize::new(500, 500)))
    }

    fn unlock(&mut self, _id: PipelineId) {}
}

impl ExternalImageHandler for ExternalHandler {
    fn lock(
        &mut self,
        _key: ExternalImageId,
        _channel_index: u8,
        _rendering: ImageRendering
    ) -> ExternalImage {
        ExternalImage {
            uv: TexelRect::new(0.0, 0.0, 1.0, 1.0),
            source: ExternalImageSource::NativeTexture(self.texture_id),
        }
    }
    fn unlock(&mut self, _key: ExternalImageId, _channel_index: u8) {}
}

impl App {
    fn init_output_document(
        &mut self,
        api: &mut RenderApi,
        device_size: DeviceIntSize,
        device_pixel_ratio: f32,
    ) {
        // Generate the external image key that will be used to render the output document to the root document.
        self.external_image_key = Some(api.generate_image_key());

        let pipeline_id = PipelineId(1, 0);
        let layer = 1;
        let color = ColorF::new(1., 1., 0., 1.);
        let document_id = api.add_document(device_size, layer);
        api.enable_frame_output(document_id, pipeline_id, true);
        api.set_document_view(
            document_id,
            device_size.into(),
            device_pixel_ratio,
        );

        let document = Document {
            id: document_id,
            pipeline_id,
            content_rect: LayoutRect::new(
                LayoutPoint::zero(),
                device_size.to_f32() / Scale::new(device_pixel_ratio),
            ),
            color,
        };

        let mut txn = Transaction::new();

        txn.add_image(
            self.external_image_key.unwrap(),
            ImageDescriptor::new(100, 100, ImageFormat::BGRA8, ImageDescriptorFlags::IS_OPAQUE),
            ImageData::External(ExternalImageData {
                id: ExternalImageId(0),
                channel_index: 0,
                image_type: ExternalImageType::TextureHandle(TextureTarget::Default),
            }),
            None,
        );

        let space_and_clip = SpaceAndClipInfo::root_scroll(pipeline_id);
        let mut builder = DisplayListBuilder::new(
            document.pipeline_id,
            document.content_rect.size,
        );

        builder.push_simple_stacking_context(
            document.content_rect.origin,
            space_and_clip.spatial_id,
            PrimitiveFlags::IS_BACKFACE_VISIBLE,
        );

        builder.push_rect(
            &CommonItemProperties::new(document.content_rect, space_and_clip),
            document.content_rect,
            ColorF::new(1.0, 1.0, 0.0, 1.0)
        );
        builder.pop_stacking_context();

        txn.set_root_pipeline(pipeline_id);
        txn.set_display_list(
            Epoch(0),
            Some(document.color),
            document.content_rect.size,
            builder.finalize(),
            true,
        );
        txn.generate_frame();
        api.send_transaction(document.id, txn);
        self.output_document = Some(document);
    }
}

impl Example for App {
    fn render(
        &mut self,
        api: &mut RenderApi,
        builder: &mut DisplayListBuilder,
        _txn: &mut Transaction,
        device_size: DeviceIntSize,
        pipeline_id: PipelineId,
        _document_id: DocumentId,
    ) {
        if self.output_document.is_none() {
            let device_pixel_ratio = device_size.width as f32 /
                builder.content_size().width;
            self.init_output_document(api, DeviceIntSize::new(200, 200), device_pixel_ratio);
        }

        let bounds = (100, 100).to(200, 200);
        let space_and_clip = SpaceAndClipInfo::root_scroll(pipeline_id);

        builder.push_simple_stacking_context(
            bounds.origin,
            space_and_clip.spatial_id,
            PrimitiveFlags::IS_BACKFACE_VISIBLE,
        );

        builder.push_image(
            &CommonItemProperties::new(bounds, space_and_clip),
            bounds,
            ImageRendering::Auto,
            AlphaType::PremultipliedAlpha,
            self.external_image_key.unwrap(),
            ColorF::WHITE,
        );

        builder.pop_stacking_context();
    }

    fn get_image_handlers(
        &mut self,
        gl: &dyn gl::Gl,
    ) -> (Option<Box<dyn ExternalImageHandler>>,
          Option<Box<dyn OutputImageHandler>>) {
        let texture_id = gl.gen_textures(1)[0];

        gl.bind_texture(gl::TEXTURE_2D, texture_id);
        gl.tex_parameter_i(
            gl::TEXTURE_2D,
            gl::TEXTURE_MAG_FILTER,
            gl::LINEAR as gl::GLint,
        );
        gl.tex_parameter_i(
            gl::TEXTURE_2D,
            gl::TEXTURE_MIN_FILTER,
            gl::LINEAR as gl::GLint,
        );
        gl.tex_parameter_i(
            gl::TEXTURE_2D,
            gl::TEXTURE_WRAP_S,
            gl::CLAMP_TO_EDGE as gl::GLint,
        );
        gl.tex_parameter_i(
            gl::TEXTURE_2D,
            gl::TEXTURE_WRAP_T,
            gl::CLAMP_TO_EDGE as gl::GLint,
        );
        gl.tex_image_2d(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as gl::GLint,
            100,
            100,
            0,
            gl::BGRA,
            gl::UNSIGNED_BYTE,
            None,
        );
        gl.bind_texture(gl::TEXTURE_2D, 0);

        (
            Some(Box::new(ExternalHandler { texture_id })),
            Some(Box::new(OutputHandler { texture_id }))
        )
    }
}

fn main() {
    let mut app = App {
        external_image_key: None,
        output_document: None
    };

    boilerplate::main_wrapper(&mut app, None);
}
