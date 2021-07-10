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

#[macro_use]
extern crate log;

pub use background_hang_monitor;
pub use bluetooth;
pub use bluetooth_traits;
pub use canvas;
pub use canvas_traits;
pub use compositing;
pub use constellation;
pub use devtools;
pub use devtools_traits;
pub use embedder_traits;
pub use euclid;
pub use gfx;
pub use ipc_channel;
pub use layout_thread;
pub use media;
pub use msg;
pub use net;
pub use net_traits;
pub use profile;
pub use profile_traits;
pub use script;
pub use script_layout_interface;
pub use script_traits;
pub use servo_config;
pub use servo_geometry;
pub use servo_url;
pub use style;
pub use style_traits;
pub use webgpu;
pub use webrender_api;
pub use webrender_surfman;
pub use webrender_traits;

#[cfg(feature = "webdriver")]
fn webdriver(port: u16, constellation: Sender<ConstellationMsg>) {
    webdriver_server::start_server(port, constellation);
}

#[cfg(not(feature = "webdriver"))]
fn webdriver(_port: u16, _constellation: Sender<ConstellationMsg>) {}

use bluetooth::BluetoothThreadFactory;
use bluetooth_traits::BluetoothRequest;
use canvas::canvas_paint_thread::{self, CanvasPaintThread};
use canvas::WebGLComm;
use canvas_traits::webgl::WebGLThreads;
use compositing::compositor_thread::{
    CompositorProxy, CompositorReceiver, InitialCompositorState, Msg, WebrenderCanvasMsg,
    WebrenderFontMsg, WebrenderMsg,
};
use compositing::windowing::{EmbedderMethods, WindowEvent, WindowMethods};
use compositing::{CompositingReason, ConstellationMsg, IOCompositor, ShutdownState};
#[cfg(all(
    not(target_os = "windows"),
    not(target_os = "ios"),
    not(target_os = "android"),
    not(target_arch = "arm"),
    not(target_arch = "aarch64")
))]
use constellation::content_process_sandbox_profile;
use constellation::{Constellation, InitialConstellationState, UnprivilegedContent};
use constellation::{FromCompositorLogger, FromScriptLogger};
use crossbeam_channel::{unbounded, Sender};
use embedder_traits::{EmbedderMsg, EmbedderProxy, EmbedderReceiver, EventLoopWaker};
use env_logger::Builder as EnvLoggerBuilder;
use euclid::{Scale, Size2D};
#[cfg(all(
    not(target_os = "windows"),
    not(target_os = "ios"),
    not(target_os = "android"),
    not(target_arch = "arm"),
    not(target_arch = "aarch64")
))]
use gaol::sandbox::{ChildSandbox, ChildSandboxMethods};
use gfx::font_cache_thread::FontCacheThread;
use ipc_channel::ipc::{self, IpcSender};
use log::{Log, Metadata, Record};
use media::{GLPlayerThreads, WindowGLContext};
use msg::constellation_msg::{PipelineNamespace, PipelineNamespaceId};
use net::resource_thread::new_resource_threads;
use net_traits::IpcSend;
use profile::mem as profile_mem;
use profile::time as profile_time;
use profile_traits::mem;
use profile_traits::time;
use script::serviceworker_manager::ServiceWorkerManager;
use script::JSEngineSetup;
use script_traits::{ScriptToConstellationChan, WindowSizeData};
use servo_config::opts;
use servo_config::{pref, prefs};
use servo_media::player::context::GlContext;
use servo_media::ServoMedia;
use std::borrow::Cow;
use std::cmp::max;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use surfman::GLApi;
use webrender::ShaderPrecacheFlags;
use webrender_traits::WebrenderExternalImageHandlers;
use webrender_traits::WebrenderExternalImageRegistry;
use webrender_traits::WebrenderImageHandlerType;

pub use gleam::gl;
pub use keyboard_types;
pub use msg::constellation_msg::TopLevelBrowsingContextId as BrowserId;
pub use servo_config as config;
pub use servo_url as url;

#[cfg(feature = "media-gstreamer")]
mod media_platform {
    #[cfg(any(windows, target_os = "macos"))]
    mod gstreamer_plugins {
        include!(concat!(env!("OUT_DIR"), "/gstreamer_plugins.rs"));
    }

    use super::ServoMedia;
    use servo_media_gstreamer::GStreamerBackend;

    #[cfg(feature = "uwp")]
    fn set_gstreamer_log_handler() {
        use gstreamer::{debug_add_log_function, DebugLevel};

        debug_add_log_function(|cat, level, file, function, line, _, message| {
            let message = format!(
                "{:?} {:?} {:?}:{:?}:{:?} {:?}",
                cat.get_name(),
                level,
                file,
                line,
                function,
                message
            );
            match level {
                DebugLevel::Debug => debug!("{}", message),
                DebugLevel::Error => error!("{}", message),
                DebugLevel::Warning => warn!("{}", message),
                DebugLevel::Fixme | DebugLevel::Info => info!("{}", message),
                DebugLevel::Memdump | DebugLevel::Count | DebugLevel::Trace => {
                    trace!("{}", message)
                },
                _ => (),
            }
        });
    }

    #[cfg(any(windows, target_os = "macos"))]
    pub fn init() {
        // UWP apps have the working directory set appropriately. Win32 apps
        // do not and need some assistance finding the DLLs.
        let plugin_dir = if cfg!(feature = "uwp") {
            std::path::PathBuf::new()
        } else {
            let mut plugin_dir = std::env::current_exe().unwrap();
            plugin_dir.pop();
            plugin_dir
        };

        let backend = match GStreamerBackend::init_with_plugins(
            plugin_dir,
            &gstreamer_plugins::GSTREAMER_PLUGINS,
        ) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Error initializing GStreamer: {:?}", e);
                std::process::exit(1);
            },
        };
        ServoMedia::init_with_backend(backend);
        #[cfg(feature = "uwp")]
        {
            set_gstreamer_log_handler();
        }
    }

    #[cfg(not(any(windows, target_os = "macos")))]
    pub fn init() {
        ServoMedia::init::<GStreamerBackend>();
    }
}

#[cfg(feature = "media-dummy")]
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
    embedder_events: Vec<(Option<BrowserId>, EmbedderMsg)>,
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
        RenderNotifier {
            compositor_proxy: compositor_proxy,
        }
    }
}

impl webrender_api::RenderNotifier for RenderNotifier {
    fn clone(&self) -> Box<dyn webrender_api::RenderNotifier> {
        Box::new(RenderNotifier::new(self.compositor_proxy.clone()))
    }

    fn wake_up(&self) {
        self.compositor_proxy
            .recomposite(CompositingReason::NewWebRenderFrame);
    }

    fn new_frame_ready(
        &self,
        _document_id: webrender_api::DocumentId,
        scrolled: bool,
        composite_needed: bool,
        _render_time_ns: Option<u64>,
    ) {
        if scrolled {
            self.compositor_proxy
                .send(Msg::NewScrollFrameReady(composite_needed));
        } else {
            self.wake_up();
        }
    }
}

impl<Window> Servo<Window>
where
    Window: WindowMethods + 'static + ?Sized,
{
    pub fn new(
        mut embedder: Box<dyn EmbedderMethods>,
        window: Rc<Window>,
        user_agent: Option<String>,
    ) -> Servo<Window> {
        // Global configuration options, parsed from the command line.
        let opts = opts::get();

        use std::sync::atomic::Ordering;

        style::context::DEFAULT_DISABLE_STYLE_SHARING_CACHE
            .store(opts.disable_share_style_cache, Ordering::Relaxed);
        style::context::DEFAULT_DUMP_STYLE_STATISTICS
            .store(opts.style_sharing_stats, Ordering::Relaxed);
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
            Some(ua) => ua.into(),
            None => embedder
                .get_user_agent_string()
                .map(Into::into)
                .unwrap_or(default_user_agent_string_for(DEFAULT_USER_AGENT).into()),
        };

        // Initialize surfman
        let webrender_surfman = window.webrender_surfman();

        // Get GL bindings
        let webrender_gl = match webrender_surfman.connection().gl_api() {
            GLApi::GL => unsafe { gl::GlFns::load_with(|s| webrender_surfman.get_proc_address(s)) },
            GLApi::GLES => unsafe {
                gl::GlesFns::load_with(|s| webrender_surfman.get_proc_address(s))
            },
        };

        // Make sure the gl context is made current.
        webrender_surfman.make_gl_context_current().unwrap();
        debug_assert_eq!(webrender_gl.get_error(), gleam::gl::NO_ERROR,);

        // Bind the webrender framebuffer
        let framebuffer_object = webrender_surfman
            .context_surface_info()
            .unwrap_or(None)
            .map(|info| info.framebuffer_object)
            .unwrap_or(0);
        webrender_gl.bind_framebuffer(gleam::gl::FRAMEBUFFER, framebuffer_object);

        // Reserving a namespace to create TopLevelBrowserContextId.
        PipelineNamespace::install(PipelineNamespaceId(0));

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

        let devtools_chan = if opts.devtools_server_enabled {
            Some(devtools::start_server(
                opts.devtools_port,
                embedder_proxy.clone(),
            ))
        } else {
            None
        };

        let coordinates = window.get_coordinates();
        let device_pixel_ratio = coordinates.hidpi_factor.get();
        let viewport_size = coordinates.viewport.size.to_f32() / device_pixel_ratio;

        let (mut webrender, webrender_api_sender) = {
            let mut debug_flags = webrender::DebugFlags::empty();
            debug_flags.set(webrender::DebugFlags::PROFILER_DBG, opts.webrender_stats);

            let render_notifier = Box::new(RenderNotifier::new(compositor_proxy.clone()));

            // Cast from `DeviceIndependentPixel` to `DevicePixel`
            let window_size = Size2D::from_untyped(viewport_size.to_i32().to_untyped());

            webrender::Renderer::new(
                webrender_gl.clone(),
                render_notifier,
                webrender::RendererOptions {
                    device_pixel_ratio,
                    resource_override_path: opts.shaders_dir.clone(),
                    enable_aa: opts.enable_text_antialiasing,
                    debug_flags: debug_flags,
                    precache_flags: if opts.precache_shaders {
                        ShaderPrecacheFlags::FULL_COMPILE
                    } else {
                        ShaderPrecacheFlags::empty()
                    },
                    enable_subpixel_aa: opts.enable_subpixel_text_antialiasing,
                    allow_texture_swizzling: pref!(gfx.texture_swizzling.enabled),
                    clear_color: None,
                    ..Default::default()
                },
                None,
                window_size,
            )
            .expect("Unable to initialize webrender!")
        };

        let webrender_api = webrender_api_sender.create_api();
        let wr_document_layer = 0; //TODO
        let webrender_document =
            webrender_api.add_document(coordinates.framebuffer, wr_document_layer);

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
            output_handler,
        } = WebGLComm::new(
            webrender_surfman.clone(),
            webrender_gl.clone(),
            webrender_api.create_sender(),
            webrender_document,
            external_images.clone(),
            gl_type,
        );

        // Set webrender external image handler for WebGL textures
        external_image_handlers.set_handler(image_handler, WebrenderImageHandlerType::WebGL);

        // Set DOM to texture handler, if enabled.
        if let Some(output_handler) = output_handler {
            webrender.set_output_image_handler(output_handler);
        }

        // Create the WebXR main thread
        let mut webxr_main_thread =
            webxr::MainThreadRegistry::new(event_loop_waker, webxr_layer_grand_manager)
                .expect("Failed to create WebXR device registry");
        if pref!(dom.webxr.enabled) {
            embedder.register_webxr(&mut webxr_main_thread, embedder_proxy.clone());
        }

        let glplayer_threads = match window.get_gl_context() {
            GlContext::Unknown => None,
            _ => {
                let (glplayer_threads, image_handler) =
                    GLPlayerThreads::new(external_images.clone());
                external_image_handlers
                    .set_handler(image_handler, WebrenderImageHandlerType::Media);
                Some(glplayer_threads)
            },
        };

        let wgpu_image_handler = webgpu::WGPUExternalImages::new();
        let wgpu_image_map = wgpu_image_handler.images.clone();
        external_image_handlers.set_handler(
            Box::new(wgpu_image_handler),
            WebrenderImageHandlerType::WebGPU,
        );

        let player_context = WindowGLContext {
            gl_context: window.get_gl_context(),
            native_display: window.get_native_display(),
            gl_api: window.get_gl_api(),
            glplayer_chan: glplayer_threads.as_ref().map(GLPlayerThreads::pipeline),
        };

        webrender.set_external_image_handler(external_image_handlers);

        let event_loop_waker = None;

        // The division by 1 represents the page's default zoom of 100%,
        // and gives us the appropriate CSSPixel type for the viewport.
        let window_size = WindowSizeData {
            initial_viewport: viewport_size / Scale::new(1.0),
            device_pixel_ratio: Scale::new(device_pixel_ratio),
        };

        // Create the constellation, which maintains the engine
        // pipelines, including the script and layout threads, as well
        // as the navigation context.
        let constellation_chan = create_constellation(
            user_agent,
            opts.config_dir.clone(),
            embedder_proxy,
            compositor_proxy.clone(),
            time_profiler_chan.clone(),
            mem_profiler_chan.clone(),
            devtools_chan,
            webrender_document,
            webrender_api_sender,
            webxr_main_thread.registry(),
            player_context,
            Some(webgl_threads),
            glplayer_threads,
            event_loop_waker,
            window_size,
            external_images,
            wgpu_image_map,
        );

        if cfg!(feature = "webdriver") {
            if let Some(port) = opts.webdriver_port {
                webdriver(port, constellation_chan.clone());
            }
        }

        // The compositor coordinates with the client window to create the final
        // rendered page and display it somewhere.
        let compositor = IOCompositor::create(
            window,
            InitialCompositorState {
                sender: compositor_proxy,
                receiver: compositor_receiver,
                constellation_chan: constellation_chan.clone(),
                time_profiler_chan: time_profiler_chan,
                mem_profiler_chan: mem_profiler_chan,
                webrender,
                webrender_document,
                webrender_api,
                webrender_surfman,
                webrender_gl,
                webxr_main_thread,
            },
            opts.output_file.clone(),
            opts.is_running_problem_test,
            opts.exit_after_load,
            opts.convert_mouse_to_touch,
        );

        Servo {
            compositor: compositor,
            constellation_chan: constellation_chan,
            embedder_receiver: embedder_receiver,
            embedder_events: Vec::new(),
            profiler_enabled: false,
            _js_engine_setup: js_engine_setup,
        }
    }

    fn handle_window_event(&mut self, event: WindowEvent) -> bool {
        match event {
            WindowEvent::Idle => {},

            WindowEvent::Refresh => {
                self.compositor.composite();
            },

            WindowEvent::Resize => {
                return self.compositor.on_resize_window_event();
            },

            WindowEvent::AllowNavigationResponse(pipeline_id, allowed) => {
                let msg = ConstellationMsg::AllowNavigationResponse(pipeline_id, allowed);
                if let Err(e) = self.constellation_chan.send(msg) {
                    warn!(
                        "Sending allow navigation to constellation failed ({:?}).",
                        e
                    );
                }
            },

            WindowEvent::LoadUrl(top_level_browsing_context_id, url) => {
                let msg = ConstellationMsg::LoadUrl(top_level_browsing_context_id, url);
                if let Err(e) = self.constellation_chan.send(msg) {
                    warn!("Sending load url to constellation failed ({:?}).", e);
                }
            },

            WindowEvent::ClearCache => {
                let msg = ConstellationMsg::ClearCache;
                if let Err(e) = self.constellation_chan.send(msg) {
                    warn!("Sending clear cache to constellation failed ({:?}).", e);
                }
            },

            WindowEvent::MouseWindowEventClass(mouse_window_event) => {
                self.compositor
                    .on_mouse_window_event_class(mouse_window_event);
            },

            WindowEvent::MouseWindowMoveEventClass(cursor) => {
                self.compositor.on_mouse_window_move_event_class(cursor);
            },

            WindowEvent::Touch(event_type, identifier, location) => {
                self.compositor
                    .on_touch_event(event_type, identifier, location);
            },

            WindowEvent::Wheel(delta, location) => {
                self.compositor.on_wheel_event(delta, location);
            },

            WindowEvent::Scroll(delta, cursor, phase) => {
                self.compositor.on_scroll_event(delta, cursor, phase);
            },

            WindowEvent::Zoom(magnification) => {
                self.compositor.on_zoom_window_event(magnification);
            },

            WindowEvent::ResetZoom => {
                self.compositor.on_zoom_reset_window_event();
            },

            WindowEvent::PinchZoom(magnification) => {
                self.compositor.on_pinch_zoom_window_event(magnification);
            },

            WindowEvent::Navigation(top_level_browsing_context_id, direction) => {
                let msg =
                    ConstellationMsg::TraverseHistory(top_level_browsing_context_id, direction);
                if let Err(e) = self.constellation_chan.send(msg) {
                    warn!("Sending navigation to constellation failed ({:?}).", e);
                }
            },

            WindowEvent::Keyboard(key_event) => {
                let msg = ConstellationMsg::Keyboard(key_event);
                if let Err(e) = self.constellation_chan.send(msg) {
                    warn!("Sending keyboard event to constellation failed ({:?}).", e);
                }
            },

            WindowEvent::IMEDismissed => {
                let msg = ConstellationMsg::IMEDismissed;
                if let Err(e) = self.constellation_chan.send(msg) {
                    warn!(
                        "Sending IMEDismissed event to constellation failed ({:?}).",
                        e
                    );
                }
            },

            WindowEvent::Quit => {
                self.compositor.maybe_start_shutting_down();
            },

            WindowEvent::ExitFullScreen(top_level_browsing_context_id) => {
                let msg = ConstellationMsg::ExitFullScreen(top_level_browsing_context_id);
                if let Err(e) = self.constellation_chan.send(msg) {
                    warn!("Sending exit fullscreen to constellation failed ({:?}).", e);
                }
            },

            WindowEvent::Reload(top_level_browsing_context_id) => {
                let msg = ConstellationMsg::Reload(top_level_browsing_context_id);
                if let Err(e) = self.constellation_chan.send(msg) {
                    warn!("Sending reload to constellation failed ({:?}).", e);
                }
            },

            WindowEvent::ToggleSamplingProfiler(rate, max_duration) => {
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

            WindowEvent::ToggleWebRenderDebug(option) => {
                self.compositor.toggle_webrender_debug(option);
            },

            WindowEvent::CaptureWebRender => {
                self.compositor.capture_webrender();
            },

            WindowEvent::NewBrowser(url, browser_id) => {
                let msg = ConstellationMsg::NewBrowser(url, browser_id);
                if let Err(e) = self.constellation_chan.send(msg) {
                    warn!(
                        "Sending NewBrowser message to constellation failed ({:?}).",
                        e
                    );
                }
            },

            WindowEvent::SelectBrowser(ctx) => {
                let msg = ConstellationMsg::SelectBrowser(ctx);
                if let Err(e) = self.constellation_chan.send(msg) {
                    warn!(
                        "Sending SelectBrowser message to constellation failed ({:?}).",
                        e
                    );
                }
            },

            WindowEvent::CloseBrowser(ctx) => {
                let msg = ConstellationMsg::CloseBrowser(ctx);
                if let Err(e) = self.constellation_chan.send(msg) {
                    warn!(
                        "Sending CloseBrowser message to constellation failed ({:?}).",
                        e
                    );
                }
            },

            WindowEvent::SendError(ctx, e) => {
                let msg = ConstellationMsg::SendError(ctx, e);
                if let Err(e) = self.constellation_chan.send(msg) {
                    warn!(
                        "Sending SendError message to constellation failed ({:?}).",
                        e
                    );
                }
            },

            WindowEvent::MediaSessionAction(a) => {
                let msg = ConstellationMsg::MediaSessionAction(a);
                if let Err(e) = self.constellation_chan.send(msg) {
                    warn!(
                        "Sending MediaSessionAction message to constellation failed ({:?}).",
                        e
                    );
                }
            },

            WindowEvent::ChangeBrowserVisibility(top_level_browsing_context_id, visible) => {
                let msg = ConstellationMsg::ChangeBrowserVisibility(
                    top_level_browsing_context_id,
                    visible,
                );
                if let Err(e) = self.constellation_chan.send(msg) {
                    warn!(
                        "Sending ChangeBrowserVisibility to constellation failed ({:?}).",
                        e
                    );
                }
            },
        }
        return false;
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
                    self.embedder_events.push(event);
                },

                (msg, ShutdownState::NotShuttingDown) => {
                    self.embedder_events.push((top_level_browsing_context, msg));
                },
            }
        }
    }

    pub fn get_events(&mut self) -> Vec<(Option<BrowserId>, EmbedderMsg)> {
        ::std::mem::replace(&mut self.embedder_events, Vec::new())
    }

    pub fn handle_events(&mut self, events: Vec<WindowEvent>) -> bool {
        if self.compositor.receive_messages() {
            self.receive_messages();
        }
        let mut need_resize = false;
        for event in events {
            need_resize |= self.handle_window_event(event);
        }
        if self.compositor.shutdown_state != ShutdownState::FinishedShuttingDown {
            self.compositor.perform_updates();
        } else {
            self.embedder_events.push((None, EmbedderMsg::Shutdown));
        }
        need_resize
    }

    pub fn repaint_synchronously(&mut self) {
        self.compositor.repaint_synchronously()
    }

    pub fn pinch_zoom_level(&self) -> f32 {
        self.compositor.pinch_zoom_level()
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
}

fn create_embedder_channel(
    event_loop_waker: Box<dyn EventLoopWaker>,
) -> (EmbedderProxy, EmbedderReceiver) {
    let (sender, receiver) = unbounded();
    (
        EmbedderProxy {
            sender: sender,
            event_loop_waker: event_loop_waker,
        },
        EmbedderReceiver { receiver: receiver },
    )
}

fn create_compositor_channel(
    event_loop_waker: Box<dyn EventLoopWaker>,
) -> (CompositorProxy, CompositorReceiver) {
    let (sender, receiver) = unbounded();
    (
        CompositorProxy {
            sender: sender,
            event_loop_waker: event_loop_waker,
        },
        CompositorReceiver { receiver: receiver },
    )
}

fn create_constellation(
    user_agent: Cow<'static, str>,
    config_dir: Option<PathBuf>,
    embedder_proxy: EmbedderProxy,
    compositor_proxy: CompositorProxy,
    time_profiler_chan: time::ProfilerChan,
    mem_profiler_chan: mem::ProfilerChan,
    devtools_chan: Option<Sender<devtools_traits::DevtoolsControlMsg>>,
    webrender_document: webrender_api::DocumentId,
    webrender_api_sender: webrender_api::RenderApiSender,
    webxr_registry: webxr_api::Registry,
    player_context: WindowGLContext,
    webgl_threads: Option<WebGLThreads>,
    glplayer_threads: Option<GLPlayerThreads>,
    event_loop_waker: Option<Box<dyn EventLoopWaker>>,
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
        devtools_chan.clone(),
        time_profiler_chan.clone(),
        mem_profiler_chan.clone(),
        embedder_proxy.clone(),
        config_dir,
        opts.certificate_path.clone(),
    );

    let font_cache_thread = FontCacheThread::new(
        public_resource_threads.sender(),
        Box::new(FontCacheWR(compositor_proxy.clone())),
    );

    let (canvas_chan, ipc_canvas_chan) = CanvasPaintThread::start(
        Box::new(CanvasWebrenderApi(compositor_proxy.clone())),
        font_cache_thread.clone(),
    );

    let initial_state = InitialConstellationState {
        compositor_proxy,
        embedder_proxy,
        devtools_chan,
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
        event_loop_waker,
        user_agent,
        webrender_external_images: external_images,
        wgpu_image_map,
    };

    let constellation_chan = Constellation::<
        script_layout_interface::message::Msg,
        layout_thread::LayoutThread,
        script::script_thread::ScriptThread,
        script::serviceworker_manager::ServiceWorkerManager,
    >::start(
        initial_state,
        initial_window_size,
        opts.random_pipeline_closure_probability,
        opts.random_pipeline_closure_seed,
        opts.is_running_problem_test,
        opts.hard_fail,
        opts.enable_canvas_antialiasing,
        canvas_chan,
        ipc_canvas_chan,
    );

    constellation_chan
}

struct FontCacheWR(CompositorProxy);

impl gfx_traits::WebrenderApi for FontCacheWR {
    fn add_font_instance(
        &self,
        font_key: webrender_api::FontKey,
        size: f32,
    ) -> webrender_api::FontInstanceKey {
        let (sender, receiver) = unbounded();
        let _ = self.0.send(Msg::Webrender(WebrenderMsg::Font(
            WebrenderFontMsg::AddFontInstance(font_key, size, sender),
        )));
        receiver.recv().unwrap()
    }
    fn add_font(&self, data: gfx_traits::FontData) -> webrender_api::FontKey {
        let (sender, receiver) = unbounded();
        let _ = self.0.send(Msg::Webrender(WebrenderMsg::Font(
            WebrenderFontMsg::AddFont(data, sender),
        )));
        receiver.recv().unwrap()
    }
}

#[derive(Clone)]
struct CanvasWebrenderApi(CompositorProxy);

impl canvas_paint_thread::WebrenderApi for CanvasWebrenderApi {
    fn generate_key(&self) -> Result<webrender_api::ImageKey, ()> {
        let (sender, receiver) = unbounded();
        let _ = self.0.send(Msg::Webrender(WebrenderMsg::Canvas(
            WebrenderCanvasMsg::GenerateKey(sender),
        )));
        receiver.recv().map_err(|_| ())
    }
    fn update_images(&self, updates: Vec<canvas_paint_thread::ImageUpdate>) {
        let _ = self.0.send(Msg::Webrender(WebrenderMsg::Canvas(
            WebrenderCanvasMsg::UpdateImages(updates),
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

            content.start_all::<script_layout_interface::message::Msg,
                                             layout_thread::LayoutThread,
                                             script::script_thread::ScriptThread>(
                                                 true,
                                                 background_hang_monitor_register,
                                                 None,
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
    not(target_arch = "aarch64")
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
    target_arch = "aarch64"
))]
fn create_sandbox() {
    panic!("Sandboxing is not supported on Windows, iOS, ARM targets and android.");
}

enum UserAgent {
    Desktop,
    Android,
    #[allow(non_camel_case_types)]
    iOS,
}

fn default_user_agent_string_for(agent: UserAgent) -> &'static str {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    const DESKTOP_UA_STRING: &'static str =
        "Mozilla/5.0 (X11; Linux x86_64; rv:78.0) Servo/1.0 Firefox/78.0";
    #[cfg(all(target_os = "linux", not(target_arch = "x86_64")))]
    const DESKTOP_UA_STRING: &'static str =
        "Mozilla/5.0 (X11; Linux i686; rv:78.0) Servo/1.0 Firefox/78.0";

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    const DESKTOP_UA_STRING: &'static str =
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:78.0) Servo/1.0 Firefox/78.0";
    #[cfg(all(target_os = "windows", not(target_arch = "x86_64")))]
    const DESKTOP_UA_STRING: &'static str =
        "Mozilla/5.0 (Windows NT 10.0; rv:78.0) Servo/1.0 Firefox/78.0";

    #[cfg(target_os = "macos")]
    const DESKTOP_UA_STRING: &'static str =
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:78.0) Servo/1.0 Firefox/78.0";

    match agent {
        UserAgent::Desktop => DESKTOP_UA_STRING,
        UserAgent::Android => "Mozilla/5.0 (Android; Mobile; rv:68.0) Servo/1.0 Firefox/68.0",
        UserAgent::iOS => {
            "Mozilla/5.0 (iPhone; CPU iPhone OS 13_6 like Mac OS X; rv:78.0) Servo/1.0 Firefox/78.0"
        },
    }
}

#[cfg(target_os = "android")]
const DEFAULT_USER_AGENT: UserAgent = UserAgent::Android;

#[cfg(target_os = "ios")]
const DEFAULT_USER_AGENT: UserAgent = UserAgent::iOS;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
const DEFAULT_USER_AGENT: UserAgent = UserAgent::Desktop;
