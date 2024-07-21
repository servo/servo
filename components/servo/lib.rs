/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Servo, the mighty web browser engine from the future.
//!
//! This is a very simple library that wires all of Servo's components
//! together as type `Servo`, along with a generic client
//! implementing the `WindowMethods` trait, to create a working web
//! browser.
//!
//! The `Servo` type is responsible for configuring a
//! `Constellation`, which does the heavy lifting of coordinating all
//! of Servo's internal subsystems, including the `ScriptThread` and the
//! `LayoutThread`, as well maintains the navigation context.
//!
//! `Servo` is fed events from a generic type that implements the
//! `WindowMethods` trait.

use std::borrow::{BorrowMut, Cow};
use std::cmp::max;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::vec::Drain;

pub use base::id::TopLevelBrowsingContextId;
use base::id::{PipelineNamespace, PipelineNamespaceId};
use bluetooth::BluetoothThreadFactory;
use bluetooth_traits::BluetoothRequest;
use canvas::canvas_paint_thread::{self, CanvasPaintThread};
use canvas::WebGLComm;
use canvas_traits::webgl::WebGLThreads;
use compositing::webview::UnknownWebView;
use compositing::windowing::{EmbedderEvent, EmbedderMethods, WindowMethods};
use compositing::{CompositeTarget, IOCompositor, InitialCompositorState, ShutdownState};
use compositing_traits::{
    CompositorMsg, CompositorProxy, CompositorReceiver, ConstellationMsg, ForwardedToCompositorMsg,
};
#[cfg(all(
    not(target_os = "windows"),
    not(target_os = "ios"),
    not(target_os = "android"),
    not(target_arch = "arm"),
    not(target_arch = "aarch64"),
    not(target_env = "ohos"),
))]
use constellation::content_process_sandbox_profile;
use constellation::{
    Constellation, FromCompositorLogger, FromScriptLogger, InitialConstellationState,
    UnprivilegedContent,
};
use crossbeam_channel::{unbounded, Sender};
use embedder_traits::{EmbedderMsg, EmbedderProxy, EmbedderReceiver, EventLoopWaker};
use env_logger::Builder as EnvLoggerBuilder;
use euclid::Scale;
use fonts::FontCacheThread;
#[cfg(all(
    not(target_os = "windows"),
    not(target_os = "ios"),
    not(target_os = "android"),
    not(target_arch = "arm"),
    not(target_arch = "aarch64"),
    not(target_env = "ohos"),
))]
use gaol::sandbox::{ChildSandbox, ChildSandboxMethods};
pub use gleam::gl;
use ipc_channel::ipc::{self, IpcSender};
#[cfg(feature = "layout_2013")]
pub use layout_thread_2013;
use log::{error, trace, warn, Log, Metadata, Record};
use media::{GLPlayerThreads, GlApi, NativeDisplay, WindowGLContext};
use net::resource_thread::new_resource_threads;
use profile::{mem as profile_mem, time as profile_time};
use profile_traits::{mem, time};
use script::serviceworker_manager::ServiceWorkerManager;
use script::JSEngineSetup;
use script_layout_interface::LayoutFactory;
use script_traits::{ScriptToConstellationChan, WindowSizeData};
use servo_config::{opts, pref, prefs};
use servo_media::player::context::GlContext;
use servo_media::ServoMedia;
#[cfg(all(target_os = "linux", not(target_env = "ohos")))]
use surfman::platform::generic::multi::connection::NativeConnection as LinuxNativeConnection;
#[cfg(all(target_os = "linux", not(target_env = "ohos")))]
use surfman::platform::generic::multi::context::NativeContext as LinuxNativeContext;
use surfman::{GLApi, GLVersion};
#[cfg(all(target_os = "linux", not(target_env = "ohos")))]
use surfman::{NativeConnection, NativeContext};
use webrender::{RenderApiSender, ShaderPrecacheFlags};
use webrender_api::{
    ColorF, DocumentId, FontInstanceFlags, FontInstanceKey, FontKey, FramePublishId, ImageKey,
    NativeFontHandle,
};
use webrender_traits::{
    CanvasToCompositorMsg, FontToCompositorMsg, ImageUpdate, RenderingContext, WebRenderFontApi,
    WebrenderExternalImageHandlers, WebrenderExternalImageRegistry, WebrenderImageHandlerType,
};
pub use {
    background_hang_monitor, base, bluetooth, bluetooth_traits, canvas, canvas_traits, compositing,
    constellation, devtools, devtools_traits, embedder_traits, euclid, fonts, ipc_channel,
    keyboard_types, layout_thread_2020, media, net, net_traits, profile, profile_traits, script,
    script_layout_interface, script_traits, servo_config as config, servo_config, servo_geometry,
    servo_url as url, servo_url, style, style_traits, webgpu, webrender_api, webrender_traits,
};

#[cfg(feature = "webdriver")]
fn webdriver(port: u16, constellation: Sender<ConstellationMsg>) {
    webdriver_server::start_server(port, constellation);
}

#[cfg(not(feature = "webdriver"))]
fn webdriver(_port: u16, _constellation: Sender<ConstellationMsg>) {}

#[cfg(feature = "media-gstreamer")]
mod media_platform {
    #[cfg(any(windows, target_os = "macos"))]
    mod gstreamer_plugins {
        include!(concat!(env!("OUT_DIR"), "/gstreamer_plugins.rs"));
    }

    use servo_media_gstreamer::GStreamerBackend;

    use super::ServoMedia;

    #[cfg(any(windows, target_os = "macos"))]
    pub fn init() {
        ServoMedia::init_with_backend(|| {
            let mut plugin_dir = std::env::current_exe().unwrap();
            plugin_dir.pop();

            if cfg!(target_os = "macos") {
                plugin_dir.push("lib");
            }

            match GStreamerBackend::init_with_plugins(
                plugin_dir,
                gstreamer_plugins::GSTREAMER_PLUGINS,
            ) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("Error initializing GStreamer: {:?}", e);
                    std::process::exit(1);
                },
            }
        });
    }

    #[cfg(not(any(windows, target_os = "macos")))]
    pub fn init() {
        ServoMedia::init::<GStreamerBackend>();
    }
}

#[cfg(not(feature = "media-gstreamer"))]
mod media_platform {
    use super::ServoMedia;
    pub fn init() {
        ServoMedia::init::<servo_media_dummy::DummyBackend>();
    }
}

/// The in-process interface to Servo.
///
/// It does everything necessary to render the web, primarily
/// orchestrating the interaction between JavaScript, CSS layout,
/// rendering, and the client window.
///
/// Clients create a `Servo` instance for a given reference-counted type
/// implementing `WindowMethods`, which is the bridge to whatever
/// application Servo is embedded in. Clients then create an event
/// loop to pump messages between the embedding application and
/// various browser components.
pub struct Servo<Window: WindowMethods + 'static + ?Sized> {
    compositor: IOCompositor<Window>,
    constellation_chan: Sender<ConstellationMsg>,
    embedder_receiver: EmbedderReceiver,
    messages_for_embedder: Vec<(Option<TopLevelBrowsingContextId>, EmbedderMsg)>,
    profiler_enabled: bool,
    /// For single-process Servo instances, this field controls the initialization
    /// and deinitialization of the JS Engine. Multiprocess Servo instances have their
    /// own instance that exists in the content process instead.
    _js_engine_setup: Option<JSEngineSetup>,
}

#[derive(Clone)]
struct RenderNotifier {
    compositor_proxy: CompositorProxy,
}

impl RenderNotifier {
    pub fn new(compositor_proxy: CompositorProxy) -> RenderNotifier {
        RenderNotifier { compositor_proxy }
    }
}

impl webrender_api::RenderNotifier for RenderNotifier {
    fn clone(&self) -> Box<dyn webrender_api::RenderNotifier> {
        Box::new(RenderNotifier::new(self.compositor_proxy.clone()))
    }

    fn wake_up(&self, _composite_needed: bool) {}

    fn new_frame_ready(
        &self,
        _document_id: DocumentId,
        _scrolled: bool,
        composite_needed: bool,
        _frame_publish_id: FramePublishId,
    ) {
        self.compositor_proxy
            .send(CompositorMsg::NewWebRenderFrameReady(composite_needed));
    }
}

pub struct InitializedServo<Window: WindowMethods + 'static + ?Sized> {
    pub servo: Servo<Window>,
    pub browser_id: TopLevelBrowsingContextId,
}

impl<Window> Servo<Window>
where
    Window: WindowMethods + 'static + ?Sized,
{
    pub fn new(
        mut embedder: Box<dyn EmbedderMethods>,
        window: Rc<Window>,
        user_agent: Option<String>,
        composite_target: CompositeTarget,
    ) -> InitializedServo<Window> {
        // Global configuration options, parsed from the command line.
        let opts = opts::get();

        use std::sync::atomic::Ordering;

        style::context::DEFAULT_DISABLE_STYLE_SHARING_CACHE
            .store(opts.debug.disable_share_style_cache, Ordering::Relaxed);
        style::context::DEFAULT_DUMP_STYLE_STATISTICS
            .store(opts.debug.dump_style_statistics, Ordering::Relaxed);
        style::traversal::IS_SERVO_NONINCREMENTAL_LAYOUT
            .store(opts.nonincremental_layout, Ordering::Relaxed);

        if !opts.multiprocess {
            media_platform::init();
        }

        let user_agent = match user_agent {
            Some(ref ua) if ua == "ios" => default_user_agent_string_for(UserAgent::iOS).into(),
            Some(ref ua) if ua == "android" => {
                default_user_agent_string_for(UserAgent::Android).into()
            },
            Some(ref ua) if ua == "desktop" => {
                default_user_agent_string_for(UserAgent::Desktop).into()
            },
            Some(ref ua) if ua == "ohos" => {
                default_user_agent_string_for(UserAgent::OpenHarmony).into()
            },
            Some(ua) => ua.into(),
            None => embedder
                .get_user_agent_string()
                .map(Into::into)
                .unwrap_or(default_user_agent_string_for(DEFAULT_USER_AGENT).into()),
        };

        // Initialize surfman
        let rendering_context = window.rendering_context();

        // Get GL bindings
        let webrender_gl = match rendering_context.connection().gl_api() {
            GLApi::GL => unsafe { gl::GlFns::load_with(|s| rendering_context.get_proc_address(s)) },
            GLApi::GLES => unsafe {
                gl::GlesFns::load_with(|s| rendering_context.get_proc_address(s))
            },
        };

        // Make sure the gl context is made current.
        rendering_context.make_gl_context_current().unwrap();
        debug_assert_eq!(webrender_gl.get_error(), gleam::gl::NO_ERROR,);

        // Bind the webrender framebuffer
        let framebuffer_object = rendering_context
            .context_surface_info()
            .unwrap_or(None)
            .map(|info| info.framebuffer_object)
            .unwrap_or(0);
        webrender_gl.bind_framebuffer(gleam::gl::FRAMEBUFFER, framebuffer_object);

        // Reserving a namespace to create TopLevelBrowsingContextId.
        PipelineNamespace::install(PipelineNamespaceId(0));
        let top_level_browsing_context_id = TopLevelBrowsingContextId::new();

        // Get both endpoints of a special channel for communication between
        // the client window and the compositor. This channel is unique because
        // messages to client may need to pump a platform-specific event loop
        // to deliver the message.
        let event_loop_waker = embedder.create_event_loop_waker();
        let (compositor_proxy, compositor_receiver) =
            create_compositor_channel(event_loop_waker.clone());
        let (embedder_proxy, embedder_receiver) = create_embedder_channel(event_loop_waker.clone());
        let time_profiler_chan = profile_time::Profiler::create(
            &opts.time_profiling,
            opts.time_profiler_trace_path.clone(),
        );
        let mem_profiler_chan = profile_mem::Profiler::create(opts.mem_profiler_period);

        let devtools_sender = if opts.devtools_server_enabled {
            Some(devtools::start_server(
                opts.devtools_port,
                embedder_proxy.clone(),
            ))
        } else {
            None
        };

        let coordinates: compositing::windowing::EmbedderCoordinates = window.get_coordinates();
        let device_pixel_ratio = coordinates.hidpi_factor.get();
        let viewport_size = coordinates.viewport.size().to_f32() / device_pixel_ratio;

        let (mut webrender, webrender_api_sender) = {
            let mut debug_flags = webrender::DebugFlags::empty();
            debug_flags.set(
                webrender::DebugFlags::PROFILER_DBG,
                opts.debug.webrender_stats,
            );

            let render_notifier = Box::new(RenderNotifier::new(compositor_proxy.clone()));
            let clear_color = servo_config::pref!(shell.background_color.rgba);
            let clear_color = ColorF::new(
                clear_color[0] as f32,
                clear_color[1] as f32,
                clear_color[2] as f32,
                clear_color[3] as f32,
            );
            webrender::create_webrender_instance(
                webrender_gl.clone(),
                render_notifier,
                webrender::WebRenderOptions {
                    // We force the use of optimized shaders here because rendering is broken
                    // on Android emulators with unoptimized shaders. This is due to a known
                    // issue in the emulator's OpenGL emulation layer.
                    // See: https://github.com/servo/servo/issues/31726
                    use_optimized_shaders: true,
                    resource_override_path: opts.shaders_dir.clone(),
                    enable_aa: !opts.debug.disable_text_antialiasing,
                    debug_flags,
                    precache_flags: if opts.debug.precache_shaders {
                        ShaderPrecacheFlags::FULL_COMPILE
                    } else {
                        ShaderPrecacheFlags::empty()
                    },
                    enable_subpixel_aa: pref!(gfx.subpixel_text_antialiasing.enabled) &&
                        !opts.debug.disable_subpixel_text_antialiasing,
                    allow_texture_swizzling: pref!(gfx.texture_swizzling.enabled),
                    clear_color,
                    ..Default::default()
                },
                None,
            )
            .expect("Unable to initialize webrender!")
        };

        let webrender_api = webrender_api_sender.create_api();
        let webrender_document = webrender_api.add_document(coordinates.get_viewport().size());

        // Important that this call is done in a single-threaded fashion, we
        // can't defer it after `create_constellation` has started.
        let js_engine_setup = if !opts.multiprocess {
            Some(script::init())
        } else {
            None
        };

        // Create the webgl thread
        let gl_type = match webrender_gl.get_type() {
            gleam::gl::GlType::Gl => sparkle::gl::GlType::Gl,
            gleam::gl::GlType::Gles => sparkle::gl::GlType::Gles,
        };

        let (external_image_handlers, external_images) = WebrenderExternalImageHandlers::new();
        let mut external_image_handlers = Box::new(external_image_handlers);

        let WebGLComm {
            webgl_threads,
            webxr_layer_grand_manager,
            image_handler,
        } = WebGLComm::new(
            rendering_context.clone(),
            webrender_api.create_sender(),
            webrender_document,
            external_images.clone(),
            gl_type,
        );

        // Set webrender external image handler for WebGL textures
        external_image_handlers.set_handler(image_handler, WebrenderImageHandlerType::WebGL);

        // Create the WebXR main thread
        let mut webxr_main_thread =
            webxr::MainThreadRegistry::new(event_loop_waker, webxr_layer_grand_manager)
                .expect("Failed to create WebXR device registry");
        if pref!(dom.webxr.enabled) {
            embedder.register_webxr(&mut webxr_main_thread, embedder_proxy.clone());
        }

        let wgpu_image_handler = webgpu::WGPUExternalImages::default();
        let wgpu_image_map = wgpu_image_handler.images.clone();
        external_image_handlers.set_handler(
            Box::new(wgpu_image_handler),
            WebrenderImageHandlerType::WebGPU,
        );

        let (player_context, glplayer_threads) = Self::create_media_window_gl_context(
            external_image_handlers.borrow_mut(),
            external_images.clone(),
            &rendering_context,
        );

        webrender.set_external_image_handler(external_image_handlers);

        // The division by 1 represents the page's default zoom of 100%,
        // and gives us the appropriate CSSPixel type for the viewport.
        let window_size = WindowSizeData {
            initial_viewport: viewport_size / Scale::new(1.0),
            device_pixel_ratio: Scale::new(device_pixel_ratio),
        };

        // Create the constellation, which maintains the engine pipelines, including script and
        // layout, as well as the navigation context.
        let constellation_chan = create_constellation(
            user_agent,
            opts.config_dir.clone(),
            embedder_proxy,
            compositor_proxy.clone(),
            time_profiler_chan.clone(),
            mem_profiler_chan.clone(),
            devtools_sender,
            webrender_document,
            webrender_api_sender,
            webxr_main_thread.registry(),
            player_context,
            Some(webgl_threads),
            glplayer_threads,
            window_size,
            external_images,
            wgpu_image_map,
        );

        if cfg!(feature = "webdriver") {
            if let Some(port) = opts.webdriver_port {
                webdriver(port, constellation_chan.clone());
            }
        }

        let composite_target = if let Some(path) = opts.output_file.clone() {
            CompositeTarget::PngFile(path.into())
        } else {
            composite_target
        };

        // The compositor coordinates with the client window to create the final
        // rendered page and display it somewhere.
        let compositor = IOCompositor::create(
            window,
            InitialCompositorState {
                sender: compositor_proxy,
                receiver: compositor_receiver,
                constellation_chan: constellation_chan.clone(),
                time_profiler_chan,
                mem_profiler_chan,
                webrender,
                webrender_document,
                webrender_api,
                rendering_context,
                webrender_gl,
                webxr_main_thread,
            },
            composite_target,
            opts.exit_after_load,
            opts.debug.convert_mouse_to_touch,
            top_level_browsing_context_id,
        );

        let servo = Servo {
            compositor,
            constellation_chan,
            embedder_receiver,
            messages_for_embedder: Vec::new(),
            profiler_enabled: false,
            _js_engine_setup: js_engine_setup,
        };
        InitializedServo {
            servo,
            browser_id: top_level_browsing_context_id,
        }
    }

    #[cfg(all(target_os = "linux", not(target_env = "ohos")))]
    fn get_native_media_display_and_gl_context(
        rendering_context: &RenderingContext,
    ) -> Option<(NativeDisplay, GlContext)> {
        let gl_context = match rendering_context.native_context() {
            NativeContext::Default(LinuxNativeContext::Default(native_context)) => {
                GlContext::Egl(native_context.egl_context as usize)
            },
            NativeContext::Default(LinuxNativeContext::Alternate(native_context)) => {
                GlContext::Egl(native_context.egl_context as usize)
            },
            NativeContext::Alternate(_) => return None,
        };

        let native_display = match rendering_context.connection().native_connection() {
            NativeConnection::Default(LinuxNativeConnection::Default(connection)) => {
                NativeDisplay::Egl(connection.0 as usize)
            },
            NativeConnection::Default(LinuxNativeConnection::Alternate(connection)) => {
                NativeDisplay::X11(connection.x11_display as usize)
            },
            NativeConnection::Alternate(_) => return None,
        };
        Some((native_display, gl_context))
    }

    // @TODO(victor): https://github.com/servo/media/pull/315
    #[cfg(target_os = "windows")]
    fn get_native_media_display_and_gl_context(
        rendering_context: &RenderingContext,
    ) -> Option<(NativeDisplay, GlContext)> {
        #[cfg(feature = "no-wgl")]
        {
            let gl_context =
                GlContext::Egl(rendering_context.native_context().egl_context as usize);
            let native_display =
                NativeDisplay::Egl(rendering_context.native_device().egl_display as usize);
            Some((native_display, gl_context))
        }
        #[cfg(not(feature = "no-wgl"))]
        None
    }

    #[cfg(not(any(
        target_os = "windows",
        all(target_os = "linux", not(target_env = "ohos"))
    )))]
    fn get_native_media_display_and_gl_context(
        _rendering_context: &RenderingContext,
    ) -> Option<(NativeDisplay, GlContext)> {
        None
    }

    fn create_media_window_gl_context(
        external_image_handlers: &mut WebrenderExternalImageHandlers,
        external_images: Arc<Mutex<WebrenderExternalImageRegistry>>,
        rendering_context: &RenderingContext,
    ) -> (WindowGLContext, Option<GLPlayerThreads>) {
        if !pref!(media.glvideo.enabled) {
            return (
                WindowGLContext {
                    gl_context: GlContext::Unknown,
                    gl_api: GlApi::None,
                    native_display: NativeDisplay::Unknown,
                    glplayer_chan: None,
                },
                None,
            );
        }

        let (native_display, gl_context) =
            match Self::get_native_media_display_and_gl_context(rendering_context) {
                Some((native_display, gl_context)) => (native_display, gl_context),
                None => {
                    return (
                        WindowGLContext {
                            gl_context: GlContext::Unknown,
                            gl_api: GlApi::None,
                            native_display: NativeDisplay::Unknown,
                            glplayer_chan: None,
                        },
                        None,
                    );
                },
            };

        let api = rendering_context.connection().gl_api();
        let attributes = rendering_context.context_attributes();
        let GLVersion { major, minor } = attributes.version;
        let gl_api = match api {
            GLApi::GL if major >= 3 && minor >= 2 => GlApi::OpenGL3,
            GLApi::GL => GlApi::OpenGL,
            GLApi::GLES if major > 1 => GlApi::Gles2,
            GLApi::GLES => GlApi::Gles1,
        };

        assert!(!matches!(gl_context, GlContext::Unknown));
        let (glplayer_threads, image_handler) = GLPlayerThreads::new(external_images.clone());
        external_image_handlers.set_handler(image_handler, WebrenderImageHandlerType::Media);

        (
            WindowGLContext {
                gl_context,
                native_display,
                gl_api,
                glplayer_chan: Some(GLPlayerThreads::pipeline(&glplayer_threads)),
            },
            Some(glplayer_threads),
        )
    }

    fn handle_window_event(&mut self, event: EmbedderEvent) -> bool {
        match event {
            EmbedderEvent::Idle => {},

            EmbedderEvent::Refresh => {
                self.compositor.composite();
            },

            EmbedderEvent::WindowResize => {
                return self.compositor.on_resize_window_event();
            },
            EmbedderEvent::InvalidateNativeSurface => {
                self.compositor.invalidate_native_surface();
            },
            EmbedderEvent::ReplaceNativeSurface(native_widget, coords) => {
                self.compositor
                    .replace_native_surface(native_widget, coords);
                self.compositor.composite();
            },
            EmbedderEvent::AllowNavigationResponse(pipeline_id, allowed) => {
                let msg = ConstellationMsg::AllowNavigationResponse(pipeline_id, allowed);
                if let Err(e) = self.constellation_chan.send(msg) {
                    warn!(
                        "Sending allow navigation to constellation failed ({:?}).",
                        e
                    );
                }
            },

            EmbedderEvent::LoadUrl(top_level_browsing_context_id, url) => {
                let msg = ConstellationMsg::LoadUrl(top_level_browsing_context_id, url);
                if let Err(e) = self.constellation_chan.send(msg) {
                    warn!("Sending load url to constellation failed ({:?}).", e);
                }
            },

            EmbedderEvent::ClearCache => {
                let msg = ConstellationMsg::ClearCache;
                if let Err(e) = self.constellation_chan.send(msg) {
                    warn!("Sending clear cache to constellation failed ({:?}).", e);
                }
            },

            EmbedderEvent::MouseWindowEventClass(mouse_window_event) => {
                self.compositor
                    .on_mouse_window_event_class(mouse_window_event);
            },

            EmbedderEvent::MouseWindowMoveEventClass(cursor) => {
                self.compositor.on_mouse_window_move_event_class(cursor);
            },

            EmbedderEvent::Touch(event_type, identifier, location) => {
                self.compositor
                    .on_touch_event(event_type, identifier, location);
            },

            EmbedderEvent::Wheel(delta, location) => {
                self.compositor.on_wheel_event(delta, location);
            },

            EmbedderEvent::Scroll(scroll_location, cursor, phase) => {
                self.compositor
                    .on_scroll_event(scroll_location, cursor, phase);
            },

            EmbedderEvent::Zoom(magnification) => {
                self.compositor.on_zoom_window_event(magnification);
            },

            EmbedderEvent::ResetZoom => {
                self.compositor.on_zoom_reset_window_event();
            },

            EmbedderEvent::PinchZoom(zoom) => {
                self.compositor.on_pinch_zoom_window_event(zoom);
            },

            EmbedderEvent::Navigation(top_level_browsing_context_id, direction) => {
                let msg =
                    ConstellationMsg::TraverseHistory(top_level_browsing_context_id, direction);
                if let Err(e) = self.constellation_chan.send(msg) {
                    warn!("Sending navigation to constellation failed ({:?}).", e);
                }
                self.messages_for_embedder.push((
                    Some(top_level_browsing_context_id),
                    EmbedderMsg::Status(None),
                ));
            },

            EmbedderEvent::Keyboard(key_event) => {
                let msg = ConstellationMsg::Keyboard(key_event);
                if let Err(e) = self.constellation_chan.send(msg) {
                    warn!("Sending keyboard event to constellation failed ({:?}).", e);
                }
            },

            EmbedderEvent::IMEDismissed => {
                let msg = ConstellationMsg::IMEDismissed;
                if let Err(e) = self.constellation_chan.send(msg) {
                    warn!(
                        "Sending IMEDismissed event to constellation failed ({:?}).",
                        e
                    );
                }
            },

            EmbedderEvent::Quit => {
                self.compositor.maybe_start_shutting_down();
            },

            EmbedderEvent::ExitFullScreen(top_level_browsing_context_id) => {
                let msg = ConstellationMsg::ExitFullScreen(top_level_browsing_context_id);
                if let Err(e) = self.constellation_chan.send(msg) {
                    warn!("Sending exit fullscreen to constellation failed ({:?}).", e);
                }
            },

            EmbedderEvent::Reload(top_level_browsing_context_id) => {
                let msg = ConstellationMsg::Reload(top_level_browsing_context_id);
                if let Err(e) = self.constellation_chan.send(msg) {
                    warn!("Sending reload to constellation failed ({:?}).", e);
                }
            },

            EmbedderEvent::ToggleSamplingProfiler(rate, max_duration) => {
                self.profiler_enabled = !self.profiler_enabled;
                let msg = if self.profiler_enabled {
                    ConstellationMsg::EnableProfiler(rate, max_duration)
                } else {
                    ConstellationMsg::DisableProfiler
                };
                if let Err(e) = self.constellation_chan.send(msg) {
                    warn!("Sending profiler toggle to constellation failed ({:?}).", e);
                }
            },

            EmbedderEvent::ToggleWebRenderDebug(option) => {
                self.compositor.toggle_webrender_debug(option);
            },

            EmbedderEvent::CaptureWebRender => {
                self.compositor.capture_webrender();
            },

            EmbedderEvent::NewWebView(url, top_level_browsing_context_id) => {
                let msg = ConstellationMsg::NewWebView(url, top_level_browsing_context_id);
                if let Err(e) = self.constellation_chan.send(msg) {
                    warn!(
                        "Sending NewBrowser message to constellation failed ({:?}).",
                        e
                    );
                }
            },

            EmbedderEvent::FocusWebView(top_level_browsing_context_id) => {
                let msg = ConstellationMsg::FocusWebView(top_level_browsing_context_id);
                if let Err(e) = self.constellation_chan.send(msg) {
                    warn!(
                        "Sending FocusBrowser message to constellation failed ({:?}).",
                        e
                    );
                }
            },

            EmbedderEvent::CloseWebView(top_level_browsing_context_id) => {
                let msg = ConstellationMsg::CloseWebView(top_level_browsing_context_id);
                if let Err(e) = self.constellation_chan.send(msg) {
                    warn!(
                        "Sending CloseBrowser message to constellation failed ({:?}).",
                        e
                    );
                }
            },

            EmbedderEvent::MoveResizeWebView(webview_id, rect) => {
                self.compositor.move_resize_webview(webview_id, rect);
            },
            EmbedderEvent::ShowWebView(webview_id, hide_others) => {
                if let Err(UnknownWebView(webview_id)) =
                    self.compositor.show_webview(webview_id, hide_others)
                {
                    warn!("{webview_id}: ShowWebView on unknown webview id");
                }
            },
            EmbedderEvent::HideWebView(webview_id) => {
                if let Err(UnknownWebView(webview_id)) = self.compositor.hide_webview(webview_id) {
                    warn!("{webview_id}: HideWebView on unknown webview id");
                }
            },
            EmbedderEvent::RaiseWebViewToTop(webview_id, hide_others) => {
                if let Err(UnknownWebView(webview_id)) = self
                    .compositor
                    .raise_webview_to_top(webview_id, hide_others)
                {
                    warn!("{webview_id}: RaiseWebViewToTop on unknown webview id");
                }
            },
            EmbedderEvent::BlurWebView => {
                self.send_to_constellation(ConstellationMsg::BlurWebView);
            },

            EmbedderEvent::SendError(top_level_browsing_context_id, e) => {
                let msg = ConstellationMsg::SendError(top_level_browsing_context_id, e);
                if let Err(e) = self.constellation_chan.send(msg) {
                    warn!(
                        "Sending SendError message to constellation failed ({:?}).",
                        e
                    );
                }
            },

            EmbedderEvent::MediaSessionAction(a) => {
                let msg = ConstellationMsg::MediaSessionAction(a);
                if let Err(e) = self.constellation_chan.send(msg) {
                    warn!(
                        "Sending MediaSessionAction message to constellation failed ({:?}).",
                        e
                    );
                }
            },

            EmbedderEvent::SetWebViewThrottled(webview_id, throttled) => {
                let msg = ConstellationMsg::SetWebViewThrottled(webview_id, throttled);
                if let Err(e) = self.constellation_chan.send(msg) {
                    warn!(
                        "Sending SetWebViewThrottled to constellation failed ({:?}).",
                        e
                    );
                }
            },

            EmbedderEvent::Gamepad(gamepad_event) => {
                let msg = ConstellationMsg::Gamepad(gamepad_event);
                if let Err(e) = self.constellation_chan.send(msg) {
                    warn!("Sending Gamepad event to constellation failed ({:?}).", e);
                }
            },
        }
        false
    }

    fn send_to_constellation(&self, msg: ConstellationMsg) {
        let variant_name = msg.variant_name();
        if let Err(e) = self.constellation_chan.send(msg) {
            warn!("Sending {variant_name} to constellation failed: {e:?}");
        }
    }

    fn receive_messages(&mut self) {
        while let Some((top_level_browsing_context, msg)) =
            self.embedder_receiver.try_recv_embedder_msg()
        {
            match (msg, self.compositor.shutdown_state) {
                (_, ShutdownState::FinishedShuttingDown) => {
                    error!(
                        "embedder shouldn't be handling messages after compositor has shut down"
                    );
                },

                (_, ShutdownState::ShuttingDown) => {},

                (EmbedderMsg::Keyboard(key_event), ShutdownState::NotShuttingDown) => {
                    let event = (top_level_browsing_context, EmbedderMsg::Keyboard(key_event));
                    self.messages_for_embedder.push(event);
                },

                (msg, ShutdownState::NotShuttingDown) => {
                    self.messages_for_embedder
                        .push((top_level_browsing_context, msg));
                },
            }
        }
    }

    pub fn get_events(&mut self) -> Drain<'_, (Option<TopLevelBrowsingContextId>, EmbedderMsg)> {
        self.messages_for_embedder.drain(..)
    }

    pub fn handle_events(&mut self, events: impl IntoIterator<Item = EmbedderEvent>) -> bool {
        if self.compositor.receive_messages() {
            self.receive_messages();
        }
        let mut need_resize = false;
        for event in events {
            trace!("servo <- embedder EmbedderEvent {:?}", event);
            need_resize |= self.handle_window_event(event);
        }
        if self.compositor.shutdown_state != ShutdownState::FinishedShuttingDown {
            self.compositor.perform_updates();
        } else {
            self.messages_for_embedder
                .push((None, EmbedderMsg::Shutdown));
        }
        need_resize
    }

    pub fn repaint_synchronously(&mut self) {
        self.compositor.repaint_synchronously()
    }

    pub fn pinch_zoom_level(&self) -> f32 {
        self.compositor.pinch_zoom_level().get()
    }

    pub fn setup_logging(&self) {
        let constellation_chan = self.constellation_chan.clone();
        let env = env_logger::Env::default();
        let env_logger = EnvLoggerBuilder::from_env(env).build();
        let con_logger = FromCompositorLogger::new(constellation_chan);

        let filter = max(env_logger.filter(), con_logger.filter());
        let logger = BothLogger(env_logger, con_logger);

        log::set_boxed_logger(Box::new(logger)).expect("Failed to set logger.");
        log::set_max_level(filter);
    }

    pub fn window(&self) -> &Window {
        &self.compositor.window
    }

    pub fn deinit(self) {
        self.compositor.deinit();
    }

    pub fn present(&mut self) {
        self.compositor.present();
    }

    /// Return the OpenGL framebuffer name of the most-recently-completed frame when compositing to
    /// [`CompositeTarget::Fbo`], or None otherwise.
    pub fn offscreen_framebuffer_id(&self) -> Option<u32> {
        self.compositor.offscreen_framebuffer_id()
    }
}

fn create_embedder_channel(
    event_loop_waker: Box<dyn EventLoopWaker>,
) -> (EmbedderProxy, EmbedderReceiver) {
    let (sender, receiver) = unbounded();
    (
        EmbedderProxy {
            sender,
            event_loop_waker,
        },
        EmbedderReceiver { receiver },
    )
}

fn create_compositor_channel(
    event_loop_waker: Box<dyn EventLoopWaker>,
) -> (CompositorProxy, CompositorReceiver) {
    let (sender, receiver) = unbounded();
    (
        CompositorProxy {
            sender,
            event_loop_waker,
        },
        CompositorReceiver { receiver },
    )
}

fn get_layout_factory(legacy_layout: bool) -> Arc<dyn LayoutFactory> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "layout_2013")] {
            if legacy_layout {
                return Arc::new(layout_thread_2013::LayoutFactoryImpl());
            }
        } else {
            if legacy_layout {
                warn!("Runtime option `legacy_layout` was enabled, but the `layout_2013` \
                feature was not enabled at compile time. Falling back to layout 2020! ");
           }
        }
    }
    Arc::new(layout_thread_2020::LayoutFactoryImpl())
}

fn create_constellation(
    user_agent: Cow<'static, str>,
    config_dir: Option<PathBuf>,
    embedder_proxy: EmbedderProxy,
    compositor_proxy: CompositorProxy,
    time_profiler_chan: time::ProfilerChan,
    mem_profiler_chan: mem::ProfilerChan,
    devtools_sender: Option<Sender<devtools_traits::DevtoolsControlMsg>>,
    webrender_document: DocumentId,
    webrender_api_sender: RenderApiSender,
    webxr_registry: webxr_api::Registry,
    player_context: WindowGLContext,
    webgl_threads: Option<WebGLThreads>,
    glplayer_threads: Option<GLPlayerThreads>,
    initial_window_size: WindowSizeData,
    external_images: Arc<Mutex<WebrenderExternalImageRegistry>>,
    wgpu_image_map: Arc<Mutex<HashMap<u64, webgpu::PresentationData>>>,
) -> Sender<ConstellationMsg> {
    // Global configuration options, parsed from the command line.
    let opts = opts::get();

    let bluetooth_thread: IpcSender<BluetoothRequest> =
        BluetoothThreadFactory::new(embedder_proxy.clone());

    let (public_resource_threads, private_resource_threads) = new_resource_threads(
        user_agent.clone(),
        devtools_sender.clone(),
        time_profiler_chan.clone(),
        mem_profiler_chan.clone(),
        embedder_proxy.clone(),
        config_dir,
        opts.certificate_path.clone(),
        opts.ignore_certificate_errors,
    );

    let font_cache_thread = FontCacheThread::new(Box::new(WebRenderFontApiCompositorProxy(
        compositor_proxy.clone(),
    )));

    let (canvas_create_sender, canvas_ipc_sender) = CanvasPaintThread::start(
        Box::new(CanvasWebrenderApi(compositor_proxy.clone())),
        font_cache_thread.clone(),
        public_resource_threads.clone(),
    );

    let initial_state = InitialConstellationState {
        compositor_proxy,
        embedder_proxy,
        devtools_sender,
        bluetooth_thread,
        font_cache_thread,
        public_resource_threads,
        private_resource_threads,
        time_profiler_chan,
        mem_profiler_chan,
        webrender_document,
        webrender_api_sender,
        webxr_registry,
        webgl_threads,
        glplayer_threads,
        player_context,
        user_agent,
        webrender_external_images: external_images,
        wgpu_image_map,
    };

    let layout_factory: Arc<dyn LayoutFactory> = get_layout_factory(opts::get().legacy_layout);

    Constellation::<
        script::script_thread::ScriptThread,
        script::serviceworker_manager::ServiceWorkerManager,
    >::start(
        initial_state,
        layout_factory,
        initial_window_size,
        opts.random_pipeline_closure_probability,
        opts.random_pipeline_closure_seed,
        opts.hard_fail,
        !opts.debug.disable_canvas_antialiasing,
        canvas_create_sender,
        canvas_ipc_sender,
    )
}

struct WebRenderFontApiCompositorProxy(CompositorProxy);

impl WebRenderFontApi for WebRenderFontApiCompositorProxy {
    fn add_font_instance(
        &self,
        font_key: FontKey,
        size: f32,
        flags: FontInstanceFlags,
    ) -> FontInstanceKey {
        let (sender, receiver) = unbounded();
        self.0
            .send(CompositorMsg::Forwarded(ForwardedToCompositorMsg::Font(
                FontToCompositorMsg::AddFontInstance(font_key, size, flags, sender),
            )));
        receiver.recv().unwrap()
    }

    fn add_font(&self, data: Arc<Vec<u8>>, index: u32) -> FontKey {
        let (sender, receiver) = unbounded();
        let (bytes_sender, bytes_receiver) =
            ipc::bytes_channel().expect("failed to create IPC channel");
        self.0
            .send(CompositorMsg::Forwarded(ForwardedToCompositorMsg::Font(
                FontToCompositorMsg::AddFont(sender, index, bytes_receiver),
            )));
        let _ = bytes_sender.send(&data);
        receiver.recv().unwrap()
    }

    fn add_system_font(&self, handle: NativeFontHandle) -> FontKey {
        let (sender, receiver) = unbounded();
        self.0
            .send(CompositorMsg::Forwarded(ForwardedToCompositorMsg::Font(
                FontToCompositorMsg::AddSystemFont(sender, handle),
            )));
        receiver.recv().unwrap()
    }

    fn forward_add_font_message(
        &self,
        bytes_receiver: ipc::IpcBytesReceiver,
        font_index: u32,
        result_sender: IpcSender<FontKey>,
    ) {
        let (sender, receiver) = unbounded();
        self.0
            .send(CompositorMsg::Forwarded(ForwardedToCompositorMsg::Font(
                FontToCompositorMsg::AddFont(sender, font_index, bytes_receiver),
            )));
        let _ = result_sender.send(receiver.recv().unwrap());
    }

    fn forward_add_font_instance_message(
        &self,
        font_key: FontKey,
        size: f32,
        flags: FontInstanceFlags,
        result_sender: IpcSender<FontInstanceKey>,
    ) {
        let (sender, receiver) = unbounded();
        self.0
            .send(CompositorMsg::Forwarded(ForwardedToCompositorMsg::Font(
                FontToCompositorMsg::AddFontInstance(font_key, size, flags, sender),
            )));
        let _ = result_sender.send(receiver.recv().unwrap());
    }
}

#[derive(Clone)]
struct CanvasWebrenderApi(CompositorProxy);

impl canvas_paint_thread::WebrenderApi for CanvasWebrenderApi {
    fn generate_key(&self) -> Option<ImageKey> {
        let (sender, receiver) = unbounded();
        self.0
            .send(CompositorMsg::Forwarded(ForwardedToCompositorMsg::Canvas(
                CanvasToCompositorMsg::GenerateKey(sender),
            )));
        receiver.recv().ok()
    }
    fn update_images(&self, updates: Vec<ImageUpdate>) {
        self.0
            .send(CompositorMsg::Forwarded(ForwardedToCompositorMsg::Canvas(
                CanvasToCompositorMsg::UpdateImages(updates),
            )));
    }
    fn clone(&self) -> Box<dyn canvas_paint_thread::WebrenderApi> {
        Box::new(<Self as Clone>::clone(self))
    }
}

// A logger that logs to two downstream loggers.
// This should probably be in the log crate.
struct BothLogger<Log1, Log2>(Log1, Log2);

impl<Log1, Log2> Log for BothLogger<Log1, Log2>
where
    Log1: Log,
    Log2: Log,
{
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.0.enabled(metadata) || self.1.enabled(metadata)
    }

    fn log(&self, record: &Record) {
        self.0.log(record);
        self.1.log(record);
    }

    fn flush(&self) {
        self.0.flush();
        self.1.flush();
    }
}

pub fn set_logger(script_to_constellation_chan: ScriptToConstellationChan) {
    let con_logger = FromScriptLogger::new(script_to_constellation_chan);
    let env = env_logger::Env::default();
    let env_logger = EnvLoggerBuilder::from_env(env).build();

    let filter = max(env_logger.filter(), con_logger.filter());
    let logger = BothLogger(env_logger, con_logger);

    log::set_boxed_logger(Box::new(logger)).expect("Failed to set logger.");
    log::set_max_level(filter);
}

/// Content process entry point.
pub fn run_content_process(token: String) {
    let (unprivileged_content_sender, unprivileged_content_receiver) =
        ipc::channel::<UnprivilegedContent>().unwrap();
    let connection_bootstrap: IpcSender<IpcSender<UnprivilegedContent>> =
        IpcSender::connect(token).unwrap();
    connection_bootstrap
        .send(unprivileged_content_sender)
        .unwrap();

    let unprivileged_content = unprivileged_content_receiver.recv().unwrap();
    opts::set_options(unprivileged_content.opts());
    prefs::pref_map()
        .set_all(unprivileged_content.prefs())
        .expect("Failed to set preferences");

    // Enter the sandbox if necessary.
    if opts::get().sandbox {
        create_sandbox();
    }

    let _js_engine_setup = script::init();

    match unprivileged_content {
        UnprivilegedContent::Pipeline(mut content) => {
            media_platform::init();

            set_logger(content.script_to_constellation_chan().clone());

            let background_hang_monitor_register = content.register_with_background_hang_monitor();
            let layout_factory: Arc<dyn LayoutFactory> =
                get_layout_factory(opts::get().legacy_layout);

            content.start_all::<script::script_thread::ScriptThread>(
                true,
                layout_factory,
                background_hang_monitor_register,
            );
        },
        UnprivilegedContent::ServiceWorker(content) => {
            content.start::<ServiceWorkerManager>();
        },
    }
}

#[cfg(all(
    not(target_os = "windows"),
    not(target_os = "ios"),
    not(target_os = "android"),
    not(target_arch = "arm"),
    not(target_arch = "aarch64"),
    not(target_env = "ohos"),
))]
fn create_sandbox() {
    ChildSandbox::new(content_process_sandbox_profile())
        .activate()
        .expect("Failed to activate sandbox!");
}

#[cfg(any(
    target_os = "windows",
    target_os = "ios",
    target_os = "android",
    target_arch = "arm",
    target_arch = "aarch64",
    target_env = "ohos",
))]
fn create_sandbox() {
    panic!("Sandboxing is not supported on Windows, iOS, ARM targets and android.");
}

enum UserAgent {
    Desktop,
    Android,
    OpenHarmony,
    #[allow(non_camel_case_types)]
    iOS,
}

fn default_user_agent_string_for(agent: UserAgent) -> &'static str {
    #[cfg(all(target_os = "linux", target_arch = "x86_64", not(target_env = "ohos")))]
    const DESKTOP_UA_STRING: &str =
        "Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Servo/1.0 Firefox/111.0";
    #[cfg(all(
        target_os = "linux",
        not(target_arch = "x86_64"),
        not(target_env = "ohos")
    ))]
    const DESKTOP_UA_STRING: &str =
        "Mozilla/5.0 (X11; Linux i686; rv:109.0) Servo/1.0 Firefox/111.0";

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    const DESKTOP_UA_STRING: &str =
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Servo/1.0 Firefox/111.0";
    #[cfg(all(target_os = "windows", not(target_arch = "x86_64")))]
    const DESKTOP_UA_STRING: &str =
        "Mozilla/5.0 (Windows NT 10.0; rv:109.0) Servo/1.0 Firefox/111.0";

    #[cfg(target_os = "macos")]
    const DESKTOP_UA_STRING: &str =
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:109.0) Servo/1.0 Firefox/111.0";

    #[cfg(any(target_os = "android", target_env = "ohos"))]
    const DESKTOP_UA_STRING: &str = "";

    match agent {
        UserAgent::Desktop => DESKTOP_UA_STRING,
        UserAgent::Android => "Mozilla/5.0 (Android; Mobile; rv:109.0) Servo/1.0 Firefox/111.0",
        UserAgent::OpenHarmony => "Mozilla/5.0 (OpenHarmony; Mobile; rv:109.0) Servo/1.0 Firefox/111.0",
        UserAgent::iOS => {
            "Mozilla/5.0 (iPhone; CPU iPhone OS 16_4 like Mac OS X; rv:109.0) Servo/1.0 Firefox/111.0"
        },
    }
}

#[cfg(target_os = "android")]
const DEFAULT_USER_AGENT: UserAgent = UserAgent::Android;

#[cfg(target_env = "ohos")]
const DEFAULT_USER_AGENT: UserAgent = UserAgent::OpenHarmony;

#[cfg(target_os = "ios")]
const DEFAULT_USER_AGENT: UserAgent = UserAgent::iOS;

#[cfg(not(any(target_os = "android", target_os = "ios", target_env = "ohos")))]
const DEFAULT_USER_AGENT: UserAgent = UserAgent::Desktop;
