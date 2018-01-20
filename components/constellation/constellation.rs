/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The `Constellation`, Servo's Grand Central Station
//!
//! The constellation tracks all information kept globally by the
//! browser engine, which includes:
//!
//! * The set of all `EventLoop` objects. Each event loop is
//!   the constellation's view of a script thread. The constellation
//!   interacts with a script thread by message-passing.
//!
//! * The set of all `Pipeline` objects.  Each pipeline gives the
//!   constellation's view of a `Window`, with its script thread and
//!   layout threads.  Pipelines may share script threads, but not
//!   layout threads.
//!
//! * The set of all `BrowsingContext` objects. Each browsing context
//!   gives the constellation's view of a `WindowProxy`.
//!   Each browsing context stores an independent
//!   session history, created by navigation. The session
//!   history can be traversed, for example by the back and forwards UI,
//!   so each session history maintains a list of past and future pipelines,
//!   as well as the current active pipeline.
//!
//! There are two kinds of browsing context: top-level ones (for
//! example tabs in a browser UI), and nested ones (typically caused
//! by `iframe` elements). Browsing contexts have a hierarchy
//! (typically caused by `iframe`s containing `iframe`s), giving rise
//! to a forest whose roots are top-level browsing context.  The logical
//! relationship between these types is:
//!
//! ```
//! +------------+                      +------------+                 +---------+
//! |  Browsing  | ------parent?------> |  Pipeline  | --event_loop--> |  Event  |
//! |  Context   | ------current------> |            |                 |  Loop   |
//! |            | ------prev*--------> |            | <---pipeline*-- |         |
//! |            | ------next*--------> |            |                 +---------+
//! |            |                      |            |
//! |            | <-top_level--------- |            |
//! |            | <-browsing_context-- |            |
//! +------------+                      +------------+
//! ```
//
//! Complicating matters, there are also mozbrowser iframes, which are top-level
//! iframes with a parent.
//!
//! The constellation also maintains channels to threads, including:
//!
//! * The script and layout threads.
//! * The graphics compositor.
//! * The font cache, image cache, and resource manager, which load
//!   and cache shared fonts, images, or other resources.
//! * The service worker manager.
//! * The devtools, debugger and webdriver servers.
//!
//! The constellation passes messages between the threads, and updates its state
//! to track the evolving state of the browsing context tree.
//!
//! The constellation acts as a logger, tracking any `warn!` messages from threads,
//! and converting any `error!` or `panic!` into a crash report, which is filed
//! using an appropriate `mozbrowsererror` event.
//!
//! Since there is only one constellation, and its responsibilities include crash reporting,
//! it is very important that it does not panic.
//!
//! It's also important that the constellation not deadlock. In particular, we need
//! to be careful that we don't introduce any cycles in the can-block-on relation.
//! Blocking is typically introduced by `receiver.recv()`, which blocks waiting for the
//! sender to send some data. Servo tries to achieve deadlock-freedom by using the following
//! can-block-on relation:
//!
//! * Layout can block on canvas
//! * Layout can block on font cache
//! * Layout can block on image cache
//! * Constellation can block on compositor
//! * Constellation can block on embedder
//! * Constellation can block on layout
//! * Script can block on anything (other than script)
//! * Blocking is transitive (if T1 can block on T2 and T2 can block on T3 then T1 can block on T3)
//! * Nothing can block on itself!
//!
//! There is a complexity intoduced by IPC channels, since they do not support
//! non-blocking send. This means that as well as `receiver.recv()` blocking,
//! `sender.send(data)` can also block when the IPC buffer is full. For this reason it is
//! very important that all IPC receivers where we depend on non-blocking send
//! use a router to route IPC messages to an mpsc channel. The reason why that solves
//! the problem is that under the hood, the router uses a dedicated thread to forward
//! messages, and:
//!
//! * Anything (other than a routing thread) can block on a routing thread
//!
//! See https://github.com/servo/servo/issues/14704

use backtrace::Backtrace;
use bluetooth_traits::BluetoothRequest;
use browsingcontext::{BrowsingContext, SessionHistoryChange, SessionHistoryEntry};
use browsingcontext::{FullyActiveBrowsingContextsIterator, AllBrowsingContextsIterator};
use canvas::canvas_paint_thread::CanvasPaintThread;
use canvas::webgl_thread::WebGLThreads;
use canvas_traits::canvas::CanvasMsg;
use clipboard::{ClipboardContext, ClipboardProvider};
use compositing::SendableFrameTree;
use compositing::compositor_thread::{CompositorProxy, EmbedderMsg, EmbedderProxy};
use compositing::compositor_thread::Msg as ToCompositorMsg;
use debugger;
use devtools_traits::{ChromeToDevtoolsControlMsg, DevtoolsControlMsg};
use euclid::{Size2D, TypedSize2D, TypedScale};
use event_loop::EventLoop;
use gfx::font_cache_thread::FontCacheThread;
use gfx_traits::Epoch;
use ipc_channel::{Error as IpcError};
use ipc_channel::ipc::{self, IpcSender, IpcReceiver};
use ipc_channel::router::ROUTER;
use itertools::Itertools;
use layout_traits::LayoutThreadFactory;
use log::{Log, LogLevel, LogLevelFilter, LogMetadata, LogRecord};
use msg::constellation_msg::{BrowsingContextId, TopLevelBrowsingContextId, FrameType, PipelineId};
use msg::constellation_msg::{Key, KeyModifiers, KeyState};
use msg::constellation_msg::{PipelineNamespace, PipelineNamespaceId, TraversalDirection};
use net_traits::{self, IpcSend, FetchResponseMsg, ResourceThreads};
use net_traits::pub_domains::reg_host;
use net_traits::request::RequestInit;
use net_traits::storage_thread::{StorageThreadMsg, StorageType};
use network_listener::NetworkListener;
use pipeline::{InitialPipelineState, Pipeline};
use profile_traits::mem;
use profile_traits::time;
use script_traits::{AnimationState, AnimationTickType, CompositorEvent};
use script_traits::{ConstellationControlMsg, ConstellationMsg as FromCompositorMsg, DiscardBrowsingContext};
use script_traits::{DocumentActivity, DocumentState, LayoutControlMsg, LoadData};
use script_traits::{IFrameLoadInfo, IFrameLoadInfoWithData, IFrameSandboxState, TimerSchedulerMsg};
use script_traits::{LayoutMsg as FromLayoutMsg, ScriptMsg as FromScriptMsg, ScriptThreadFactory};
use script_traits::{LogEntry, ScriptToConstellationChan, ServiceWorkerMsg, webdriver_msg};
use script_traits::{MozBrowserErrorType, MozBrowserEvent, WebDriverCommandMsg, WindowSizeData};
use script_traits::{SWManagerMsg, ScopeThings, UpdatePipelineIdReason, WindowSizeType};
use serde::{Deserialize, Serialize};
use servo_config::opts;
use servo_config::prefs::PREFS;
use servo_rand::{Rng, SeedableRng, ServoRng, random};
use servo_remutex::ReentrantMutex;
use servo_url::{Host, ImmutableOrigin, ServoUrl};
use std::borrow::ToOwned;
use std::cmp::Ordering;
use std::collections::{HashMap, VecDeque};
use std::iter::once;
use std::marker::PhantomData;
use std::process;
use std::rc::{Rc, Weak};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use style_traits::CSSPixel;
use style_traits::cursor::CursorKind;
use style_traits::viewport::ViewportConstraints;
use timer_scheduler::TimerScheduler;
use webrender_api;
use webvr_traits::{WebVREvent, WebVRMsg};

/// The `Constellation` itself. In the servo browser, there is one
/// constellation, which maintains all of the browser global data.
/// In embedded applications, there may be more than one constellation,
/// which are independent of each other.
///
/// The constellation may be in a different process from the pipelines,
/// and communicates using IPC.
///
/// It is parameterized over a `LayoutThreadFactory` and a
/// `ScriptThreadFactory` (which in practice are implemented by
/// `LayoutThread` in the `layout` crate, and `ScriptThread` in
/// the `script` crate). Script and layout communicate using a `Message`
/// type.
pub struct Constellation<Message, LTF, STF> {
    /// An IPC channel for script threads to send messages to the constellation.
    /// This is the script threads' view of `script_receiver`.
    script_sender: IpcSender<(PipelineId, FromScriptMsg)>,

    /// A channel for the constellation to receive messages from script threads.
    /// This is the constellation's view of `script_sender`.
    script_receiver: Receiver<Result<(PipelineId, FromScriptMsg), IpcError>>,

    /// An IPC channel for layout threads to send messages to the constellation.
    /// This is the layout threads' view of `layout_receiver`.
    layout_sender: IpcSender<FromLayoutMsg>,

    /// A channel for the constellation to receive messages from layout threads.
    /// This is the constellation's view of `layout_sender`.
    layout_receiver: Receiver<Result<FromLayoutMsg, IpcError>>,

    /// A channel for network listener to send messages to the constellation.
    network_listener_sender: Sender<(PipelineId, FetchResponseMsg)>,

    /// A channel for the constellation to receive messages from network listener.
    network_listener_receiver: Receiver<(PipelineId, FetchResponseMsg)>,

    /// A channel for the constellation to receive messages from the compositor thread.
    compositor_receiver: Receiver<FromCompositorMsg>,

    /// A channel through which messages can be sent to the embedder.
    embedder_proxy: EmbedderProxy,

    /// A channel (the implementation of which is port-specific) for the
    /// constellation to send messages to the compositor thread.
    compositor_proxy: CompositorProxy,

    /// The last frame tree sent to WebRender.
    active_browser_id: Option<TopLevelBrowsingContextId>,

    /// Channels for the constellation to send messages to the public
    /// resource-related threads.  There are two groups of resource
    /// threads: one for public browsing, and one for private
    /// browsing.
    public_resource_threads: ResourceThreads,

    /// Channels for the constellation to send messages to the private
    /// resource-related threads.  There are two groups of resource
    /// threads: one for public browsing, and one for private
    /// browsing.
    private_resource_threads: ResourceThreads,

    /// A channel for the constellation to send messages to the font
    /// cache thread.
    font_cache_thread: FontCacheThread,

    /// A channel for the constellation to send messages to the
    /// debugger thread.
    debugger_chan: Option<debugger::Sender>,

    /// A channel for the constellation to send messages to the
    /// devtools thread.
    devtools_chan: Option<Sender<DevtoolsControlMsg>>,

    /// An IPC channel for the constellation to send messages to the
    /// bluetooth thread.
    bluetooth_thread: IpcSender<BluetoothRequest>,

    /// An IPC channel for the constellation to send messages to the
    /// Service Worker Manager thread.
    swmanager_chan: Option<IpcSender<ServiceWorkerMsg>>,

    /// An IPC channel for Service Worker Manager threads to send
    /// messages to the constellation.  This is the SW Manager thread's
    /// view of `swmanager_receiver`.
    swmanager_sender: IpcSender<SWManagerMsg>,

    /// A channel for the constellation to receive messages from the
    /// Service Worker Manager thread. This is the constellation's view of
    /// `swmanager_sender`.
    swmanager_receiver: Receiver<Result<SWManagerMsg, IpcError>>,

    /// A channel for the constellation to send messages to the
    /// time profiler thread.
    time_profiler_chan: time::ProfilerChan,

    /// A channel for the constellation to send messages to the
    /// memory profiler thread.
    mem_profiler_chan: mem::ProfilerChan,

    /// A channel for the constellation to send messages to the
    /// timer thread.
    scheduler_chan: IpcSender<TimerSchedulerMsg>,

    /// A single WebRender document the constellation operates on.
    webrender_document: webrender_api::DocumentId,

    /// A channel for the constellation to send messages to the
    /// WebRender thread.
    webrender_api_sender: webrender_api::RenderApiSender,

    /// The set of all event loops in the browser. We generate a new
    /// event loop for each registered domain name (aka eTLD+1) in
    /// each top-level browsing context. We store the event loops in a map
    /// indexed by top-level browsing context id
    /// (as a `TopLevelBrowsingContextId`) and registered
    /// domain name (as a `Host`) to event loops. This double
    /// indirection ensures that separate tabs do not share event
    /// loops, even if the same domain is loaded in each.
    /// It is important that scripts with the same eTLD+1
    /// share an event loop, since they can use `document.domain`
    /// to become same-origin, at which point they can share DOM objects.
    event_loops: HashMap<TopLevelBrowsingContextId, HashMap<Host, Weak<EventLoop>>>,

    /// The set of all the pipelines in the browser.
    /// (See the `pipeline` module for more details.)
    pipelines: HashMap<PipelineId, Pipeline>,

    /// The set of all the browsing contexts in the browser.
    browsing_contexts: HashMap<BrowsingContextId, BrowsingContext>,

    /// When a navigation is performed, we do not immediately update
    /// the session history, instead we ask the event loop to begin loading
    /// the new document, and do not update the browsing context until the
    /// document is active. Between starting the load and it activating,
    /// we store a `SessionHistoryChange` object for the navigation in progress.
    pending_changes: Vec<SessionHistoryChange>,

    /// The currently focused pipeline for key events.
    focus_pipeline_id: Option<PipelineId>,

    /// Pipeline IDs are namespaced in order to avoid name collisions,
    /// and the namespaces are allocated by the constellation.
    next_pipeline_namespace_id: PipelineNamespaceId,

    /// The size of the top-level window.
    window_size: WindowSizeData,

    /// Means of accessing the clipboard
    clipboard_ctx: Option<ClipboardContext>,

    /// Bits of state used to interact with the webdriver implementation
    webdriver: WebDriverData,

    /// Document states for loaded pipelines (used only when writing screenshots).
    document_states: HashMap<PipelineId, DocumentState>,

    /// Are we shutting down?
    shutting_down: bool,

    /// Have we seen any warnings? Hopefully always empty!
    /// The buffer contains `(thread_name, reason)` entries.
    handled_warnings: VecDeque<(Option<String>, String)>,

    /// The random number generator and probability for closing pipelines.
    /// This is for testing the hardening of the constellation.
    random_pipeline_closure: Option<(ServoRng, f32)>,

    /// Phantom data that keeps the Rust type system happy.
    phantom: PhantomData<(Message, LTF, STF)>,

    /// Entry point to create and get channels to a WebGLThread.
    webgl_threads: WebGLThreads,

    /// A channel through which messages can be sent to the webvr thread.
    webvr_chan: Option<IpcSender<WebVRMsg>>,
}

/// State needed to construct a constellation.
pub struct InitialConstellationState {
    /// A channel through which messages can be sent to the embedder.
    pub embedder_proxy: EmbedderProxy,

    /// A channel through which messages can be sent to the compositor.
    pub compositor_proxy: CompositorProxy,

    /// A channel to the debugger, if applicable.
    pub debugger_chan: Option<debugger::Sender>,

    /// A channel to the developer tools, if applicable.
    pub devtools_chan: Option<Sender<DevtoolsControlMsg>>,

    /// A channel to the bluetooth thread.
    pub bluetooth_thread: IpcSender<BluetoothRequest>,

    /// A channel to the font cache thread.
    pub font_cache_thread: FontCacheThread,

    /// A channel to the resource thread.
    pub public_resource_threads: ResourceThreads,

    /// A channel to the resource thread.
    pub private_resource_threads: ResourceThreads,

    /// A channel to the time profiler thread.
    pub time_profiler_chan: time::ProfilerChan,

    /// A channel to the memory profiler thread.
    pub mem_profiler_chan: mem::ProfilerChan,

    /// Webrender document ID.
    pub webrender_document: webrender_api::DocumentId,

    /// Webrender API.
    pub webrender_api_sender: webrender_api::RenderApiSender,

    /// Entry point to create and get channels to a WebGLThread.
    pub webgl_threads: WebGLThreads,

    /// A channel to the webgl thread.
    pub webvr_chan: Option<IpcSender<WebVRMsg>>,

    /// Whether the constellation supports the clipboard.
    /// TODO: this field is not used, remove it?
    pub supports_clipboard: bool,
}

/// Data needed for webdriver
struct WebDriverData {
    load_channel: Option<(PipelineId, IpcSender<webdriver_msg::LoadStatus>)>,
    resize_channel: Option<IpcSender<WindowSizeData>>,
}

impl WebDriverData {
    fn new() -> WebDriverData {
        WebDriverData {
            load_channel: None,
            resize_channel: None,
        }
    }
}

/// When we are running reftests, we save an image to compare against a reference.
/// This enum gives the possible states of preparing such an image.
#[derive(Debug, PartialEq)]
enum ReadyToSave {
    NoTopLevelBrowsingContext,
    PendingChanges,
    WebFontNotLoaded,
    DocumentLoading,
    EpochMismatch,
    PipelineUnknown,
    Ready,
}

/// When we are exiting a pipeline, we can either force exiting or not.
/// A normal exit waits for the compositor to update its state before
/// exiting, and delegates layout exit to script. A forced exit does
/// not notify the compositor, and exits layout without involving script.
#[derive(Clone, Copy)]
enum ExitPipelineMode {
    Normal,
    Force,
}

/// The constellation uses logging to perform crash reporting.
/// The constellation receives all `warn!`, `error!` and `panic!` messages,
/// and generates a crash report when it receives a panic.

/// A logger directed at the constellation from content processes
#[derive(Clone)]
pub struct FromScriptLogger {
    /// A channel to the constellation
    pub script_to_constellation_chan: Arc<ReentrantMutex<ScriptToConstellationChan>>,
}

impl FromScriptLogger {
    /// Create a new constellation logger.
    pub fn new(script_to_constellation_chan: ScriptToConstellationChan) -> FromScriptLogger {
        FromScriptLogger {
            script_to_constellation_chan: Arc::new(ReentrantMutex::new(script_to_constellation_chan))
        }
    }

    /// The maximum log level the constellation logger is interested in.
    pub fn filter(&self) -> LogLevelFilter {
        LogLevelFilter::Warn
    }
}

impl Log for FromScriptLogger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= LogLevel::Warn
    }

    fn log(&self, record: &LogRecord) {
        if let Some(entry) = log_entry(record) {
            debug!("Sending log entry {:?}.", entry);
            let thread_name = thread::current().name().map(ToOwned::to_owned);
            let msg = FromScriptMsg::LogEntry(thread_name, entry);
            let chan = self.script_to_constellation_chan.lock().unwrap_or_else(|err| err.into_inner());
            let _ = chan.send(msg);
        }
    }
}

/// A logger directed at the constellation from the compositor
#[derive(Clone)]
pub struct FromCompositorLogger {
    /// A channel to the constellation
    pub constellation_chan: Arc<ReentrantMutex<Sender<FromCompositorMsg>>>,
}

impl FromCompositorLogger {
    /// Create a new constellation logger.
    pub fn new(constellation_chan: Sender<FromCompositorMsg>) -> FromCompositorLogger {
        FromCompositorLogger {
            constellation_chan: Arc::new(ReentrantMutex::new(constellation_chan))
        }
    }

    /// The maximum log level the constellation logger is interested in.
    pub fn filter(&self) -> LogLevelFilter {
        LogLevelFilter::Warn
    }
}

impl Log for FromCompositorLogger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= LogLevel::Warn
    }

    fn log(&self, record: &LogRecord) {
        if let Some(entry) = log_entry(record) {
            debug!("Sending log entry {:?}.", entry);
            let top_level_id = TopLevelBrowsingContextId::installed();
            let thread_name = thread::current().name().map(ToOwned::to_owned);
            let msg = FromCompositorMsg::LogEntry(top_level_id, thread_name, entry);
            let chan = self.constellation_chan.lock().unwrap_or_else(|err| err.into_inner());
            let _ = chan.send(msg);
        }
    }
}

/// Rust uses `LogRecord` for storing logging, but servo converts that to
/// a `LogEntry`. We do this so that we can record panics as well as log
/// messages, and because `LogRecord` does not implement serde (de)serialization,
/// so cannot be used over an IPC channel.
fn log_entry(record: &LogRecord) -> Option<LogEntry> {
    match record.level() {
        LogLevel::Error if thread::panicking() => Some(LogEntry::Panic(
            format!("{}", record.args()),
            format!("{:?}", Backtrace::new())
        )),
        LogLevel::Error => Some(LogEntry::Error(
            format!("{}", record.args())
        )),
        LogLevel::Warn => Some(LogEntry::Warn(
            format!("{}", record.args())
        )),
        _ => None,
    }
}

/// The number of warnings to include in each crash report.
const WARNINGS_BUFFER_SIZE: usize = 32;

/// Route an ipc receiver to an mpsc receiver, preserving any errors.
/// This is the same as `route_ipc_receiver_to_new_mpsc_receiver`,
/// but does not panic on deserializtion errors.
fn route_ipc_receiver_to_new_mpsc_receiver_preserving_errors<T>(ipc_receiver: IpcReceiver<T>)
    -> Receiver<Result<T, IpcError>>
    where T: for<'de> Deserialize<'de> + Serialize + Send + 'static
{
        let (mpsc_sender, mpsc_receiver) = channel();
        ROUTER.add_route(ipc_receiver.to_opaque(), Box::new(move |message| {
            drop(mpsc_sender.send(message.to::<T>()))
        }));
        mpsc_receiver
}

impl<Message, LTF, STF> Constellation<Message, LTF, STF>
    where LTF: LayoutThreadFactory<Message=Message>,
          STF: ScriptThreadFactory<Message=Message>
{
    /// Create a new constellation thread.
    pub fn start(state: InitialConstellationState) -> (Sender<FromCompositorMsg>, IpcSender<SWManagerMsg>) {
        let (compositor_sender, compositor_receiver) = channel();

        // service worker manager to communicate with constellation
        let (swmanager_sender, swmanager_receiver) = ipc::channel().expect("ipc channel failure");
        let sw_mgr_clone = swmanager_sender.clone();

        thread::Builder::new().name("Constellation".to_owned()).spawn(move || {
            let (ipc_script_sender, ipc_script_receiver) = ipc::channel().expect("ipc channel failure");
            let script_receiver = route_ipc_receiver_to_new_mpsc_receiver_preserving_errors(ipc_script_receiver);

            let (ipc_layout_sender, ipc_layout_receiver) = ipc::channel().expect("ipc channel failure");
            let layout_receiver = route_ipc_receiver_to_new_mpsc_receiver_preserving_errors(ipc_layout_receiver);

            let (network_listener_sender, network_listener_receiver) = channel();

            let swmanager_receiver = route_ipc_receiver_to_new_mpsc_receiver_preserving_errors(swmanager_receiver);

            PipelineNamespace::install(PipelineNamespaceId(0));

            let mut constellation: Constellation<Message, LTF, STF> = Constellation {
                script_sender: ipc_script_sender,
                layout_sender: ipc_layout_sender,
                script_receiver: script_receiver,
                compositor_receiver: compositor_receiver,
                layout_receiver: layout_receiver,
                network_listener_sender: network_listener_sender,
                network_listener_receiver: network_listener_receiver,
                embedder_proxy: state.embedder_proxy,
                compositor_proxy: state.compositor_proxy,
                active_browser_id: None,
                debugger_chan: state.debugger_chan,
                devtools_chan: state.devtools_chan,
                bluetooth_thread: state.bluetooth_thread,
                public_resource_threads: state.public_resource_threads,
                private_resource_threads: state.private_resource_threads,
                font_cache_thread: state.font_cache_thread,
                swmanager_chan: None,
                swmanager_receiver: swmanager_receiver,
                swmanager_sender: sw_mgr_clone,
                event_loops: HashMap::new(),
                pipelines: HashMap::new(),
                browsing_contexts: HashMap::new(),
                pending_changes: vec!(),
                // We initialize the namespace at 1, since we reserved namespace 0 for the constellation
                next_pipeline_namespace_id: PipelineNamespaceId(1),
                focus_pipeline_id: None,
                time_profiler_chan: state.time_profiler_chan,
                mem_profiler_chan: state.mem_profiler_chan,
                window_size: WindowSizeData {
                    initial_viewport: opts::get().initial_window_size.to_f32() *
                        TypedScale::new(1.0),
                    device_pixel_ratio:
                        TypedScale::new(opts::get().device_pixels_per_px.unwrap_or(1.0)),
                },
                phantom: PhantomData,
                clipboard_ctx: if state.supports_clipboard {
                    match ClipboardContext::new() {
                        Ok(c) => Some(c),
                        Err(e) => {
                            warn!("Error creating clipboard context ({})", e);
                            None
                        },
                    }
                } else {
                    None
                },
                webdriver: WebDriverData::new(),
                scheduler_chan: TimerScheduler::start(),
                document_states: HashMap::new(),
                webrender_document: state.webrender_document,
                webrender_api_sender: state.webrender_api_sender,
                shutting_down: false,
                handled_warnings: VecDeque::new(),
                random_pipeline_closure: opts::get().random_pipeline_closure_probability.map(|prob| {
                    let seed = opts::get().random_pipeline_closure_seed.unwrap_or_else(random);
                    let rng = ServoRng::from_seed(&[seed]);
                    warn!("Randomly closing pipelines.");
                    info!("Using seed {} for random pipeline closure.", seed);
                    (rng, prob)
                }),
                webgl_threads: state.webgl_threads,
                webvr_chan: state.webvr_chan,
            };

            constellation.run();
        }).expect("Thread spawning failed");

        (compositor_sender, swmanager_sender)
    }

    /// The main event loop for the constellation.
    fn run(&mut self) {
        while !self.shutting_down || !self.pipelines.is_empty() {
            // Randomly close a pipeline if --random-pipeline-closure-probability is set
            // This is for testing the hardening of the constellation.
            self.maybe_close_random_pipeline();
            self.handle_request();
        }
        self.handle_shutdown();
    }

    /// Generate a new pipeline id namespace.
    fn next_pipeline_namespace_id(&mut self) -> PipelineNamespaceId {
        let namespace_id = self.next_pipeline_namespace_id;
        let PipelineNamespaceId(ref mut i) = self.next_pipeline_namespace_id;
        *i += 1;
        namespace_id
    }

    /// Helper function for creating a pipeline
    fn new_pipeline(&mut self,
                    pipeline_id: PipelineId,
                    browsing_context_id: BrowsingContextId,
                    top_level_browsing_context_id: TopLevelBrowsingContextId,
                    parent_info: Option<(PipelineId, FrameType)>,
                    initial_window_size: Option<TypedSize2D<f32, CSSPixel>>,
                    // TODO: we have to provide ownership of the LoadData
                    // here, because it will be send on an ipc channel,
                    // and ipc channels take onership of their data.
                    // https://github.com/servo/ipc-channel/issues/138
                    load_data: LoadData,
                    sandbox: IFrameSandboxState,
                    is_private: bool) {
        if self.shutting_down { return; }

        debug!("Creating new pipeline {} in browsing context {}.", pipeline_id, browsing_context_id);

        let (event_loop, host) = match sandbox {
            IFrameSandboxState::IFrameSandboxed => (None, None),
            IFrameSandboxState::IFrameUnsandboxed => {
                // If this is an about:blank load, it must share the creator's event loop.
                // This must match the logic in the script thread when determining the proper origin.
                if load_data.url.as_str() != "about:blank" {
                    match reg_host(&load_data.url) {
                        None => (None, None),
                        Some(host) => {
                            let event_loop = self.event_loops.get(&top_level_browsing_context_id)
                                .and_then(|map| map.get(&host))
                                .and_then(|weak| weak.upgrade());
                            match event_loop {
                                None => (None, Some(host)),
                                Some(event_loop) => (Some(event_loop.clone()), None),
                            }
                        },
                    }
                } else if let Some(parent) = parent_info
                        .and_then(|(pipeline_id, _)| self.pipelines.get(&pipeline_id)) {
                    (Some(parent.event_loop.clone()), None)
                } else if let Some(creator) = load_data.creator_pipeline_id
                        .and_then(|pipeline_id| self.pipelines.get(&pipeline_id)) {
                    (Some(creator.event_loop.clone()), None)
                } else {
                    (None, None)
                }
            },
        };

        let resource_threads = if is_private {
            self.private_resource_threads.clone()
        } else {
            self.public_resource_threads.clone()
        };

        let parent_visibility = parent_info
            .and_then(|(parent_pipeline_id, _)| self.pipelines.get(&parent_pipeline_id))
            .map(|pipeline| pipeline.visible);

        let prev_visibility = self.browsing_contexts.get(&browsing_context_id)
            .and_then(|browsing_context| self.pipelines.get(&browsing_context.pipeline_id))
            .map(|pipeline| pipeline.visible)
            .or(parent_visibility);

        let result = Pipeline::spawn::<Message, LTF, STF>(InitialPipelineState {
            id: pipeline_id,
            browsing_context_id,
            top_level_browsing_context_id,
            parent_info,
            script_to_constellation_chan: ScriptToConstellationChan {
                sender: self.script_sender.clone(),
                pipeline_id: pipeline_id,
            },
            layout_to_constellation_chan: self.layout_sender.clone(),
            scheduler_chan: self.scheduler_chan.clone(),
            compositor_proxy: self.compositor_proxy.clone(),
            devtools_chan: self.devtools_chan.clone(),
            bluetooth_thread: self.bluetooth_thread.clone(),
            swmanager_thread: self.swmanager_sender.clone(),
            font_cache_thread: self.font_cache_thread.clone(),
            resource_threads,
            time_profiler_chan: self.time_profiler_chan.clone(),
            mem_profiler_chan: self.mem_profiler_chan.clone(),
            window_size: initial_window_size,
            event_loop,
            load_data,
            device_pixel_ratio: self.window_size.device_pixel_ratio,
            pipeline_namespace_id: self.next_pipeline_namespace_id(),
            prev_visibility,
            webrender_api_sender: self.webrender_api_sender.clone(),
            webrender_document: self.webrender_document,
            is_private,
            webgl_chan: self.webgl_threads.pipeline(),
            webvr_chan: self.webvr_chan.clone()
        });

        let pipeline = match result {
            Ok(result) => result,
            Err(e) => return self.handle_send_error(pipeline_id, e),
        };

        if let Some(host) = host {
            debug!("Adding new host entry {} for top-level browsing context {}.", host, top_level_browsing_context_id);
            self.event_loops.entry(top_level_browsing_context_id)
                .or_insert_with(HashMap::new)
                .insert(host, Rc::downgrade(&pipeline.event_loop));
        }

        assert!(!self.pipelines.contains_key(&pipeline_id));
        self.pipelines.insert(pipeline_id, pipeline);
    }

    /// Get an iterator for the fully active browsing contexts in a subtree.
    fn fully_active_descendant_browsing_contexts_iter(&self, browsing_context_id: BrowsingContextId)
                                                      -> FullyActiveBrowsingContextsIterator
    {
        FullyActiveBrowsingContextsIterator {
            stack: vec!(browsing_context_id),
            pipelines: &self.pipelines,
            browsing_contexts: &self.browsing_contexts,
        }
    }

    /// Get an iterator for the fully active browsing contexts in a tree.
    fn fully_active_browsing_contexts_iter(&self, top_level_browsing_context_id: TopLevelBrowsingContextId)
                                           -> FullyActiveBrowsingContextsIterator
    {
        self.fully_active_descendant_browsing_contexts_iter(BrowsingContextId::from(top_level_browsing_context_id))
    }

    /// Get an iterator for the browsing contexts in a subtree.
    fn all_descendant_browsing_contexts_iter(&self, browsing_context_id: BrowsingContextId)
                                             -> AllBrowsingContextsIterator
    {
        AllBrowsingContextsIterator {
            stack: vec!(browsing_context_id),
            pipelines: &self.pipelines,
            browsing_contexts: &self.browsing_contexts,
        }
    }

    /// Get an iterator for the browsing contexts in a tree.
    fn all_browsing_contexts_iter(&self, top_level_browsing_context_id: TopLevelBrowsingContextId)
                                  -> AllBrowsingContextsIterator
    {
        self.all_descendant_browsing_contexts_iter(BrowsingContextId::from(top_level_browsing_context_id))
    }

    #[cfg(feature = "unstable")]
    /// The joint session future is the merge of the session future of every
    /// browsing_context, sorted chronologically.
    fn joint_session_future<'a>(&'a self, top_level_browsing_context_id: TopLevelBrowsingContextId)
                                -> impl Iterator<Item = &'a SessionHistoryEntry> + 'a
    {
        self.all_browsing_contexts_iter(top_level_browsing_context_id)
            .map(|browsing_context| browsing_context.next.iter().rev())
            .kmerge_by(|a, b| a.instant.cmp(&b.instant) == Ordering::Less)
    }

    #[cfg(not(feature = "unstable"))]
    /// The joint session future is the merge of the session future of every
    /// browsing_context, sorted chronologically.
    fn joint_session_future<'a>(&'a self, top_level_browsing_context_id: TopLevelBrowsingContextId)
                                -> Box<Iterator<Item = &'a SessionHistoryEntry> + 'a>
    {
        Box::new(
            self.all_browsing_contexts_iter(top_level_browsing_context_id)
                .map(|browsing_context| browsing_context.next.iter().rev())
                .kmerge_by(|a, b| a.instant.cmp(&b.instant) == Ordering::Less)
        )
    }

    /// Is the joint session future empty?
    fn joint_session_future_is_empty(&self, top_level_browsing_context_id: TopLevelBrowsingContextId) -> bool {
        self.all_browsing_contexts_iter(top_level_browsing_context_id)
            .all(|browsing_context| browsing_context.next.is_empty())
    }

    #[cfg(feature = "unstable")]
    /// The joint session past is the merge of the session past of every
    /// browsing_context, sorted reverse chronologically.
    fn joint_session_past<'a>(&'a self, top_level_browsing_context_id: TopLevelBrowsingContextId)
                              -> impl Iterator<Item = &'a SessionHistoryEntry> + 'a
    {
        self.all_browsing_contexts_iter(top_level_browsing_context_id)
            .map(|browsing_context| browsing_context.prev.iter().rev()
                 .scan(browsing_context.instant, |prev_instant, entry| {
                     let instant = *prev_instant;
                     *prev_instant = entry.instant;
                     Some((instant, entry))
                 }))
            .kmerge_by(|a, b| a.0.cmp(&b.0) == Ordering::Greater)
            .map(|(_, entry)| entry)
    }

    #[cfg(not(feature = "unstable"))]
    /// The joint session past is the merge of the session past of every
    /// browsing_context, sorted reverse chronologically.
    fn joint_session_past<'a>(&'a self, top_level_browsing_context_id: TopLevelBrowsingContextId)
                              -> Box<Iterator<Item = &'a SessionHistoryEntry> + 'a>
    {
        Box::new(
            self.all_browsing_contexts_iter(top_level_browsing_context_id)
                .map(|browsing_context| browsing_context.prev.iter().rev()
                     .scan(browsing_context.instant, |prev_instant, entry| {
                         let instant = *prev_instant;
                         *prev_instant = entry.instant;
                         Some((instant, entry))
                     }))
                .kmerge_by(|a, b| a.0.cmp(&b.0) == Ordering::Greater)
                .map(|(_, entry)| entry)
        )
    }

    /// Is the joint session past empty?
    fn joint_session_past_is_empty(&self, top_level_browsing_context_id: TopLevelBrowsingContextId) -> bool {
        self.all_browsing_contexts_iter(top_level_browsing_context_id)
            .all(|browsing_context| browsing_context.prev.is_empty())
    }

    /// Create a new browsing context and update the internal bookkeeping.
    fn new_browsing_context(&mut self,
                 browsing_context_id: BrowsingContextId,
                 top_level_id: TopLevelBrowsingContextId,
                 pipeline_id: PipelineId,
                 load_data: LoadData) {
        debug!("Creating new browsing context {}", browsing_context_id);
        let browsing_context = BrowsingContext::new(browsing_context_id, top_level_id, pipeline_id, load_data);
        self.browsing_contexts.insert(browsing_context_id, browsing_context);

        // If a child browsing_context, add it to the parent pipeline.
        let parent_info = self.pipelines.get(&pipeline_id)
            .and_then(|pipeline| pipeline.parent_info);
        if let Some((parent_id, _)) = parent_info {
            if let Some(parent) = self.pipelines.get_mut(&parent_id) {
                parent.add_child(browsing_context_id);
            }
        }
    }

    fn add_pending_change(&mut self, change: SessionHistoryChange) {
        self.handle_load_start_msg(change.top_level_browsing_context_id, change.new_pipeline_id);
        self.pending_changes.push(change);
    }

    /// Handles loading pages, navigation, and granting access to the compositor
    #[allow(unsafe_code)]
    fn handle_request(&mut self) {
        enum Request {
            Script((PipelineId, FromScriptMsg)),
            Compositor(FromCompositorMsg),
            Layout(FromLayoutMsg),
            NetworkListener((PipelineId, FetchResponseMsg)),
            FromSWManager(SWManagerMsg),
        }

        // Get one incoming request.
        // This is one of the few places where the compositor is
        // allowed to panic. If one of the receiver.recv() calls
        // fails, it is because the matching sender has been
        // reclaimed, but this can't happen in normal execution
        // because the constellation keeps a pointer to the sender,
        // so it should never be reclaimed. A possible scenario in
        // which receiver.recv() fails is if some unsafe code
        // produces undefined behaviour, resulting in the destructor
        // being called. If this happens, there's not much we can do
        // other than panic.
        let request = {
            let receiver_from_script = &self.script_receiver;
            let receiver_from_compositor = &self.compositor_receiver;
            let receiver_from_layout = &self.layout_receiver;
            let receiver_from_network_listener = &self.network_listener_receiver;
            let receiver_from_swmanager = &self.swmanager_receiver;
            select! {
                msg = receiver_from_script.recv() =>
                    msg.expect("Unexpected script channel panic in constellation").map(Request::Script),
                msg = receiver_from_compositor.recv() =>
                    Ok(Request::Compositor(msg.expect("Unexpected compositor channel panic in constellation"))),
                msg = receiver_from_layout.recv() =>
                    msg.expect("Unexpected layout channel panic in constellation").map(Request::Layout),
                msg = receiver_from_network_listener.recv() =>
                    Ok(Request::NetworkListener(
                        msg.expect("Unexpected network listener channel panic in constellation")
                    )),
                msg = receiver_from_swmanager.recv() =>
                    msg.expect("Unexpected panic channel panic in constellation").map(Request::FromSWManager)
            }
        };

        let request = match request {
            Ok(request) => request,
            Err(err) => return error!("Deserialization failed ({}).", err),
        };

        match request {
            Request::Compositor(message) => {
                self.handle_request_from_compositor(message)
            },
            Request::Script(message) => {
                self.handle_request_from_script(message);
            },
            Request::Layout(message) => {
                self.handle_request_from_layout(message);
            },
            Request::NetworkListener(message) => {
                self.handle_request_from_network_listener(message);
            },
            Request::FromSWManager(message) => {
                self.handle_request_from_swmanager(message);
            }
        }
    }

    fn handle_request_from_network_listener(&mut self, message: (PipelineId, FetchResponseMsg)) {
        let (id, message_) = message;
        let result = match self.pipelines.get(&id) {
            Some(pipeline) => {
                let msg = ConstellationControlMsg::NavigationResponse(id, message_);
                pipeline.event_loop.send(msg)
            },
            None => {
                return warn!("Pipeline {:?} got fetch data after closure!", id);
            },
        };
        if let Err(e) = result {
            self.handle_send_error(id, e);
        }
    }

    fn handle_request_from_swmanager(&mut self, message: SWManagerMsg) {
        match message {
            SWManagerMsg::OwnSender(sw_sender) => {
                // store service worker manager for communicating with it.
                self.swmanager_chan = Some(sw_sender);
            }
        }
    }

    fn handle_request_from_compositor(&mut self, message: FromCompositorMsg) {
        match message {
            FromCompositorMsg::Exit => {
                debug!("constellation exiting");
                self.handle_exit();
            }
            FromCompositorMsg::GetBrowsingContext(pipeline_id, resp_chan) => {
                debug!("constellation got get browsing context message");
                self.handle_get_browsing_context(pipeline_id, resp_chan);
            }
            FromCompositorMsg::GetPipeline(browsing_context_id, resp_chan) => {
                debug!("constellation got get pipeline message");
                self.handle_get_pipeline(browsing_context_id, resp_chan);
            }
            FromCompositorMsg::GetFocusTopLevelBrowsingContext(resp_chan) => {
                debug!("constellation got get focus browsing context message");
                let focus_browsing_context = self.focus_pipeline_id
                    .and_then(|pipeline_id| self.pipelines.get(&pipeline_id))
                    .map(|pipeline| pipeline.top_level_browsing_context_id);
                let _ = resp_chan.send(focus_browsing_context);
            }
            FromCompositorMsg::KeyEvent(ch, key, state, modifiers) => {
                debug!("constellation got key event message");
                self.handle_key_msg(ch, key, state, modifiers);
            }
            // Load a new page from a typed url
            // If there is already a pending page (self.pending_changes), it will not be overridden;
            // However, if the id is not encompassed by another change, it will be.
            FromCompositorMsg::LoadUrl(top_level_browsing_context_id, url) => {
                debug!("constellation got URL load message from compositor");
                let load_data = LoadData::new(url, None, None, None);
                let ctx_id = BrowsingContextId::from(top_level_browsing_context_id);
                let pipeline_id = match self.browsing_contexts.get(&ctx_id) {
                    Some(ctx) => ctx.pipeline_id,
                    None => return warn!("LoadUrl for unknow browsing context: {:?}", top_level_browsing_context_id),
                };
                self.handle_load_url_msg(top_level_browsing_context_id, pipeline_id, load_data, false);
            }
            FromCompositorMsg::IsReadyToSaveImage(pipeline_states) => {
                let is_ready = self.handle_is_ready_to_save_image(pipeline_states);
                debug!("Ready to save image {:?}.", is_ready);
                if opts::get().is_running_problem_test {
                    println!("got ready to save image query, result is {:?}", is_ready);
                }
                let is_ready = is_ready == ReadyToSave::Ready;
                self.compositor_proxy.send(ToCompositorMsg::IsReadyToSaveImageReply(is_ready));
                if opts::get().is_running_problem_test {
                    println!("sent response");
                }
            }
            // Create a new top level browsing context. Will use response_chan to return
            // the browsing context id.
            FromCompositorMsg::NewBrowser(url, response_chan) => {
                debug!("constellation got NewBrowser message");
                self.handle_new_top_level_browsing_context(url, response_chan);
            }
            // Close a top level browsing context.
            FromCompositorMsg::CloseBrowser(top_level_browsing_context_id) => {
                debug!("constellation got CloseBrowser message");
                self.handle_close_top_level_browsing_context(top_level_browsing_context_id);
            }
            // Send frame tree to WebRender. Make it visible.
            FromCompositorMsg::SelectBrowser(top_level_browsing_context_id) => {
                self.send_frame_tree(top_level_browsing_context_id);
            }
            // Handle a forward or back request
            FromCompositorMsg::TraverseHistory(top_level_browsing_context_id, direction) => {
                debug!("constellation got traverse history message from compositor");
                self.handle_traverse_history_msg(top_level_browsing_context_id, direction);
            }
            FromCompositorMsg::WindowSize(top_level_browsing_context_id, new_size, size_type) => {
                debug!("constellation got window resize message");
                self.handle_window_size_msg(top_level_browsing_context_id, new_size, size_type);
            }
            FromCompositorMsg::TickAnimation(pipeline_id, tick_type) => {
                self.handle_tick_animation(pipeline_id, tick_type)
            }
            FromCompositorMsg::WebDriverCommand(command) => {
                debug!("constellation got webdriver command message");
                self.handle_webdriver_msg(command);
            }
            FromCompositorMsg::Reload(top_level_browsing_context_id) => {
                debug!("constellation got reload message");
                self.handle_reload_msg(top_level_browsing_context_id);
            }
            FromCompositorMsg::LogEntry(top_level_browsing_context_id, thread_name, entry) => {
                self.handle_log_entry(top_level_browsing_context_id, thread_name, entry);
            }
            FromCompositorMsg::WebVREvents(pipeline_ids, events) => {
                debug!("constellation got {:?} WebVR events", events.len());
                self.handle_webvr_events(pipeline_ids, events);
            }
            FromCompositorMsg::ForwardEvent(destination_pipeline_id, event) => {
                self.forward_event(destination_pipeline_id, event);
            }
            FromCompositorMsg::SetCursor(cursor) => {
                self.handle_set_cursor_msg(cursor)
            }
        }
    }

    fn handle_request_from_script(&mut self, message: (PipelineId, FromScriptMsg)) {
        let (source_pipeline_id, content) = message;
        let source_top_ctx_id = match self.pipelines.get(&source_pipeline_id)
            .map(|pipeline| pipeline.top_level_browsing_context_id) {
                None => return warn!("ScriptMsg from closed pipeline {:?}.", source_pipeline_id),
                Some(ctx) => ctx,
        };

        let source_is_top_level_pipeline = self.browsing_contexts
            .get(&BrowsingContextId::from(source_top_ctx_id))
            .map(|ctx| ctx.pipeline_id == source_pipeline_id)
            .unwrap_or(false);

        match content {
            FromScriptMsg::PipelineExited => {
                self.handle_pipeline_exited(source_pipeline_id);
            }
            FromScriptMsg::InitiateNavigateRequest(req_init, cancel_chan) => {
                debug!("constellation got initiate navigate request message");
                self.handle_navigate_request(source_pipeline_id, req_init, cancel_chan);
            }
            FromScriptMsg::ScriptLoadedURLInIFrame(load_info) => {
                debug!("constellation got iframe URL load message {:?} {:?} {:?}",
                       load_info.info.parent_pipeline_id,
                       load_info.old_pipeline_id,
                       load_info.info.new_pipeline_id);
                self.handle_script_loaded_url_in_iframe_msg(load_info);
            }
            FromScriptMsg::ScriptNewIFrame(load_info, layout_sender) => {
                debug!("constellation got loaded `about:blank` in iframe message {:?} {:?}",
                       load_info.parent_pipeline_id,
                       load_info.new_pipeline_id);
                self.handle_script_new_iframe(load_info, layout_sender);
            }
            FromScriptMsg::ChangeRunningAnimationsState(animation_state) => {
                self.handle_change_running_animations_state(source_pipeline_id, animation_state)
            }
            // Load a new page from a mouse click
            // If there is already a pending page (self.pending_changes), it will not be overridden;
            // However, if the id is not encompassed by another change, it will be.
            FromScriptMsg::LoadUrl(load_data, replace) => {
                debug!("constellation got URL load message from script");
                self.handle_load_url_msg(source_top_ctx_id, source_pipeline_id, load_data, replace);
            }
            FromScriptMsg::AbortLoadUrl => {
                debug!("constellation got abort URL load message from script");
                self.handle_abort_load_url_msg(source_pipeline_id);
            }
            // A page loaded has completed all parsing, script, and reflow messages have been sent.
            FromScriptMsg::LoadComplete => {
                debug!("constellation got load complete message");
                self.handle_load_complete_msg(source_top_ctx_id, source_pipeline_id)
            }
            // Handle a forward or back request
            FromScriptMsg::TraverseHistory(direction) => {
                debug!("constellation got traverse history message from script");
                self.handle_traverse_history_msg(source_top_ctx_id, direction);
            }
            // Handle a joint session history length request.
            FromScriptMsg::JointSessionHistoryLength(sender) => {
                debug!("constellation got joint session history length message from script");
                self.handle_joint_session_history_length(source_top_ctx_id, sender);
            }
            // Notification that the new document is ready to become active
            FromScriptMsg::ActivateDocument => {
                debug!("constellation got activate document message");
                self.handle_activate_document_msg(source_pipeline_id);
            }
            // Update pipeline url after redirections
            FromScriptMsg::SetFinalUrl(final_url) => {
                // The script may have finished loading after we already started shutting down.
                if let Some(ref mut pipeline) = self.pipelines.get_mut(&source_pipeline_id) {
                    debug!("constellation got set final url message");
                    pipeline.url = final_url;
                } else {
                    warn!("constellation got set final url message for dead pipeline");
                }
            }
            FromScriptMsg::PostMessage(browsing_context_id, origin, data) => {
                debug!("constellation got postMessage message");
                self.handle_post_message_msg(browsing_context_id, origin, data);
            }
            FromScriptMsg::MozBrowserEvent(pipeline_id, event) => {
                debug!("constellation got mozbrowser event message");
                self.handle_mozbrowser_event_msg(pipeline_id, source_top_ctx_id, event);
            }
            FromScriptMsg::Focus => {
                debug!("constellation got focus message");
                self.handle_focus_msg(source_pipeline_id);
            }
            FromScriptMsg::ForwardEvent(destination_pipeline_id, event) => {
                self.forward_event(destination_pipeline_id, event);
            }
            FromScriptMsg::GetClipboardContents(sender) => {
                let contents = match self.clipboard_ctx {
                    Some(ref mut ctx) => match ctx.get_contents() {
                        Ok(c) => c,
                        Err(e) => {
                            warn!("Error getting clipboard contents ({}), defaulting to empty string", e);
                            "".to_owned()
                        },
                    },
                    None => "".to_owned(),
                };
                if let Err(e) = sender.send(contents.to_owned()) {
                    warn!("Failed to send clipboard ({})", e);
                }
            }
            FromScriptMsg::SetClipboardContents(s) => {
                if let Some(ref mut ctx) = self.clipboard_ctx {
                    if let Err(e) = ctx.set_contents(s) {
                        warn!("Error setting clipboard contents ({})", e);
                    }
                }
            }
            FromScriptMsg::SetVisible(visible) => {
                debug!("constellation got set visible messsage");
                self.handle_set_visible_msg(source_pipeline_id, visible);
            }
            FromScriptMsg::VisibilityChangeComplete(visible) => {
                debug!("constellation got set visibility change complete message");
                self.handle_visibility_change_complete(source_pipeline_id, visible);
            }
            FromScriptMsg::RemoveIFrame(browsing_context_id, sender) => {
                debug!("constellation got remove iframe message");
                let removed_pipeline_ids = self.handle_remove_iframe_msg(browsing_context_id);
                if let Err(e) = sender.send(removed_pipeline_ids) {
                    warn!("Error replying to remove iframe ({})", e);
                }
            }
            FromScriptMsg::NewFavicon(url) => {
                debug!("constellation got new favicon message");
                if source_is_top_level_pipeline {
                    self.embedder_proxy.send(EmbedderMsg::NewFavicon(source_top_ctx_id, url));
                }
            }
            FromScriptMsg::HeadParsed => {
                debug!("constellation got head parsed message");
                if source_is_top_level_pipeline {
                    self.embedder_proxy.send(EmbedderMsg::HeadParsed(source_top_ctx_id));
                }
            }
            FromScriptMsg::CreateCanvasPaintThread(size, sender) => {
                debug!("constellation got create-canvas-paint-thread message");
                self.handle_create_canvas_paint_thread_msg(&size, sender)
            }
            FromScriptMsg::NodeStatus(message) => {
                debug!("constellation got NodeStatus message");
                self.embedder_proxy.send(EmbedderMsg::Status(source_top_ctx_id, message));
            }
            FromScriptMsg::SetDocumentState(state) => {
                debug!("constellation got SetDocumentState message");
                self.document_states.insert(source_pipeline_id, state);
            }
            FromScriptMsg::Alert(message, sender) => {
                debug!("constellation got Alert message");
                self.handle_alert(source_top_ctx_id, message, sender);
            }
            FromScriptMsg::GetClientWindow(send) => {
                self.embedder_proxy.send(EmbedderMsg::GetClientWindow(source_top_ctx_id, send));
            }

            FromScriptMsg::MoveTo(point) => {
                self.embedder_proxy.send(EmbedderMsg::MoveTo(source_top_ctx_id, point));
            }

            FromScriptMsg::ResizeTo(size) => {
                self.embedder_proxy.send(EmbedderMsg::ResizeTo(source_top_ctx_id, size));
            }

            FromScriptMsg::GetScreenSize(send) => {
                self.embedder_proxy.send(EmbedderMsg::GetScreenSize(source_top_ctx_id, send));
            }

            FromScriptMsg::GetScreenAvailSize(send) => {
                self.embedder_proxy.send(EmbedderMsg::GetScreenAvailSize(source_top_ctx_id, send));
            }

            FromScriptMsg::Exit => {
                self.compositor_proxy.send(ToCompositorMsg::Exit);
            }
            FromScriptMsg::LogEntry(thread_name, entry) => {
                self.handle_log_entry(Some(source_top_ctx_id), thread_name, entry);
            }

            FromScriptMsg::SetTitle(title) => {
                if source_is_top_level_pipeline {
                    self.embedder_proxy.send(EmbedderMsg::ChangePageTitle(source_top_ctx_id, title))
                }
            }

            FromScriptMsg::SendKeyEvent(ch, key, key_state, key_modifiers) => {
                let event = EmbedderMsg::KeyEvent(Some(source_top_ctx_id), ch, key, key_state, key_modifiers);
                self.embedder_proxy.send(event);
            }

            FromScriptMsg::TouchEventProcessed(result) => {
                self.compositor_proxy.send(ToCompositorMsg::TouchEventProcessed(result))
            }
            FromScriptMsg::GetBrowsingContextId(pipeline_id, sender) => {
                let result = self.pipelines.get(&pipeline_id).map(|pipeline| pipeline.browsing_context_id);
                if let Err(e) = sender.send(result) {
                    warn!("Sending reply to get browsing context failed ({:?}).", e);
                }
            }
            FromScriptMsg::GetParentInfo(pipeline_id, sender) => {
                let result = self.pipelines.get(&pipeline_id).and_then(|pipeline| pipeline.parent_info);
                if let Err(e) = sender.send(result) {
                    warn!("Sending reply to get parent info failed ({:?}).", e);
                }
            }
            FromScriptMsg::RegisterServiceWorker(scope_things, scope) => {
                debug!("constellation got store registration scope message");
                self.handle_register_serviceworker(scope_things, scope);
            }
            FromScriptMsg::ForwardDOMMessage(msg_vec, scope_url) => {
                if let Some(ref mgr) = self.swmanager_chan {
                    let _ = mgr.send(ServiceWorkerMsg::ForwardDOMMessage(msg_vec, scope_url));
                } else {
                    warn!("Unable to forward DOMMessage for postMessage call");
                }
            }
            FromScriptMsg::BroadcastStorageEvent(storage, url, key, old_value, new_value) => {
                self.handle_broadcast_storage_event(source_pipeline_id, storage, url, key, old_value, new_value);
            }
            FromScriptMsg::SetFullscreenState(state) => {
                self.embedder_proxy.send(EmbedderMsg::SetFullscreenState(source_top_ctx_id, state));
            }
        }
    }

    fn handle_request_from_layout(&mut self, message: FromLayoutMsg) {
        match message {
            FromLayoutMsg::ChangeRunningAnimationsState(pipeline_id, animation_state) => {
                self.handle_change_running_animations_state(pipeline_id, animation_state)
            }
            // Layout sends new sizes for all subframes. This needs to be reflected by all
            // frame trees in the navigation context containing the subframe.
            FromLayoutMsg::IFrameSizes(iframe_sizes) => {
                debug!("constellation got iframe size message");
                self.handle_iframe_size_msg(iframe_sizes);
            }
            FromLayoutMsg::PendingPaintMetric(pipeline_id, epoch) => {
                debug!("constellation got a pending paint metric message");
                self.handle_pending_paint_metric(pipeline_id, epoch);
            }
            FromLayoutMsg::SetCursor(cursor) => {
                self.handle_set_cursor_msg(cursor)
            }
            FromLayoutMsg::ViewportConstrained(pipeline_id, constraints) => {
                debug!("constellation got viewport-constrained event message");
                self.handle_viewport_constrained_msg(pipeline_id, constraints);
            }
        }
    }

    fn handle_register_serviceworker(&self, scope_things: ScopeThings, scope: ServoUrl) {
        if let Some(ref mgr) = self.swmanager_chan {
            let _ = mgr.send(ServiceWorkerMsg::RegisterServiceWorker(scope_things, scope));
        } else {
            warn!("sending scope info to service worker manager failed");
        }
    }

    fn handle_broadcast_storage_event(&self, pipeline_id: PipelineId, storage: StorageType, url: ServoUrl,
                                      key: Option<String>, old_value: Option<String>, new_value: Option<String>) {
        let origin = url.origin();
        for pipeline in self.pipelines.values() {
            if (pipeline.id != pipeline_id) && (pipeline.url.origin() == origin) {
                let msg = ConstellationControlMsg::DispatchStorageEvent(
                    pipeline.id, storage, url.clone(), key.clone(), old_value.clone(), new_value.clone()
                );
                if let Err(err) = pipeline.event_loop.send(msg) {
                    warn!("Failed to broadcast storage event to pipeline {} ({:?}).", pipeline.id, err);
                }
            }
        }
    }

    fn handle_exit(&mut self) {
        // TODO: add a timer, which forces shutdown if threads aren't responsive.
        if self.shutting_down { return; }
        self.shutting_down = true;

        self.mem_profiler_chan.send(mem::ProfilerMsg::Exit);

        // Close the top-level browsing contexts
        let browsing_context_ids: Vec<BrowsingContextId> = self.browsing_contexts.values()
            .filter(|browsing_context| browsing_context.is_top_level())
            .map(|browsing_context| browsing_context.id)
            .collect();
        for browsing_context_id in browsing_context_ids {
            debug!("Removing top-level browsing context {}.", browsing_context_id);
            self.close_browsing_context(browsing_context_id, ExitPipelineMode::Normal);
        }

        // Close any pending changes and pipelines
        while let Some(pending) = self.pending_changes.pop() {
            debug!("Removing pending browsing context {}.", pending.browsing_context_id);
            self.close_browsing_context(pending.browsing_context_id, ExitPipelineMode::Normal);
            debug!("Removing pending pipeline {}.", pending.new_pipeline_id);
            self.close_pipeline(pending.new_pipeline_id, DiscardBrowsingContext::Yes, ExitPipelineMode::Normal);
        }

        // In case there are browsing contexts which weren't attached, we close them.
        let browsing_context_ids: Vec<BrowsingContextId> = self.browsing_contexts.keys().cloned().collect();
        for browsing_context_id in browsing_context_ids {
            debug!("Removing detached browsing context {}.", browsing_context_id);
            self.close_browsing_context(browsing_context_id, ExitPipelineMode::Normal);
        }

        // In case there are pipelines which weren't attached to the pipeline tree, we close them.
        let pipeline_ids: Vec<PipelineId> = self.pipelines.keys().cloned().collect();
        for pipeline_id in pipeline_ids {
            debug!("Removing detached pipeline {}.", pipeline_id);
            self.close_pipeline(pipeline_id, DiscardBrowsingContext::Yes, ExitPipelineMode::Normal);
        }
    }

    fn handle_shutdown(&mut self) {
        // At this point, there are no active pipelines,
        // so we can safely block on other threads, without worrying about deadlock.
        // Channels to receive signals when threads are done exiting.
        let (core_sender, core_receiver) = ipc::channel().expect("Failed to create IPC channel!");
        let (storage_sender, storage_receiver) = ipc::channel().expect("Failed to create IPC channel!");

        debug!("Exiting core resource threads.");
        if let Err(e) = self.public_resource_threads.send(net_traits::CoreResourceMsg::Exit(core_sender)) {
            warn!("Exit resource thread failed ({})", e);
        }

        if let Some(ref chan) = self.debugger_chan {
            debugger::shutdown_server(chan);
        }

        if let Some(ref chan) = self.devtools_chan {
            debug!("Exiting devtools.");
            let msg = DevtoolsControlMsg::FromChrome(ChromeToDevtoolsControlMsg::ServerExitMsg);
            if let Err(e) = chan.send(msg) {
                warn!("Exit devtools failed ({})", e);
            }
        }

        debug!("Exiting storage resource threads.");
        if let Err(e) = self.public_resource_threads.send(StorageThreadMsg::Exit(storage_sender)) {
            warn!("Exit storage thread failed ({})", e);
        }

        debug!("Exiting bluetooth thread.");
        if let Err(e) = self.bluetooth_thread.send(BluetoothRequest::Exit) {
            warn!("Exit bluetooth thread failed ({})", e);
        }

        debug!("Exiting service worker manager thread.");
        if let Some(mgr) = self.swmanager_chan.as_ref() {
            if let Err(e) = mgr.send(ServiceWorkerMsg::Exit) {
                warn!("Exit service worker manager failed ({})", e);
            }
        }

        debug!("Exiting WebGL thread.");
        if let Err(e) = self.webgl_threads.exit() {
            warn!("Exit WebGL Thread failed ({})", e);
        }

        if let Some(chan) = self.webvr_chan.as_ref() {
            debug!("Exiting WebVR thread.");
            if let Err(e) = chan.send(WebVRMsg::Exit) {
                warn!("Exit WebVR thread failed ({})", e);
            }
        }

        debug!("Exiting timer scheduler.");
        if let Err(e) = self.scheduler_chan.send(TimerSchedulerMsg::Exit) {
            warn!("Exit timer scheduler failed ({})", e);
        }

        debug!("Exiting font cache thread.");
        self.font_cache_thread.exit();

        // Receive exit signals from threads.
        if let Err(e) = core_receiver.recv() {
            warn!("Exit resource thread failed ({})", e);
        }
        if let Err(e) = storage_receiver.recv() {
            warn!("Exit storage thread failed ({})", e);
        }

        debug!("Asking compositor to complete shutdown.");
        self.compositor_proxy.send(ToCompositorMsg::ShutdownComplete);
    }

    fn handle_pipeline_exited(&mut self, pipeline_id: PipelineId) {
        debug!("Pipeline {:?} exited.", pipeline_id);
        self.pipelines.remove(&pipeline_id);
    }

    fn handle_send_error(&mut self, pipeline_id: PipelineId, err: IpcError) {
        // Treat send error the same as receiving a panic message
        error!("Pipeline {} send error ({}).", pipeline_id, err);
        let top_level_browsing_context_id = self.pipelines.get(&pipeline_id)
            .map(|pipeline| pipeline.top_level_browsing_context_id);
        if let Some(top_level_browsing_context_id) = top_level_browsing_context_id {
            let reason = format!("Send failed ({})", err);
            self.handle_panic(top_level_browsing_context_id, reason, None);
        }
    }

    fn handle_panic(&mut self,
                    top_level_browsing_context_id: TopLevelBrowsingContextId,
                    reason: String,
                    backtrace: Option<String>)
    {
        if opts::get().hard_fail {
            // It's quite difficult to make Servo exit cleanly if some threads have failed.
            // Hard fail exists for test runners so we crash and that's good enough.
            println!("Pipeline failed in hard-fail mode.  Crashing!");
            process::exit(1);
        }

        debug!("Panic handler for top-level browsing context {}: {}.", top_level_browsing_context_id, reason);

        let browsing_context_id = BrowsingContextId::from(top_level_browsing_context_id);

        // Notify the browser chrome that the pipeline has failed
        self.trigger_mozbrowsererror(top_level_browsing_context_id, reason, backtrace);

        let (window_size, pipeline_id) = {
            let browsing_context = self.browsing_contexts.get(&browsing_context_id);
            let window_size = browsing_context.and_then(|browsing_context| browsing_context.size);
            let pipeline_id = browsing_context.map(|browsing_context| browsing_context.pipeline_id);
            (window_size, pipeline_id)
        };

        let (pipeline_url, parent_info) = {
            let pipeline = pipeline_id.and_then(|id| self.pipelines.get(&id));
            let pipeline_url = pipeline.map(|pipeline| pipeline.url.clone());
            let parent_info = pipeline.and_then(|pipeline| pipeline.parent_info);
            (pipeline_url, parent_info)
        };

        self.close_browsing_context_children(browsing_context_id,
                                             DiscardBrowsingContext::No,
                                             ExitPipelineMode::Force);

        let failure_url = ServoUrl::parse("about:failure").expect("infallible");

        if let Some(pipeline_url) = pipeline_url {
            if pipeline_url == failure_url {
                return error!("about:failure failed");
            }
        }

        warn!("creating replacement pipeline for about:failure");

        let new_pipeline_id = PipelineId::new();
        let load_data = LoadData::new(failure_url, None, None, None);
        let sandbox = IFrameSandboxState::IFrameSandboxed;
        self.new_pipeline(new_pipeline_id, browsing_context_id, top_level_browsing_context_id, parent_info,
                          window_size, load_data.clone(), sandbox, false);
        self.add_pending_change(SessionHistoryChange {
            top_level_browsing_context_id: top_level_browsing_context_id,
            browsing_context_id: browsing_context_id,
            new_pipeline_id: new_pipeline_id,
            load_data: load_data,
            replace_instant: None,
        });
    }

    fn handle_log_entry(&mut self,
                        top_level_browsing_context_id: Option<TopLevelBrowsingContextId>,
                        thread_name: Option<String>,
                        entry: LogEntry)
    {
        debug!("Received log entry {:?}.", entry);
        match (entry, top_level_browsing_context_id) {
            (LogEntry::Panic(reason, backtrace), Some(top_level_browsing_context_id)) => {
                self.handle_panic(top_level_browsing_context_id, reason, Some(backtrace));
            },
            (LogEntry::Panic(reason, _), _) | (LogEntry::Error(reason), _) | (LogEntry::Warn(reason), _) => {
                // VecDeque::truncate is unstable
                if WARNINGS_BUFFER_SIZE <= self.handled_warnings.len() {
                    self.handled_warnings.pop_front();
                }
                self.handled_warnings.push_back((thread_name, reason));
            },
        }
    }

    fn handle_webvr_events(&mut self, ids: Vec<PipelineId>, events: Vec<WebVREvent>) {
        for id in ids {
            match self.pipelines.get_mut(&id) {
                Some(ref pipeline) => {
                    // Notify script thread
                    let _ = pipeline.event_loop.send(ConstellationControlMsg::WebVREvents(id, events.clone()));
                },
                None => warn!("constellation got webvr event for dead pipeline")
            }
        }
    }

    fn forward_event(&mut self, destination_pipeline_id: PipelineId, event: CompositorEvent) {
        let msg = ConstellationControlMsg::SendEvent(destination_pipeline_id, event);
        let result = match self.pipelines.get(&destination_pipeline_id) {
            None => {
                debug!("Pipeline {:?} got event after closure.", destination_pipeline_id);
                return;
            }
            Some(pipeline) => pipeline.event_loop.send(msg),
        };
        if let Err(e) = result {
            self.handle_send_error(destination_pipeline_id, e);
        }
    }

    fn handle_new_top_level_browsing_context(&mut self, url: ServoUrl, reply: IpcSender<TopLevelBrowsingContextId>) {
        let window_size = self.window_size.initial_viewport;
        let pipeline_id = PipelineId::new();
        let top_level_browsing_context_id = TopLevelBrowsingContextId::new();
        if let Err(e) = reply.send(top_level_browsing_context_id) {
            warn!("Failed to send newly created top level browsing context ({}).", e);
        }
        let browsing_context_id = BrowsingContextId::from(top_level_browsing_context_id);
        let load_data = LoadData::new(url.clone(), None, None, None);
        let sandbox = IFrameSandboxState::IFrameUnsandboxed;
        if self.focus_pipeline_id.is_none() {
            self.focus_pipeline_id = Some(pipeline_id);
        }
        self.new_pipeline(pipeline_id,
                          browsing_context_id,
                          top_level_browsing_context_id,
                          None,
                          Some(window_size),
                          load_data.clone(),
                          sandbox,
                          false);
        self.add_pending_change(SessionHistoryChange {
            top_level_browsing_context_id: top_level_browsing_context_id,
            browsing_context_id: browsing_context_id,
            new_pipeline_id: pipeline_id,
            load_data: load_data,
            replace_instant: None,
        });
    }

    fn handle_close_top_level_browsing_context(&mut self, top_level_browsing_context_id: TopLevelBrowsingContextId) {
        let browsing_context_id = BrowsingContextId::from(top_level_browsing_context_id);
        self.close_browsing_context(browsing_context_id, ExitPipelineMode::Normal);
    }

    fn handle_iframe_size_msg(&mut self,
                              iframe_sizes: Vec<(BrowsingContextId, TypedSize2D<f32, CSSPixel>)>) {
        for (browsing_context_id, size) in iframe_sizes {
            let window_size = WindowSizeData {
                initial_viewport: size,
                device_pixel_ratio: self.window_size.device_pixel_ratio,
            };

            self.resize_browsing_context(window_size, WindowSizeType::Initial, browsing_context_id);
        }
    }

    fn handle_subframe_loaded(&mut self, pipeline_id: PipelineId) {
        let (browsing_context_id, parent_id) = match self.pipelines.get(&pipeline_id) {
            Some(pipeline) => match pipeline.parent_info {
                Some((parent_id, _)) => (pipeline.browsing_context_id, parent_id),
                None => return debug!("Pipeline {} has no parent.", pipeline_id),
            },
            None => return warn!("Pipeline {} loaded after closure.", pipeline_id),
        };
        let msg = ConstellationControlMsg::DispatchIFrameLoadEvent {
            target: browsing_context_id,
            parent: parent_id,
            child: pipeline_id,
        };
        let result = match self.pipelines.get(&parent_id) {
            Some(parent) => parent.event_loop.send(msg),
            None => return warn!("Parent {} browsing context loaded after closure.", parent_id),
        };
        if let Err(e) = result {
            self.handle_send_error(parent_id, e);
        }
    }

    fn handle_navigate_request(&self,
                              id: PipelineId,
                              req_init: RequestInit,
                              cancel_chan: IpcReceiver<()>) {
        let listener = NetworkListener::new(
                           req_init,
                           id,
                           self.public_resource_threads.clone(),
                           self.network_listener_sender.clone());

        listener.initiate_fetch(Some(cancel_chan));
    }

    // The script thread associated with pipeline_id has loaded a URL in an iframe via script. This
    // will result in a new pipeline being spawned and a child being added to
    // the parent pipeline. This message is never the result of a
    // page navigation.
    fn handle_script_loaded_url_in_iframe_msg(&mut self, load_info: IFrameLoadInfoWithData) {
        let (load_data, window_size, is_private) = {
            let old_pipeline = load_info.old_pipeline_id
                .and_then(|old_pipeline_id| self.pipelines.get(&old_pipeline_id));

            let source_pipeline = match self.pipelines.get(&load_info.info.parent_pipeline_id) {
                Some(source_pipeline) => source_pipeline,
                None => return warn!("Script loaded url in closed iframe {}.", load_info.info.parent_pipeline_id),
            };

            // If no url is specified, reload.
            let load_data = load_info.load_data.unwrap_or_else(|| {
                let url = match old_pipeline {
                    Some(old_pipeline) => old_pipeline.url.clone(),
                    None => ServoUrl::parse("about:blank").expect("infallible"),
                };

                // TODO - loaddata here should have referrer info (not None, None)
                LoadData::new(url, Some(source_pipeline.id), None, None)
            });

            let is_private = load_info.info.is_private || source_pipeline.is_private;

            let window_size = self.browsing_contexts.get(&load_info.info.browsing_context_id)
                .and_then(|browsing_context| browsing_context.size);

            (load_data, window_size, is_private)
        };

        let replace_instant = if load_info.info.replace {
            self.browsing_contexts.get(&load_info.info.browsing_context_id)
                .map(|browsing_context| browsing_context.instant)
        } else {
            None
        };

        // Create the new pipeline, attached to the parent and push to pending changes
        self.new_pipeline(load_info.info.new_pipeline_id,
                          load_info.info.browsing_context_id,
                          load_info.info.top_level_browsing_context_id,
                          Some((load_info.info.parent_pipeline_id, load_info.info.frame_type)),
                          window_size,
                          load_data.clone(),
                          load_info.sandbox,
                          is_private);
        self.add_pending_change(SessionHistoryChange {
            top_level_browsing_context_id: load_info.info.top_level_browsing_context_id,
            browsing_context_id: load_info.info.browsing_context_id,
            new_pipeline_id: load_info.info.new_pipeline_id,
            load_data: load_data,
            replace_instant: replace_instant,
        });
    }

    fn handle_script_new_iframe(&mut self,
                                load_info: IFrameLoadInfo,
                                layout_sender: IpcSender<LayoutControlMsg>) {
        let IFrameLoadInfo {
            parent_pipeline_id,
            new_pipeline_id,
            frame_type,
            replace,
            browsing_context_id,
            top_level_browsing_context_id,
            is_private,
        } = load_info;

        let url = ServoUrl::parse("about:blank").expect("infallible");

        let pipeline = {
            let parent_pipeline = match self.pipelines.get(&parent_pipeline_id) {
                Some(parent_pipeline) => parent_pipeline,
                None => return warn!("Script loaded url in closed iframe {}.", parent_pipeline_id),
            };

            let script_sender = parent_pipeline.event_loop.clone();

            Pipeline::new(new_pipeline_id,
                          browsing_context_id,
                          top_level_browsing_context_id,
                          Some((parent_pipeline_id, frame_type)),
                          script_sender,
                          layout_sender,
                          self.compositor_proxy.clone(),
                          is_private || parent_pipeline.is_private,
                          url.clone(),
                          parent_pipeline.visible)
        };

        // TODO: Referrer?
        let load_data = LoadData::new(url, Some(parent_pipeline_id), None, None);

        let replace_instant = if replace {
            self.browsing_contexts.get(&browsing_context_id).map(|browsing_context| browsing_context.instant)
        } else {
            None
        };

        assert!(!self.pipelines.contains_key(&new_pipeline_id));
        self.pipelines.insert(new_pipeline_id, pipeline);

        self.add_pending_change(SessionHistoryChange {
            top_level_browsing_context_id: top_level_browsing_context_id,
            browsing_context_id: browsing_context_id,
            new_pipeline_id: new_pipeline_id,
            load_data: load_data,
            replace_instant: replace_instant,
        });
    }

    fn handle_pending_paint_metric(&self, pipeline_id: PipelineId, epoch: Epoch) {
        self.compositor_proxy.send(ToCompositorMsg::PendingPaintMetric(pipeline_id, epoch))
    }

    fn handle_set_cursor_msg(&mut self, cursor: CursorKind) {
        self.embedder_proxy.send(EmbedderMsg::SetCursor(cursor))
    }

    fn handle_change_running_animations_state(&mut self,
                                              pipeline_id: PipelineId,
                                              animation_state: AnimationState) {
        self.compositor_proxy.send(ToCompositorMsg::ChangeRunningAnimationsState(pipeline_id,
                                                                               animation_state))
    }

    fn handle_tick_animation(&mut self, pipeline_id: PipelineId, tick_type: AnimationTickType) {
        let result = match tick_type {
            AnimationTickType::Script => {
                let msg = ConstellationControlMsg::TickAllAnimations(pipeline_id);
                match self.pipelines.get(&pipeline_id) {
                    Some(pipeline) => pipeline.event_loop.send(msg),
                    None => return warn!("Pipeline {:?} got script tick after closure.", pipeline_id),
                }
            }
            AnimationTickType::Layout => {
                let msg = LayoutControlMsg::TickAnimations;
                match self.pipelines.get(&pipeline_id) {
                    Some(pipeline) => pipeline.layout_chan.send(msg),
                    None => return warn!("Pipeline {:?} got layout tick after closure.", pipeline_id),
                }
            }
        };
        if let Err(e) = result {
            self.handle_send_error(pipeline_id, e);
        }
    }

    fn handle_alert(&mut self,
                    top_level_browsing_context_id: TopLevelBrowsingContextId,
                    message: String,
                    sender: IpcSender<bool>) {
        let browser_pipeline_id = self.browsing_contexts.get(&BrowsingContextId::from(top_level_browsing_context_id))
            .and_then(|browsing_context| self.pipelines.get(&browsing_context.pipeline_id))
            .and_then(|pipeline| pipeline.parent_info)
            .map(|(browser_pipeline_id, _)| browser_pipeline_id);
        let mozbrowser_modal_prompt = PREFS.is_mozbrowser_enabled() && browser_pipeline_id.is_some();

        if mozbrowser_modal_prompt {
            // https://developer.mozilla.org/en-US/docs/Web/Events/mozbrowsershowmodalprompt
            let prompt_type = String::from("alert");
            let title = String::from("Alert");
            let return_value = String::from("");
            let event = MozBrowserEvent::ShowModalPrompt(prompt_type, title, message, return_value);
            match browser_pipeline_id.and_then(|id| self.pipelines.get(&id)) {
                None => warn!("Alert sent after browser pipeline closure."),
                Some(pipeline) => pipeline.trigger_mozbrowser_event(Some(top_level_browsing_context_id), event),
            }
        }

        let result = sender.send(!mozbrowser_modal_prompt);
        if let Err(e) = result {
            let ctx_id = BrowsingContextId::from(top_level_browsing_context_id);
            let pipeline_id = match self.browsing_contexts.get(&ctx_id) {
                Some(ctx) => ctx.pipeline_id,
                None => return warn!("Alert sent for unknown browsing context."),
            };
            self.handle_send_error(pipeline_id, e);
        }
    }

    fn handle_load_url_msg(&mut self, top_level_browsing_context_id: TopLevelBrowsingContextId, source_id: PipelineId,
                           load_data: LoadData, replace: bool) {
        self.load_url(top_level_browsing_context_id, source_id, load_data, replace);
    }

    fn load_url(&mut self, top_level_browsing_context_id: TopLevelBrowsingContextId, source_id: PipelineId,
                load_data: LoadData, replace: bool) -> Option<PipelineId> {
        // Allow the embedder to handle the url itself
        let (chan, port) = ipc::channel().expect("Failed to create IPC channel!");
        let msg = EmbedderMsg::AllowNavigation(top_level_browsing_context_id, load_data.url.clone(), chan);
        self.embedder_proxy.send(msg);
        if let Ok(false) = port.recv() {
            return None;
        }

        debug!("Loading {} in pipeline {}.", load_data.url, source_id);
        // If this load targets an iframe, its framing element may exist
        // in a separate script thread than the framed document that initiated
        // the new load. The framing element must be notified about the
        // requested change so it can update its internal state.
        //
        // If replace is true, the current entry is replaced instead of a new entry being added.
        let (browsing_context_id, parent_info) = match self.pipelines.get(&source_id) {
            Some(pipeline) => (pipeline.browsing_context_id, pipeline.parent_info),
            None => {
                warn!("Pipeline {} loaded after closure.", source_id);
                return None;
            }
        };
        match parent_info {
            Some((parent_pipeline_id, _)) => {
                // Find the script thread for the pipeline containing the iframe
                // and issue an iframe load through there.
                let msg = ConstellationControlMsg::Navigate(parent_pipeline_id,
                                                            browsing_context_id,
                                                            load_data,
                                                            replace);
                let result = match self.pipelines.get(&parent_pipeline_id) {
                    Some(parent_pipeline) => parent_pipeline.event_loop.send(msg),
                    None => {
                        warn!("Pipeline {:?} child loaded after closure", parent_pipeline_id);
                        return None;
                    },
                };
                if let Err(e) = result {
                    self.handle_send_error(parent_pipeline_id, e);
                }
                None
            }
            None => {
                // Make sure no pending page would be overridden.
                for change in &self.pending_changes {
                    if change.browsing_context_id == browsing_context_id {
                        // id that sent load msg is being changed already; abort
                        return None;
                    }
                }

                if self.get_activity(source_id) == DocumentActivity::Inactive {
                    // Disregard this load if the navigating pipeline is not actually
                    // active. This could be caused by a delayed navigation (eg. from
                    // a timer) or a race between multiple navigations (such as an
                    // onclick handler on an anchor element).
                    return None;
                }

                // Being here means either there are no pending changes, or none of the pending
                // changes would be overridden by changing the subframe associated with source_id.

                // Create the new pipeline
                let (top_level_id, window_size, timestamp) = match self.browsing_contexts.get(&browsing_context_id) {
                    Some(context) => (context.top_level_id, context.size, context.instant),
                    None => {
                        warn!("Browsing context {} loaded after closure.", browsing_context_id);
                        return None;
                    }
                };
                let new_pipeline_id = PipelineId::new();
                let sandbox = IFrameSandboxState::IFrameUnsandboxed;
                let replace_instant = if replace { Some(timestamp) } else { None };
                self.new_pipeline(new_pipeline_id,
                                  browsing_context_id,
                                  top_level_id,
                                  None,
                                  window_size,
                                  load_data.clone(),
                                  sandbox,
                                  false);
                self.add_pending_change(SessionHistoryChange {
                    top_level_browsing_context_id: top_level_id,
                    browsing_context_id: browsing_context_id,
                    new_pipeline_id: new_pipeline_id,
                    load_data: load_data,
                    replace_instant: replace_instant,
                });
                Some(new_pipeline_id)
            }
        }
    }

    fn handle_abort_load_url_msg(&mut self, new_pipeline_id: PipelineId) {
        let pending_index = self.pending_changes.iter().rposition(|change| {
            change.new_pipeline_id == new_pipeline_id
        });

        // If it is found, remove it from the pending changes.
        if let Some(pending_index) = pending_index {
            self.pending_changes.remove(pending_index);
            self.close_pipeline(new_pipeline_id, DiscardBrowsingContext::No, ExitPipelineMode::Normal);
        }
    }

    fn handle_load_start_msg(&mut self, top_level_browsing_context_id: TopLevelBrowsingContextId,
                             pipeline_id: PipelineId) {
        if self.pipelines.get(&pipeline_id).and_then(|p| p.parent_info).is_none() {
            // Notify embedder top level document started loading.
            self.embedder_proxy.send(EmbedderMsg::LoadStart(top_level_browsing_context_id));
        }
    }

    fn handle_load_complete_msg(&mut self, top_level_browsing_context_id: TopLevelBrowsingContextId,
                                pipeline_id: PipelineId) {
        let mut webdriver_reset = false;
        if let Some((expected_pipeline_id, ref reply_chan)) = self.webdriver.load_channel {
            debug!("Sending load to WebDriver");
            if expected_pipeline_id == pipeline_id {
                let _ = reply_chan.send(webdriver_msg::LoadStatus::LoadComplete);
                webdriver_reset = true;
            }
        }
        if webdriver_reset {
            self.webdriver.load_channel = None;
        }

        // Notify the embedder that the TopLevelBrowsingContext current document
        // has finished loading.
        // We need to make sure the pipeline that has finished loading is the current
        // pipeline and that no pending pipeline will replace the current one.
        let pipeline_is_top_level_pipeline = self.browsing_contexts
            .get(&BrowsingContextId::from(top_level_browsing_context_id))
            .map(|ctx| ctx.pipeline_id == pipeline_id)
            .unwrap_or(false);
        if pipeline_is_top_level_pipeline {
            // Is there any pending pipeline that will replace the current top level pipeline
            let current_top_level_pipeline_will_be_replaced = self.pending_changes.iter()
                .any(|change| change.browsing_context_id == top_level_browsing_context_id);

            if !current_top_level_pipeline_will_be_replaced {
                // Notify embedder and compositor top level document finished loading.
                self.compositor_proxy.send(ToCompositorMsg::LoadComplete(top_level_browsing_context_id));
                self.embedder_proxy.send(EmbedderMsg::LoadComplete(top_level_browsing_context_id));
            }
        }
        self.handle_subframe_loaded(pipeline_id);
    }

    fn handle_traverse_history_msg(&mut self,
                                   top_level_browsing_context_id: TopLevelBrowsingContextId,
                                   direction: TraversalDirection)
    {
        let mut size = 0;
        let mut table = HashMap::new();

        match direction {
            TraversalDirection::Forward(delta) => {
                for entry in self.joint_session_future(top_level_browsing_context_id).take(delta) {
                    size = size + 1;
                    table.insert(entry.browsing_context_id, entry.clone());
                }
                if size < delta {
                    return debug!("Traversing forward too much.");
                }
            },
            TraversalDirection::Back(delta) => {
                for entry in self.joint_session_past(top_level_browsing_context_id).take(delta) {
                    size = size + 1;
                    table.insert(entry.browsing_context_id, entry.clone());
                }
                if size < delta {
                    return debug!("Traversing back too much.");
                }
            },
        }

        for (_, entry) in table {
            self.traverse_to_entry(entry);
        }
    }

    fn handle_joint_session_history_length(&self,
                                           top_level_browsing_context_id: TopLevelBrowsingContextId,
                                           sender: IpcSender<u32>)
    {
        // Initialize length at 1 to count for the current active entry
        let mut length = 1;
        for browsing_context in self.all_browsing_contexts_iter(top_level_browsing_context_id) {
            length += browsing_context.next.len();
            length += browsing_context.prev.len();
        }
        let _ = sender.send(length as u32);
    }

    fn handle_key_msg(&mut self, ch: Option<char>, key: Key, state: KeyState, mods: KeyModifiers) {
        // Send to the explicitly focused pipeline. If it doesn't exist, fall back to sending to
        // the compositor.
        match self.focus_pipeline_id {
            Some(pipeline_id) => {
                let event = CompositorEvent::KeyEvent(ch, key, state, mods);
                let msg = ConstellationControlMsg::SendEvent(pipeline_id, event);
                let result = match self.pipelines.get(&pipeline_id) {
                    Some(pipeline) => pipeline.event_loop.send(msg),
                    None => return debug!("Pipeline {:?} got key event after closure.", pipeline_id),
                };
                if let Err(e) = result {
                    self.handle_send_error(pipeline_id, e);
                }
            },
            None => {
                let event = EmbedderMsg::KeyEvent(None, ch, key, state, mods);
                self.embedder_proxy.clone().send(event);
            }
        }
    }

    fn handle_reload_msg(&mut self, top_level_browsing_context_id: TopLevelBrowsingContextId) {
        let browsing_context_id = BrowsingContextId::from(top_level_browsing_context_id);
        let pipeline_id = match self.browsing_contexts.get(&browsing_context_id) {
            Some(browsing_context) => browsing_context.pipeline_id,
            None => return warn!("Browsing context {} got reload event after closure.", browsing_context_id),
        };
        let msg = ConstellationControlMsg::Reload(pipeline_id);
        let result = match self.pipelines.get(&pipeline_id) {
            None => return warn!("Pipeline {} got reload event after closure.", pipeline_id),
            Some(pipeline) => pipeline.event_loop.send(msg),
        };
        if let Err(e) = result {
            self.handle_send_error(pipeline_id, e);
        }
    }

    fn handle_post_message_msg(&mut self,
                               browsing_context_id: BrowsingContextId,
                               origin: Option<ImmutableOrigin>,
                               data: Vec<u8>)
    {
        let pipeline_id = match self.browsing_contexts.get(&browsing_context_id) {
            None => return warn!("postMessage to closed browsing_context {}.", browsing_context_id),
            Some(browsing_context) => browsing_context.pipeline_id,
        };
        let msg = ConstellationControlMsg::PostMessage(pipeline_id, origin, data);
        let result = match self.pipelines.get(&pipeline_id) {
            Some(pipeline) => pipeline.event_loop.send(msg),
            None => return warn!("postMessage to closed pipeline {}.", pipeline_id),
        };
        if let Err(e) = result {
            self.handle_send_error(pipeline_id, e);
        }
    }

    fn handle_mozbrowser_event_msg(&mut self,
                                   pipeline_id: PipelineId,
                                   top_level_browsing_context_id: TopLevelBrowsingContextId,
                                   event: MozBrowserEvent) {
        assert!(PREFS.is_mozbrowser_enabled());

        // Find the script channel for the given parent pipeline,
        // and pass the event to that script thread.
        // If the pipeline lookup fails, it is because we have torn down the pipeline,
        // so it is reasonable to silently ignore the event.
        match self.pipelines.get(&pipeline_id) {
            Some(pipeline) => pipeline.trigger_mozbrowser_event(Some(top_level_browsing_context_id), event),
            None => warn!("Pipeline {:?} handling mozbrowser event after closure.", pipeline_id),
        }
    }

    fn handle_get_pipeline(&mut self,
                           browsing_context_id: BrowsingContextId,
                           resp_chan: IpcSender<Option<PipelineId>>) {
        let current_pipeline_id = self.browsing_contexts.get(&browsing_context_id)
            .map(|browsing_context| browsing_context.pipeline_id);
        let pipeline_id_loaded = self.pending_changes.iter().rev()
            .find(|x| x.browsing_context_id == browsing_context_id)
            .map(|x| x.new_pipeline_id)
            .or(current_pipeline_id);
        if let Err(e) = resp_chan.send(pipeline_id_loaded) {
            warn!("Failed get_pipeline response ({}).", e);
        }
    }

    fn handle_get_browsing_context(&mut self,
                                   pipeline_id: PipelineId,
                                   resp_chan: IpcSender<Option<BrowsingContextId>>) {
        let browsing_context_id = self.pipelines.get(&pipeline_id).map(|pipeline| pipeline.browsing_context_id);
        if let Err(e) = resp_chan.send(browsing_context_id) {
            warn!("Failed get_browsing_context response ({}).", e);
        }
    }

    fn focus_parent_pipeline(&mut self, pipeline_id: PipelineId) {
        let (browsing_context_id, parent_info) = match self.pipelines.get(&pipeline_id) {
            Some(pipeline) => (pipeline.browsing_context_id, pipeline.parent_info),
            None => return warn!("Pipeline {:?} focus parent after closure.", pipeline_id),
        };
        let (parent_pipeline_id, _) = match parent_info {
            Some(info) => info,
            None => return debug!("Pipeline {:?} focus has no parent.", pipeline_id),
        };

        // Send a message to the parent of the provided pipeline (if it exists)
        // telling it to mark the iframe element as focused.
        let msg = ConstellationControlMsg::FocusIFrame(parent_pipeline_id, browsing_context_id);
        let result = match self.pipelines.get(&parent_pipeline_id) {
            Some(pipeline) => pipeline.event_loop.send(msg),
            None => return warn!("Pipeline {:?} focus after closure.", parent_pipeline_id),
        };
        if let Err(e) = result {
            self.handle_send_error(parent_pipeline_id, e);
        }
        self.focus_parent_pipeline(parent_pipeline_id);
    }

    fn handle_focus_msg(&mut self, pipeline_id: PipelineId) {
        self.focus_pipeline_id = Some(pipeline_id);

        // Focus parent iframes recursively
        self.focus_parent_pipeline(pipeline_id);
    }

    fn handle_remove_iframe_msg(&mut self, browsing_context_id: BrowsingContextId) -> Vec<PipelineId> {
        let result = self.all_descendant_browsing_contexts_iter(browsing_context_id)
            .flat_map(|browsing_context| browsing_context.next.iter().chain(browsing_context.prev.iter())
                      .filter_map(|entry| entry.pipeline_id)
                      .chain(once(browsing_context.pipeline_id)))
            .collect();
        self.close_browsing_context(browsing_context_id, ExitPipelineMode::Normal);
        result
    }

    fn handle_set_visible_msg(&mut self, pipeline_id: PipelineId, visible: bool) {
        let browsing_context_id = match self.pipelines.get(&pipeline_id) {
            Some(pipeline) => pipeline.browsing_context_id,
            None => return warn!("No browsing context associated with pipeline {:?}", pipeline_id),
        };

        let child_pipeline_ids: Vec<PipelineId> = self.all_descendant_browsing_contexts_iter(browsing_context_id)
            .flat_map(|browsing_context| browsing_context.prev.iter().chain(browsing_context.next.iter())
                      .filter_map(|entry| entry.pipeline_id)
                      .chain(once(browsing_context.pipeline_id)))
            .collect();

        for id in child_pipeline_ids {
            if let Some(pipeline) = self.pipelines.get_mut(&id) {
                pipeline.change_visibility(visible);
            }
        }
    }

    fn handle_visibility_change_complete(&mut self, pipeline_id: PipelineId, visibility: bool) {
        let (browsing_context_id, parent_pipeline_info) = match self.pipelines.get(&pipeline_id) {
            None => return warn!("Visibity change for closed pipeline {:?}.", pipeline_id),
            Some(pipeline) => (pipeline.browsing_context_id, pipeline.parent_info),
        };
        if let Some((parent_pipeline_id, _)) = parent_pipeline_info {
            let visibility_msg = ConstellationControlMsg::NotifyVisibilityChange(parent_pipeline_id,
                                                                                 browsing_context_id,
                                                                                 visibility);
            let result = match self.pipelines.get(&parent_pipeline_id) {
                None => return warn!("Parent pipeline {:?} closed", parent_pipeline_id),
                Some(parent_pipeline) => parent_pipeline.event_loop.send(visibility_msg),
            };

            if let Err(e) = result {
                self.handle_send_error(parent_pipeline_id, e);
            }
        }
    }

    fn handle_create_canvas_paint_thread_msg(
            &mut self,
            size: &Size2D<i32>,
            response_sender: IpcSender<IpcSender<CanvasMsg>>) {
        let webrender_api = self.webrender_api_sender.clone();
        let sender = CanvasPaintThread::start(*size, webrender_api,
                                              opts::get().enable_canvas_antialiasing);
        if let Err(e) = response_sender.send(sender) {
            warn!("Create canvas paint thread response failed ({})", e);
        }
    }

    fn handle_webdriver_msg(&mut self, msg: WebDriverCommandMsg) {
        // Find the script channel for the given parent pipeline,
        // and pass the event to that script thread.
        match msg {
            WebDriverCommandMsg::GetWindowSize(_, reply) => {
               let _ = reply.send(self.window_size);
            },
            WebDriverCommandMsg::SetWindowSize(top_level_browsing_context_id, size, reply) => {
                self.webdriver.resize_channel = Some(reply);
                self.embedder_proxy.send(EmbedderMsg::ResizeTo(top_level_browsing_context_id, size));
            },
            WebDriverCommandMsg::LoadUrl(top_level_browsing_context_id, load_data, reply) => {
                self.load_url_for_webdriver(top_level_browsing_context_id, load_data, reply, false);
            },
            WebDriverCommandMsg::Refresh(top_level_browsing_context_id, reply) => {
                let browsing_context_id = BrowsingContextId::from(top_level_browsing_context_id);
                let load_data = match self.browsing_contexts.get(&browsing_context_id) {
                    Some(browsing_context) => browsing_context.load_data.clone(),
                    None => return warn!("Browsing context {} Refresh after closure.", browsing_context_id),
                };
                self.load_url_for_webdriver(top_level_browsing_context_id, load_data, reply, true);
            }
            WebDriverCommandMsg::ScriptCommand(browsing_context_id, cmd) => {
                let pipeline_id = match self.browsing_contexts.get(&browsing_context_id) {
                    Some(browsing_context) => browsing_context.pipeline_id,
                    None => return warn!("Browsing context {} ScriptCommand after closure.", browsing_context_id),
                };
                let control_msg = ConstellationControlMsg::WebDriverScriptCommand(pipeline_id, cmd);
                let result = match self.pipelines.get(&pipeline_id) {
                    Some(pipeline) => pipeline.event_loop.send(control_msg),
                    None => return warn!("Pipeline {:?} ScriptCommand after closure.", pipeline_id),
                };
                if let Err(e) = result {
                    self.handle_send_error(pipeline_id, e);
                }
            },
            WebDriverCommandMsg::SendKeys(browsing_context_id, cmd) => {
                let pipeline_id = match self.browsing_contexts.get(&browsing_context_id) {
                    Some(browsing_context) => browsing_context.pipeline_id,
                    None => return warn!("Browsing context {} SendKeys after closure.", browsing_context_id),
                };
                let event_loop = match self.pipelines.get(&pipeline_id) {
                    Some(pipeline) => pipeline.event_loop.clone(),
                    None => return warn!("Pipeline {} SendKeys after closure.", pipeline_id),
                };
                for (key, mods, state) in cmd {
                    let event = CompositorEvent::KeyEvent(None, key, state, mods);
                    let control_msg = ConstellationControlMsg::SendEvent(pipeline_id, event);
                    if let Err(e) = event_loop.send(control_msg) {
                        return self.handle_send_error(pipeline_id, e);
                    }
                }
            },
            WebDriverCommandMsg::TakeScreenshot(_, reply) => {
                self.compositor_proxy.send(ToCompositorMsg::CreatePng(reply));
            },
        }
    }

    // https://html.spec.whatwg.org/multipage/#traverse-the-history
    fn traverse_to_entry(&mut self, entry: SessionHistoryEntry) {
        // Step 1.
        let browsing_context_id = entry.browsing_context_id;
        let pipeline_id = match entry.pipeline_id {
            Some(pipeline_id) => pipeline_id,
            None => {
                // If there is no pipeline, then the document for this
                // entry has been discarded, so we navigate to the entry
                // URL instead. When the document has activated, it will
                // traverse to the entry, but with the new pipeline id.
                debug!("Reloading document {} in browsing context {}.", entry.load_data.url, entry.browsing_context_id);
                // TODO: save the sandbox state so it can be restored here.
                let sandbox = IFrameSandboxState::IFrameUnsandboxed;
                let new_pipeline_id = PipelineId::new();
                let load_data = entry.load_data;
                let (top_level_id, parent_info, window_size, is_private) =
                    match self.browsing_contexts.get(&browsing_context_id)
                {
                    Some(browsing_context) => match self.pipelines.get(&browsing_context.pipeline_id) {
                        Some(pipeline) => (browsing_context.top_level_id,
                                           pipeline.parent_info,
                                           browsing_context.size,
                                           pipeline.is_private),
                        None => (browsing_context.top_level_id,
                                 None,
                                 browsing_context.size,
                                 false),
                    },
                    None => return warn!("no browsing context to traverse"),
                };
                self.new_pipeline(new_pipeline_id, browsing_context_id, top_level_id, parent_info,
                                  window_size, load_data.clone(), sandbox, is_private);
                self.add_pending_change(SessionHistoryChange {
                    top_level_browsing_context_id: top_level_id,
                    browsing_context_id: browsing_context_id,
                    new_pipeline_id: new_pipeline_id,
                    load_data: load_data,
                    replace_instant: Some(entry.instant),
                });
                return;
            }
        };

        // Check if the currently focused pipeline is the pipeline being replaced
        // (or a child of it). This has to be done here, before the current
        // frame tree is modified below.
        let update_focus_pipeline = self.focused_pipeline_is_descendant_of(entry.browsing_context_id);

        let (old_pipeline_id, replaced_pipeline_id, top_level_id) =
            match self.browsing_contexts.get_mut(&browsing_context_id)
        {
            Some(browsing_context) => {
                let old_pipeline_id = browsing_context.pipeline_id;
                let top_level_id = browsing_context.top_level_id;
                let mut curr_entry = browsing_context.current();

                if entry.instant > browsing_context.instant {
                    // We are traversing to the future.
                    while let Some(next) = browsing_context.next.pop() {
                        browsing_context.prev.push(curr_entry);
                        curr_entry = next;
                        if entry.instant <= curr_entry.instant { break; }
                    }
                } else if entry.instant < browsing_context.instant {
                    // We are traversing to the past.
                    while let Some(prev) = browsing_context.prev.pop() {
                        browsing_context.next.push(curr_entry);
                        curr_entry = prev;
                        if entry.instant >= curr_entry.instant { break; }
                    }
                }

                debug_assert_eq!(entry.instant, curr_entry.instant);

                let replaced_pipeline_id = curr_entry.pipeline_id;

                browsing_context.update_current(pipeline_id, entry);

                (old_pipeline_id, replaced_pipeline_id, top_level_id)
            },
            None => return warn!("no browsing context to traverse"),
        };

        let parent_info = self.pipelines.get(&old_pipeline_id)
            .and_then(|pipeline| pipeline.parent_info);

        // If the currently focused pipeline is the one being changed (or a child
        // of the pipeline being changed) then update the focus pipeline to be
        // the replacement.
        if update_focus_pipeline {
            self.focus_pipeline_id = Some(pipeline_id);
        }

        // If we replaced a pipeline, close it.
        if let Some(replaced_pipeline_id) = replaced_pipeline_id {
            if replaced_pipeline_id != pipeline_id {
                self.close_pipeline(replaced_pipeline_id, DiscardBrowsingContext::No, ExitPipelineMode::Normal);
            }
        }

        // Deactivate the old pipeline, and activate the new one.
        self.update_activity(old_pipeline_id);
        self.update_activity(pipeline_id);
        self.notify_history_changed(top_level_id);

        self.update_frame_tree_if_active(top_level_id);

        // Update the owning iframe to point to the new pipeline id.
        // This makes things like contentDocument work correctly.
        if let Some((parent_pipeline_id, frame_type)) = parent_info {
            let msg = ConstellationControlMsg::UpdatePipelineId(parent_pipeline_id,
                browsing_context_id, pipeline_id, UpdatePipelineIdReason::Traversal);
            let result = match self.pipelines.get(&parent_pipeline_id) {
                None => return warn!("Pipeline {:?} child traversed after closure.", parent_pipeline_id),
                Some(pipeline) => pipeline.event_loop.send(msg),
            };
            if let Err(e) = result {
                self.handle_send_error(parent_pipeline_id, e);
            }

            // If this is a mozbrowser iframe, send a mozbrowser location change event.
            // This is the result of a back/forward traversal.
            if frame_type == FrameType::MozBrowserIFrame {
                self.trigger_mozbrowserlocationchange(top_level_id);
            }
        }
    }

    fn notify_history_changed(&self, top_level_browsing_context_id: TopLevelBrowsingContextId) {
        // Send a flat projection of the history.
        // The final vector is a concatenation of the LoadData of the past entries,
        // the current entry and the future entries.
        // LoadData of inner frames are ignored and replaced with the LoadData of the parent.

        // Ignore LoadData of non-top-level browsing contexts.
        let keep_load_data_if_top_browsing_context = |entry: &SessionHistoryEntry| {
            match entry.pipeline_id {
                None => Some(entry.load_data.clone()),
                Some(pipeline_id) => {
                    match self.pipelines.get(&pipeline_id) {
                        None => Some(entry.load_data.clone()),
                        Some(pipeline) => match pipeline.parent_info {
                            None => Some(entry.load_data.clone()),
                            Some(_) => None,
                        }
                    }
                }
            }
        };

        // If LoadData was ignored, use the LoadData of the previous SessionHistoryEntry, which
        // is the LoadData of the parent browsing context.
        let resolve_load_data = |previous_load_data: &mut LoadData, load_data| {
            let load_data = match load_data {
                None => previous_load_data.clone(),
                Some(load_data) => load_data,
            };
            *previous_load_data = load_data.clone();
            Some(load_data)
        };

        let browsing_context_id = BrowsingContextId::from(top_level_browsing_context_id);
        let current_load_data = match self.browsing_contexts.get(&browsing_context_id) {
            Some(browsing_context) => browsing_context.load_data.clone(),
            None => return warn!("notify_history_changed error after top-level browsing context closed."),
        };

        let mut entries: Vec<LoadData> = self.joint_session_past(top_level_browsing_context_id)
            .map(&keep_load_data_if_top_browsing_context)
            .scan(current_load_data.clone(), &resolve_load_data)
            .collect();

        entries.reverse();

        let current_index = entries.len();

        entries.push(current_load_data.clone());

        entries.extend(self.joint_session_future(top_level_browsing_context_id)
                       .map(&keep_load_data_if_top_browsing_context)
                       .scan(current_load_data.clone(), &resolve_load_data));

        let msg = EmbedderMsg::HistoryChanged(top_level_browsing_context_id, entries, current_index);
        self.embedder_proxy.send(msg);
    }

    fn load_url_for_webdriver(&mut self,
                              top_level_browsing_context_id: TopLevelBrowsingContextId,
                              load_data: LoadData,
                              reply: IpcSender<webdriver_msg::LoadStatus>,
                              replace: bool)
    {
        let browsing_context_id = BrowsingContextId::from(top_level_browsing_context_id);
        let pipeline_id = match self.browsing_contexts.get(&browsing_context_id) {
            Some(browsing_context) => browsing_context.pipeline_id,
            None => return warn!("Webdriver load for closed browsing context {}.", browsing_context_id),
        };
        if let Some(new_pipeline_id) = self.load_url(top_level_browsing_context_id, pipeline_id, load_data, replace) {
            self.webdriver.load_channel = Some((new_pipeline_id, reply));
        }
    }

    fn change_session_history(&mut self, change: SessionHistoryChange) {
        debug!("Setting browsing context {} to be pipeline {}.", change.browsing_context_id, change.new_pipeline_id);

        // If the currently focused pipeline is the one being changed (or a child
        // of the pipeline being changed) then update the focus pipeline to be
        // the replacement.
        if self.focused_pipeline_is_descendant_of(change.browsing_context_id) {
            self.focus_pipeline_id = Some(change.new_pipeline_id);
        }

        let (evicted_id, new_context, navigated) = if let Some(instant) = change.replace_instant {
            debug!("Replacing pipeline in existing browsing context with timestamp {:?}.", instant);
            let entry = SessionHistoryEntry {
                browsing_context_id: change.browsing_context_id,
                pipeline_id: Some(change.new_pipeline_id),
                load_data: change.load_data.clone(),
                instant: instant,
            };
            self.traverse_to_entry(entry);
            (None, false, None)
        } else if let Some(browsing_context) = self.browsing_contexts.get_mut(&change.browsing_context_id) {
            debug!("Adding pipeline to existing browsing context.");
            let old_pipeline_id = browsing_context.pipeline_id;
            browsing_context.load(change.new_pipeline_id, change.load_data.clone());
            let evicted_id = browsing_context.prev.len()
                .checked_sub(PREFS.get("session-history.max-length").as_u64().unwrap_or(20) as usize)
                .and_then(|index| browsing_context.prev.get_mut(index))
                .and_then(|entry| entry.pipeline_id.take());
            (evicted_id, false, Some(old_pipeline_id))
        } else {
            debug!("Adding pipeline to new browsing context.");
            (None, true, None)
        };

        if let Some(evicted_id) = evicted_id {
            self.close_pipeline(evicted_id, DiscardBrowsingContext::No, ExitPipelineMode::Normal);
        }

        if new_context {
            self.new_browsing_context(change.browsing_context_id,
                                      change.top_level_browsing_context_id,
                                      change.new_pipeline_id,
                                      change.load_data);
            self.update_activity(change.new_pipeline_id);
            self.notify_history_changed(change.top_level_browsing_context_id);
        };

        if let Some(old_pipeline_id) = navigated {
            // Deactivate the old pipeline, and activate the new one.
            self.update_activity(old_pipeline_id);
            self.update_activity(change.new_pipeline_id);
            // Clear the joint session future
            self.clear_joint_session_future(change.top_level_browsing_context_id);
            self.notify_history_changed(change.top_level_browsing_context_id);
        }

        // If the navigation is for a top-level browsing context, inform mozbrowser
        if change.browsing_context_id == change.top_level_browsing_context_id {
            self.trigger_mozbrowserlocationchange(change.top_level_browsing_context_id);
        }

        self.update_frame_tree_if_active(change.top_level_browsing_context_id);
    }

    fn handle_activate_document_msg(&mut self, pipeline_id: PipelineId) {
        debug!("Document ready to activate {}", pipeline_id);

        // Notify the parent (if there is one).
        if let Some(pipeline) = self.pipelines.get(&pipeline_id) {
            if let Some((parent_pipeline_id, _)) = pipeline.parent_info {
                if let Some(parent_pipeline) = self.pipelines.get(&parent_pipeline_id) {
                    let msg = ConstellationControlMsg::UpdatePipelineId(parent_pipeline_id,
                        pipeline.browsing_context_id, pipeline_id, UpdatePipelineIdReason::Navigation);
                    let _ = parent_pipeline.event_loop.send(msg);
                }
            }
        }

        // Find the pending change whose new pipeline id is pipeline_id.
        let pending_index = self.pending_changes.iter().rposition(|change| {
            change.new_pipeline_id == pipeline_id
        });

        // If it is found, remove it from the pending changes, and make it
        // the active document of its frame.
        if let Some(pending_index) = pending_index {
            let change = self.pending_changes.swap_remove(pending_index);
            self.change_session_history(change);
        }
    }

    /// Called when the window is resized.
    fn handle_window_size_msg(&mut self,
                              top_level_browsing_context_id: TopLevelBrowsingContextId,
                              new_size: WindowSizeData,
                              size_type: WindowSizeType)
    {
        debug!("handle_window_size_msg: {:?}", new_size.initial_viewport.to_untyped());

        let browsing_context_id = BrowsingContextId::from(top_level_browsing_context_id);
        self.resize_browsing_context(new_size, size_type, browsing_context_id);

        if let Some(resize_channel) = self.webdriver.resize_channel.take() {
            let _ = resize_channel.send(new_size);
        }

        self.window_size = new_size;
    }

    /// Handle updating actual viewport / zoom due to @viewport rules
    fn handle_viewport_constrained_msg(&mut self,
                                       pipeline_id: PipelineId,
                                       constraints: ViewportConstraints) {
        self.compositor_proxy.send(ToCompositorMsg::ViewportConstrained(pipeline_id, constraints));
    }

    /// Checks the state of all script and layout pipelines to see if they are idle
    /// and compares the current layout state to what the compositor has. This is used
    /// to check if the output image is "stable" and can be written as a screenshot
    /// for reftests.
    /// Since this function is only used in reftests, we do not harden it against panic.
    fn handle_is_ready_to_save_image(&mut self, pipeline_states: HashMap<PipelineId, Epoch>) -> ReadyToSave {
        // Note that this function can panic, due to ipc-channel creation failure.
        // avoiding this panic would require a mechanism for dealing
        // with low-resource scenarios.
        //
        // If there is no focus browsing context yet, the initial page has
        // not loaded, so there is nothing to save yet.
        let top_level_browsing_context_id = self.focus_pipeline_id
            .and_then(|pipeline_id| self.pipelines.get(&pipeline_id))
            .map(|pipeline| pipeline.top_level_browsing_context_id);
        let top_level_browsing_context_id = match top_level_browsing_context_id {
            Some(id) => id,
            None => return ReadyToSave::NoTopLevelBrowsingContext,
        };

        // If there are pending loads, wait for those to complete.
        if !self.pending_changes.is_empty() {
            return ReadyToSave::PendingChanges;
        }

        let (state_sender, state_receiver) = ipc::channel().expect("Failed to create IPC channel!");
        let (epoch_sender, epoch_receiver) = ipc::channel().expect("Failed to create IPC channel!");

        // Step through the fully active browsing contexts, checking that the script
        // thread is idle, and that the current epoch of the layout thread
        // matches what the compositor has painted. If all these conditions
        // are met, then the output image should not change and a reftest
        // screenshot can safely be written.
        for browsing_context in self.fully_active_browsing_contexts_iter(top_level_browsing_context_id) {
            let pipeline_id = browsing_context.pipeline_id;
            debug!("Checking readiness of browsing context {}, pipeline {}.", browsing_context.id, pipeline_id);

            let pipeline = match self.pipelines.get(&pipeline_id) {
                None => {
                    warn!("Pipeline {} screenshot while closing.", pipeline_id);
                    continue;
                },
                Some(pipeline) => pipeline,
            };

            // Check to see if there are any webfonts still loading.
            //
            // If GetWebFontLoadState returns false, either there are no
            // webfonts loading, or there's a WebFontLoaded message waiting in
            // script_chan's message queue. Therefore, we need to check this
            // before we check whether the document is ready; otherwise,
            // there's a race condition where a webfont has finished loading,
            // but hasn't yet notified the document.
            let msg = LayoutControlMsg::GetWebFontLoadState(state_sender.clone());
            if let Err(e) = pipeline.layout_chan.send(msg) {
                warn!("Get web font failed ({})", e);
            }
            if state_receiver.recv().unwrap_or(true) {
                return ReadyToSave::WebFontNotLoaded;
            }

            // See if this pipeline has reached idle script state yet.
            match self.document_states.get(&browsing_context.pipeline_id) {
                Some(&DocumentState::Idle) => {}
                Some(&DocumentState::Pending) | None => {
                    return ReadyToSave::DocumentLoading;
                }
            }

            // Check the visible rectangle for this pipeline. If the constellation has received a
            // size for the pipeline, then its painting should be up to date. If the constellation
            // *hasn't* received a size, it could be that the layer was hidden by script before the
            // compositor discovered it, so we just don't check the layer.
            if let Some(size) = browsing_context.size {
                // If the rectangle for this pipeline is zero sized, it will
                // never be painted. In this case, don't query the layout
                // thread as it won't contribute to the final output image.
                if size == TypedSize2D::zero() {
                    continue;
                }

                // Get the epoch that the compositor has drawn for this pipeline.
                let compositor_epoch = pipeline_states.get(&browsing_context.pipeline_id);
                match compositor_epoch {
                    Some(compositor_epoch) => {
                        // Synchronously query the layout thread to see if the current
                        // epoch matches what the compositor has drawn. If they match
                        // (and script is idle) then this pipeline won't change again
                        // and can be considered stable.
                        let message = LayoutControlMsg::GetCurrentEpoch(epoch_sender.clone());
                        if let Err(e) = pipeline.layout_chan.send(message) {
                            warn!("Failed to send GetCurrentEpoch ({}).", e);
                        }
                        match epoch_receiver.recv() {
                            Err(e) => warn!("Failed to receive current epoch ({}).", e),
                            Ok(layout_thread_epoch) => if layout_thread_epoch != *compositor_epoch {
                                return ReadyToSave::EpochMismatch;
                            },
                        }
                    }
                    None => {
                        // The compositor doesn't know about this pipeline yet.
                        // Assume it hasn't rendered yet.
                        return ReadyToSave::PipelineUnknown;
                    }
                }
            }
        }

        // All script threads are idle and layout epochs match compositor, so output image!
        ReadyToSave::Ready
    }

    /// Get the current activity of a pipeline.
    fn get_activity(&self, pipeline_id: PipelineId) -> DocumentActivity {
        let mut ancestor_id = pipeline_id;
        loop {
            if let Some(ancestor) = self.pipelines.get(&ancestor_id) {
                if let Some(browsing_context) = self.browsing_contexts.get(&ancestor.browsing_context_id) {
                    if browsing_context.pipeline_id == ancestor_id {
                        if let Some((parent_id, FrameType::IFrame)) = ancestor.parent_info {
                            ancestor_id = parent_id;
                            continue;
                        } else {
                            return DocumentActivity::FullyActive;
                        }
                    }
                }
            }
            if pipeline_id == ancestor_id {
                return DocumentActivity::Inactive;
            } else {
                return DocumentActivity::Active;
            }
        }
    }

    /// Set the current activity of a pipeline.
    fn set_activity(&self, pipeline_id: PipelineId, activity: DocumentActivity) {
        debug!("Setting activity of {} to be {:?}.", pipeline_id, activity);
        if let Some(pipeline) = self.pipelines.get(&pipeline_id) {
            pipeline.set_activity(activity);
            let child_activity = if activity == DocumentActivity::Inactive {
                DocumentActivity::Active
            } else {
                activity
            };
            for child_id in &pipeline.children {
                if let Some(child) = self.browsing_contexts.get(child_id) {
                    self.set_activity(child.pipeline_id, child_activity);
                }
            }
        }
    }

    /// Update the current activity of a pipeline.
    fn update_activity(&self, pipeline_id: PipelineId) {
        self.set_activity(pipeline_id, self.get_activity(pipeline_id));
    }

    /// Handle updating the size of a browsing context.
    /// This notifies every pipeline in the context of the new size.
    fn resize_browsing_context(&mut self,
                               new_size: WindowSizeData,
                               size_type: WindowSizeType,
                               browsing_context_id: BrowsingContextId)
    {
        if let Some(browsing_context) = self.browsing_contexts.get_mut(&browsing_context_id) {
            browsing_context.size = Some(new_size.initial_viewport);
        }

        if let Some(browsing_context) = self.browsing_contexts.get(&browsing_context_id) {
            // Send Resize (or ResizeInactive) messages to each
            // pipeline in the frame tree.
            let pipeline_id = browsing_context.pipeline_id;
            let pipeline = match self.pipelines.get(&pipeline_id) {
                None => return warn!("Pipeline {:?} resized after closing.", pipeline_id),
                Some(pipeline) => pipeline,
            };
            let _ = pipeline.event_loop.send(ConstellationControlMsg::Resize(
                pipeline.id,
                new_size,
                size_type
            ));
            let pipelines = browsing_context.prev.iter().chain(browsing_context.next.iter())
                .filter_map(|entry| entry.pipeline_id)
                .filter_map(|pipeline_id| self.pipelines.get(&pipeline_id));
            for pipeline in pipelines {
                let _ = pipeline.event_loop.send(ConstellationControlMsg::ResizeInactive(
                    pipeline.id,
                    new_size
                ));
            }
        }

        // Send resize message to any pending pipelines that aren't loaded yet.
        for change in &self.pending_changes {
            let pipeline_id = change.new_pipeline_id;
            let pipeline = match self.pipelines.get(&pipeline_id) {
                None => { warn!("Pending pipeline {:?} is closed", pipeline_id); continue; }
                Some(pipeline) => pipeline,
            };
            if pipeline.browsing_context_id == browsing_context_id {
                let _ = pipeline.event_loop.send(ConstellationControlMsg::Resize(
                    pipeline.id,
                    new_size,
                    size_type
                ));
            }
        }
    }

    fn clear_joint_session_future(&mut self, top_level_browsing_context_id: TopLevelBrowsingContextId) {
        let browsing_context_ids: Vec<BrowsingContextId> =
            self.all_browsing_contexts_iter(top_level_browsing_context_id)
            .map(|browsing_context| browsing_context.id)
            .collect();
        for browsing_context_id in browsing_context_ids {
            let evicted = match self.browsing_contexts.get_mut(&browsing_context_id) {
                Some(browsing_context) => browsing_context.remove_forward_entries(),
                None => continue,
            };
            for entry in evicted {
                if let Some(pipeline_id) = entry.pipeline_id {
                    self.close_pipeline(pipeline_id, DiscardBrowsingContext::No, ExitPipelineMode::Normal);
                }
            }
        }
    }

    // Close a browsing context (and all children)
    fn close_browsing_context(&mut self, browsing_context_id: BrowsingContextId, exit_mode: ExitPipelineMode) {
        debug!("Closing browsing context {}.", browsing_context_id);

        self.close_browsing_context_children(browsing_context_id, DiscardBrowsingContext::Yes, exit_mode);

        let browsing_context = match self.browsing_contexts.remove(&browsing_context_id) {
            None => return warn!("Closing browsing context {:?} twice.", browsing_context_id),
            Some(browsing_context) => browsing_context,
        };

        if BrowsingContextId::from(browsing_context.top_level_id) == browsing_context_id {
            self.event_loops.remove(&browsing_context.top_level_id);
        }

        let parent_info = self.pipelines.get(&browsing_context.pipeline_id)
            .and_then(|pipeline| pipeline.parent_info);

        if let Some((parent_pipeline_id, _)) = parent_info {
            match self.pipelines.get_mut(&parent_pipeline_id) {
                None => return warn!("Pipeline {:?} child closed after parent.", parent_pipeline_id),
                Some(parent_pipeline) => parent_pipeline.remove_child(browsing_context_id),
            };
        }
        debug!("Closed browsing context {:?}.", browsing_context_id);
    }

    // Close the children of a browsing context
    fn close_browsing_context_children(&mut self,
                                       browsing_context_id: BrowsingContextId,
                                       dbc: DiscardBrowsingContext,
                                       exit_mode: ExitPipelineMode)
    {
        debug!("Closing browsing context children {}.", browsing_context_id);
        // Store information about the pipelines to be closed. Then close the
        // pipelines, before removing ourself from the browsing_contexts hash map. This
        // ordering is vital - so that if close_pipeline() ends up closing
        // any child browsing contexts, they can be removed from the parent browsing context correctly.
        let mut pipelines_to_close: Vec<PipelineId> = self.pending_changes.iter()
            .filter(|change| change.browsing_context_id == browsing_context_id)
            .map(|change| change.new_pipeline_id)
            .collect();

        if let Some(browsing_context) = self.browsing_contexts.get(&browsing_context_id) {
            pipelines_to_close.extend(browsing_context.next.iter().filter_map(|state| state.pipeline_id));
            pipelines_to_close.push(browsing_context.pipeline_id);
            pipelines_to_close.extend(browsing_context.prev.iter().filter_map(|state| state.pipeline_id));
        }

        for pipeline_id in pipelines_to_close {
            self.close_pipeline(pipeline_id, dbc, exit_mode);
        }

        debug!("Closed browsing context children {}.", browsing_context_id);
    }

    // Close all pipelines at and beneath a given browsing context
    fn close_pipeline(&mut self, pipeline_id: PipelineId, dbc: DiscardBrowsingContext, exit_mode: ExitPipelineMode) {
        debug!("Closing pipeline {:?}.", pipeline_id);
        // Store information about the browsing contexts to be closed. Then close the
        // browsing contexts, before removing ourself from the pipelines hash map. This
        // ordering is vital - so that if close_browsing_context() ends up closing
        // any child pipelines, they can be removed from the parent pipeline correctly.
        let browsing_contexts_to_close = {
            let mut browsing_contexts_to_close = vec!();

            if let Some(pipeline) = self.pipelines.get(&pipeline_id) {
                browsing_contexts_to_close.extend_from_slice(&pipeline.children);
            }

            browsing_contexts_to_close
        };

        // Remove any child browsing contexts
        for child_browsing_context in &browsing_contexts_to_close {
            self.close_browsing_context(*child_browsing_context, exit_mode);
        }

        // Note, we don't remove the pipeline now, we wait for the message to come back from
        // the pipeline.
        let pipeline = match self.pipelines.get(&pipeline_id) {
            Some(pipeline) => pipeline,
            None => return warn!("Closing pipeline {:?} twice.", pipeline_id),
        };

        // Remove this pipeline from pending changes if it hasn't loaded yet.
        let pending_index = self.pending_changes.iter().position(|change| {
            change.new_pipeline_id == pipeline_id
        });
        if let Some(pending_index) = pending_index {
            self.pending_changes.remove(pending_index);
        }

        // Inform script, compositor that this pipeline has exited.
        match exit_mode {
            ExitPipelineMode::Normal => pipeline.exit(dbc),
            ExitPipelineMode::Force => pipeline.force_exit(dbc),
        }
        debug!("Closed pipeline {:?}.", pipeline_id);
    }

    // Randomly close a pipeline -if --random-pipeline-closure-probability is set
    fn maybe_close_random_pipeline(&mut self) {
        match self.random_pipeline_closure {
            Some((ref mut rng, probability)) => if probability <= rng.gen::<f32>() { return },
            _ => return,
        };
        // In order to get repeatability, we sort the pipeline ids.
        let mut pipeline_ids: Vec<&PipelineId> = self.pipelines.keys().collect();
        pipeline_ids.sort();
        if let Some((ref mut rng, probability)) = self.random_pipeline_closure {
            if let Some(pipeline_id) = rng.choose(&*pipeline_ids) {
                if let Some(pipeline) = self.pipelines.get(pipeline_id) {
                    // Don't kill the mozbrowser pipeline
                    if PREFS.is_mozbrowser_enabled() && pipeline.parent_info.is_none() {
                        info!("Not closing mozbrowser pipeline {}.", pipeline_id);
                    } else if
                        self.pending_changes.iter().any(|change| change.new_pipeline_id == pipeline.id) &&
                        probability <= rng.gen::<f32>()
                    {
                        // We tend not to close pending pipelines, as that almost always
                        // results in pipelines being closed early in their lifecycle,
                        // and not stressing the constellation as much.
                        // https://github.com/servo/servo/issues/18852
                        info!("Not closing pending pipeline {}.", pipeline_id);
                    } else {
                        // Note that we deliberately do not do any of the tidying up
                        // associated with closing a pipeline. The constellation should cope!
                        warn!("Randomly closing pipeline {}.", pipeline_id);
                        pipeline.force_exit(DiscardBrowsingContext::No);
                    }
                }
            }
        }
    }

    // Convert a browsing context to a sendable form to pass to the compositor
    fn browsing_context_to_sendable(&self, browsing_context_id: BrowsingContextId) -> Option<SendableFrameTree> {
        self.browsing_contexts.get(&browsing_context_id).and_then(|browsing_context| {
            self.pipelines.get(&browsing_context.pipeline_id).map(|pipeline| {
                let mut frame_tree = SendableFrameTree {
                    pipeline: pipeline.to_sendable(),
                    size: browsing_context.size,
                    children: vec!(),
                };

                for child_browsing_context_id in &pipeline.children {
                    if let Some(child) = self.browsing_context_to_sendable(*child_browsing_context_id) {
                        frame_tree.children.push(child);
                    }
                }

                frame_tree
            })
        })
    }

    /// Re-send the frame tree to the compositor.
    fn update_frame_tree_if_active(&mut self, mut top_level_browsing_context_id: TopLevelBrowsingContextId) {
        // This might be a mozbrowser iframe, so we need to climb the parent hierarchy,
        // even though it's a top-level browsing context.
        // FIXME(paul): to remove once mozbrowser API is removed.
        let browsing_context_id = BrowsingContextId::from(top_level_browsing_context_id);
        let mut pipeline_id = match self.browsing_contexts.get(&browsing_context_id) {
            Some(browsing_context) => browsing_context.pipeline_id,
            None => return warn!("Sending frame tree for discarded browsing context {}.", browsing_context_id),
        };

        while let Some(pipeline) = self.pipelines.get(&pipeline_id) {
            match pipeline.parent_info {
                Some((parent_id, _)) => pipeline_id = parent_id,
                None => {
                    top_level_browsing_context_id = pipeline.top_level_browsing_context_id;
                    break;
                },
            }
        }

        // Only send the frame tree if it's the active one or if no frame tree
        // has been sent yet.
        if self.active_browser_id.is_none() || Some(top_level_browsing_context_id) == self.active_browser_id {
            self.send_frame_tree(top_level_browsing_context_id);
        }

    }

    /// Send the current frame tree to compositor
    fn send_frame_tree(&mut self, top_level_browsing_context_id: TopLevelBrowsingContextId) {
        self.active_browser_id = Some(top_level_browsing_context_id);
        let browsing_context_id = BrowsingContextId::from(top_level_browsing_context_id);

        // Note that this function can panic, due to ipc-channel creation failure.
        // avoiding this panic would require a mechanism for dealing
        // with low-resource scenarios.
        debug!("Sending frame tree for browsing context {}.", browsing_context_id);
        if let Some(frame_tree) = self.browsing_context_to_sendable(browsing_context_id) {
            self.compositor_proxy.send(ToCompositorMsg::SetFrameTree(frame_tree));
        }
    }

    // https://developer.mozilla.org/en-US/docs/Web/Events/mozbrowserlocationchange
    // Note that this is a no-op if the pipeline is not a mozbrowser iframe
    fn trigger_mozbrowserlocationchange(&self,
                                        top_level_browsing_context_id: TopLevelBrowsingContextId)
    {
        let browsing_context_id = BrowsingContextId::from(top_level_browsing_context_id);
        let pipeline_id = match self.browsing_contexts.get(&browsing_context_id) {
            Some(browsing_context) => browsing_context.pipeline_id,
            None => return warn!("mozbrowser location change on closed browsing context {}.", browsing_context_id),
        };
        let (url, parent_info) = match self.pipelines.get(&pipeline_id) {
            Some(pipeline) => (pipeline.url.clone(), pipeline.parent_info),
            None => return warn!("mozbrowser location change on closed pipeline {}.", pipeline_id),
        };
        let parent_id = match parent_info {
            Some((parent_id, FrameType::MozBrowserIFrame)) => parent_id,
            _ => return debug!("mozbrowser location change on a regular iframe {}", browsing_context_id),
        };
        let can_go_forward = !self.joint_session_future_is_empty(top_level_browsing_context_id);
        let can_go_back = !self.joint_session_past_is_empty(top_level_browsing_context_id);
        let event = MozBrowserEvent::LocationChange(url.to_string(), can_go_back, can_go_forward);
        match self.pipelines.get(&parent_id) {
            Some(parent) => parent.trigger_mozbrowser_event(Some(top_level_browsing_context_id), event),
            None => return warn!("mozbrowser location change on closed parent {}", parent_id),
        };
    }

    // https://developer.mozilla.org/en-US/docs/Web/Events/mozbrowsererror
    fn trigger_mozbrowsererror(&mut self,
                               top_level_browsing_context_id: TopLevelBrowsingContextId,
                               reason: String,
                               backtrace: Option<String>)
    {
        if !PREFS.is_mozbrowser_enabled() { return; }

        let mut report = String::new();
        for (thread_name, warning) in self.handled_warnings.drain(..) {
            report.push_str("\nWARNING: ");
            if let Some(thread_name) = thread_name {
                report.push_str("<");
                report.push_str(&*thread_name);
                report.push_str(">: ");
            }
            report.push_str(&*warning);
        }
        report.push_str("\nERROR: ");
        report.push_str(&*reason);
        if let Some(backtrace) = backtrace {
            report.push_str("\n\n");
            report.push_str(&*backtrace);
        }

        let event = MozBrowserEvent::Error(MozBrowserErrorType::Fatal, reason, report);
        let browsing_context_id = BrowsingContextId::from(top_level_browsing_context_id);
        let pipeline_id = match self.browsing_contexts.get(&browsing_context_id) {
            Some(browsing_context) => browsing_context.pipeline_id,
            None => return warn!("Mozbrowser error after top-level browsing context closed."),
        };
        let parent_id = match self.pipelines.get(&pipeline_id) {
            Some(pipeline) => match pipeline.parent_info {
                Some((parent_id, FrameType::MozBrowserIFrame)) => parent_id,
                _ => return pipeline.trigger_mozbrowser_event(None, event),
            },
            None => return warn!("Mozbrowser error on a closed pipeline {}", pipeline_id),
        };
        match self.pipelines.get(&parent_id) {
            None => warn!("Mozbrowser error after parent pipeline {} closed.", parent_id),
            Some(parent) => parent.trigger_mozbrowser_event(Some(top_level_browsing_context_id), event),
        };
    }

    fn focused_pipeline_is_descendant_of(&self, browsing_context_id: BrowsingContextId) -> bool {
        self.focus_pipeline_id.map_or(false, |pipeline_id| {
            self.fully_active_descendant_browsing_contexts_iter(browsing_context_id)
                .any(|browsing_context| browsing_context.pipeline_id == pipeline_id)
        })
    }
}
