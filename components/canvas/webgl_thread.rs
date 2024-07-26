/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::{slice, thread};

use bitflags::bitflags;
use byteorder::{ByteOrder, NativeEndian, WriteBytesExt};
use canvas_traits::webgl;
use canvas_traits::webgl::{
    webgl_channel, ActiveAttribInfo, ActiveUniformBlockInfo, ActiveUniformInfo, AlphaTreatment,
    GLContextAttributes, GLLimits, GlType, InternalFormatIntVec, ProgramLinkInfo, TexDataType,
    TexFormat, WebGLBufferId, WebGLChan, WebGLCommand, WebGLCommandBacktrace, WebGLContextId,
    WebGLCreateContextResult, WebGLFramebufferBindingRequest, WebGLFramebufferId, WebGLMsg,
    WebGLMsgSender, WebGLProgramId, WebGLQueryId, WebGLReceiver, WebGLRenderbufferId,
    WebGLSLVersion, WebGLSamplerId, WebGLSender, WebGLShaderId, WebGLSyncId, WebGLTextureId,
    WebGLVersion, WebGLVertexArrayId, WebXRCommand, WebXRLayerManagerId, YAxisTreatment,
};
use euclid::default::Size2D;
use fnv::FnvHashMap;
use half::f16;
use log::{debug, error, trace, warn};
use pixels::{self, PixelFormat};
use sparkle::gl;
use sparkle::gl::{GLint, GLuint, Gl};
use surfman::chains::{PreserveBuffer, SwapChains, SwapChainsAPI};
use surfman::{
    self, Adapter, Connection, Context, ContextAttributeFlags, ContextAttributes, Device,
    GLVersion, SurfaceAccess, SurfaceInfo, SurfaceType,
};
use webrender::{RenderApi, RenderApiSender, Transaction};
use webrender_api::units::DeviceIntSize;
use webrender_api::{
    DirtyRect, DocumentId, ExternalImageData, ExternalImageId, ExternalImageType, ImageBufferKind,
    ImageData, ImageDescriptor, ImageDescriptorFlags, ImageFormat, ImageKey,
};
use webrender_traits::{WebrenderExternalImageRegistry, WebrenderImageHandlerType};
use webxr::SurfmanGL as WebXRSurfman;
use webxr_api::{
    ContextId as WebXRContextId, Error as WebXRError, GLContexts as WebXRContexts,
    GLTypes as WebXRTypes, LayerGrandManager as WebXRLayerGrandManager,
    LayerGrandManagerAPI as WebXRLayerGrandManagerAPI, LayerId as WebXRLayerId,
    LayerInit as WebXRLayerInit, LayerManager as WebXRLayerManager,
    LayerManagerAPI as WebXRLayerManagerAPI, LayerManagerFactory as WebXRLayerManagerFactory,
    SubImages as WebXRSubImages,
};

use crate::webgl_limits::GLLimitsDetect;

struct GLContextData {
    ctx: Context,
    gl: Rc<Gl>,
    state: GLState,
    attributes: GLContextAttributes,
}

#[derive(Debug)]
pub struct GLState {
    webgl_version: WebGLVersion,
    gl_version: GLVersion,
    requested_flags: ContextAttributeFlags,
    // This is the WebGL view of the color mask
    // The GL view may be different: if the GL context supports alpha
    // but the WebGL context doesn't, then color_write_mask.3 might be true
    // but the GL color write mask is false.
    color_write_mask: [bool; 4],
    clear_color: (f32, f32, f32, f32),
    scissor_test_enabled: bool,
    // The WebGL view of the stencil write mask (see comment re `color_write_mask`)
    stencil_write_mask: (u32, u32),
    stencil_test_enabled: bool,
    stencil_clear_value: i32,
    // The WebGL view of the depth write mask (see comment re `color_write_mask`)
    depth_write_mask: bool,
    depth_test_enabled: bool,
    depth_clear_value: f64,
    // True when the default framebuffer is bound to DRAW_FRAMEBUFFER
    drawing_to_default_framebuffer: bool,
    default_vao: gl::GLuint,
}

impl GLState {
    // Are we faking having no alpha / depth / stencil?
    fn fake_no_alpha(&self) -> bool {
        self.drawing_to_default_framebuffer &
            !self.requested_flags.contains(ContextAttributeFlags::ALPHA)
    }

    fn fake_no_depth(&self) -> bool {
        self.drawing_to_default_framebuffer &
            !self.requested_flags.contains(ContextAttributeFlags::DEPTH)
    }

    fn fake_no_stencil(&self) -> bool {
        self.drawing_to_default_framebuffer &
            !self
                .requested_flags
                .contains(ContextAttributeFlags::STENCIL)
    }

    // We maintain invariants between the GLState object and the GL state.
    fn restore_invariant(&self, gl: &Gl) {
        self.restore_clear_color_invariant(gl);
        self.restore_scissor_invariant(gl);
        self.restore_alpha_invariant(gl);
        self.restore_depth_invariant(gl);
        self.restore_stencil_invariant(gl);
    }

    fn restore_clear_color_invariant(&self, gl: &Gl) {
        let (r, g, b, a) = self.clear_color;
        gl.clear_color(r, g, b, a);
    }

    fn restore_scissor_invariant(&self, gl: &Gl) {
        if self.scissor_test_enabled {
            gl.enable(gl::SCISSOR_TEST);
        } else {
            gl.disable(gl::SCISSOR_TEST);
        }
    }

    fn restore_alpha_invariant(&self, gl: &Gl) {
        let [r, g, b, a] = self.color_write_mask;
        if self.fake_no_alpha() {
            gl.color_mask(r, g, b, false);
        } else {
            gl.color_mask(r, g, b, a);
        }
    }

    fn restore_depth_invariant(&self, gl: &Gl) {
        if self.fake_no_depth() {
            gl.depth_mask(false);
            gl.disable(gl::DEPTH_TEST);
        } else {
            gl.depth_mask(self.depth_write_mask);
            if self.depth_test_enabled {
                gl.enable(gl::DEPTH_TEST);
            } else {
                gl.disable(gl::DEPTH_TEST);
            }
        }
    }

    fn restore_stencil_invariant(&self, gl: &Gl) {
        if self.fake_no_stencil() {
            gl.stencil_mask(0);
            gl.disable(gl::STENCIL_TEST);
        } else {
            let (f, b) = self.stencil_write_mask;
            gl.stencil_mask_separate(gl::FRONT, f);
            gl.stencil_mask_separate(gl::BACK, b);
            if self.stencil_test_enabled {
                gl.enable(gl::STENCIL_TEST);
            } else {
                gl.disable(gl::STENCIL_TEST);
            }
        }
    }
}

impl Default for GLState {
    fn default() -> GLState {
        GLState {
            gl_version: GLVersion { major: 1, minor: 0 },
            webgl_version: WebGLVersion::WebGL1,
            requested_flags: ContextAttributeFlags::empty(),
            color_write_mask: [true, true, true, true],
            clear_color: (0., 0., 0., 0.),
            scissor_test_enabled: false,
            // Should these be 0xFFFF_FFFF?
            stencil_write_mask: (0, 0),
            stencil_test_enabled: false,
            stencil_clear_value: 0,
            depth_write_mask: true,
            depth_test_enabled: false,
            depth_clear_value: 1.,
            default_vao: 0,
            drawing_to_default_framebuffer: true,
        }
    }
}

/// A WebGLThread manages the life cycle and message multiplexing of
/// a set of WebGLContexts living in the same thread.
pub(crate) struct WebGLThread {
    /// The GPU device.
    device: Device,
    /// Channel used to generate/update or delete `ImageKey`s.
    webrender_api: RenderApi,
    webrender_doc: DocumentId,
    /// Map of live WebGLContexts.
    contexts: FnvHashMap<WebGLContextId, GLContextData>,
    /// Cached information for WebGLContexts.
    cached_context_info: FnvHashMap<WebGLContextId, WebGLContextInfo>,
    /// Current bound context.
    bound_context_id: Option<WebGLContextId>,
    /// List of registered webrender external images.
    /// We use it to get an unique ID for new WebGLContexts.
    external_images: Arc<Mutex<WebrenderExternalImageRegistry>>,
    /// The receiver that will be used for processing WebGL messages.
    receiver: crossbeam_channel::Receiver<WebGLMsg>,
    /// The receiver that should be used to send WebGL messages for processing.
    sender: WebGLSender<WebGLMsg>,
    /// The swap chains used by webrender
    webrender_swap_chains: SwapChains<WebGLContextId, Device>,
    /// Whether this context is a GL or GLES context.
    api_type: gl::GlType,
    /// The bridge to WebXR
    pub webxr_bridge: WebXRBridge,
}

/// The data required to initialize an instance of the WebGLThread type.
pub(crate) struct WebGLThreadInit {
    pub webrender_api_sender: RenderApiSender,
    pub webrender_doc: DocumentId,
    pub external_images: Arc<Mutex<WebrenderExternalImageRegistry>>,
    pub sender: WebGLSender<WebGLMsg>,
    pub receiver: WebGLReceiver<WebGLMsg>,
    pub webrender_swap_chains: SwapChains<WebGLContextId, Device>,
    pub connection: Connection,
    pub adapter: Adapter,
    pub api_type: gl::GlType,
    pub webxr_init: WebXRBridgeInit,
}

// A size at which it should be safe to create GL contexts
const SAFE_VIEWPORT_DIMS: [u32; 2] = [1024, 1024];

impl WebGLThread {
    /// Create a new instance of WebGLThread.
    pub(crate) fn new(
        WebGLThreadInit {
            webrender_api_sender,
            webrender_doc,
            external_images,
            sender,
            receiver,
            webrender_swap_chains,
            connection,
            adapter,
            api_type,
            webxr_init,
        }: WebGLThreadInit,
    ) -> Self {
        WebGLThread {
            device: connection
                .create_device(&adapter)
                .expect("Couldn't open WebGL device!"),
            webrender_api: webrender_api_sender.create_api(),
            webrender_doc,
            contexts: Default::default(),
            cached_context_info: Default::default(),
            bound_context_id: None,
            external_images,
            sender,
            receiver: receiver.into_inner(),
            webrender_swap_chains,
            api_type,
            webxr_bridge: WebXRBridge::new(webxr_init),
        }
    }

    /// Perform all initialization required to run an instance of WebGLThread
    /// in parallel on its own dedicated thread.
    pub(crate) fn run_on_own_thread(init: WebGLThreadInit) {
        thread::Builder::new()
            .name("WebGL".to_owned())
            .spawn(move || {
                let mut data = WebGLThread::new(init);
                data.process();
            })
            .expect("Thread spawning failed");
    }

    fn process(&mut self) {
        let webgl_chan = WebGLChan(self.sender.clone());
        while let Ok(msg) = self.receiver.recv() {
            let exit = self.handle_msg(msg, &webgl_chan);
            if exit {
                // Call remove_context functions in order to correctly delete WebRender image keys.
                let context_ids: Vec<WebGLContextId> = self.contexts.keys().copied().collect();
                for id in context_ids {
                    self.remove_webgl_context(id);
                }

                // Block on shutting-down WebRender.
                self.webrender_api.shut_down(true);
                return;
            }
        }
    }

    /// Handles a generic WebGLMsg message
    fn handle_msg(&mut self, msg: WebGLMsg, webgl_chan: &WebGLChan) -> bool {
        trace!("processing {:?}", msg);
        match msg {
            WebGLMsg::CreateContext(version, size, attributes, result_sender) => {
                let result = self.create_webgl_context(version, size, attributes);

                result_sender
                    .send(result.map(|(id, limits)| {
                        let image_key = self
                            .cached_context_info
                            .get_mut(&id)
                            .expect("Where's the cached context info?")
                            .image_key;

                        let data = Self::make_current_if_needed(
                            &self.device,
                            id,
                            &self.contexts,
                            &mut self.bound_context_id,
                        )
                        .expect("WebGLContext not found");
                        let glsl_version = Self::get_glsl_version(&data.gl);
                        let api_type = match data.gl.get_type() {
                            gl::GlType::Gl => GlType::Gl,
                            gl::GlType::Gles => GlType::Gles,
                        };

                        // FIXME(nox): Should probably be done by surfman.
                        if api_type != GlType::Gles {
                            // Points sprites are enabled by default in OpenGL 3.2 core
                            // and in GLES. Rather than doing version detection, it does
                            // not hurt to enable them anyways.

                            data.gl.enable(gl::POINT_SPRITE);
                            let err = data.gl.get_error();
                            if err != 0 {
                                warn!("Error enabling GL point sprites: {}", err);
                            }

                            data.gl.enable(gl::PROGRAM_POINT_SIZE);
                            let err = data.gl.get_error();
                            if err != 0 {
                                warn!("Error enabling GL program point size: {}", err);
                            }
                        }

                        WebGLCreateContextResult {
                            sender: WebGLMsgSender::new(id, webgl_chan.clone()),
                            limits,
                            glsl_version,
                            api_type,
                            image_key,
                        }
                    }))
                    .unwrap();
            },
            WebGLMsg::ResizeContext(ctx_id, size, sender) => {
                let _ = sender.send(self.resize_webgl_context(ctx_id, size));
            },
            WebGLMsg::RemoveContext(ctx_id) => {
                self.remove_webgl_context(ctx_id);
            },
            WebGLMsg::WebGLCommand(ctx_id, command, backtrace) => {
                self.handle_webgl_command(ctx_id, command, backtrace);
            },
            WebGLMsg::WebXRCommand(command) => {
                self.handle_webxr_command(command);
            },
            WebGLMsg::SwapBuffers(swap_ids, sender, sent_time) => {
                self.handle_swap_buffers(swap_ids, sender, sent_time);
            },
            WebGLMsg::Exit => {
                return true;
            },
        }

        false
    }

    /// Handles a WebXR message
    fn handle_webxr_command(&mut self, command: WebXRCommand) {
        trace!("processing {:?}", command);
        let mut contexts = WebXRBridgeContexts {
            contexts: &mut self.contexts,
            bound_context_id: &mut self.bound_context_id,
        };
        match command {
            WebXRCommand::CreateLayerManager(sender) => {
                let result = self
                    .webxr_bridge
                    .create_layer_manager(&mut self.device, &mut contexts);
                let _ = sender.send(result);
            },
            WebXRCommand::DestroyLayerManager(manager_id) => {
                self.webxr_bridge.destroy_layer_manager(manager_id);
            },
            WebXRCommand::CreateLayer(manager_id, context_id, layer_init, sender) => {
                let result = self.webxr_bridge.create_layer(
                    manager_id,
                    &mut self.device,
                    &mut contexts,
                    context_id,
                    layer_init,
                );
                let _ = sender.send(result);
            },
            WebXRCommand::DestroyLayer(manager_id, context_id, layer_id) => {
                self.webxr_bridge.destroy_layer(
                    manager_id,
                    &mut self.device,
                    &mut contexts,
                    context_id,
                    layer_id,
                );
            },
            WebXRCommand::BeginFrame(manager_id, layers, sender) => {
                let result = self.webxr_bridge.begin_frame(
                    manager_id,
                    &mut self.device,
                    &mut contexts,
                    &layers[..],
                );
                let _ = sender.send(result);
            },
            WebXRCommand::EndFrame(manager_id, layers, sender) => {
                let result = self.webxr_bridge.end_frame(
                    manager_id,
                    &mut self.device,
                    &mut contexts,
                    &layers[..],
                );
                let _ = sender.send(result);
            },
        }
    }

    /// Handles a WebGLCommand for a specific WebGLContext
    fn handle_webgl_command(
        &mut self,
        context_id: WebGLContextId,
        command: WebGLCommand,
        backtrace: WebGLCommandBacktrace,
    ) {
        if self.cached_context_info.get_mut(&context_id).is_none() {
            return;
        }
        let data = Self::make_current_if_needed_mut(
            &self.device,
            context_id,
            &mut self.contexts,
            &mut self.bound_context_id,
        );
        if let Some(data) = data {
            WebGLImpl::apply(
                &self.device,
                &data.ctx,
                &data.gl,
                &mut data.state,
                &data.attributes,
                command,
                backtrace,
            );
        }
    }

    /// Creates a new WebGLContext
    #[allow(unsafe_code)]
    fn create_webgl_context(
        &mut self,
        webgl_version: WebGLVersion,
        requested_size: Size2D<u32>,
        attributes: GLContextAttributes,
    ) -> Result<(WebGLContextId, webgl::GLLimits), String> {
        debug!(
            "WebGLThread::create_webgl_context({:?}, {:?}, {:?})",
            webgl_version, requested_size, attributes
        );
        #[cfg(target_env = "ohos")]
        return Err("WebGL is not working yet on ohos".into());

        // Creating a new GLContext may make the current bound context_id dirty.
        // Clear it to ensure that  make_current() is called in subsequent commands.
        self.bound_context_id = None;

        let requested_flags =
            attributes.to_surfman_context_attribute_flags(webgl_version, self.api_type);
        // Some GL implementations seem to only allow famebuffers
        // to have alpha, depth and stencil if their creating context does.
        // WebGL requires all contexts to be able to create framebuffers with
        // alpha, depth and stencil. So we always create a context with them,
        // and fake not having them if requested.
        let flags = requested_flags |
            ContextAttributeFlags::ALPHA |
            ContextAttributeFlags::DEPTH |
            ContextAttributeFlags::STENCIL;
        let context_attributes = &ContextAttributes {
            version: webgl_version.to_surfman_version(self.api_type),
            flags,
        };

        let context_descriptor = self
            .device
            .create_context_descriptor(context_attributes)
            .map_err(|err| format!("Failed to create context descriptor: {:?}", err))?;

        let safe_size = Size2D::new(
            requested_size.width.min(SAFE_VIEWPORT_DIMS[0]).max(1),
            requested_size.height.min(SAFE_VIEWPORT_DIMS[1]).max(1),
        );
        let surface_type = SurfaceType::Generic {
            size: safe_size.to_i32(),
        };
        let surface_access = self.surface_access();

        let mut ctx = self
            .device
            .create_context(&context_descriptor, None)
            .map_err(|err| format!("Failed to create the GL context: {:?}", err))?;
        let surface = self
            .device
            .create_surface(&ctx, surface_access, surface_type)
            .map_err(|err| format!("Failed to create the initial surface: {:?}", err))?;
        self.device
            .bind_surface_to_context(&mut ctx, surface)
            .map_err(|err| format!("Failed to bind initial surface: {:?}", err))?;
        // https://github.com/pcwalton/surfman/issues/7
        self.device
            .make_context_current(&ctx)
            .map_err(|err| format!("Failed to make new context current: {:?}", err))?;

        let id = WebGLContextId(
            self.external_images
                .lock()
                .expect("Lock poisoned?")
                .next_id(WebrenderImageHandlerType::WebGL)
                .0,
        );

        self.webrender_swap_chains
            .create_attached_swap_chain(id, &mut self.device, &mut ctx, surface_access)
            .map_err(|err| format!("Failed to create swap chain: {:?}", err))?;

        let swap_chain = self
            .webrender_swap_chains
            .get(id)
            .expect("Failed to get the swap chain");

        debug!(
            "Created webgl context {:?}/{:?}",
            id,
            self.device.context_id(&ctx),
        );

        let gl = match self.api_type {
            gl::GlType::Gl => Gl::gl_fns(gl::ffi_gl::Gl::load_with(|symbol_name| {
                self.device.get_proc_address(&ctx, symbol_name)
            })),
            gl::GlType::Gles => Gl::gles_fns(gl::ffi_gles::Gles2::load_with(|symbol_name| {
                self.device.get_proc_address(&ctx, symbol_name)
            })),
        };

        let limits = GLLimits::detect(&gl, webgl_version);

        let size = clamp_viewport(&gl, requested_size);
        if safe_size != size {
            debug!("Resizing swap chain from {:?} to {:?}", safe_size, size);
            swap_chain
                .resize(&mut self.device, &mut ctx, size.to_i32())
                .map_err(|err| format!("Failed to resize swap chain: {:?}", err))?;
        }

        let descriptor = self.device.context_descriptor(&ctx);
        let descriptor_attributes = self.device.context_descriptor_attributes(&descriptor);
        let gl_version = descriptor_attributes.version;
        let has_alpha = requested_flags.contains(ContextAttributeFlags::ALPHA);
        let image_buffer_kind = current_wr_image_buffer_kind(&self.device);

        self.device.make_context_current(&ctx).unwrap();
        let framebuffer = self
            .device
            .context_surface_info(&ctx)
            .map_err(|err| format!("Failed to get context surface info: {:?}", err))?
            .ok_or_else(|| "Failed to get context surface info".to_string())?
            .framebuffer_object;

        gl.bind_framebuffer(gl::FRAMEBUFFER, framebuffer);
        gl.viewport(0, 0, size.width as i32, size.height as i32);
        gl.scissor(0, 0, size.width as i32, size.height as i32);
        gl.clear_color(0., 0., 0., !has_alpha as u32 as f32);
        gl.clear_depth(1.);
        gl.clear_stencil(0);
        gl.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
        gl.clear_color(0., 0., 0., 0.);
        debug_assert_eq!(gl.get_error(), gl::NO_ERROR);

        let use_apple_vertex_array = WebGLImpl::needs_apple_vertex_arrays(gl_version);
        let default_vao = if let Some(vao) =
            WebGLImpl::create_vertex_array(&gl, use_apple_vertex_array, webgl_version)
        {
            let vao = vao.get();
            WebGLImpl::bind_vertex_array(&gl, vao, use_apple_vertex_array, webgl_version);
            vao
        } else {
            0
        };

        let state = GLState {
            gl_version,
            webgl_version,
            requested_flags,
            default_vao,
            ..Default::default()
        };
        debug!("Created state {:?}", state);

        state.restore_invariant(&gl);
        debug_assert_eq!(gl.get_error(), gl::NO_ERROR);

        self.contexts.insert(
            id,
            GLContextData {
                ctx,
                gl,
                state,
                attributes,
            },
        );

        let image_key = Self::create_wr_external_image(
            &mut self.webrender_api,
            self.webrender_doc,
            size.to_i32(),
            has_alpha,
            id,
            image_buffer_kind,
        );

        self.cached_context_info
            .insert(id, WebGLContextInfo { image_key });

        Ok((id, limits))
    }

    /// Resizes a WebGLContext
    fn resize_webgl_context(
        &mut self,
        context_id: WebGLContextId,
        requested_size: Size2D<u32>,
    ) -> Result<(), String> {
        let data = Self::make_current_if_needed_mut(
            &self.device,
            context_id,
            &mut self.contexts,
            &mut self.bound_context_id,
        )
        .expect("Missing WebGL context!");

        let size = clamp_viewport(&data.gl, requested_size);

        // Check to see if any of the current framebuffer bindings are the surface we're about to
        // throw out. If so, we'll have to reset them after destroying the surface.
        let framebuffer_rebinding_info =
            FramebufferRebindingInfo::detect(&self.device, &data.ctx, &data.gl);

        // Resize the swap chains
        if let Some(swap_chain) = self.webrender_swap_chains.get(context_id) {
            let alpha = data
                .state
                .requested_flags
                .contains(ContextAttributeFlags::ALPHA);
            let clear_color = [0.0, 0.0, 0.0, !alpha as i32 as f32];
            swap_chain
                .resize(&mut self.device, &mut data.ctx, size.to_i32())
                .map_err(|err| format!("Failed to resize swap chain: {:?}", err))?;
            swap_chain
                .clear_surface(&mut self.device, &mut data.ctx, &data.gl, clear_color)
                .map_err(|err| format!("Failed to clear resized swap chain: {:?}", err))?;
        } else {
            error!("Failed to find swap chain");
        }

        // Reset framebuffer bindings as appropriate.
        framebuffer_rebinding_info.apply(&self.device, &data.ctx, &data.gl);
        debug_assert_eq!(data.gl.get_error(), gl::NO_ERROR);

        let has_alpha = data
            .state
            .requested_flags
            .contains(ContextAttributeFlags::ALPHA);
        self.update_wr_image_for_context(context_id, size.to_i32(), has_alpha);

        Ok(())
    }

    /// Removes a WebGLContext and releases attached resources.
    fn remove_webgl_context(&mut self, context_id: WebGLContextId) {
        // Release webrender image keys.
        if let Some(info) = self.cached_context_info.remove(&context_id) {
            let mut txn = Transaction::new();
            txn.delete_image(info.image_key);
            self.webrender_api.send_transaction(self.webrender_doc, txn)
        }

        // We need to make the context current so its resources can be disposed of.
        Self::make_current_if_needed(
            &self.device,
            context_id,
            &self.contexts,
            &mut self.bound_context_id,
        );

        // Destroy WebXR layers associated with this context
        let webxr_context_id = WebXRContextId::from(context_id);
        let mut webxr_contexts = WebXRBridgeContexts {
            contexts: &mut self.contexts,
            bound_context_id: &mut self.bound_context_id,
        };
        self.webxr_bridge.destroy_all_layers(
            &mut self.device,
            &mut webxr_contexts,
            webxr_context_id,
        );

        // Release GL context.
        let mut data = match self.contexts.remove(&context_id) {
            Some(data) => data,
            None => return,
        };

        // Destroy the swap chains
        self.webrender_swap_chains
            .destroy(context_id, &mut self.device, &mut data.ctx)
            .unwrap();

        // Destroy the context
        self.device.destroy_context(&mut data.ctx).unwrap();

        // Removing a GLContext may make the current bound context_id dirty.
        self.bound_context_id = None;
    }

    fn handle_swap_buffers(
        &mut self,
        context_ids: Vec<WebGLContextId>,
        completed_sender: WebGLSender<u64>,
        _sent_time: u64,
    ) {
        debug!("handle_swap_buffers()");
        for context_id in context_ids {
            let data = Self::make_current_if_needed_mut(
                &self.device,
                context_id,
                &mut self.contexts,
                &mut self.bound_context_id,
            )
            .expect("Where's the GL data?");

            // Ensure there are no pending GL errors from other parts of the pipeline.
            debug_assert_eq!(data.gl.get_error(), gl::NO_ERROR);

            // Check to see if any of the current framebuffer bindings are the surface we're about
            // to swap out. If so, we'll have to reset them after destroying the surface.
            let framebuffer_rebinding_info =
                FramebufferRebindingInfo::detect(&self.device, &data.ctx, &data.gl);
            debug_assert_eq!(data.gl.get_error(), gl::NO_ERROR);

            debug!("Getting swap chain for {:?}", context_id);
            let swap_chain = self
                .webrender_swap_chains
                .get(context_id)
                .expect("Where's the swap chain?");

            debug!("Swapping {:?}", context_id);
            swap_chain
                .swap_buffers(
                    &mut self.device,
                    &mut data.ctx,
                    if data.attributes.preserve_drawing_buffer {
                        PreserveBuffer::Yes(&data.gl)
                    } else {
                        PreserveBuffer::No
                    },
                )
                .unwrap();
            debug_assert_eq!(data.gl.get_error(), gl::NO_ERROR);

            if !data.attributes.preserve_drawing_buffer {
                debug!("Clearing {:?}", context_id);
                let alpha = data
                    .state
                    .requested_flags
                    .contains(ContextAttributeFlags::ALPHA);
                let clear_color = [0.0, 0.0, 0.0, !alpha as i32 as f32];
                swap_chain
                    .clear_surface(&mut self.device, &mut data.ctx, &data.gl, clear_color)
                    .unwrap();
                debug_assert_eq!(data.gl.get_error(), gl::NO_ERROR);
            }

            // Rebind framebuffers as appropriate.
            debug!("Rebinding {:?}", context_id);
            framebuffer_rebinding_info.apply(&self.device, &data.ctx, &data.gl);
            debug_assert_eq!(data.gl.get_error(), gl::NO_ERROR);

            let SurfaceInfo {
                size,
                framebuffer_object,
                id,
                ..
            } = self
                .device
                .context_surface_info(&data.ctx)
                .unwrap()
                .unwrap();
            debug!(
                "... rebound framebuffer {}, new back buffer surface is {:?}",
                framebuffer_object, id
            );

            let has_alpha = data
                .state
                .requested_flags
                .contains(ContextAttributeFlags::ALPHA);
            self.update_wr_image_for_context(context_id, size, has_alpha);
        }

        #[allow(unused)]
        let mut end_swap = 0;
        completed_sender.send(end_swap).unwrap();
    }

    /// Which access mode to use
    fn surface_access(&self) -> SurfaceAccess {
        SurfaceAccess::GPUOnly
    }

    /// Gets a reference to a Context for a given WebGLContextId and makes it current if required.
    fn make_current_if_needed<'a>(
        device: &Device,
        context_id: WebGLContextId,
        contexts: &'a FnvHashMap<WebGLContextId, GLContextData>,
        bound_id: &mut Option<WebGLContextId>,
    ) -> Option<&'a GLContextData> {
        let data = contexts.get(&context_id);

        if let Some(data) = data {
            if Some(context_id) != *bound_id {
                device.make_context_current(&data.ctx).unwrap();
                *bound_id = Some(context_id);
            }
        }

        data
    }

    /// Gets a mutable reference to a GLContextWrapper for a WebGLContextId and makes it current if required.
    fn make_current_if_needed_mut<'a>(
        device: &Device,
        context_id: WebGLContextId,
        contexts: &'a mut FnvHashMap<WebGLContextId, GLContextData>,
        bound_id: &mut Option<WebGLContextId>,
    ) -> Option<&'a mut GLContextData> {
        let data = contexts.get_mut(&context_id);

        if let Some(ref data) = data {
            if Some(context_id) != *bound_id {
                device.make_context_current(&data.ctx).unwrap();
                *bound_id = Some(context_id);
            }
        }

        data
    }

    /// Creates a `webrender_api::ImageKey` that uses shared textures.
    fn create_wr_external_image(
        webrender_api: &mut RenderApi,
        webrender_doc: DocumentId,
        size: Size2D<i32>,
        alpha: bool,
        context_id: WebGLContextId,
        image_buffer_kind: ImageBufferKind,
    ) -> ImageKey {
        let descriptor = Self::image_descriptor(size, alpha);
        let data = Self::external_image_data(context_id, image_buffer_kind);

        let image_key = webrender_api.generate_image_key();
        let mut txn = Transaction::new();
        txn.add_image(image_key, descriptor, data, None);
        webrender_api.send_transaction(webrender_doc, txn);

        image_key
    }

    /// Tell WebRender to invalidate any cached tiles for a given `WebGLContextId`
    /// when the underlying surface has changed e.g due to resize or buffer swap
    fn update_wr_image_for_context(
        &mut self,
        context_id: WebGLContextId,
        size: Size2D<i32>,
        has_alpha: bool,
    ) {
        let info = self.cached_context_info.get(&context_id).unwrap();
        let image_buffer_kind = current_wr_image_buffer_kind(&self.device);

        let descriptor = Self::image_descriptor(size, has_alpha);
        let image_data = Self::external_image_data(context_id, image_buffer_kind);

        let mut txn = Transaction::new();
        txn.update_image(info.image_key, descriptor, image_data, &DirtyRect::All);
        self.webrender_api.send_transaction(self.webrender_doc, txn);
    }

    /// Helper function to create a `ImageDescriptor`.
    fn image_descriptor(size: Size2D<i32>, alpha: bool) -> ImageDescriptor {
        let mut flags = ImageDescriptorFlags::empty();
        flags.set(ImageDescriptorFlags::IS_OPAQUE, !alpha);
        ImageDescriptor {
            size: DeviceIntSize::new(size.width, size.height),
            stride: None,
            format: ImageFormat::BGRA8,
            offset: 0,
            flags,
        }
    }

    /// Helper function to create a `ImageData::External` instance.
    fn external_image_data(
        context_id: WebGLContextId,
        image_buffer_kind: ImageBufferKind,
    ) -> ImageData {
        let data = ExternalImageData {
            id: ExternalImageId(context_id.0),
            channel_index: 0,
            image_type: ExternalImageType::TextureHandle(image_buffer_kind),
        };
        ImageData::External(data)
    }

    /// Gets the GLSL Version supported by a GLContext.
    fn get_glsl_version(gl: &Gl) -> WebGLSLVersion {
        let version = gl.get_string(gl::SHADING_LANGUAGE_VERSION);
        // Fomat used by SHADING_LANGUAGE_VERSION query : major.minor[.release] [vendor info]
        let mut values = version.split(&['.', ' '][..]);
        let major = values
            .next()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(1);
        let minor = values
            .next()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(20);

        WebGLSLVersion { major, minor }
    }
}

/// Helper struct to store cached WebGLContext information.
struct WebGLContextInfo {
    /// Currently used WebRender image key.
    image_key: ImageKey,
}

// TODO(pcwalton): Add `GL_TEXTURE_EXTERNAL_OES`?
fn current_wr_image_buffer_kind(device: &Device) -> ImageBufferKind {
    match device.surface_gl_texture_target() {
        gl::TEXTURE_RECTANGLE => ImageBufferKind::TextureRect,
        _ => ImageBufferKind::Texture2D,
    }
}

/// WebGL Commands Implementation
pub struct WebGLImpl;

impl WebGLImpl {
    #[allow(unsafe_code)]
    pub fn apply(
        device: &Device,
        ctx: &Context,
        gl: &Gl,
        state: &mut GLState,
        attributes: &GLContextAttributes,
        command: WebGLCommand,
        _backtrace: WebGLCommandBacktrace,
    ) {
        debug!("WebGLImpl::apply({:?})", command);

        // Ensure there are no pending GL errors from other parts of the pipeline.
        debug_assert_eq!(gl.get_error(), gl::NO_ERROR);

        match command {
            WebGLCommand::GetContextAttributes(ref sender) => sender.send(*attributes).unwrap(),
            WebGLCommand::ActiveTexture(target) => gl.active_texture(target),
            WebGLCommand::AttachShader(program_id, shader_id) => {
                gl.attach_shader(program_id.get(), shader_id.get())
            },
            WebGLCommand::DetachShader(program_id, shader_id) => {
                gl.detach_shader(program_id.get(), shader_id.get())
            },
            WebGLCommand::BindAttribLocation(program_id, index, ref name) => {
                gl.bind_attrib_location(program_id.get(), index, &to_name_in_compiled_shader(name))
            },
            WebGLCommand::BlendColor(r, g, b, a) => gl.blend_color(r, g, b, a),
            WebGLCommand::BlendEquation(mode) => gl.blend_equation(mode),
            WebGLCommand::BlendEquationSeparate(mode_rgb, mode_alpha) => {
                gl.blend_equation_separate(mode_rgb, mode_alpha)
            },
            WebGLCommand::BlendFunc(src, dest) => gl.blend_func(src, dest),
            WebGLCommand::BlendFuncSeparate(src_rgb, dest_rgb, src_alpha, dest_alpha) => {
                gl.blend_func_separate(src_rgb, dest_rgb, src_alpha, dest_alpha)
            },
            WebGLCommand::BufferData(buffer_type, ref receiver, usage) => {
                gl::buffer_data(gl, buffer_type, &receiver.recv().unwrap(), usage)
            },
            WebGLCommand::BufferSubData(buffer_type, offset, ref receiver) => {
                gl::buffer_sub_data(gl, buffer_type, offset, &receiver.recv().unwrap())
            },
            WebGLCommand::CopyBufferSubData(src, dst, src_offset, dst_offset, size) => {
                gl.copy_buffer_sub_data(
                    src,
                    dst,
                    src_offset as isize,
                    dst_offset as isize,
                    size as isize,
                );
            },
            WebGLCommand::GetBufferSubData(buffer_type, offset, length, ref sender) => {
                let ptr = gl.map_buffer_range(
                    buffer_type,
                    offset as isize,
                    length as isize,
                    gl::MAP_READ_BIT,
                );
                let data: &[u8] = unsafe { slice::from_raw_parts(ptr as _, length) };
                sender.send(data).unwrap();
                gl.unmap_buffer(buffer_type);
            },
            WebGLCommand::Clear(mask) => {
                gl.clear(mask);
            },
            WebGLCommand::ClearColor(r, g, b, a) => {
                state.clear_color = (r, g, b, a);
                gl.clear_color(r, g, b, a);
            },
            WebGLCommand::ClearDepth(depth) => {
                let value = depth.max(0.).min(1.) as f64;
                state.depth_clear_value = value;
                gl.clear_depth(value)
            },
            WebGLCommand::ClearStencil(stencil) => {
                state.stencil_clear_value = stencil;
                gl.clear_stencil(stencil);
            },
            WebGLCommand::ColorMask(r, g, b, a) => {
                state.color_write_mask = [r, g, b, a];
                state.restore_alpha_invariant(gl);
            },
            WebGLCommand::CopyTexImage2D(
                target,
                level,
                internal_format,
                x,
                y,
                width,
                height,
                border,
            ) => gl.copy_tex_image_2d(target, level, internal_format, x, y, width, height, border),
            WebGLCommand::CopyTexSubImage2D(
                target,
                level,
                xoffset,
                yoffset,
                x,
                y,
                width,
                height,
            ) => gl.copy_tex_sub_image_2d(target, level, xoffset, yoffset, x, y, width, height),
            WebGLCommand::CullFace(mode) => gl.cull_face(mode),
            WebGLCommand::DepthFunc(func) => gl.depth_func(func),
            WebGLCommand::DepthMask(flag) => {
                state.depth_write_mask = flag;
                state.restore_depth_invariant(gl);
            },
            WebGLCommand::DepthRange(near, far) => {
                gl.depth_range(near.max(0.).min(1.) as f64, far.max(0.).min(1.) as f64)
            },
            WebGLCommand::Disable(cap) => match cap {
                gl::SCISSOR_TEST => {
                    state.scissor_test_enabled = false;
                    state.restore_scissor_invariant(gl);
                },
                gl::DEPTH_TEST => {
                    state.depth_test_enabled = false;
                    state.restore_depth_invariant(gl);
                },
                gl::STENCIL_TEST => {
                    state.stencil_test_enabled = false;
                    state.restore_stencil_invariant(gl);
                },
                _ => gl.disable(cap),
            },
            WebGLCommand::Enable(cap) => match cap {
                gl::SCISSOR_TEST => {
                    state.scissor_test_enabled = true;
                    state.restore_scissor_invariant(gl);
                },
                gl::DEPTH_TEST => {
                    state.depth_test_enabled = true;
                    state.restore_depth_invariant(gl);
                },
                gl::STENCIL_TEST => {
                    state.stencil_test_enabled = true;
                    state.restore_stencil_invariant(gl);
                },
                _ => gl.enable(cap),
            },
            WebGLCommand::FramebufferRenderbuffer(target, attachment, renderbuffertarget, rb) => {
                let attach = |attachment| {
                    gl.framebuffer_renderbuffer(
                        target,
                        attachment,
                        renderbuffertarget,
                        rb.map_or(0, WebGLRenderbufferId::get),
                    )
                };
                if attachment == gl::DEPTH_STENCIL_ATTACHMENT {
                    attach(gl::DEPTH_ATTACHMENT);
                    attach(gl::STENCIL_ATTACHMENT);
                } else {
                    attach(attachment);
                }
            },
            WebGLCommand::FramebufferTexture2D(target, attachment, textarget, texture, level) => {
                let attach = |attachment| {
                    gl.framebuffer_texture_2d(
                        target,
                        attachment,
                        textarget,
                        texture.map_or(0, WebGLTextureId::get),
                        level,
                    )
                };
                if attachment == gl::DEPTH_STENCIL_ATTACHMENT {
                    attach(gl::DEPTH_ATTACHMENT);
                    attach(gl::STENCIL_ATTACHMENT);
                } else {
                    attach(attachment)
                }
            },
            WebGLCommand::FrontFace(mode) => gl.front_face(mode),
            WebGLCommand::DisableVertexAttribArray(attrib_id) => {
                gl.disable_vertex_attrib_array(attrib_id)
            },
            WebGLCommand::EnableVertexAttribArray(attrib_id) => {
                gl.enable_vertex_attrib_array(attrib_id)
            },
            WebGLCommand::Hint(name, val) => gl.hint(name, val),
            WebGLCommand::LineWidth(width) => {
                gl.line_width(width);
                // In OpenGL Core Profile >3.2, any non-1.0 value will generate INVALID_VALUE.
                if width != 1.0 {
                    let _ = gl.get_error();
                }
            },
            WebGLCommand::PixelStorei(name, val) => gl.pixel_store_i(name, val),
            WebGLCommand::PolygonOffset(factor, units) => gl.polygon_offset(factor, units),
            WebGLCommand::ReadPixels(rect, format, pixel_type, ref sender) => {
                let pixels = gl.read_pixels(
                    rect.origin.x as i32,
                    rect.origin.y as i32,
                    rect.size.width as i32,
                    rect.size.height as i32,
                    format,
                    pixel_type,
                );
                sender.send(&pixels).unwrap();
            },
            WebGLCommand::ReadPixelsPP(rect, format, pixel_type, offset) => unsafe {
                gl.read_pixels_into_pixel_pack_buffer(
                    rect.origin.x,
                    rect.origin.y,
                    rect.size.width,
                    rect.size.height,
                    format,
                    pixel_type,
                    offset,
                );
            },
            WebGLCommand::RenderbufferStorage(target, format, width, height) => {
                gl.renderbuffer_storage(target, format, width, height)
            },
            WebGLCommand::RenderbufferStorageMultisample(
                target,
                samples,
                format,
                width,
                height,
            ) => gl.renderbuffer_storage_multisample(target, samples, format, width, height),
            WebGLCommand::SampleCoverage(value, invert) => gl.sample_coverage(value, invert),
            WebGLCommand::Scissor(x, y, width, height) => {
                // FIXME(nox): Kinda unfortunate that some u32 values could
                // end up as negative numbers here, but I don't even think
                // that can happen in the real world.
                gl.scissor(x, y, width as i32, height as i32);
            },
            WebGLCommand::StencilFunc(func, ref_, mask) => gl.stencil_func(func, ref_, mask),
            WebGLCommand::StencilFuncSeparate(face, func, ref_, mask) => {
                gl.stencil_func_separate(face, func, ref_, mask)
            },
            WebGLCommand::StencilMask(mask) => {
                state.stencil_write_mask = (mask, mask);
                state.restore_stencil_invariant(gl);
            },
            WebGLCommand::StencilMaskSeparate(face, mask) => {
                if face == gl::FRONT {
                    state.stencil_write_mask.0 = mask;
                } else {
                    state.stencil_write_mask.1 = mask;
                }
                state.restore_stencil_invariant(gl);
            },
            WebGLCommand::StencilOp(fail, zfail, zpass) => gl.stencil_op(fail, zfail, zpass),
            WebGLCommand::StencilOpSeparate(face, fail, zfail, zpass) => {
                gl.stencil_op_separate(face, fail, zfail, zpass)
            },
            WebGLCommand::GetRenderbufferParameter(target, pname, ref chan) => {
                Self::get_renderbuffer_parameter(gl, target, pname, chan)
            },
            WebGLCommand::CreateTransformFeedback(ref sender) => {
                let value = gl.gen_transform_feedbacks();
                sender.send(value).unwrap()
            },
            WebGLCommand::DeleteTransformFeedback(id) => {
                gl.delete_transform_feedbacks(id);
            },
            WebGLCommand::IsTransformFeedback(id, ref sender) => {
                let value = gl.is_transform_feedback(id);
                sender.send(value).unwrap()
            },
            WebGLCommand::BindTransformFeedback(target, id) => {
                gl.bind_transform_feedback(target, id);
            },
            WebGLCommand::BeginTransformFeedback(mode) => {
                gl.begin_transform_feedback(mode);
            },
            WebGLCommand::EndTransformFeedback() => {
                gl.end_transform_feedback();
            },
            WebGLCommand::PauseTransformFeedback() => {
                gl.pause_transform_feedback();
            },
            WebGLCommand::ResumeTransformFeedback() => {
                gl.resume_transform_feedback();
            },
            WebGLCommand::GetTransformFeedbackVarying(program, index, ref sender) => {
                let (size, ty, mut name) = gl.get_transform_feedback_varying(program.get(), index);
                // We need to split, because the name starts with '_u' prefix.
                name = name.split_off(2);
                sender.send((size, ty, name)).unwrap();
            },
            WebGLCommand::TransformFeedbackVaryings(program, ref varyings, buffer_mode) => {
                gl.transform_feedback_varyings(program.get(), varyings.as_slice(), buffer_mode);
            },
            WebGLCommand::GetFramebufferAttachmentParameter(
                target,
                attachment,
                pname,
                ref chan,
            ) => Self::get_framebuffer_attachment_parameter(gl, target, attachment, pname, chan),
            WebGLCommand::GetShaderPrecisionFormat(shader_type, precision_type, ref chan) => {
                Self::shader_precision_format(gl, shader_type, precision_type, chan)
            },
            WebGLCommand::GetExtensions(ref chan) => Self::get_extensions(gl, chan),
            WebGLCommand::GetFragDataLocation(program_id, ref name, ref sender) => {
                let location =
                    gl.get_frag_data_location(program_id.get(), &to_name_in_compiled_shader(name));
                sender.send(location).unwrap();
            },
            WebGLCommand::GetUniformLocation(program_id, ref name, ref chan) => {
                Self::uniform_location(gl, program_id, name, chan)
            },
            WebGLCommand::GetShaderInfoLog(shader_id, ref chan) => {
                Self::shader_info_log(gl, shader_id, chan)
            },
            WebGLCommand::GetProgramInfoLog(program_id, ref chan) => {
                Self::program_info_log(gl, program_id, chan)
            },
            WebGLCommand::CompileShader(shader_id, ref source) => {
                Self::compile_shader(gl, shader_id, source)
            },
            WebGLCommand::CreateBuffer(ref chan) => Self::create_buffer(gl, chan),
            WebGLCommand::CreateFramebuffer(ref chan) => Self::create_framebuffer(gl, chan),
            WebGLCommand::CreateRenderbuffer(ref chan) => Self::create_renderbuffer(gl, chan),
            WebGLCommand::CreateTexture(ref chan) => Self::create_texture(gl, chan),
            WebGLCommand::CreateProgram(ref chan) => Self::create_program(gl, chan),
            WebGLCommand::CreateShader(shader_type, ref chan) => {
                Self::create_shader(gl, shader_type, chan)
            },
            WebGLCommand::DeleteBuffer(id) => gl.delete_buffers(&[id.get()]),
            WebGLCommand::DeleteFramebuffer(id) => gl.delete_framebuffers(&[id.get()]),
            WebGLCommand::DeleteRenderbuffer(id) => gl.delete_renderbuffers(&[id.get()]),
            WebGLCommand::DeleteTexture(id) => gl.delete_textures(&[id.get()]),
            WebGLCommand::DeleteProgram(id) => gl.delete_program(id.get()),
            WebGLCommand::DeleteShader(id) => gl.delete_shader(id.get()),
            WebGLCommand::BindBuffer(target, id) => {
                gl.bind_buffer(target, id.map_or(0, WebGLBufferId::get))
            },
            WebGLCommand::BindFramebuffer(target, request) => {
                Self::bind_framebuffer(gl, target, request, ctx, device, state)
            },
            WebGLCommand::BindRenderbuffer(target, id) => {
                gl.bind_renderbuffer(target, id.map_or(0, WebGLRenderbufferId::get))
            },
            WebGLCommand::BindTexture(target, id) => {
                gl.bind_texture(target, id.map_or(0, WebGLTextureId::get))
            },
            WebGLCommand::Uniform1f(uniform_id, v) => gl.uniform_1f(uniform_id, v),
            WebGLCommand::Uniform1fv(uniform_id, ref v) => gl.uniform_1fv(uniform_id, v),
            WebGLCommand::Uniform1i(uniform_id, v) => gl.uniform_1i(uniform_id, v),
            WebGLCommand::Uniform1iv(uniform_id, ref v) => gl.uniform_1iv(uniform_id, v),
            WebGLCommand::Uniform1ui(uniform_id, v) => gl.uniform_1ui(uniform_id, v),
            WebGLCommand::Uniform1uiv(uniform_id, ref v) => gl.uniform_1uiv(uniform_id, v),
            WebGLCommand::Uniform2f(uniform_id, x, y) => gl.uniform_2f(uniform_id, x, y),
            WebGLCommand::Uniform2fv(uniform_id, ref v) => gl.uniform_2fv(uniform_id, v),
            WebGLCommand::Uniform2i(uniform_id, x, y) => gl.uniform_2i(uniform_id, x, y),
            WebGLCommand::Uniform2iv(uniform_id, ref v) => gl.uniform_2iv(uniform_id, v),
            WebGLCommand::Uniform2ui(uniform_id, x, y) => gl.uniform_2ui(uniform_id, x, y),
            WebGLCommand::Uniform2uiv(uniform_id, ref v) => gl.uniform_2uiv(uniform_id, v),
            WebGLCommand::Uniform3f(uniform_id, x, y, z) => gl.uniform_3f(uniform_id, x, y, z),
            WebGLCommand::Uniform3fv(uniform_id, ref v) => gl.uniform_3fv(uniform_id, v),
            WebGLCommand::Uniform3i(uniform_id, x, y, z) => gl.uniform_3i(uniform_id, x, y, z),
            WebGLCommand::Uniform3iv(uniform_id, ref v) => gl.uniform_3iv(uniform_id, v),
            WebGLCommand::Uniform3ui(uniform_id, x, y, z) => gl.uniform_3ui(uniform_id, x, y, z),
            WebGLCommand::Uniform3uiv(uniform_id, ref v) => gl.uniform_3uiv(uniform_id, v),
            WebGLCommand::Uniform4f(uniform_id, x, y, z, w) => {
                gl.uniform_4f(uniform_id, x, y, z, w)
            },
            WebGLCommand::Uniform4fv(uniform_id, ref v) => gl.uniform_4fv(uniform_id, v),
            WebGLCommand::Uniform4i(uniform_id, x, y, z, w) => {
                gl.uniform_4i(uniform_id, x, y, z, w)
            },
            WebGLCommand::Uniform4iv(uniform_id, ref v) => gl.uniform_4iv(uniform_id, v),
            WebGLCommand::Uniform4ui(uniform_id, x, y, z, w) => {
                gl.uniform_4ui(uniform_id, x, y, z, w)
            },
            WebGLCommand::Uniform4uiv(uniform_id, ref v) => gl.uniform_4uiv(uniform_id, v),
            WebGLCommand::UniformMatrix2fv(uniform_id, ref v) => {
                gl.uniform_matrix_2fv(uniform_id, false, v)
            },
            WebGLCommand::UniformMatrix3fv(uniform_id, ref v) => {
                gl.uniform_matrix_3fv(uniform_id, false, v)
            },
            WebGLCommand::UniformMatrix4fv(uniform_id, ref v) => {
                gl.uniform_matrix_4fv(uniform_id, false, v)
            },
            WebGLCommand::UniformMatrix3x2fv(uniform_id, ref v) => {
                gl.uniform_matrix_3x2fv(uniform_id, false, v)
            },
            WebGLCommand::UniformMatrix4x2fv(uniform_id, ref v) => {
                gl.uniform_matrix_4x2fv(uniform_id, false, v)
            },
            WebGLCommand::UniformMatrix2x3fv(uniform_id, ref v) => {
                gl.uniform_matrix_2x3fv(uniform_id, false, v)
            },
            WebGLCommand::UniformMatrix4x3fv(uniform_id, ref v) => {
                gl.uniform_matrix_4x3fv(uniform_id, false, v)
            },
            WebGLCommand::UniformMatrix2x4fv(uniform_id, ref v) => {
                gl.uniform_matrix_2x4fv(uniform_id, false, v)
            },
            WebGLCommand::UniformMatrix3x4fv(uniform_id, ref v) => {
                gl.uniform_matrix_3x4fv(uniform_id, false, v)
            },
            WebGLCommand::ValidateProgram(program_id) => gl.validate_program(program_id.get()),
            WebGLCommand::VertexAttrib(attrib_id, x, y, z, w) => {
                gl.vertex_attrib_4f(attrib_id, x, y, z, w)
            },
            WebGLCommand::VertexAttribI(attrib_id, x, y, z, w) => {
                gl.vertex_attrib_4i(attrib_id, x, y, z, w)
            },
            WebGLCommand::VertexAttribU(attrib_id, x, y, z, w) => {
                gl.vertex_attrib_4ui(attrib_id, x, y, z, w)
            },
            WebGLCommand::VertexAttribPointer2f(attrib_id, size, normalized, stride, offset) => {
                gl.vertex_attrib_pointer_f32(attrib_id, size, normalized, stride, offset)
            },
            WebGLCommand::VertexAttribPointer(
                attrib_id,
                size,
                data_type,
                normalized,
                stride,
                offset,
            ) => gl.vertex_attrib_pointer(attrib_id, size, data_type, normalized, stride, offset),
            WebGLCommand::SetViewport(x, y, width, height) => gl.viewport(x, y, width, height),
            WebGLCommand::TexImage2D {
                target,
                level,
                internal_format,
                size,
                format,
                data_type,
                effective_data_type,
                unpacking_alignment,
                alpha_treatment,
                y_axis_treatment,
                pixel_format,
                ref data,
            } => {
                let pixels = prepare_pixels(
                    internal_format,
                    data_type,
                    size,
                    unpacking_alignment,
                    alpha_treatment,
                    y_axis_treatment,
                    pixel_format,
                    Cow::Borrowed(data),
                );

                gl.pixel_store_i(gl::UNPACK_ALIGNMENT, unpacking_alignment as i32);
                gl.tex_image_2d(
                    target,
                    level as i32,
                    internal_format.as_gl_constant() as i32,
                    size.width as i32,
                    size.height as i32,
                    0,
                    format.as_gl_constant(),
                    effective_data_type,
                    gl::TexImageSource::Pixels(Some(&pixels)),
                );
            },
            WebGLCommand::TexImage2DPBO {
                target,
                level,
                internal_format,
                size,
                format,
                effective_data_type,
                unpacking_alignment,
                offset,
            } => {
                gl.pixel_store_i(gl::UNPACK_ALIGNMENT, unpacking_alignment as i32);

                gl.tex_image_2d(
                    target,
                    level as i32,
                    internal_format.as_gl_constant() as i32,
                    size.width as i32,
                    size.height as i32,
                    0,
                    format.as_gl_constant(),
                    effective_data_type,
                    gl::TexImageSource::BufferOffset(offset),
                );
            },
            WebGLCommand::TexSubImage2D {
                target,
                level,
                xoffset,
                yoffset,
                size,
                format,
                data_type,
                effective_data_type,
                unpacking_alignment,
                alpha_treatment,
                y_axis_treatment,
                pixel_format,
                ref data,
            } => {
                let pixels = prepare_pixels(
                    format,
                    data_type,
                    size,
                    unpacking_alignment,
                    alpha_treatment,
                    y_axis_treatment,
                    pixel_format,
                    Cow::Borrowed(data),
                );

                gl.pixel_store_i(gl::UNPACK_ALIGNMENT, unpacking_alignment as i32);
                gl.tex_sub_image_2d(
                    target,
                    level as i32,
                    xoffset,
                    yoffset,
                    size.width as i32,
                    size.height as i32,
                    format.as_gl_constant(),
                    effective_data_type,
                    &pixels,
                );
            },
            WebGLCommand::CompressedTexImage2D {
                target,
                level,
                internal_format,
                size,
                ref data,
            } => {
                gl.compressed_tex_image_2d(
                    target,
                    level as i32,
                    internal_format,
                    size.width as i32,
                    size.height as i32,
                    0,
                    data,
                );
            },
            WebGLCommand::CompressedTexSubImage2D {
                target,
                level,
                xoffset,
                yoffset,
                size,
                format,
                ref data,
            } => {
                gl.compressed_tex_sub_image_2d(
                    target,
                    level,
                    xoffset,
                    yoffset,
                    size.width as i32,
                    size.height as i32,
                    format,
                    data,
                );
            },
            WebGLCommand::TexStorage2D(target, levels, internal_format, width, height) => gl
                .tex_storage_2d(
                    target,
                    levels as i32,
                    internal_format.as_gl_constant(),
                    width as i32,
                    height as i32,
                ),
            WebGLCommand::TexStorage3D(target, levels, internal_format, width, height, depth) => gl
                .tex_storage_3d(
                    target,
                    levels as i32,
                    internal_format.as_gl_constant(),
                    width as i32,
                    height as i32,
                    depth as i32,
                ),
            WebGLCommand::DrawingBufferWidth(ref sender) => {
                let size = device
                    .context_surface_info(ctx)
                    .unwrap()
                    .expect("Where's the front buffer?")
                    .size;
                sender.send(size.width).unwrap()
            },
            WebGLCommand::DrawingBufferHeight(ref sender) => {
                let size = device
                    .context_surface_info(ctx)
                    .unwrap()
                    .expect("Where's the front buffer?")
                    .size;
                sender.send(size.height).unwrap()
            },
            WebGLCommand::Finish(ref sender) => Self::finish(gl, sender),
            WebGLCommand::Flush => gl.flush(),
            WebGLCommand::GenerateMipmap(target) => gl.generate_mipmap(target),
            WebGLCommand::CreateVertexArray(ref chan) => {
                let use_apple_vertex_array = Self::needs_apple_vertex_arrays(state.gl_version);
                let id = Self::create_vertex_array(gl, use_apple_vertex_array, state.webgl_version);
                let _ = chan.send(id);
            },
            WebGLCommand::DeleteVertexArray(id) => {
                let use_apple_vertex_array = Self::needs_apple_vertex_arrays(state.gl_version);
                let id = id.get();
                Self::delete_vertex_array(gl, id, use_apple_vertex_array, state.webgl_version);
            },
            WebGLCommand::BindVertexArray(id) => {
                let id = id.map_or(state.default_vao, WebGLVertexArrayId::get);
                let use_apple_vertex_array = Self::needs_apple_vertex_arrays(state.gl_version);
                Self::bind_vertex_array(gl, id, use_apple_vertex_array, state.webgl_version);
            },
            WebGLCommand::GetParameterBool(param, ref sender) => {
                let value = match param {
                    webgl::ParameterBool::DepthWritemask => state.depth_write_mask,
                    _ => unsafe {
                        let mut value = [0];
                        gl.get_boolean_v(param as u32, &mut value);
                        value[0] != 0
                    },
                };
                sender.send(value).unwrap()
            },
            WebGLCommand::FenceSync(ref sender) => {
                let value = gl.fence_sync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0);
                sender
                    .send(unsafe { WebGLSyncId::new(value as u64) })
                    .unwrap();
            },
            WebGLCommand::IsSync(sync_id, ref sender) => {
                let value = gl.is_sync(sync_id.get() as *const _);
                sender.send(value).unwrap();
            },
            WebGLCommand::ClientWaitSync(sync_id, flags, timeout, ref sender) => {
                let value = gl.client_wait_sync(sync_id.get() as *const _, flags, timeout);
                sender.send(value).unwrap();
            },
            WebGLCommand::WaitSync(sync_id, flags, timeout) => {
                gl.wait_sync(sync_id.get() as *const _, flags, timeout as u64);
            },
            WebGLCommand::GetSyncParameter(sync_id, param, ref sender) => {
                let value = gl.get_sync_iv(sync_id.get() as *const _, param);
                sender.send(value[0] as u32).unwrap();
            },
            WebGLCommand::DeleteSync(sync_id) => {
                gl.delete_sync(sync_id.get() as *const _);
            },
            WebGLCommand::GetParameterBool4(param, ref sender) => {
                let value = match param {
                    webgl::ParameterBool4::ColorWritemask => state.color_write_mask,
                };
                sender.send(value).unwrap()
            },
            WebGLCommand::GetParameterInt(param, ref sender) => {
                let value = match param {
                    webgl::ParameterInt::AlphaBits if state.fake_no_alpha() => 0,
                    webgl::ParameterInt::DepthBits if state.fake_no_depth() => 0,
                    webgl::ParameterInt::StencilBits if state.fake_no_stencil() => 0,
                    webgl::ParameterInt::StencilWritemask => state.stencil_write_mask.0 as i32,
                    webgl::ParameterInt::StencilBackWritemask => state.stencil_write_mask.1 as i32,
                    _ => unsafe {
                        let mut value = [0];
                        gl.get_integer_v(param as u32, &mut value);
                        value[0]
                    },
                };
                sender.send(value).unwrap()
            },
            WebGLCommand::GetParameterInt2(param, ref sender) => {
                let mut value = [0; 2];
                unsafe {
                    gl.get_integer_v(param as u32, &mut value);
                }
                sender.send(value).unwrap()
            },
            WebGLCommand::GetParameterInt4(param, ref sender) => {
                let mut value = [0; 4];
                unsafe {
                    gl.get_integer_v(param as u32, &mut value);
                }
                sender.send(value).unwrap()
            },
            WebGLCommand::GetParameterFloat(param, ref sender) => {
                let mut value = [0.];
                unsafe {
                    gl.get_float_v(param as u32, &mut value);
                }
                sender.send(value[0]).unwrap()
            },
            WebGLCommand::GetParameterFloat2(param, ref sender) => {
                let mut value = [0.; 2];
                unsafe {
                    gl.get_float_v(param as u32, &mut value);
                }
                sender.send(value).unwrap()
            },
            WebGLCommand::GetParameterFloat4(param, ref sender) => {
                let mut value = [0.; 4];
                unsafe {
                    gl.get_float_v(param as u32, &mut value);
                }
                sender.send(value).unwrap()
            },
            WebGLCommand::GetProgramValidateStatus(program, ref sender) => {
                let mut value = [0];
                unsafe {
                    gl.get_program_iv(program.get(), gl::VALIDATE_STATUS, &mut value);
                }
                sender.send(value[0] != 0).unwrap()
            },
            WebGLCommand::GetProgramActiveUniforms(program, ref sender) => {
                let mut value = [0];
                unsafe {
                    gl.get_program_iv(program.get(), gl::ACTIVE_UNIFORMS, &mut value);
                }
                sender.send(value[0]).unwrap()
            },
            WebGLCommand::GetCurrentVertexAttrib(index, ref sender) => {
                let mut value = [0.; 4];
                unsafe {
                    gl.get_vertex_attrib_fv(index, gl::CURRENT_VERTEX_ATTRIB, &mut value);
                }
                sender.send(value).unwrap();
            },
            WebGLCommand::GetTexParameterFloat(target, param, ref sender) => {
                sender
                    .send(gl.get_tex_parameter_fv(target, param as u32))
                    .unwrap();
            },
            WebGLCommand::GetTexParameterInt(target, param, ref sender) => {
                sender
                    .send(gl.get_tex_parameter_iv(target, param as u32))
                    .unwrap();
            },
            WebGLCommand::GetTexParameterBool(target, param, ref sender) => {
                sender
                    .send(gl.get_tex_parameter_iv(target, param as u32) != 0)
                    .unwrap();
            },
            WebGLCommand::GetInternalFormatIntVec(target, internal_format, param, ref sender) => {
                match param {
                    InternalFormatIntVec::Samples => {
                        let mut count = [0; 1];
                        gl.get_internal_format_iv(
                            target,
                            internal_format,
                            gl::NUM_SAMPLE_COUNTS,
                            &mut count,
                        );
                        assert!(count[0] >= 0);

                        let mut values = vec![0; count[0] as usize];
                        gl.get_internal_format_iv(
                            target,
                            internal_format,
                            param as u32,
                            &mut values,
                        );
                        sender.send(values).unwrap()
                    },
                }
            },
            WebGLCommand::TexParameteri(target, param, value) => {
                gl.tex_parameter_i(target, param, value)
            },
            WebGLCommand::TexParameterf(target, param, value) => {
                gl.tex_parameter_f(target, param, value)
            },
            WebGLCommand::LinkProgram(program_id, ref sender) => {
                return sender.send(Self::link_program(gl, program_id)).unwrap();
            },
            WebGLCommand::UseProgram(program_id) => {
                gl.use_program(program_id.map_or(0, |p| p.get()))
            },
            WebGLCommand::DrawArrays { mode, first, count } => gl.draw_arrays(mode, first, count),
            WebGLCommand::DrawArraysInstanced {
                mode,
                first,
                count,
                primcount,
            } => gl.draw_arrays_instanced(mode, first, count, primcount),
            WebGLCommand::DrawElements {
                mode,
                count,
                type_,
                offset,
            } => gl.draw_elements(mode, count, type_, offset),
            WebGLCommand::DrawElementsInstanced {
                mode,
                count,
                type_,
                offset,
                primcount,
            } => gl.draw_elements_instanced(mode, count, type_, offset, primcount),
            WebGLCommand::VertexAttribDivisor { index, divisor } => {
                gl.vertex_attrib_divisor(index, divisor)
            },
            WebGLCommand::GetUniformBool(program_id, loc, ref sender) => {
                let mut value = [0];
                unsafe {
                    gl.get_uniform_iv(program_id.get(), loc, &mut value);
                }
                sender.send(value[0] != 0).unwrap();
            },
            WebGLCommand::GetUniformBool2(program_id, loc, ref sender) => {
                let mut value = [0; 2];
                unsafe {
                    gl.get_uniform_iv(program_id.get(), loc, &mut value);
                }
                let value = [value[0] != 0, value[1] != 0];
                sender.send(value).unwrap();
            },
            WebGLCommand::GetUniformBool3(program_id, loc, ref sender) => {
                let mut value = [0; 3];
                unsafe {
                    gl.get_uniform_iv(program_id.get(), loc, &mut value);
                }
                let value = [value[0] != 0, value[1] != 0, value[2] != 0];
                sender.send(value).unwrap();
            },
            WebGLCommand::GetUniformBool4(program_id, loc, ref sender) => {
                let mut value = [0; 4];
                unsafe {
                    gl.get_uniform_iv(program_id.get(), loc, &mut value);
                }
                let value = [value[0] != 0, value[1] != 0, value[2] != 0, value[3] != 0];
                sender.send(value).unwrap();
            },
            WebGLCommand::GetUniformInt(program_id, loc, ref sender) => {
                let mut value = [0];
                unsafe {
                    gl.get_uniform_iv(program_id.get(), loc, &mut value);
                }
                sender.send(value[0]).unwrap();
            },
            WebGLCommand::GetUniformInt2(program_id, loc, ref sender) => {
                let mut value = [0; 2];
                unsafe {
                    gl.get_uniform_iv(program_id.get(), loc, &mut value);
                }
                sender.send(value).unwrap();
            },
            WebGLCommand::GetUniformInt3(program_id, loc, ref sender) => {
                let mut value = [0; 3];
                unsafe {
                    gl.get_uniform_iv(program_id.get(), loc, &mut value);
                }
                sender.send(value).unwrap();
            },
            WebGLCommand::GetUniformInt4(program_id, loc, ref sender) => {
                let mut value = [0; 4];
                unsafe {
                    gl.get_uniform_iv(program_id.get(), loc, &mut value);
                }
                sender.send(value).unwrap();
            },
            WebGLCommand::GetUniformUint(program_id, loc, ref sender) => {
                let mut value = [0];
                unsafe {
                    gl.get_uniform_uiv(program_id.get(), loc, &mut value);
                }
                sender.send(value[0]).unwrap();
            },
            WebGLCommand::GetUniformUint2(program_id, loc, ref sender) => {
                let mut value = [0; 2];
                unsafe {
                    gl.get_uniform_uiv(program_id.get(), loc, &mut value);
                }
                sender.send(value).unwrap();
            },
            WebGLCommand::GetUniformUint3(program_id, loc, ref sender) => {
                let mut value = [0; 3];
                unsafe {
                    gl.get_uniform_uiv(program_id.get(), loc, &mut value);
                }
                sender.send(value).unwrap();
            },
            WebGLCommand::GetUniformUint4(program_id, loc, ref sender) => {
                let mut value = [0; 4];
                unsafe {
                    gl.get_uniform_uiv(program_id.get(), loc, &mut value);
                }
                sender.send(value).unwrap();
            },
            WebGLCommand::GetUniformFloat(program_id, loc, ref sender) => {
                let mut value = [0.];
                unsafe {
                    gl.get_uniform_fv(program_id.get(), loc, &mut value);
                }
                sender.send(value[0]).unwrap();
            },
            WebGLCommand::GetUniformFloat2(program_id, loc, ref sender) => {
                let mut value = [0.; 2];
                unsafe {
                    gl.get_uniform_fv(program_id.get(), loc, &mut value);
                }
                sender.send(value).unwrap();
            },
            WebGLCommand::GetUniformFloat3(program_id, loc, ref sender) => {
                let mut value = [0.; 3];
                unsafe {
                    gl.get_uniform_fv(program_id.get(), loc, &mut value);
                }
                sender.send(value).unwrap();
            },
            WebGLCommand::GetUniformFloat4(program_id, loc, ref sender) => {
                let mut value = [0.; 4];
                unsafe {
                    gl.get_uniform_fv(program_id.get(), loc, &mut value);
                }
                sender.send(value).unwrap();
            },
            WebGLCommand::GetUniformFloat9(program_id, loc, ref sender) => {
                let mut value = [0.; 9];
                unsafe {
                    gl.get_uniform_fv(program_id.get(), loc, &mut value);
                }
                sender.send(value).unwrap();
            },
            WebGLCommand::GetUniformFloat16(program_id, loc, ref sender) => {
                let mut value = [0.; 16];
                unsafe {
                    gl.get_uniform_fv(program_id.get(), loc, &mut value);
                }
                sender.send(value).unwrap();
            },
            WebGLCommand::GetUniformFloat2x3(program_id, loc, ref sender) => {
                let mut value = [0.; 2 * 3];
                unsafe {
                    gl.get_uniform_fv(program_id.get(), loc, &mut value);
                }
                sender.send(value).unwrap()
            },
            WebGLCommand::GetUniformFloat2x4(program_id, loc, ref sender) => {
                let mut value = [0.; 2 * 4];
                unsafe {
                    gl.get_uniform_fv(program_id.get(), loc, &mut value);
                }
                sender.send(value).unwrap()
            },
            WebGLCommand::GetUniformFloat3x2(program_id, loc, ref sender) => {
                let mut value = [0.; 3 * 2];
                unsafe {
                    gl.get_uniform_fv(program_id.get(), loc, &mut value);
                }
                sender.send(value).unwrap()
            },
            WebGLCommand::GetUniformFloat3x4(program_id, loc, ref sender) => {
                let mut value = [0.; 3 * 4];
                unsafe {
                    gl.get_uniform_fv(program_id.get(), loc, &mut value);
                }
                sender.send(value).unwrap()
            },
            WebGLCommand::GetUniformFloat4x2(program_id, loc, ref sender) => {
                let mut value = [0.; 4 * 2];
                unsafe {
                    gl.get_uniform_fv(program_id.get(), loc, &mut value);
                }
                sender.send(value).unwrap()
            },
            WebGLCommand::GetUniformFloat4x3(program_id, loc, ref sender) => {
                let mut value = [0.; 4 * 3];
                unsafe {
                    gl.get_uniform_fv(program_id.get(), loc, &mut value);
                }
                sender.send(value).unwrap()
            },
            WebGLCommand::GetUniformBlockIndex(program_id, ref name, ref sender) => {
                let name = to_name_in_compiled_shader(name);
                let index = gl.get_uniform_block_index(program_id.get(), &name);
                sender.send(index).unwrap();
            },
            WebGLCommand::GetUniformIndices(program_id, ref names, ref sender) => {
                let names = names
                    .iter()
                    .map(|name| to_name_in_compiled_shader(name))
                    .collect::<Vec<_>>();
                let name_strs = names.iter().map(|name| name.as_str()).collect::<Vec<_>>();
                let indices = gl.get_uniform_indices(program_id.get(), &name_strs);
                sender.send(indices).unwrap();
            },
            WebGLCommand::GetActiveUniforms(program_id, ref indices, pname, ref sender) => {
                let results = gl.get_active_uniforms_iv(program_id.get(), indices, pname);
                sender.send(results).unwrap();
            },
            WebGLCommand::GetActiveUniformBlockName(program_id, block_idx, ref sender) => {
                let name = gl.get_active_uniform_block_name(program_id.get(), block_idx);
                sender.send(name).unwrap();
            },
            WebGLCommand::GetActiveUniformBlockParameter(
                program_id,
                block_idx,
                pname,
                ref sender,
            ) => {
                let results = gl.get_active_uniform_block_iv(program_id.get(), block_idx, pname);
                sender.send(results).unwrap();
            },
            WebGLCommand::UniformBlockBinding(program_id, block_idx, block_binding) => {
                gl.uniform_block_binding(program_id.get(), block_idx, block_binding)
            },
            WebGLCommand::InitializeFramebuffer {
                color,
                depth,
                stencil,
            } => Self::initialize_framebuffer(gl, state, color, depth, stencil),
            WebGLCommand::BeginQuery(target, query_id) => {
                gl.begin_query(target, query_id.get());
            },
            WebGLCommand::EndQuery(target) => {
                gl.end_query(target);
            },
            WebGLCommand::DeleteQuery(query_id) => {
                gl.delete_queries(&[query_id.get()]);
            },
            WebGLCommand::GenerateQuery(ref sender) => {
                let id = gl.gen_queries(1)[0];
                sender.send(unsafe { WebGLQueryId::new(id) }).unwrap()
            },
            WebGLCommand::GetQueryState(ref sender, query_id, pname) => {
                let value = gl.get_query_object_uiv(query_id.get(), pname);
                sender.send(value).unwrap()
            },
            WebGLCommand::GenerateSampler(ref sender) => {
                let id = gl.gen_samplers(1)[0];
                sender.send(unsafe { WebGLSamplerId::new(id) }).unwrap()
            },
            WebGLCommand::DeleteSampler(sampler_id) => {
                gl.delete_samplers(&[sampler_id.get()]);
            },
            WebGLCommand::BindSampler(unit, sampler_id) => {
                gl.bind_sampler(unit, sampler_id.get());
            },
            WebGLCommand::SetSamplerParameterInt(sampler_id, pname, value) => {
                gl.sampler_parameter_i(sampler_id.get(), pname, value);
            },
            WebGLCommand::SetSamplerParameterFloat(sampler_id, pname, value) => {
                gl.sampler_parameter_f(sampler_id.get(), pname, value);
            },
            WebGLCommand::GetSamplerParameterInt(sampler_id, pname, ref sender) => {
                let value = gl.get_sampler_parameter_iv(sampler_id.get(), pname)[0];
                sender.send(value).unwrap();
            },
            WebGLCommand::GetSamplerParameterFloat(sampler_id, pname, ref sender) => {
                let value = gl.get_sampler_parameter_fv(sampler_id.get(), pname)[0];
                sender.send(value).unwrap();
            },
            WebGLCommand::BindBufferBase(target, index, id) => {
                // https://searchfox.org/mozilla-central/rev/13b081a62d3f3e3e3120f95564529257b0bf451c/dom/canvas/WebGLContextBuffers.cpp#208-210
                // BindBufferBase/Range will fail (on some drivers) if the buffer name has
                // never been bound. (GenBuffers makes a name, but BindBuffer initializes
                // that name as a real buffer object)
                let id = id.map_or(0, WebGLBufferId::get);
                gl.bind_buffer(target, id);
                gl.bind_buffer(target, 0);
                gl.bind_buffer_base(target, index, id);
            },
            WebGLCommand::BindBufferRange(target, index, id, offset, size) => {
                // https://searchfox.org/mozilla-central/rev/13b081a62d3f3e3e3120f95564529257b0bf451c/dom/canvas/WebGLContextBuffers.cpp#208-210
                // BindBufferBase/Range will fail (on some drivers) if the buffer name has
                // never been bound. (GenBuffers makes a name, but BindBuffer initializes
                // that name as a real buffer object)
                let id = id.map_or(0, WebGLBufferId::get);
                gl.bind_buffer(target, id);
                gl.bind_buffer(target, 0);
                gl.bind_buffer_range(target, index, id, offset as isize, size as isize);
            },
            WebGLCommand::ClearBufferfv(buffer, draw_buffer, ref value) => {
                gl.clear_buffer_fv(buffer, draw_buffer, value)
            },
            WebGLCommand::ClearBufferiv(buffer, draw_buffer, ref value) => {
                gl.clear_buffer_iv(buffer, draw_buffer, value)
            },
            WebGLCommand::ClearBufferuiv(buffer, draw_buffer, ref value) => {
                gl.clear_buffer_uiv(buffer, draw_buffer, value)
            },
            WebGLCommand::ClearBufferfi(buffer, draw_buffer, depth, stencil) => {
                gl.clear_buffer_fi(buffer, draw_buffer, depth, stencil)
            },
            WebGLCommand::InvalidateFramebuffer(target, ref attachments) => {
                gl.invalidate_framebuffer(target, attachments)
            },
            WebGLCommand::InvalidateSubFramebuffer(target, ref attachments, x, y, w, h) => {
                gl.invalidate_sub_framebuffer(target, attachments, x, y, w, h)
            },
            WebGLCommand::FramebufferTextureLayer(target, attachment, tex_id, level, layer) => {
                let tex_id = tex_id.map_or(0, WebGLTextureId::get);
                let attach = |attachment| {
                    gl.framebuffer_texture_layer(target, attachment, tex_id, level, layer)
                };

                if attachment == gl::DEPTH_STENCIL_ATTACHMENT {
                    attach(gl::DEPTH_ATTACHMENT);
                    attach(gl::STENCIL_ATTACHMENT);
                } else {
                    attach(attachment)
                }
            },
            WebGLCommand::ReadBuffer(buffer) => gl.read_buffer(buffer),
            WebGLCommand::DrawBuffers(ref buffers) => gl.draw_buffers(buffers),
        }

        // If debug asertions are enabled, then check the error state.
        #[cfg(debug_assertions)]
        {
            let error = gl.get_error();
            if error != gl::NO_ERROR {
                error!("Last GL operation failed: {:?}", command);
                if error == gl::INVALID_FRAMEBUFFER_OPERATION {
                    let mut framebuffer_bindings = [0];
                    unsafe {
                        gl.get_integer_v(gl::DRAW_FRAMEBUFFER_BINDING, &mut framebuffer_bindings);
                    }
                    debug!(
                        "(thread {:?}) Current draw framebuffer binding: {}",
                        ::std::thread::current().id(),
                        framebuffer_bindings[0]
                    );
                }
                #[cfg(feature = "webgl_backtrace")]
                {
                    error!("Backtrace from failed WebGL API:\n{}", _backtrace.backtrace);
                    if let Some(backtrace) = _backtrace.js_backtrace {
                        error!("JS backtrace from failed WebGL API:\n{}", backtrace);
                    }
                }
                // TODO(servo#30568) revert to panic!() once underlying bug is fixed
                log::warn!(
                    "debug assertion failed! Unexpected WebGL error: 0x{:x} ({}) [{:?}]",
                    error,
                    error,
                    command
                );
            }
        }
    }

    fn initialize_framebuffer(gl: &Gl, state: &GLState, color: bool, depth: bool, stencil: bool) {
        let bits = [
            (color, gl::COLOR_BUFFER_BIT),
            (depth, gl::DEPTH_BUFFER_BIT),
            (stencil, gl::STENCIL_BUFFER_BIT),
        ]
        .iter()
        .fold(0, |bits, &(enabled, bit)| {
            bits | if enabled { bit } else { 0 }
        });

        gl.disable(gl::SCISSOR_TEST);
        gl.color_mask(true, true, true, true);
        gl.clear_color(0., 0., 0., 0.);
        gl.depth_mask(true);
        gl.clear_depth(1.);
        gl.stencil_mask_separate(gl::FRONT, 0xFFFFFFFF);
        gl.stencil_mask_separate(gl::BACK, 0xFFFFFFFF);
        gl.clear_stencil(0);
        gl.clear(bits);

        state.restore_invariant(gl);
    }

    #[allow(unsafe_code)]
    fn link_program(gl: &Gl, program: WebGLProgramId) -> ProgramLinkInfo {
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
                active_uniform_blocks: vec![].into(),
                transform_feedback_length: Default::default(),
                transform_feedback_mode: Default::default(),
            };
        }
        let mut num_active_attribs = [0];
        unsafe {
            gl.get_program_iv(
                program.get(),
                gl::ACTIVE_ATTRIBUTES,
                &mut num_active_attribs,
            );
        }
        let active_attribs = (0..num_active_attribs[0] as u32)
            .map(|i| {
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
            })
            .collect::<Vec<_>>()
            .into();

        let mut num_active_uniforms = [0];
        unsafe {
            gl.get_program_iv(program.get(), gl::ACTIVE_UNIFORMS, &mut num_active_uniforms);
        }
        let active_uniforms = (0..num_active_uniforms[0] as u32)
            .map(|i| {
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
                    bind_index: None,
                }
            })
            .collect::<Vec<_>>()
            .into();

        let mut num_active_uniform_blocks = [0];
        unsafe {
            gl.get_program_iv(
                program.get(),
                gl::ACTIVE_UNIFORM_BLOCKS,
                &mut num_active_uniform_blocks,
            );
        }
        let active_uniform_blocks = (0..num_active_uniform_blocks[0] as u32)
            .map(|i| {
                let name = gl.get_active_uniform_block_name(program.get(), i);
                let size =
                    gl.get_active_uniform_block_iv(program.get(), i, gl::UNIFORM_BLOCK_DATA_SIZE)
                        [0];
                ActiveUniformBlockInfo { name, size }
            })
            .collect::<Vec<_>>()
            .into();

        let mut transform_feedback_length = [0];
        unsafe {
            gl.get_program_iv(
                program.get(),
                gl::TRANSFORM_FEEDBACK_VARYINGS,
                &mut transform_feedback_length,
            );
        }
        let mut transform_feedback_mode = [0];
        unsafe {
            gl.get_program_iv(
                program.get(),
                gl::TRANSFORM_FEEDBACK_BUFFER_MODE,
                &mut transform_feedback_mode,
            );
        }

        ProgramLinkInfo {
            linked: true,
            active_attribs,
            active_uniforms,
            active_uniform_blocks,
            transform_feedback_length: transform_feedback_length[0],
            transform_feedback_mode: transform_feedback_mode[0],
        }
    }

    fn finish(gl: &Gl, chan: &WebGLSender<()>) {
        gl.finish();
        chan.send(()).unwrap();
    }

    fn shader_precision_format(
        gl: &Gl,
        shader_type: u32,
        precision_type: u32,
        chan: &WebGLSender<(i32, i32, i32)>,
    ) {
        let result = gl.get_shader_precision_format(shader_type, precision_type);
        chan.send(result).unwrap();
    }

    // surfman creates a legacy OpenGL context on macOS when
    // OpenGL 2 support is requested. Legacy contexts return GL errors for the vertex
    // array object functions, but support a set of APPLE extension functions that
    // provide VAO support instead.
    fn needs_apple_vertex_arrays(gl_version: GLVersion) -> bool {
        cfg!(target_os = "macos") && gl_version.major < 3
    }

    #[allow(unsafe_code)]
    fn get_extensions(gl: &Gl, chan: &WebGLSender<String>) {
        let mut ext_count = [0];
        unsafe {
            gl.get_integer_v(gl::NUM_EXTENSIONS, &mut ext_count);
        }
        // Fall back to the depricated extensions API if that fails
        if gl.get_error() != gl::NO_ERROR {
            chan.send(gl.get_string(gl::EXTENSIONS)).unwrap();
            return;
        }
        let ext_count = ext_count[0] as usize;
        let mut extensions = Vec::with_capacity(ext_count);
        for idx in 0..ext_count {
            extensions.push(gl.get_string_i(gl::EXTENSIONS, idx as u32))
        }
        let extensions = extensions.join(" ");
        chan.send(extensions).unwrap();
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn get_framebuffer_attachment_parameter(
        gl: &Gl,
        target: u32,
        attachment: u32,
        pname: u32,
        chan: &WebGLSender<i32>,
    ) {
        let parameter = gl.get_framebuffer_attachment_parameter_iv(target, attachment, pname);
        chan.send(parameter).unwrap();
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7
    fn get_renderbuffer_parameter(gl: &Gl, target: u32, pname: u32, chan: &WebGLSender<i32>) {
        let parameter = gl.get_renderbuffer_parameter_iv(target, pname);
        chan.send(parameter).unwrap();
    }

    fn uniform_location(gl: &Gl, program_id: WebGLProgramId, name: &str, chan: &WebGLSender<i32>) {
        let location = gl.get_uniform_location(program_id.get(), &to_name_in_compiled_shader(name));
        chan.send(location).unwrap();
    }

    fn shader_info_log(gl: &Gl, shader_id: WebGLShaderId, chan: &WebGLSender<String>) {
        let log = gl.get_shader_info_log(shader_id.get());
        chan.send(log).unwrap();
    }

    fn program_info_log(gl: &Gl, program_id: WebGLProgramId, chan: &WebGLSender<String>) {
        let log = gl.get_program_info_log(program_id.get());
        chan.send(log).unwrap();
    }

    #[allow(unsafe_code)]
    fn create_buffer(gl: &Gl, chan: &WebGLSender<Option<WebGLBufferId>>) {
        let buffer = gl.gen_buffers(1)[0];
        let buffer = if buffer == 0 {
            None
        } else {
            Some(unsafe { WebGLBufferId::new(buffer) })
        };
        chan.send(buffer).unwrap();
    }

    #[allow(unsafe_code)]
    fn create_framebuffer(gl: &Gl, chan: &WebGLSender<Option<WebGLFramebufferId>>) {
        let framebuffer = gl.gen_framebuffers(1)[0];
        let framebuffer = if framebuffer == 0 {
            None
        } else {
            Some(unsafe { WebGLFramebufferId::new(framebuffer) })
        };
        chan.send(framebuffer).unwrap();
    }

    #[allow(unsafe_code)]
    fn create_renderbuffer(gl: &Gl, chan: &WebGLSender<Option<WebGLRenderbufferId>>) {
        let renderbuffer = gl.gen_renderbuffers(1)[0];
        let renderbuffer = if renderbuffer == 0 {
            None
        } else {
            Some(unsafe { WebGLRenderbufferId::new(renderbuffer) })
        };
        chan.send(renderbuffer).unwrap();
    }

    #[allow(unsafe_code)]
    fn create_texture(gl: &Gl, chan: &WebGLSender<Option<WebGLTextureId>>) {
        let texture = gl.gen_textures(1)[0];
        let texture = if texture == 0 {
            None
        } else {
            Some(unsafe { WebGLTextureId::new(texture) })
        };
        chan.send(texture).unwrap();
    }

    #[allow(unsafe_code)]
    fn create_program(gl: &Gl, chan: &WebGLSender<Option<WebGLProgramId>>) {
        let program = gl.create_program();
        let program = if program == 0 {
            None
        } else {
            Some(unsafe { WebGLProgramId::new(program) })
        };
        chan.send(program).unwrap();
    }

    #[allow(unsafe_code)]
    fn create_shader(gl: &Gl, shader_type: u32, chan: &WebGLSender<Option<WebGLShaderId>>) {
        let shader = gl.create_shader(shader_type);
        let shader = if shader == 0 {
            None
        } else {
            Some(unsafe { WebGLShaderId::new(shader) })
        };
        chan.send(shader).unwrap();
    }

    #[allow(unsafe_code)]
    fn create_vertex_array(
        gl: &Gl,
        use_apple_ext: bool,
        version: WebGLVersion,
    ) -> Option<WebGLVertexArrayId> {
        let vao = match gl {
            Gl::Gl(ref gl) if use_apple_ext => {
                let mut ids = vec![0];
                unsafe {
                    gl.GenVertexArraysAPPLE(ids.len() as gl::GLsizei, ids.as_mut_ptr());
                }
                ids[0]
            },
            Gl::Gles(ref gles) if version == WebGLVersion::WebGL1 => {
                let mut ids = vec![0];
                unsafe { gles.GenVertexArraysOES(ids.len() as gl::GLsizei, ids.as_mut_ptr()) }
                ids[0]
            },
            _ => gl.gen_vertex_arrays(1)[0],
        };
        if vao == 0 {
            let code = gl.get_error();
            warn!("Failed to create vertex array with error code {:x}", code);
            None
        } else {
            Some(unsafe { WebGLVertexArrayId::new(vao) })
        }
    }

    #[allow(unsafe_code)]
    fn bind_vertex_array(gl: &Gl, vao: GLuint, use_apple_ext: bool, version: WebGLVersion) {
        match gl {
            Gl::Gl(ref gl) if use_apple_ext => unsafe {
                gl.BindVertexArrayAPPLE(vao);
            },
            Gl::Gles(ref gles) if version == WebGLVersion::WebGL1 => unsafe {
                gles.BindVertexArrayOES(vao);
            },
            _ => gl.bind_vertex_array(vao),
        }
        debug_assert_eq!(gl.get_error(), gl::NO_ERROR);
    }

    #[allow(unsafe_code)]
    fn delete_vertex_array(gl: &Gl, vao: GLuint, use_apple_ext: bool, version: WebGLVersion) {
        let vaos = [vao];
        match gl {
            Gl::Gl(ref gl) if use_apple_ext => unsafe {
                gl.DeleteVertexArraysAPPLE(vaos.len() as gl::GLsizei, vaos.as_ptr());
            },
            Gl::Gles(ref gl) if version == WebGLVersion::WebGL1 => unsafe {
                gl.DeleteVertexArraysOES(vaos.len() as gl::GLsizei, vaos.as_ptr());
            },
            _ => gl.delete_vertex_arrays(&vaos),
        }
        debug_assert_eq!(gl.get_error(), gl::NO_ERROR);
    }

    #[inline]
    fn bind_framebuffer(
        gl: &Gl,
        target: u32,
        request: WebGLFramebufferBindingRequest,
        ctx: &Context,
        device: &Device,
        state: &mut GLState,
    ) {
        let id = match request {
            WebGLFramebufferBindingRequest::Explicit(id) => id.get(),
            WebGLFramebufferBindingRequest::Default => {
                device
                    .context_surface_info(ctx)
                    .unwrap()
                    .expect("No surface attached!")
                    .framebuffer_object
            },
        };

        debug!("WebGLImpl::bind_framebuffer: {:?}", id);
        gl.bind_framebuffer(target, id);

        if (target == gl::FRAMEBUFFER) || (target == gl::DRAW_FRAMEBUFFER) {
            state.drawing_to_default_framebuffer =
                request == WebGLFramebufferBindingRequest::Default;
            state.restore_invariant(gl);
        }
    }

    #[inline]
    fn compile_shader(gl: &Gl, shader_id: WebGLShaderId, source: &str) {
        gl.shader_source(shader_id.get(), &[source.as_bytes()]);
        gl.compile_shader(shader_id.get());
    }
}

/// ANGLE adds a `_u` prefix to variable names:
///
/// <https://chromium.googlesource.com/angle/angle/+/855d964bd0d05f6b2cb303f625506cf53d37e94f>
///
/// To avoid hard-coding this we would need to use the `sh::GetAttributes` and `sh::GetUniforms`
/// API to look up the `x.name` and `x.mappedName` members.
const ANGLE_NAME_PREFIX: &str = "_u";

fn to_name_in_compiled_shader(s: &str) -> String {
    map_dot_separated(s, |s, mapped| {
        mapped.push_str(ANGLE_NAME_PREFIX);
        mapped.push_str(s);
    })
}

fn from_name_in_compiled_shader(s: &str) -> String {
    map_dot_separated(s, |s, mapped| {
        mapped.push_str(if let Some(stripped) = s.strip_prefix(ANGLE_NAME_PREFIX) {
            stripped
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

#[allow(clippy::too_many_arguments)]
fn prepare_pixels(
    internal_format: TexFormat,
    data_type: TexDataType,
    size: Size2D<u32>,
    unpacking_alignment: u32,
    alpha_treatment: Option<AlphaTreatment>,
    y_axis_treatment: YAxisTreatment,
    pixel_format: Option<PixelFormat>,
    mut pixels: Cow<[u8]>,
) -> Cow<[u8]> {
    match alpha_treatment {
        Some(AlphaTreatment::Premultiply) => {
            if let Some(pixel_format) = pixel_format {
                match pixel_format {
                    PixelFormat::BGRA8 | PixelFormat::RGBA8 => {},
                    _ => unimplemented!("unsupported pixel format ({:?})", pixel_format),
                }
                premultiply_inplace(TexFormat::RGBA, TexDataType::UnsignedByte, pixels.to_mut());
            } else {
                premultiply_inplace(internal_format, data_type, pixels.to_mut());
            }
        },
        Some(AlphaTreatment::Unmultiply) => {
            assert!(pixel_format.is_some());
            unmultiply_inplace(pixels.to_mut());
        },
        None => {},
    }

    if let Some(pixel_format) = pixel_format {
        pixels = image_to_tex_image_data(
            pixel_format,
            internal_format,
            data_type,
            pixels.into_owned(),
        )
        .into();
    }

    if y_axis_treatment == YAxisTreatment::Flipped {
        // FINISHME: Consider doing premultiply and flip in a single mutable Vec.
        pixels = flip_pixels_y(
            internal_format,
            data_type,
            size.width as usize,
            size.height as usize,
            unpacking_alignment as usize,
            pixels.into_owned(),
        )
        .into();
    }

    pixels
}

/// Translates an image in rgba8 (red in the first byte) format to
/// the format that was requested of TexImage.
fn image_to_tex_image_data(
    pixel_format: PixelFormat,
    format: TexFormat,
    data_type: TexDataType,
    mut pixels: Vec<u8>,
) -> Vec<u8> {
    // hint for vector allocation sizing.
    let pixel_count = pixels.len() / 4;

    match pixel_format {
        PixelFormat::BGRA8 => pixels::rgba8_byte_swap_colors_inplace(&mut pixels),
        PixelFormat::RGBA8 => {},
        _ => unimplemented!("unsupported pixel format ({:?})", pixel_format),
    }

    match (format, data_type) {
        (TexFormat::RGBA, TexDataType::UnsignedByte) |
        (TexFormat::RGBA8, TexDataType::UnsignedByte) => pixels,
        (TexFormat::RGB, TexDataType::UnsignedByte) |
        (TexFormat::RGB8, TexDataType::UnsignedByte) => {
            for i in 0..pixel_count {
                let rgb = {
                    let rgb = &pixels[i * 4..i * 4 + 3];
                    [rgb[0], rgb[1], rgb[2]]
                };
                pixels[i * 3..i * 3 + 3].copy_from_slice(&rgb);
            }
            pixels.truncate(pixel_count * 3);
            pixels
        },
        (TexFormat::Alpha, TexDataType::UnsignedByte) => {
            for i in 0..pixel_count {
                let p = pixels[i * 4 + 3];
                pixels[i] = p;
            }
            pixels.truncate(pixel_count);
            pixels
        },
        (TexFormat::Luminance, TexDataType::UnsignedByte) => {
            for i in 0..pixel_count {
                let p = pixels[i * 4];
                pixels[i] = p;
            }
            pixels.truncate(pixel_count);
            pixels
        },
        (TexFormat::LuminanceAlpha, TexDataType::UnsignedByte) => {
            for i in 0..pixel_count {
                let (lum, a) = {
                    let rgba = &pixels[i * 4..i * 4 + 4];
                    (rgba[0], rgba[3])
                };
                pixels[i * 2] = lum;
                pixels[i * 2 + 1] = a;
            }
            pixels.truncate(pixel_count * 2);
            pixels
        },
        (TexFormat::RGBA, TexDataType::UnsignedShort4444) => {
            for i in 0..pixel_count {
                let p = {
                    let rgba = &pixels[i * 4..i * 4 + 4];
                    (rgba[0] as u16 & 0xf0) << 8 |
                        (rgba[1] as u16 & 0xf0) << 4 |
                        (rgba[2] as u16 & 0xf0) |
                        (rgba[3] as u16 & 0xf0) >> 4
                };
                NativeEndian::write_u16(&mut pixels[i * 2..i * 2 + 2], p);
            }
            pixels.truncate(pixel_count * 2);
            pixels
        },
        (TexFormat::RGBA, TexDataType::UnsignedShort5551) => {
            for i in 0..pixel_count {
                let p = {
                    let rgba = &pixels[i * 4..i * 4 + 4];
                    (rgba[0] as u16 & 0xf8) << 8 |
                        (rgba[1] as u16 & 0xf8) << 3 |
                        (rgba[2] as u16 & 0xf8) >> 2 |
                        (rgba[3] as u16) >> 7
                };
                NativeEndian::write_u16(&mut pixels[i * 2..i * 2 + 2], p);
            }
            pixels.truncate(pixel_count * 2);
            pixels
        },
        (TexFormat::RGB, TexDataType::UnsignedShort565) => {
            for i in 0..pixel_count {
                let p = {
                    let rgb = &pixels[i * 4..i * 4 + 3];
                    (rgb[0] as u16 & 0xf8) << 8 |
                        (rgb[1] as u16 & 0xfc) << 3 |
                        (rgb[2] as u16 & 0xf8) >> 3
                };
                NativeEndian::write_u16(&mut pixels[i * 2..i * 2 + 2], p);
            }
            pixels.truncate(pixel_count * 2);
            pixels
        },
        (TexFormat::RGBA, TexDataType::Float) | (TexFormat::RGBA32f, TexDataType::Float) => {
            let mut rgbaf32 = Vec::<u8>::with_capacity(pixel_count * 16);
            for rgba8 in pixels.chunks(4) {
                rgbaf32.write_f32::<NativeEndian>(rgba8[0] as f32).unwrap();
                rgbaf32.write_f32::<NativeEndian>(rgba8[1] as f32).unwrap();
                rgbaf32.write_f32::<NativeEndian>(rgba8[2] as f32).unwrap();
                rgbaf32.write_f32::<NativeEndian>(rgba8[3] as f32).unwrap();
            }
            rgbaf32
        },

        (TexFormat::RGB, TexDataType::Float) | (TexFormat::RGB32f, TexDataType::Float) => {
            let mut rgbf32 = Vec::<u8>::with_capacity(pixel_count * 12);
            for rgba8 in pixels.chunks(4) {
                rgbf32.write_f32::<NativeEndian>(rgba8[0] as f32).unwrap();
                rgbf32.write_f32::<NativeEndian>(rgba8[1] as f32).unwrap();
                rgbf32.write_f32::<NativeEndian>(rgba8[2] as f32).unwrap();
            }
            rgbf32
        },

        (TexFormat::Alpha, TexDataType::Float) | (TexFormat::Alpha32f, TexDataType::Float) => {
            for rgba8 in pixels.chunks_mut(4) {
                let p = rgba8[3] as f32;
                NativeEndian::write_f32(rgba8, p);
            }
            pixels
        },

        (TexFormat::Luminance, TexDataType::Float) |
        (TexFormat::Luminance32f, TexDataType::Float) => {
            for rgba8 in pixels.chunks_mut(4) {
                let p = rgba8[0] as f32;
                NativeEndian::write_f32(rgba8, p);
            }
            pixels
        },

        (TexFormat::LuminanceAlpha, TexDataType::Float) |
        (TexFormat::LuminanceAlpha32f, TexDataType::Float) => {
            let mut data = Vec::<u8>::with_capacity(pixel_count * 8);
            for rgba8 in pixels.chunks(4) {
                data.write_f32::<NativeEndian>(rgba8[0] as f32).unwrap();
                data.write_f32::<NativeEndian>(rgba8[3] as f32).unwrap();
            }
            data
        },

        (TexFormat::RGBA, TexDataType::HalfFloat) |
        (TexFormat::RGBA16f, TexDataType::HalfFloat) => {
            let mut rgbaf16 = Vec::<u8>::with_capacity(pixel_count * 8);
            for rgba8 in pixels.chunks(4) {
                rgbaf16
                    .write_u16::<NativeEndian>(f16::from_f32(rgba8[0] as f32).to_bits())
                    .unwrap();
                rgbaf16
                    .write_u16::<NativeEndian>(f16::from_f32(rgba8[1] as f32).to_bits())
                    .unwrap();
                rgbaf16
                    .write_u16::<NativeEndian>(f16::from_f32(rgba8[2] as f32).to_bits())
                    .unwrap();
                rgbaf16
                    .write_u16::<NativeEndian>(f16::from_f32(rgba8[3] as f32).to_bits())
                    .unwrap();
            }
            rgbaf16
        },

        (TexFormat::RGB, TexDataType::HalfFloat) | (TexFormat::RGB16f, TexDataType::HalfFloat) => {
            let mut rgbf16 = Vec::<u8>::with_capacity(pixel_count * 6);
            for rgba8 in pixels.chunks(4) {
                rgbf16
                    .write_u16::<NativeEndian>(f16::from_f32(rgba8[0] as f32).to_bits())
                    .unwrap();
                rgbf16
                    .write_u16::<NativeEndian>(f16::from_f32(rgba8[1] as f32).to_bits())
                    .unwrap();
                rgbf16
                    .write_u16::<NativeEndian>(f16::from_f32(rgba8[2] as f32).to_bits())
                    .unwrap();
            }
            rgbf16
        },
        (TexFormat::Alpha, TexDataType::HalfFloat) |
        (TexFormat::Alpha16f, TexDataType::HalfFloat) => {
            for i in 0..pixel_count {
                let p = f16::from_f32(pixels[i * 4 + 3] as f32).to_bits();
                NativeEndian::write_u16(&mut pixels[i * 2..i * 2 + 2], p);
            }
            pixels.truncate(pixel_count * 2);
            pixels
        },
        (TexFormat::Luminance, TexDataType::HalfFloat) |
        (TexFormat::Luminance16f, TexDataType::HalfFloat) => {
            for i in 0..pixel_count {
                let p = f16::from_f32(pixels[i * 4] as f32).to_bits();
                NativeEndian::write_u16(&mut pixels[i * 2..i * 2 + 2], p);
            }
            pixels.truncate(pixel_count * 2);
            pixels
        },
        (TexFormat::LuminanceAlpha, TexDataType::HalfFloat) |
        (TexFormat::LuminanceAlpha16f, TexDataType::HalfFloat) => {
            for rgba8 in pixels.chunks_mut(4) {
                let lum = f16::from_f32(rgba8[0] as f32).to_bits();
                let a = f16::from_f32(rgba8[3] as f32).to_bits();
                NativeEndian::write_u16(&mut rgba8[0..2], lum);
                NativeEndian::write_u16(&mut rgba8[2..4], a);
            }
            pixels
        },

        // Validation should have ensured that we only hit the
        // above cases, but we haven't turned the (format, type)
        // into an enum yet so there's a default case here.
        _ => unreachable!("Unsupported formats {:?} {:?}", format, data_type),
    }
}

fn premultiply_inplace(format: TexFormat, data_type: TexDataType, pixels: &mut [u8]) {
    match (format, data_type) {
        (TexFormat::RGBA, TexDataType::UnsignedByte) => {
            pixels::rgba8_premultiply_inplace(pixels);
        },
        (TexFormat::LuminanceAlpha, TexDataType::UnsignedByte) => {
            for la in pixels.chunks_mut(2) {
                la[0] = pixels::multiply_u8_color(la[0], la[1]);
            }
        },
        (TexFormat::RGBA, TexDataType::UnsignedShort5551) => {
            for rgba in pixels.chunks_mut(2) {
                if NativeEndian::read_u16(rgba) & 1 == 0 {
                    NativeEndian::write_u16(rgba, 0);
                }
            }
        },
        (TexFormat::RGBA, TexDataType::UnsignedShort4444) => {
            for rgba in pixels.chunks_mut(2) {
                let pix = NativeEndian::read_u16(rgba);
                let extend_to_8_bits = |val| (val | val << 4) as u8;
                let r = extend_to_8_bits(pix >> 12 & 0x0f);
                let g = extend_to_8_bits(pix >> 8 & 0x0f);
                let b = extend_to_8_bits(pix >> 4 & 0x0f);
                let a = extend_to_8_bits(pix & 0x0f);
                NativeEndian::write_u16(
                    rgba,
                    ((pixels::multiply_u8_color(r, a) & 0xf0) as u16) << 8 |
                        ((pixels::multiply_u8_color(g, a) & 0xf0) as u16) << 4 |
                        ((pixels::multiply_u8_color(b, a) & 0xf0) as u16) |
                        ((a & 0x0f) as u16),
                );
            }
        },
        // Other formats don't have alpha, so return their data untouched.
        _ => {},
    }
}

fn unmultiply_inplace(pixels: &mut [u8]) {
    for rgba in pixels.chunks_mut(4) {
        let a = (rgba[3] as f32) / 255.0;
        rgba[0] = (rgba[0] as f32 / a) as u8;
        rgba[1] = (rgba[1] as f32 / a) as u8;
        rgba[2] = (rgba[2] as f32 / a) as u8;
    }
}

/// Flips the pixels in the Vec on the Y axis.
fn flip_pixels_y(
    internal_format: TexFormat,
    data_type: TexDataType,
    width: usize,
    height: usize,
    unpacking_alignment: usize,
    pixels: Vec<u8>,
) -> Vec<u8> {
    let cpp = (data_type.element_size() * internal_format.components() /
        data_type.components_per_element()) as usize;

    let stride = (width * cpp + unpacking_alignment - 1) & !(unpacking_alignment - 1);

    let mut flipped = Vec::<u8>::with_capacity(pixels.len());

    for y in 0..height {
        let flipped_y = height - 1 - y;
        let start = flipped_y * stride;

        flipped.extend_from_slice(&pixels[start..(start + width * cpp)]);
        flipped.extend(vec![0u8; stride - width * cpp]);
    }

    flipped
}

// Clamp a size to the current GL context's max viewport
fn clamp_viewport(gl: &Gl, size: Size2D<u32>) -> Size2D<u32> {
    let mut max_viewport = [i32::max_value(), i32::max_value()];
    let mut max_renderbuffer = [i32::max_value()];
    #[allow(unsafe_code)]
    unsafe {
        gl.get_integer_v(gl::MAX_VIEWPORT_DIMS, &mut max_viewport);
        gl.get_integer_v(gl::MAX_RENDERBUFFER_SIZE, &mut max_renderbuffer);
        debug_assert_eq!(gl.get_error(), gl::NO_ERROR);
    }
    Size2D::new(
        size.width
            .min(max_viewport[0] as u32)
            .min(max_renderbuffer[0] as u32)
            .max(1),
        size.height
            .min(max_viewport[1] as u32)
            .min(max_renderbuffer[0] as u32)
            .max(1),
    )
}

trait ToSurfmanVersion {
    fn to_surfman_version(self, api_type: gl::GlType) -> GLVersion;
}

impl ToSurfmanVersion for WebGLVersion {
    fn to_surfman_version(self, api_type: gl::GlType) -> GLVersion {
        if api_type == gl::GlType::Gles {
            return GLVersion::new(3, 0);
        }
        match self {
            // We make use of GL_PACK_PIXEL_BUFFER, which needs at least GL2.1
            // We make use of compatibility mode, which needs at most GL3.0
            WebGLVersion::WebGL1 => GLVersion::new(2, 1),
            // The WebGL2 conformance tests use std140 layout, which needs at GL3.1
            WebGLVersion::WebGL2 => GLVersion::new(3, 2),
        }
    }
}

trait SurfmanContextAttributeFlagsConvert {
    fn to_surfman_context_attribute_flags(
        &self,
        webgl_version: WebGLVersion,
        api_type: gl::GlType,
    ) -> ContextAttributeFlags;
}

impl SurfmanContextAttributeFlagsConvert for GLContextAttributes {
    fn to_surfman_context_attribute_flags(
        &self,
        webgl_version: WebGLVersion,
        api_type: gl::GlType,
    ) -> ContextAttributeFlags {
        let mut flags = ContextAttributeFlags::empty();
        flags.set(ContextAttributeFlags::ALPHA, self.alpha);
        flags.set(ContextAttributeFlags::DEPTH, self.depth);
        flags.set(ContextAttributeFlags::STENCIL, self.stencil);
        if (webgl_version == WebGLVersion::WebGL1) && (api_type == gl::GlType::Gl) {
            flags.set(ContextAttributeFlags::COMPATIBILITY_PROFILE, true);
        }
        flags
    }
}

bitflags! {
    struct FramebufferRebindingFlags: u8 {
        const REBIND_READ_FRAMEBUFFER = 0x1;
        const REBIND_DRAW_FRAMEBUFFER = 0x2;
    }
}

struct FramebufferRebindingInfo {
    flags: FramebufferRebindingFlags,
    viewport: [GLint; 4],
}

impl FramebufferRebindingInfo {
    #[allow(unsafe_code)]
    fn detect(device: &Device, context: &Context, gl: &Gl) -> FramebufferRebindingInfo {
        unsafe {
            let (mut read_framebuffer, mut draw_framebuffer) = ([0], [0]);
            gl.get_integer_v(gl::READ_FRAMEBUFFER_BINDING, &mut read_framebuffer);
            gl.get_integer_v(gl::DRAW_FRAMEBUFFER_BINDING, &mut draw_framebuffer);

            let context_surface_framebuffer = device
                .context_surface_info(context)
                .unwrap()
                .unwrap()
                .framebuffer_object;

            let mut flags = FramebufferRebindingFlags::empty();
            if context_surface_framebuffer == read_framebuffer[0] as GLuint {
                flags.insert(FramebufferRebindingFlags::REBIND_READ_FRAMEBUFFER);
            }
            if context_surface_framebuffer == draw_framebuffer[0] as GLuint {
                flags.insert(FramebufferRebindingFlags::REBIND_DRAW_FRAMEBUFFER);
            }

            let mut viewport = [0; 4];
            gl.get_integer_v(gl::VIEWPORT, &mut viewport);

            FramebufferRebindingInfo { flags, viewport }
        }
    }

    fn apply(self, device: &Device, context: &Context, gl: &Gl) {
        if self.flags.is_empty() {
            return;
        }

        let context_surface_framebuffer = device
            .context_surface_info(context)
            .unwrap()
            .unwrap()
            .framebuffer_object;
        if self
            .flags
            .contains(FramebufferRebindingFlags::REBIND_READ_FRAMEBUFFER)
        {
            gl.bind_framebuffer(gl::READ_FRAMEBUFFER, context_surface_framebuffer);
        }
        if self
            .flags
            .contains(FramebufferRebindingFlags::REBIND_DRAW_FRAMEBUFFER)
        {
            gl.bind_framebuffer(gl::DRAW_FRAMEBUFFER, context_surface_framebuffer);
        }

        gl.viewport(
            self.viewport[0],
            self.viewport[1],
            self.viewport[2],
            self.viewport[3],
        );
    }
}

/// Bridge between WebGL and WebXR
pub(crate) struct WebXRBridge {
    factory_receiver: crossbeam_channel::Receiver<WebXRLayerManagerFactory<WebXRSurfman>>,
    managers: HashMap<WebXRLayerManagerId, Box<dyn WebXRLayerManagerAPI<WebXRSurfman>>>,
    next_manager_id: u32,
}

impl WebXRBridge {
    pub(crate) fn new(init: WebXRBridgeInit) -> WebXRBridge {
        let WebXRBridgeInit {
            factory_receiver, ..
        } = init;
        let managers = HashMap::new();
        let next_manager_id = 1;
        WebXRBridge {
            factory_receiver,
            managers,
            next_manager_id,
        }
    }
}

impl WebXRBridge {
    #[allow(unsafe_code)]
    fn create_layer_manager(
        &mut self,
        device: &mut Device,
        contexts: &mut dyn WebXRContexts<WebXRSurfman>,
    ) -> Result<WebXRLayerManagerId, WebXRError> {
        let factory = self
            .factory_receiver
            .recv()
            .map_err(|_| WebXRError::CommunicationError)?;
        let manager = factory.build(device, contexts)?;
        let manager_id = unsafe { WebXRLayerManagerId::new(self.next_manager_id) };
        self.next_manager_id += 1;
        self.managers.insert(manager_id, manager);
        Ok(manager_id)
    }

    fn destroy_layer_manager(&mut self, manager_id: WebXRLayerManagerId) {
        self.managers.remove(&manager_id);
    }

    fn create_layer(
        &mut self,
        manager_id: WebXRLayerManagerId,
        device: &mut Device,
        contexts: &mut dyn WebXRContexts<WebXRSurfman>,
        context_id: WebXRContextId,
        layer_init: WebXRLayerInit,
    ) -> Result<WebXRLayerId, WebXRError> {
        let manager = self
            .managers
            .get_mut(&manager_id)
            .ok_or(WebXRError::NoMatchingDevice)?;
        manager.create_layer(device, contexts, context_id, layer_init)
    }

    fn destroy_layer(
        &mut self,
        manager_id: WebXRLayerManagerId,
        device: &mut Device,
        contexts: &mut dyn WebXRContexts<WebXRSurfman>,
        context_id: WebXRContextId,
        layer_id: WebXRLayerId,
    ) {
        if let Some(manager) = self.managers.get_mut(&manager_id) {
            manager.destroy_layer(device, contexts, context_id, layer_id);
        }
    }

    fn destroy_all_layers(
        &mut self,
        device: &mut Device,
        contexts: &mut dyn WebXRContexts<WebXRSurfman>,
        context_id: WebXRContextId,
    ) {
        for manager in self.managers.values_mut() {
            #[allow(clippy::unnecessary_to_owned)] // Needs mutable borrow later in destroy
            for (other_id, layer_id) in manager.layers().to_vec() {
                if other_id == context_id {
                    manager.destroy_layer(device, contexts, context_id, layer_id);
                }
            }
        }
    }

    fn begin_frame(
        &mut self,
        manager_id: WebXRLayerManagerId,
        device: &mut Device,
        contexts: &mut dyn WebXRContexts<WebXRSurfman>,
        layers: &[(WebXRContextId, WebXRLayerId)],
    ) -> Result<Vec<WebXRSubImages>, WebXRError> {
        let manager = self
            .managers
            .get_mut(&manager_id)
            .ok_or(WebXRError::NoMatchingDevice)?;
        manager.begin_frame(device, contexts, layers)
    }

    fn end_frame(
        &mut self,
        manager_id: WebXRLayerManagerId,
        device: &mut Device,
        contexts: &mut dyn WebXRContexts<WebXRSurfman>,
        layers: &[(WebXRContextId, WebXRLayerId)],
    ) -> Result<(), WebXRError> {
        let manager = self
            .managers
            .get_mut(&manager_id)
            .ok_or(WebXRError::NoMatchingDevice)?;
        manager.end_frame(device, contexts, layers)
    }
}

pub(crate) struct WebXRBridgeInit {
    sender: WebGLSender<WebGLMsg>,
    factory_receiver: crossbeam_channel::Receiver<WebXRLayerManagerFactory<WebXRSurfman>>,
    factory_sender: crossbeam_channel::Sender<WebXRLayerManagerFactory<WebXRSurfman>>,
}

impl WebXRBridgeInit {
    pub(crate) fn new(sender: WebGLSender<WebGLMsg>) -> WebXRBridgeInit {
        let (factory_sender, factory_receiver) = crossbeam_channel::unbounded();
        WebXRBridgeInit {
            sender,
            factory_sender,
            factory_receiver,
        }
    }

    pub(crate) fn layer_grand_manager(&self) -> WebXRLayerGrandManager<WebXRSurfman> {
        WebXRLayerGrandManager::new(WebXRBridgeGrandManager {
            sender: self.sender.clone(),
            factory_sender: self.factory_sender.clone(),
        })
    }
}

struct WebXRBridgeGrandManager {
    sender: WebGLSender<WebGLMsg>,
    // WebXR layer manager factories use generic trait objects under the
    // hood, which aren't deserializable (even using typetag)
    // so we can't send them over the regular webgl channel.
    // Fortunately, the webgl thread runs in the same process as
    // the webxr threads, so we can use a crossbeam channel to send
    // factories.
    factory_sender: crossbeam_channel::Sender<WebXRLayerManagerFactory<WebXRSurfman>>,
}

impl WebXRLayerGrandManagerAPI<WebXRSurfman> for WebXRBridgeGrandManager {
    fn create_layer_manager(
        &self,
        factory: WebXRLayerManagerFactory<WebXRSurfman>,
    ) -> Result<WebXRLayerManager, WebXRError> {
        let (sender, receiver) = webgl_channel().ok_or(WebXRError::CommunicationError)?;
        let _ = self.factory_sender.send(factory);
        let _ = self
            .sender
            .send(WebGLMsg::WebXRCommand(WebXRCommand::CreateLayerManager(
                sender,
            )));
        let sender = self.sender.clone();
        let manager_id = receiver
            .recv()
            .map_err(|_| WebXRError::CommunicationError)??;
        let layers = Vec::new();
        Ok(WebXRLayerManager::new(WebXRBridgeManager {
            manager_id,
            sender,
            layers,
        }))
    }

    fn clone_layer_grand_manager(&self) -> WebXRLayerGrandManager<WebXRSurfman> {
        WebXRLayerGrandManager::new(WebXRBridgeGrandManager {
            sender: self.sender.clone(),
            factory_sender: self.factory_sender.clone(),
        })
    }
}

struct WebXRBridgeManager {
    sender: WebGLSender<WebGLMsg>,
    manager_id: WebXRLayerManagerId,
    layers: Vec<(WebXRContextId, WebXRLayerId)>,
}

impl<GL: WebXRTypes> WebXRLayerManagerAPI<GL> for WebXRBridgeManager {
    fn create_layer(
        &mut self,
        _: &mut GL::Device,
        _: &mut dyn WebXRContexts<GL>,
        context_id: WebXRContextId,
        init: WebXRLayerInit,
    ) -> Result<WebXRLayerId, WebXRError> {
        let (sender, receiver) = webgl_channel().ok_or(WebXRError::CommunicationError)?;
        let _ = self
            .sender
            .send(WebGLMsg::WebXRCommand(WebXRCommand::CreateLayer(
                self.manager_id,
                context_id,
                init,
                sender,
            )));
        let layer_id = receiver
            .recv()
            .map_err(|_| WebXRError::CommunicationError)??;
        self.layers.push((context_id, layer_id));
        Ok(layer_id)
    }

    fn destroy_layer(
        &mut self,
        _: &mut GL::Device,
        _: &mut dyn WebXRContexts<GL>,
        context_id: WebXRContextId,
        layer_id: WebXRLayerId,
    ) {
        self.layers.retain(|&ids| ids != (context_id, layer_id));
        let _ = self
            .sender
            .send(WebGLMsg::WebXRCommand(WebXRCommand::DestroyLayer(
                self.manager_id,
                context_id,
                layer_id,
            )));
    }

    fn layers(&self) -> &[(WebXRContextId, WebXRLayerId)] {
        &self.layers[..]
    }

    fn begin_frame(
        &mut self,
        _: &mut GL::Device,
        _: &mut dyn WebXRContexts<GL>,
        layers: &[(WebXRContextId, WebXRLayerId)],
    ) -> Result<Vec<WebXRSubImages>, WebXRError> {
        let (sender, receiver) = webgl_channel().ok_or(WebXRError::CommunicationError)?;
        let _ = self
            .sender
            .send(WebGLMsg::WebXRCommand(WebXRCommand::BeginFrame(
                self.manager_id,
                layers.to_vec(),
                sender,
            )));
        receiver
            .recv()
            .map_err(|_| WebXRError::CommunicationError)?
    }

    fn end_frame(
        &mut self,
        _: &mut GL::Device,
        _: &mut dyn WebXRContexts<GL>,
        layers: &[(WebXRContextId, WebXRLayerId)],
    ) -> Result<(), WebXRError> {
        let (sender, receiver) = webgl_channel().ok_or(WebXRError::CommunicationError)?;
        let _ = self
            .sender
            .send(WebGLMsg::WebXRCommand(WebXRCommand::EndFrame(
                self.manager_id,
                layers.to_vec(),
                sender,
            )));
        receiver
            .recv()
            .map_err(|_| WebXRError::CommunicationError)?
    }
}

impl Drop for WebXRBridgeManager {
    fn drop(&mut self) {
        let _ = self
            .sender
            .send(WebGLMsg::WebXRCommand(WebXRCommand::DestroyLayerManager(
                self.manager_id,
            )));
    }
}

struct WebXRBridgeContexts<'a> {
    contexts: &'a mut FnvHashMap<WebGLContextId, GLContextData>,
    bound_context_id: &'a mut Option<WebGLContextId>,
}

impl<'a> WebXRContexts<WebXRSurfman> for WebXRBridgeContexts<'a> {
    fn context(&mut self, device: &Device, context_id: WebXRContextId) -> Option<&mut Context> {
        let data = WebGLThread::make_current_if_needed_mut(
            device,
            WebGLContextId::from(context_id),
            self.contexts,
            self.bound_context_id,
        )?;
        Some(&mut data.ctx)
    }
    fn bindings(&mut self, device: &Device, context_id: WebXRContextId) -> Option<&Gl> {
        let data = WebGLThread::make_current_if_needed(
            device,
            WebGLContextId::from(context_id),
            self.contexts,
            self.bound_context_id,
        )?;
        Some(&data.gl)
    }
}
