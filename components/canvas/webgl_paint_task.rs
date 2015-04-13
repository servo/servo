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

pub struct WebGLPaintTask {
    size: Size2D<i32>,
    gl_context: GLContext
}

// This allows trying to create the PaintTask
// before creating the thread
unsafe impl Send for WebGLPaintTask {}

impl WebGLPaintTask {
    fn new(size: Size2D<i32>) -> Result<WebGLPaintTask, &'static str> {
        // TODO(ecoal95): Handle error nicely instead of `unwrap` (make getContext return null)
        //   Maybe allowing Send on WebGLPaintTask?
        let context = try!(GLContext::create_offscreen(size));
        Ok(WebGLPaintTask {
            size: size,
            gl_context: context
        })
    }

    pub fn start(size: Size2D<i32>) -> Result<Sender<CanvasMsg>, &'static str> {
        let (chan, port) = channel::<CanvasMsg>();
        let mut painter = try!(WebGLPaintTask::new(size));
        spawn_named("WebGLTask".to_owned(), move || {
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
                    CanvasMsg::SendPixelContents(chan) => painter.send_pixel_contents(chan),
                    _ => panic!("Wrong message sent to WebGLTask"),
                }
            }
        });

        Ok(chan)
    }

    fn clear(&self, mask: u32) {
        gl::clear(mask);
    }

    fn clear_color(&self, r: f32, g: f32, b: f32, a: f32) {
        gl::clear_color(r, g, b, a);
    }

    fn send_pixel_contents(&mut self, chan: Sender<Vec<u8>>) {
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
