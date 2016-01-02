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

#[cfg(not(target_os = "windows"))]
extern crate gaol;
#[macro_use]
extern crate gleam;

pub extern crate canvas;
pub extern crate canvas_traits;
pub extern crate compositing;
pub extern crate devtools;
pub extern crate devtools_traits;
pub extern crate euclid;
pub extern crate gfx;
pub extern crate ipc_channel;
pub extern crate layers;
pub extern crate layout;
pub extern crate msg;
pub extern crate net;
pub extern crate net_traits;
pub extern crate profile;
pub extern crate profile_traits;
pub extern crate script;
pub extern crate script_traits;
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

use compositing::CompositorEventListener;
use compositing::CompositorMsg as ConstellationMsg;
use compositing::compositor_thread::InitialCompositorState;
use compositing::constellation::InitialConstellationState;
use compositing::pipeline::UnprivilegedPipelineContent;
#[cfg(not(target_os = "windows"))]
use compositing::sandboxing;
use compositing::windowing::WindowEvent;
use compositing::windowing::WindowMethods;
use compositing::{CompositorProxy, CompositorThread, Constellation};
#[cfg(not(target_os = "windows"))]
use gaol::sandbox::{ChildSandbox, ChildSandboxMethods};
use gfx::font_cache_thread::FontCacheThread;
use ipc_channel::ipc::{self, IpcSender};
use net::image_cache_thread::new_image_cache_thread;
use net::resource_thread::new_resource_thread;
use net::storage_thread::StorageThreadFactory;
use net_traits::storage_thread::StorageThread;
use profile::mem as profile_mem;
use profile::time as profile_time;
use profile_traits::mem;
use profile_traits::time;
use std::rc::Rc;
use std::sync::mpsc::Sender;
use util::opts;
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
pub struct Browser {
    compositor: Box<CompositorEventListener + 'static>,
}

impl Browser {
    pub fn new<Window>(window: Rc<Window>) -> Browser
                       where Window: WindowMethods + 'static {
        // Global configuration options, parsed from the command line.
        let opts = opts::get();

        script::init();

        // Get both endpoints of a special channel for communication between
        // the client window and the compositor. This channel is unique because
        // messages to client may need to pump a platform-specific event loop
        // to deliver the message.
        let (compositor_proxy, compositor_receiver) =
            window.create_compositor_channel();
        let supports_clipboard = window.supports_clipboard();
        let time_profiler_chan = profile_time::Profiler::create(opts.time_profiler_period);
        let mem_profiler_chan = profile_mem::Profiler::create(opts.mem_profiler_period);
        let devtools_chan = opts.devtools_port.map(|port| {
            devtools::start_server(port)
        });

        let (webrender, webrender_api_sender) = if opts::get().use_webrender {
            let mut resource_path = resources_dir_path();
            resource_path.push("shaders");

            // TODO(gw): Duplicates device_pixels_per_screen_px from compositor. Tidy up!
            let hidpi_factor = window.hidpi_factor().get();
            let device_pixel_ratio = match opts.device_pixels_per_px {
                Some(device_pixels_per_px) => device_pixels_per_px,
                None => match opts.output_file {
                    Some(_) => 1.0,
                    None => hidpi_factor,
                }
            };

            let (webrender, webrender_sender) =
                webrender::Renderer::new(webrender::RendererOptions {
                    device_pixel_ratio: device_pixel_ratio,
                    resource_path: resource_path,
                    enable_aa: opts.enable_text_antialiasing,
                    enable_msaa: opts.use_msaa,
                    enable_profiler: opts.webrender_stats,
                });
            (Some(webrender), Some(webrender_sender))
        } else {
            (None, None)
        };

        // Create the constellation, which maintains the engine
        // pipelines, including the script and layout threads, as well
        // as the navigation context.
        let constellation_chan = create_constellation(opts.clone(),
                                                      compositor_proxy.clone_compositor_proxy(),
                                                      time_profiler_chan.clone(),
                                                      mem_profiler_chan.clone(),
                                                      devtools_chan,
                                                      supports_clipboard,
                                                      webrender_api_sender.clone());

        if cfg!(feature = "webdriver") {
            if let Some(port) = opts.webdriver_port {
                webdriver(port, constellation_chan.clone());
            }
        }

        // The compositor coordinates with the client window to create the final
        // rendered page and display it somewhere.
        let compositor = CompositorThread::create(window, InitialCompositorState {
            sender: compositor_proxy,
            receiver: compositor_receiver,
            constellation_chan: constellation_chan,
            time_profiler_chan: time_profiler_chan,
            mem_profiler_chan: mem_profiler_chan,
            webrender: webrender,
            webrender_api_sender: webrender_api_sender,
        });

        Browser {
            compositor: compositor,
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
}

fn create_constellation(opts: opts::Opts,
                        compositor_proxy: Box<CompositorProxy + Send>,
                        time_profiler_chan: time::ProfilerChan,
                        mem_profiler_chan: mem::ProfilerChan,
                        devtools_chan: Option<Sender<devtools_traits::DevtoolsControlMsg>>,
                        supports_clipboard: bool,
                        webrender_api_sender: Option<webrender_traits::RenderApiSender>) -> Sender<ConstellationMsg> {
    let resource_thread = new_resource_thread(opts.user_agent.clone(), devtools_chan.clone());
    let image_cache_thread = new_image_cache_thread(resource_thread.clone(),
                                                    webrender_api_sender.as_ref().map(|wr| wr.create_api()));
    let font_cache_thread = FontCacheThread::new(resource_thread.clone(),
                                                 webrender_api_sender.as_ref().map(|wr| wr.create_api()));
    let storage_thread: StorageThread = StorageThreadFactory::new();

    let initial_state = InitialConstellationState {
        compositor_proxy: compositor_proxy,
        devtools_chan: devtools_chan,
        image_cache_thread: image_cache_thread,
        font_cache_thread: font_cache_thread,
        resource_thread: resource_thread,
        storage_thread: storage_thread,
        time_profiler_chan: time_profiler_chan,
        mem_profiler_chan: mem_profiler_chan,
        supports_clipboard: supports_clipboard,
        webrender_api_sender: webrender_api_sender,
    };
    let constellation_chan =
        Constellation::<layout::layout_thread::LayoutThread,
                        script::script_thread::ScriptThread>::start(initial_state);

    // Send the URL command to the constellation.
    match opts.url {
        Some(url) => {
            constellation_chan.send(ConstellationMsg::InitLoadUrl(url)).unwrap();
        },
        None => ()
    };

    constellation_chan
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

    // Enter the sandbox if necessary.
    if opts::get().sandbox {
       create_sandbox();
    }

    script::init();

    unprivileged_content.start_all::<layout::layout_thread::LayoutThread,
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
    ChildSandbox::new(sandboxing::content_process_sandbox_profile()).activate().unwrap();
}

#[cfg(target_os = "windows")]
fn create_sandbox() {
    panic!("Sandboxing is not supported on Windows.");
}
