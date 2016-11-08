/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Servo, the mighty web browser engine from the future.
//!
//! This is a very simple library that wires all of Servo's components
//! together as type `Browser`, along with a generic client
//! implementing the `WindowMethods` trait, to create a working web
//! browser.
//!
//! The `Browser` type is responsible for configuring a
//! `Constellation`, which does the heavy lifting of coordinating all
//! of Servo's internal subsystems, including the `ScriptThread` and the
//! `LayoutThread`, as well maintains the navigation context.
//!
//! The `Browser` is fed events from a generic type that implements the
//! `WindowMethods` trait.

extern crate env_logger;
#[cfg(not(target_os = "windows"))]
extern crate gaol;
#[macro_use]
extern crate gleam;
extern crate log;

pub extern crate bluetooth;
pub extern crate bluetooth_traits;
pub extern crate canvas;
pub extern crate canvas_traits;
pub extern crate compositing;
pub extern crate constellation;
pub extern crate debugger;
pub extern crate devtools;
pub extern crate devtools_traits;
pub extern crate euclid;
pub extern crate gfx;
pub extern crate ipc_channel;
pub extern crate layout_thread;
pub extern crate msg;
pub extern crate net;
pub extern crate net_traits;
pub extern crate profile;
pub extern crate profile_traits;
pub extern crate script;
pub extern crate script_traits;
pub extern crate script_layout_interface;
pub extern crate style;
pub extern crate url;
pub extern crate util;

#[cfg(feature = "webdriver")]
extern crate webdriver_server;

extern crate webrender;
extern crate webrender_traits;

#[cfg(feature = "webdriver")]
fn webdriver(port: u16, constellation: Sender<ConstellationMsg>) {
    webdriver_server::start_server(port, constellation);
}

#[cfg(not(feature = "webdriver"))]
fn webdriver(_port: u16, _constellation: Sender<ConstellationMsg>) { }

use bluetooth::BluetoothThreadFactory;
use bluetooth_traits::BluetoothMethodMsg;
use compositing::{CompositorProxy, IOCompositor};
use compositing::compositor_thread::InitialCompositorState;
use compositing::windowing::WindowEvent;
use compositing::windowing::WindowMethods;
use constellation::{Constellation, InitialConstellationState, UnprivilegedPipelineContent};
use constellation::{FromCompositorLogger, FromScriptLogger};
#[cfg(not(target_os = "windows"))]
use constellation::content_process_sandbox_profile;
use env_logger::Logger as EnvLogger;
#[cfg(not(target_os = "windows"))]
use gaol::sandbox::{ChildSandbox, ChildSandboxMethods};
use gfx::font_cache_thread::FontCacheThread;
use ipc_channel::ipc::{self, IpcSender};
use log::{Log, LogMetadata, LogRecord};
use net::image_cache_thread::new_image_cache_thread;
use net::resource_thread::new_resource_threads;
use net_traits::IpcSend;
use profile::mem as profile_mem;
use profile::time as profile_time;
use profile_traits::mem;
use profile_traits::time;
use script_traits::{ConstellationMsg, SWManagerSenders, ScriptMsg};
use std::cmp::max;
use std::rc::Rc;
use std::sync::mpsc::Sender;
use util::opts;
use util::prefs::PREFS;
use util::resource_files::resources_dir_path;

pub use gleam::gl;

/// The in-process interface to Servo.
///
/// It does everything necessary to render the web, primarily
/// orchestrating the interaction between JavaScript, CSS layout,
/// rendering, and the client window.
///
/// Clients create a `Browser` for a given reference-counted type
/// implementing `WindowMethods`, which is the bridge to whatever
/// application Servo is embedded in. Clients then create an event
/// loop to pump messages between the embedding application and
/// various browser components.
pub struct Browser<Window: WindowMethods + 'static> {
    compositor: IOCompositor<Window>,
    constellation_chan: Sender<ConstellationMsg>,
}

impl<Window> Browser<Window> where Window: WindowMethods + 'static {
    pub fn new(window: Rc<Window>) -> Browser<Window> {
        // Global configuration options, parsed from the command line.
        let opts = opts::get();


        // Get both endpoints of a special channel for communication between
        // the client window and the compositor. This channel is unique because
        // messages to client may need to pump a platform-specific event loop
        // to deliver the message.
        let (compositor_proxy, compositor_receiver) =
            window.create_compositor_channel();
        let supports_clipboard = window.supports_clipboard();
        let time_profiler_chan = profile_time::Profiler::create(&opts.time_profiling,
                                                                opts.time_profiler_trace_path.clone());
        let mem_profiler_chan = profile_mem::Profiler::create(opts.mem_profiler_period);
        let debugger_chan = opts.debugger_port.map(|port| {
            debugger::start_server(port)
        });
        let devtools_chan = opts.devtools_port.map(|port| {
            devtools::start_server(port)
        });

        let mut resource_path = resources_dir_path().unwrap();
        resource_path.push("shaders");

        let (webrender, webrender_api_sender) = {
            // TODO(gw): Duplicates device_pixels_per_screen_px from compositor. Tidy up!
            let scale_factor = window.scale_factor().get();
            let device_pixel_ratio = match opts.device_pixels_per_px {
                Some(device_pixels_per_px) => device_pixels_per_px,
                None => match opts.output_file {
                    Some(_) => 1.0,
                    None => scale_factor,
                }
            };

            let renderer_kind = if opts::get().should_use_osmesa() {
                webrender_traits::RendererKind::OSMesa
            } else {
                webrender_traits::RendererKind::Native
            };

            webrender::Renderer::new(webrender::RendererOptions {
                device_pixel_ratio: device_pixel_ratio,
                resource_path: resource_path,
                enable_aa: opts.enable_text_antialiasing,
                enable_msaa: opts.use_msaa,
                enable_profiler: opts.webrender_stats,
                debug: opts.webrender_debug,
                enable_recording: false,
                precache_shaders: opts.precache_shaders,
                enable_scrollbars: opts.output_file.is_none(),
                renderer_kind: renderer_kind,
                enable_subpixel_aa: opts.enable_subpixel_text_antialiasing,
            })
        };

        // Create the constellation, which maintains the engine
        // pipelines, including the script and layout threads, as well
        // as the navigation context.
        let (constellation_chan, sw_senders) = create_constellation(opts.clone(),
                                                                    compositor_proxy.clone_compositor_proxy(),
                                                                    time_profiler_chan.clone(),
                                                                    mem_profiler_chan.clone(),
                                                                    debugger_chan,
                                                                    devtools_chan,
                                                                    supports_clipboard,
                                                                    webrender_api_sender.clone());

        // Send the constellation's swmanager sender to service worker manager thread
        script::init(sw_senders);

        if cfg!(feature = "webdriver") {
            if let Some(port) = opts.webdriver_port {
                webdriver(port, constellation_chan.clone());
            }
        }

        // The compositor coordinates with the client window to create the final
        // rendered page and display it somewhere.
        let compositor = IOCompositor::create(window, InitialCompositorState {
            sender: compositor_proxy,
            receiver: compositor_receiver,
            constellation_chan: constellation_chan.clone(),
            time_profiler_chan: time_profiler_chan,
            mem_profiler_chan: mem_profiler_chan,
            webrender: webrender,
            webrender_api_sender: webrender_api_sender,
        });

        Browser {
            compositor: compositor,
            constellation_chan: constellation_chan,
        }
    }

    pub fn handle_events(&mut self, events: Vec<WindowEvent>) -> bool {
        self.compositor.handle_events(events)
    }

    pub fn repaint_synchronously(&mut self) {
        self.compositor.repaint_synchronously()
    }

    pub fn pinch_zoom_level(&self) -> f32 {
        self.compositor.pinch_zoom_level()
    }

    pub fn request_title_for_main_frame(&self) {
        self.compositor.title_for_main_frame()
    }

    pub fn setup_logging(&self) {
        let constellation_chan = self.constellation_chan.clone();
        log::set_logger(|max_log_level| {
            let env_logger = EnvLogger::new();
            let con_logger = FromCompositorLogger::new(constellation_chan);
            let filter = max(env_logger.filter(), con_logger.filter());
            let logger = BothLogger(env_logger, con_logger);
            max_log_level.set(filter);
            Box::new(logger)
        }).expect("Failed to set logger.")
    }
}

fn create_constellation(opts: opts::Opts,
                        compositor_proxy: Box<CompositorProxy + Send>,
                        time_profiler_chan: time::ProfilerChan,
                        mem_profiler_chan: mem::ProfilerChan,
                        debugger_chan: Option<debugger::Sender>,
                        devtools_chan: Option<Sender<devtools_traits::DevtoolsControlMsg>>,
                        supports_clipboard: bool,
                        webrender_api_sender: webrender_traits::RenderApiSender)
                        -> (Sender<ConstellationMsg>, SWManagerSenders) {
    let bluetooth_thread: IpcSender<BluetoothMethodMsg> = BluetoothThreadFactory::new();

    let (public_resource_threads, private_resource_threads) =
        new_resource_threads(opts.user_agent,
                             devtools_chan.clone(),
                             time_profiler_chan.clone(),
                             opts.config_dir.map(Into::into));
    let image_cache_thread = new_image_cache_thread(public_resource_threads.sender(),
                                                    webrender_api_sender.create_api());
    let font_cache_thread = FontCacheThread::new(public_resource_threads.sender(),
                                                 Some(webrender_api_sender.create_api()));

    let resource_sender = public_resource_threads.sender();

    let initial_state = InitialConstellationState {
        compositor_proxy: compositor_proxy,
        debugger_chan: debugger_chan,
        devtools_chan: devtools_chan,
        bluetooth_thread: bluetooth_thread,
        image_cache_thread: image_cache_thread,
        font_cache_thread: font_cache_thread,
        public_resource_threads: public_resource_threads,
        private_resource_threads: private_resource_threads,
        time_profiler_chan: time_profiler_chan,
        mem_profiler_chan: mem_profiler_chan,
        supports_clipboard: supports_clipboard,
        webrender_api_sender: webrender_api_sender,
    };
    let (constellation_chan, from_swmanager_sender) =
        Constellation::<script_layout_interface::message::Msg,
                        layout_thread::LayoutThread,
                        script::script_thread::ScriptThread>::start(initial_state);

    if let Some(url) = opts.url {
        constellation_chan.send(ConstellationMsg::InitLoadUrl(url)).unwrap();
    };

    // channels to communicate with Service Worker Manager
    let sw_senders = SWManagerSenders {
        swmanager_sender: from_swmanager_sender,
        resource_sender: resource_sender
    };

    (constellation_chan, sw_senders)
}

// A logger that logs to two downstream loggers.
// This should probably be in the log crate.
struct BothLogger<Log1, Log2>(Log1, Log2);

impl<Log1, Log2> Log for BothLogger<Log1, Log2> where Log1: Log, Log2: Log {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        self.0.enabled(metadata) || self.1.enabled(metadata)
    }

    fn log(&self, record: &LogRecord) {
        self.0.log(record);
        self.1.log(record);
    }
}

pub fn set_logger(constellation_chan: IpcSender<ScriptMsg>) {
    log::set_logger(|max_log_level| {
        let env_logger = EnvLogger::new();
        let con_logger = FromScriptLogger::new(constellation_chan);
        let filter = max(env_logger.filter(), con_logger.filter());
        let logger = BothLogger(env_logger, con_logger);
        max_log_level.set(filter);
        Box::new(logger)
    }).expect("Failed to set logger.")
}

/// Content process entry point.
pub fn run_content_process(token: String) {
    let (unprivileged_content_sender, unprivileged_content_receiver) =
        ipc::channel::<UnprivilegedPipelineContent>().unwrap();
    let connection_bootstrap: IpcSender<IpcSender<UnprivilegedPipelineContent>> =
        IpcSender::connect(token).unwrap();
    connection_bootstrap.send(unprivileged_content_sender).unwrap();

    let unprivileged_content = unprivileged_content_receiver.recv().unwrap();
    opts::set_defaults(unprivileged_content.opts());
    PREFS.extend(unprivileged_content.prefs());
    set_logger(unprivileged_content.constellation_chan());

    // Enter the sandbox if necessary.
    if opts::get().sandbox {
       create_sandbox();
    }

    // send the required channels to the service worker manager
    let sw_senders = unprivileged_content.swmanager_senders();
    script::init(sw_senders);

    unprivileged_content.start_all::<script_layout_interface::message::Msg,
                                     layout_thread::LayoutThread,
                                     script::script_thread::ScriptThread>(true);
}

// This is a workaround for https://github.com/rust-lang/rust/pull/30175 until
// https://github.com/lfairy/rust-errno/pull/5 lands, and should be removed once
// we update Servo with the rust-errno crate.
#[cfg(target_os = "android")]
#[no_mangle]
pub unsafe extern fn __errno_location() -> *mut i32 {
    extern { fn __errno() -> *mut i32; }
    __errno()
}

#[cfg(not(target_os = "windows"))]
fn create_sandbox() {
    ChildSandbox::new(content_process_sandbox_profile()).activate()
        .expect("Failed to activate sandbox!");
}

#[cfg(target_os = "windows")]
fn create_sandbox() {
    panic!("Sandboxing is not supported on Windows.");
}
