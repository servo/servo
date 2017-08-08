/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_traits::canvas::byte_swap;
use canvas_traits::webgl::*;
use euclid::Size2D;
use gleam::gl;
use offscreen_gl_context::{GLContext, GLContextAttributes, GLLimits, NativeGLContextMethods};
use std::collections::HashMap;
use std::mem;
use std::sync::Arc;
use std::thread;
use super::gl_context::{GLContextFactory, GLContextWrapper};
use webrender;
use webrender_api;

/// WebGL Threading API entry point that lives in the constellation.
/// It allows to get a WebGLThread handle for each script pipeline.
pub use ::webgl_mode::WebGLThreads;

/// A WebGLThread manages the life cycle and message multiplexing of
/// a set of WebGLContexts living in the same thread.
pub struct WebGLThread<VR: WebVRRenderHandler + 'static, OB: WebGLThreadObserver> {
    /// Factory used to create a new GLContext shared with the WR/Main thread.
    gl_factory: Arc<GLContextFactory>,
    /// Channel used to generate/update or delete `webrender_api::ImageKey`s.
    webrender_api: webrender_api::RenderApi,
    /// Map of live WebGLContexts.
    contexts: HashMap<WebGLContextId, GLContextWrapper>,
    /// Cached information for WebGLContexts.
    cached_context_info: HashMap<WebGLContextId, WebGLContextInfo>,
    /// Current bound context.
    current_bound_webgl_context_id: Option<WebGLContextId>,
    /// Id generator for new WebGLContexts.
    next_webgl_id: usize,
    /// Handler user to send WebVR commands.
    webvr_compositor: Option<VR>,
    /// Generic observer that listens WebGLContext creation, resize or removal events.
    observer: OB,
}

impl<VR: WebVRRenderHandler + 'static, OB: WebGLThreadObserver> WebGLThread<VR, OB> {
    pub fn new(gl_factory: Arc<GLContextFactory>,
               webrender_api_sender: webrender_api::RenderApiSender,
               webvr_compositor: Option<VR>,
               observer: OB) -> Self {
        WebGLThread {
            gl_factory: gl_factory,
            webrender_api: webrender_api_sender.create_api(),
            contexts: HashMap::new(),
            cached_context_info: HashMap::new(),
            current_bound_webgl_context_id: None,
            next_webgl_id: 0,
            webvr_compositor: webvr_compositor,
            observer: observer,
        }
    }

    /// Creates a new `WebGLThread` and returns a Sender to
    /// communicate with it.
    pub fn start(gl_factory: Arc<GLContextFactory>,
                 webrender_api_sender: webrender_api::RenderApiSender,
                 webvr_compositor: Option<VR>,
                 observer: OB)
                 -> WebGLSender<WebGLMsg> {
        let (sender, receiver) = webgl_channel::<WebGLMsg>().unwrap();
        let result = sender.clone();
        thread::Builder::new().name("WebGLThread".to_owned()).spawn(move || {
            let mut renderer = WebGLThread::new(gl_factory,
                                                webrender_api_sender,
                                                webvr_compositor,
                                                observer);
            let webgl_chan = WebGLChan(sender);
            loop {
                let msg = receiver.recv().unwrap();
                let exit = renderer.handle_msg(msg, &webgl_chan);
                if exit {
                    return;
                }
            }
        }).expect("Thread spawning failed");

        result
    }

    /// Handles a generic WebGLMsg message
    #[inline]
    pub fn handle_msg(&mut self, msg: WebGLMsg, webgl_chan: &WebGLChan) -> bool {
        match msg {
            WebGLMsg::CreateContext(size, attributes, result_sender) => {
                let result = self.create_webgl_context(size, attributes);
                result_sender.send(result.map(|(id, limits, share_mode)|
                    WebGLCreateContextResult {
                        sender: WebGLMsgSender::new(id, webgl_chan.clone()),
                        limits: limits,
                        share_mode: share_mode,
                    }
                )).unwrap();
            },
            WebGLMsg::ResizeContext(ctx_id, size, sender) => {
                self.resize_webgl_context(ctx_id, size, sender);
            },
            WebGLMsg::RemoveContext(ctx_id) => {
                self.remove_webgl_context(ctx_id);
            },
            WebGLMsg::WebGLCommand(ctx_id, command) => {
                self.handle_webgl_command(ctx_id, command);
            },
            WebGLMsg::WebVRCommand(ctx_id, command) => {
                self.handle_webvr_command(ctx_id, command);
            },
            WebGLMsg::Lock(ctx_id, sender) => {
                self.handle_lock(ctx_id, sender);
            },
            WebGLMsg::Unlock(ctx_id) => {
                self.handle_unlock(ctx_id);
            },
            WebGLMsg::UpdateWebRenderImage(ctx_id, sender) => {
                self.handle_update_wr_image(ctx_id, sender);
            },
            WebGLMsg::Exit => {
                return true;
            }
        }

        false
    }

    /// Handles a WebGLCommand for a specific WebGLContext
    fn handle_webgl_command(&mut self, context_id: WebGLContextId, command: WebGLCommand) {
        let ctx = &self.contexts[&context_id];
        if Some(context_id) != self.current_bound_webgl_context_id {
            ctx.make_current();
            self.current_bound_webgl_context_id = Some(context_id);
        }
        ctx.apply_command(command);
    }

    /// Handles a WebVRCommand for a specific WebGLContext
    fn handle_webvr_command(&mut self, context_id: WebGLContextId, command: WebVRCommand) {
        if Some(context_id) != self.current_bound_webgl_context_id {
            self.contexts[&context_id].make_current();
            self.current_bound_webgl_context_id = Some(context_id);
        }

        let texture = match command {
            WebVRCommand::SubmitFrame(..) => {
                self.cached_context_info.get(&context_id)
            },
            _ => None
        };
        self.webvr_compositor.as_mut().unwrap().handle(command, texture.map(|t| (t.texture_id, t.size)));
    }

    /// Handles a lock external callback received from webrender::ExternalImageHandler
    fn handle_lock(&mut self, context_id: WebGLContextId, sender: WebGLSender<(u32, Size2D<i32>)>) {
        let ctx = &self.contexts[&context_id];
        if Some(context_id) != self.current_bound_webgl_context_id {
            ctx.make_current();
            self.current_bound_webgl_context_id = Some(context_id);
        }
        let info = self.cached_context_info.get_mut(&context_id).unwrap();
        // Use a OpenGL Fence to perform the lock.
        info.gl_sync = Some(ctx.gl().fence_sync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0));

        sender.send((info.texture_id, info.size)).unwrap();
    }

    /// Handles a unlock external callback received from webrender::ExternalImageHandler
    fn handle_unlock(&mut self, context_id: WebGLContextId) {
        let ctx = &self.contexts[&context_id];
        if Some(context_id) != self.current_bound_webgl_context_id {
            ctx.make_current();
            self.current_bound_webgl_context_id = Some(context_id);
        }
        let info = self.cached_context_info.get_mut(&context_id).unwrap();
        if let Some(gl_sync) = info.gl_sync.take() {
            // glFlush must be called before glWaitSync.
            ctx.gl().flush();
            // Waint until the GLSync object is signaled.
            ctx.gl().wait_sync(gl_sync, 0, gl::TIMEOUT_IGNORED);
            // Release the GLSync object.
            ctx.gl().delete_sync(gl_sync);
        }
    }

    /// Creates a new WebGLContext
    fn create_webgl_context(&mut self,
                            size: Size2D<i32>,
                            attributes: GLContextAttributes)
                            -> Result<(WebGLContextId, GLLimits, WebGLContextShareMode), String> {
        // First try to create a shared context for the best performance.
        // Fallback to readback mode if the shared context creation fails.
        let result = self.gl_factory.new_shared_context(size, attributes)
                                    .map(|r| (r, WebGLContextShareMode::SharedTexture))
                                    .or_else(|_| {
                                        let ctx = self.gl_factory.new_context(size, attributes);
                                        ctx.map(|r| (r, WebGLContextShareMode::Readback))
                                    });

        // Creating a new GLContext may make the current bound context_id dirty.
        // Clear it to ensure that  make_current() is called in subsequent commands.
        self.current_bound_webgl_context_id = None;

        match result {
            Ok((ctx, share_mode)) => {
                let id = WebGLContextId(self.next_webgl_id);
                let (real_size, texture_id, limits) = ctx.get_info();
                self.next_webgl_id += 1;
                self.contexts.insert(id, ctx);
                self.cached_context_info.insert(id, WebGLContextInfo {
                    texture_id: texture_id,
                    size: real_size,
                    alpha: attributes.alpha,
                    image_key: None,
                    share_mode: share_mode,
                    gl_sync: None,
                    old_image_key: None,
                    very_old_image_key: None,
                });

                self.observer.on_context_create(id, texture_id, real_size);

                Ok((id, limits, share_mode))
            },
            Err(msg) => {
                Err(msg.to_owned())
            }
        }
    }

    /// Resizes a WebGLContext
    fn resize_webgl_context(&mut self,
                            context_id: WebGLContextId,
                            size: Size2D<i32>,
                            sender: WebGLSender<Result<(), String>>) {
        let ctx = self.contexts.get_mut(&context_id).unwrap();
        if Some(context_id) != self.current_bound_webgl_context_id {
            ctx.make_current();
            self.current_bound_webgl_context_id = Some(context_id);
        }
        match ctx.resize(size) {
            Ok(_) => {
                let (real_size, texture_id, _) = ctx.get_info();
                self.observer.on_context_resize(context_id, texture_id, real_size);

                let info = self.cached_context_info.get_mut(&context_id).unwrap();
                // Update webgl texture size. Texture id may change too.
                info.texture_id = texture_id;
                info.size = real_size;
                // WR doesn't support resizing and requires to create a new `ImageKey`.
                // Mark the current image_key to be deleted later in the next epoch.
                if let Some(image_key) = info.image_key.take() {
                    // If this executes, then we are in a new epoch since we last recreated the canvas,
                    // so `old_image_key` must be `None`.
                    debug_assert!(info.old_image_key.is_none());
                    info.old_image_key = Some(image_key);
                }

                sender.send(Ok(())).unwrap();
            },
            Err(msg) => {
                sender.send(Err(msg.into())).unwrap();
            }
        }
    }

    /// Removes a WebGLContext and releases attached resources.
    fn remove_webgl_context(&mut self, context_id: WebGLContextId) {
        // Release webrender image keys.
        if let Some(info) = self.cached_context_info.remove(&context_id) {
            if let Some(image_key) = info.image_key {
                self.webrender_api.delete_image(image_key);
            }
            if let Some(image_key) = info.old_image_key {
                self.webrender_api.delete_image(image_key);
            }
            if let Some(image_key) = info.very_old_image_key {
                self.webrender_api.delete_image(image_key);
            }
        }

        // Release GL context.
        if self.contexts.remove(&context_id).is_some() {
            self.observer.on_context_delete(context_id);
        }

        // Removing a GLContext may make the current bound context_id dirty.
        self.current_bound_webgl_context_id = None;
    }

    /// Handles the creation/update of webrender_api::ImageKeys fpr a specific WebGLContext.
    /// This method is invoked from a UpdateWebRenderImage message sent by the layout thread.
    /// If SharedTexture is used the UpdateWebRenderImage message is sent only after a WebGLContext creation or resize.
    /// If Readback is used UpdateWebRenderImage message is sent always on each layout iteration in order to
    /// submit the updated raw pixels.
    fn handle_update_wr_image(&mut self, context_id: WebGLContextId, sender: WebGLSender<webrender_api::ImageKey>) {
        let info = self.cached_context_info.get_mut(&context_id).unwrap();
        let webrender_api = &self.webrender_api;

        let image_key = match info.share_mode {
            WebGLContextShareMode::SharedTexture => {
                let size = info.size;
                let alpha = info.alpha;
                // Reuse existing ImageKey or generate a new one.
                // When using a shared texture ImageKeys are only generated after a WebGLContext creation or resize.
                *info.image_key.get_or_insert_with(|| {
                    Self::create_wr_external_image(webrender_api, size, alpha, context_id)
                })
            },
            WebGLContextShareMode::Readback => {
                let pixels = Self::raw_pixels(&self.contexts[&context_id], info.size);
                match info.image_key.clone() {
                    Some(image_key) => {
                        // ImageKey was already created, but WR Images must
                        // be updated every frame in readback mode to send the new raw pixels.
                        Self::update_wr_readback_image(webrender_api,
                                                       info.size,
                                                       info.alpha,
                                                       image_key,
                                                       pixels);

                        image_key
                    },
                    None => {
                        // Generate a new ImageKey for Readback mode.
                        let image_key = Self::create_wr_readback_image(webrender_api,
                                                                       info.size,
                                                                       info.alpha,
                                                                       pixels);
                        info.image_key = Some(image_key);
                        image_key
                    }
                }
            }
        };

        // Delete old image
        if let Some(image_key) = mem::replace(&mut info.very_old_image_key, info.old_image_key.take()) {
            webrender_api.delete_image(image_key);
        }

        // Send the ImageKey to the Layout thread.
        sender.send(image_key).unwrap();
    }

    /// Creates a `webrender_api::ImageKey` that uses shared textures.
    fn create_wr_external_image(webrender_api: &webrender_api::RenderApi,
                                size: Size2D<i32>,
                                alpha: bool,
                                context_id: WebGLContextId) -> webrender_api::ImageKey {
        let descriptor = Self::image_descriptor(size, alpha);

        let data = webrender_api::ExternalImageData {
            id: webrender_api::ExternalImageId(context_id.0 as u64),
            channel_index: 0,
            image_type: webrender_api::ExternalImageType::Texture2DHandle,
        };
        let data = webrender_api::ImageData::External(data);

        let image_key = webrender_api.generate_image_key();
        webrender_api.add_image(image_key,
                                descriptor,
                                data,
                                None);
        image_key
    }

    /// Creates a `webrender_api::ImageKey` that uses raw pixels.
    fn create_wr_readback_image(webrender_api: &webrender_api::RenderApi,
                                size: Size2D<i32>,
                                alpha: bool,
                                data: Vec<u8>) -> webrender_api::ImageKey {
        let descriptor = Self::image_descriptor(size, alpha);
        let data = webrender_api::ImageData::new(data);

        let image_key = webrender_api.generate_image_key();
        webrender_api.add_image(image_key,
                                descriptor,
                                data,
                                None);
        image_key
    }

    /// Creates a `webrender_api::ImageKey` that uses raw pixels.
    fn update_wr_readback_image(webrender_api: &webrender_api::RenderApi,
                                size: Size2D<i32>,
                                alpha: bool,
                                image_key: webrender_api::ImageKey,
                                data: Vec<u8>) {
        let descriptor = Self::image_descriptor(size, alpha);
        let data = webrender_api::ImageData::new(data);

        webrender_api.update_image(image_key,
                                   descriptor,
                                   data,
                                   None);
    }

    /// Helper function to create a `webrender_api::ImageDescriptor`.
    fn image_descriptor(size: Size2D<i32>, alpha: bool) -> webrender_api::ImageDescriptor {
        webrender_api::ImageDescriptor {
            width: size.width as u32,
            height: size.height as u32,
            stride: None,
            format: if alpha { webrender_api::ImageFormat::RGB8 } else { webrender_api::ImageFormat::BGRA8 },
            offset: 0,
            is_opaque: !alpha,
        }
    }

    /// Helper function to fetch the raw pixels used in readback mode.
    fn raw_pixels(context: &GLContextWrapper, size: Size2D<i32>) -> Vec<u8> {
        let width = size.width as usize;
        let height = size.height as usize;

        let mut pixels = context.gl().read_pixels(0, 0,
                                                  size.width as gl::GLsizei,
                                                  size.height as gl::GLsizei,
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
        byte_swap(&mut pixels);
        pixels
    }
}

impl<VR: WebVRRenderHandler + 'static, OB: WebGLThreadObserver> Drop for WebGLThread<VR, OB> {
    fn drop(&mut self) {
        // Call remove_context functions in order to correctly delete WebRender image keys.
        let context_ids: Vec<WebGLContextId> = self.contexts.keys().map(|id| *id).collect();
        for id in context_ids {
            self.remove_webgl_context(id);
        }
    }
}

/// Helper struct to store cached WebGLContext information.
struct WebGLContextInfo {
    /// Render to texture identifier used by the WebGLContext.
    texture_id: u32,
    /// Size of the WebGLContext.
    size: Size2D<i32>,
    /// True if the WebGLContext uses an alpha channel.
    alpha: bool,
    /// Currently used WebRender image key.
    image_key: Option<webrender_api::ImageKey>,
    /// The sharing mode used to send the image to WebRender.
    share_mode: WebGLContextShareMode,
    /// GLSync Object used for a correct synchronization with Webrender external image callbacks.
    gl_sync: Option<gl::GLsync>,
    /// An old WebRender image key that can be deleted when the next epoch ends.
    old_image_key: Option<webrender_api::ImageKey>,
    /// An old WebRender image key that can be deleted when the current epoch ends.
    very_old_image_key: Option<webrender_api::ImageKey>,
}

/// Trait used to observe events in a WebGL Thread.
/// Used in webrender::ExternalImageHandler when multiple WebGL threads are used.
pub trait WebGLThreadObserver: Send + 'static {
    fn on_context_create(&mut self, ctx_id: WebGLContextId, texture_id: u32, size: Size2D<i32>);
    fn on_context_resize(&mut self, ctx_id: WebGLContextId, texture_id: u32, size: Size2D<i32>);
    fn on_context_delete(&mut self, ctx_id: WebGLContextId);
}

/// Trait used by the generic WebGLExternalImageHandler implementation
pub trait WebGLExternalImageApi {
    fn lock(&mut self, ctx_id: WebGLContextId) -> (u32, Size2D<i32>);
    fn unlock(&mut self, ctx_id: WebGLContextId);
}

/// WebRender External Image Handler implementation
pub struct WebGLExternalImageHandler<T: WebGLExternalImageApi> {
    handler: T,
}

impl<T: WebGLExternalImageApi> WebGLExternalImageHandler<T> {
    pub fn new(handler: T) -> Self {
        Self {
            handler: handler
        }
    }
}

impl<T: WebGLExternalImageApi> webrender::ExternalImageHandler for WebGLExternalImageHandler<T> {
    /// Lock the external image. Then, WR could start to read the image content.
    /// The WR client should not change the image content until the unlock() call.
    fn lock(&mut self,
            key: webrender_api::ExternalImageId,
            _channel_index: u8) -> webrender::ExternalImage {
        let ctx_id = WebGLContextId(key.0 as _);
        let (texture_id, size) = self.handler.lock(ctx_id);

        webrender::ExternalImage {
            u0: 0.0,
            u1: size.width as f32,
            v1: 0.0,
            v0: size.height as f32,
            source: webrender::ExternalImageSource::NativeTexture(texture_id),
        }

    }
    /// Unlock the external image. The WR should not read the image content
    /// after this call.
    fn unlock(&mut self,
              key: webrender_api::ExternalImageId,
              _channel_index: u8) {
        let ctx_id = WebGLContextId(key.0 as _);
        self.handler.unlock(ctx_id);
    }
}
