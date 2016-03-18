/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_traits::{CanvasCommonMsg, CanvasMsg, CanvasPixelData, CanvasData, CanvasWebGLMsg};
use canvas_traits::{FromLayoutMsg, FromPaintMsg};
use euclid::size::Size2D;
use gleam::gl;
use ipc_channel::ipc::{self, IpcSender, IpcSharedMemory};
use ipc_channel::router::ROUTER;
use layers::platform::surface::NativeSurface;
use offscreen_gl_context::{ColorAttachmentType, GLContext, GLContextAttributes, NativeGLContext};
use std::borrow::ToOwned;
use std::sync::mpsc::{Sender, channel};
use util::thread::spawn_named;
use util::vec::byte_swap;
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
           webrender_api_sender: Option<webrender_traits::RenderApiSender>) -> Result<WebGLPaintThread, String> {
        let data = if let Some(sender) = webrender_api_sender {
            let webrender_api = sender.create_api();
            let id = try!(webrender_api.request_webgl_context(&size, attrs));
            WebGLPaintTaskData::WebRender(webrender_api, id)
        } else {
            let context = try!(GLContext::<NativeGLContext>::new(size, attrs, ColorAttachmentType::Texture, None));
            WebGLPaintTaskData::Servo(context)
        };

        Ok(WebGLPaintThread {
            size: size,
            data: data,
        })
    }

    pub fn handle_webgl_message(&self, message: CanvasWebGLMsg) {
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

    /// Creates a new `WebGLPaintThread` and returns the out-of-process sender and the in-process
    /// sender for it.
    pub fn start(size: Size2D<i32>,
                 attrs: GLContextAttributes,
                 webrender_api_sender: Option<webrender_traits::RenderApiSender>)
                 -> Result<(IpcSender<CanvasMsg>, Sender<CanvasMsg>), String> {
        let (in_process_chan, in_process_port) = channel();
        let (result_chan, result_port) = channel();
        spawn_named("WebGLThread".to_owned(), move || {
            let mut painter = match WebGLPaintThread::new(size, attrs, webrender_api_sender) {
                Ok(thread) => {
                    result_chan.send(Ok(())).unwrap();
                    thread
                },
                Err(e) => {
                    result_chan.send(Err(e)).unwrap();
                    return
                }
            };
            painter.init();
            loop {
                match in_process_port.recv().unwrap() {
                    CanvasMsg::WebGL(message) => painter.handle_webgl_message(message),
                    CanvasMsg::Common(message) => {
                        match message {
                            CanvasCommonMsg::Close => break,
                            // TODO(ecoal95): handle error nicely
                            CanvasCommonMsg::Recreate(size) => painter.recreate(size).unwrap(),
                        }
                    },
                    CanvasMsg::FromLayout(message) => {
                        match message {
                            FromLayoutMsg::SendData(chan) =>
                                painter.send_data(chan),
                        }
                    }
                    CanvasMsg::FromPaint(message) => {
                        match message {
                            FromPaintMsg::SendNativeSurface(chan) =>
                                painter.send_native_surface(chan),
                        }
                    }
                    CanvasMsg::Canvas2d(_) => panic!("Wrong message sent to WebGLThread"),
                }
            }
        });

        result_port.recv().unwrap().map(|_| {
             let (out_of_process_chan, out_of_process_port) = ipc::channel::<CanvasMsg>().unwrap();
             ROUTER.route_ipc_receiver_to_mpsc_sender(out_of_process_port, in_process_chan.clone());
             (out_of_process_chan, in_process_chan)
        })
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

    fn send_native_surface(&self, _: Sender<NativeSurface>) {
        // FIXME(ecoal95): We need to make a clone of the surface in order to
        // implement this
        unimplemented!()
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
