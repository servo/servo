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

mod clipboard_delegate;
mod proxies;
mod servo_delegate;
mod webview;
mod webview_delegate;

use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::cmp::max;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex};
use std::thread;

pub use base::id::TopLevelBrowsingContextId;
use base::id::{PipelineNamespace, PipelineNamespaceId, WebViewId};
#[cfg(feature = "bluetooth")]
use bluetooth::BluetoothThreadFactory;
#[cfg(feature = "bluetooth")]
use bluetooth_traits::BluetoothRequest;
use canvas::canvas_paint_thread::CanvasPaintThread;
use canvas::WebGLComm;
use canvas_traits::webgl::{GlType, WebGLThreads};
use clipboard_delegate::StringRequest;
use compositing::windowing::{EmbedderMethods, WindowMethods};
use compositing::{IOCompositor, InitialCompositorState};
use compositing_traits::{CompositorMsg, CompositorProxy, CompositorReceiver, ConstellationMsg};
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
use crossbeam_channel::{unbounded, Receiver, Sender};
pub use embedder_traits::*;
use env_logger::Builder as EnvLoggerBuilder;
use euclid::Scale;
use fonts::SystemFontService;
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
use gleam::gl::RENDERER;
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
pub use keyboard_types::*;
#[cfg(feature = "layout_2013")]
pub use layout_thread_2013;
use log::{debug, warn, Log, Metadata, Record};
use media::{GlApi, NativeDisplay, WindowGLContext};
use net::protocols::ProtocolRegistry;
use net::resource_thread::new_resource_threads;
use profile::{mem as profile_mem, time as profile_time};
use profile_traits::{mem, time};
use script::{JSEngineSetup, ServiceWorkerManager};
use script_layout_interface::LayoutFactory;
use script_traits::{ScriptToConstellationChan, WindowSizeData};
use servo_config::opts::Opts;
use servo_config::prefs::Preferences;
use servo_config::{opts, pref, prefs};
use servo_delegate::DefaultServoDelegate;
use servo_media::player::context::GlContext;
use servo_media::ServoMedia;
use servo_url::ServoUrl;
#[cfg(feature = "webgpu")]
pub use webgpu;
#[cfg(feature = "webgpu")]
use webgpu::swapchain::WGPUImageMap;
use webrender::{RenderApiSender, ShaderPrecacheFlags, UploadMethod, ONE_TIME_USAGE_HINT};
use webrender_api::{ColorF, DocumentId, FramePublishId};
pub use webrender_traits::rendering_context::{
    OffscreenRenderingContext, RenderingContext, SoftwareRenderingContext, SurfmanRenderingContext,
    WindowRenderingContext,
};
use webrender_traits::{
    CrossProcessCompositorApi, WebrenderExternalImageHandlers, WebrenderExternalImageRegistry,
    WebrenderImageHandlerType,
};
use webview::WebViewInner;
#[cfg(feature = "webxr")]
pub use webxr;
pub use {
    background_hang_monitor, base, canvas, canvas_traits, compositing, devtools, devtools_traits,
    euclid, fonts, ipc_channel, layout_thread_2020, media, net, net_traits, profile,
    profile_traits, script, script_layout_interface, script_traits, servo_config as config,
    servo_config, servo_geometry, servo_url, style, style_traits, webrender_api,
};
#[cfg(feature = "bluetooth")]
pub use {bluetooth, bluetooth_traits};

use crate::proxies::ConstellationProxy;
pub use crate::servo_delegate::{ServoDelegate, ServoError};
pub use crate::webview::WebView;
pub use crate::webview_delegate::{
    AllowOrDenyRequest, AuthenticationRequest, NavigationRequest, PermissionRequest,
    WebViewDelegate,
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
pub struct Servo {
    delegate: RefCell<Rc<dyn ServoDelegate>>,
    compositor: Rc<RefCell<IOCompositor>>,
    constellation_proxy: ConstellationProxy,
    embedder_receiver: Receiver<EmbedderMsg>,
    /// Tracks whether we are in the process of shutting down, or have shut down.
    /// This is shared with `WebView`s and the `ServoRenderer`.
    shutdown_state: Rc<Cell<ShutdownState>>,
    /// A map  [`WebView`]s that are managed by this [`Servo`] instance. These are stored
    /// as `Weak` references so that the embedding application can control their lifetime.
    /// When accessed, `Servo` will be reponsible for cleaning up the invalid `Weak`
    /// references.
    webviews: RefCell<HashMap<WebViewId, Weak<RefCell<WebViewInner>>>>,
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
        document_id: DocumentId,
        _scrolled: bool,
        composite_needed: bool,
        _frame_publish_id: FramePublishId,
    ) {
        self.compositor_proxy
            .send(CompositorMsg::NewWebRenderFrameReady(
                document_id,
                composite_needed,
            ));
    }
}

impl Servo {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip(preferences, rendering_context, embedder, window),
            fields(servo_profiling = true),
            level = "trace",
        )
    )]
    pub fn new(
        opts: Opts,
        preferences: Preferences,
        rendering_context: Rc<dyn RenderingContext>,
        mut embedder: Box<dyn EmbedderMethods>,
        window: Rc<dyn WindowMethods>,
        user_agent: Option<String>,
    ) -> Self {
        // Global configuration options, parsed from the command line.
        opts::set_options(opts);
        let opts = opts::get();

        // Set the preferences globally.
        // TODO: It would be better to make these private to a particular Servo instance.
        servo_config::prefs::set(preferences);

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

        // Get GL bindings
        let webrender_gl = rendering_context.gl_api();

        // Make sure the gl context is made current.
        if let Err(err) = rendering_context.make_current() {
            warn!("Failed to make the rendering context current: {:?}", err);
        }
        debug_assert_eq!(webrender_gl.get_error(), gleam::gl::NO_ERROR,);

        // Reserving a namespace to create TopLevelBrowsingContextId.
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

        let devtools_sender = if pref!(devtools_server_enabled) {
            Some(devtools::start_server(
                pref!(devtools_server_port) as u16,
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

            rendering_context.prepare_for_rendering();
            let render_notifier = Box::new(RenderNotifier::new(compositor_proxy.clone()));
            let clear_color = servo_config::pref!(shell_background_color_rgba);
            let clear_color = ColorF::new(
                clear_color[0] as f32,
                clear_color[1] as f32,
                clear_color[2] as f32,
                clear_color[3] as f32,
            );

            // Use same texture upload method as Gecko with ANGLE:
            // https://searchfox.org/mozilla-central/source/gfx/webrender_bindings/src/bindings.rs#1215-1219
            let upload_method = if webrender_gl.get_string(RENDERER).starts_with("ANGLE") {
                UploadMethod::Immediate
            } else {
                UploadMethod::PixelBuffer(ONE_TIME_USAGE_HINT)
            };
            let worker_threads = thread::available_parallelism()
                .map(|i| i.get())
                .unwrap_or(pref!(threadpools_fallback_worker_num) as usize)
                .min(pref!(threadpools_webrender_workers_max).max(1) as usize);
            let workers = Some(Arc::new(
                rayon::ThreadPoolBuilder::new()
                    .num_threads(worker_threads)
                    .thread_name(|idx| format!("WRWorker#{}", idx))
                    .build()
                    .unwrap(),
            ));
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
                    debug_flags,
                    precache_flags: if pref!(gfx_precache_shaders) {
                        ShaderPrecacheFlags::FULL_COMPILE
                    } else {
                        ShaderPrecacheFlags::empty()
                    },
                    enable_aa: pref!(gfx_text_antialiasing_enabled),
                    enable_subpixel_aa: pref!(gfx_subpixel_text_antialiasing_enabled),
                    allow_texture_swizzling: pref!(gfx_texture_swizzling_enabled),
                    clear_color,
                    upload_method,
                    workers,
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
            gleam::gl::GlType::Gl => GlType::Gl,
            gleam::gl::GlType::Gles => GlType::Gles,
        };

        let (external_image_handlers, external_images) = WebrenderExternalImageHandlers::new();
        let mut external_image_handlers = Box::new(external_image_handlers);

        let WebGLComm {
            webgl_threads,
            #[cfg(feature = "webxr")]
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
        #[cfg(feature = "webxr")]
        let mut webxr_main_thread =
            webxr::MainThreadRegistry::new(event_loop_waker, webxr_layer_grand_manager)
                .expect("Failed to create WebXR device registry");
        #[cfg(feature = "webxr")]
        if pref!(dom_webxr_enabled) {
            embedder.register_webxr(&mut webxr_main_thread, embedder_proxy.clone());
        }

        #[cfg(feature = "webgpu")]
        let wgpu_image_handler = webgpu::WGPUExternalImages::default();
        #[cfg(feature = "webgpu")]
        let wgpu_image_map = wgpu_image_handler.images.clone();
        #[cfg(feature = "webgpu")]
        external_image_handlers.set_handler(
            Box::new(wgpu_image_handler),
            WebrenderImageHandlerType::WebGPU,
        );

        WindowGLContext::initialize_image_handler(
            &mut external_image_handlers,
            external_images.clone(),
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
        let mut protocols = ProtocolRegistry::with_internal_protocols();
        protocols.merge(embedder.get_protocol_handlers());

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
            #[cfg(feature = "webxr")]
            webxr_main_thread.registry(),
            Some(webgl_threads),
            window_size,
            external_images,
            #[cfg(feature = "webgpu")]
            wgpu_image_map,
            protocols,
        );

        if cfg!(feature = "webdriver") {
            if let Some(port) = opts.webdriver_port {
                webdriver(port, constellation_chan.clone());
            }
        }

        // The compositor coordinates with the client window to create the final
        // rendered page and display it somewhere.
        let shutdown_state = Rc::new(Cell::new(ShutdownState::NotShuttingDown));
        let compositor = IOCompositor::new(
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
                #[cfg(feature = "webxr")]
                webxr_main_thread,
                shutdown_state: shutdown_state.clone(),
            },
            opts.debug.convert_mouse_to_touch,
            embedder.get_version_string().unwrap_or_default(),
        );

        Self {
            delegate: RefCell::new(Rc::new(DefaultServoDelegate)),
            compositor: Rc::new(RefCell::new(compositor)),
            constellation_proxy: ConstellationProxy::new(constellation_chan),
            embedder_receiver,
            shutdown_state,
            webviews: Default::default(),
            _js_engine_setup: js_engine_setup,
        }
    }

    pub fn delegate(&self) -> Rc<dyn ServoDelegate> {
        self.delegate.borrow().clone()
    }

    pub fn set_delegate(&self, delegate: Rc<dyn ServoDelegate>) {
        *self.delegate.borrow_mut() = delegate;
    }

    /// **EXPERIMENTAL:** Intialize GL accelerated media playback. This currently only works on a limited number
    /// of platforms. This should be run *before* calling [`Servo::new`] and creating the first [`WebView`].
    pub fn initialize_gl_accelerated_media(display: NativeDisplay, api: GlApi, context: GlContext) {
        WindowGLContext::initialize(display, api, context)
    }

    /// Spin the Servo event loop, which:
    ///
    ///   - Performs updates in the compositor, such as queued pinch zoom events
    ///   - Runs delebgate methods on all `WebView`s and `Servo` itself
    ///   - Maybe update the rendered compositor output, but *without* swapping buffers.
    ///
    /// The return value of this method indicates whether or not Servo, false indicates that Servo
    /// has finished shutting down and you should not spin the event loop any longer.
    pub fn spin_event_loop(&self) -> bool {
        if self.shutdown_state.get() == ShutdownState::FinishedShuttingDown {
            return false;
        }

        self.compositor.borrow_mut().receive_messages();

        // Only handle incoming embedder messages if the compositor hasn't already started shutting down.
        while let Ok(message) = self.embedder_receiver.try_recv() {
            self.handle_embedder_message(message);

            if self.shutdown_state.get() == ShutdownState::FinishedShuttingDown {
                break;
            }
        }

        if self.constellation_proxy.disconnected() {
            self.delegate()
                .notify_error(self, ServoError::LostConnectionWithBackend);
        }

        self.compositor.borrow_mut().perform_updates();
        self.send_new_frame_ready_messages();
        self.clean_up_destroyed_webview_handles();

        if self.shutdown_state.get() == ShutdownState::FinishedShuttingDown {
            return false;
        }

        true
    }

    fn send_new_frame_ready_messages(&self) {
        if !self.compositor.borrow().needs_repaint() {
            return;
        }

        for webview in self
            .webviews
            .borrow()
            .values()
            .filter_map(WebView::from_weak_handle)
        {
            webview.delegate().notify_new_frame_ready(webview);
        }
    }

    fn clean_up_destroyed_webview_handles(&self) {
        // Remove any webview handles that have been destroyed and would not be upgradable.
        // Note that `retain` is O(capacity) because it visits empty buckets, so it may be worth
        // calling `shrink_to_fit` at some point to deal with cases where a long-running Servo
        // instance goes from many open webviews to only a few.
        self.webviews
            .borrow_mut()
            .retain(|_webview_id, webview| webview.strong_count() > 0);
    }

    pub fn pinch_zoom_level(&self) -> f32 {
        self.compositor.borrow_mut().pinch_zoom_level().get()
    }

    pub fn setup_logging(&self) {
        let constellation_chan = self.constellation_proxy.sender();
        let env = env_logger::Env::default();
        let env_logger = EnvLoggerBuilder::from_env(env).build();
        let con_logger = FromCompositorLogger::new(constellation_chan);

        let filter = max(env_logger.filter(), con_logger.filter());
        let logger = BothLogger(env_logger, con_logger);

        log::set_boxed_logger(Box::new(logger)).expect("Failed to set logger.");
        log::set_max_level(filter);
    }

    pub fn start_shutting_down(&self) {
        if self.shutdown_state.get() != ShutdownState::NotShuttingDown {
            warn!("Requested shutdown while already shutting down");
            return;
        }

        debug!("Sending Exit message to Constellation");
        self.constellation_proxy.send(ConstellationMsg::Exit);
        self.shutdown_state.set(ShutdownState::ShuttingDown);
    }

    fn finish_shutting_down(&self) {
        debug!("Servo received message that Constellation shutdown is complete");
        self.shutdown_state.set(ShutdownState::FinishedShuttingDown);
        self.compositor.borrow_mut().finish_shutting_down();
    }

    pub fn deinit(&self) {
        self.compositor.borrow_mut().deinit();
    }

    pub fn new_webview(&self, url: url::Url) -> WebView {
        let webview = WebView::new(&self.constellation_proxy, self.compositor.clone());
        self.webviews
            .borrow_mut()
            .insert(webview.id(), webview.weak_handle());
        self.constellation_proxy
            .send(ConstellationMsg::NewWebView(url.into(), webview.id()));
        webview
    }

    pub fn new_auxiliary_webview(&self) -> WebView {
        let webview = WebView::new(&self.constellation_proxy, self.compositor.clone());
        self.webviews
            .borrow_mut()
            .insert(webview.id(), webview.weak_handle());
        webview
    }

    fn get_webview_handle(&self, id: WebViewId) -> Option<WebView> {
        self.webviews
            .borrow()
            .get(&id)
            .and_then(WebView::from_weak_handle)
    }

    fn handle_embedder_message(&self, message: EmbedderMsg) {
        match message {
            EmbedderMsg::ShutdownComplete => self.finish_shutting_down(),
            EmbedderMsg::Status(webview_id, status_text) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview.set_status_text(status_text);
                }
            },
            EmbedderMsg::ChangePageTitle(webview_id, title) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview.set_page_title(title);
                }
            },
            EmbedderMsg::MoveTo(webview_id, position) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview.delegate().request_move_to(webview, position);
                }
            },
            EmbedderMsg::ResizeTo(webview_id, size) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview.delegate().request_resize_to(webview, size);
                }
            },
            EmbedderMsg::Prompt(webview_id, prompt_definition, prompt_origin) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview
                        .delegate()
                        .show_prompt(webview, prompt_definition, prompt_origin);
                }
            },
            EmbedderMsg::ShowContextMenu(webview_id, ipc_sender, title, items) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview
                        .delegate()
                        .show_context_menu(webview, ipc_sender, title, items);
                }
            },
            EmbedderMsg::AllowNavigationRequest(webview_id, pipeline_id, servo_url) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    let request = NavigationRequest {
                        url: servo_url.into_url(),
                        pipeline_id,
                        constellation_proxy: self.constellation_proxy.clone(),
                        response_sent: false,
                    };
                    webview.delegate().request_navigation(webview, request);
                }
            },
            EmbedderMsg::AllowOpeningWebView(webview_id, response_sender) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    let new_webview = webview.delegate().request_open_auxiliary_webview(webview);
                    let _ = response_sender.send(new_webview.map(|webview| webview.id()));
                }
            },
            EmbedderMsg::WebViewOpened(webview_id) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview.delegate().notify_ready_to_show(webview);
                }
            },
            EmbedderMsg::WebViewClosed(webview_id) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview.delegate().notify_closed(webview);
                }
            },
            EmbedderMsg::WebViewFocused(webview_id) => {
                for id in self.webviews.borrow().keys() {
                    if let Some(webview) = self.get_webview_handle(*id) {
                        let focused = webview.id() == webview_id;
                        webview.set_focused(focused);
                    }
                }
            },
            EmbedderMsg::WebViewBlurred => {
                for id in self.webviews.borrow().keys() {
                    if let Some(webview) = self.get_webview_handle(*id) {
                        webview.set_focused(false);
                    }
                }
            },
            EmbedderMsg::AllowUnload(webview_id, response_sender) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    let request = AllowOrDenyRequest {
                        response_sender,
                        response_sent: false,
                        default_response: AllowOrDeny::Allow,
                    };
                    webview.delegate().request_unload(webview, request);
                }
            },
            EmbedderMsg::Keyboard(webview_id, keyboard_event) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview
                        .delegate()
                        .notify_keyboard_event(webview, keyboard_event);
                }
            },
            EmbedderMsg::ClearClipboard(webview_id) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview.clipboard_delegate().clear(webview);
                }
            },
            EmbedderMsg::GetClipboardText(webview_id, result_sender) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview
                        .clipboard_delegate()
                        .get_text(webview, StringRequest::from(result_sender));
                }
            },
            EmbedderMsg::SetClipboardText(webview_id, string) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview.clipboard_delegate().set_text(webview, string);
                }
            },
            EmbedderMsg::SetCursor(webview_id, cursor) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview.set_cursor(cursor);
                }
            },
            EmbedderMsg::NewFavicon(webview_id, url) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview.set_favicon_url(url.into_url());
                }
            },
            EmbedderMsg::NotifyLoadStatusChanged(webview_id, load_status) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview.set_load_status(load_status);
                }
            },
            EmbedderMsg::HistoryChanged(webview_id, urls, current_index) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    let urls: Vec<_> = urls.into_iter().map(ServoUrl::into_url).collect();
                    let current_url = urls[current_index].clone();

                    webview
                        .delegate()
                        .notify_history_changed(webview.clone(), urls, current_index);
                    webview.set_url(current_url);
                }
            },
            EmbedderMsg::NotifyFullscreenStateChanged(webview_id, fullscreen) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview
                        .delegate()
                        .notify_fullscreen_state_changed(webview, fullscreen);
                }
            },
            EmbedderMsg::WebResourceRequested(
                webview_id,
                web_resource_request,
                response_sender,
            ) => {
                let webview = webview_id.and_then(|webview_id| self.get_webview_handle(webview_id));
                if let Some(webview) = webview.clone() {
                    webview.delegate().intercept_web_resource_load(
                        webview,
                        &web_resource_request,
                        response_sender.clone(),
                    );
                }

                self.delegate().intercept_web_resource_load(
                    webview,
                    &web_resource_request,
                    response_sender,
                );
            },
            EmbedderMsg::Panic(webview_id, reason, backtrace) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview
                        .delegate()
                        .notify_crashed(webview, reason, backtrace);
                }
            },
            EmbedderMsg::GetSelectedBluetoothDevice(webview_id, items, response_sender) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview.delegate().show_bluetooth_device_dialog(
                        webview,
                        items,
                        response_sender,
                    );
                }
            },
            EmbedderMsg::SelectFiles(
                webview_id,
                filter_patterns,
                allow_select_multiple,
                response_sender,
            ) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview.delegate().show_file_selection_dialog(
                        webview,
                        filter_patterns,
                        allow_select_multiple,
                        response_sender,
                    );
                }
            },
            EmbedderMsg::RequestAuthentication(webview_id, url, for_proxy, response_sender) => {
                let authentication_request = AuthenticationRequest {
                    url: url.into_url(),
                    for_proxy,
                    response_sender,
                    response_sent: false,
                };
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview
                        .delegate()
                        .request_authentication(webview, authentication_request);
                }
            },
            EmbedderMsg::PromptPermission(webview_id, requested_feature, response_sender) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    let permission_request = PermissionRequest {
                        requested_feature,
                        allow_deny_request: AllowOrDenyRequest {
                            response_sender,
                            response_sent: false,
                            default_response: AllowOrDeny::Deny,
                        },
                    };
                    webview
                        .delegate()
                        .request_permission(webview, permission_request);
                }
            },
            EmbedderMsg::ShowIME(webview_id, input_method_type, text, multiline, position) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview.delegate().show_ime(
                        webview,
                        input_method_type,
                        text,
                        multiline,
                        position,
                    );
                }
            },
            EmbedderMsg::HideIME(webview_id) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview.delegate().hide_ime(webview);
                }
            },
            EmbedderMsg::ReportProfile(_items) => {},
            EmbedderMsg::MediaSessionEvent(webview_id, media_session_event) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview
                        .delegate()
                        .notify_media_session_event(webview, media_session_event);
                }
            },
            EmbedderMsg::OnDevtoolsStarted(port, token) => match port {
                Ok(port) => self
                    .delegate()
                    .notify_devtools_server_started(self, port, token),
                Err(()) => self
                    .delegate()
                    .notify_error(self, ServoError::DevtoolsFailedToStart),
            },
            EmbedderMsg::RequestDevtoolsConnection(response_sender) => {
                self.delegate().request_devtools_connection(
                    self,
                    AllowOrDenyRequest {
                        response_sender,
                        response_sent: false,
                        default_response: AllowOrDeny::Deny,
                    },
                );
            },
            EmbedderMsg::PlayGamepadHapticEffect(
                webview_id,
                gamepad_index,
                gamepad_haptic_effect_type,
                ipc_sender,
            ) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview.delegate().play_gamepad_haptic_effect(
                        webview,
                        gamepad_index,
                        gamepad_haptic_effect_type,
                        ipc_sender,
                    );
                }
            },
            EmbedderMsg::StopGamepadHapticEffect(webview_id, gamepad_index, ipc_sender) => {
                if let Some(webview) = self.get_webview_handle(webview_id) {
                    webview.delegate().stop_gamepad_haptic_effect(
                        webview,
                        gamepad_index,
                        ipc_sender,
                    );
                }
            },
        }
    }
}

fn create_embedder_channel(
    event_loop_waker: Box<dyn EventLoopWaker>,
) -> (EmbedderProxy, Receiver<EmbedderMsg>) {
    let (sender, receiver) = unbounded();
    (
        EmbedderProxy {
            sender,
            event_loop_waker,
        },
        receiver,
    )
}

fn create_compositor_channel(
    event_loop_waker: Box<dyn EventLoopWaker>,
) -> (CompositorProxy, CompositorReceiver) {
    let (sender, receiver) = unbounded();

    let (compositor_ipc_sender, compositor_ipc_receiver) =
        ipc::channel().expect("ipc channel failure");

    let cross_process_compositor_api = CrossProcessCompositorApi(compositor_ipc_sender);
    let compositor_proxy = CompositorProxy {
        sender,
        cross_process_compositor_api,
        event_loop_waker,
    };

    let compositor_proxy_clone = compositor_proxy.clone();
    ROUTER.add_typed_route(
        compositor_ipc_receiver,
        Box::new(move |message| {
            compositor_proxy_clone.send(CompositorMsg::CrossProcess(
                message.expect("Could not convert Compositor message"),
            ));
        }),
    );

    (compositor_proxy, CompositorReceiver { receiver })
}

fn get_layout_factory(legacy_layout: bool) -> Arc<dyn LayoutFactory> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "layout_2013")] {
            if legacy_layout {
                return Arc::new(layout_thread_2013::LayoutFactoryImpl());
            }
        } else {
            if legacy_layout {
                panic!("Runtime option `legacy_layout` was enabled, but the `layout_2013` \
                feature was not enabled at compile time! ");
           }
        }
    }
    Arc::new(layout_thread_2020::LayoutFactoryImpl())
}

#[allow(clippy::too_many_arguments)]
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
    #[cfg(feature = "webxr")] webxr_registry: webxr_api::Registry,
    webgl_threads: Option<WebGLThreads>,
    initial_window_size: WindowSizeData,
    external_images: Arc<Mutex<WebrenderExternalImageRegistry>>,
    #[cfg(feature = "webgpu")] wgpu_image_map: WGPUImageMap,
    protocols: ProtocolRegistry,
) -> Sender<ConstellationMsg> {
    // Global configuration options, parsed from the command line.
    let opts = opts::get();

    #[cfg(feature = "bluetooth")]
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
        Arc::new(protocols),
    );

    let system_font_service = Arc::new(
        SystemFontService::spawn(compositor_proxy.cross_process_compositor_api.clone()).to_proxy(),
    );

    let (canvas_create_sender, canvas_ipc_sender) = CanvasPaintThread::start(
        compositor_proxy.cross_process_compositor_api.clone(),
        system_font_service.clone(),
        public_resource_threads.clone(),
    );

    let initial_state = InitialConstellationState {
        compositor_proxy,
        embedder_proxy,
        devtools_sender,
        #[cfg(feature = "bluetooth")]
        bluetooth_thread,
        system_font_service,
        public_resource_threads,
        private_resource_threads,
        time_profiler_chan,
        mem_profiler_chan,
        webrender_document,
        webrender_api_sender,
        #[cfg(feature = "webxr")]
        webxr_registry: Some(webxr_registry),
        #[cfg(not(feature = "webxr"))]
        webxr_registry: None,
        webgl_threads,
        user_agent,
        webrender_external_images: external_images,
        #[cfg(feature = "webgpu")]
        wgpu_image_map,
    };

    let layout_factory: Arc<dyn LayoutFactory> = get_layout_factory(opts::get().legacy_layout);

    Constellation::<script::ScriptThread, script::ServiceWorkerManager>::start(
        initial_state,
        layout_factory,
        initial_window_size,
        opts.random_pipeline_closure_probability,
        opts.random_pipeline_closure_seed,
        opts.hard_fail,
        canvas_create_sender,
        canvas_ipc_sender,
    )
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
    prefs::set(unprivileged_content.prefs().clone());

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

            content.start_all::<script::ScriptThread>(
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

fn get_servo_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn default_user_agent_string_for(agent: UserAgent) -> String {
    let servo_version = get_servo_version();

    #[cfg(all(target_os = "linux", target_arch = "x86_64", not(target_env = "ohos")))]
    let desktop_ua_string =
        format!("Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Servo/{servo_version} Firefox/128.0");
    #[cfg(all(
        target_os = "linux",
        not(target_arch = "x86_64"),
        not(target_env = "ohos")
    ))]
    let desktop_ua_string =
        format!("Mozilla/5.0 (X11; Linux i686; rv:128.0) Servo/{servo_version} Firefox/128.0");

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    let desktop_ua_string = format!(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:128.0) Servo/{servo_version} Firefox/128.0"
    );
    #[cfg(all(target_os = "windows", not(target_arch = "x86_64")))]
    let desktop_ua_string =
        format!("Mozilla/5.0 (Windows NT 10.0; rv:128.0) Servo/{servo_version} Firefox/128.0");

    #[cfg(target_os = "macos")]
    let desktop_ua_string = format!(
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:128.0) Servo/{servo_version} Firefox/128.0"
    );

    #[cfg(any(target_os = "android", target_env = "ohos"))]
    let desktop_ua_string = "".to_string();

    match agent {
        UserAgent::Desktop => desktop_ua_string,
        UserAgent::Android => format!(
            "Mozilla/5.0 (Android; Mobile; rv:128.0) Servo/{servo_version} Firefox/128.0"
        ),
        UserAgent::OpenHarmony => format!(
            "Mozilla/5.0 (OpenHarmony; Mobile; rv:128.0) Servo/{servo_version} Firefox/128.0"
        ),
        UserAgent::iOS => format!(
            "Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X; rv:128.0) Servo/{servo_version} Firefox/128.0"
        ),
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
