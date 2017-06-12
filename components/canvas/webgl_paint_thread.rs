/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_traits::{CanvasCommonMsg, CanvasData, CanvasMsg, CanvasImageData};
use canvas_traits::{FromLayoutMsg, FromScriptMsg, byte_swap};
use euclid::size::Size2D;
use gleam::gl;
use ipc_channel::ipc::{self, IpcSender};
use offscreen_gl_context::{ColorAttachmentType, GLContext, GLLimits};
use offscreen_gl_context::{GLContextAttributes, NativeGLContext, OSMesaContext};
use servo_config::opts;
use std::borrow::ToOwned;
use std::sync::Arc;
use std::sync::mpsc::channel;
use std::thread;
use webrender_traits;

enum GLContextWrapper {
    Native(GLContext<NativeGLContext>),
    OSMesa(GLContext<OSMesaContext>),
}

impl GLContextWrapper {
    fn new(size: Size2D<i32>,
           attributes: GLContextAttributes,
           gl_type: gl::GlType) -> Result<GLContextWrapper, &'static str> {
        if opts::get().should_use_osmesa() {
            let ctx = GLContext::<OSMesaContext>::new(size,
                                                      attributes,
                                                      ColorAttachmentType::Texture,
                                                      gl_type,
                                                      None);
            ctx.map(GLContextWrapper::OSMesa)
        } else {
            let ctx = GLContext::<NativeGLContext>::new(size,
                                                        attributes,
                                                        ColorAttachmentType::Texture,
                                                        gl_type,
                                                        None);
            ctx.map(GLContextWrapper::Native)
        }
    }

    pub fn get_limits(&self) -> GLLimits {
        match *self {
            GLContextWrapper::Native(ref ctx) => {
                ctx.borrow_limits().clone()
            }
            GLContextWrapper::OSMesa(ref ctx) => {
                ctx.borrow_limits().clone()
            }
        }
    }

    fn resize(&mut self, size: Size2D<i32>) -> Result<Size2D<i32>, &'static str> {
        match *self {
            GLContextWrapper::Native(ref mut ctx) => {
                try!(ctx.resize(size));
                Ok(ctx.borrow_draw_buffer().unwrap().size())
            }
            GLContextWrapper::OSMesa(ref mut ctx) => {
                try!(ctx.resize(size));
                Ok(ctx.borrow_draw_buffer().unwrap().size())
            }
        }
    }

    fn gl(&self) -> &gl::Gl {
        match *self {
            GLContextWrapper::Native(ref ctx) => {
                ctx.gl()
            }
            GLContextWrapper::OSMesa(ref ctx) => {
                ctx.gl()
            }
        }
    }

    pub fn make_current(&self) {
        match *self {
            GLContextWrapper::Native(ref ctx) => {
                ctx.make_current().unwrap();
            }
            GLContextWrapper::OSMesa(ref ctx) => {
                ctx.make_current().unwrap();
            }
        }
    }

    pub fn apply_command(&self, cmd: webrender_traits::WebGLCommand) {
        match *self {
            GLContextWrapper::Native(ref ctx) => {
                cmd.apply(ctx);
            }
            GLContextWrapper::OSMesa(ref ctx) => {
                cmd.apply(ctx);
            }
        }
    }
}

enum WebGLPaintTaskData {
    WebRender(webrender_traits::RenderApi, webrender_traits::WebGLContextId),
    Readback(GLContextWrapper, webrender_traits::RenderApi, Option<webrender_traits::ImageKey>),
}

pub struct WebGLPaintThread {
    size: Size2D<i32>,
    data: WebGLPaintTaskData,
}

fn create_readback_painter(size: Size2D<i32>,
                           attrs: GLContextAttributes,
                           webrender_api: webrender_traits::RenderApi,
                           gl_type: gl::GlType)
    -> Result<(WebGLPaintThread, GLLimits), String> {
    let context = try!(GLContextWrapper::new(size, attrs, gl_type));
    let limits = context.get_limits();
    let painter = WebGLPaintThread {
        size: size,
        data: WebGLPaintTaskData::Readback(context, webrender_api, None)
    };

    Ok((painter, limits))
}

impl WebGLPaintThread {
    fn new(size: Size2D<i32>,
           attrs: GLContextAttributes,
           webrender_api_sender: webrender_traits::RenderApiSender,
           gl_type: gl::GlType)
        -> Result<(WebGLPaintThread, GLLimits), String> {
        let wr_api = webrender_api_sender.create_api();
        let device_size = webrender_traits::DeviceIntSize::from_untyped(&size);
        match wr_api.request_webgl_context(&device_size, attrs) {
            Ok((id, limits)) => {
                let painter = WebGLPaintThread {
                    data: WebGLPaintTaskData::WebRender(wr_api, id),
                    size: size
                };
                Ok((painter, limits))
            },
            Err(msg) => {
                warn!("Initial context creation failed, falling back to readback: {}", msg);
                create_readback_painter(size, attrs, wr_api, gl_type)
            }
        }
    }

    fn handle_webgl_message(&self, message: webrender_traits::WebGLCommand) {
        debug!("WebGL message: {:?}", message);
        match self.data {
            WebGLPaintTaskData::WebRender(ref api, id) => {
                api.send_webgl_command(id, message);
            }
            WebGLPaintTaskData::Readback(ref ctx, _, _) => {
                ctx.apply_command(message);
            }
        }
    }

    fn handle_webvr_message(&self, message: webrender_traits::VRCompositorCommand) {
        match self.data {
            WebGLPaintTaskData::WebRender(ref api, id) => {
                api.send_vr_compositor_command(id, message);
            }
            WebGLPaintTaskData::Readback(..) => {
                error!("Webrender is required for WebVR implementation");
            }
        }
    }


    /// Creates a new `WebGLPaintThread` and returns an `IpcSender` to
    /// communicate with it.
    pub fn start(size: Size2D<i32>,
                 attrs: GLContextAttributes,
                 webrender_api_sender: webrender_traits::RenderApiSender)
                 -> Result<(IpcSender<CanvasMsg>, GLLimits), String> {
        let (sender, receiver) = ipc::channel::<CanvasMsg>().unwrap();
        let (result_chan, result_port) = channel();
        thread::Builder::new().name("WebGLThread".to_owned()).spawn(move || {
            let gl_type = gl::GlType::default();
            let mut painter = match WebGLPaintThread::new(size, attrs, webrender_api_sender, gl_type) {
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
                    CanvasMsg::FromScript(message) => {
                        match message {
                            FromScriptMsg::SendPixels(chan) =>{
                                // Read the comment on
                                // HTMLCanvasElement::fetch_all_data.
                                chan.send(None).unwrap();
                            }
                        }
                    }
                    CanvasMsg::FromLayout(message) => {
                        match message {
                            FromLayoutMsg::SendData(chan) =>
                                painter.send_data(chan),
                        }
                    }
                    CanvasMsg::Canvas2d(_) => panic!("Wrong message sent to WebGLThread"),
                    CanvasMsg::WebVR(message) => painter.handle_webvr_message(message)
                }
            }
        }).expect("Thread spawning failed");

        result_port.recv().unwrap().map(|limits| (sender, limits))
    }

    fn send_data(&mut self, chan: IpcSender<CanvasData>) {
        match self.data {
            WebGLPaintTaskData::Readback(ref ctx, ref webrender_api, ref mut image_key) => {
                let width = self.size.width as usize;
                let height = self.size.height as usize;

                let mut pixels = ctx.gl().read_pixels(0, 0,
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

                let descriptor = webrender_traits::ImageDescriptor {
                    width: width as u32,
                    height: height as u32,
                    stride: None,
                    format: webrender_traits::ImageFormat::RGBA8,
                    offset: 0,
                    is_opaque: false,
                };
                let data = webrender_traits::ImageData::Raw(Arc::new(pixels));

                match *image_key {
                    Some(image_key) => {
                        webrender_api.update_image(image_key,
                                                   descriptor,
                                                   data,
                                                   None);
                    }
                    None => {
                        *image_key = Some(webrender_api.generate_image_key());
                        webrender_api.add_image(image_key.unwrap(),
                                                descriptor,
                                                data,
                                                None);
                    }
                }

                let image_data = CanvasImageData {
                    image_key: image_key.unwrap(),
                };

                chan.send(CanvasData::Image(image_data)).unwrap();
            }
            WebGLPaintTaskData::WebRender(_, id) => {
                chan.send(CanvasData::WebGL(id)).unwrap();
            }
        }
    }

    #[allow(unsafe_code)]
    fn recreate(&mut self, size: Size2D<i32>) -> Result<(), &'static str> {
        match self.data {
            WebGLPaintTaskData::Readback(ref mut context, ref webrender_api, ref mut image_key) => {
                if size.width > self.size.width ||
                   size.height > self.size.height {
                    self.size = try!(context.resize(size));
                } else {
                    self.size = size;
                    context.gl().scissor(0, 0, size.width, size.height);
                }
                // Webrender doesn't let images change size, so we clear the webrender image key.
                if let Some(image_key) = image_key.take() {
                    webrender_api.delete_image(image_key);
                }
            }
            WebGLPaintTaskData::WebRender(ref api, id) => {
                let device_size = webrender_traits::DeviceIntSize::from_untyped(&size);
                api.resize_webgl_context(id, &device_size);
            }
        }

        Ok(())
    }

    fn init(&mut self) {
        if let WebGLPaintTaskData::Readback(ref context, _, _) = self.data {
            context.make_current();
        }
    }
}

impl Drop for WebGLPaintThread {
    fn drop(&mut self) {
        if let WebGLPaintTaskData::Readback(_, ref mut wr, image_key) = self.data {
            if let Some(image_key) = image_key {
                wr.delete_image(image_key);
            }
        }
    }
}
