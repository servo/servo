/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_msg::{CanvasWebGLMsg, CanvasCommonMsg, CanvasMsg};
use geom::size::Size2D;

use gleam::gl;
use gleam::gl::types::{GLint, GLsizei};

use util::task::spawn_named;

use std::borrow::ToOwned;
use std::sync::mpsc::{channel, Sender};
use util::vec::byte_swap;

use glutin::{HeadlessRendererBuilder};

pub struct WebGLPaintTask {
    size: Size2D<i32>,
}

impl WebGLPaintTask {
    fn new(size: Size2D<i32>) -> WebGLPaintTask {
        WebGLPaintTask::create(size);
        WebGLPaintTask {
            size: size,
        }
    }

    pub fn start(size: Size2D<i32>) -> Sender<CanvasMsg> {
        let (chan, port) = channel::<CanvasMsg>();
        spawn_named("WebGLTask".to_owned(), move || {
            let mut painter = WebGLPaintTask::new(size);
            painter.init();
            loop {
                match port.recv().unwrap() {
                    CanvasMsg::WebGL(message) => {
                        match message {
                            CanvasWebGLMsg::Clear(mask) => painter.clear(mask),
                            CanvasWebGLMsg::ClearColor(r, g, b, a) => painter.clear_color(r, g, b, a),
                        }
                    },
                    CanvasMsg::Common(message) => {
                        match message {
                            CanvasCommonMsg::Close => break,
                            CanvasCommonMsg::SendPixelContents(chan) => painter.send_pixel_contents(chan),
                            CanvasCommonMsg::Recreate(size) => painter.recreate(size),
                        }
                    },
                    CanvasMsg::Canvas2d(_) => panic!("Wrong message sent to WebGLTask"),
                }
            }
        });
        chan
    }

    fn create(size: Size2D<i32>) {
        // It creates OpenGL context
        let context = HeadlessRendererBuilder::new(size.width as u32, size.height as u32).build().unwrap();
        unsafe {
            context.make_current();
        }
    }

    fn init(&self) {
        let framebuffer_ids = gl::gen_framebuffers(1);
        gl::bind_framebuffer(gl::FRAMEBUFFER, framebuffer_ids[0]);

        let texture_ids = gl::gen_textures(1);
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
    }

    fn clear_color(&self, r: f32, g: f32, b: f32, a: f32) {
        gl::clear_color(r, g, b, a);
    }

    fn send_pixel_contents(&mut self, chan: Sender<Vec<u8>>) {
        // FIXME(#5652, dmarcos) Instead of a readback strategy we have
        // to layerize the canvas
        let mut pixels = gl::read_pixels(0, 0,
                                    self.size.width as gl::GLsizei,
                                    self.size.height as gl::GLsizei,
                                    gl::RGBA, gl::UNSIGNED_BYTE);

        // rgba -> bgra
        byte_swap(pixels.as_mut_slice());
        chan.send(pixels).unwrap();
    }

    fn recreate(&mut self, size: Size2D<i32>) {
        self.size = size;
        self.init();
    }

}
