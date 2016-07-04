/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_traits::{CanvasCommonMsg, CanvasData, CanvasMsg, CanvasPixelData};
use canvas_traits::{FromLayoutMsg, byte_swap};
use euclid::size::Size2D;
use gleam::gl;
use ipc_channel::ipc::{self, IpcSender, IpcSharedMemory};
use offscreen_gl_context::{ColorAttachmentType, GLContext, GLLimits, GLContextAttributes, NativeGLContext};
use std::borrow::ToOwned;
use std::sync::mpsc::channel;
use util::thread::spawn_named;
use webrender_traits;

enum WebGLPaintTaskData {
    WebRender(webrender_traits::RenderApi, webrender_traits::WebGLContextId),
    Servo(GLContext<NativeGLContext>),
}

pub struct WebGLPaintThread {
    size: Size2D<i32>,
    data: WebGLPaintTaskData,
}

impl WebGLPaintThread {
    fn new(size: Size2D<i32>,
           attrs: GLContextAttributes,
           webrender_api_sender: Option<webrender_traits::RenderApiSender>)
        -> Result<(WebGLPaintThread, GLLimits), String> {
        let (data, limits) = if let Some(sender) = webrender_api_sender {
            let webrender_api = sender.create_api();
            let (id, limits) = try!(webrender_api.request_webgl_context(&size, attrs));
            (WebGLPaintTaskData::WebRender(webrender_api, id), limits)
        } else {
            let context = try!(GLContext::<NativeGLContext>::new(size, attrs, ColorAttachmentType::Texture, None));
            let limits = context.borrow_limits().clone();
            (WebGLPaintTaskData::Servo(context), limits)
        };

        let painter_object = WebGLPaintThread {
            size: size,
            data: data,
        };

        Ok((painter_object, limits))
    }

    pub fn handle_webgl_message(&self, message: webrender_traits::WebGLCommand) {
        debug!("WebGL message: {:?}", message);
        match self.data {
            WebGLPaintTaskData::WebRender(ref api, id) => {
                api.send_webgl_command(id, message);
            }
            WebGLPaintTaskData::Servo(ref ctx) => {
                message.apply(ctx);
            }
        }
    }

    /// Creates a new `WebGLPaintThread` and returns an `IpcSender` to
    /// communicate with it.
    pub fn start(size: Size2D<i32>,
                 attrs: GLContextAttributes,
                 webrender_api_sender: Option<webrender_traits::RenderApiSender>)
                 -> Result<(IpcSender<CanvasMsg>, GLLimits), String> {
        let (sender, receiver) = ipc::channel::<CanvasMsg>().unwrap();
        let (result_chan, result_port) = channel();
        spawn_named("WebGLThread".to_owned(), move || {
            let mut painter = match WebGLPaintThread::new(size, attrs, webrender_api_sender) {
                Ok((thread, limits)) => {
                    result_chan.send(Ok(limits)).unwrap();
                    thread
                },
                Err(e) => {
                    result_chan.send(Err(e)).unwrap();
                    return
                }
            };
            painter.init();
            loop {
                match receiver.recv().unwrap() {
                    CanvasMsg::WebGL(message) => painter.handle_webgl_message(message),
                    CanvasMsg::Common(message) => {
                        match message {
                            CanvasCommonMsg::Close => break,
                            // TODO(emilio): handle error nicely
                            CanvasCommonMsg::Recreate(size) => painter.recreate(size).unwrap(),
                        }
                    },
                    CanvasMsg::FromLayout(message) => {
                        match message {
                            FromLayoutMsg::SendData(chan) =>
                                painter.send_data(chan),
                        }
                    }
                    CanvasMsg::Canvas2d(_) => panic!("Wrong message sent to WebGLThread"),
                }
            }
        });

        result_port.recv().unwrap().map(|limits| (sender, limits))
    }

    fn send_data(&mut self, chan: IpcSender<CanvasData>) {
        match self.data {
            WebGLPaintTaskData::Servo(_) => {
                let width = self.size.width as usize;
                let height = self.size.height as usize;

                let mut pixels = gl::read_pixels(0, 0,
                                                 self.size.width as gl::GLsizei,
                                                 self.size.height as gl::GLsizei,
                                                 gl::RGBA, gl::UNSIGNED_BYTE);
                // flip image vertically (texture is upside down)
                let orig_pixels = pixels.clone();
                let stride = width * 4;
                for y in 0..height {
                    let dst_start = y * stride;
                    let src_start = (height - y - 1) * stride;
                    let src_slice = &orig_pixels[src_start .. src_start + stride];
                    (&mut pixels[dst_start .. dst_start + stride]).clone_from_slice(&src_slice[..stride]);
                }

                // rgba -> bgra
                byte_swap(&mut pixels);

                let pixel_data = CanvasPixelData {
                    image_data: IpcSharedMemory::from_bytes(&pixels[..]),
                    image_key: None,
                };

                chan.send(CanvasData::Pixels(pixel_data)).unwrap();
            }
            WebGLPaintTaskData::WebRender(_, id) => {
                chan.send(CanvasData::WebGL(id)).unwrap();
            }
        }
    }

    #[allow(unsafe_code)]
    fn recreate(&mut self, size: Size2D<i32>) -> Result<(), &'static str> {
        match self.data {
            WebGLPaintTaskData::Servo(ref mut context) => {
                if size.width > self.size.width ||
                   size.height > self.size.height {
                    try!(context.resize(size));
                    self.size = context.borrow_draw_buffer().unwrap().size();
                } else {
                    self.size = size;
                    unsafe { gl::Scissor(0, 0, size.width, size.height); }
                }
            }
            WebGLPaintTaskData::WebRender(_, _) => {
                // TODO
            }
        }

        Ok(())
    }

    fn init(&mut self) {
        if let WebGLPaintTaskData::Servo(ref context) = self.data {
            context.make_current().unwrap();
        }
    }
}
