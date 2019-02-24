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
pub use debugger;
pub use devtools;
pub use devtools_traits;
pub use embedder_traits;
pub use euclid;
pub use gfx;
pub use ipc_channel;
pub use layout_thread;
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
pub use webrender_api;
pub use webvr;
pub use webvr_traits;

#[cfg(feature = "webdriver")]
fn webdriver(port: u16, constellation: Sender<ConstellationMsg>) {
    webdriver_server::start_server(port, constellation);
}

#[cfg(not(feature = "webdriver"))]
fn webdriver(_port: u16, _constellation: Sender<ConstellationMsg>) {}

use bluetooth::BluetoothThreadFactory;
use bluetooth_traits::BluetoothRequest;
use canvas::gl_context::GLContextFactory;
use canvas::webgl_thread::WebGLThreads;
use compositing::compositor_thread::{
    CompositorProxy, CompositorReceiver, InitialCompositorState, Msg,
};
use compositing::windowing::{EmbedderMethods, WindowEvent, WindowMethods};
use compositing::{CompositingReason, IOCompositor, ShutdownState};
#[cfg(all(
    not(target_os = "windows"),
    not(target_os = "ios"),
    not(target_os = "android"),
    not(target_arch = "arm"),
    not(target_arch = "aarch64")
))]
use constellation::content_process_sandbox_profile;
use constellation::{Constellation, InitialConstellationState, UnprivilegedPipelineContent};
use constellation::{FromCompositorLogger, FromScriptLogger};
use crossbeam_channel::{unbounded, Sender};
use embedder_traits::{EmbedderMsg, EmbedderProxy, EmbedderReceiver, EventLoopWaker};
use env_logger::Builder as EnvLoggerBuilder;
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
use msg::constellation_msg::{PipelineNamespace, PipelineNamespaceId};
use net::resource_thread::new_resource_threads;
use net_traits::IpcSend;
use profile::mem as profile_mem;
use profile::time as profile_time;
use profile_traits::mem;
use profile_traits::time;
use script_traits::{ConstellationMsg, SWManagerSenders, ScriptToConstellationChan};
use servo_config::opts;
use servo_config::{pref, prefs};
use servo_media::ServoMedia;
use std::borrow::Cow;
use std::cmp::max;
use std::path::PathBuf;
use std::rc::Rc;
use webrender::{RendererKind, ShaderPrecacheFlags};
use webvr::{VRServiceManager, WebVRCompositorHandler, WebVRThread};

pub use gleam::gl;
pub use keyboard_types;
pub use msg::constellation_msg::TopLevelBrowsingContextId as BrowserId;
pub use servo_config as config;
pub use servo_url as url;

#[cfg(any(
    all(target_os = "android", target_arch = "arm"),
    target_arch = "x86_64"
))]
mod media_platform {
    pub use self::servo_media_gstreamer::GStreamerBackend as MediaBackend;
    use servo_media_gstreamer;
}

#[cfg(not(any(
    all(target_os = "android", target_arch = "arm"),
    target_arch = "x86_64"
)))]
mod media_platform {
    pub use self::servo_media_dummy::DummyBackend as MediaBackend;
    use servo_media_dummy;
}

type MediaBackend = media_platform::MediaBackend;

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
    pub fn new(embedder: Box<EmbedderMethods>, window: Rc<Window>) -> Servo<Window> {
        // Global configuration options, parsed from the command line.
        let opts = opts::get();

        if !opts.multiprocess {
            ServoMedia::init::<MediaBackend>();
        }

        // Make sure the gl context is made current.
        window.prepare_for_composite();

        // Reserving a namespace to create TopLevelBrowserContextId.
        PipelineNamespace::install(PipelineNamespaceId(0));

        // Get both endpoints of a special channel for communication between
        // the client window and the compositor. This channel is unique because
        // messages to client may need to pump a platform-specific event loop
        // to deliver the message.
        let (compositor_proxy, compositor_receiver) =
            create_compositor_channel(embedder.create_event_loop_waker());
        let (embedder_proxy, embedder_receiver) =
            create_embedder_channel(embedder.create_event_loop_waker());
        let time_profiler_chan = profile_time::Profiler::create(
            &opts.time_profiling,
            opts.time_profiler_trace_path.clone(),
        );
        let mem_profiler_chan = profile_mem::Profiler::create(opts.mem_profiler_period);
        let debugger_chan = opts.debugger_port.map(|port| debugger::start_server(port));
        let devtools_chan = opts.devtools_port.map(|port| devtools::start_server(port));

        let coordinates = window.get_coordinates();

        let (mut webrender, webrender_api_sender) = {
            let renderer_kind = if opts::get().should_use_osmesa() {
                RendererKind::OSMesa
            } else {
                RendererKind::Native
            };

            let recorder = if opts.webrender_record {
                let record_path = PathBuf::from("wr-record.bin");
                let recorder = Box::new(webrender::BinaryRecorder::new(&record_path));
                Some(recorder as Box<dyn webrender::ApiRecordingReceiver>)
            } else {
                None
            };

            let mut debug_flags = webrender::DebugFlags::empty();
            debug_flags.set(webrender::DebugFlags::PROFILER_DBG, opts.webrender_stats);

            let render_notifier = Box::new(RenderNotifier::new(compositor_proxy.clone()));

            webrender::Renderer::new(
                window.gl(),
                render_notifier,
                webrender::RendererOptions {
                    device_pixel_ratio: coordinates.hidpi_factor.get(),
                    resource_override_path: opts.shaders_dir.clone(),
                    enable_aa: opts.enable_text_antialiasing,
                    debug_flags: debug_flags,
                    recorder: recorder,
                    precache_flags: if opts.precache_shaders {
                        ShaderPrecacheFlags::FULL_COMPILE
                    } else {
                        ShaderPrecacheFlags::empty()
                    },
                    renderer_kind: renderer_kind,
                    enable_subpixel_aa: opts.enable_subpixel_text_antialiasing,
                    clear_color: None,
                    ..Default::default()
                },
                None,
            )
            .expect("Unable to initialize webrender!")
        };

        let webrender_api = webrender_api_sender.create_api();
        let wr_document_layer = 0; //TODO
        let webrender_document =
            webrender_api.add_document(coordinates.framebuffer, wr_document_layer);

        // Important that this call is done in a single-threaded fashion, we
        // can't defer it after `create_constellation` has started.
        script::init();

        let mut webvr_heartbeats = Vec::new();
        let webvr_services = if pref!(dom.webvr.enabled) {
            let mut services = VRServiceManager::new();
            services.register_defaults();
            embedder.register_vr_services(&mut services, &mut webvr_heartbeats);
            Some(services)
        } else {
            None
        };

        // Create the constellation, which maintains the engine
        // pipelines, including the script and layout threads, as well
        // as the navigation context.
        let (constellation_chan, sw_senders) = create_constellation(
            opts.user_agent.clone(),
            opts.config_dir.clone(),
            embedder_proxy.clone(),
            compositor_proxy.clone(),
            time_profiler_chan.clone(),
            mem_profiler_chan.clone(),
            debugger_chan,
            devtools_chan,
            &mut webrender,
            webrender_document,
            webrender_api_sender,
            window.gl(),
            webvr_services,
        );

        // Send the constellation's swmanager sender to service worker manager thread
        script::init_service_workers(sw_senders);

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
                webvr_heartbeats,
            },
        );

        Servo {
            compositor: compositor,
            constellation_chan: constellation_chan,
            embedder_receiver: embedder_receiver,
            embedder_events: Vec::new(),
            profiler_enabled: false,
        }
    }

    fn handle_window_event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::Idle => {},

            WindowEvent::Refresh => {
                self.compositor.composite();
            },

            WindowEvent::Resize => {
                self.compositor.on_resize_window_event();
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

    pub fn handle_events(&mut self, events: Vec<WindowEvent>) {
        if self.compositor.receive_messages() {
            self.receive_messages();
        }
        for event in events {
            self.handle_window_event(event);
        }
        if self.compositor.shutdown_state != ShutdownState::FinishedShuttingDown {
            self.compositor.perform_updates();
        } else {
            self.embedder_events.push((None, EmbedderMsg::Shutdown));
        }
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
    debugger_chan: Option<debugger::Sender>,
    devtools_chan: Option<Sender<devtools_traits::DevtoolsControlMsg>>,
    webrender: &mut webrender::Renderer,
    webrender_document: webrender_api::DocumentId,
    webrender_api_sender: webrender_api::RenderApiSender,
    window_gl: Rc<dyn gl::Gl>,
    webvr_services: Option<VRServiceManager>,
) -> (Sender<ConstellationMsg>, SWManagerSenders) {
    let bluetooth_thread: IpcSender<BluetoothRequest> =
        BluetoothThreadFactory::new(embedder_proxy.clone());

    let (public_resource_threads, private_resource_threads) = new_resource_threads(
        user_agent,
        devtools_chan.clone(),
        time_profiler_chan.clone(),
        mem_profiler_chan.clone(),
        embedder_proxy.clone(),
        config_dir,
    );
    let font_cache_thread = FontCacheThread::new(
        public_resource_threads.sender(),
        webrender_api_sender.create_api(),
    );

    let resource_sender = public_resource_threads.sender();

    let (webvr_chan, webvr_constellation_sender, webvr_compositor) =
        if let Some(services) = webvr_services {
            // WebVR initialization
            let (mut handler, sender) = WebVRCompositorHandler::new();
            let (webvr_thread, constellation_sender) = WebVRThread::spawn(sender, services);
            handler.set_webvr_thread_sender(webvr_thread.clone());
            (
                Some(webvr_thread),
                Some(constellation_sender),
                Some(handler),
            )
        } else {
            (None, None, None)
        };

    // GLContext factory used to create WebGL Contexts
    let gl_factory = if opts::get().should_use_osmesa() {
        GLContextFactory::current_osmesa_handle()
    } else {
        GLContextFactory::current_native_handle(&compositor_proxy)
    };

    // Initialize WebGL Thread entry point.
    let webgl_threads = gl_factory.map(|factory| {
        let (webgl_threads, image_handler, output_handler) = WebGLThreads::new(
            factory,
            window_gl,
            webrender_api_sender.clone(),
            webvr_compositor.map(|c| c as Box<_>),
        );

        // Set webrender external image handler for WebGL textures
        webrender.set_external_image_handler(image_handler);

        // Set DOM to texture handler, if enabled.
        if let Some(output_handler) = output_handler {
            webrender.set_output_image_handler(output_handler);
        }

        webgl_threads
    });

    let initial_state = InitialConstellationState {
        compositor_proxy,
        embedder_proxy,
        debugger_chan,
        devtools_chan,
        bluetooth_thread,
        font_cache_thread,
        public_resource_threads,
        private_resource_threads,
        time_profiler_chan,
        mem_profiler_chan,
        webrender_document,
        webrender_api_sender,
        webgl_threads,
        webvr_chan,
    };
    let (constellation_chan, from_swmanager_sender) = Constellation::<
        script_layout_interface::message::Msg,
        layout_thread::LayoutThread,
        script::script_thread::ScriptThread,
    >::start(initial_state);

    if let Some(webvr_constellation_sender) = webvr_constellation_sender {
        // Set constellation channel used by WebVR thread to broadcast events
        webvr_constellation_sender
            .send(constellation_chan.clone())
            .unwrap();
    }

    // channels to communicate with Service Worker Manager
    let sw_senders = SWManagerSenders {
        swmanager_sender: from_swmanager_sender,
        resource_sender: resource_sender,
    };

    (constellation_chan, sw_senders)
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
        ipc::channel::<UnprivilegedPipelineContent>().unwrap();
    let connection_bootstrap: IpcSender<IpcSender<UnprivilegedPipelineContent>> =
        IpcSender::connect(token).unwrap();
    connection_bootstrap
        .send(unprivileged_content_sender)
        .unwrap();

    let mut unprivileged_content = unprivileged_content_receiver.recv().unwrap();
    opts::set_options(unprivileged_content.opts());
    prefs::pref_map()
        .set_all(unprivileged_content.prefs())
        .expect("Failed to set preferences");
    set_logger(unprivileged_content.script_to_constellation_chan().clone());

    // Enter the sandbox if necessary.
    if opts::get().sandbox {
        create_sandbox();
    }

    let background_hang_monitor_register =
        unprivileged_content.register_with_background_hang_monitor();

    // send the required channels to the service worker manager
    let sw_senders = unprivileged_content.swmanager_senders();
    script::init();
    script::init_service_workers(sw_senders);

    ServoMedia::init::<MediaBackend>();

    unprivileged_content.start_all::<script_layout_interface::message::Msg,
                                     layout_thread::LayoutThread,
                                     script::script_thread::ScriptThread>(
                                         true,
                                         background_hang_monitor_register
                                     );
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
