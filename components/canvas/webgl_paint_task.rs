/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use azure::azure::AzFloat;
use azure::azure_hl::{DrawTarget, SurfaceFormat, BackendType, DrawOptions, Filter, DrawSurfaceOptions};
use canvas_msg::CanvasMsg;
use compositing::windowing::{WindowMethods};
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;

use gleam::gl;
use gleam::gl::types::{GLint, GLsizei};

use layers::platform::surface::{NativeGraphicsMetadata, NativePaintingGraphicsContext};
use util::task::spawn_named;

use skia::SkiaGrGLNativeContextRef;
use std::borrow::ToOwned;
use std::sync::mpsc::{channel, Sender};
use util::vec::byte_swap;

pub struct WebGLPaintTask {
    size: Size2D<i32>,
    drawtarget: DrawTarget,
    native_graphics_context: NativePaintingGraphicsContext
}

impl WebGLPaintTask {
    fn new(size: Size2D<i32>, native_graphics_metadata: &NativeGraphicsMetadata) -> WebGLPaintTask {
        let native_graphics_context = NativePaintingGraphicsContext::from_metadata(&native_graphics_metadata);
        let draw_target = WebGLPaintTask::create(size, &native_graphics_context);
        WebGLPaintTask {
            size: size,
            drawtarget: draw_target,
            native_graphics_context: native_graphics_context,
        }
    }

    fn create(size: Size2D<i32>,
              native_graphics_context: &NativePaintingGraphicsContext) -> DrawTarget {
        let graphics_context = native_graphics_context as *const _ as SkiaGrGLNativeContextRef;
        let draw_target = DrawTarget::new_with_fbo(BackendType::Skia,
                                                   graphics_context,
                                                   size,
                                                   SurfaceFormat::B8G8R8A8);
        draw_target
    }

    pub fn start(size: Size2D<i32>,
                 native_metadata: NativeGraphicsMetadata) -> Sender<CanvasMsg> {
            let (chan, port) = channel::<CanvasMsg>();
        spawn_named("WebGLTask".to_owned(), move || {
            let native_metadata = native_metadata;
            let mut painter = WebGLPaintTask::new(size, &native_metadata);
            painter.init();
            loop {
                match port.recv().unwrap() {
                    CanvasMsg::Clear(mask) => {
                      painter.clear(mask);
                    },
                    CanvasMsg::ClearColor(r, g, b, a) => {
                      painter.clear_color(r, g, b, a);
                    },
                    CanvasMsg::Close => break,
                    CanvasMsg::Recreate(size) => painter.recreate(size),
                    CanvasMsg::Render => {
                      painter.render();
                    },
                    CanvasMsg::SendPixelContents(chan) => painter.send_pixel_contents(chan),
                    _ => panic!("Wrong message sent to WebGLTask"),
                }
            }
        });
        chan
    }

    fn init(&self) {
        let mut framebuffer_ids = vec!();
        let mut texture_ids = vec!();
        framebuffer_ids = gl::gen_framebuffers(1);
        gl::bind_framebuffer(gl::FRAMEBUFFER, framebuffer_ids[0]);

        texture_ids = gl::gen_textures(1);
        gl::bind_texture(gl::TEXTURE_2D, texture_ids[0]);

        gl::tex_image_2d(gl::TEXTURE_2D, 0, gl::RGB as GLint, self.size.width as GLsizei,
                         self.size.height as GLsizei, 0, gl::RGB, gl::UNSIGNED_BYTE, None);
        gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
        gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);

        gl::framebuffer_texture_2d(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D,
                                   texture_ids[0], 0);
        gl::bind_texture(gl::TEXTURE_2D, 0);

        gl::viewport(0 as GLint, 0 as GLint,
                     self.size.width as GLsizei, self.size.height as GLsizei);
    }

    fn clear(&self, mask: u32) {
        gl::clear(mask);
        self.render();
    }

    fn clear_color(&self, r: f32, g: f32, b: f32, a: f32) {
        gl::clear_color(r, g, b, a);
    }

    fn send_pixel_contents(&mut self, chan: Sender<Vec<u8>>) {
        self.drawtarget.snapshot().get_data_surface().with_data(|element| {
            chan.send(element.to_vec()).unwrap();
        })
    }

    fn recreate(&mut self, size: Size2D<i32>) {
        self.drawtarget = WebGLPaintTask::create(size,
                                                 &self.native_graphics_context);
    }

    fn render(&self) {
        let mut pixels = gl::read_pixels(0, 0,
                                    self.size.width as gl::GLsizei,
                                    self.size.height as gl::GLsizei,
                                    gl::RGBA, gl::UNSIGNED_BYTE);

        // rgba -> bgra
        byte_swap(pixels.as_mut_slice());

        let source_surface = self.drawtarget.create_source_surface_from_data(&pixels,
            self.size, self.size.width * 4, SurfaceFormat::B8G8R8A8);

        let canvas_rect = Rect(Point2D(0i32, 0i32), self.size);
        let draw_surface_options = DrawSurfaceOptions::new(Filter::Linear, true);
        let draw_options = DrawOptions::new(1.0f64 as AzFloat, 0);

        self.drawtarget.draw_surface(source_surface,
                                     canvas_rect.to_azfloat(),
                                     canvas_rect.to_azfloat(),
                                     draw_surface_options, draw_options);

    }

}

pub trait ToAzFloat {
    fn to_azfloat(&self) -> Rect<AzFloat>;
}

impl ToAzFloat for Rect<i32> {
    fn to_azfloat(&self) -> Rect<AzFloat> {
        Rect(Point2D(self.origin.x as AzFloat, self.origin.y as AzFloat),
             Size2D(self.size.width as AzFloat, self.size.height as AzFloat))
    }
}
