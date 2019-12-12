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
pub use webrender_traits;
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
use canvas::WebGLComm;
use canvas_traits::webgl::WebGLThreads;
use compositing::compositor_thread::{
    CompositorProxy, CompositorReceiver, InitialCompositorState, Msg,
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
use constellation::{Constellation, InitialConstellationState, UnprivilegedPipelineContent};
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
use script::JSEngineSetup;
use script_traits::{SWManagerSenders, ScriptToConstellationChan, WindowSizeData};
use servo_config::opts;
use servo_config::{pref, prefs};
use servo_media::player::context::GlContext;
use servo_media::ServoMedia;
use std::borrow::Cow;
use std::cmp::max;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
#[cfg(not(target_os = "windows"))]
use surfman::platform::default::device::Device as HWDevice;
#[cfg(not(target_os = "windows"))]
use surfman::platform::generic::osmesa::device::Device as SWDevice;
#[cfg(not(target_os = "windows"))]
use surfman::platform::generic::universal::context::Context;
use surfman::platform::generic::universal::device::Device;
use webrender::{RendererKind, ShaderPrecacheFlags};
use webrender_traits::WebrenderImageHandlerType;
use webrender_traits::{WebrenderExternalImageHandlers, WebrenderExternalImageRegistry};
use webvr::{VRServiceManager, WebVRCompositorHandler, WebVRThread};
use webvr_traits::WebVRMsg;

pub use gleam::gl;
pub use keyboard_types;
pub use msg::constellation_msg::TopLevelBrowsingContextId as BrowserId;
pub use servo_config as config;
pub use servo_url as url;

#[cfg(feature = "media-gstreamer")]
mod media_platform {
    use super::ServoMedia;
    use servo_media_gstreamer::GStreamerBackend;

    #[cfg(target_os = "windows")]
    fn set_gstreamer_log_handler() {
        use gstreamer::{debug_add_log_function, debug_remove_default_log_function, DebugLevel};

        debug_remove_default_log_function();
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

    #[cfg(windows)]
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

        let uwp_plugins = [
            "gstapp.dll",
            "gstaudioconvert.dll",
            "gstaudiofx.dll",
            "gstaudioparsers.dll",
            "gstaudioresample.dll",
            "gstautodetect.dll",
            "gstcoreelements.dll",
            "gstdeinterlace.dll",
            "gstinterleave.dll",
            "gstisomp4.dll",
            "gstlibav.dll",
            "gstplayback.dll",
            "gstproxy.dll",
            "gsttypefindfunctions.dll",
            "gstvideoconvert.dll",
            "gstvideofilter.dll",
            "gstvideoparsersbad.dll",
            "gstvideoscale.dll",
            "gstvolume.dll",
            "gstwasapi.dll",
        ];

        let non_uwp_plugins = [
            "gstmatroska.dll",
            "gstnice.dll",
            "gstogg.dll",
            "gstopengl.dll",
            "gstopus.dll",
            "gstrtp.dll",
            "gsttheora.dll",
            "gstvorbis.dll",
            "gstvpx.dll",
            "gstwebrtc.dll",
        ];

        let plugins: Vec<_> = if cfg!(feature = "uwp") {
            uwp_plugins.to_vec()
        } else {
            uwp_plugins
                .iter()
                .map(|&s| s)
                .chain(non_uwp_plugins.iter().map(|&s| s))
                .collect()
        };

        let backend = match GStreamerBackend::init_with_plugins(plugin_dir, &plugins) {
            Ok(b) => b,
            Err(e) => {
                error!("Error initializing GStreamer: {:?}", e);
                panic!()
            },
        };
        ServoMedia::init_with_backend(backend);
        if cfg!(feature = "uwp") {
            set_gstreamer_log_handler();
        }
    }

    #[cfg(not(windows))]
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
    pub fn new(mut embedder: Box<dyn EmbedderMethods>, window: Rc<Window>) -> Servo<Window> {
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

        // Make sure the gl context is made current.
        window.make_gl_context_current();

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
            opts.profile_heartbeats,
        );
        let mem_profiler_chan = profile_mem::Profiler::create(opts.mem_profiler_period);

        let debugger_chan = opts.debugger_port.map(|port| debugger::start_server(port));
        let devtools_chan = opts
            .devtools_port
            .map(|port| devtools::start_server(port, embedder_proxy.clone()));

        let coordinates = window.get_coordinates();
        let device_pixel_ratio = coordinates.hidpi_factor.get();
        let viewport_size = coordinates.viewport.size.to_f32() / device_pixel_ratio;

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

            // Cast from `DeviceIndependentPixel` to `DevicePixel`
            let window_size = Size2D::from_untyped(viewport_size.to_i32().to_untyped());

            webrender::Renderer::new(
                window.gl(),
                render_notifier,
                webrender::RendererOptions {
                    device_pixel_ratio,
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

        if pref!(dom.webxr.enabled) && pref!(dom.webvr.enabled) {
            panic!("We don't currently support running both WebVR and WebXR");
        }

        // For the moment, we enable use both the webxr crate and the rust-webvr crate,
        // but we are migrating over to just using webxr.
        let mut webxr_main_thread = webxr::MainThreadRegistry::new(event_loop_waker)
            .expect("Failed to create WebXR device registry");
        if pref!(dom.webxr.enabled) {
            embedder.register_webxr(&mut webxr_main_thread);
        }

        let mut webvr_heartbeats = Vec::new();
        let webvr_services = if pref!(dom.webvr.enabled) {
            let mut services = VRServiceManager::new();
            services.register_defaults();
            embedder.register_vr_services(&mut services, &mut webvr_heartbeats);
            Some(services)
        } else {
            None
        };

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

        let (external_image_handlers, external_images) = WebrenderExternalImageHandlers::new();
        let mut external_image_handlers = Box::new(external_image_handlers);

        let webgl_threads = create_webgl_threads(
            &*window,
            &mut webrender,
            webrender_api_sender.clone(),
            webvr_compositor,
            &mut webxr_main_thread,
            &mut external_image_handlers,
            external_images.clone(),
        );

        let glplayer_threads = match window.get_gl_context() {
            GlContext::Unknown => None,
            _ => {
                let (glplayer_threads, image_handler) = GLPlayerThreads::new(external_images);
                external_image_handlers
                    .set_handler(image_handler, WebrenderImageHandlerType::Media);
                Some(glplayer_threads)
            },
        };

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
        let (constellation_chan, sw_senders) = create_constellation(
            opts.user_agent.clone(),
            opts.config_dir.clone(),
            embedder_proxy.clone(),
            compositor_proxy.clone(),
            time_profiler_chan.clone(),
            mem_profiler_chan.clone(),
            debugger_chan,
            devtools_chan,
            webrender_document,
            webrender_api_sender,
            webxr_main_thread.registry(),
            player_context,
            webgl_threads,
            webvr_chan,
            webvr_constellation_sender,
            glplayer_threads,
            event_loop_waker,
            window_size,
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

            WindowEvent::MediaSessionAction(a) => {
                let msg = ConstellationMsg::MediaSessionAction(a);
                if let Err(e) = self.constellation_chan.send(msg) {
                    warn!(
                        "Sending MediaSessionAction message to constellation failed ({:?}).",
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
    webrender_document: webrender_api::DocumentId,
    webrender_api_sender: webrender_api::RenderApiSender,
    webxr_registry: webxr_api::Registry,
    player_context: WindowGLContext,
    webgl_threads: Option<WebGLThreads>,
    webvr_chan: Option<IpcSender<WebVRMsg>>,
    webvr_constellation_sender: Option<Sender<Sender<ConstellationMsg>>>,
    glplayer_threads: Option<GLPlayerThreads>,
    event_loop_waker: Option<Box<dyn EventLoopWaker>>,
    initial_window_size: WindowSizeData,
) -> (Sender<ConstellationMsg>, SWManagerSenders) {
    // Global configuration options, parsed from the command line.
    let opts = opts::get();

    let bluetooth_thread: IpcSender<BluetoothRequest> =
        BluetoothThreadFactory::new(embedder_proxy.clone());

    let (public_resource_threads, private_resource_threads) = new_resource_threads(
        user_agent,
        devtools_chan.clone(),
        time_profiler_chan.clone(),
        mem_profiler_chan.clone(),
        embedder_proxy.clone(),
        config_dir,
        opts.certificate_path.clone(),
    );
    let font_cache_thread = FontCacheThread::new(
        public_resource_threads.sender(),
        webrender_api_sender.create_api(),
    );

    let resource_sender = public_resource_threads.sender();

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
        webxr_registry,
        glplayer_threads,
        player_context,
        event_loop_waker,
    };
    let (constellation_chan, from_swmanager_sender) = Constellation::<
        script_layout_interface::message::Msg,
        layout_thread::LayoutThread,
        script::script_thread::ScriptThread,
    >::start(
        initial_state,
        initial_window_size,
        opts.random_pipeline_closure_probability,
        opts.random_pipeline_closure_seed,
        opts.is_running_problem_test,
        opts.hard_fail,
        opts.enable_canvas_antialiasing,
    );

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
    let _js_engine_setup = script::init();
    script::init_service_workers(sw_senders);

    media_platform::init();

    unprivileged_content.start_all::<script_layout_interface::message::Msg,
                                     layout_thread::LayoutThread,
                                     script::script_thread::ScriptThread>(
                                         true,
                                         background_hang_monitor_register,
                                         None,
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

// Initializes the WebGL thread.
fn create_webgl_threads<W>(
    window: &W,
    webrender: &mut webrender::Renderer,
    webrender_api_sender: webrender_api::RenderApiSender,
    webvr_compositor: Option<Box<WebVRCompositorHandler>>,
    webxr_main_thread: &mut webxr::MainThreadRegistry,
    external_image_handlers: &mut WebrenderExternalImageHandlers,
    external_images: Arc<Mutex<WebrenderExternalImageRegistry>>,
) -> Option<WebGLThreads>
where
    W: WindowMethods + 'static + ?Sized,
{
    // Create a `surfman` device and context.
    window.make_gl_context_current();

    #[cfg(not(target_os = "windows"))]
    let (device, context) = unsafe {
        if opts::get().headless {
            let (device, context) = match SWDevice::from_current_context() {
                Ok(a) => a,
                Err(e) => {
                    warn!("Failed to create software graphics context: {:?}", e);
                    return None;
                },
            };
            (Device::Software(device), Context::Software(context))
        } else {
            let (device, context) = match HWDevice::from_current_context() {
                Ok(a) => a,
                Err(e) => {
                    warn!("Failed to create hardware graphics context: {:?}", e);
                    return None;
                },
            };
            (Device::Hardware(device), Context::Hardware(context))
        }
    };
    #[cfg(target_os = "windows")]
    let (device, context) = match unsafe { Device::from_current_context() } {
        Ok(a) => a,
        Err(e) => {
            warn!("Failed to create graphics context: {:?}", e);
            return None;
        },
    };

    let gl_type = match window.gl().get_type() {
        gleam::gl::GlType::Gl => sparkle::gl::GlType::Gl,
        gleam::gl::GlType::Gles => sparkle::gl::GlType::Gles,
    };

    let WebGLComm {
        webgl_threads,
        webxr_swap_chains,
        image_handler,
        output_handler,
    } = WebGLComm::new(
        device,
        context,
        window.gl(),
        webrender_api_sender,
        webvr_compositor.map(|compositor| compositor as Box<_>),
        external_images,
        gl_type,
    );

    // Set webrender external image handler for WebGL textures
    external_image_handlers.set_handler(image_handler, WebrenderImageHandlerType::WebGL);

    // Set webxr external image handler for WebGL textures
    webxr_main_thread.set_swap_chains(webxr_swap_chains);

    // Set DOM to texture handler, if enabled.
    if let Some(output_handler) = output_handler {
        webrender.set_output_image_handler(output_handler);
    }

    Some(webgl_threads)
}
