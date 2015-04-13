/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_msg::CanvasMsg;
use geom::size::Size2D;

use gleam::gl;
use gleam::gl::types::{GLsizei};

use util::task::spawn_named;

use std::borrow::ToOwned;
use std::sync::mpsc::{channel, Sender};
use util::vec::byte_swap;
use offscreen_gl_context::{GLContext, GLContextMethods};

use glutin::{HeadlessRendererBuilder};

pub struct WebGLPaintTask {
    size: Size2D<i32>,
    gl_context: GLContext
}

impl WebGLPaintTask {
    fn new(size: Size2D<i32>) -> WebGLPaintTask {
        // TODO(ecoal95): Handle error nicely instead of `unwrap` (make getContext return null)
        //   Maybe allowing Send on WebGLPaintTask?
        let context = GLContext::create_offscreen(size).unwrap();
        WebGLPaintTask {
            size: size,
            gl_context: context
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

        // Fixed by https://github.com/servo/gleam/pull/17
        unsafe {
            let len = pixels.len() * 4 / 3;
            pixels.set_len(len);
        };

        // rgba -> bgra
        byte_swap(pixels.as_mut_slice());
        chan.send(pixels).unwrap();
    }

    fn recreate(&mut self, size: Size2D<i32>) {
        // FIXME(ecoal95): Resizing properly: This just works for less size than when it was
        // created
        self.size = size;
        gl::viewport(0, 0, size.width, size.height);
        unsafe { gl::Scissor(0, 0, size.width, size.height); }
    }

    fn init(&mut self) {
        self.gl_context.make_current().unwrap();
    }

}
