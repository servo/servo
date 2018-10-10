/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_traits::webgl::*;
use euclid::Size2D;
use fnv::FnvHashMap;
use gleam::gl;
use offscreen_gl_context::{GLContext, GLContextAttributes, GLLimits, NativeGLContextMethods};
use pixels;
use std::thread;
use super::gl_context::{GLContextFactory, GLContextWrapper};
use webrender;
use webrender_api;

/// WebGL Threading API entry point that lives in the constellation.
/// It allows to get a WebGLThread handle for each script pipeline.
pub use ::webgl_mode::WebGLThreads;

struct GLContextData {
    ctx: GLContextWrapper,
    state: GLState,
}

pub struct GLState {
    clear_color: (f32, f32, f32, f32),
    scissor_test_enabled: bool,
    stencil_write_mask: (u32, u32),
    stencil_clear_value: i32,
    depth_write_mask: bool,
    depth_clear_value: f64,
}

impl Default for GLState {
    fn default() -> GLState {
        GLState {
            clear_color: (0., 0., 0., 0.),
            scissor_test_enabled: false,
            stencil_write_mask: (0, 0),
            stencil_clear_value: 0,
            depth_write_mask: true,
            depth_clear_value: 1.,
        }
    }
}

/// A WebGLThread manages the life cycle and message multiplexing of
/// a set of WebGLContexts living in the same thread.
pub struct WebGLThread<VR: WebVRRenderHandler + 'static> {
    /// Factory used to create a new GLContext shared with the WR/Main thread.
    gl_factory: GLContextFactory,
    /// Channel used to generate/update or delete `webrender_api::ImageKey`s.
    webrender_api: webrender_api::RenderApi,
    /// Map of live WebGLContexts.
    contexts: FnvHashMap<WebGLContextId, GLContextData>,
    /// Cached information for WebGLContexts.
    cached_context_info: FnvHashMap<WebGLContextId, WebGLContextInfo>,
    /// Current bound context.
    bound_context_id: Option<WebGLContextId>,
    /// Id generator for new WebGLContexts.
    next_webgl_id: usize,
    /// Handler user to send WebVR commands.
    webvr_compositor: Option<VR>,
    /// Texture ids and sizes used in DOM to texture outputs.
    dom_outputs: FnvHashMap<webrender_api::PipelineId, DOMToTextureData>,
}

impl<VR: WebVRRenderHandler + 'static> WebGLThread<VR> {
    pub fn new(
        gl_factory: GLContextFactory,
        webrender_api_sender: webrender_api::RenderApiSender,
        webvr_compositor: Option<VR>,
    ) -> Self {
        WebGLThread {
            gl_factory,
            webrender_api: webrender_api_sender.create_api(),
            contexts: Default::default(),
            cached_context_info: Default::default(),
            bound_context_id: None,
            next_webgl_id: 0,
            webvr_compositor,
            dom_outputs: Default::default(),
        }
    }

    /// Creates a new `WebGLThread` and returns a Sender to
    /// communicate with it.
    pub fn start(
        gl_factory: GLContextFactory,
        webrender_api_sender: webrender_api::RenderApiSender,
        webvr_compositor: Option<VR>,
    ) -> WebGLSender<WebGLMsg> {
        let (sender, receiver) = webgl_channel::<WebGLMsg>().unwrap();
        let result = sender.clone();
        thread::Builder::new().name("WebGLThread".to_owned()).spawn(move || {
            let mut renderer = WebGLThread::new(
                gl_factory,
                webrender_api_sender,
                webvr_compositor,
            );
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
                    let data = Self::make_current_if_needed(id, &self.contexts, &mut self.bound_context_id)
                                    .expect("WebGLContext not found");
                    let glsl_version = Self::get_glsl_version(&data.ctx);

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
            WebGLMsg::WebGLCommand(ctx_id, command, backtrace) => {
                self.handle_webgl_command(ctx_id, command, backtrace);
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
    fn handle_webgl_command(
        &mut self,
        context_id: WebGLContextId,
        command: WebGLCommand,
        backtrace: WebGLCommandBacktrace,
    ) {
        let data = Self::make_current_if_needed_mut(context_id, &mut self.contexts, &mut self.bound_context_id);
        if let Some(data) = data {
            data.ctx.apply_command(command, backtrace, &mut data.state);
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
        let data = Self::make_current_if_needed(context_id, &self.contexts, &mut self.bound_context_id)
                        .expect("WebGLContext not found in a WebGLMsg::Lock message");
        let info = self.cached_context_info.get_mut(&context_id).unwrap();
        // Insert a OpenGL Fence sync object that sends a signal when all the WebGL commands are finished.
        // The related gl().wait_sync call is performed in the WR thread. See WebGLExternalImageApi for mor details.
        let gl_sync = data.ctx.gl().fence_sync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0);
        info.gl_sync = Some(gl_sync);
        // It is important that the fence sync is properly flushed into the GPU's command queue.
        // Without proper flushing, the sync object may never be signaled.
        data.ctx.gl().flush();

        sender.send((info.texture_id, info.size, gl_sync as usize)).unwrap();
    }

    /// Handles an unlock external callback received from webrender::ExternalImageHandler
    fn handle_unlock(&mut self, context_id: WebGLContextId) {
        let data = Self::make_current_if_needed(context_id, &self.contexts, &mut self.bound_context_id)
                        .expect("WebGLContext not found in a WebGLMsg::Unlock message");
        let info = self.cached_context_info.get_mut(&context_id).unwrap();
        if let Some(gl_sync) = info.gl_sync.take() {
            // Release the GLSync object.
            data.ctx.gl().delete_sync(gl_sync);
        }
    }

    /// Creates a new WebGLContext
    fn create_webgl_context(
        &mut self,
        version: WebGLVersion,
        size: Size2D<u32>,
        attributes: GLContextAttributes,
    ) -> Result<(WebGLContextId, GLLimits, WebGLContextShareMode), String> {
        // Creating a new GLContext may make the current bound context_id dirty.
        // Clear it to ensure that  make_current() is called in subsequent commands.
        self.bound_context_id = None;

        // First try to create a shared context for the best performance.
        // Fallback to readback mode if the shared context creation fails.
        let (ctx, share_mode) = self.gl_factory
            .new_shared_context(version, size, attributes)
            .map(|r| (r, WebGLContextShareMode::SharedTexture))
            .or_else(|err| {
                warn!(
                    "Couldn't create shared GL context ({}), using slow readback context instead.",
                    err
                );
                let ctx = self.gl_factory.new_context(version, size, attributes)?;
                Ok((ctx, WebGLContextShareMode::Readback))
            })
            .map_err(|msg: &str| msg.to_owned())?;

        let id = WebGLContextId(self.next_webgl_id);
        let (size, texture_id, limits) = ctx.get_info();
        self.next_webgl_id += 1;
        self.contexts.insert(id, GLContextData {
            ctx,
            state: Default::default(),
        });
        self.cached_context_info.insert(id, WebGLContextInfo {
            texture_id,
            size,
            alpha: attributes.alpha,
            image_key: None,
            share_mode,
            gl_sync: None,
        });

        Ok((id, limits, share_mode))
    }

    /// Resizes a WebGLContext
    fn resize_webgl_context(
        &mut self,
        context_id: WebGLContextId,
        size: Size2D<u32>,
        sender: WebGLSender<Result<(), String>>,
    ) {
        let data = Self::make_current_if_needed_mut(
            context_id,
            &mut self.contexts,
            &mut self.bound_context_id
        ).expect("Missing WebGL context!");
        match data.ctx.resize(size) {
            Ok(_) => {
                let (real_size, texture_id, _) = data.ctx.get_info();
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
            let mut txn = webrender_api::Transaction::new();

            if let Some(image_key) = info.image_key {
                txn.delete_image(image_key);
            }

            self.webrender_api.update_resources(txn.resource_updates)
        }

        // Release GL context.
        self.contexts.remove(&context_id);

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
                let pixels = Self::raw_pixels(&self.contexts[&context_id].ctx, info.size);
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
                let data = Self::make_current_if_needed(context_id, &self.contexts, &mut self.bound_context_id)
                                .expect("WebGLContext not found in a WebGL DOMToTextureCommand::Attach command");
                // Initialize the texture that WR will use for frame outputs.
                data.ctx.gl().tex_image_2d(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGBA as gl::GLint,
                    size.width,
                    size.height,
                    0,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    None
                );
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
                let result = self.dom_outputs.get(&pipeline_id).and_then(|dom_data| {
                    let data = Self::make_current_if_needed(dom_data.context_id, contexts, bound_context_id);
                    data.and_then(|data| {
                        // The next glWaitSync call is used to synchronize the two flows of
                        // OpenGL commands (WR and WebGL) in order to avoid using semi-ready WR textures.
                        // glWaitSync doesn't block WebGL CPU thread.
                        data.ctx.gl().wait_sync(gl_sync as gl::GLsync, 0, gl::TIMEOUT_IGNORED);
                        Some((dom_data.texture_id.get(), dom_data.size))
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
                                  contexts: &'a FnvHashMap<WebGLContextId, GLContextData>,
                                  bound_id: &mut Option<WebGLContextId>) -> Option<&'a GLContextData> {
        let data = contexts.get(&context_id);

        if let Some(data) = data {
            if Some(context_id) != *bound_id {
                data.ctx.make_current();
                *bound_id = Some(context_id);
            }
        }

        data
    }

    /// Gets a mutable reference to a GLContextWrapper for a WebGLContextId and makes it current if required.
    fn make_current_if_needed_mut<'a>(
        context_id: WebGLContextId,
        contexts: &'a mut FnvHashMap<WebGLContextId, GLContextData>,
        bound_id: &mut Option<WebGLContextId>)
        -> Option<&'a mut GLContextData>
    {
        let data = contexts.get_mut(&context_id);

        if let Some(ref data) = data {
            if Some(context_id) != *bound_id {
                data.ctx.make_current();
                *bound_id = Some(context_id);
            }
        }

        data
    }

    /// Creates a `webrender_api::ImageKey` that uses shared textures.
    fn create_wr_external_image(webrender_api: &webrender_api::RenderApi,
                                size: Size2D<i32>,
                                alpha: bool,
                                context_id: WebGLContextId) -> webrender_api::ImageKey {
        let descriptor = Self::image_descriptor(size, alpha);
        let data = Self::external_image_data(context_id);

        let image_key = webrender_api.generate_image_key();
        let mut txn = webrender_api::Transaction::new();
        txn.add_image(image_key, descriptor, data, None);
        webrender_api.update_resources(txn.resource_updates);

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

        let mut txn = webrender_api::Transaction::new();
        txn.update_image(image_key, descriptor, data, None);
        webrender_api.update_resources(txn.resource_updates);
    }

    /// Creates a `webrender_api::ImageKey` that uses raw pixels.
    fn create_wr_readback_image(webrender_api: &webrender_api::RenderApi,
                                size: Size2D<i32>,
                                alpha: bool,
                                data: Vec<u8>) -> webrender_api::ImageKey {
        let descriptor = Self::image_descriptor(size, alpha);
        let data = webrender_api::ImageData::new(data);

        let image_key = webrender_api.generate_image_key();
        let mut txn = webrender_api::Transaction::new();
        txn.add_image(image_key, descriptor, data, None);
        webrender_api.update_resources(txn.resource_updates);

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

        let mut txn = webrender_api::Transaction::new();
        txn.update_image(image_key, descriptor, data, None);
        webrender_api.update_resources(txn.resource_updates);
    }

    /// Helper function to create a `webrender_api::ImageDescriptor`.
    fn image_descriptor(size: Size2D<i32>, alpha: bool) -> webrender_api::ImageDescriptor {
        webrender_api::ImageDescriptor {
            size: webrender_api::DeviceUintSize::new(size.width as u32, size.height as u32),
            stride: None,
            format: webrender_api::ImageFormat::BGRA8,
            offset: 0,
            is_opaque: !alpha,
            allow_mipmaps: false,
        }
    }

    /// Helper function to create a `webrender_api::ImageData::External` instance.
    fn external_image_data(context_id: WebGLContextId) -> webrender_api::ImageData {
        let data = webrender_api::ExternalImageData {
            id: webrender_api::ExternalImageId(context_id.0 as u64),
            channel_index: 0,
            image_type: webrender_api::ExternalImageType::TextureHandle(
                webrender_api::TextureTarget::Default,
            ),
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
        pixels::byte_swap_colors_inplace(&mut pixels);
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

impl<VR: WebVRRenderHandler + 'static> Drop for WebGLThread<VR> {
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
            uv: webrender_api::TexelRect::new(
                0.0,
                size.height as f32,
                size.width as f32,
                0.0,
            ),
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
    #[allow(unsafe_code)]
    pub fn apply<Native: NativeGLContextMethods>(
        ctx: &GLContext<Native>,
        state: &mut GLState,
        command: WebGLCommand,
        _backtrace: WebGLCommandBacktrace,
    ) {
        match command {
            WebGLCommand::GetContextAttributes(ref sender) =>
                sender.send(*ctx.borrow_attributes()).unwrap(),
            WebGLCommand::ActiveTexture(target) =>
                ctx.gl().active_texture(target),
            WebGLCommand::AttachShader(program_id, shader_id) =>
                ctx.gl().attach_shader(program_id.get(), shader_id.get()),
            WebGLCommand::DetachShader(program_id, shader_id) =>
                ctx.gl().detach_shader(program_id.get(), shader_id.get()),
            WebGLCommand::BindAttribLocation(program_id, index, ref name) => {
                ctx.gl().bind_attrib_location(program_id.get(), index, &to_name_in_compiled_shader(name))
            }
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
            WebGLCommand::BufferData(buffer_type, ref receiver, usage) => {
                gl::buffer_data(ctx.gl(), buffer_type, &receiver.recv().unwrap(), usage)
            },
            WebGLCommand::BufferSubData(buffer_type, offset, ref receiver) => {
                gl::buffer_sub_data(ctx.gl(), buffer_type, offset, &receiver.recv().unwrap())
            },
            WebGLCommand::Clear(mask) =>
                ctx.gl().clear(mask),
            WebGLCommand::ClearColor(r, g, b, a) => {
                state.clear_color = (r, g, b, a);
                ctx.gl().clear_color(r, g, b, a);
            }
            WebGLCommand::ClearDepth(depth) => {
                let value = depth.max(0.).min(1.) as f64;
                state.depth_clear_value = value;
                ctx.gl().clear_depth(value)
            }
            WebGLCommand::ClearStencil(stencil) => {
                state.stencil_clear_value = stencil;
                ctx.gl().clear_stencil(stencil);
            }
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
            WebGLCommand::DepthMask(flag) => {
                state.depth_write_mask = flag;
                ctx.gl().depth_mask(flag);
            }
            WebGLCommand::DepthRange(near, far) => {
                ctx.gl().depth_range(near.max(0.).min(1.) as f64, far.max(0.).min(1.) as f64)
            }
            WebGLCommand::Disable(cap) => {
                if cap == gl::SCISSOR_TEST {
                    state.scissor_test_enabled = false;
                }
                ctx.gl().disable(cap);
            }
            WebGLCommand::Enable(cap) => {
                if cap == gl::SCISSOR_TEST {
                    state.scissor_test_enabled = true;
                }
                ctx.gl().enable(cap);
            }
            WebGLCommand::FramebufferRenderbuffer(target, attachment, renderbuffertarget, rb) => {
                let attach = |attachment| {
                    ctx.gl().framebuffer_renderbuffer(target, attachment,
                                                      renderbuffertarget,
                                                      rb.map_or(0, WebGLRenderbufferId::get))
                };
                if attachment == gl::DEPTH_STENCIL_ATTACHMENT {
                    attach(gl::DEPTH_ATTACHMENT);
                    attach(gl::STENCIL_ATTACHMENT);
                } else {
                    attach(attachment);
                }
            }
            WebGLCommand::FramebufferTexture2D(target, attachment, textarget, texture, level) => {
                let attach = |attachment| {
                    ctx.gl().framebuffer_texture_2d(target, attachment, textarget,
                                                    texture.map_or(0, WebGLTextureId::get), level)
                };
                if attachment == gl::DEPTH_STENCIL_ATTACHMENT {
                    attach(gl::DEPTH_ATTACHMENT);
                    attach(gl::STENCIL_ATTACHMENT);
                } else {
                    attach(attachment)
                }
            }
            WebGLCommand::FrontFace(mode) =>
                ctx.gl().front_face(mode),
            WebGLCommand::DisableVertexAttribArray(attrib_id) =>
                ctx.gl().disable_vertex_attrib_array(attrib_id),
            WebGLCommand::EnableVertexAttribArray(attrib_id) =>
                ctx.gl().enable_vertex_attrib_array(attrib_id),
            WebGLCommand::Hint(name, val) =>
                ctx.gl().hint(name, val),
            WebGLCommand::LineWidth(width) =>
                ctx.gl().line_width(width),
            WebGLCommand::PixelStorei(name, val) =>
                ctx.gl().pixel_store_i(name, val),
            WebGLCommand::PolygonOffset(factor, units) =>
                ctx.gl().polygon_offset(factor, units),
            WebGLCommand::ReadPixels(rect, format, pixel_type, ref sender) => {
                let pixels = ctx.gl().read_pixels(
                    rect.origin.x as i32,
                    rect.origin.y as i32,
                    rect.size.width as i32,
                    rect.size.height as i32,
                    format,
                    pixel_type,
                );
                sender.send(&pixels).unwrap();
            }
            WebGLCommand::RenderbufferStorage(target, format, width, height) =>
                ctx.gl().renderbuffer_storage(target, format, width, height),
            WebGLCommand::SampleCoverage(value, invert) =>
                ctx.gl().sample_coverage(value, invert),
            WebGLCommand::Scissor(x, y, width, height) => {
                // FIXME(nox): Kinda unfortunate that some u32 values could
                // end up as negative numbers here, but I don't even think
                // that can happen in the real world.
                ctx.gl().scissor(x, y, width as i32, height as i32);
            },
            WebGLCommand::StencilFunc(func, ref_, mask) =>
                ctx.gl().stencil_func(func, ref_, mask),
            WebGLCommand::StencilFuncSeparate(face, func, ref_, mask) =>
                ctx.gl().stencil_func_separate(face, func, ref_, mask),
            WebGLCommand::StencilMask(mask) => {
                state.stencil_write_mask = (mask, mask);
                ctx.gl().stencil_mask(mask);
            }
            WebGLCommand::StencilMaskSeparate(face, mask) => {
                if face == gl::FRONT {
                    state.stencil_write_mask.0 = mask;
                } else {
                    state.stencil_write_mask.1 = mask;
                }
                ctx.gl().stencil_mask_separate(face, mask);
            }
            WebGLCommand::StencilOp(fail, zfail, zpass) =>
                ctx.gl().stencil_op(fail, zfail, zpass),
            WebGLCommand::StencilOpSeparate(face, fail, zfail, zpass) =>
                ctx.gl().stencil_op_separate(face, fail, zfail, zpass),
            WebGLCommand::GetRenderbufferParameter(target, pname, ref chan) =>
                Self::get_renderbuffer_parameter(ctx.gl(), target, pname, chan),
            WebGLCommand::GetFramebufferAttachmentParameter(target, attachment, pname, ref chan) =>
                Self::get_framebuffer_attachment_parameter(ctx.gl(), target, attachment, pname, chan),
            WebGLCommand::GetShaderPrecisionFormat(shader_type, precision_type, ref chan) =>
                Self::shader_precision_format(ctx.gl(), shader_type, precision_type, chan),
            WebGLCommand::GetExtensions(ref chan) =>
                Self::get_extensions(ctx.gl(), chan),
            WebGLCommand::GetUniformLocation(program_id, ref name, ref chan) =>
                Self::uniform_location(ctx.gl(), program_id, &name, chan),
            WebGLCommand::GetShaderInfoLog(shader_id, ref chan) =>
                Self::shader_info_log(ctx.gl(), shader_id, chan),
            WebGLCommand::GetProgramInfoLog(program_id, ref chan) =>
                Self::program_info_log(ctx.gl(), program_id, chan),
            WebGLCommand::CompileShader(shader_id, ref source) =>
                Self::compile_shader(ctx.gl(), shader_id, &source),
            WebGLCommand::CreateBuffer(ref chan) =>
                Self::create_buffer(ctx.gl(), chan),
            WebGLCommand::CreateFramebuffer(ref chan) =>
                Self::create_framebuffer(ctx.gl(), chan),
            WebGLCommand::CreateRenderbuffer(ref chan) =>
                Self::create_renderbuffer(ctx.gl(), chan),
            WebGLCommand::CreateTexture(ref chan) =>
                Self::create_texture(ctx.gl(), chan),
            WebGLCommand::CreateProgram(ref chan) =>
                Self::create_program(ctx.gl(), chan),
            WebGLCommand::CreateShader(shader_type, ref chan) =>
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
            WebGLCommand::Uniform1f(uniform_id, v) =>
                ctx.gl().uniform_1f(uniform_id, v),
            WebGLCommand::Uniform1fv(uniform_id, ref v) =>
                ctx.gl().uniform_1fv(uniform_id, v),
            WebGLCommand::Uniform1i(uniform_id, v) =>
                ctx.gl().uniform_1i(uniform_id, v),
            WebGLCommand::Uniform1iv(uniform_id, ref v) =>
                ctx.gl().uniform_1iv(uniform_id, v),
            WebGLCommand::Uniform2f(uniform_id, x, y) =>
                ctx.gl().uniform_2f(uniform_id, x, y),
            WebGLCommand::Uniform2fv(uniform_id, ref v) =>
                ctx.gl().uniform_2fv(uniform_id, v),
            WebGLCommand::Uniform2i(uniform_id, x, y) =>
                ctx.gl().uniform_2i(uniform_id, x, y),
            WebGLCommand::Uniform2iv(uniform_id, ref v) =>
                ctx.gl().uniform_2iv(uniform_id, v),
            WebGLCommand::Uniform3f(uniform_id, x, y, z) =>
                ctx.gl().uniform_3f(uniform_id, x, y, z),
            WebGLCommand::Uniform3fv(uniform_id, ref v) =>
                ctx.gl().uniform_3fv(uniform_id, v),
            WebGLCommand::Uniform3i(uniform_id, x, y, z) =>
                ctx.gl().uniform_3i(uniform_id, x, y, z),
            WebGLCommand::Uniform3iv(uniform_id, ref v) =>
                ctx.gl().uniform_3iv(uniform_id, v),
            WebGLCommand::Uniform4f(uniform_id, x, y, z, w) =>
                ctx.gl().uniform_4f(uniform_id, x, y, z, w),
            WebGLCommand::Uniform4fv(uniform_id, ref v) =>
                ctx.gl().uniform_4fv(uniform_id, v),
            WebGLCommand::Uniform4i(uniform_id, x, y, z, w) =>
                ctx.gl().uniform_4i(uniform_id, x, y, z, w),
            WebGLCommand::Uniform4iv(uniform_id, ref v) =>
                ctx.gl().uniform_4iv(uniform_id, v),
            WebGLCommand::UniformMatrix2fv(uniform_id, ref v) => {
                ctx.gl().uniform_matrix_2fv(uniform_id, false, v)
            }
            WebGLCommand::UniformMatrix3fv(uniform_id, ref v) => {
                ctx.gl().uniform_matrix_3fv(uniform_id, false, v)
            }
            WebGLCommand::UniformMatrix4fv(uniform_id, ref v) => {
                ctx.gl().uniform_matrix_4fv(uniform_id, false, v)
            }
            WebGLCommand::ValidateProgram(program_id) =>
                ctx.gl().validate_program(program_id.get()),
            WebGLCommand::VertexAttrib(attrib_id, x, y, z, w) =>
                ctx.gl().vertex_attrib_4f(attrib_id, x, y, z, w),
            WebGLCommand::VertexAttribPointer2f(attrib_id, size, normalized, stride, offset) =>
                ctx.gl().vertex_attrib_pointer_f32(attrib_id, size, normalized, stride, offset),
            WebGLCommand::VertexAttribPointer(attrib_id, size, data_type, normalized, stride, offset) =>
                ctx.gl().vertex_attrib_pointer(attrib_id, size, data_type, normalized, stride, offset),
            WebGLCommand::SetViewport(x, y, width, height) => {
                ctx.gl().viewport(x, y, width, height);
            }
            WebGLCommand::TexImage2D(target, level, internal, width, height, format, data_type, ref chan) => {
                ctx.gl().tex_image_2d(
                    target,
                    level,
                    internal,
                    width,
                    height,
                    0,
                    format,
                    data_type,
                    Some(&chan.recv().unwrap()),
                )
            }
            WebGLCommand::TexSubImage2D(target, level, xoffset, yoffset, x, y, width, height, ref chan) => {
                ctx.gl().tex_sub_image_2d(
                    target,
                    level,
                    xoffset,
                    yoffset,
                    x,
                    y,
                    width,
                    height,
                    &chan.recv().unwrap(),
                )
            }
            WebGLCommand::DrawingBufferWidth(ref sender) =>
                sender.send(ctx.borrow_draw_buffer().unwrap().size().width).unwrap(),
            WebGLCommand::DrawingBufferHeight(ref sender) =>
                sender.send(ctx.borrow_draw_buffer().unwrap().size().height).unwrap(),
            WebGLCommand::Finish(ref sender) =>
                Self::finish(ctx.gl(), sender),
            WebGLCommand::Flush =>
                ctx.gl().flush(),
            WebGLCommand::GenerateMipmap(target) =>
                ctx.gl().generate_mipmap(target),
            WebGLCommand::CreateVertexArray(ref chan) =>
                Self::create_vertex_array(ctx.gl(), chan),
            WebGLCommand::DeleteVertexArray(id) =>
                ctx.gl().delete_vertex_arrays(&[id.get()]),
            WebGLCommand::BindVertexArray(id) =>
                ctx.gl().bind_vertex_array(id.map_or(0, WebGLVertexArrayId::get)),
            WebGLCommand::GetParameterBool(param, ref sender) => {
                let mut value = [0];
                unsafe {
                    ctx.gl().get_boolean_v(param as u32, &mut value);
                }
                sender.send(value[0] != 0).unwrap()
            }
            WebGLCommand::GetParameterBool4(param, ref sender) => {
                let mut value = [0; 4];
                unsafe {
                    ctx.gl().get_boolean_v(param as u32, &mut value);
                }
                let value = [
                    value[0] != 0,
                    value[1] != 0,
                    value[2] != 0,
                    value[3] != 0,
                ];
                sender.send(value).unwrap()
            }
            WebGLCommand::GetParameterInt(param, ref sender) => {
                let mut value = [0];
                unsafe {
                    ctx.gl().get_integer_v(param as u32, &mut value);
                }
                sender.send(value[0]).unwrap()
            }
            WebGLCommand::GetParameterInt2(param, ref sender) => {
                let mut value = [0; 2];
                unsafe {
                    ctx.gl().get_integer_v(param as u32, &mut value);
                }
                sender.send(value).unwrap()
            }
            WebGLCommand::GetParameterInt4(param, ref sender) => {
                let mut value = [0; 4];
                unsafe {
                    ctx.gl().get_integer_v(param as u32, &mut value);
                }
                sender.send(value).unwrap()
            }
            WebGLCommand::GetParameterFloat(param, ref sender) => {
                let mut value = [0.];
                unsafe {
                    ctx.gl().get_float_v(param as u32, &mut value);
                }
                sender.send(value[0]).unwrap()
            }
            WebGLCommand::GetParameterFloat2(param, ref sender) => {
                let mut value = [0.; 2];
                unsafe {
                    ctx.gl().get_float_v(param as u32, &mut value);
                }
                sender.send(value).unwrap()
            }
            WebGLCommand::GetParameterFloat4(param, ref sender) => {
                let mut value = [0.; 4];
                unsafe {
                    ctx.gl().get_float_v(param as u32, &mut value);
                }
                sender.send(value).unwrap()
            }
            WebGLCommand::GetProgramValidateStatus(program, ref sender) => {
                let mut value = [0];
                unsafe {
                    ctx.gl().get_program_iv(program.get(), gl::VALIDATE_STATUS, &mut value);
                }
                sender.send(value[0] != 0).unwrap()
            }
            WebGLCommand::GetProgramActiveUniforms(program, ref sender) => {
                let mut value = [0];
                unsafe {
                    ctx.gl().get_program_iv(program.get(), gl::ACTIVE_UNIFORMS, &mut value);
                }
                sender.send(value[0]).unwrap()
            }
            WebGLCommand::GetCurrentVertexAttrib(index, ref sender) => {
                let mut value = [0.; 4];
                unsafe {
                    ctx.gl().get_vertex_attrib_fv(index, gl::CURRENT_VERTEX_ATTRIB, &mut value);
                }
                sender.send(value).unwrap();
            }
            WebGLCommand::GetTexParameterFloat(target, param, ref sender) => {
                sender.send(ctx.gl().get_tex_parameter_fv(target, param as u32)).unwrap();
            }
            WebGLCommand::GetTexParameterInt(target, param, ref sender) => {
                sender.send(ctx.gl().get_tex_parameter_iv(target, param as u32)).unwrap();
            }
            WebGLCommand::TexParameteri(target, param, value) => {
                ctx.gl().tex_parameter_i(target, param as u32, value)
            }
            WebGLCommand::TexParameterf(target, param, value) => {
                ctx.gl().tex_parameter_f(target, param as u32, value)
            }
            WebGLCommand::LinkProgram(program_id, ref sender) => {
                return sender.send(Self::link_program(ctx.gl(), program_id)).unwrap();
            }
            WebGLCommand::UseProgram(program_id) => {
                ctx.gl().use_program(program_id.map_or(0, |p| p.get()))
            }
            WebGLCommand::DrawArrays { mode, first, count } => {
                ctx.gl().draw_arrays(mode, first, count)
            }
            WebGLCommand::DrawArraysInstanced { mode, first, count, primcount } => {
                ctx.gl().draw_arrays_instanced(mode, first, count, primcount)
            }
            WebGLCommand::DrawElements { mode, count, type_, offset } => {
                ctx.gl().draw_elements(mode, count, type_, offset)
            }
            WebGLCommand::DrawElementsInstanced { mode, count, type_, offset, primcount } => {
                ctx.gl().draw_elements_instanced(mode, count, type_, offset, primcount)
            }
            WebGLCommand::VertexAttribDivisor { index, divisor } => {
                ctx.gl().vertex_attrib_divisor(index, divisor)
            }
            WebGLCommand::GetUniformBool(program_id, loc, ref sender) => {
                let mut value = [0];
                unsafe {
                    ctx.gl().get_uniform_iv(program_id.get(), loc, &mut value);
                }
                sender.send(value[0] != 0).unwrap();
            }
            WebGLCommand::GetUniformBool2(program_id, loc, ref sender) => {
                let mut value = [0; 2];
                unsafe {
                    ctx.gl().get_uniform_iv(program_id.get(), loc, &mut value);
                }
                let value = [
                    value[0] != 0,
                    value[1] != 0,
                ];
                sender.send(value).unwrap();
            }
            WebGLCommand::GetUniformBool3(program_id, loc, ref sender) => {
                let mut value = [0; 3];
                unsafe {
                    ctx.gl().get_uniform_iv(program_id.get(), loc, &mut value);
                }
                let value = [
                    value[0] != 0,
                    value[1] != 0,
                    value[2] != 0,
                ];
                sender.send(value).unwrap();
            }
            WebGLCommand::GetUniformBool4(program_id, loc, ref sender) => {
                let mut value = [0; 4];
                unsafe {
                    ctx.gl().get_uniform_iv(program_id.get(), loc, &mut value);
                }
                let value = [
                    value[0] != 0,
                    value[1] != 0,
                    value[2] != 0,
                    value[3] != 0,
                ];
                sender.send(value).unwrap();
            }
            WebGLCommand::GetUniformInt(program_id, loc, ref sender) => {
                let mut value = [0];
                unsafe {
                    ctx.gl().get_uniform_iv(program_id.get(), loc, &mut value);
                }
                sender.send(value[0]).unwrap();
            }
            WebGLCommand::GetUniformInt2(program_id, loc, ref sender) => {
                let mut value = [0; 2];
                unsafe {
                    ctx.gl().get_uniform_iv(program_id.get(), loc, &mut value);
                }
                sender.send(value).unwrap();
            }
            WebGLCommand::GetUniformInt3(program_id, loc, ref sender) => {
                let mut value = [0; 3];
                unsafe {
                    ctx.gl().get_uniform_iv(program_id.get(), loc, &mut value);
                }
                sender.send(value).unwrap();
            }
            WebGLCommand::GetUniformInt4(program_id, loc, ref sender) => {
                let mut value = [0; 4];
                unsafe {
                    ctx.gl().get_uniform_iv(program_id.get(), loc, &mut value);
                }
                sender.send(value).unwrap();
            }
            WebGLCommand::GetUniformFloat(program_id, loc, ref sender) => {
                let mut value = [0.];
                unsafe {
                    ctx.gl().get_uniform_fv(program_id.get(), loc, &mut value);
                }
                sender.send(value[0]).unwrap();
            }
            WebGLCommand::GetUniformFloat2(program_id, loc, ref sender) => {
                let mut value = [0.; 2];
                unsafe {
                    ctx.gl().get_uniform_fv(program_id.get(), loc, &mut value);
                }
                sender.send(value).unwrap();
            }
            WebGLCommand::GetUniformFloat3(program_id, loc, ref sender) => {
                let mut value = [0.; 3];
                unsafe {
                    ctx.gl().get_uniform_fv(program_id.get(), loc, &mut value);
                }
                sender.send(value).unwrap();
            }
            WebGLCommand::GetUniformFloat4(program_id, loc, ref sender) => {
                let mut value = [0.; 4];
                unsafe {
                    ctx.gl().get_uniform_fv(program_id.get(), loc, &mut value);
                }
                sender.send(value).unwrap();
            }
            WebGLCommand::GetUniformFloat9(program_id, loc, ref sender) => {
                let mut value = [0.; 9];
                unsafe {
                    ctx.gl().get_uniform_fv(program_id.get(), loc, &mut value);
                }
                sender.send(value).unwrap();
            }
            WebGLCommand::GetUniformFloat16(program_id, loc, ref sender) => {
                let mut value = [0.; 16];
                unsafe {
                    ctx.gl().get_uniform_fv(program_id.get(), loc, &mut value);
                }
                sender.send(value).unwrap();
            }
            WebGLCommand::InitializeFramebuffer { color, depth, stencil } => {
                Self::initialize_framebuffer(ctx.gl(), state, color, depth, stencil)
            }
        }

        // TODO: update test expectations in order to enable debug assertions
        let error = ctx.gl().get_error();
        if error != gl::NO_ERROR {
            error!("Last GL operation failed: {:?}", command);
            #[cfg(feature = "webgl_backtrace")]
            {
                error!("Backtrace from failed WebGL API:\n{}", _backtrace.backtrace);
                if let Some(backtrace) = _backtrace.js_backtrace {
                    error!("JS backtrace from failed WebGL API:\n{}", backtrace);
                }
            }
        }
        assert_eq!(error, gl::NO_ERROR, "Unexpected WebGL error: 0x{:x} ({})", error, error);
    }

    fn initialize_framebuffer(
        gl: &gl::Gl,
        state: &GLState,
        color: bool,
        depth: bool,
        stencil: bool,
    ) {
        let bits = [
            (color, gl::COLOR_BUFFER_BIT),
            (depth, gl::DEPTH_BUFFER_BIT),
            (stencil, gl::STENCIL_BUFFER_BIT),
        ].iter().fold(0, |bits, &(enabled, bit)| bits | if enabled { bit } else { 0 });

        if state.scissor_test_enabled {
            gl.disable(gl::SCISSOR_TEST);
        }

        if color {
            gl.clear_color(0., 0., 0., 0.);
        }

        if depth {
            gl.depth_mask(true);
            gl.clear_depth(1.);
        }

        if stencil {
            gl.stencil_mask_separate(gl::FRONT, 0xFFFFFFFF);
            gl.stencil_mask_separate(gl::BACK, 0xFFFFFFFF);
            gl.clear_stencil(0);
        }

        gl.clear(bits);

        if state.scissor_test_enabled {
            gl.enable(gl::SCISSOR_TEST);
        }

        if color {
            let (r, g, b, a) = state.clear_color;
            gl.clear_color(r, g, b, a);
        }

        if depth {
            gl.depth_mask(state.depth_write_mask);
            gl.clear_depth(state.depth_clear_value);
        }

        if stencil {
            let (front, back) = state.stencil_write_mask;
            gl.stencil_mask_separate(gl::FRONT, front);
            gl.stencil_mask_separate(gl::BACK, back);
            gl.clear_stencil(state.stencil_clear_value);
        }
    }

    #[allow(unsafe_code)]
    fn link_program(gl: &gl::Gl, program: WebGLProgramId) -> ProgramLinkInfo {
        gl.link_program(program.get());
        let mut linked = [0];
        unsafe {
            gl.get_program_iv(program.get(), gl::LINK_STATUS, &mut linked);
        }
        if linked[0] == 0 {
            return ProgramLinkInfo {
                linked: false,
                active_attribs: vec![].into(),
                active_uniforms: vec![].into(),
            }
        }

        let mut num_active_attribs = [0];
        unsafe {
            gl.get_program_iv(program.get(), gl::ACTIVE_ATTRIBUTES, &mut num_active_attribs);
        }
        let active_attribs = (0..num_active_attribs[0] as u32).map(|i| {
            // FIXME(nox): This allocates strings sometimes for nothing
            // and the gleam method keeps getting ACTIVE_ATTRIBUTE_MAX_LENGTH.
            let (size, type_, name) = gl.get_active_attrib(program.get(), i);
            let location = if name.starts_with("gl_") {
                -1
            } else {
                gl.get_attrib_location(program.get(), &name)
            };
            ActiveAttribInfo {
                name: from_name_in_compiled_shader(&name),
                size,
                type_,
                location,
            }
        }).collect::<Vec<_>>().into();

        let mut num_active_uniforms = [0];
        unsafe {
            gl.get_program_iv(program.get(), gl::ACTIVE_UNIFORMS, &mut num_active_uniforms);
        }
        let active_uniforms = (0..num_active_uniforms[0] as u32).map(|i| {
            // FIXME(nox): This allocates strings sometimes for nothing
            // and the gleam method keeps getting ACTIVE_UNIFORM_MAX_LENGTH.
            let (size, type_, mut name) = gl.get_active_uniform(program.get(), i);
            let is_array = name.ends_with("[0]");
            if is_array {
                // FIXME(nox): NLL
                let len = name.len();
                name.truncate(len - 3);
            }
            ActiveUniformInfo {
                base_name: from_name_in_compiled_shader(&name).into(),
                size: if is_array { Some(size) } else { None },
                type_,
            }
        }).collect::<Vec<_>>().into();

        ProgramLinkInfo {
            linked: true,
            active_attribs,
            active_uniforms,
        }
    }

    fn finish(gl: &gl::Gl, chan: &WebGLSender<()>) {
        gl.finish();
        chan.send(()).unwrap();
    }

    fn shader_precision_format(gl: &gl::Gl,
                               shader_type: u32,
                               precision_type: u32,
                               chan: &WebGLSender<(i32, i32, i32)>) {
        let result = gl.get_shader_precision_format(shader_type, precision_type);
        chan.send(result).unwrap();
    }

    fn get_extensions(gl: &gl::Gl, chan: &WebGLSender<String>) {
        chan.send(gl.get_string(gl::EXTENSIONS)).unwrap();
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn get_framebuffer_attachment_parameter(
        gl: &gl::Gl,
        target: u32,
        attachment: u32,
        pname: u32,
        chan: &WebGLSender<i32>
    ) {
        let parameter = gl.get_framebuffer_attachment_parameter_iv(target, attachment, pname);
        chan.send(parameter).unwrap();
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7
    fn get_renderbuffer_parameter(
        gl: &gl::Gl,
        target: u32,
        pname: u32,
        chan: &WebGLSender<i32>
    ) {
        let parameter = gl.get_renderbuffer_parameter_iv(target, pname);
        chan.send(parameter).unwrap();
    }

    fn uniform_location(
        gl: &gl::Gl,
        program_id: WebGLProgramId,
        name: &str,
        chan: &WebGLSender<i32>,
    ) {
        let location = gl.get_uniform_location(program_id.get(), &to_name_in_compiled_shader(name));
        assert!(location >= 0);
        chan.send(location).unwrap();
    }


    fn shader_info_log(gl: &gl::Gl, shader_id: WebGLShaderId, chan: &WebGLSender<String>) {
        let log = gl.get_shader_info_log(shader_id.get());
        chan.send(log).unwrap();
    }

    fn program_info_log(gl: &gl::Gl, program_id: WebGLProgramId, chan: &WebGLSender<String>) {
        let log = gl.get_program_info_log(program_id.get());
        chan.send(log).unwrap();
    }

    #[allow(unsafe_code)]
    fn create_buffer(gl: &gl::Gl, chan: &WebGLSender<Option<WebGLBufferId>>) {
        let buffer = gl.gen_buffers(1)[0];
        let buffer = if buffer == 0 {
            None
        } else {
            Some(unsafe { WebGLBufferId::new(buffer) })
        };
        chan.send(buffer).unwrap();
    }

    #[allow(unsafe_code)]
    fn create_framebuffer(gl: &gl::Gl, chan: &WebGLSender<Option<WebGLFramebufferId>>) {
        let framebuffer = gl.gen_framebuffers(1)[0];
        let framebuffer = if framebuffer == 0 {
            None
        } else {
            Some(unsafe { WebGLFramebufferId::new(framebuffer) })
        };
        chan.send(framebuffer).unwrap();
    }

    #[allow(unsafe_code)]
    fn create_renderbuffer(gl: &gl::Gl, chan: &WebGLSender<Option<WebGLRenderbufferId>>) {
        let renderbuffer = gl.gen_renderbuffers(1)[0];
        let renderbuffer = if renderbuffer == 0 {
            None
        } else {
            Some(unsafe { WebGLRenderbufferId::new(renderbuffer) })
        };
        chan.send(renderbuffer).unwrap();
    }

    #[allow(unsafe_code)]
    fn create_texture(gl: &gl::Gl, chan: &WebGLSender<Option<WebGLTextureId>>) {
        let texture = gl.gen_textures(1)[0];
        let texture = if texture == 0 {
            None
        } else {
            Some(unsafe { WebGLTextureId::new(texture) })
        };
        chan.send(texture).unwrap();
    }

    #[allow(unsafe_code)]
    fn create_program(gl: &gl::Gl, chan: &WebGLSender<Option<WebGLProgramId>>) {
        let program = gl.create_program();
        let program = if program == 0 {
            None
        } else {
            Some(unsafe { WebGLProgramId::new(program) })
        };
        chan.send(program).unwrap();
    }

    #[allow(unsafe_code)]
    fn create_shader(gl: &gl::Gl, shader_type: u32, chan: &WebGLSender<Option<WebGLShaderId>>) {
        let shader = gl.create_shader(shader_type);
        let shader = if shader == 0 {
            None
        } else {
            Some(unsafe { WebGLShaderId::new(shader) })
        };
        chan.send(shader).unwrap();
    }

    #[allow(unsafe_code)]
    fn create_vertex_array(gl: &gl::Gl, chan: &WebGLSender<Option<WebGLVertexArrayId>>) {
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
    fn compile_shader(gl: &gl::Gl, shader_id: WebGLShaderId, source: &str) {
        gl.shader_source(shader_id.get(), &[source.as_bytes()]);
        gl.compile_shader(shader_id.get());
    }
}

/// ANGLE adds a `_u` prefix to variable names:
///
/// https://chromium.googlesource.com/angle/angle/+/855d964bd0d05f6b2cb303f625506cf53d37e94f
///
/// To avoid hard-coding this we would need to use the `sh::GetAttributes` and `sh::GetUniforms`
/// API to look up the `x.name` and `x.mappedName` members.
const ANGLE_NAME_PREFIX: &'static str = "_u";

fn to_name_in_compiled_shader(s: &str) -> String {
    map_dot_separated(s, |s, mapped| {
        mapped.push_str(ANGLE_NAME_PREFIX);
        mapped.push_str(s);
    })
}

fn from_name_in_compiled_shader(s: &str) -> String {
    map_dot_separated(s, |s, mapped| {
        mapped.push_str(if s.starts_with(ANGLE_NAME_PREFIX) {
            &s[ANGLE_NAME_PREFIX.len()..]
        } else {
            s
        })
    })
}

fn map_dot_separated<F: Fn(&str, &mut String)>(s: &str, f: F) -> String {
    let mut iter = s.split('.');
    let mut mapped = String::new();
    f(iter.next().unwrap(), &mut mapped);
    for s in iter {
        mapped.push('.');
        f(s, &mut mapped);
    }
    mapped
}
