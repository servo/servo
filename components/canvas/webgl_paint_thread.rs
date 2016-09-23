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
    WebRender(webrender_traits::RenderApi, webrender_traits::WebGLContextId, Option<webrender_traits::ImageKey>),
    Readback(GLContext<NativeGLContext>),
}

pub struct WebGLPaintThread {
    size: Size2D<i32>,
    data: WebGLPaintTaskData,
}

fn create_readback_painter(size: Size2D<i32>, attrs: GLContextAttributes)
                            -> Result<(WebGLPaintThread, GLLimits), String> {
    let context = try!(GLContext::<NativeGLContext>::new(size, attrs, ColorAttachmentType::Texture, None));
    let limits = context.borrow_limits().clone();
    let painter = WebGLPaintThread {
        size: size,
        data: WebGLPaintTaskData::Readback(context)
    };

    Ok((painter, limits))
}

impl WebGLPaintThread {
    fn new(size: Size2D<i32>,
           attrs: GLContextAttributes,
           webrender_api_sender: Option<webrender_traits::RenderApiSender>)
        -> Result<(WebGLPaintThread, GLLimits), String> {
        if let Some(sender) = webrender_api_sender {
            let wr_api = sender.create_api();
            let (id, limits, sharing) = try!(wr_api.request_webgl_context(&size, attrs));
            let image_key = match sharing {
                webrender_traits::ContextSharing::Shared => None,
                webrender_traits::ContextSharing::NotShared => Some(wr_api.alloc_image()),
            };

            let painter = WebGLPaintThread {
                data: WebGLPaintTaskData::WebRender(wr_api, id, image_key),
                size: size
            };
            Ok((painter, limits))
        } else {
            create_readback_painter(size, attrs)
        }
    }

    fn handle_webgl_message(&self, message: webrender_traits::WebGLCommand) {
        debug!("WebGL message: {:?}", message);
        match self.data {
            WebGLPaintTaskData::WebRender(ref api, id, _) => {
                api.send_webgl_command(id, message);
            }
            WebGLPaintTaskData::Readback(ref ctx) => {
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

    fn read_back_pixels(&self) -> Vec<u8> {
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

        pixels
    }

    fn send_data(&mut self, chan: IpcSender<CanvasData>) {
        match self.data {
            WebGLPaintTaskData::Readback(..) => {
                let pixels = self.read_back_pixels();

                let pixel_data = CanvasPixelData {
                    image_data: IpcSharedMemory::from_bytes(&pixels[..]),
                    image_key: None,
                };

                chan.send(CanvasData::Pixels(pixel_data)).unwrap();
            }
            WebGLPaintTaskData::WebRender(ref wr_api, _, Some(image_key)) => {
                let pixels = self.read_back_pixels();

                // TODO: This shouldn't be a common path, but try to avoid
                // the spurious clone().
                wr_api.update_image(image_key,
                                    self.size.width as u32,
                                    self.size.height as u32,
                                    webrender_traits::ImageFormat::RGBA8,
                                    pixels.clone());

                let pixel_data = CanvasPixelData {
                    image_data: IpcSharedMemory::from_bytes(&pixels[..]),
                    image_key: Some(image_key),
                };

                chan.send(CanvasData::Pixels(pixel_data)).unwrap();
            }
            WebGLPaintTaskData::WebRender(_, id, None) => {
                chan.send(CanvasData::WebGL(id)).unwrap();
            }
        }
    }

    #[allow(unsafe_code)]
    fn recreate(&mut self, size: Size2D<i32>) -> Result<(), &'static str> {
        match self.data {
            WebGLPaintTaskData::Readback(ref mut context) => {
                if size.width > self.size.width ||
                   size.height > self.size.height {
                    try!(context.resize(size));
                    self.size = context.borrow_draw_buffer().unwrap().size();
                } else {
                    self.size = size;
                    unsafe { gl::Scissor(0, 0, size.width, size.height); }
                }
            }
            WebGLPaintTaskData::WebRender(..) => {
                // TODO
            }
        }

        Ok(())
    }

    fn init(&mut self) {
        if let WebGLPaintTaskData::Readback(ref context) = self.data {
            context.make_current().unwrap();
        }
    }
}

impl Drop for WebGLPaintThread {
    fn drop(&mut self) {
        if let WebGLPaintTaskData::WebRender(ref wr, _, Some(image_key)) = self.data {
            wr.delete_image(image_key);
        }
    }
}
