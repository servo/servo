/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_traits::canvas::byte_swap;
use canvas_traits::webgl::*;
use euclid::Size2D;
use fnv::FnvHashMap;
use gleam::gl;
use offscreen_gl_context::{GLContext, GLContextAttributes, GLLimits, NativeGLContextMethods};
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
    gl_factory: GLContextFactory,
    /// Channel used to generate/update or delete `webrender_api::ImageKey`s.
    webrender_api: webrender_api::RenderApi,
    /// Map of live WebGLContexts.
    contexts: FnvHashMap<WebGLContextId, GLContextWrapper>,
    /// Cached information for WebGLContexts.
    cached_context_info: FnvHashMap<WebGLContextId, WebGLContextInfo>,
    /// Current bound context.
    bound_context_id: Option<WebGLContextId>,
    /// Id generator for new WebGLContexts.
    next_webgl_id: usize,
    /// Handler user to send WebVR commands.
    webvr_compositor: Option<VR>,
    /// Generic observer that listens WebGLContext creation, resize or removal events.
    observer: OB,
    /// Texture ids and sizes used in DOM to texture outputs.
    dom_outputs: FnvHashMap<webrender_api::PipelineId, DOMToTextureData>,
}

impl<VR: WebVRRenderHandler + 'static, OB: WebGLThreadObserver> WebGLThread<VR, OB> {
    pub fn new(gl_factory: GLContextFactory,
               webrender_api_sender: webrender_api::RenderApiSender,
               webvr_compositor: Option<VR>,
               observer: OB) -> Self {
        WebGLThread {
            gl_factory,
            webrender_api: webrender_api_sender.create_api(),
            contexts: Default::default(),
            cached_context_info: Default::default(),
            bound_context_id: None,
            next_webgl_id: 0,
            webvr_compositor,
            observer: observer,
            dom_outputs: Default::default(),
        }
    }

    /// Creates a new `WebGLThread` and returns a Sender to
    /// communicate with it.
    pub fn start(gl_factory: GLContextFactory,
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
    fn handle_msg(&mut self, msg: WebGLMsg, webgl_chan: &WebGLChan) -> bool {
        match msg {
            WebGLMsg::CreateContext(version, size, attributes, result_sender) => {
                let result = self.create_webgl_context(version, size, attributes);
                result_sender.send(result.map(|(id, limits, share_mode)| {
                    let ctx = Self::make_current_if_needed(id, &self.contexts, &mut self.bound_context_id)
                                    .expect("WebGLContext not found");
                    let glsl_version = Self::get_glsl_version(ctx);

                    WebGLCreateContextResult {
                        sender: WebGLMsgSender::new(id, webgl_chan.clone()),
                        limits,
                        share_mode,
                        glsl_version,
                    }
                })).unwrap();
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
            WebGLMsg::DOMToTextureCommand(command) => {
                self.handle_dom_to_texture(command);
            },
            WebGLMsg::Exit => {
                return true;
            }
        }

        false
    }

    /// Handles a WebGLCommand for a specific WebGLContext
    fn handle_webgl_command(&mut self, context_id: WebGLContextId, command: WebGLCommand) {
        if let Some(ctx) = Self::make_current_if_needed(context_id, &self.contexts, &mut self.bound_context_id) {
            ctx.apply_command(command);
        }
    }

    /// Handles a WebVRCommand for a specific WebGLContext
    fn handle_webvr_command(&mut self, context_id: WebGLContextId, command: WebVRCommand) {
        Self::make_current_if_needed(context_id, &self.contexts, &mut self.bound_context_id);
        let texture = match command {
            WebVRCommand::SubmitFrame(..) => {
                self.cached_context_info.get(&context_id)
            },
            _ => None
        };
        self.webvr_compositor.as_mut().unwrap().handle(command, texture.map(|t| (t.texture_id, t.size)));
    }

    /// Handles a lock external callback received from webrender::ExternalImageHandler
    fn handle_lock(&mut self, context_id: WebGLContextId, sender: WebGLSender<(u32, Size2D<i32>, usize)>) {
        let ctx = Self::make_current_if_needed(context_id, &self.contexts, &mut self.bound_context_id)
                        .expect("WebGLContext not found in a WebGLMsg::Lock message");
        let info = self.cached_context_info.get_mut(&context_id).unwrap();
        // Insert a OpenGL Fence sync object that sends a signal when all the WebGL commands are finished.
        // The related gl().wait_sync call is performed in the WR thread. See WebGLExternalImageApi for mor details.
        let gl_sync = ctx.gl().fence_sync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0);
        info.gl_sync = Some(gl_sync);
        // It is important that the fence sync is properly flushed into the GPU's command queue.
        // Without proper flushing, the sync object may never be signaled.
        ctx.gl().flush();

        sender.send((info.texture_id, info.size, gl_sync as usize)).unwrap();
    }

    /// Handles an unlock external callback received from webrender::ExternalImageHandler
    fn handle_unlock(&mut self, context_id: WebGLContextId) {
        let ctx = Self::make_current_if_needed(context_id, &self.contexts, &mut self.bound_context_id)
                        .expect("WebGLContext not found in a WebGLMsg::Unlock message");
        let info = self.cached_context_info.get_mut(&context_id).unwrap();
        if let Some(gl_sync) = info.gl_sync.take() {
            // Release the GLSync object.
            ctx.gl().delete_sync(gl_sync);
        }
    }

    /// Creates a new WebGLContext
    fn create_webgl_context(&mut self,
                            version: WebGLVersion,
                            size: Size2D<i32>,
                            attributes: GLContextAttributes)
                            -> Result<(WebGLContextId, GLLimits, WebGLContextShareMode), String> {
        // First try to create a shared context for the best performance.
        // Fallback to readback mode if the shared context creation fails.
        let result = self.gl_factory.new_shared_context(version, size, attributes)
                                    .map(|r| (r, WebGLContextShareMode::SharedTexture))
                                    .or_else(|_| {
                                        let ctx = self.gl_factory.new_context(version, size, attributes);
                                        ctx.map(|r| (r, WebGLContextShareMode::Readback))
                                    });

        // Creating a new GLContext may make the current bound context_id dirty.
        // Clear it to ensure that  make_current() is called in subsequent commands.
        self.bound_context_id = None;

        match result {
            Ok((ctx, share_mode)) => {
                let id = WebGLContextId(self.next_webgl_id);
                let (size, texture_id, limits) = ctx.get_info();
                self.next_webgl_id += 1;
                self.contexts.insert(id, ctx);
                self.cached_context_info.insert(id, WebGLContextInfo {
                    texture_id,
                    size,
                    alpha: attributes.alpha,
                    image_key: None,
                    share_mode,
                    gl_sync: None,
                });

                self.observer.on_context_create(id, texture_id, size);

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
        let ctx = Self::make_current_if_needed_mut(context_id, &mut self.contexts, &mut self.bound_context_id);
        match ctx.resize(size) {
            Ok(_) => {
                let (real_size, texture_id, _) = ctx.get_info();
                self.observer.on_context_resize(context_id, texture_id, real_size);

                let info = self.cached_context_info.get_mut(&context_id).unwrap();
                // Update webgl texture size. Texture id may change too.
                info.texture_id = texture_id;
                info.size = real_size;
                // Update WR image if needed. Resize image updates are only required for SharedTexture mode.
                // Readback mode already updates the image every frame to send the raw pixels.
                // See `handle_update_wr_image`.
                match (info.image_key, info.share_mode) {
                    (Some(image_key), WebGLContextShareMode::SharedTexture) => {
                        Self::update_wr_external_image(&self.webrender_api,
                                                       info.size,
                                                       info.alpha,
                                                       context_id,
                                                       image_key);
                    },
                    _ => {}
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
            let mut updates = webrender_api::ResourceUpdates::new();

            if let Some(image_key) = info.image_key {
                updates.delete_image(image_key);
            }

            self.webrender_api.update_resources(updates)
        }

        // Release GL context.
        if self.contexts.remove(&context_id).is_some() {
            self.observer.on_context_delete(context_id);
        }

        // Removing a GLContext may make the current bound context_id dirty.
        self.bound_context_id = None;
    }

    /// Handles the creation/update of webrender_api::ImageKeys for a specific WebGLContext.
    /// This method is invoked from a UpdateWebRenderImage message sent by the layout thread.
    /// If SharedTexture is used the UpdateWebRenderImage message is sent only after a WebGLContext creation.
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
                // When using a shared texture ImageKeys are only generated after a WebGLContext creation.
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

        // Send the ImageKey to the Layout thread.
        sender.send(image_key).unwrap();
    }

    fn handle_dom_to_texture(&mut self, command: DOMToTextureCommand) {
        match command {
            DOMToTextureCommand::Attach(context_id, texture_id, document_id, pipeline_id, size) => {
                let ctx = Self::make_current_if_needed(context_id, &self.contexts, &mut self.bound_context_id)
                                .expect("WebGLContext not found in a WebGL DOMToTextureCommand::Attach command");
                // Initialize the texture that WR will use for frame outputs.
                ctx.gl().tex_image_2d(gl::TEXTURE_2D,
                                      0,
                                      gl::RGBA as gl::GLint,
                                      size.width,
                                      size.height,
                                      0,
                                      gl::RGBA,
                                      gl::UNSIGNED_BYTE,
                                      None);
                self.dom_outputs.insert(pipeline_id, DOMToTextureData {
                    context_id, texture_id, document_id, size
                });
                let mut txn = webrender_api::Transaction::new();
                txn.enable_frame_output(pipeline_id, true);
                self.webrender_api.send_transaction(document_id, txn);
            },
            DOMToTextureCommand::Lock(pipeline_id, gl_sync, sender) => {
                let contexts = &self.contexts;
                let bound_context_id = &mut self.bound_context_id;
                let result = self.dom_outputs.get(&pipeline_id).and_then(|data| {
                    let ctx = Self::make_current_if_needed(data.context_id, contexts, bound_context_id);
                    ctx.and_then(|ctx| {
                        // The next glWaitSync call is used to synchronize the two flows of
                        // OpenGL commands (WR and WebGL) in order to avoid using semi-ready WR textures.
                        // glWaitSync doesn't block WebGL CPU thread.
                        ctx.gl().wait_sync(gl_sync as gl::GLsync, 0, gl::TIMEOUT_IGNORED);
                        Some((data.texture_id.get(), data.size))
                    })
                });

                // Send the texture id and size to WR.
                sender.send(result).unwrap();
            },
            DOMToTextureCommand::Detach(texture_id) => {
                if let Some((pipeline_id, document_id)) = self.dom_outputs.iter()
                                                                          .find(|&(_, v)| v.texture_id == texture_id)
                                                                          .map(|(k, v)| (*k, v.document_id)) {
                    let mut txn = webrender_api::Transaction::new();
                    txn.enable_frame_output(pipeline_id, false);
                    self.webrender_api.send_transaction(document_id, txn);
                    self.dom_outputs.remove(&pipeline_id);
                }
            },
        }
    }

    /// Gets a reference to a GLContextWrapper for a given WebGLContextId and makes it current if required.
    fn make_current_if_needed<'a>(context_id: WebGLContextId,
                                  contexts: &'a FnvHashMap<WebGLContextId, GLContextWrapper>,
                                  bound_id: &mut Option<WebGLContextId>) -> Option<&'a GLContextWrapper> {
        contexts.get(&context_id).and_then(|ctx| {
            if Some(context_id) != *bound_id {
                ctx.make_current();
                *bound_id = Some(context_id);
            }

            Some(ctx)
        })
    }

    /// Gets a mutable reference to a GLContextWrapper for a WebGLContextId and makes it current if required.
    fn make_current_if_needed_mut<'a>(context_id: WebGLContextId,
                                      contexts: &'a mut FnvHashMap<WebGLContextId, GLContextWrapper>,
                                      bound_id: &mut Option<WebGLContextId>) -> &'a mut GLContextWrapper {
        let ctx = contexts.get_mut(&context_id).expect("WebGLContext not found!");
        if Some(context_id) != *bound_id {
            ctx.make_current();
            *bound_id = Some(context_id);
        }
        ctx
    }

    /// Creates a `webrender_api::ImageKey` that uses shared textures.
    fn create_wr_external_image(webrender_api: &webrender_api::RenderApi,
                                size: Size2D<i32>,
                                alpha: bool,
                                context_id: WebGLContextId) -> webrender_api::ImageKey {
        let descriptor = Self::image_descriptor(size, alpha);
        let data = Self::external_image_data(context_id);

        let image_key = webrender_api.generate_image_key();
        let mut updates = webrender_api::ResourceUpdates::new();
        updates.add_image(image_key,
                          descriptor,
                          data,
                          None);
        webrender_api.update_resources(updates);

        image_key
    }

    /// Updates a `webrender_api::ImageKey` that uses shared textures.
    fn update_wr_external_image(webrender_api: &webrender_api::RenderApi,
                                size: Size2D<i32>,
                                alpha: bool,
                                context_id: WebGLContextId,
                                image_key: webrender_api::ImageKey) {
        let descriptor = Self::image_descriptor(size, alpha);
        let data = Self::external_image_data(context_id);

        let mut updates = webrender_api::ResourceUpdates::new();
        updates.update_image(image_key,
                             descriptor,
                             data,
                             None);
        webrender_api.update_resources(updates);
    }

    /// Creates a `webrender_api::ImageKey` that uses raw pixels.
    fn create_wr_readback_image(webrender_api: &webrender_api::RenderApi,
                                size: Size2D<i32>,
                                alpha: bool,
                                data: Vec<u8>) -> webrender_api::ImageKey {
        let descriptor = Self::image_descriptor(size, alpha);
        let data = webrender_api::ImageData::new(data);

        let image_key = webrender_api.generate_image_key();
        let mut updates = webrender_api::ResourceUpdates::new();
        updates.add_image(image_key,
                          descriptor,
                          data,
                          None);
        webrender_api.update_resources(updates);

        image_key
    }

    /// Updates a `webrender_api::ImageKey` that uses raw pixels.
    fn update_wr_readback_image(webrender_api: &webrender_api::RenderApi,
                                size: Size2D<i32>,
                                alpha: bool,
                                image_key: webrender_api::ImageKey,
                                data: Vec<u8>) {
        let descriptor = Self::image_descriptor(size, alpha);
        let data = webrender_api::ImageData::new(data);

        let mut updates = webrender_api::ResourceUpdates::new();
        updates.update_image(image_key,
                             descriptor,
                             data,
                             None);
        webrender_api.update_resources(updates);
    }

    /// Helper function to create a `webrender_api::ImageDescriptor`.
    fn image_descriptor(size: Size2D<i32>, alpha: bool) -> webrender_api::ImageDescriptor {
        webrender_api::ImageDescriptor {
            width: size.width as u32,
            height: size.height as u32,
            stride: None,
            format: webrender_api::ImageFormat::BGRA8,
            offset: 0,
            is_opaque: !alpha,
        }
    }

    /// Helper function to create a `webrender_api::ImageData::External` instance.
    fn external_image_data(context_id: WebGLContextId) -> webrender_api::ImageData {
        let data = webrender_api::ExternalImageData {
            id: webrender_api::ExternalImageId(context_id.0 as u64),
            channel_index: 0,
            image_type: webrender_api::ExternalImageType::Texture2DHandle,
        };
        webrender_api::ImageData::External(data)
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

    /// Gets the GLSL Version supported by a GLContext.
    fn get_glsl_version(context: &GLContextWrapper) -> WebGLSLVersion {
        let version = context.gl().get_string(gl::SHADING_LANGUAGE_VERSION);
        // Fomat used by SHADING_LANGUAGE_VERSION query : major.minor[.release] [vendor info]
        let mut values = version.split(&['.', ' '][..]);
        let major = values.next().and_then(|v| v.parse::<u32>().ok()).unwrap_or(1);
        let minor = values.next().and_then(|v| v.parse::<u32>().ok()).unwrap_or(20);

        WebGLSLVersion {
            major,
            minor,
        }
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
}

/// Trait used to observe events in a WebGL Thread.
/// Used in webrender::ExternalImageHandler when multiple WebGL threads are used.
pub trait WebGLThreadObserver: Send + 'static {
    fn on_context_create(&mut self, ctx_id: WebGLContextId, texture_id: u32, size: Size2D<i32>);
    fn on_context_resize(&mut self, ctx_id: WebGLContextId, texture_id: u32, size: Size2D<i32>);
    fn on_context_delete(&mut self, ctx_id: WebGLContextId);
}

/// This trait is used as a bridge between the `WebGLThreads` implementation and
/// the WR ExternalImageHandler API implemented in the `WebGLExternalImageHandler` struct.
/// `WebGLExternalImageHandler<T>` takes care of type conversions between WR and WebGL info (e.g keys, uvs).
/// It uses this trait to notify lock/unlock messages and get the required info that WR needs.
/// `WebGLThreads` receives lock/unlock message notifications and takes care of sending
/// the unlock/lock messages to the appropiate `WebGLThread`.
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

/// Data about the linked DOM<->WebGLTexture elements.
struct DOMToTextureData {
    context_id: WebGLContextId,
    texture_id: WebGLTextureId,
    document_id: webrender_api::DocumentId,
    size: Size2D<i32>,
}

/// WebGL Commands Implementation
pub struct WebGLImpl;

impl WebGLImpl {
    pub fn apply<Native: NativeGLContextMethods>(ctx: &GLContext<Native>, command: WebGLCommand) {
        match command {
            WebGLCommand::GetContextAttributes(sender) =>
                sender.send(*ctx.borrow_attributes()).unwrap(),
            WebGLCommand::ActiveTexture(target) =>
                ctx.gl().active_texture(target),
            WebGLCommand::AttachShader(program_id, shader_id) =>
                ctx.gl().attach_shader(program_id.get(), shader_id.get()),
            WebGLCommand::DetachShader(program_id, shader_id) =>
                ctx.gl().detach_shader(program_id.get(), shader_id.get()),
            WebGLCommand::BindAttribLocation(program_id, index, name) =>
                ctx.gl().bind_attrib_location(program_id.get(), index, &name),
            WebGLCommand::BlendColor(r, g, b, a) =>
                ctx.gl().blend_color(r, g, b, a),
            WebGLCommand::BlendEquation(mode) =>
                ctx.gl().blend_equation(mode),
            WebGLCommand::BlendEquationSeparate(mode_rgb, mode_alpha) =>
                ctx.gl().blend_equation_separate(mode_rgb, mode_alpha),
            WebGLCommand::BlendFunc(src, dest) =>
                ctx.gl().blend_func(src, dest),
            WebGLCommand::BlendFuncSeparate(src_rgb, dest_rgb, src_alpha, dest_alpha) =>
                ctx.gl().blend_func_separate(src_rgb, dest_rgb, src_alpha, dest_alpha),
            WebGLCommand::BufferData(buffer_type, data, usage) =>
                gl::buffer_data(ctx.gl(), buffer_type, &data, usage),
            WebGLCommand::BufferSubData(buffer_type, offset, data) =>
                gl::buffer_sub_data(ctx.gl(), buffer_type, offset, &data),
            WebGLCommand::Clear(mask) =>
                ctx.gl().clear(mask),
            WebGLCommand::ClearColor(r, g, b, a) =>
                ctx.gl().clear_color(r, g, b, a),
            WebGLCommand::ClearDepth(depth) =>
                ctx.gl().clear_depth(depth),
            WebGLCommand::ClearStencil(stencil) =>
                ctx.gl().clear_stencil(stencil),
            WebGLCommand::ColorMask(r, g, b, a) =>
                ctx.gl().color_mask(r, g, b, a),
            WebGLCommand::CopyTexImage2D(target, level, internal_format, x, y, width, height, border) =>
                ctx.gl().copy_tex_image_2d(target, level, internal_format, x, y, width, height, border),
            WebGLCommand::CopyTexSubImage2D(target, level, xoffset, yoffset, x, y, width, height) =>
                ctx.gl().copy_tex_sub_image_2d(target, level, xoffset, yoffset, x, y, width, height),
            WebGLCommand::CullFace(mode) =>
                ctx.gl().cull_face(mode),
            WebGLCommand::DepthFunc(func) =>
                ctx.gl().depth_func(func),
            WebGLCommand::DepthMask(flag) =>
                ctx.gl().depth_mask(flag),
            WebGLCommand::DepthRange(near, far) =>
                ctx.gl().depth_range(near, far),
            WebGLCommand::Disable(cap) =>
                ctx.gl().disable(cap),
            WebGLCommand::Enable(cap) =>
                ctx.gl().enable(cap),
            WebGLCommand::FramebufferRenderbuffer(target, attachment, renderbuffertarget, rb) =>
                ctx.gl().framebuffer_renderbuffer(target, attachment, renderbuffertarget,
                                                  rb.map_or(0, WebGLRenderbufferId::get)),
            WebGLCommand::FramebufferTexture2D(target, attachment, textarget, texture, level) =>
                ctx.gl().framebuffer_texture_2d(target, attachment, textarget,
                                                texture.map_or(0, WebGLTextureId::get), level),
            WebGLCommand::FrontFace(mode) =>
                ctx.gl().front_face(mode),
            WebGLCommand::DisableVertexAttribArray(attrib_id) =>
                ctx.gl().disable_vertex_attrib_array(attrib_id),
            WebGLCommand::DrawArrays(mode, first, count) =>
                ctx.gl().draw_arrays(mode, first, count),
            WebGLCommand::DrawElements(mode, count, type_, offset) =>
                ctx.gl().draw_elements(mode, count, type_, offset as u32),
            WebGLCommand::EnableVertexAttribArray(attrib_id) =>
                ctx.gl().enable_vertex_attrib_array(attrib_id),
            WebGLCommand::Hint(name, val) =>
                ctx.gl().hint(name, val),
            WebGLCommand::IsEnabled(cap, chan) =>
                chan.send(ctx.gl().is_enabled(cap) != 0).unwrap(),
            WebGLCommand::LineWidth(width) =>
                ctx.gl().line_width(width),
            WebGLCommand::PixelStorei(name, val) =>
                ctx.gl().pixel_store_i(name, val),
            WebGLCommand::PolygonOffset(factor, units) =>
                ctx.gl().polygon_offset(factor, units),
            WebGLCommand::ReadPixels(x, y, width, height, format, pixel_type, chan) =>
                Self::read_pixels(ctx.gl(), x, y, width, height, format, pixel_type, chan),
            WebGLCommand::RenderbufferStorage(target, format, width, height) =>
                ctx.gl().renderbuffer_storage(target, format, width, height),
            WebGLCommand::SampleCoverage(value, invert) =>
                ctx.gl().sample_coverage(value, invert),
            WebGLCommand::Scissor(x, y, width, height) =>
                ctx.gl().scissor(x, y, width, height),
            WebGLCommand::StencilFunc(func, ref_, mask) =>
                ctx.gl().stencil_func(func, ref_, mask),
            WebGLCommand::StencilFuncSeparate(face, func, ref_, mask) =>
                ctx.gl().stencil_func_separate(face, func, ref_, mask),
            WebGLCommand::StencilMask(mask) =>
                ctx.gl().stencil_mask(mask),
            WebGLCommand::StencilMaskSeparate(face, mask) =>
                ctx.gl().stencil_mask_separate(face, mask),
            WebGLCommand::StencilOp(fail, zfail, zpass) =>
                ctx.gl().stencil_op(fail, zfail, zpass),
            WebGLCommand::StencilOpSeparate(face, fail, zfail, zpass) =>
                ctx.gl().stencil_op_separate(face, fail, zfail, zpass),
            WebGLCommand::GetActiveAttrib(program_id, index, chan) =>
                Self::active_attrib(ctx.gl(), program_id, index, chan),
            WebGLCommand::GetActiveUniform(program_id, index, chan) =>
                Self::active_uniform(ctx.gl(), program_id, index, chan),
            WebGLCommand::GetAttribLocation(program_id, name, chan) =>
                Self::attrib_location(ctx.gl(), program_id, name, chan),
            WebGLCommand::GetVertexAttrib(index, pname, chan) =>
                Self::vertex_attrib(ctx.gl(), index, pname, chan),
            WebGLCommand::GetVertexAttribOffset(index, pname, chan) =>
                Self::vertex_attrib_offset(ctx.gl(), index, pname, chan),
            WebGLCommand::GetBufferParameter(target, param_id, chan) =>
                Self::buffer_parameter(ctx.gl(), target, param_id, chan),
            WebGLCommand::GetParameter(param_id, chan) =>
                Self::parameter(ctx.gl(), param_id, chan),
            WebGLCommand::GetProgramParameter(program_id, param_id, chan) =>
                Self::program_parameter(ctx.gl(), program_id, param_id, chan),
            WebGLCommand::GetShaderParameter(shader_id, param_id, chan) =>
                Self::shader_parameter(ctx.gl(), shader_id, param_id, chan),
            WebGLCommand::GetShaderPrecisionFormat(shader_type, precision_type, chan) =>
                Self::shader_precision_format(ctx.gl(), shader_type, precision_type, chan),
            WebGLCommand::GetExtensions(chan) =>
                Self::get_extensions(ctx.gl(), chan),
            WebGLCommand::GetUniformLocation(program_id, name, chan) =>
                Self::uniform_location(ctx.gl(), program_id, name, chan),
            WebGLCommand::GetShaderInfoLog(shader_id, chan) =>
                Self::shader_info_log(ctx.gl(), shader_id, chan),
            WebGLCommand::GetProgramInfoLog(program_id, chan) =>
                Self::program_info_log(ctx.gl(), program_id, chan),
            WebGLCommand::CompileShader(shader_id, source) =>
                Self::compile_shader(ctx.gl(), shader_id, source),
            WebGLCommand::CreateBuffer(chan) =>
                Self::create_buffer(ctx.gl(), chan),
            WebGLCommand::CreateFramebuffer(chan) =>
                Self::create_framebuffer(ctx.gl(), chan),
            WebGLCommand::CreateRenderbuffer(chan) =>
                Self::create_renderbuffer(ctx.gl(), chan),
            WebGLCommand::CreateTexture(chan) =>
                Self::create_texture(ctx.gl(), chan),
            WebGLCommand::CreateProgram(chan) =>
                Self::create_program(ctx.gl(), chan),
            WebGLCommand::CreateShader(shader_type, chan) =>
                Self::create_shader(ctx.gl(), shader_type, chan),
            WebGLCommand::DeleteBuffer(id) =>
                ctx.gl().delete_buffers(&[id.get()]),
            WebGLCommand::DeleteFramebuffer(id) =>
                ctx.gl().delete_framebuffers(&[id.get()]),
            WebGLCommand::DeleteRenderbuffer(id) =>
                ctx.gl().delete_renderbuffers(&[id.get()]),
            WebGLCommand::DeleteTexture(id) =>
                ctx.gl().delete_textures(&[id.get()]),
            WebGLCommand::DeleteProgram(id) =>
                ctx.gl().delete_program(id.get()),
            WebGLCommand::DeleteShader(id) =>
                ctx.gl().delete_shader(id.get()),
            WebGLCommand::BindBuffer(target, id) =>
                ctx.gl().bind_buffer(target, id.map_or(0, WebGLBufferId::get)),
            WebGLCommand::BindFramebuffer(target, request) =>
                Self::bind_framebuffer(ctx.gl(), target, request, ctx),
            WebGLCommand::BindRenderbuffer(target, id) =>
                ctx.gl().bind_renderbuffer(target, id.map_or(0, WebGLRenderbufferId::get)),
            WebGLCommand::BindTexture(target, id) =>
                ctx.gl().bind_texture(target, id.map_or(0, WebGLTextureId::get)),
            WebGLCommand::LinkProgram(program_id) =>
                ctx.gl().link_program(program_id.get()),
            WebGLCommand::Uniform1f(uniform_id, v) =>
                ctx.gl().uniform_1f(uniform_id, v),
            WebGLCommand::Uniform1fv(uniform_id, v) =>
                ctx.gl().uniform_1fv(uniform_id, &v),
            WebGLCommand::Uniform1i(uniform_id, v) =>
                ctx.gl().uniform_1i(uniform_id, v),
            WebGLCommand::Uniform1iv(uniform_id, v) =>
                ctx.gl().uniform_1iv(uniform_id, &v),
            WebGLCommand::Uniform2f(uniform_id, x, y) =>
                ctx.gl().uniform_2f(uniform_id, x, y),
            WebGLCommand::Uniform2fv(uniform_id, v) =>
                ctx.gl().uniform_2fv(uniform_id, &v),
            WebGLCommand::Uniform2i(uniform_id, x, y) =>
                ctx.gl().uniform_2i(uniform_id, x, y),
            WebGLCommand::Uniform2iv(uniform_id, v) =>
                ctx.gl().uniform_2iv(uniform_id, &v),
            WebGLCommand::Uniform3f(uniform_id, x, y, z) =>
                ctx.gl().uniform_3f(uniform_id, x, y, z),
            WebGLCommand::Uniform3fv(uniform_id, v) =>
                ctx.gl().uniform_3fv(uniform_id, &v),
            WebGLCommand::Uniform3i(uniform_id, x, y, z) =>
                ctx.gl().uniform_3i(uniform_id, x, y, z),
            WebGLCommand::Uniform3iv(uniform_id, v) =>
                ctx.gl().uniform_3iv(uniform_id, &v),
            WebGLCommand::Uniform4f(uniform_id, x, y, z, w) =>
                ctx.gl().uniform_4f(uniform_id, x, y, z, w),
            WebGLCommand::Uniform4fv(uniform_id, v) =>
                ctx.gl().uniform_4fv(uniform_id, &v),
            WebGLCommand::Uniform4i(uniform_id, x, y, z, w) =>
                ctx.gl().uniform_4i(uniform_id, x, y, z, w),
            WebGLCommand::Uniform4iv(uniform_id, v) =>
                ctx.gl().uniform_4iv(uniform_id, &v),
            WebGLCommand::UniformMatrix2fv(uniform_id, transpose,  v) =>
                ctx.gl().uniform_matrix_2fv(uniform_id, transpose, &v),
            WebGLCommand::UniformMatrix3fv(uniform_id, transpose,  v) =>
                ctx.gl().uniform_matrix_3fv(uniform_id, transpose, &v),
            WebGLCommand::UniformMatrix4fv(uniform_id, transpose,  v) =>
                ctx.gl().uniform_matrix_4fv(uniform_id, transpose, &v),
            WebGLCommand::UseProgram(program_id) =>
                ctx.gl().use_program(program_id.get()),
            WebGLCommand::ValidateProgram(program_id) =>
                ctx.gl().validate_program(program_id.get()),
            WebGLCommand::VertexAttrib(attrib_id, x, y, z, w) =>
                ctx.gl().vertex_attrib_4f(attrib_id, x, y, z, w),
            WebGLCommand::VertexAttribPointer2f(attrib_id, size, normalized, stride, offset) =>
                ctx.gl().vertex_attrib_pointer_f32(attrib_id, size, normalized, stride, offset),
            WebGLCommand::VertexAttribPointer(attrib_id, size, data_type, normalized, stride, offset) =>
                ctx.gl().vertex_attrib_pointer(attrib_id, size, data_type, normalized, stride, offset),
            WebGLCommand::Viewport(x, y, width, height) =>
                ctx.gl().viewport(x, y, width, height),
            WebGLCommand::TexImage2D(target, level, internal, width, height, format, data_type, data) =>
                ctx.gl().tex_image_2d(target, level, internal, width, height,
                                      /*border*/0, format, data_type, Some(&data)),
            WebGLCommand::TexParameteri(target, name, value) =>
                ctx.gl().tex_parameter_i(target, name, value),
            WebGLCommand::TexParameterf(target, name, value) =>
                ctx.gl().tex_parameter_f(target, name, value),
            WebGLCommand::TexSubImage2D(target, level, xoffset, yoffset, x, y, width, height, data) =>
                ctx.gl().tex_sub_image_2d(target, level, xoffset, yoffset, x, y, width, height, &data),
            WebGLCommand::DrawingBufferWidth(sender) =>
                sender.send(ctx.borrow_draw_buffer().unwrap().size().width).unwrap(),
            WebGLCommand::DrawingBufferHeight(sender) =>
                sender.send(ctx.borrow_draw_buffer().unwrap().size().height).unwrap(),
            WebGLCommand::Finish(sender) =>
                Self::finish(ctx.gl(), sender),
            WebGLCommand::Flush =>
                ctx.gl().flush(),
            WebGLCommand::GenerateMipmap(target) =>
                ctx.gl().generate_mipmap(target),
            WebGLCommand::CreateVertexArray(chan) =>
                Self::create_vertex_array(ctx.gl(), chan),
            WebGLCommand::DeleteVertexArray(id) =>
                ctx.gl().delete_vertex_arrays(&[id.get()]),
            WebGLCommand::BindVertexArray(id) =>
                ctx.gl().bind_vertex_array(id.map_or(0, WebGLVertexArrayId::get)),
        }

        // TODO: update test expectations in order to enable debug assertions
        //if cfg!(debug_assertions) {
            let error = ctx.gl().get_error();
            assert!(error == gl::NO_ERROR, "Unexpected WebGL error: 0x{:x} ({})", error, error);
        //}
    }

    fn read_pixels(gl: &gl::Gl, x: i32, y: i32, width: i32, height: i32, format: u32, pixel_type: u32,
                   chan: WebGLSender<Vec<u8>>) {
      let result = gl.read_pixels(x, y, width, height, format, pixel_type);
      chan.send(result).unwrap()
    }

    fn active_attrib(gl: &gl::Gl,
                     program_id: WebGLProgramId,
                     index: u32,
                     chan: WebGLSender<WebGLResult<(i32, u32, String)>>) {
        let result = if index >= gl.get_program_iv(program_id.get(), gl::ACTIVE_ATTRIBUTES) as u32 {
            Err(WebGLError::InvalidValue)
        } else {
            Ok(gl.get_active_attrib(program_id.get(), index))
        };
        chan.send(result).unwrap();
    }

    fn active_uniform(gl: &gl::Gl,
                      program_id: WebGLProgramId,
                      index: u32,
                      chan: WebGLSender<WebGLResult<(i32, u32, String)>>) {
        let result = if index >= gl.get_program_iv(program_id.get(), gl::ACTIVE_UNIFORMS) as u32 {
            Err(WebGLError::InvalidValue)
        } else {
            Ok(gl.get_active_uniform(program_id.get(), index))
        };
        chan.send(result).unwrap();
    }

    fn attrib_location(gl: &gl::Gl,
                       program_id: WebGLProgramId,
                       name: String,
                       chan: WebGLSender<Option<i32>> ) {
        let attrib_location = gl.get_attrib_location(program_id.get(), &name);

        let attrib_location = if attrib_location == -1 {
            None
        } else {
            Some(attrib_location)
        };

        chan.send(attrib_location).unwrap();
    }

    fn parameter(gl: &gl::Gl,
                 param_id: u32,
                 chan: WebGLSender<WebGLResult<WebGLParameter>>) {
        let result = match param_id {
            gl::ACTIVE_TEXTURE |
            gl::ALPHA_BITS |
            gl::BLEND_DST_ALPHA |
            gl::BLEND_DST_RGB |
            gl::BLEND_EQUATION_ALPHA |
            gl::BLEND_EQUATION_RGB |
            gl::BLEND_SRC_ALPHA |
            gl::BLEND_SRC_RGB |
            gl::BLUE_BITS |
            gl::CULL_FACE_MODE |
            gl::DEPTH_BITS |
            gl::DEPTH_FUNC |
            gl::FRONT_FACE |
            //gl::GENERATE_MIPMAP_HINT |
            gl::GREEN_BITS |
            //gl::IMPLEMENTATION_COLOR_READ_FORMAT |
            //gl::IMPLEMENTATION_COLOR_READ_TYPE |
            gl::MAX_COMBINED_TEXTURE_IMAGE_UNITS |
            gl::MAX_CUBE_MAP_TEXTURE_SIZE |
            //gl::MAX_FRAGMENT_UNIFORM_VECTORS |
            gl::MAX_RENDERBUFFER_SIZE |
            gl::MAX_TEXTURE_IMAGE_UNITS |
            gl::MAX_TEXTURE_SIZE |
            //gl::MAX_VARYING_VECTORS |
            gl::MAX_VERTEX_ATTRIBS |
            gl::MAX_VERTEX_TEXTURE_IMAGE_UNITS |
            //gl::MAX_VERTEX_UNIFORM_VECTORS |
            gl::PACK_ALIGNMENT |
            gl::RED_BITS |
            gl::SAMPLE_BUFFERS |
            gl::SAMPLES |
            gl::STENCIL_BACK_FAIL |
            gl::STENCIL_BACK_FUNC |
            gl::STENCIL_BACK_PASS_DEPTH_FAIL |
            gl::STENCIL_BACK_PASS_DEPTH_PASS |
            gl::STENCIL_BACK_REF |
            gl::STENCIL_BACK_VALUE_MASK |
            gl::STENCIL_BACK_WRITEMASK |
            gl::STENCIL_BITS |
            gl::STENCIL_CLEAR_VALUE |
            gl::STENCIL_FAIL |
            gl::STENCIL_FUNC |
            gl::STENCIL_PASS_DEPTH_FAIL |
            gl::STENCIL_PASS_DEPTH_PASS |
            gl::STENCIL_REF |
            gl::STENCIL_VALUE_MASK |
            gl::STENCIL_WRITEMASK |
            gl::SUBPIXEL_BITS |
            gl::UNPACK_ALIGNMENT |
            gl::FRAGMENT_SHADER_DERIVATIVE_HINT =>
            //gl::UNPACK_COLORSPACE_CONVERSION_WEBGL =>
                Ok(WebGLParameter::Int(gl.get_integer_v(param_id))),

            gl::BLEND |
            gl::CULL_FACE |
            gl::DEPTH_TEST |
            gl::DEPTH_WRITEMASK |
            gl::DITHER |
            gl::POLYGON_OFFSET_FILL |
            gl::SAMPLE_COVERAGE_INVERT |
            gl::STENCIL_TEST =>
            //gl::UNPACK_FLIP_Y_WEBGL |
            //gl::UNPACK_PREMULTIPLY_ALPHA_WEBGL =>
                Ok(WebGLParameter::Bool(gl.get_boolean_v(param_id) != 0)),

            gl::DEPTH_CLEAR_VALUE |
            gl::LINE_WIDTH |
            gl::POLYGON_OFFSET_FACTOR |
            gl::POLYGON_OFFSET_UNITS |
            gl::SAMPLE_COVERAGE_VALUE =>
                Ok(WebGLParameter::Float(gl.get_float_v(param_id))),

            gl::VERSION => Ok(WebGLParameter::String("WebGL 1.0".to_owned())),
            gl::RENDERER |
            gl::VENDOR => Ok(WebGLParameter::String("Mozilla/Servo".to_owned())),
            gl::SHADING_LANGUAGE_VERSION => Ok(WebGLParameter::String("WebGL GLSL ES 1.0".to_owned())),

            // TODO(zbarsky, emilio): Implement support for the following valid parameters
            // Float32Array
            gl::ALIASED_LINE_WIDTH_RANGE |
            //gl::ALIASED_POINT_SIZE_RANGE |
            //gl::BLEND_COLOR |
            gl::COLOR_CLEAR_VALUE |
            gl::DEPTH_RANGE |

            // WebGLBuffer
            gl::ARRAY_BUFFER_BINDING |
            gl::ELEMENT_ARRAY_BUFFER_BINDING |

            // WebGLFrameBuffer
            gl::FRAMEBUFFER_BINDING |

            // WebGLRenderBuffer
            gl::RENDERBUFFER_BINDING |

            // WebGLProgram
            gl::CURRENT_PROGRAM |

            // WebGLTexture
            gl::TEXTURE_BINDING_2D |
            gl::TEXTURE_BINDING_CUBE_MAP |

            // sequence<GlBoolean>
            gl::COLOR_WRITEMASK |

            // Uint32Array
            gl::COMPRESSED_TEXTURE_FORMATS |

            // Int32Array
            gl::MAX_VIEWPORT_DIMS |
            gl::SCISSOR_BOX |
            gl::VIEWPORT => Err(WebGLError::InvalidEnum),

            // Invalid parameters
            _ => Err(WebGLError::InvalidEnum)
        };

        chan.send(result).unwrap();
    }

    fn finish(gl: &gl::Gl, chan: WebGLSender<()>) {
        gl.finish();
        chan.send(()).unwrap();
    }

    fn vertex_attrib(gl: &gl::Gl,
                     index: u32,
                     pname: u32,
                     chan: WebGLSender<WebGLResult<WebGLParameter>>) {
        let result = if index >= gl.get_integer_v(gl::MAX_VERTEX_ATTRIBS) as u32 {
            Err(WebGLError::InvalidValue)
        } else {
            match pname {
                gl::VERTEX_ATTRIB_ARRAY_ENABLED |
                gl::VERTEX_ATTRIB_ARRAY_NORMALIZED =>
                    Ok(WebGLParameter::Bool(gl.get_vertex_attrib_iv(index, pname) != 0)),
                gl::VERTEX_ATTRIB_ARRAY_SIZE |
                gl::VERTEX_ATTRIB_ARRAY_STRIDE |
                gl::VERTEX_ATTRIB_ARRAY_TYPE =>
                    Ok(WebGLParameter::Int(gl.get_vertex_attrib_iv(index, pname))),
                gl::CURRENT_VERTEX_ATTRIB =>
                    Ok(WebGLParameter::FloatArray(gl.get_vertex_attrib_fv(index, pname))),
                // gl::VERTEX_ATTRIB_ARRAY_BUFFER_BINDING should return WebGLBuffer
                _ => Err(WebGLError::InvalidEnum),
            }
        };

        chan.send(result).unwrap();
    }

    fn vertex_attrib_offset(gl: &gl::Gl,
                            index: u32,
                            pname: u32,
                            chan: WebGLSender<WebGLResult<isize>>) {
        let result = match pname {
                gl::VERTEX_ATTRIB_ARRAY_POINTER => Ok(gl.get_vertex_attrib_pointer_v(index, pname)),
                _ => Err(WebGLError::InvalidEnum),
        };

        chan.send(result).unwrap();
    }

    fn buffer_parameter(gl: &gl::Gl,
                        target: u32,
                        param_id: u32,
                        chan: WebGLSender<WebGLResult<WebGLParameter>>) {
        let result = match param_id {
            gl::BUFFER_SIZE |
            gl::BUFFER_USAGE =>
                Ok(WebGLParameter::Int(gl.get_buffer_parameter_iv(target, param_id))),
            _ => Err(WebGLError::InvalidEnum),
        };

        chan.send(result).unwrap();
    }

    fn program_parameter(gl: &gl::Gl,
                         program_id: WebGLProgramId,
                         param_id: u32,
                         chan: WebGLSender<WebGLResult<WebGLParameter>>) {
        let result = match param_id {
            gl::DELETE_STATUS |
            gl::LINK_STATUS |
            gl::VALIDATE_STATUS =>
                Ok(WebGLParameter::Bool(gl.get_program_iv(program_id.get(), param_id) != 0)),
            gl::ATTACHED_SHADERS |
            gl::ACTIVE_ATTRIBUTES |
            gl::ACTIVE_UNIFORMS =>
                Ok(WebGLParameter::Int(gl.get_program_iv(program_id.get(), param_id))),
            _ => Err(WebGLError::InvalidEnum),
        };

        chan.send(result).unwrap();
    }

    fn shader_parameter(gl: &gl::Gl,
                        shader_id: WebGLShaderId,
                        param_id: u32,
                        chan: WebGLSender<WebGLResult<WebGLParameter>>) {
        let result = match param_id {
            gl::SHADER_TYPE =>
                Ok(WebGLParameter::Int(gl.get_shader_iv(shader_id.get(), param_id))),
            gl::DELETE_STATUS |
            gl::COMPILE_STATUS =>
                Ok(WebGLParameter::Bool(gl.get_shader_iv(shader_id.get(), param_id) != 0)),
            _ => Err(WebGLError::InvalidEnum),
        };

        chan.send(result).unwrap();
    }

    fn shader_precision_format(gl: &gl::Gl,
                               shader_type: u32,
                               precision_type: u32,
                               chan: WebGLSender<WebGLResult<(i32, i32, i32)>>) {
        let result = match precision_type {
            gl::LOW_FLOAT |
            gl::MEDIUM_FLOAT |
            gl::HIGH_FLOAT |
            gl::LOW_INT |
            gl::MEDIUM_INT |
            gl::HIGH_INT => {
                Ok(gl.get_shader_precision_format(shader_type, precision_type))
            },
            _=> {
                Err(WebGLError::InvalidEnum)
            }
        };

        chan.send(result).unwrap();
    }

    fn get_extensions(gl: &gl::Gl, chan: WebGLSender<String>) {
        chan.send(gl.get_string(gl::EXTENSIONS)).unwrap();
    }

    fn uniform_location(gl: &gl::Gl,
                        program_id: WebGLProgramId,
                        name: String,
                        chan: WebGLSender<Option<i32>>) {
        let location = gl.get_uniform_location(program_id.get(), &name);
        let location = if location == -1 {
            None
        } else {
            Some(location)
        };

        chan.send(location).unwrap();
    }


    fn shader_info_log(gl: &gl::Gl, shader_id: WebGLShaderId, chan: WebGLSender<String>) {
        let log = gl.get_shader_info_log(shader_id.get());
        chan.send(log).unwrap();
    }

    fn program_info_log(gl: &gl::Gl, program_id: WebGLProgramId, chan: WebGLSender<String>) {
        let log = gl.get_program_info_log(program_id.get());
        chan.send(log).unwrap();
    }

    #[allow(unsafe_code)]
    fn create_buffer(gl: &gl::Gl, chan: WebGLSender<Option<WebGLBufferId>>) {
        let buffer = gl.gen_buffers(1)[0];
        let buffer = if buffer == 0 {
            None
        } else {
            Some(unsafe { WebGLBufferId::new(buffer) })
        };
        chan.send(buffer).unwrap();
    }

    #[allow(unsafe_code)]
    fn create_framebuffer(gl: &gl::Gl, chan: WebGLSender<Option<WebGLFramebufferId>>) {
        let framebuffer = gl.gen_framebuffers(1)[0];
        let framebuffer = if framebuffer == 0 {
            None
        } else {
            Some(unsafe { WebGLFramebufferId::new(framebuffer) })
        };
        chan.send(framebuffer).unwrap();
    }

    #[allow(unsafe_code)]
    fn create_renderbuffer(gl: &gl::Gl, chan: WebGLSender<Option<WebGLRenderbufferId>>) {
        let renderbuffer = gl.gen_renderbuffers(1)[0];
        let renderbuffer = if renderbuffer == 0 {
            None
        } else {
            Some(unsafe { WebGLRenderbufferId::new(renderbuffer) })
        };
        chan.send(renderbuffer).unwrap();
    }

    #[allow(unsafe_code)]
    fn create_texture(gl: &gl::Gl, chan: WebGLSender<Option<WebGLTextureId>>) {
        let texture = gl.gen_textures(1)[0];
        let texture = if texture == 0 {
            None
        } else {
            Some(unsafe { WebGLTextureId::new(texture) })
        };
        chan.send(texture).unwrap();
    }

    #[allow(unsafe_code)]
    fn create_program(gl: &gl::Gl, chan: WebGLSender<Option<WebGLProgramId>>) {
        let program = gl.create_program();
        let program = if program == 0 {
            None
        } else {
            Some(unsafe { WebGLProgramId::new(program) })
        };
        chan.send(program).unwrap();
    }

    #[allow(unsafe_code)]
    fn create_shader(gl: &gl::Gl, shader_type: u32, chan: WebGLSender<Option<WebGLShaderId>>) {
        let shader = gl.create_shader(shader_type);
        let shader = if shader == 0 {
            None
        } else {
            Some(unsafe { WebGLShaderId::new(shader) })
        };
        chan.send(shader).unwrap();
    }

    #[allow(unsafe_code)]
    fn create_vertex_array(gl: &gl::Gl, chan: WebGLSender<Option<WebGLVertexArrayId>>) {
        let vao = gl.gen_vertex_arrays(1)[0];
        let vao = if vao == 0 {
            None
        } else {
            Some(unsafe { WebGLVertexArrayId::new(vao) })
        };
        chan.send(vao).unwrap();
    }

    #[inline]
    fn bind_framebuffer<Native: NativeGLContextMethods>(gl: &gl::Gl,
                                                        target: u32,
                                                        request: WebGLFramebufferBindingRequest,
                                                        ctx: &GLContext<Native>) {
        let id = match request {
            WebGLFramebufferBindingRequest::Explicit(id) => id.get(),
            WebGLFramebufferBindingRequest::Default =>
                ctx.borrow_draw_buffer().unwrap().get_framebuffer(),
        };

        gl.bind_framebuffer(target, id);
    }


    #[inline]
    fn compile_shader(gl: &gl::Gl, shader_id: WebGLShaderId, source: String) {
        gl.shader_source(shader_id.get(), &[source.as_bytes()]);
        gl.compile_shader(shader_id.get());
    }
}
