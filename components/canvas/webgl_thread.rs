/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::webgl_limits::GLLimitsDetect;
use byteorder::{ByteOrder, NativeEndian, WriteBytesExt};
use canvas_traits::webgl;
use canvas_traits::webgl::ActiveAttribInfo;
use canvas_traits::webgl::ActiveUniformBlockInfo;
use canvas_traits::webgl::ActiveUniformInfo;
use canvas_traits::webgl::AlphaTreatment;
use canvas_traits::webgl::DOMToTextureCommand;
use canvas_traits::webgl::GLContextAttributes;
use canvas_traits::webgl::GLLimits;
use canvas_traits::webgl::GlType;
use canvas_traits::webgl::ProgramLinkInfo;
use canvas_traits::webgl::SwapChainId;
use canvas_traits::webgl::TexDataType;
use canvas_traits::webgl::TexFormat;
use canvas_traits::webgl::WebGLBufferId;
use canvas_traits::webgl::WebGLChan;
use canvas_traits::webgl::WebGLCommand;
use canvas_traits::webgl::WebGLCommandBacktrace;
use canvas_traits::webgl::WebGLContextId;
use canvas_traits::webgl::WebGLCreateContextResult;
use canvas_traits::webgl::WebGLFramebufferBindingRequest;
use canvas_traits::webgl::WebGLFramebufferId;
use canvas_traits::webgl::WebGLMsg;
use canvas_traits::webgl::WebGLMsgSender;
use canvas_traits::webgl::WebGLOpaqueFramebufferId;
use canvas_traits::webgl::WebGLProgramId;
use canvas_traits::webgl::WebGLQueryId;
use canvas_traits::webgl::WebGLReceiver;
use canvas_traits::webgl::WebGLRenderbufferId;
use canvas_traits::webgl::WebGLSLVersion;
use canvas_traits::webgl::WebGLSamplerId;
use canvas_traits::webgl::WebGLSender;
use canvas_traits::webgl::WebGLShaderId;
use canvas_traits::webgl::WebGLSyncId;
use canvas_traits::webgl::WebGLTextureId;
use canvas_traits::webgl::WebGLTransparentFramebufferId;
use canvas_traits::webgl::WebGLVersion;
use canvas_traits::webgl::WebGLVertexArrayId;
use canvas_traits::webgl::WebVRCommand;
use canvas_traits::webgl::WebVRRenderHandler;
use canvas_traits::webgl::YAxisTreatment;
use euclid::default::Size2D;
use fnv::FnvHashMap;
use half::f16;
use pixels::{self, PixelFormat};
use servo_config::opts;
use sparkle::gl;
use sparkle::gl::GLint;
use sparkle::gl::GLuint;
use sparkle::gl::Gl;
use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::slice;
use std::sync::{Arc, Mutex};
use std::thread;
use surfman;
use surfman::platform::generic::universal::adapter::Adapter;
use surfman::platform::generic::universal::connection::Connection;
use surfman::platform::generic::universal::context::Context;
use surfman::platform::generic::universal::device::Device;
use surfman::ContextAttributeFlags;
use surfman::ContextAttributes;
use surfman::GLVersion;
use surfman::SurfaceAccess;
use surfman::SurfaceInfo;
use surfman::SurfaceType;
use surfman_chains::SwapChains;
use surfman_chains_api::SwapChainsAPI;
use webrender_traits::{WebrenderExternalImageRegistry, WebrenderImageHandlerType};
use webxr_api::SwapChainId as WebXRSwapChainId;

#[cfg(feature = "xr-profile")]
fn to_ms(ns: u64) -> f64 {
    ns as f64 / 1_000_000.
}

struct GLContextData {
    ctx: Context,
    gl: Rc<Gl>,
    state: GLState,
    attributes: GLContextAttributes,
}

pub struct GLState {
    webgl_version: WebGLVersion,
    gl_version: GLVersion,
    clear_color: (f32, f32, f32, f32),
    scissor_test_enabled: bool,
    stencil_write_mask: (u32, u32),
    stencil_clear_value: i32,
    depth_write_mask: bool,
    depth_clear_value: f64,
    default_vao: gl::GLuint,
}

impl Default for GLState {
    fn default() -> GLState {
        GLState {
            gl_version: GLVersion { major: 1, minor: 0 },
            webgl_version: WebGLVersion::WebGL1,
            clear_color: (0., 0., 0., 0.),
            scissor_test_enabled: false,
            stencil_write_mask: (0, 0),
            stencil_clear_value: 0,
            depth_write_mask: true,
            depth_clear_value: 1.,
            default_vao: 0,
        }
    }
}

/// A WebGLThread manages the life cycle and message multiplexing of
/// a set of WebGLContexts living in the same thread.
pub(crate) struct WebGLThread {
    /// The GPU device.
    device: Device,
    /// Channel used to generate/update or delete `webrender_api::ImageKey`s.
    webrender_api: webrender_api::RenderApi,
    /// Map of live WebGLContexts.
    contexts: FnvHashMap<WebGLContextId, GLContextData>,
    /// Cached information for WebGLContexts.
    cached_context_info: FnvHashMap<WebGLContextId, WebGLContextInfo>,
    /// Current bound context.
    bound_context_id: Option<WebGLContextId>,
    /// Handler user to send WebVR commands.
    // TODO: replace webvr implementation with one built on top of webxr
    #[allow(dead_code)]
    webvr_compositor: Option<Box<dyn WebVRRenderHandler>>,
    /// Texture ids and sizes used in DOM to texture outputs.
    dom_outputs: FnvHashMap<webrender_api::PipelineId, DOMToTextureData>,
    /// List of registered webrender external images.
    /// We use it to get an unique ID for new WebGLContexts.
    external_images: Arc<Mutex<WebrenderExternalImageRegistry>>,
    /// The receiver that will be used for processing WebGL messages.
    receiver: WebGLReceiver<WebGLMsg>,
    /// The receiver that should be used to send WebGL messages for processing.
    sender: WebGLSender<WebGLMsg>,
    /// The swap chains used by webrender
    webrender_swap_chains: SwapChains<WebGLContextId>,
    /// The swap chains used by webxr
    webxr_swap_chains: SwapChains<WebXRSwapChainId>,
    /// Whether this context is a GL or GLES context.
    api_type: gl::GlType,
}

#[derive(PartialEq)]
enum EventLoop {
    Blocking,
    Nonblocking,
}

/// The data required to initialize an instance of the WebGLThread type.
pub(crate) struct WebGLThreadInit {
    pub webrender_api_sender: webrender_api::RenderApiSender,
    pub webvr_compositor: Option<Box<dyn WebVRRenderHandler>>,
    pub external_images: Arc<Mutex<WebrenderExternalImageRegistry>>,
    pub sender: WebGLSender<WebGLMsg>,
    pub receiver: WebGLReceiver<WebGLMsg>,
    pub webrender_swap_chains: SwapChains<WebGLContextId>,
    pub webxr_swap_chains: SwapChains<WebXRSwapChainId>,
    pub connection: Connection,
    pub adapter: Adapter,
    pub api_type: gl::GlType,
}

/// The extra data required to run an instance of WebGLThread when it is
/// not running in its own thread.
pub struct WebGLMainThread {
    pub(crate) thread_data: RefCell<WebGLThread>,
    shut_down: Cell<bool>,
}

impl WebGLMainThread {
    /// Synchronously process all outstanding WebGL messages.
    pub fn process(&self) {
        if self.shut_down.get() {
            return;
        }

        // Any context could be current when we start.
        self.thread_data.borrow_mut().bound_context_id = None;
        let result = self
            .thread_data
            .borrow_mut()
            .process(EventLoop::Nonblocking);
        if !result {
            self.shut_down.set(true);
            WEBGL_MAIN_THREAD.with(|thread_data| thread_data.borrow_mut().take());
        }
    }
}

thread_local! {
    static WEBGL_MAIN_THREAD: RefCell<Option<Rc<WebGLMainThread>>> = RefCell::new(None);
}

// A size at which it should be safe to create GL contexts
const SAFE_VIEWPORT_DIMS: [u32; 2] = [1024, 1024];

impl WebGLThread {
    /// Create a new instance of WebGLThread.
    pub(crate) fn new(
        WebGLThreadInit {
            webrender_api_sender,
            webvr_compositor,
            external_images,
            sender,
            receiver,
            webrender_swap_chains,
            webxr_swap_chains,
            connection,
            adapter,
            api_type,
        }: WebGLThreadInit,
    ) -> Self {
        WebGLThread {
            device: Device::new(&connection, &adapter).expect("Couldn't open WebGL device!"),
            webrender_api: webrender_api_sender.create_api(),
            contexts: Default::default(),
            cached_context_info: Default::default(),
            bound_context_id: None,
            webvr_compositor,
            dom_outputs: Default::default(),
            external_images,
            sender,
            receiver,
            webrender_swap_chains,
            webxr_swap_chains,
            api_type,
        }
    }

    /// Perform all initialization required to run an instance of WebGLThread
    /// in parallel on its own dedicated thread.
    pub(crate) fn run_on_own_thread(init: WebGLThreadInit) {
        thread::Builder::new()
            .name("WebGL thread".to_owned())
            .spawn(move || {
                let mut data = WebGLThread::new(init);
                data.process(EventLoop::Blocking);
            })
            .expect("Thread spawning failed");
    }

    fn process(&mut self, loop_type: EventLoop) -> bool {
        let webgl_chan = WebGLChan(self.sender.clone());
        while let Ok(msg) = match loop_type {
            EventLoop::Blocking => self.receiver.recv(),
            EventLoop::Nonblocking => self.receiver.try_recv(),
        } {
            let exit = self.handle_msg(msg, &webgl_chan);
            if exit {
                return false;
            }
        }
        true
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
                        let glsl_version = Self::get_glsl_version(&*data.gl);
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
            WebGLMsg::CreateWebXRSwapChain(ctx_id, size, sender) => {
                let _ = sender.send(self.create_webxr_swap_chain(ctx_id, size));
            },
            WebGLMsg::SwapBuffers(swap_ids, sender, sent_time) => {
                self.handle_swap_buffers(swap_ids, sender, sent_time);
            },
            WebGLMsg::DOMToTextureCommand(command) => {
                self.handle_dom_to_texture(command);
            },
            WebGLMsg::Exit => {
                return true;
            },
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
            match command {
                // We have to handle framebuffer binding differently, because `apply`
                // assumes that the currently attached surface is the right one for binding
                // the framebuffer, and since it doesn't get passed the swap buffers
                // it casn't do that itself. At some point we could refactor apply so
                // it takes a self parameter, at which point that won't be necessary.
                WebGLCommand::BindFramebuffer(_, request) => {
                    WebGLImpl::attach_surface(
                        context_id,
                        &self.webrender_swap_chains,
                        &self.webxr_swap_chains,
                        request,
                        &mut data.ctx,
                        &mut self.device,
                    );
                },
                // Similarly, dropping a WebGL framebuffer needs access to the swap chains,
                // in order to delete the entry.
                WebGLCommand::DeleteFramebuffer(WebGLFramebufferId::Opaque(
                    WebGLOpaqueFramebufferId::WebXR(id),
                )) => {
                    let _ = self
                        .webxr_swap_chains
                        .destroy(id, &mut self.device, &mut data.ctx);
                },
                _ => {},
            }

            WebGLImpl::apply(
                &self.device,
                &data.ctx,
                &*data.gl,
                &mut data.state,
                &data.attributes,
                command,
                backtrace,
            );
        }
    }

    /// Handles a WebVRCommand for a specific WebGLContext
    fn handle_webvr_command(&mut self, _context_id: WebGLContextId, _command: WebVRCommand) {
        // TODO(pcwalton): Reenable.
    }

    /// Creates a new WebGLContext
    #[allow(unsafe_code)]
    fn create_webgl_context(
        &mut self,
        webgl_version: WebGLVersion,
        requested_size: Size2D<u32>,
        attributes: GLContextAttributes,
    ) -> Result<(WebGLContextId, webgl::GLLimits), String> {
        debug!("WebGLThread::create_webgl_context({:?})", requested_size);

        // Creating a new GLContext may make the current bound context_id dirty.
        // Clear it to ensure that  make_current() is called in subsequent commands.
        self.bound_context_id = None;

        let context_attributes = &ContextAttributes {
            version: webgl_version.to_surfman_version(),
            flags: attributes.to_surfman_context_attribute_flags(webgl_version),
        };

        let context_descriptor = self
            .device
            .create_context_descriptor(&context_attributes)
            .unwrap();

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
            .create_context(&context_descriptor)
            .expect("Failed to create the GL context!");
        let surface = self
            .device
            .create_surface(&ctx, surface_access, &surface_type)
            .expect("Failed to create the initial surface!");
        self.device
            .bind_surface_to_context(&mut ctx, surface)
            .unwrap();
        // https://github.com/pcwalton/surfman/issues/7
        self.device
            .make_context_current(&ctx)
            .expect("failed to make new context current");

        let id = WebGLContextId(
            self.external_images
                .lock()
                .unwrap()
                .next_id(WebrenderImageHandlerType::WebGL)
                .0,
        );

        self.webrender_swap_chains
            .create_attached_swap_chain(id, &mut self.device, &mut ctx, surface_access)
            .expect("Failed to create the swap chain");

        let swap_chain = self
            .webrender_swap_chains
            .get(id)
            .expect("Failed to get the swap chain");

        debug!(
            "Created webgl context {:?}/{:?}",
            id,
            self.device.context_id(&ctx)
        );

        let gl = match self.api_type {
            gl::GlType::Gl => Gl::gl_fns(gl::ffi_gl::Gl::load_with(|symbol_name| {
                self.device.get_proc_address(&ctx, symbol_name)
            })),
            gl::GlType::Gles => Gl::gles_fns(gl::ffi_gles::Gles2::load_with(|symbol_name| {
                self.device.get_proc_address(&ctx, symbol_name)
            })),
        };

        let limits = GLLimits::detect(&*gl, webgl_version);

        let size = clamp_viewport(&gl, requested_size);
        if safe_size != size {
            debug!("Resizing swap chain from {} to {}", safe_size, size);
            swap_chain
                .resize(&mut self.device, &mut ctx, size.to_i32())
                .expect("Failed to resize swap chain");
        }

        self.device.make_context_current(&ctx).unwrap();
        let framebuffer = self
            .device
            .context_surface_info(&ctx)
            .unwrap()
            .unwrap()
            .framebuffer_object;
        gl.bind_framebuffer(gl::FRAMEBUFFER, framebuffer);
        gl.viewport(0, 0, size.width as i32, size.height as i32);
        gl.scissor(0, 0, size.width as i32, size.height as i32);
        gl.clear_color(0., 0., 0., 0.);
        gl.clear_depth(1.);
        gl.clear_stencil(0);
        gl.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
        debug_assert_eq!(gl.get_error(), gl::NO_ERROR);

        let descriptor = self.device.context_descriptor(&ctx);
        let descriptor_attributes = self.device.context_descriptor_attributes(&descriptor);

        let gl_version = descriptor_attributes.version;
        let has_alpha = descriptor_attributes
            .flags
            .contains(ContextAttributeFlags::ALPHA);
        let texture_target = current_wr_texture_target(&self.device);

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
            default_vao,
            ..Default::default()
        };
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
            &self.webrender_api,
            size.to_i32(),
            has_alpha,
            id,
            texture_target,
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
        sender: WebGLSender<Result<(), String>>,
    ) {
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
            FramebufferRebindingInfo::detect(&self.device, &data.ctx, &*data.gl);

        // Resize the swap chains
        if let Some(swap_chain) = self.webrender_swap_chains.get(context_id) {
            swap_chain
                .resize(&mut self.device, &mut data.ctx, size.to_i32())
                .expect("Failed to resize swap chain");
            // temporary, till https://github.com/pcwalton/surfman/issues/35 is fixed
            self.device
                .make_context_current(&data.ctx)
                .expect("Failed to make context current again");
            swap_chain
                .clear_surface(&mut self.device, &mut data.ctx, &*data.gl)
                .expect("Failed to clear resized swap chain");
        } else {
            error!("Failed to find swap chain");
        }

        // Reset framebuffer bindings as appropriate.
        framebuffer_rebinding_info.apply(&self.device, &data.ctx, &*data.gl);

        // Update WR image if needed.
        let info = self.cached_context_info.get_mut(&context_id).unwrap();
        let context_descriptor = self.device.context_descriptor(&data.ctx);
        let has_alpha = self
            .device
            .context_descriptor_attributes(&context_descriptor)
            .flags
            .contains(ContextAttributeFlags::ALPHA);
        let texture_target = current_wr_texture_target(&self.device);
        Self::update_wr_external_image(
            &self.webrender_api,
            size.to_i32(),
            has_alpha,
            context_id,
            info.image_key,
            texture_target,
        );

        debug_assert_eq!(data.gl.get_error(), gl::NO_ERROR);

        sender.send(Ok(())).unwrap();
    }

    /// Removes a WebGLContext and releases attached resources.
    fn remove_webgl_context(&mut self, context_id: WebGLContextId) {
        // Release webrender image keys.
        if let Some(info) = self.cached_context_info.remove(&context_id) {
            let mut txn = webrender_api::Transaction::new();
            txn.delete_image(info.image_key);
            self.webrender_api.update_resources(txn.resource_updates)
        }

        // We need to make the context current so its resources can be disposed of.
        Self::make_current_if_needed(
            &self.device,
            context_id,
            &self.contexts,
            &mut self.bound_context_id,
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
        self.webxr_swap_chains
            .destroy_all(&mut self.device, &mut data.ctx)
            .unwrap();

        // Destroy the context
        self.device.destroy_context(&mut data.ctx).unwrap();

        // Removing a GLContext may make the current bound context_id dirty.
        self.bound_context_id = None;
    }

    fn handle_swap_buffers(
        &mut self,
        swap_ids: Vec<SwapChainId>,
        completed_sender: WebGLSender<u64>,
        _sent_time: u64,
    ) {
        #[cfg(feature = "xr-profile")]
        let start_swap = time::precise_time_ns();
        #[cfg(feature = "xr-profile")]
        println!(
            "WEBXR PROFILING [swap request]:\t{}ms",
            to_ms(start_swap - _sent_time)
        );
        debug!("handle_swap_buffers()");
        for swap_id in swap_ids {
            let context_id = swap_id.context_id();

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
                FramebufferRebindingInfo::detect(&self.device, &data.ctx, &*data.gl);
            debug_assert_eq!(data.gl.get_error(), gl::NO_ERROR);

            debug!("Getting swap chain for {:?}", swap_id);
            let swap_chain = match swap_id {
                SwapChainId::Context(id) => self.webrender_swap_chains.get(id),
                SwapChainId::Framebuffer(_, WebGLOpaqueFramebufferId::WebXR(id)) => {
                    self.webxr_swap_chains.get(id)
                },
            }
            .expect("Where's the swap chain?");

            debug!("Swapping {:?}", swap_id);
            swap_chain
                .swap_buffers(&mut self.device, &mut data.ctx)
                .unwrap();
            debug_assert_eq!(data.gl.get_error(), gl::NO_ERROR);

            // TODO: if preserveDrawingBuffer is true, then blit the front buffer to the back buffer
            // https://github.com/servo/servo/issues/24604
            debug!("Clearing {:?}", swap_id);
            swap_chain
                .clear_surface(&mut self.device, &mut data.ctx, &*data.gl)
                .unwrap();
            debug_assert_eq!(data.gl.get_error(), gl::NO_ERROR);

            // Rebind framebuffers as appropriate.
            debug!("Rebinding {:?}", swap_id);
            framebuffer_rebinding_info.apply(&self.device, &data.ctx, &*data.gl);
            debug_assert_eq!(data.gl.get_error(), gl::NO_ERROR);

            let SurfaceInfo {
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
        }

        #[allow(unused)]
        let mut end_swap = 0;
        #[cfg(feature = "xr-profile")]
        {
            end_swap = time::precise_time_ns();
            println!(
                "WEBXR PROFILING [swap buffer]:\t{}ms",
                to_ms(end_swap - start_swap)
            );
        }
        completed_sender.send(end_swap).unwrap();
    }

    /// Creates a new WebXR swap chain
    #[allow(unsafe_code)]
    fn create_webxr_swap_chain(
        &mut self,
        context_id: WebGLContextId,
        size: Size2D<i32>,
    ) -> Option<WebXRSwapChainId> {
        debug!("WebGLThread::create_webxr_swap_chain()");
        let id = WebXRSwapChainId::new();
        let surface_access = self.surface_access();
        let data = Self::make_current_if_needed_mut(
            &self.device,
            context_id,
            &mut self.contexts,
            &mut self.bound_context_id,
        )?;
        self.webxr_swap_chains
            .create_detached_swap_chain(id, size, &mut self.device, &mut data.ctx, surface_access)
            .ok()?;
        debug!("Created swap chain {:?}", id);
        Some(id)
    }

    /// Which access mode to use
    fn surface_access(&self) -> SurfaceAccess {
        SurfaceAccess::GPUOnly
    }

    fn handle_dom_to_texture(&mut self, command: DOMToTextureCommand) {
        match command {
            DOMToTextureCommand::Attach(context_id, texture_id, document_id, pipeline_id, size) => {
                let data = Self::make_current_if_needed(
                    &self.device,
                    context_id,
                    &self.contexts,
                    &mut self.bound_context_id,
                )
                .expect("WebGLContext not found in a WebGL DOMToTextureCommand::Attach command");
                // Initialize the texture that WR will use for frame outputs.
                data.gl.tex_image_2d(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGBA as gl::GLint,
                    size.width,
                    size.height,
                    0,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    None,
                );
                self.dom_outputs.insert(
                    pipeline_id,
                    DOMToTextureData {
                        context_id,
                        texture_id,
                        document_id,
                        size,
                    },
                );
                let mut txn = webrender_api::Transaction::new();
                txn.enable_frame_output(pipeline_id, true);
                self.webrender_api.send_transaction(document_id, txn);
            },
            DOMToTextureCommand::Lock(pipeline_id, gl_sync, sender) => {
                let result = self.handle_dom_to_texture_lock(pipeline_id, gl_sync);
                // Send the texture id and size to WR.
                sender.send(result).unwrap();
            },
            DOMToTextureCommand::Detach(texture_id) => {
                if let Some((pipeline_id, document_id)) = self
                    .dom_outputs
                    .iter()
                    .find(|&(_, v)| v.texture_id == texture_id)
                    .map(|(k, v)| (*k, v.document_id))
                {
                    let mut txn = webrender_api::Transaction::new();
                    txn.enable_frame_output(pipeline_id, false);
                    self.webrender_api.send_transaction(document_id, txn);
                    self.dom_outputs.remove(&pipeline_id);
                }
            },
        }
    }

    pub(crate) fn handle_dom_to_texture_lock(
        &mut self,
        pipeline_id: webrender_api::PipelineId,
        gl_sync: usize,
    ) -> Option<(u32, Size2D<i32>)> {
        let device = &self.device;
        let contexts = &self.contexts;
        let bound_context_id = &mut self.bound_context_id;
        self.dom_outputs.get(&pipeline_id).and_then(|dom_data| {
            let data = Self::make_current_if_needed(
                device,
                dom_data.context_id,
                contexts,
                bound_context_id,
            );
            data.and_then(|data| {
                // The next glWaitSync call is used to synchronize the two flows of
                // OpenGL commands (WR and WebGL) in order to avoid using semi-ready WR textures.
                // glWaitSync doesn't block WebGL CPU thread.
                data.gl
                    .wait_sync(gl_sync as gl::GLsync, 0, gl::TIMEOUT_IGNORED);
                Some((dom_data.texture_id.get(), dom_data.size))
            })
        })
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
        webrender_api: &webrender_api::RenderApi,
        size: Size2D<i32>,
        alpha: bool,
        context_id: WebGLContextId,
        target: webrender_api::TextureTarget,
    ) -> webrender_api::ImageKey {
        let descriptor = Self::image_descriptor(size, alpha);
        let data = Self::external_image_data(context_id, target);

        let image_key = webrender_api.generate_image_key();
        let mut txn = webrender_api::Transaction::new();
        txn.add_image(image_key, descriptor, data, None);
        webrender_api.update_resources(txn.resource_updates);

        image_key
    }

    /// Updates a `webrender_api::ImageKey` that uses shared textures.
    fn update_wr_external_image(
        webrender_api: &webrender_api::RenderApi,
        size: Size2D<i32>,
        alpha: bool,
        context_id: WebGLContextId,
        image_key: webrender_api::ImageKey,
        target: webrender_api::TextureTarget,
    ) {
        let descriptor = Self::image_descriptor(size, alpha);
        let data = Self::external_image_data(context_id, target);

        let mut txn = webrender_api::Transaction::new();
        txn.update_image(image_key, descriptor, data, &webrender_api::DirtyRect::All);
        webrender_api.update_resources(txn.resource_updates);
    }

    /// Helper function to create a `webrender_api::ImageDescriptor`.
    fn image_descriptor(size: Size2D<i32>, alpha: bool) -> webrender_api::ImageDescriptor {
        let mut flags = webrender_api::ImageDescriptorFlags::empty();
        flags.set(webrender_api::ImageDescriptorFlags::IS_OPAQUE, !alpha);
        webrender_api::ImageDescriptor {
            size: webrender_api::units::DeviceIntSize::new(size.width, size.height),
            stride: None,
            format: webrender_api::ImageFormat::BGRA8,
            offset: 0,
            flags,
        }
    }

    /// Helper function to create a `webrender_api::ImageData::External` instance.
    fn external_image_data(
        context_id: WebGLContextId,
        target: webrender_api::TextureTarget,
    ) -> webrender_api::ImageData {
        let data = webrender_api::ExternalImageData {
            id: webrender_api::ExternalImageId(context_id.0 as u64),
            channel_index: 0,
            image_type: webrender_api::ExternalImageType::TextureHandle(target),
        };
        webrender_api::ImageData::External(data)
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

impl Drop for WebGLThread {
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
    /// Currently used WebRender image key.
    image_key: webrender_api::ImageKey,
}

// TODO(pcwalton): Add `GL_TEXTURE_EXTERNAL_OES`?
fn current_wr_texture_target(device: &Device) -> webrender_api::TextureTarget {
    match device.surface_gl_texture_target() {
        gl::TEXTURE_RECTANGLE => webrender_api::TextureTarget::Rect,
        _ => webrender_api::TextureTarget::Default,
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
            WebGLCommand::ColorMask(r, g, b, a) => gl.color_mask(r, g, b, a),
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
                gl.depth_mask(flag);
            },
            WebGLCommand::DepthRange(near, far) => {
                gl.depth_range(near.max(0.).min(1.) as f64, far.max(0.).min(1.) as f64)
            },
            WebGLCommand::Disable(cap) => {
                if cap == gl::SCISSOR_TEST {
                    state.scissor_test_enabled = false;
                }
                gl.disable(cap);
            },
            WebGLCommand::Enable(cap) => {
                if cap == gl::SCISSOR_TEST {
                    state.scissor_test_enabled = true;
                }
                gl.enable(cap);
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
            WebGLCommand::LineWidth(width) => gl.line_width(width),
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
                gl.stencil_mask(mask);
            },
            WebGLCommand::StencilMaskSeparate(face, mask) => {
                if face == gl::FRONT {
                    state.stencil_write_mask.0 = mask;
                } else {
                    state.stencil_write_mask.1 = mask;
                }
                gl.stencil_mask_separate(face, mask);
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
            WebGLCommand::GetUniformLocation(program_id, ref name, ref chan) => {
                Self::uniform_location(gl, program_id, &name, chan)
            },
            WebGLCommand::GetShaderInfoLog(shader_id, ref chan) => {
                Self::shader_info_log(gl, shader_id, chan)
            },
            WebGLCommand::GetProgramInfoLog(program_id, ref chan) => {
                Self::program_info_log(gl, program_id, chan)
            },
            WebGLCommand::CompileShader(shader_id, ref source) => {
                Self::compile_shader(gl, shader_id, &source)
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
            WebGLCommand::DeleteFramebuffer(WebGLFramebufferId::Transparent(id)) => {
                gl.delete_framebuffers(&[id.get()])
            },
            WebGLCommand::DeleteFramebuffer(WebGLFramebufferId::Opaque(_)) => {},
            WebGLCommand::DeleteRenderbuffer(id) => gl.delete_renderbuffers(&[id.get()]),
            WebGLCommand::DeleteTexture(id) => gl.delete_textures(&[id.get()]),
            WebGLCommand::DeleteProgram(id) => gl.delete_program(id.get()),
            WebGLCommand::DeleteShader(id) => gl.delete_shader(id.get()),
            WebGLCommand::BindBuffer(target, id) => {
                gl.bind_buffer(target, id.map_or(0, WebGLBufferId::get))
            },
            WebGLCommand::BindFramebuffer(target, request) => {
                Self::bind_framebuffer(gl, target, request, ctx, device)
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
                effective_internal_format,
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
                    Cow::Borrowed(&*data),
                );

                gl.pixel_store_i(gl::UNPACK_ALIGNMENT, unpacking_alignment as i32);
                gl.tex_image_2d(
                    target,
                    level as i32,
                    effective_internal_format as i32,
                    size.width as i32,
                    size.height as i32,
                    0,
                    format.as_gl_constant(),
                    effective_data_type,
                    Some(&pixels),
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
                    Cow::Borrowed(&*data),
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
                    &*data,
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
                    level as i32,
                    xoffset as i32,
                    yoffset as i32,
                    size.width as i32,
                    size.height as i32,
                    format,
                    &*data,
                );
            },
            WebGLCommand::DrawingBufferWidth(ref sender) => {
                let size = device
                    .context_surface_info(&ctx)
                    .unwrap()
                    .expect("Where's the front buffer?")
                    .size;
                sender.send(size.width).unwrap()
            },
            WebGLCommand::DrawingBufferHeight(ref sender) => {
                let size = device
                    .context_surface_info(&ctx)
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
                let mut value = [0];
                unsafe {
                    gl.get_boolean_v(param as u32, &mut value);
                }
                sender.send(value[0] != 0).unwrap()
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
                let value = gl.client_wait_sync(sync_id.get() as *const _, flags, timeout as u64);
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
                let mut value = [0; 4];
                unsafe {
                    gl.get_boolean_v(param as u32, &mut value);
                }
                let value = [value[0] != 0, value[1] != 0, value[2] != 0, value[3] != 0];
                sender.send(value).unwrap()
            },
            WebGLCommand::GetParameterInt(param, ref sender) => {
                let mut value = [0];
                unsafe {
                    gl.get_integer_v(param as u32, &mut value);
                }
                sender.send(value[0]).unwrap()
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
            WebGLCommand::TexParameteri(target, param, value) => {
                gl.tex_parameter_i(target, param as u32, value)
            },
            WebGLCommand::TexParameterf(target, param, value) => {
                gl.tex_parameter_f(target, param as u32, value)
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
                gl.bind_buffer_base(target, index, id.map_or(0, WebGLBufferId::get))
            },
            WebGLCommand::BindBufferRange(target, index, id, offset, size) => gl.bind_buffer_range(
                target,
                index,
                id.map_or(0, WebGLBufferId::get),
                offset as isize,
                size as isize,
            ),
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
                panic!(
                    "Unexpected WebGL error: 0x{:x} ({}) [{:?}]",
                    error, error, command
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
        cfg!(target_os = "macos") && !opts::get().headless && gl_version.major < 3
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
        assert!(location >= 0);
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
    fn create_framebuffer(gl: &Gl, chan: &WebGLSender<Option<WebGLTransparentFramebufferId>>) {
        let framebuffer = gl.gen_framebuffers(1)[0];
        let framebuffer = if framebuffer == 0 {
            None
        } else {
            Some(unsafe { WebGLTransparentFramebufferId::new(framebuffer) })
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

    /// Updates the swap buffers if the context surface needs to be changed
    fn attach_surface(
        context_id: WebGLContextId,
        webrender_swap_chains: &SwapChains<WebGLContextId>,
        webxr_swap_chains: &SwapChains<WebXRSwapChainId>,
        request: WebGLFramebufferBindingRequest,
        ctx: &mut Context,
        device: &mut Device,
    ) -> Option<()> {
        debug!(
            "WebGLImpl::attach_surface({:?} in {:?})",
            request, context_id
        );
        let requested_framebuffer = match request {
            WebGLFramebufferBindingRequest::Explicit(WebGLFramebufferId::Opaque(id)) => Some(id),
            WebGLFramebufferBindingRequest::Explicit(WebGLFramebufferId::Transparent(_)) => {
                return None
            },
            WebGLFramebufferBindingRequest::Default => None,
        };
        let attached_framebuffer = webxr_swap_chains
            .iter(device, ctx)
            .filter_map(|(id, swap_chain)| {
                if swap_chain.is_attached() {
                    Some(id)
                } else {
                    None
                }
            })
            .map(WebGLOpaqueFramebufferId::WebXR)
            .next();
        if requested_framebuffer == attached_framebuffer {
            return None;
        }
        let requested_swap_chain = match requested_framebuffer {
            Some(WebGLOpaqueFramebufferId::WebXR(id)) => webxr_swap_chains.get(id)?,
            None => webrender_swap_chains.get(context_id)?,
        };
        let current_swap_chain = match attached_framebuffer {
            Some(WebGLOpaqueFramebufferId::WebXR(id)) => webxr_swap_chains.get(id)?,
            None => webrender_swap_chains.get(context_id)?,
        };
        requested_swap_chain
            .take_attachment_from(device, ctx, &current_swap_chain)
            .unwrap();
        Some(())
    }

    #[inline]
    fn bind_framebuffer(
        gl: &Gl,
        target: u32,
        request: WebGLFramebufferBindingRequest,
        ctx: &Context,
        device: &Device,
    ) {
        let id = match request {
            WebGLFramebufferBindingRequest::Explicit(WebGLFramebufferId::Transparent(id)) => {
                id.get()
            },
            WebGLFramebufferBindingRequest::Explicit(WebGLFramebufferId::Opaque(_)) |
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
    }

    #[inline]
    fn compile_shader(gl: &Gl, shader_id: WebGLShaderId, source: &str) {
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
        (TexFormat::RGBA, TexDataType::UnsignedByte) => pixels,
        (TexFormat::RGB, TexDataType::UnsignedByte) => {
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
        (TexFormat::RGBA, TexDataType::Float) => {
            let mut rgbaf32 = Vec::<u8>::with_capacity(pixel_count * 16);
            for rgba8 in pixels.chunks(4) {
                rgbaf32.write_f32::<NativeEndian>(rgba8[0] as f32).unwrap();
                rgbaf32.write_f32::<NativeEndian>(rgba8[1] as f32).unwrap();
                rgbaf32.write_f32::<NativeEndian>(rgba8[2] as f32).unwrap();
                rgbaf32.write_f32::<NativeEndian>(rgba8[3] as f32).unwrap();
            }
            rgbaf32
        },

        (TexFormat::RGB, TexDataType::Float) => {
            let mut rgbf32 = Vec::<u8>::with_capacity(pixel_count * 12);
            for rgba8 in pixels.chunks(4) {
                rgbf32.write_f32::<NativeEndian>(rgba8[0] as f32).unwrap();
                rgbf32.write_f32::<NativeEndian>(rgba8[1] as f32).unwrap();
                rgbf32.write_f32::<NativeEndian>(rgba8[2] as f32).unwrap();
            }
            rgbf32
        },

        (TexFormat::Alpha, TexDataType::Float) => {
            for rgba8 in pixels.chunks_mut(4) {
                let p = rgba8[3] as f32;
                NativeEndian::write_f32(rgba8, p);
            }
            pixels
        },

        (TexFormat::Luminance, TexDataType::Float) => {
            for rgba8 in pixels.chunks_mut(4) {
                let p = rgba8[0] as f32;
                NativeEndian::write_f32(rgba8, p);
            }
            pixels
        },

        (TexFormat::LuminanceAlpha, TexDataType::Float) => {
            let mut data = Vec::<u8>::with_capacity(pixel_count * 8);
            for rgba8 in pixels.chunks(4) {
                data.write_f32::<NativeEndian>(rgba8[0] as f32).unwrap();
                data.write_f32::<NativeEndian>(rgba8[3] as f32).unwrap();
            }
            data
        },

        (TexFormat::RGBA, TexDataType::HalfFloat) => {
            let mut rgbaf16 = Vec::<u8>::with_capacity(pixel_count * 8);
            for rgba8 in pixels.chunks(4) {
                rgbaf16
                    .write_u16::<NativeEndian>(f16::from_f32(rgba8[0] as f32).as_bits())
                    .unwrap();
                rgbaf16
                    .write_u16::<NativeEndian>(f16::from_f32(rgba8[1] as f32).as_bits())
                    .unwrap();
                rgbaf16
                    .write_u16::<NativeEndian>(f16::from_f32(rgba8[2] as f32).as_bits())
                    .unwrap();
                rgbaf16
                    .write_u16::<NativeEndian>(f16::from_f32(rgba8[3] as f32).as_bits())
                    .unwrap();
            }
            rgbaf16
        },

        (TexFormat::RGB, TexDataType::HalfFloat) => {
            let mut rgbf16 = Vec::<u8>::with_capacity(pixel_count * 6);
            for rgba8 in pixels.chunks(4) {
                rgbf16
                    .write_u16::<NativeEndian>(f16::from_f32(rgba8[0] as f32).as_bits())
                    .unwrap();
                rgbf16
                    .write_u16::<NativeEndian>(f16::from_f32(rgba8[1] as f32).as_bits())
                    .unwrap();
                rgbf16
                    .write_u16::<NativeEndian>(f16::from_f32(rgba8[2] as f32).as_bits())
                    .unwrap();
            }
            rgbf16
        },
        (TexFormat::Alpha, TexDataType::HalfFloat) => {
            for i in 0..pixel_count {
                let p = f16::from_f32(pixels[i * 4 + 3] as f32).as_bits();
                NativeEndian::write_u16(&mut pixels[i * 2..i * 2 + 2], p);
            }
            pixels.truncate(pixel_count * 2);
            pixels
        },
        (TexFormat::Luminance, TexDataType::HalfFloat) => {
            for i in 0..pixel_count {
                let p = f16::from_f32(pixels[i * 4] as f32).as_bits();
                NativeEndian::write_u16(&mut pixels[i * 2..i * 2 + 2], p);
            }
            pixels.truncate(pixel_count * 2);
            pixels
        },
        (TexFormat::LuminanceAlpha, TexDataType::HalfFloat) => {
            for rgba8 in pixels.chunks_mut(4) {
                let lum = f16::from_f32(rgba8[0] as f32).as_bits();
                let a = f16::from_f32(rgba8[3] as f32).as_bits();
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
    let mut max_size = [i32::max_value(), i32::max_value()];
    #[allow(unsafe_code)]
    unsafe {
        gl.get_integer_v(gl::MAX_VIEWPORT_DIMS, &mut max_size);
        debug_assert_eq!(gl.get_error(), gl::NO_ERROR);
    }
    Size2D::new(
        size.width.min(max_size[0] as u32).max(1),
        size.height.min(max_size[1] as u32).max(1),
    )
}

trait ToSurfmanVersion {
    fn to_surfman_version(self) -> GLVersion;
}

impl ToSurfmanVersion for WebGLVersion {
    fn to_surfman_version(self) -> GLVersion {
        match self {
            WebGLVersion::WebGL1 => GLVersion::new(2, 0),
            WebGLVersion::WebGL2 => GLVersion::new(3, 0),
        }
    }
}

trait SurfmanContextAttributeFlagsConvert {
    fn to_surfman_context_attribute_flags(
        &self,
        webgl_version: WebGLVersion,
    ) -> ContextAttributeFlags;
}

impl SurfmanContextAttributeFlagsConvert for GLContextAttributes {
    fn to_surfman_context_attribute_flags(
        &self,
        webgl_version: WebGLVersion,
    ) -> ContextAttributeFlags {
        let mut flags = ContextAttributeFlags::empty();
        flags.set(ContextAttributeFlags::ALPHA, self.alpha);
        flags.set(ContextAttributeFlags::DEPTH, self.depth);
        flags.set(ContextAttributeFlags::STENCIL, self.stencil);
        if webgl_version == WebGLVersion::WebGL1 {
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
