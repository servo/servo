/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_msg::{CanvasMsg, CanvasWebGLMsg, CanvasCommonMsg};
use geom::size::Size2D;

use gleam::gl;
use gleam::gl::types::{GLsizei};

use util::task::spawn_named;

use std::borrow::ToOwned;
use std::sync::mpsc::{channel, Sender};
use util::vec::byte_swap;
use offscreen_gl_context::{GLContext, GLContextAttributes};

use glutin::{HeadlessRendererBuilder, HeadlessContext};

// FIXME(ecoal95): We use glutin as a fallback until GLContext support improves.
enum PlatformIndependentContext {
    GLContext(GLContext),
    GlutinContext(HeadlessContext),
}

impl PlatformIndependentContext {
    fn make_current(&self) {
        match *self {
            PlatformIndependentContext::GLContext(ref ctx) => ctx.make_current().unwrap(),
            PlatformIndependentContext::GlutinContext(ref ctx) => unsafe { ctx.make_current() }
        }
    }
}

fn create_offscreen_context(size: Size2D<i32>, attrs: GLContextAttributes) -> Result<PlatformIndependentContext, &'static str> {
    match GLContext::create_offscreen(size, attrs) {
        Ok(ctx) => Ok(PlatformIndependentContext::GLContext(ctx)),
        Err(msg) => {
            debug!("GLContext creation error: {}", msg);
            match HeadlessRendererBuilder::new(size.width as u32, size.height as u32).build() {
                Ok(ctx) => Ok(PlatformIndependentContext::GlutinContext(ctx)),
                Err(_) => Err("Glutin headless context creation failed")
            }
        }
    }
}

pub struct WebGLPaintTask {
    size: Size2D<i32>,
    original_context_size: Size2D<i32>,
    gl_context: PlatformIndependentContext,
}

// This allows trying to create the PaintTask
// before creating the thread
unsafe impl Send for WebGLPaintTask {}

impl WebGLPaintTask {
    fn new(size: Size2D<i32>) -> Result<WebGLPaintTask, &'static str> {
        let context = try!(create_offscreen_context(size, GLContextAttributes::default()));
        Ok(WebGLPaintTask {
            size: size,
            original_context_size: size,
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

        Ok(chan)
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
        byte_swap(&mut pixels);
        chan.send(pixels).unwrap();
    }

    fn recreate(&mut self, size: Size2D<i32>) {
        // TODO(ecoal95): GLContext should support a resize() method
        if size.width > self.original_context_size.width ||
           size.height > self.original_context_size.height {
            panic!("Can't grow a GLContext (yet)");
        } else {
            // Right now we just crop the viewport, it will do the job
            self.size = size;
            gl::viewport(0, 0, size.width, size.height);
            unsafe { gl::Scissor(0, 0, size.width, size.height); }
        }
    }

    fn init(&mut self) {
        self.gl_context.make_current();
    }
}
