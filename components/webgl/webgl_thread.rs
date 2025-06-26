/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
#![allow(unsafe_code)]
use std::borrow::Cow;
use std::num::NonZeroU32;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::{slice, thread};

use bitflags::bitflags;
use byteorder::{ByteOrder, NativeEndian, WriteBytesExt};
use canvas_traits::webgl;
#[cfg(feature = "webxr")]
use canvas_traits::webgl::WebXRCommand;
use canvas_traits::webgl::{
    ActiveAttribInfo, ActiveUniformBlockInfo, ActiveUniformInfo, AlphaTreatment,
    GLContextAttributes, GLLimits, GlType, InternalFormatIntVec, ProgramLinkInfo, TexDataType,
    TexFormat, WebGLBufferId, WebGLChan, WebGLCommand, WebGLCommandBacktrace, WebGLContextId,
    WebGLCreateContextResult, WebGLFramebufferBindingRequest, WebGLFramebufferId, WebGLMsg,
    WebGLMsgSender, WebGLProgramId, WebGLQueryId, WebGLReceiver, WebGLRenderbufferId,
    WebGLSLVersion, WebGLSamplerId, WebGLSender, WebGLShaderId, WebGLSyncId, WebGLTextureId,
    WebGLVersion, WebGLVertexArrayId, YAxisTreatment,
};
use compositing_traits::{
    CrossProcessCompositorApi, ImageUpdate, SerializableImageData, WebrenderExternalImageRegistry,
    WebrenderImageHandlerType,
};
use euclid::default::Size2D;
use fnv::FnvHashMap;
use glow::{
    self as gl, ActiveTransformFeedback, Context as Gl, HasContext, NativeTransformFeedback,
    NativeUniformLocation, NativeVertexArray, PixelUnpackData, ShaderPrecisionFormat,
    bytes_per_type, components_per_format,
};
use half::f16;
use ipc_channel::ipc::IpcSharedMemory;
use itertools::Itertools;
use log::{debug, error, trace, warn};
use pixels::{self, PixelFormat, SnapshotAlphaMode, unmultiply_inplace};
use surfman::chains::{PreserveBuffer, SwapChains, SwapChainsAPI};
use surfman::{
    self, Adapter, Connection, Context, ContextAttributeFlags, ContextAttributes, Device,
    GLVersion, SurfaceAccess, SurfaceInfo, SurfaceType,
};
use webrender::{RenderApi, RenderApiSender};
use webrender_api::units::DeviceIntSize;
use webrender_api::{
    ExternalImageData, ExternalImageId, ExternalImageType, ImageBufferKind, ImageDescriptor,
    ImageDescriptorFlags, ImageFormat, ImageKey,
};

use crate::webgl_limits::GLLimitsDetect;
#[cfg(feature = "webxr")]
use crate::webxr::{WebXRBridge, WebXRBridgeContexts, WebXRBridgeInit};

type GLint = i32;

fn native_uniform_location(location: i32) -> Option<NativeUniformLocation> {
    location.try_into().ok().map(NativeUniformLocation)
}

pub(crate) struct GLContextData {
    pub(crate) ctx: Context,
    pub(crate) gl: Rc<glow::Context>,
    state: GLState,
    attributes: GLContextAttributes,
}

#[derive(Debug)]
pub struct GLState {
    _webgl_version: WebGLVersion,
    _gl_version: GLVersion,
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
    default_vao: Option<NativeVertexArray>,
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
        unsafe { gl.clear_color(r, g, b, a) };
    }

    fn restore_scissor_invariant(&self, gl: &Gl) {
        if self.scissor_test_enabled {
            unsafe { gl.enable(gl::SCISSOR_TEST) };
        } else {
            unsafe { gl.disable(gl::SCISSOR_TEST) };
        }
    }

    fn restore_alpha_invariant(&self, gl: &Gl) {
        let [r, g, b, a] = self.color_write_mask;
        if self.fake_no_alpha() {
            unsafe { gl.color_mask(r, g, b, false) };
        } else {
            unsafe { gl.color_mask(r, g, b, a) };
        }
    }

    fn restore_depth_invariant(&self, gl: &Gl) {
        unsafe {
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
    }

    fn restore_stencil_invariant(&self, gl: &Gl) {
        unsafe {
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
}

impl Default for GLState {
    fn default() -> GLState {
        GLState {
            _gl_version: GLVersion { major: 1, minor: 0 },
            _webgl_version: WebGLVersion::WebGL1,
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
            default_vao: None,
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
    compositor_api: CrossProcessCompositorApi,
    webrender_api: RenderApi,
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
    api_type: GlType,
    #[cfg(feature = "webxr")]
    /// The bridge to WebXR
    pub webxr_bridge: WebXRBridge,
}

/// The data required to initialize an instance of the WebGLThread type.
pub(crate) struct WebGLThreadInit {
    pub compositor_api: CrossProcessCompositorApi,
    pub webrender_api_sender: RenderApiSender,
    pub external_images: Arc<Mutex<WebrenderExternalImageRegistry>>,
    pub sender: WebGLSender<WebGLMsg>,
    pub receiver: WebGLReceiver<WebGLMsg>,
    pub webrender_swap_chains: SwapChains<WebGLContextId, Device>,
    pub connection: Connection,
    pub adapter: Adapter,
    pub api_type: GlType,
    #[cfg(feature = "webxr")]
    pub webxr_init: WebXRBridgeInit,
}

// A size at which it should be safe to create GL contexts
const SAFE_VIEWPORT_DIMS: [u32; 2] = [1024, 1024];

impl WebGLThread {
    /// Create a new instance of WebGLThread.
    pub(crate) fn new(
        WebGLThreadInit {
            compositor_api,
            webrender_api_sender,
            external_images,
            sender,
            receiver,
            webrender_swap_chains,
            connection,
            adapter,
            api_type,
            #[cfg(feature = "webxr")]
            webxr_init,
        }: WebGLThreadInit,
    ) -> Self {
        WebGLThread {
            device: connection
                .create_device(&adapter)
                .expect("Couldn't open WebGL device!"),
            compositor_api,
            webrender_api: webrender_api_sender.create_api(),
            contexts: Default::default(),
            cached_context_info: Default::default(),
            bound_context_id: None,
            external_images,
            sender,
            receiver: receiver.into_inner(),
            webrender_swap_chains,
            api_type,
            #[cfg(feature = "webxr")]
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
                break;
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
                        let api_type = if data.gl.version().is_embedded {
                            GlType::Gles
                        } else {
                            GlType::Gl
                        };

                        // FIXME(nox): Should probably be done by surfman.
                        if api_type != GlType::Gles {
                            // Points sprites are enabled by default in OpenGL 3.2 core
                            // and in GLES. Rather than doing version detection, it does
                            // not hurt to enable them anyways.

                            unsafe {
                                // XXX: Do we even need to this?
                                const GL_POINT_SPRITE: u32 = 0x8861;
                                data.gl.enable(GL_POINT_SPRITE);
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
            WebGLMsg::WebXRCommand(_command) => {
                #[cfg(feature = "webxr")]
                self.handle_webxr_command(_command);
            },
            WebGLMsg::SwapBuffers(swap_ids, sender, sent_time) => {
                self.handle_swap_buffers(swap_ids, sender, sent_time);
            },
            WebGLMsg::Exit(sender) => {
                // Call remove_context functions in order to correctly delete WebRender image keys.
                let context_ids: Vec<WebGLContextId> = self.contexts.keys().copied().collect();
                for id in context_ids {
                    self.remove_webgl_context(id);
                }

                // Block on shutting-down WebRender.
                self.webrender_api.shut_down(true);
                if let Err(e) = sender.send(()) {
                    warn!("Failed to send response to WebGLMsg::Exit ({e})");
                }
                return true;
            },
        }

        false
    }

    #[cfg(feature = "webxr")]
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

        let gl = unsafe {
            Rc::new(match self.api_type {
                GlType::Gl => glow::Context::from_loader_function(|symbol_name| {
                    self.device.get_proc_address(&ctx, symbol_name)
                }),
                GlType::Gles => glow::Context::from_loader_function(|symbol_name| {
                    self.device.get_proc_address(&ctx, symbol_name)
                }),
            })
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

        unsafe {
            gl.bind_framebuffer(gl::FRAMEBUFFER, framebuffer);
            gl.viewport(0, 0, size.width as i32, size.height as i32);
            gl.scissor(0, 0, size.width as i32, size.height as i32);
            gl.clear_color(0., 0., 0., !has_alpha as u32 as f32);
            gl.clear_depth(1.);
            gl.clear_stencil(0);
            gl.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
            gl.clear_color(0., 0., 0., 0.);
            debug_assert_eq!(gl.get_error(), gl::NO_ERROR);
        }

        let default_vao = if let Some(vao) = WebGLImpl::create_vertex_array(&gl) {
            WebGLImpl::bind_vertex_array(&gl, Some(vao.glow()));
            Some(vao.glow())
        } else {
            None
        };

        let state = GLState {
            _gl_version: gl_version,
            _webgl_version: webgl_version,
            requested_flags,
            default_vao,
            ..Default::default()
        };
        debug!("Created state {:?}", state);

        state.restore_invariant(&gl);
        debug_assert_eq!(unsafe { gl.get_error() }, gl::NO_ERROR);

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
            &self.compositor_api,
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
        debug_assert_eq!(unsafe { data.gl.get_error() }, gl::NO_ERROR);

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
            self.compositor_api
                .update_images(vec![ImageUpdate::DeleteImage(info.image_key)]);
        }

        // We need to make the context current so its resources can be disposed of.
        Self::make_current_if_needed(
            &self.device,
            context_id,
            &self.contexts,
            &mut self.bound_context_id,
        );

        #[cfg(feature = "webxr")]
        {
            // Destroy WebXR layers associated with this context
            let webxr_context_id = webxr_api::ContextId::from(context_id);
            let mut webxr_contexts = WebXRBridgeContexts {
                contexts: &mut self.contexts,
                bound_context_id: &mut self.bound_context_id,
            };
            self.webxr_bridge.destroy_all_layers(
                &mut self.device,
                &mut webxr_contexts,
                webxr_context_id,
            );
        }

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
            debug_assert_eq!(unsafe { data.gl.get_error() }, gl::NO_ERROR);

            // Check to see if any of the current framebuffer bindings are the surface we're about
            // to swap out. If so, we'll have to reset them after destroying the surface.
            let framebuffer_rebinding_info =
                FramebufferRebindingInfo::detect(&self.device, &data.ctx, &data.gl);
            debug_assert_eq!(unsafe { data.gl.get_error() }, gl::NO_ERROR);

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
            debug_assert_eq!(unsafe { data.gl.get_error() }, gl::NO_ERROR);

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
                debug_assert_eq!(unsafe { data.gl.get_error() }, gl::NO_ERROR);
            }

            // Rebind framebuffers as appropriate.
            debug!("Rebinding {:?}", context_id);
            framebuffer_rebinding_info.apply(&self.device, &data.ctx, &data.gl);
            debug_assert_eq!(unsafe { data.gl.get_error() }, gl::NO_ERROR);

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
                "... rebound framebuffer {:?}, new back buffer surface is {:?}",
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
    pub(crate) fn make_current_if_needed<'a>(
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
    pub(crate) fn make_current_if_needed_mut<'a>(
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
        compositor_api: &CrossProcessCompositorApi,
        size: Size2D<i32>,
        alpha: bool,
        context_id: WebGLContextId,
        image_buffer_kind: ImageBufferKind,
    ) -> ImageKey {
        let descriptor = Self::image_descriptor(size, alpha);
        let data = Self::external_image_data(context_id, image_buffer_kind);

        let image_key = compositor_api.generate_image_key().unwrap();
        compositor_api.add_image(image_key, descriptor, data);

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

        self.compositor_api
            .update_images(vec![ImageUpdate::UpdateImage(
                info.image_key,
                descriptor,
                image_data,
            )]);
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
    ) -> SerializableImageData {
        let data = ExternalImageData {
            id: ExternalImageId(context_id.0),
            channel_index: 0,
            image_type: ExternalImageType::TextureHandle(image_buffer_kind),
            normalized_uvs: false,
        };
        SerializableImageData::External(data)
    }

    /// Gets the GLSL Version supported by a GLContext.
    fn get_glsl_version(gl: &Gl) -> WebGLSLVersion {
        let version = unsafe { gl.get_parameter_string(gl::SHADING_LANGUAGE_VERSION) };
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
        debug_assert_eq!(unsafe { gl.get_error() }, gl::NO_ERROR);

        match command {
            WebGLCommand::GetContextAttributes(ref sender) => sender.send(*attributes).unwrap(),
            WebGLCommand::ActiveTexture(target) => unsafe { gl.active_texture(target) },
            WebGLCommand::AttachShader(program_id, shader_id) => unsafe {
                gl.attach_shader(program_id.glow(), shader_id.glow())
            },
            WebGLCommand::DetachShader(program_id, shader_id) => unsafe {
                gl.detach_shader(program_id.glow(), shader_id.glow())
            },
            WebGLCommand::BindAttribLocation(program_id, index, ref name) => unsafe {
                gl.bind_attrib_location(program_id.glow(), index, &to_name_in_compiled_shader(name))
            },
            WebGLCommand::BlendColor(r, g, b, a) => unsafe { gl.blend_color(r, g, b, a) },
            WebGLCommand::BlendEquation(mode) => unsafe { gl.blend_equation(mode) },
            WebGLCommand::BlendEquationSeparate(mode_rgb, mode_alpha) => unsafe {
                gl.blend_equation_separate(mode_rgb, mode_alpha)
            },
            WebGLCommand::BlendFunc(src, dest) => unsafe { gl.blend_func(src, dest) },
            WebGLCommand::BlendFuncSeparate(src_rgb, dest_rgb, src_alpha, dest_alpha) => unsafe {
                gl.blend_func_separate(src_rgb, dest_rgb, src_alpha, dest_alpha)
            },
            WebGLCommand::BufferData(buffer_type, ref receiver, usage) => unsafe {
                gl.buffer_data_u8_slice(buffer_type, &receiver.recv().unwrap(), usage)
            },
            WebGLCommand::BufferSubData(buffer_type, offset, ref receiver) => unsafe {
                gl.buffer_sub_data_u8_slice(buffer_type, offset as i32, &receiver.recv().unwrap())
            },
            WebGLCommand::CopyBufferSubData(src, dst, src_offset, dst_offset, size) => {
                unsafe {
                    gl.copy_buffer_sub_data(
                        src,
                        dst,
                        src_offset as i32,
                        dst_offset as i32,
                        size as i32,
                    )
                };
            },
            WebGLCommand::GetBufferSubData(buffer_type, offset, length, ref sender) => unsafe {
                let ptr = gl.map_buffer_range(
                    buffer_type,
                    offset as i32,
                    length as i32,
                    gl::MAP_READ_BIT,
                );
                let data: &[u8] = slice::from_raw_parts(ptr as _, length);
                sender.send(data).unwrap();
                gl.unmap_buffer(buffer_type);
            },
            WebGLCommand::Clear(mask) => {
                unsafe { gl.clear(mask) };
            },
            WebGLCommand::ClearColor(r, g, b, a) => {
                state.clear_color = (r, g, b, a);
                unsafe { gl.clear_color(r, g, b, a) };
            },
            WebGLCommand::ClearDepth(depth) => {
                let value = depth.clamp(0., 1.) as f64;
                state.depth_clear_value = value;
                unsafe { gl.clear_depth(value) }
            },
            WebGLCommand::ClearStencil(stencil) => {
                state.stencil_clear_value = stencil;
                unsafe { gl.clear_stencil(stencil) };
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
            ) => unsafe {
                gl.copy_tex_image_2d(target, level, internal_format, x, y, width, height, border)
            },
            WebGLCommand::CopyTexSubImage2D(
                target,
                level,
                xoffset,
                yoffset,
                x,
                y,
                width,
                height,
            ) => unsafe {
                gl.copy_tex_sub_image_2d(target, level, xoffset, yoffset, x, y, width, height)
            },
            WebGLCommand::CullFace(mode) => unsafe { gl.cull_face(mode) },
            WebGLCommand::DepthFunc(func) => unsafe { gl.depth_func(func) },
            WebGLCommand::DepthMask(flag) => {
                state.depth_write_mask = flag;
                state.restore_depth_invariant(gl);
            },
            WebGLCommand::DepthRange(near, far) => unsafe {
                gl.depth_range(near.clamp(0., 1.) as f64, far.clamp(0., 1.) as f64)
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
                _ => unsafe { gl.disable(cap) },
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
                _ => unsafe { gl.enable(cap) },
            },
            WebGLCommand::FramebufferRenderbuffer(target, attachment, renderbuffertarget, rb) => {
                let attach = |attachment| unsafe {
                    gl.framebuffer_renderbuffer(
                        target,
                        attachment,
                        renderbuffertarget,
                        rb.map(WebGLRenderbufferId::glow),
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
                let attach = |attachment| unsafe {
                    gl.framebuffer_texture_2d(
                        target,
                        attachment,
                        textarget,
                        texture.map(WebGLTextureId::glow),
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
            WebGLCommand::FrontFace(mode) => unsafe { gl.front_face(mode) },
            WebGLCommand::DisableVertexAttribArray(attrib_id) => unsafe {
                gl.disable_vertex_attrib_array(attrib_id)
            },
            WebGLCommand::EnableVertexAttribArray(attrib_id) => unsafe {
                gl.enable_vertex_attrib_array(attrib_id)
            },
            WebGLCommand::Hint(name, val) => unsafe { gl.hint(name, val) },
            WebGLCommand::LineWidth(width) => {
                unsafe { gl.line_width(width) };
                // In OpenGL Core Profile >3.2, any non-1.0 value will generate INVALID_VALUE.
                if width != 1.0 {
                    let _ = unsafe { gl.get_error() };
                }
            },
            WebGLCommand::PixelStorei(name, val) => unsafe { gl.pixel_store_i32(name, val) },
            WebGLCommand::PolygonOffset(factor, units) => unsafe {
                gl.polygon_offset(factor, units)
            },
            WebGLCommand::ReadPixels(rect, format, pixel_type, ref sender) => {
                let len = bytes_per_type(pixel_type) *
                    components_per_format(format) *
                    rect.size.area() as usize;
                let mut pixels = vec![0; len];
                unsafe {
                    // We don't want any alignment padding on pixel rows.
                    gl.pixel_store_i32(glow::PACK_ALIGNMENT, 1);
                    gl.read_pixels(
                        rect.origin.x as i32,
                        rect.origin.y as i32,
                        rect.size.width as i32,
                        rect.size.height as i32,
                        format,
                        pixel_type,
                        glow::PixelPackData::Slice(Some(&mut pixels)),
                    )
                };
                let alpha_mode = match (attributes.alpha, attributes.premultiplied_alpha) {
                    (true, premultiplied) => SnapshotAlphaMode::Transparent { premultiplied },
                    (false, _) => SnapshotAlphaMode::Opaque,
                };
                sender
                    .send((IpcSharedMemory::from_bytes(&pixels), alpha_mode))
                    .unwrap();
            },
            WebGLCommand::ReadPixelsPP(rect, format, pixel_type, offset) => unsafe {
                gl.read_pixels(
                    rect.origin.x,
                    rect.origin.y,
                    rect.size.width,
                    rect.size.height,
                    format,
                    pixel_type,
                    glow::PixelPackData::BufferOffset(offset as u32),
                );
            },
            WebGLCommand::RenderbufferStorage(target, format, width, height) => unsafe {
                gl.renderbuffer_storage(target, format, width, height)
            },
            WebGLCommand::RenderbufferStorageMultisample(
                target,
                samples,
                format,
                width,
                height,
            ) => unsafe {
                gl.renderbuffer_storage_multisample(target, samples, format, width, height)
            },
            WebGLCommand::SampleCoverage(value, invert) => unsafe {
                gl.sample_coverage(value, invert)
            },
            WebGLCommand::Scissor(x, y, width, height) => {
                // FIXME(nox): Kinda unfortunate that some u32 values could
                // end up as negative numbers here, but I don't even think
                // that can happen in the real world.
                unsafe { gl.scissor(x, y, width as i32, height as i32) };
            },
            WebGLCommand::StencilFunc(func, ref_, mask) => unsafe {
                gl.stencil_func(func, ref_, mask)
            },
            WebGLCommand::StencilFuncSeparate(face, func, ref_, mask) => unsafe {
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
            WebGLCommand::StencilOp(fail, zfail, zpass) => unsafe {
                gl.stencil_op(fail, zfail, zpass)
            },
            WebGLCommand::StencilOpSeparate(face, fail, zfail, zpass) => unsafe {
                gl.stencil_op_separate(face, fail, zfail, zpass)
            },
            WebGLCommand::GetRenderbufferParameter(target, pname, ref chan) => {
                Self::get_renderbuffer_parameter(gl, target, pname, chan)
            },
            WebGLCommand::CreateTransformFeedback(ref sender) => {
                let value = unsafe { gl.create_transform_feedback() }.ok();
                sender
                    .send(value.map(|ntf| ntf.0.get()).unwrap_or_default())
                    .unwrap()
            },
            WebGLCommand::DeleteTransformFeedback(id) => {
                if let Some(tf) = NonZeroU32::new(id) {
                    unsafe { gl.delete_transform_feedback(NativeTransformFeedback(tf)) };
                }
            },
            WebGLCommand::IsTransformFeedback(id, ref sender) => {
                let value = NonZeroU32::new(id)
                    .map(|id| unsafe { gl.is_transform_feedback(NativeTransformFeedback(id)) })
                    .unwrap_or_default();
                sender.send(value).unwrap()
            },
            WebGLCommand::BindTransformFeedback(target, id) => {
                unsafe {
                    gl.bind_transform_feedback(
                        target,
                        NonZeroU32::new(id).map(NativeTransformFeedback),
                    )
                };
            },
            WebGLCommand::BeginTransformFeedback(mode) => {
                unsafe { gl.begin_transform_feedback(mode) };
            },
            WebGLCommand::EndTransformFeedback() => {
                unsafe { gl.end_transform_feedback() };
            },
            WebGLCommand::PauseTransformFeedback() => {
                unsafe { gl.pause_transform_feedback() };
            },
            WebGLCommand::ResumeTransformFeedback() => {
                unsafe { gl.resume_transform_feedback() };
            },
            WebGLCommand::GetTransformFeedbackVarying(program, index, ref sender) => {
                let ActiveTransformFeedback { size, tftype, name } =
                    unsafe { gl.get_transform_feedback_varying(program.glow(), index) }.unwrap();
                // We need to split, because the name starts with '_u' prefix.
                let name = from_name_in_compiled_shader(&name);
                sender.send((size, tftype, name)).unwrap();
            },
            WebGLCommand::TransformFeedbackVaryings(program, ref varyings, buffer_mode) => {
                let varyings: Vec<String> = varyings
                    .iter()
                    .map(|varying| to_name_in_compiled_shader(varying))
                    .collect();
                let varyings_refs: Vec<&str> = varyings.iter().map(String::as_ref).collect();
                unsafe {
                    gl.transform_feedback_varyings(
                        program.glow(),
                        varyings_refs.as_slice(),
                        buffer_mode,
                    )
                };
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
                let location = unsafe {
                    gl.get_frag_data_location(program_id.glow(), &to_name_in_compiled_shader(name))
                };
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
            WebGLCommand::DeleteBuffer(id) => unsafe { gl.delete_buffer(id.glow()) },
            WebGLCommand::DeleteFramebuffer(id) => unsafe { gl.delete_framebuffer(id.glow()) },
            WebGLCommand::DeleteRenderbuffer(id) => unsafe { gl.delete_renderbuffer(id.glow()) },
            WebGLCommand::DeleteTexture(id) => unsafe { gl.delete_texture(id.glow()) },
            WebGLCommand::DeleteProgram(id) => unsafe { gl.delete_program(id.glow()) },
            WebGLCommand::DeleteShader(id) => unsafe { gl.delete_shader(id.glow()) },
            WebGLCommand::BindBuffer(target, id) => unsafe {
                gl.bind_buffer(target, id.map(WebGLBufferId::glow))
            },
            WebGLCommand::BindFramebuffer(target, request) => {
                Self::bind_framebuffer(gl, target, request, ctx, device, state)
            },
            WebGLCommand::BindRenderbuffer(target, id) => unsafe {
                gl.bind_renderbuffer(target, id.map(WebGLRenderbufferId::glow))
            },
            WebGLCommand::BindTexture(target, id) => unsafe {
                gl.bind_texture(target, id.map(WebGLTextureId::glow))
            },
            WebGLCommand::BlitFrameBuffer(
                src_x0,
                src_y0,
                src_x1,
                src_y1,
                dst_x0,
                dst_y0,
                dst_x1,
                dst_y1,
                mask,
                filter,
            ) => unsafe {
                gl.blit_framebuffer(
                    src_x0, src_y0, src_x1, src_y1, dst_x0, dst_y0, dst_x1, dst_y1, mask, filter,
                );
            },
            WebGLCommand::Uniform1f(uniform_id, v) => unsafe {
                gl.uniform_1_f32(native_uniform_location(uniform_id).as_ref(), v)
            },
            WebGLCommand::Uniform1fv(uniform_id, ref v) => unsafe {
                gl.uniform_1_f32_slice(native_uniform_location(uniform_id).as_ref(), v)
            },
            WebGLCommand::Uniform1i(uniform_id, v) => unsafe {
                gl.uniform_1_i32(native_uniform_location(uniform_id).as_ref(), v)
            },
            WebGLCommand::Uniform1iv(uniform_id, ref v) => unsafe {
                gl.uniform_1_i32_slice(native_uniform_location(uniform_id).as_ref(), v)
            },
            WebGLCommand::Uniform1ui(uniform_id, v) => unsafe {
                gl.uniform_1_u32(native_uniform_location(uniform_id).as_ref(), v)
            },
            WebGLCommand::Uniform1uiv(uniform_id, ref v) => unsafe {
                gl.uniform_1_u32_slice(native_uniform_location(uniform_id).as_ref(), v)
            },
            WebGLCommand::Uniform2f(uniform_id, x, y) => unsafe {
                gl.uniform_2_f32(native_uniform_location(uniform_id).as_ref(), x, y)
            },
            WebGLCommand::Uniform2fv(uniform_id, ref v) => unsafe {
                gl.uniform_2_f32_slice(native_uniform_location(uniform_id).as_ref(), v)
            },
            WebGLCommand::Uniform2i(uniform_id, x, y) => unsafe {
                gl.uniform_2_i32(native_uniform_location(uniform_id).as_ref(), x, y)
            },
            WebGLCommand::Uniform2iv(uniform_id, ref v) => unsafe {
                gl.uniform_2_i32_slice(native_uniform_location(uniform_id).as_ref(), v)
            },
            WebGLCommand::Uniform2ui(uniform_id, x, y) => unsafe {
                gl.uniform_2_u32(native_uniform_location(uniform_id).as_ref(), x, y)
            },
            WebGLCommand::Uniform2uiv(uniform_id, ref v) => unsafe {
                gl.uniform_2_u32_slice(native_uniform_location(uniform_id).as_ref(), v)
            },
            WebGLCommand::Uniform3f(uniform_id, x, y, z) => unsafe {
                gl.uniform_3_f32(native_uniform_location(uniform_id).as_ref(), x, y, z)
            },
            WebGLCommand::Uniform3fv(uniform_id, ref v) => unsafe {
                gl.uniform_3_f32_slice(native_uniform_location(uniform_id).as_ref(), v)
            },
            WebGLCommand::Uniform3i(uniform_id, x, y, z) => unsafe {
                gl.uniform_3_i32(native_uniform_location(uniform_id).as_ref(), x, y, z)
            },
            WebGLCommand::Uniform3iv(uniform_id, ref v) => unsafe {
                gl.uniform_3_i32_slice(native_uniform_location(uniform_id).as_ref(), v)
            },
            WebGLCommand::Uniform3ui(uniform_id, x, y, z) => unsafe {
                gl.uniform_3_u32(native_uniform_location(uniform_id).as_ref(), x, y, z)
            },
            WebGLCommand::Uniform3uiv(uniform_id, ref v) => unsafe {
                gl.uniform_3_u32_slice(native_uniform_location(uniform_id).as_ref(), v)
            },
            WebGLCommand::Uniform4f(uniform_id, x, y, z, w) => unsafe {
                gl.uniform_4_f32(native_uniform_location(uniform_id).as_ref(), x, y, z, w)
            },
            WebGLCommand::Uniform4fv(uniform_id, ref v) => unsafe {
                gl.uniform_4_f32_slice(native_uniform_location(uniform_id).as_ref(), v)
            },
            WebGLCommand::Uniform4i(uniform_id, x, y, z, w) => unsafe {
                gl.uniform_4_i32(native_uniform_location(uniform_id).as_ref(), x, y, z, w)
            },
            WebGLCommand::Uniform4iv(uniform_id, ref v) => unsafe {
                gl.uniform_4_i32_slice(native_uniform_location(uniform_id).as_ref(), v)
            },
            WebGLCommand::Uniform4ui(uniform_id, x, y, z, w) => unsafe {
                gl.uniform_4_u32(native_uniform_location(uniform_id).as_ref(), x, y, z, w)
            },
            WebGLCommand::Uniform4uiv(uniform_id, ref v) => unsafe {
                gl.uniform_4_u32_slice(native_uniform_location(uniform_id).as_ref(), v)
            },
            WebGLCommand::UniformMatrix2fv(uniform_id, ref v) => unsafe {
                gl.uniform_matrix_2_f32_slice(
                    native_uniform_location(uniform_id).as_ref(),
                    false,
                    v,
                )
            },
            WebGLCommand::UniformMatrix3fv(uniform_id, ref v) => unsafe {
                gl.uniform_matrix_3_f32_slice(
                    native_uniform_location(uniform_id).as_ref(),
                    false,
                    v,
                )
            },
            WebGLCommand::UniformMatrix4fv(uniform_id, ref v) => unsafe {
                gl.uniform_matrix_4_f32_slice(
                    native_uniform_location(uniform_id).as_ref(),
                    false,
                    v,
                )
            },
            WebGLCommand::UniformMatrix3x2fv(uniform_id, ref v) => unsafe {
                gl.uniform_matrix_3x2_f32_slice(
                    native_uniform_location(uniform_id).as_ref(),
                    false,
                    v,
                )
            },
            WebGLCommand::UniformMatrix4x2fv(uniform_id, ref v) => unsafe {
                gl.uniform_matrix_4x2_f32_slice(
                    native_uniform_location(uniform_id).as_ref(),
                    false,
                    v,
                )
            },
            WebGLCommand::UniformMatrix2x3fv(uniform_id, ref v) => unsafe {
                gl.uniform_matrix_2x3_f32_slice(
                    native_uniform_location(uniform_id).as_ref(),
                    false,
                    v,
                )
            },
            WebGLCommand::UniformMatrix4x3fv(uniform_id, ref v) => unsafe {
                gl.uniform_matrix_4x3_f32_slice(
                    native_uniform_location(uniform_id).as_ref(),
                    false,
                    v,
                )
            },
            WebGLCommand::UniformMatrix2x4fv(uniform_id, ref v) => unsafe {
                gl.uniform_matrix_2x4_f32_slice(
                    native_uniform_location(uniform_id).as_ref(),
                    false,
                    v,
                )
            },
            WebGLCommand::UniformMatrix3x4fv(uniform_id, ref v) => unsafe {
                gl.uniform_matrix_3x4_f32_slice(
                    native_uniform_location(uniform_id).as_ref(),
                    false,
                    v,
                )
            },
            WebGLCommand::ValidateProgram(program_id) => unsafe {
                gl.validate_program(program_id.glow())
            },
            WebGLCommand::VertexAttrib(attrib_id, x, y, z, w) => unsafe {
                gl.vertex_attrib_4_f32(attrib_id, x, y, z, w)
            },
            WebGLCommand::VertexAttribI(attrib_id, x, y, z, w) => unsafe {
                gl.vertex_attrib_4_i32(attrib_id, x, y, z, w)
            },
            WebGLCommand::VertexAttribU(attrib_id, x, y, z, w) => unsafe {
                gl.vertex_attrib_4_u32(attrib_id, x, y, z, w)
            },
            WebGLCommand::VertexAttribPointer2f(attrib_id, size, normalized, stride, offset) => unsafe {
                gl.vertex_attrib_pointer_f32(
                    attrib_id,
                    size,
                    gl::FLOAT,
                    normalized,
                    stride,
                    offset as _,
                )
            },
            WebGLCommand::VertexAttribPointer(
                attrib_id,
                size,
                data_type,
                normalized,
                stride,
                offset,
            ) => unsafe {
                gl.vertex_attrib_pointer_f32(
                    attrib_id,
                    size,
                    data_type,
                    normalized,
                    stride,
                    offset as _,
                )
            },
            WebGLCommand::SetViewport(x, y, width, height) => unsafe {
                gl.viewport(x, y, width, height)
            },
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

                unsafe {
                    gl.pixel_store_i32(gl::UNPACK_ALIGNMENT, unpacking_alignment as i32);
                    gl.tex_image_2d(
                        target,
                        level as i32,
                        internal_format.as_gl_constant() as i32,
                        size.width as i32,
                        size.height as i32,
                        0,
                        format.as_gl_constant(),
                        effective_data_type,
                        PixelUnpackData::Slice(Some(&pixels)),
                    );
                }
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
            } => unsafe {
                gl.pixel_store_i32(gl::UNPACK_ALIGNMENT, unpacking_alignment as i32);

                gl.tex_image_2d(
                    target,
                    level as i32,
                    internal_format.as_gl_constant() as i32,
                    size.width as i32,
                    size.height as i32,
                    0,
                    format.as_gl_constant(),
                    effective_data_type,
                    PixelUnpackData::BufferOffset(offset as u32),
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

                unsafe {
                    gl.pixel_store_i32(gl::UNPACK_ALIGNMENT, unpacking_alignment as i32);
                    gl.tex_sub_image_2d(
                        target,
                        level as i32,
                        xoffset,
                        yoffset,
                        size.width as i32,
                        size.height as i32,
                        format.as_gl_constant(),
                        effective_data_type,
                        glow::PixelUnpackData::Slice(Some(&pixels)),
                    );
                }
            },
            WebGLCommand::CompressedTexImage2D {
                target,
                level,
                internal_format,
                size,
                ref data,
            } => unsafe {
                gl.compressed_tex_image_2d(
                    target,
                    level as i32,
                    internal_format as i32,
                    size.width as i32,
                    size.height as i32,
                    0,
                    data.len() as i32,
                    data,
                )
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
                unsafe {
                    gl.compressed_tex_sub_image_2d(
                        target,
                        level,
                        xoffset,
                        yoffset,
                        size.width as i32,
                        size.height as i32,
                        format,
                        glow::CompressedPixelUnpackData::Slice(data),
                    )
                };
            },
            WebGLCommand::TexStorage2D(target, levels, internal_format, width, height) => unsafe {
                gl.tex_storage_2d(
                    target,
                    levels as i32,
                    internal_format.as_gl_constant(),
                    width as i32,
                    height as i32,
                )
            },
            WebGLCommand::TexStorage3D(target, levels, internal_format, width, height, depth) => unsafe {
                gl.tex_storage_3d(
                    target,
                    levels as i32,
                    internal_format.as_gl_constant(),
                    width as i32,
                    height as i32,
                    depth as i32,
                )
            },
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
            WebGLCommand::Flush => unsafe { gl.flush() },
            WebGLCommand::GenerateMipmap(target) => unsafe { gl.generate_mipmap(target) },
            WebGLCommand::CreateVertexArray(ref chan) => {
                let id = Self::create_vertex_array(gl);
                let _ = chan.send(id);
            },
            WebGLCommand::DeleteVertexArray(id) => {
                Self::delete_vertex_array(gl, id);
            },
            WebGLCommand::BindVertexArray(id) => {
                let id = id.map(WebGLVertexArrayId::glow).or(state.default_vao);
                Self::bind_vertex_array(gl, id);
            },
            WebGLCommand::GetParameterBool(param, ref sender) => {
                let value = match param {
                    webgl::ParameterBool::DepthWritemask => state.depth_write_mask,
                    _ => unsafe { gl.get_parameter_bool(param as u32) },
                };
                sender.send(value).unwrap()
            },
            WebGLCommand::FenceSync(ref sender) => {
                let value = unsafe { gl.fence_sync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0).unwrap() };
                sender.send(WebGLSyncId::from_glow(value)).unwrap();
            },
            WebGLCommand::IsSync(sync_id, ref sender) => {
                let value = unsafe { gl.is_sync(sync_id.glow()) };
                sender.send(value).unwrap();
            },
            WebGLCommand::ClientWaitSync(sync_id, flags, timeout, ref sender) => {
                let value = unsafe { gl.client_wait_sync(sync_id.glow(), flags, timeout as _) };
                sender.send(value).unwrap();
            },
            WebGLCommand::WaitSync(sync_id, flags, timeout) => {
                unsafe { gl.wait_sync(sync_id.glow(), flags, timeout as u64) };
            },
            WebGLCommand::GetSyncParameter(sync_id, param, ref sender) => {
                let value = unsafe { gl.get_sync_parameter_i32(sync_id.glow(), param) };
                sender.send(value as u32).unwrap();
            },
            WebGLCommand::DeleteSync(sync_id) => {
                unsafe { gl.delete_sync(sync_id.glow()) };
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
                    _ => unsafe { gl.get_parameter_i32(param as u32) },
                };
                sender.send(value).unwrap()
            },
            WebGLCommand::GetParameterInt2(param, ref sender) => {
                let mut value = [0; 2];
                unsafe {
                    gl.get_parameter_i32_slice(param as u32, &mut value);
                }
                sender.send(value).unwrap()
            },
            WebGLCommand::GetParameterInt4(param, ref sender) => {
                let mut value = [0; 4];
                unsafe {
                    gl.get_parameter_i32_slice(param as u32, &mut value);
                }
                sender.send(value).unwrap()
            },
            WebGLCommand::GetParameterFloat(param, ref sender) => {
                let mut value = [0.];
                unsafe {
                    gl.get_parameter_f32_slice(param as u32, &mut value);
                }
                sender.send(value[0]).unwrap()
            },
            WebGLCommand::GetParameterFloat2(param, ref sender) => {
                let mut value = [0.; 2];
                unsafe {
                    gl.get_parameter_f32_slice(param as u32, &mut value);
                }
                sender.send(value).unwrap()
            },
            WebGLCommand::GetParameterFloat4(param, ref sender) => {
                let mut value = [0.; 4];
                unsafe {
                    gl.get_parameter_f32_slice(param as u32, &mut value);
                }
                sender.send(value).unwrap()
            },
            WebGLCommand::GetProgramValidateStatus(program, ref sender) => sender
                .send(unsafe { gl.get_program_validate_status(program.glow()) })
                .unwrap(),
            WebGLCommand::GetProgramActiveUniforms(program, ref sender) => sender
                .send(unsafe { gl.get_program_parameter_i32(program.glow(), gl::ACTIVE_UNIFORMS) })
                .unwrap(),
            WebGLCommand::GetCurrentVertexAttrib(index, ref sender) => {
                let mut value = [0.; 4];
                unsafe {
                    gl.get_vertex_attrib_parameter_f32_slice(
                        index,
                        gl::CURRENT_VERTEX_ATTRIB,
                        &mut value,
                    );
                }
                sender.send(value).unwrap();
            },
            WebGLCommand::GetTexParameterFloat(target, param, ref sender) => {
                sender
                    .send(unsafe { gl.get_tex_parameter_f32(target, param as u32) })
                    .unwrap();
            },
            WebGLCommand::GetTexParameterInt(target, param, ref sender) => {
                sender
                    .send(unsafe { gl.get_tex_parameter_i32(target, param as u32) })
                    .unwrap();
            },
            WebGLCommand::GetTexParameterBool(target, param, ref sender) => {
                sender
                    .send(unsafe { gl.get_tex_parameter_i32(target, param as u32) } != 0)
                    .unwrap();
            },
            WebGLCommand::GetInternalFormatIntVec(target, internal_format, param, ref sender) => {
                match param {
                    InternalFormatIntVec::Samples => {
                        let mut count = [0; 1];
                        unsafe {
                            gl.get_internal_format_i32_slice(
                                target,
                                internal_format,
                                gl::NUM_SAMPLE_COUNTS,
                                &mut count,
                            )
                        };
                        assert!(count[0] >= 0);

                        let mut values = vec![0; count[0] as usize];
                        unsafe {
                            gl.get_internal_format_i32_slice(
                                target,
                                internal_format,
                                param as u32,
                                &mut values,
                            )
                        };
                        sender.send(values).unwrap()
                    },
                }
            },
            WebGLCommand::TexParameteri(target, param, value) => unsafe {
                gl.tex_parameter_i32(target, param, value)
            },
            WebGLCommand::TexParameterf(target, param, value) => unsafe {
                gl.tex_parameter_f32(target, param, value)
            },
            WebGLCommand::LinkProgram(program_id, ref sender) => {
                return sender.send(Self::link_program(gl, program_id)).unwrap();
            },
            WebGLCommand::UseProgram(program_id) => unsafe {
                gl.use_program(program_id.map(|p| p.glow()))
            },
            WebGLCommand::DrawArrays { mode, first, count } => unsafe {
                gl.draw_arrays(mode, first, count)
            },
            WebGLCommand::DrawArraysInstanced {
                mode,
                first,
                count,
                primcount,
            } => unsafe { gl.draw_arrays_instanced(mode, first, count, primcount) },
            WebGLCommand::DrawElements {
                mode,
                count,
                type_,
                offset,
            } => unsafe { gl.draw_elements(mode, count, type_, offset as _) },
            WebGLCommand::DrawElementsInstanced {
                mode,
                count,
                type_,
                offset,
                primcount,
            } => unsafe {
                gl.draw_elements_instanced(mode, count, type_, offset as i32, primcount)
            },
            WebGLCommand::VertexAttribDivisor { index, divisor } => unsafe {
                gl.vertex_attrib_divisor(index, divisor)
            },
            WebGLCommand::GetUniformBool(program_id, loc, ref sender) => {
                let mut value = [0];
                unsafe {
                    gl.get_uniform_i32(
                        program_id.glow(),
                        &NativeUniformLocation(loc as u32),
                        &mut value,
                    );
                }
                sender.send(value[0] != 0).unwrap();
            },
            WebGLCommand::GetUniformBool2(program_id, loc, ref sender) => {
                let mut value = [0; 2];
                unsafe {
                    gl.get_uniform_i32(
                        program_id.glow(),
                        &NativeUniformLocation(loc as u32),
                        &mut value,
                    );
                }
                let value = [value[0] != 0, value[1] != 0];
                sender.send(value).unwrap();
            },
            WebGLCommand::GetUniformBool3(program_id, loc, ref sender) => {
                let mut value = [0; 3];
                unsafe {
                    gl.get_uniform_i32(
                        program_id.glow(),
                        &NativeUniformLocation(loc as u32),
                        &mut value,
                    );
                }
                let value = [value[0] != 0, value[1] != 0, value[2] != 0];
                sender.send(value).unwrap();
            },
            WebGLCommand::GetUniformBool4(program_id, loc, ref sender) => {
                let mut value = [0; 4];
                unsafe {
                    gl.get_uniform_i32(
                        program_id.glow(),
                        &NativeUniformLocation(loc as u32),
                        &mut value,
                    );
                }
                let value = [value[0] != 0, value[1] != 0, value[2] != 0, value[3] != 0];
                sender.send(value).unwrap();
            },
            WebGLCommand::GetUniformInt(program_id, loc, ref sender) => {
                let mut value = [0];
                unsafe {
                    gl.get_uniform_i32(
                        program_id.glow(),
                        &NativeUniformLocation(loc as u32),
                        &mut value,
                    );
                }
                sender.send(value[0]).unwrap();
            },
            WebGLCommand::GetUniformInt2(program_id, loc, ref sender) => {
                let mut value = [0; 2];
                unsafe {
                    gl.get_uniform_i32(
                        program_id.glow(),
                        &NativeUniformLocation(loc as u32),
                        &mut value,
                    );
                }
                sender.send(value).unwrap();
            },
            WebGLCommand::GetUniformInt3(program_id, loc, ref sender) => {
                let mut value = [0; 3];
                unsafe {
                    gl.get_uniform_i32(
                        program_id.glow(),
                        &NativeUniformLocation(loc as u32),
                        &mut value,
                    );
                }
                sender.send(value).unwrap();
            },
            WebGLCommand::GetUniformInt4(program_id, loc, ref sender) => {
                let mut value = [0; 4];
                unsafe {
                    gl.get_uniform_i32(
                        program_id.glow(),
                        &NativeUniformLocation(loc as u32),
                        &mut value,
                    );
                }
                sender.send(value).unwrap();
            },
            WebGLCommand::GetUniformUint(program_id, loc, ref sender) => {
                let mut value = [0];
                unsafe {
                    gl.get_uniform_u32(
                        program_id.glow(),
                        &NativeUniformLocation(loc as u32),
                        &mut value,
                    );
                }
                sender.send(value[0]).unwrap();
            },
            WebGLCommand::GetUniformUint2(program_id, loc, ref sender) => {
                let mut value = [0; 2];
                unsafe {
                    gl.get_uniform_u32(
                        program_id.glow(),
                        &NativeUniformLocation(loc as u32),
                        &mut value,
                    );
                }
                sender.send(value).unwrap();
            },
            WebGLCommand::GetUniformUint3(program_id, loc, ref sender) => {
                let mut value = [0; 3];
                unsafe {
                    gl.get_uniform_u32(
                        program_id.glow(),
                        &NativeUniformLocation(loc as u32),
                        &mut value,
                    );
                }
                sender.send(value).unwrap();
            },
            WebGLCommand::GetUniformUint4(program_id, loc, ref sender) => {
                let mut value = [0; 4];
                unsafe {
                    gl.get_uniform_u32(
                        program_id.glow(),
                        &NativeUniformLocation(loc as u32),
                        &mut value,
                    );
                }
                sender.send(value).unwrap();
            },
            WebGLCommand::GetUniformFloat(program_id, loc, ref sender) => {
                let mut value = [0.];
                unsafe {
                    gl.get_uniform_f32(
                        program_id.glow(),
                        &NativeUniformLocation(loc as u32),
                        &mut value,
                    );
                }
                sender.send(value[0]).unwrap();
            },
            WebGLCommand::GetUniformFloat2(program_id, loc, ref sender) => {
                let mut value = [0.; 2];
                unsafe {
                    gl.get_uniform_f32(
                        program_id.glow(),
                        &NativeUniformLocation(loc as u32),
                        &mut value,
                    );
                }
                sender.send(value).unwrap();
            },
            WebGLCommand::GetUniformFloat3(program_id, loc, ref sender) => {
                let mut value = [0.; 3];
                unsafe {
                    gl.get_uniform_f32(
                        program_id.glow(),
                        &NativeUniformLocation(loc as u32),
                        &mut value,
                    );
                }
                sender.send(value).unwrap();
            },
            WebGLCommand::GetUniformFloat4(program_id, loc, ref sender) => {
                let mut value = [0.; 4];
                unsafe {
                    gl.get_uniform_f32(
                        program_id.glow(),
                        &NativeUniformLocation(loc as u32),
                        &mut value,
                    );
                }
                sender.send(value).unwrap();
            },
            WebGLCommand::GetUniformFloat9(program_id, loc, ref sender) => {
                let mut value = [0.; 9];
                unsafe {
                    gl.get_uniform_f32(
                        program_id.glow(),
                        &NativeUniformLocation(loc as u32),
                        &mut value,
                    );
                }
                sender.send(value).unwrap();
            },
            WebGLCommand::GetUniformFloat16(program_id, loc, ref sender) => {
                let mut value = [0.; 16];
                unsafe {
                    gl.get_uniform_f32(
                        program_id.glow(),
                        &NativeUniformLocation(loc as u32),
                        &mut value,
                    );
                }
                sender.send(value).unwrap();
            },
            WebGLCommand::GetUniformFloat2x3(program_id, loc, ref sender) => {
                let mut value = [0.; 2 * 3];
                unsafe {
                    gl.get_uniform_f32(
                        program_id.glow(),
                        &NativeUniformLocation(loc as u32),
                        &mut value,
                    );
                }
                sender.send(value).unwrap()
            },
            WebGLCommand::GetUniformFloat2x4(program_id, loc, ref sender) => {
                let mut value = [0.; 2 * 4];
                unsafe {
                    gl.get_uniform_f32(
                        program_id.glow(),
                        &NativeUniformLocation(loc as u32),
                        &mut value,
                    );
                }
                sender.send(value).unwrap()
            },
            WebGLCommand::GetUniformFloat3x2(program_id, loc, ref sender) => {
                let mut value = [0.; 3 * 2];
                unsafe {
                    gl.get_uniform_f32(
                        program_id.glow(),
                        &NativeUniformLocation(loc as u32),
                        &mut value,
                    );
                }
                sender.send(value).unwrap()
            },
            WebGLCommand::GetUniformFloat3x4(program_id, loc, ref sender) => {
                let mut value = [0.; 3 * 4];
                unsafe {
                    gl.get_uniform_f32(
                        program_id.glow(),
                        &NativeUniformLocation(loc as u32),
                        &mut value,
                    );
                }
                sender.send(value).unwrap()
            },
            WebGLCommand::GetUniformFloat4x2(program_id, loc, ref sender) => {
                let mut value = [0.; 4 * 2];
                unsafe {
                    gl.get_uniform_f32(
                        program_id.glow(),
                        &NativeUniformLocation(loc as u32),
                        &mut value,
                    );
                }
                sender.send(value).unwrap()
            },
            WebGLCommand::GetUniformFloat4x3(program_id, loc, ref sender) => {
                let mut value = [0.; 4 * 3];
                unsafe {
                    gl.get_uniform_f32(
                        program_id.glow(),
                        &NativeUniformLocation(loc as u32),
                        &mut value,
                    );
                }
                sender.send(value).unwrap()
            },
            WebGLCommand::GetUniformBlockIndex(program_id, ref name, ref sender) => {
                let name = to_name_in_compiled_shader(name);
                let index = unsafe { gl.get_uniform_block_index(program_id.glow(), &name) };
                // TODO(#34300): use Option<u32>
                sender.send(index.unwrap_or(gl::INVALID_INDEX)).unwrap();
            },
            WebGLCommand::GetUniformIndices(program_id, ref names, ref sender) => {
                let names = names
                    .iter()
                    .map(|name| to_name_in_compiled_shader(name))
                    .collect::<Vec<_>>();
                let name_strs = names.iter().map(|name| name.as_str()).collect::<Vec<_>>();
                let indices = unsafe {
                    gl.get_uniform_indices(program_id.glow(), &name_strs)
                        .iter()
                        .map(|index| index.unwrap_or(gl::INVALID_INDEX))
                        .collect()
                };
                sender.send(indices).unwrap();
            },
            WebGLCommand::GetActiveUniforms(program_id, ref indices, pname, ref sender) => {
                let results =
                    unsafe { gl.get_active_uniforms_parameter(program_id.glow(), indices, pname) };
                sender.send(results).unwrap();
            },
            WebGLCommand::GetActiveUniformBlockName(program_id, block_idx, ref sender) => {
                let name =
                    unsafe { gl.get_active_uniform_block_name(program_id.glow(), block_idx) };
                sender.send(name).unwrap();
            },
            WebGLCommand::GetActiveUniformBlockParameter(
                program_id,
                block_idx,
                pname,
                ref sender,
            ) => {
                let size = match pname {
                    gl::UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES => unsafe {
                        gl.get_active_uniform_block_parameter_i32(
                            program_id.glow(),
                            block_idx,
                            gl::UNIFORM_BLOCK_ACTIVE_UNIFORMS,
                        ) as usize
                    },
                    _ => 1,
                };
                let mut result = vec![0; size];
                unsafe {
                    gl.get_active_uniform_block_parameter_i32_slice(
                        program_id.glow(),
                        block_idx,
                        pname,
                        &mut result,
                    )
                };
                sender.send(result).unwrap();
            },
            WebGLCommand::UniformBlockBinding(program_id, block_idx, block_binding) => unsafe {
                gl.uniform_block_binding(program_id.glow(), block_idx, block_binding)
            },
            WebGLCommand::InitializeFramebuffer {
                color,
                depth,
                stencil,
            } => Self::initialize_framebuffer(gl, state, color, depth, stencil),
            WebGLCommand::BeginQuery(target, query_id) => {
                unsafe { gl.begin_query(target, query_id.glow()) };
            },
            WebGLCommand::EndQuery(target) => {
                unsafe { gl.end_query(target) };
            },
            WebGLCommand::DeleteQuery(query_id) => {
                unsafe { gl.delete_query(query_id.glow()) };
            },
            WebGLCommand::GenerateQuery(ref sender) => {
                // TODO(#34300): use Option<WebGLQueryId>
                let id = unsafe { gl.create_query().unwrap() };
                sender.send(WebGLQueryId::from_glow(id)).unwrap()
            },
            WebGLCommand::GetQueryState(ref sender, query_id, pname) => {
                let value = unsafe { gl.get_query_parameter_u32(query_id.glow(), pname) };
                sender.send(value).unwrap()
            },
            WebGLCommand::GenerateSampler(ref sender) => {
                let id = unsafe { gl.create_sampler().unwrap() };
                sender.send(WebGLSamplerId::from_glow(id)).unwrap()
            },
            WebGLCommand::DeleteSampler(sampler_id) => {
                unsafe { gl.delete_sampler(sampler_id.glow()) };
            },
            WebGLCommand::BindSampler(unit, sampler_id) => {
                unsafe { gl.bind_sampler(unit, Some(sampler_id.glow())) };
            },
            WebGLCommand::SetSamplerParameterInt(sampler_id, pname, value) => {
                unsafe { gl.sampler_parameter_i32(sampler_id.glow(), pname, value) };
            },
            WebGLCommand::SetSamplerParameterFloat(sampler_id, pname, value) => {
                unsafe { gl.sampler_parameter_f32(sampler_id.glow(), pname, value) };
            },
            WebGLCommand::GetSamplerParameterInt(sampler_id, pname, ref sender) => {
                let value = unsafe { gl.get_sampler_parameter_i32(sampler_id.glow(), pname) };
                sender.send(value).unwrap();
            },
            WebGLCommand::GetSamplerParameterFloat(sampler_id, pname, ref sender) => {
                let value = unsafe { gl.get_sampler_parameter_f32(sampler_id.glow(), pname) };
                sender.send(value).unwrap();
            },
            WebGLCommand::BindBufferBase(target, index, id) => {
                // https://searchfox.org/mozilla-central/rev/13b081a62d3f3e3e3120f95564529257b0bf451c/dom/canvas/WebGLContextBuffers.cpp#208-210
                // BindBufferBase/Range will fail (on some drivers) if the buffer name has
                // never been bound. (GenBuffers makes a name, but BindBuffer initializes
                // that name as a real buffer object)
                let id = id.map(WebGLBufferId::glow);
                unsafe {
                    gl.bind_buffer(target, id);
                    gl.bind_buffer(target, None);
                    gl.bind_buffer_base(target, index, id);
                }
            },
            WebGLCommand::BindBufferRange(target, index, id, offset, size) => {
                // https://searchfox.org/mozilla-central/rev/13b081a62d3f3e3e3120f95564529257b0bf451c/dom/canvas/WebGLContextBuffers.cpp#208-210
                // BindBufferBase/Range will fail (on some drivers) if the buffer name has
                // never been bound. (GenBuffers makes a name, but BindBuffer initializes
                // that name as a real buffer object)
                let id = id.map(WebGLBufferId::glow);
                unsafe {
                    gl.bind_buffer(target, id);
                    gl.bind_buffer(target, None);
                    gl.bind_buffer_range(target, index, id, offset as i32, size as i32);
                }
            },
            WebGLCommand::ClearBufferfv(buffer, draw_buffer, ref value) => unsafe {
                gl.clear_buffer_f32_slice(buffer, draw_buffer as u32, value)
            },
            WebGLCommand::ClearBufferiv(buffer, draw_buffer, ref value) => unsafe {
                gl.clear_buffer_i32_slice(buffer, draw_buffer as u32, value)
            },
            WebGLCommand::ClearBufferuiv(buffer, draw_buffer, ref value) => unsafe {
                gl.clear_buffer_u32_slice(buffer, draw_buffer as u32, value)
            },
            WebGLCommand::ClearBufferfi(buffer, draw_buffer, depth, stencil) => unsafe {
                gl.clear_buffer_depth_stencil(buffer, draw_buffer as u32, depth, stencil)
            },
            WebGLCommand::InvalidateFramebuffer(target, ref attachments) => unsafe {
                gl.invalidate_framebuffer(target, attachments)
            },
            WebGLCommand::InvalidateSubFramebuffer(target, ref attachments, x, y, w, h) => unsafe {
                gl.invalidate_sub_framebuffer(target, attachments, x, y, w, h)
            },
            WebGLCommand::FramebufferTextureLayer(target, attachment, tex_id, level, layer) => {
                let tex_id = tex_id.map(WebGLTextureId::glow);
                let attach = |attachment| unsafe {
                    gl.framebuffer_texture_layer(target, attachment, tex_id, level, layer)
                };

                if attachment == gl::DEPTH_STENCIL_ATTACHMENT {
                    attach(gl::DEPTH_ATTACHMENT);
                    attach(gl::STENCIL_ATTACHMENT);
                } else {
                    attach(attachment)
                }
            },
            WebGLCommand::ReadBuffer(buffer) => unsafe { gl.read_buffer(buffer) },
            WebGLCommand::DrawBuffers(ref buffers) => unsafe { gl.draw_buffers(buffers) },
        }

        // If debug asertions are enabled, then check the error state.
        #[cfg(debug_assertions)]
        {
            let error = unsafe { gl.get_error() };
            if error != gl::NO_ERROR {
                error!("Last GL operation failed: {:?}", command);
                if error == gl::INVALID_FRAMEBUFFER_OPERATION {
                    let framebuffer_bindings =
                        unsafe { gl.get_parameter_framebuffer(gl::DRAW_FRAMEBUFFER_BINDING) };
                    debug!(
                        "(thread {:?}) Current draw framebuffer binding: {:?}",
                        ::std::thread::current().id(),
                        framebuffer_bindings
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

        unsafe {
            gl.disable(gl::SCISSOR_TEST);
            gl.color_mask(true, true, true, true);
            gl.clear_color(0., 0., 0., 0.);
            gl.depth_mask(true);
            gl.clear_depth(1.);
            gl.stencil_mask_separate(gl::FRONT, 0xFFFFFFFF);
            gl.stencil_mask_separate(gl::BACK, 0xFFFFFFFF);
            gl.clear_stencil(0);
            gl.clear(bits);
        }

        state.restore_invariant(gl);
    }

    fn link_program(gl: &Gl, program: WebGLProgramId) -> ProgramLinkInfo {
        unsafe { gl.link_program(program.glow()) };
        let linked = unsafe { gl.get_program_link_status(program.glow()) };
        if !linked {
            return ProgramLinkInfo {
                linked: false,
                active_attribs: vec![].into(),
                active_uniforms: vec![].into(),
                active_uniform_blocks: vec![].into(),
                transform_feedback_length: Default::default(),
                transform_feedback_mode: Default::default(),
            };
        }
        let num_active_attribs =
            unsafe { gl.get_program_parameter_i32(program.glow(), gl::ACTIVE_ATTRIBUTES) };
        let active_attribs = (0..num_active_attribs as u32)
            .map(|i| {
                let active_attribute =
                    unsafe { gl.get_active_attribute(program.glow(), i) }.unwrap();
                let name = &active_attribute.name;
                let location = if name.starts_with("gl_") {
                    None
                } else {
                    unsafe { gl.get_attrib_location(program.glow(), name) }
                };
                ActiveAttribInfo {
                    name: from_name_in_compiled_shader(name),
                    size: active_attribute.size,
                    type_: active_attribute.atype,
                    location,
                }
            })
            .collect::<Vec<_>>()
            .into();

        let num_active_uniforms =
            unsafe { gl.get_program_parameter_i32(program.glow(), gl::ACTIVE_UNIFORMS) };
        let active_uniforms = (0..num_active_uniforms as u32)
            .map(|i| {
                let active_uniform = unsafe { gl.get_active_uniform(program.glow(), i) }.unwrap();
                let is_array = active_uniform.name.ends_with("[0]");
                let active_uniform_name = active_uniform
                    .name
                    .strip_suffix("[0]")
                    .unwrap_or_else(|| &active_uniform.name);
                ActiveUniformInfo {
                    base_name: from_name_in_compiled_shader(active_uniform_name).into(),
                    size: if is_array {
                        Some(active_uniform.size)
                    } else {
                        None
                    },
                    type_: active_uniform.utype,
                    bind_index: None,
                }
            })
            .collect::<Vec<_>>()
            .into();

        let num_active_uniform_blocks =
            unsafe { gl.get_program_parameter_i32(program.glow(), gl::ACTIVE_UNIFORM_BLOCKS) };
        let active_uniform_blocks = (0..num_active_uniform_blocks as u32)
            .map(|i| {
                let name = unsafe { gl.get_active_uniform_block_name(program.glow(), i) };
                let size = unsafe {
                    gl.get_active_uniform_block_parameter_i32(
                        program.glow(),
                        i,
                        gl::UNIFORM_BLOCK_DATA_SIZE,
                    )
                };
                ActiveUniformBlockInfo { name, size }
            })
            .collect::<Vec<_>>()
            .into();

        let transform_feedback_length = unsafe {
            gl.get_program_parameter_i32(program.glow(), gl::TRANSFORM_FEEDBACK_VARYINGS)
        };
        let transform_feedback_mode = unsafe {
            gl.get_program_parameter_i32(program.glow(), gl::TRANSFORM_FEEDBACK_BUFFER_MODE)
        };

        ProgramLinkInfo {
            linked: true,
            active_attribs,
            active_uniforms,
            active_uniform_blocks,
            transform_feedback_length,
            transform_feedback_mode,
        }
    }

    fn finish(gl: &Gl, chan: &WebGLSender<()>) {
        unsafe { gl.finish() };
        chan.send(()).unwrap();
    }

    fn shader_precision_format(
        gl: &Gl,
        shader_type: u32,
        precision_type: u32,
        chan: &WebGLSender<(i32, i32, i32)>,
    ) {
        let ShaderPrecisionFormat {
            range_min,
            range_max,
            precision,
        } = unsafe {
            gl.get_shader_precision_format(shader_type, precision_type)
                .unwrap_or_else(|| {
                    ShaderPrecisionFormat::common_desktop_hardware(
                        precision_type,
                        gl.version().is_embedded,
                    )
                })
        };
        chan.send((range_min, range_max, precision)).unwrap();
    }

    /// This is an implementation of `getSupportedExtensions()` from
    /// <https://registry.khronos.org/webgl/specs/latest/1.0/#5.14>
    fn get_extensions(gl: &Gl, result_sender: &WebGLSender<String>) {
        let _ = result_sender.send(gl.supported_extensions().iter().join(" "));
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.6
    fn get_framebuffer_attachment_parameter(
        gl: &Gl,
        target: u32,
        attachment: u32,
        pname: u32,
        chan: &WebGLSender<i32>,
    ) {
        let parameter =
            unsafe { gl.get_framebuffer_attachment_parameter_i32(target, attachment, pname) };
        chan.send(parameter).unwrap();
    }

    // https://www.khronos.org/registry/webgl/specs/latest/1.0/#5.14.7
    fn get_renderbuffer_parameter(gl: &Gl, target: u32, pname: u32, chan: &WebGLSender<i32>) {
        let parameter = unsafe { gl.get_renderbuffer_parameter_i32(target, pname) };
        chan.send(parameter).unwrap();
    }

    fn uniform_location(gl: &Gl, program_id: WebGLProgramId, name: &str, chan: &WebGLSender<i32>) {
        let location = unsafe {
            gl.get_uniform_location(program_id.glow(), &to_name_in_compiled_shader(name))
        };
        // (#34300): replace this with WebGLUniformId
        chan.send(location.map(|l| l.0).unwrap_or_default() as i32)
            .unwrap();
    }

    fn shader_info_log(gl: &Gl, shader_id: WebGLShaderId, chan: &WebGLSender<String>) {
        let log = unsafe { gl.get_shader_info_log(shader_id.glow()) };
        chan.send(log).unwrap();
    }

    fn program_info_log(gl: &Gl, program_id: WebGLProgramId, chan: &WebGLSender<String>) {
        let log = unsafe { gl.get_program_info_log(program_id.glow()) };
        chan.send(log).unwrap();
    }

    fn create_buffer(gl: &Gl, chan: &WebGLSender<Option<WebGLBufferId>>) {
        let buffer = unsafe { gl.create_buffer() }
            .ok()
            .map(WebGLBufferId::from_glow);
        chan.send(buffer).unwrap();
    }

    fn create_framebuffer(gl: &Gl, chan: &WebGLSender<Option<WebGLFramebufferId>>) {
        let framebuffer = unsafe { gl.create_framebuffer() }
            .ok()
            .map(WebGLFramebufferId::from_glow);
        chan.send(framebuffer).unwrap();
    }

    fn create_renderbuffer(gl: &Gl, chan: &WebGLSender<Option<WebGLRenderbufferId>>) {
        let renderbuffer = unsafe { gl.create_renderbuffer() }
            .ok()
            .map(WebGLRenderbufferId::from_glow);
        chan.send(renderbuffer).unwrap();
    }

    fn create_texture(gl: &Gl, chan: &WebGLSender<Option<WebGLTextureId>>) {
        let texture = unsafe { gl.create_texture() }
            .ok()
            .map(WebGLTextureId::from_glow);
        chan.send(texture).unwrap();
    }

    fn create_program(gl: &Gl, chan: &WebGLSender<Option<WebGLProgramId>>) {
        let program = unsafe { gl.create_program() }
            .ok()
            .map(WebGLProgramId::from_glow);
        chan.send(program).unwrap();
    }

    fn create_shader(gl: &Gl, shader_type: u32, chan: &WebGLSender<Option<WebGLShaderId>>) {
        let shader = unsafe { gl.create_shader(shader_type) }
            .ok()
            .map(WebGLShaderId::from_glow);
        chan.send(shader).unwrap();
    }

    fn create_vertex_array(gl: &Gl) -> Option<WebGLVertexArrayId> {
        let vao = unsafe { gl.create_vertex_array() }
            .ok()
            .map(WebGLVertexArrayId::from_glow);
        if vao.is_none() {
            let code = unsafe { gl.get_error() };
            warn!("Failed to create vertex array with error code {:x}", code);
        }
        vao
    }

    fn bind_vertex_array(gl: &Gl, vao: Option<NativeVertexArray>) {
        unsafe { gl.bind_vertex_array(vao) }
        debug_assert_eq!(unsafe { gl.get_error() }, gl::NO_ERROR);
    }

    fn delete_vertex_array(gl: &Gl, vao: WebGLVertexArrayId) {
        unsafe { gl.delete_vertex_array(vao.glow()) };
        debug_assert_eq!(unsafe { gl.get_error() }, gl::NO_ERROR);
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
            WebGLFramebufferBindingRequest::Explicit(id) => Some(id.glow()),
            WebGLFramebufferBindingRequest::Default => {
                device
                    .context_surface_info(ctx)
                    .unwrap()
                    .expect("No surface attached!")
                    .framebuffer_object
            },
        };

        debug!("WebGLImpl::bind_framebuffer: {:?}", id);
        unsafe { gl.bind_framebuffer(target, id) };

        if (target == gl::FRAMEBUFFER) || (target == gl::DRAW_FRAMEBUFFER) {
            state.drawing_to_default_framebuffer =
                request == WebGLFramebufferBindingRequest::Default;
            state.restore_invariant(gl);
        }
    }

    #[inline]
    fn compile_shader(gl: &Gl, shader_id: WebGLShaderId, source: &str) {
        unsafe {
            gl.shader_source(shader_id.glow(), source);
            gl.compile_shader(shader_id.glow());
        }
    }
}

/// ANGLE adds a `_u` prefix to variable names:
///
/// <https://chromium.googlesource.com/angle/angle/+/855d964bd0d05f6b2cb303f625506cf53d37e94f>
///
/// To avoid hard-coding this we would need to use the `sh::GetAttributes` and `sh::GetUniforms`
/// API to look up the `x.name` and `x.mappedName` members.
const ANGLE_NAME_PREFIX: &str = "_u";

/// Adds `_u` prefix to variable names
fn to_name_in_compiled_shader(s: &str) -> String {
    map_dot_separated(s, |s, mapped| {
        mapped.push_str(ANGLE_NAME_PREFIX);
        mapped.push_str(s);
    })
}

/// Removes `_u` prefix from variable names
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
            unmultiply_inplace::<false>(pixels.to_mut());
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
                    ((rgba[0] as u16 & 0xf0) << 8) |
                        ((rgba[1] as u16 & 0xf0) << 4) |
                        (rgba[2] as u16 & 0xf0) |
                        ((rgba[3] as u16 & 0xf0) >> 4)
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
                    ((rgba[0] as u16 & 0xf8) << 8) |
                        ((rgba[1] as u16 & 0xf8) << 3) |
                        ((rgba[2] as u16 & 0xf8) >> 2) |
                        ((rgba[3] as u16) >> 7)
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
                    ((rgb[0] as u16 & 0xf8) << 8) |
                        ((rgb[1] as u16 & 0xfc) << 3) |
                        ((rgb[2] as u16 & 0xf8) >> 3)
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
                let extend_to_8_bits = |val| (val | (val << 4)) as u8;
                let r = extend_to_8_bits((pix >> 12) & 0x0f);
                let g = extend_to_8_bits((pix >> 8) & 0x0f);
                let b = extend_to_8_bits((pix >> 4) & 0x0f);
                let a = extend_to_8_bits(pix & 0x0f);
                NativeEndian::write_u16(
                    rgba,
                    (((pixels::multiply_u8_color(r, a) & 0xf0) as u16) << 8) |
                        (((pixels::multiply_u8_color(g, a) & 0xf0) as u16) << 4) |
                        ((pixels::multiply_u8_color(b, a) & 0xf0) as u16) |
                        ((a & 0x0f) as u16),
                );
            }
        },
        // Other formats don't have alpha, so return their data untouched.
        _ => {},
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
    let mut max_viewport = [i32::MAX, i32::MAX];
    let mut max_renderbuffer = [i32::MAX];

    unsafe {
        gl.get_parameter_i32_slice(gl::MAX_VIEWPORT_DIMS, &mut max_viewport);
        gl.get_parameter_i32_slice(gl::MAX_RENDERBUFFER_SIZE, &mut max_renderbuffer);
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
    fn to_surfman_version(self, api_type: GlType) -> GLVersion;
}

impl ToSurfmanVersion for WebGLVersion {
    fn to_surfman_version(self, api_type: GlType) -> GLVersion {
        if api_type == GlType::Gles {
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
        api_type: GlType,
    ) -> ContextAttributeFlags;
}

impl SurfmanContextAttributeFlagsConvert for GLContextAttributes {
    fn to_surfman_context_attribute_flags(
        &self,
        webgl_version: WebGLVersion,
        api_type: GlType,
    ) -> ContextAttributeFlags {
        let mut flags = ContextAttributeFlags::empty();
        flags.set(ContextAttributeFlags::ALPHA, self.alpha);
        flags.set(ContextAttributeFlags::DEPTH, self.depth);
        flags.set(ContextAttributeFlags::STENCIL, self.stencil);
        if (webgl_version == WebGLVersion::WebGL1) && (api_type == GlType::Gl) {
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
    fn detect(device: &Device, context: &Context, gl: &Gl) -> FramebufferRebindingInfo {
        unsafe {
            let read_framebuffer = gl.get_parameter_framebuffer(gl::READ_FRAMEBUFFER_BINDING);
            let draw_framebuffer = gl.get_parameter_framebuffer(gl::DRAW_FRAMEBUFFER_BINDING);

            let context_surface_framebuffer = device
                .context_surface_info(context)
                .unwrap()
                .unwrap()
                .framebuffer_object;

            let mut flags = FramebufferRebindingFlags::empty();
            if context_surface_framebuffer == read_framebuffer {
                flags.insert(FramebufferRebindingFlags::REBIND_READ_FRAMEBUFFER);
            }
            if context_surface_framebuffer == draw_framebuffer {
                flags.insert(FramebufferRebindingFlags::REBIND_DRAW_FRAMEBUFFER);
            }

            let mut viewport = [0; 4];
            gl.get_parameter_i32_slice(gl::VIEWPORT, &mut viewport);

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
            unsafe { gl.bind_framebuffer(gl::READ_FRAMEBUFFER, context_surface_framebuffer) };
        }
        if self
            .flags
            .contains(FramebufferRebindingFlags::REBIND_DRAW_FRAMEBUFFER)
        {
            unsafe { gl.bind_framebuffer(gl::DRAW_FRAMEBUFFER, context_surface_framebuffer) };
        }

        unsafe {
            gl.viewport(
                self.viewport[0],
                self.viewport[1],
                self.viewport[2],
                self.viewport[3],
            )
        };
    }
}
