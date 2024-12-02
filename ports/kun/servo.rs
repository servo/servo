/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use arboard::Clipboard;
use base::id::{PipelineNamespace, PipelineNamespaceId};
use bluetooth::BluetoothThreadFactory;
use bluetooth_traits::BluetoothRequest;
use canvas::canvas_paint_thread::CanvasPaintThread;
use compositing_traits::{CompositorMsg, CompositorProxy, CompositorReceiver, ConstellationMsg};
use constellation::{Constellation, FromCompositorLogger, InitialConstellationState};
use crossbeam_channel::{unbounded, Sender};
use embedder_traits::{
    EmbedderMsg, EmbedderProxy, EmbedderReceiver, EventLoopWaker, PromptDefinition, PromptOrigin,
    PromptResult,
};
use euclid::Scale;
use fonts::SystemFontService;
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use log::{Log, Metadata, Record};
use media::{GlApi, GlContext, NativeDisplay, WindowGLContext};
use net::resource_thread;
use script::{self, JSEngineSetup};
use script_traits::WindowSizeData;
use servo_config::{opts, pref};
use webrender::{create_webrender_instance, ShaderPrecacheFlags, WebRenderOptions};
use webrender_api::*;
use webrender_traits::*;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoopProxy};
use winit::window::WindowId;
use {devtools, layout_thread_2020, profile, style, webgpu};

use crate::compositor::{IOCompositor, InitialCompositorState, ShutdownState};
use crate::config::Config;
use crate::window::Window;

/// Main entry point of Servo browser.
pub struct Servo {
    windows: HashMap<WindowId, (Window, DocumentId)>,
    compositor: Option<IOCompositor>,
    constellation_sender: Sender<ConstellationMsg>,
    embedder_receiver: EmbedderReceiver,
    /// For single-process Servo instances, this field controls the initialization
    /// and deinitialization of the JS Engine. Multiprocess Servo instances have their
    /// own instance that exists in the content process instead.
    _js_engine_setup: Option<JSEngineSetup>,
    /// FIXME: It's None on wayland in Flatpak. Find a way to support this.
    clipboard: Option<Clipboard>,
}

impl Servo {
    /// Create a Servo instance from Winit's window and event loop proxy.
    ///
    /// Following threads will be created while initializing Servo based on configurations:
    /// - Time Profiler: Enabled
    /// - Memory Profiler: Enabled
    /// - DevTools: `Opts::devtools_server_enabled`
    /// - Webrender: Enabled
    /// - WebGL: Disabled
    /// - WebXR: Disabled
    /// - Bluetooth: Enabled
    /// - Resource: Enabled
    /// - Storage: Enabled
    /// - Font Cache: Enabled
    /// - Canvas: Enabled
    /// - Constellation: Enabled
    /// - Image Cache: Enabled
    pub fn new(evl: &ActiveEventLoop, proxy: EventLoopProxy<EventLoopProxyMessage>) -> Self {
        let config = Config::new();

        // Initialize configurations and Servo window
        let protocols = config.create_protocols();
        let initial_url = config.url.clone();
        let with_panel = !config.no_panel;
        let window_settings = config.window_attributes.clone();
        let user_agent: Cow<'static, str> = config
            .user_agent
            .clone()
            .unwrap_or_else(|| default_user_agent_string().to_string())
            .into();

        config.init();
        // Reserving a namespace to create TopLevelBrowsingContextId.
        PipelineNamespace::install(PipelineNamespaceId(0));
        let (mut window, rendering_context) = Window::new(evl, window_settings);

        let event_loop_waker = Box::new(Waker(proxy));
        let opts = opts::get();

        // Set Stylo flags
        style::context::DEFAULT_DISABLE_STYLE_SHARING_CACHE
            .store(opts.debug.disable_share_style_cache, Ordering::Relaxed);
        style::context::DEFAULT_DUMP_STYLE_STATISTICS
            .store(opts.debug.dump_style_statistics, Ordering::Relaxed);
        style::traversal::IS_SERVO_NONINCREMENTAL_LAYOUT
            .store(opts.nonincremental_layout, Ordering::Relaxed);

        // Initialize servo media with dummy backend
        // This will create a thread to initialize a global static of servo media.
        // The thread will be closed once the static is initialized.
        // TODO: This is used by content process. Spawn it there once if we have multiprocess mode.
        servo_media::ServoMedia::init::<servo_media_dummy::DummyBackend>();

        // Get GL bindings
        let webrender_gl = rendering_context.gl.clone();

        // Create profiler threads
        let time_profiler_sender = profile::time::Profiler::create(
            &opts.time_profiling,
            opts.time_profiler_trace_path.clone(),
        );
        let mem_profiler_sender = profile::mem::Profiler::create(opts.mem_profiler_period);

        // Create compositor and embedder channels
        let (compositor_sender, compositor_receiver) = {
            let (sender, receiver) = unbounded();
            let (compositor_ipc_sender, compositor_ipc_receiver) =
                ipc::channel().expect("ipc channel failure");
            let cross_process_compositor_api = CrossProcessCompositorApi(compositor_ipc_sender);
            let compositor_proxy = CompositorProxy {
                sender,
                cross_process_compositor_api,
                event_loop_waker: event_loop_waker.clone(),
            };

            let compositor_proxy_clone = compositor_proxy.clone();
            ROUTER.add_typed_route(
                compositor_ipc_receiver,
                Box::new(move |message| {
                    let _ = compositor_proxy_clone.send(CompositorMsg::CrossProcess(
                        message.expect("Could not convert Compositor message"),
                    ));
                }),
            );
            (compositor_proxy, CompositorReceiver { receiver })
        };
        let (embedder_sender, embedder_receiver) = {
            let (sender, receiver) = unbounded();
            (
                EmbedderProxy {
                    sender,
                    event_loop_waker: event_loop_waker.clone(),
                },
                EmbedderReceiver { receiver },
            )
        };

        // Create dev tools thread
        let devtools_sender = if opts.devtools_server_enabled {
            Some(devtools::start_server(
                opts.devtools_port,
                embedder_sender.clone(),
            ))
        } else {
            None
        };

        // Create Webrender threads
        let (mut webrender, webrender_api_sender) = {
            let mut debug_flags = DebugFlags::empty();
            debug_flags.set(DebugFlags::PROFILER_DBG, opts.debug.webrender_stats);

            let render_notifier = Box::new(RenderNotifier::new(compositor_sender.clone()));
            let clear_color = ColorF::new(0., 0., 0., 0.);
            create_webrender_instance(
                webrender_gl.clone(),
                render_notifier,
                WebRenderOptions {
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
        let webrender_document =
            webrender_api.add_document_with_id(window.size(), u64::from(window.id()) as u32);

        // Initialize js engine if it's single process mode
        let js_engine_setup = if !opts.multiprocess {
            Some(script::init())
        } else {
            None
        };

        let (external_image_handlers, external_images) = WebrenderExternalImageHandlers::new();
        let mut external_image_handlers = Box::new(external_image_handlers);
        // Create the webgl thread
        // TODO: create webGL thread based on pref
        // let gl_type = match webrender_gl.get_type() {
        //     gl::GlType::Gl => sparkle::gl::GlType::Gl,
        //     gl::GlType::Gles => sparkle::gl::GlType::Gles,
        // };
        // let WebGLComm {
        //     webgl_threads,
        //     webxr_layer_grand_manager,
        //     image_handler,
        // } = WebGLComm::new(
        //     rendering_context.clone(),
        //     webrender_api.create_sender(),
        //     webrender_document,
        //     external_images.clone(),
        //     gl_type,
        // );
        // Set webrender external image handler for WebGL textures
        // external_image_handlers.set_handler(image_handler, WebrenderImageHandlerType::WebGL);

        // Set webrender external image handler for WebGPU textures
        let wgpu_image_handler = webgpu::WGPUExternalImages::default();
        let wgpu_image_map = wgpu_image_handler.images.clone();
        external_image_handlers.set_handler(
            Box::new(wgpu_image_handler),
            WebrenderImageHandlerType::WebGPU,
        );

        // TODO enable gl media player
        let glplayer_context = WindowGLContext {
            gl_context: GlContext::Unknown,
            gl_api: GlApi::None,
            native_display: NativeDisplay::Unknown,
            glplayer_chan: None,
        };

        webrender.set_external_image_handler(external_image_handlers);

        // Create bluetooth thread
        let bluetooth_thread: IpcSender<BluetoothRequest> =
            BluetoothThreadFactory::new(embedder_sender.clone());

        // Create resource thread pool
        let (public_resource_threads, private_resource_threads) =
            resource_thread::new_resource_threads(
                user_agent.clone(),
                devtools_sender.clone(),
                time_profiler_sender.clone(),
                mem_profiler_sender.clone(),
                embedder_sender.clone(),
                opts.config_dir.clone(),
                opts.certificate_path.clone(),
                opts.ignore_certificate_errors,
                Arc::new(protocols),
            );

        // Create font cache thread
        let system_font_service = Arc::new(
            SystemFontService::spawn(compositor_sender.cross_process_compositor_api.clone())
                .to_proxy(),
        );

        // Create canvas thread
        let (canvas_create_sender, canvas_ipc_sender) = CanvasPaintThread::start(
            compositor_sender.cross_process_compositor_api.clone(),
            system_font_service.clone(),
            public_resource_threads.clone(),
        );

        // Create layout factory
        let layout_factory = Arc::new(layout_thread_2020::LayoutFactoryImpl());
        let initial_state = InitialConstellationState {
            compositor_proxy: compositor_sender.clone(),
            embedder_proxy: embedder_sender,
            devtools_sender,
            bluetooth_thread,
            system_font_service,
            public_resource_threads,
            private_resource_threads,
            time_profiler_chan: time_profiler_sender.clone(),
            mem_profiler_chan: mem_profiler_sender.clone(),
            webrender_document,
            webrender_api_sender,
            webxr_registry: None,
            webgl_threads: None,
            glplayer_threads: None,
            player_context: glplayer_context,
            user_agent,
            webrender_external_images: external_images,
            wgpu_image_map,
        };

        // The division by 1 represents the page's default zoom of 100%,
        // and gives us the appropriate CSSPixel type for the viewport.
        let window_size = WindowSizeData {
            initial_viewport: window.size().to_f32() / Scale::new(1.0),
            device_pixel_ratio: Scale::new(window.scale_factor() as f32),
        };

        // Create constellation thread
        let constellation_sender = Constellation::<
            script::script_thread::ScriptThread,
            script::serviceworker_manager::ServiceWorkerManager,
        >::start(
            initial_state,
            layout_factory,
            window_size,
            opts.random_pipeline_closure_probability,
            opts.random_pipeline_closure_seed,
            opts.hard_fail,
            !opts.debug.disable_canvas_antialiasing,
            canvas_create_sender,
            canvas_ipc_sender,
        );

        // Create webdriver thread
        if let Some(port) = opts.webdriver_port {
            webdriver_server::start_server(port, constellation_sender.clone());
        }

        // The compositor coordinates with the client window to create the final
        // rendered page and display it somewhere.
        let compositor = IOCompositor::new(
            window.id(),
            window.size(),
            Scale::new(window.scale_factor() as f32),
            InitialCompositorState {
                sender: compositor_sender,
                receiver: compositor_receiver,
                constellation_chan: constellation_sender.clone(),
                time_profiler_chan: time_profiler_sender,
                mem_profiler_chan: mem_profiler_sender,
                webrender,
                webrender_document,
                webrender_api,
                rendering_context,
                webrender_gl,
            },
            opts.exit_after_load,
            opts.debug.convert_mouse_to_touch,
        );

        if with_panel {
            window.create_panel(&constellation_sender, initial_url);
        } else if let Some(initial_url) = initial_url {
            window.create_webview(&constellation_sender, initial_url.into());
        }

        let mut windows = HashMap::new();
        windows.insert(window.id(), (window, webrender_document));

        // Create Servo instance
        let servo = Servo {
            windows,
            compositor: Some(compositor),
            constellation_sender,
            embedder_receiver,
            _js_engine_setup: js_engine_setup,
            clipboard: Clipboard::new().ok(),
        };

        servo.setup_logging();
        servo
    }

    /// Handle Winit window events. The strategy to handle event are different between platforms
    /// because the order of events might be different.
    pub fn handle_window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        #[cfg(linux)]
        if let WindowEvent::Resized(_) = event {
            self.handle_winit_window_event(window_id, event);
        } else {
            self.handle_winit_window_event(window_id, event);
            self.handle_servo_messages(event_loop);
        }

        #[cfg(apple)]
        if let WindowEvent::RedrawRequested = event {
            let resizing = self.handle_winit_window_event(window_id, event);
            if !resizing {
                self.handle_servo_messages(event_loop);
            }
        } else {
            self.handle_winit_window_event(window_id, event);
            self.handle_servo_messages(event_loop);
        }

        #[cfg(windows)]
        {
            self.handle_winit_window_event(window_id, event);
            self.handle_servo_messages(event_loop);
        }
    }

    /// Handle Winit window events
    fn handle_winit_window_event(&mut self, window_id: WindowId, event: WindowEvent) -> bool {
        log::trace!("Servo is handling Winit event: {event:?}");
        if let Some(compositor) = &mut self.compositor {
            if let WindowEvent::CloseRequested = event {
                // self.windows.remove(&window_id);
                compositor.maybe_start_shutting_down();
            } else if let Some(window) = self.windows.get_mut(&window_id) {
                window
                    .0
                    .handle_winit_window_event(&self.constellation_sender, compositor, &event);
                return window.0.resizing;
            }
        }

        false
    }

    /// Handle message came from Servo.
    pub fn handle_servo_messages(&mut self, evl: &ActiveEventLoop) {
        let mut shutdown = false;
        if let Some(compositor) = &mut self.compositor {
            // Handle Compositor's messages first
            log::trace!("Servo is handling Compositor messages");
            if compositor.receive_messages(&mut self.windows) {
                // And then handle Embedder messages
                log::trace!(
                    "Servo is handling Embedder messages when shutdown state is set to {:?}",
                    compositor.shutdown_state
                );
                while let Some((webview_id, msg)) = self.embedder_receiver.try_recv_embedder_msg() {
                    match compositor.shutdown_state {
                        ShutdownState::NotShuttingDown => {
                            if let Some(id) = webview_id {
                                for (window, document) in self.windows.values_mut() {
                                    if window.has_webview(id) {
                                        if window.handle_servo_message(
                                            id,
                                            msg,
                                            &self.constellation_sender,
                                            self.clipboard.as_mut(),
                                            compositor,
                                        ) {
                                            let mut window =
                                                Window::new_with_compositor(evl, compositor);
                                            window.create_panel(&self.constellation_sender, None);
                                            let webrender_document = document.clone();
                                            self.windows
                                                .insert(window.id(), (window, webrender_document));
                                        }
                                        break;
                                    }
                                }
                            } else {
                                // Handle message in Servo Window
                                log::trace!("Servo Window is handling Embedder message: {msg:?}");
                                match msg {
                                    EmbedderMsg::SetCursor(cursor) => {
                                        // TODO: This should move to compositor
                                        if let Some(window) =
                                            self.windows.get(&compositor.current_window)
                                        {
                                            window.0.set_cursor_icon(cursor);
                                        }
                                    },
                                    EmbedderMsg::Shutdown | EmbedderMsg::ReadyToPresent(_) => {},
                                    EmbedderMsg::Prompt(definition, origin) => match origin {
                                        // TODO: actually prompt the user with a dialog
                                        PromptOrigin::Trusted => match definition {
                                            PromptDefinition::YesNo(question, ipc_sender) => {
                                                if question ==
                                                    "Accept incoming devtools connection?"
                                                {
                                                    if let Err(err) =
                                                        ipc_sender.send(PromptResult::Primary)
                                                    {
                                                        log::error!(
                                                            "Failed to send prompt result back: {err}"
                                                        );
                                                    }
                                                }
                                            },
                                            _ => {},
                                        },
                                        _ => {},
                                    },
                                    e => {
                                        log::trace!("Servo Window isn't supporting handling this message yet: {e:?}")
                                    },
                                }
                            }
                        },
                        ShutdownState::FinishedShuttingDown => {
                            log::error!("Servo shouldn't be handling messages after compositor has shut down");
                        },
                        ShutdownState::ShuttingDown => {},
                    }
                }
            }

            if compositor.shutdown_state != ShutdownState::FinishedShuttingDown {
                // Update compositor
                compositor.perform_updates(&mut self.windows);
            } else {
                shutdown = true;
            }
        }

        // Check if Servo need to start shutting down.
        if self.windows.is_empty() {
            self.compositor
                .as_mut()
                .map(IOCompositor::maybe_start_shutting_down);
        }

        // Check compositor status and set control flow.
        if shutdown {
            // If Compositor has shut down, deinit and remove it.
            if let Some(compositor) = self.compositor.take() {
                IOCompositor::deinit(compositor)
            }
            evl.exit();
        } else if self.is_animating() {
            evl.set_control_flow(ControlFlow::Poll);
        } else {
            evl.set_control_flow(ControlFlow::Wait);
        }
    }

    /// Request Servo to redraw. It will queue a redraw event on current focused window.
    pub fn request_redraw(&mut self, evl: &ActiveEventLoop) {
        if let Some(compositor) = &mut self.compositor {
            if let Some(window) = self.windows.get(&compositor.current_window) {
                // evl.set_control_flow(ControlFlow::Poll);
                window.0.request_redraw();
            } else {
                self.handle_servo_messages(evl);
            }
        }
    }

    /// Return true if one of the Servo windows is animating.
    pub fn is_animating(&self) -> bool {
        self.compositor
            .as_ref()
            .map(|c| c.is_animating)
            .unwrap_or(false)
    }

    fn setup_logging(&self) {
        let constellation_chan = self.constellation_sender.clone();
        let env = env_logger::Env::default();
        let env_logger = env_logger::Builder::from_env(env).build();
        let con_logger = FromCompositorLogger::new(constellation_chan);

        let filter = std::cmp::max(env_logger.filter(), con_logger.filter());
        let logger = BothLogger(env_logger, con_logger);

        log::set_boxed_logger(Box::new(logger)).expect("Failed to set logger.");
        log::set_max_level(filter);
    }
}

/// Message send to the event loop
#[derive(Debug)]
pub enum EventLoopProxyMessage {
    /// Wake
    Wake,
}

#[derive(Clone, Debug)]
struct Waker(pub EventLoopProxy<EventLoopProxyMessage>);

impl EventLoopWaker for Waker {
    fn clone_box(&self) -> Box<dyn EventLoopWaker> {
        Box::new(self.clone())
    }

    fn wake(&self) {
        if let Err(e) = self.0.send_event(EventLoopProxyMessage::Wake) {
            log::error!("Servo failed to send wake up event to Servo: {e}");
        }
    }
}

fn default_user_agent_string() -> &'static str {
    #[cfg(macos)]
    const UA_STRING: &str =
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:109.0) Servo/1.0 Firefox/111.0";
    #[cfg(ios)]
    const UA_STRING: &str =
        "Mozilla/5.0 (iPhone; CPU iPhone OS 16_4 like Mac OS X; rv:109.0) Servo/1.0 Firefox/111.0";
    #[cfg(android)]
    const UA_STRING: &str = "Mozilla/5.0 (Android; Mobile; rv:109.0) Servo/1.0 Firefox/111.0";
    #[cfg(linux)]
    const UA_STRING: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Servo/1.0 Firefox/111.0";
    #[cfg(windows)]
    const UA_STRING: &str =
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Servo/1.0 Firefox/111.0";

    UA_STRING
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

impl webrender::api::RenderNotifier for RenderNotifier {
    fn clone(&self) -> Box<dyn webrender::api::RenderNotifier> {
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

pub(crate) fn send_to_constellation(sender: &Sender<ConstellationMsg>, msg: ConstellationMsg) {
    let variant_name = msg.variant_name();
    if let Err(e) = sender.send(msg) {
        log::warn!("Sending {variant_name} to constellation failed: {e:?}");
    }
}
