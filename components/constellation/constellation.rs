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
//! and converting any `error!` or `panic!` into a crash report.
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
use browsingcontext::{AllBrowsingContextsIterator, BrowsingContext, FullyActiveBrowsingContextsIterator};
use canvas::canvas_paint_thread::CanvasPaintThread;
use canvas::webgl_thread::WebGLThreads;
use canvas_traits::canvas::CanvasId;
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
use layout_traits::LayoutThreadFactory;
use log::{Log, Level, LevelFilter, Metadata, Record};
use msg::constellation_msg::{BrowsingContextId, PipelineId, HistoryStateId, TopLevelBrowsingContextId};
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
use script_traits::{SWManagerMsg, ScopeThings, UpdatePipelineIdReason, WebDriverCommandMsg};
use script_traits::{WindowSizeData, WindowSizeType};
use serde::{Deserialize, Serialize};
use servo_config::opts;
use servo_config::prefs::PREFS;
use servo_rand::{Rng, SeedableRng, ServoRng, random};
use servo_remutex::ReentrantMutex;
use servo_url::{Host, ImmutableOrigin, ServoUrl};
use session_history::{JointSessionHistory, NeedsToReload, SessionHistoryChange, SessionHistoryDiff};
use std::borrow::ToOwned;
use std::collections::{HashMap, VecDeque};
use std::marker::PhantomData;
use std::mem::replace;
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

    joint_session_histories: HashMap<TopLevelBrowsingContextId, JointSessionHistory>,

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
    webgl_threads: Option<WebGLThreads>,

    /// A channel through which messages can be sent to the webvr thread.
    webvr_chan: Option<IpcSender<WebVRMsg>>,

    /// A channel through which messages can be sent to the canvas paint thread.
    canvas_chan: IpcSender<CanvasMsg>,
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
    pub webgl_threads: Option<WebGLThreads>,

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
    pub fn filter(&self) -> LevelFilter {
        LevelFilter::Warn
    }
}

impl Log for FromScriptLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Warn
    }

    fn log(&self, record: &Record) {
        if let Some(entry) = log_entry(record) {
            debug!("Sending log entry {:?}.", entry);
            let thread_name = thread::current().name().map(ToOwned::to_owned);
            let msg = FromScriptMsg::LogEntry(thread_name, entry);
            let chan = self.script_to_constellation_chan.lock().unwrap_or_else(|err| err.into_inner());
            let _ = chan.send(msg);
        }
    }

    fn flush(&self) {}
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
    pub fn filter(&self) -> LevelFilter {
        LevelFilter::Warn
    }
}

impl Log for FromCompositorLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Warn
    }

    fn log(&self, record: &Record) {
        if let Some(entry) = log_entry(record) {
            debug!("Sending log entry {:?}.", entry);
            let top_level_id = TopLevelBrowsingContextId::installed();
            let thread_name = thread::current().name().map(ToOwned::to_owned);
            let msg = FromCompositorMsg::LogEntry(top_level_id, thread_name, entry);
            let chan = self.constellation_chan.lock().unwrap_or_else(|err| err.into_inner());
            let _ = chan.send(msg);
        }
    }

    fn flush(&self) {}
}

/// Rust uses `Record` for storing logging, but servo converts that to
/// a `LogEntry`. We do this so that we can record panics as well as log
/// messages, and because `Record` does not implement serde (de)serialization,
/// so cannot be used over an IPC channel.
fn log_entry(record: &Record) -> Option<LogEntry> {
    match record.level() {
        Level::Error if thread::panicking() => Some(LogEntry::Panic(
            format!("{}", record.args()),
            format!("{:?}", Backtrace::new())
        )),
        Level::Error => Some(LogEntry::Error(
            format!("{}", record.args())
        )),
        Level::Warn => Some(LogEntry::Warn(
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
                joint_session_histories: HashMap::new(),
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
                canvas_chan: CanvasPaintThread::start(),
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
                    parent_info: Option<PipelineId>,
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
                        .and_then(|pipeline_id| self.pipelines.get(&pipeline_id)) {
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
            .and_then(|parent_pipeline_id| self.pipelines.get(&parent_pipeline_id))
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
            webgl_chan: self.webgl_threads.as_ref().map(|threads| threads.pipeline()),
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

    /// Create a new browsing context and update the internal bookkeeping.
    fn new_browsing_context(&mut self,
                 browsing_context_id: BrowsingContextId,
                 top_level_id: TopLevelBrowsingContextId,
                 pipeline_id: PipelineId) {
        debug!("Creating new browsing context {}", browsing_context_id);
        let browsing_context = BrowsingContext::new(browsing_context_id, top_level_id, pipeline_id);
        self.browsing_contexts.insert(browsing_context_id, browsing_context);

        // If a child browsing_context, add it to the parent pipeline.
        let parent_info = self.pipelines.get(&pipeline_id)
            .and_then(|pipeline| pipeline.parent_info);
        if let Some(parent_id) = parent_info {
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
            FromScriptMsg::DiscardDocument => {
                self.handle_discard_document(source_top_ctx_id, source_pipeline_id);
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
            // Handle a push history state request.
            FromScriptMsg::PushHistoryState(history_state_id, url) => {
                debug!("constellation got push history state message from script");
                self.handle_push_history_state_msg(source_pipeline_id, history_state_id, url);
            }
            FromScriptMsg::ReplaceHistoryState(history_state_id, url) => {
                debug!("constellation got replace history state message from script");
                self.handle_replace_history_state_msg(source_pipeline_id, history_state_id, url);
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
            FromScriptMsg::Focus => {
                debug!("constellation got focus message");
                self.handle_focus_msg(source_pipeline_id);
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

            FromScriptMsg::MoveTo(point) => {
                self.embedder_proxy.send(EmbedderMsg::MoveTo(source_top_ctx_id, point));
            }

            FromScriptMsg::ResizeTo(size) => {
                self.embedder_proxy.send(EmbedderMsg::ResizeTo(source_top_ctx_id, size));
            }

            FromScriptMsg::GetClientWindow(send) => {
                self.compositor_proxy.send(ToCompositorMsg::GetClientWindow(send));
            }
            FromScriptMsg::GetScreenSize(send) => {
                self.compositor_proxy.send(ToCompositorMsg::GetScreenSize(send));
            }
            FromScriptMsg::GetScreenAvailSize(send) => {
                self.compositor_proxy.send(ToCompositorMsg::GetScreenAvailSize(send));
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
            FromScriptMsg::ShowIME(kind) => {
                debug!("constellation got ShowIME message");
                self.embedder_proxy.send(EmbedderMsg::ShowIME(source_top_ctx_id, kind));
            }
            FromScriptMsg::HideIME => {
                debug!("constellation got HideIME message");
                self.embedder_proxy.send(EmbedderMsg::HideIME(source_top_ctx_id));
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

        if let Some(webgl_threads) = self.webgl_threads.as_ref() {
            debug!("Exiting WebGL thread.");
            if let Err(e) = webgl_threads.exit() {
                warn!("Exit WebGL Thread failed ({})", e);
            }
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

        self.embedder_proxy.send(EmbedderMsg::Panic(top_level_browsing_context_id, reason, backtrace));

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
            replace: None,
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
        self.joint_session_histories.insert(top_level_browsing_context_id, JointSessionHistory::new());
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
            replace: None,
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
                Some(parent_id) => (pipeline.browsing_context_id, parent_id),
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

        let replace = if load_info.info.replace {
            self.browsing_contexts.get(&load_info.info.browsing_context_id)
                .map(|browsing_context| NeedsToReload::No(browsing_context.pipeline_id))
        } else {
            None
        };

        // Create the new pipeline, attached to the parent and push to pending changes
        self.new_pipeline(load_info.info.new_pipeline_id,
                          load_info.info.browsing_context_id,
                          load_info.info.top_level_browsing_context_id,
                          Some(load_info.info.parent_pipeline_id),
                          window_size,
                          load_data.clone(),
                          load_info.sandbox,
                          is_private);
        self.add_pending_change(SessionHistoryChange {
            top_level_browsing_context_id: load_info.info.top_level_browsing_context_id,
            browsing_context_id: load_info.info.browsing_context_id,
            new_pipeline_id: load_info.info.new_pipeline_id,
            replace,
        });
    }

    fn handle_script_new_iframe(&mut self,
                                load_info: IFrameLoadInfo,
                                layout_sender: IpcSender<LayoutControlMsg>) {
        let IFrameLoadInfo {
            parent_pipeline_id,
            new_pipeline_id,
            browsing_context_id,
            top_level_browsing_context_id,
            is_private,
            ..
        } = load_info;

        let url = ServoUrl::parse("about:blank").expect("infallible");

        // TODO: Referrer?
        let load_data = LoadData::new(url.clone(), Some(parent_pipeline_id), None, None);

        let pipeline = {
            let parent_pipeline = match self.pipelines.get(&parent_pipeline_id) {
                Some(parent_pipeline) => parent_pipeline,
                None => return warn!("Script loaded url in closed iframe {}.", parent_pipeline_id),
            };

            let script_sender = parent_pipeline.event_loop.clone();

            Pipeline::new(new_pipeline_id,
                          browsing_context_id,
                          top_level_browsing_context_id,
                          Some(parent_pipeline_id),
                          script_sender,
                          layout_sender,
                          self.compositor_proxy.clone(),
                          is_private || parent_pipeline.is_private,
                          url,
                          parent_pipeline.visible,
                          load_data)
        };

        assert!(!self.pipelines.contains_key(&new_pipeline_id));
        self.pipelines.insert(new_pipeline_id, pipeline);

        self.add_pending_change(SessionHistoryChange {
            top_level_browsing_context_id: top_level_browsing_context_id,
            browsing_context_id: browsing_context_id,
            new_pipeline_id: new_pipeline_id,
            replace: None,
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
                    _message: String,
                    sender: IpcSender<bool>) {
        // FIXME: forward alert event to embedder
        // https://github.com/servo/servo/issues/19992
        let result = sender.send(true);
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
            Some(parent_pipeline_id) => {
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
                let (top_level_id, window_size, pipeline_id) = match self.browsing_contexts.get(&browsing_context_id) {
                    Some(context) => (context.top_level_id, context.size, context.pipeline_id),
                    None => {
                        warn!("Browsing context {} loaded after closure.", browsing_context_id);
                        return None;
                    }
                };

                let replace = if replace { Some(NeedsToReload::No(pipeline_id)) } else { None };

                let new_pipeline_id = PipelineId::new();
                let sandbox = IFrameSandboxState::IFrameUnsandboxed;
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
                    replace,
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
        let mut browsing_context_changes = HashMap::<BrowsingContextId, NeedsToReload>::new();
        let mut pipeline_changes = HashMap::<PipelineId, (Option<HistoryStateId>, ServoUrl)>::new();
        let mut url_to_load = HashMap::<PipelineId, ServoUrl>::new();
        {
            let session_history = self.get_joint_session_history(top_level_browsing_context_id);
            match direction {
                TraversalDirection::Forward(forward) => {
                    let future_length = session_history.future.len();

                    if future_length < forward {
                        return warn!("Cannot traverse that far into the future.");
                    }

                    for diff in session_history.future.drain(future_length - forward..).rev() {
                        match diff {
                            SessionHistoryDiff::BrowsingContextDiff { browsing_context_id, ref new_reloader, .. } => {
                                browsing_context_changes.insert(browsing_context_id, new_reloader.clone());
                            }
                            SessionHistoryDiff::PipelineDiff {
                                ref pipeline_reloader,
                                new_history_state_id,
                                ref new_url,
                                ..
                            } => match *pipeline_reloader {
                                NeedsToReload::No(pipeline_id) => {
                                    pipeline_changes.insert(pipeline_id, (Some(new_history_state_id), new_url.clone()));
                                }
                                NeedsToReload::Yes(pipeline_id, ..) => {
                                    url_to_load.insert(pipeline_id, new_url.clone());
                                }
                            },
                        }
                        session_history.past.push(diff);
                    }
                },
                TraversalDirection::Back(back) => {
                    let past_length = session_history.past.len();

                    if past_length < back {
                        return warn!("Cannot traverse that far into the past.");
                    }

                    for diff in session_history.past.drain(past_length - back..).rev() {
                        match diff {
                            SessionHistoryDiff::BrowsingContextDiff { browsing_context_id, ref old_reloader, .. } => {
                                browsing_context_changes.insert(browsing_context_id, old_reloader.clone());
                            },
                            SessionHistoryDiff::PipelineDiff {
                                ref pipeline_reloader,
                                old_history_state_id,
                                ref old_url,
                                ..
                            } => match *pipeline_reloader {
                                NeedsToReload::No(pipeline_id) => {
                                    pipeline_changes.insert(pipeline_id, (old_history_state_id, old_url.clone()));
                                }
                                NeedsToReload::Yes(pipeline_id, ..) => {
                                    url_to_load.insert(pipeline_id, old_url.clone());
                                }
                            },
                        }
                        session_history.future.push(diff);
                    }
                }
            }
        }

        for (browsing_context_id, mut pipeline_reloader) in browsing_context_changes.drain() {
            if let NeedsToReload::Yes(pipeline_id, ref mut load_data) = pipeline_reloader {
                if let Some(url) = url_to_load.get(&pipeline_id) {
                    load_data.url = url.clone();
                }
            }
            self.update_browsing_context(browsing_context_id, pipeline_reloader);
        }

        for (pipeline_id, (history_state_id, url)) in pipeline_changes.drain() {
            self.update_pipeline(pipeline_id, history_state_id, url);
        }

        self.notify_history_changed(top_level_browsing_context_id);

        self.trim_history(top_level_browsing_context_id);
        self.update_frame_tree_if_active(top_level_browsing_context_id);
    }

    fn update_browsing_context(&mut self, browsing_context_id: BrowsingContextId, new_reloader: NeedsToReload) {
        let new_pipeline_id = match new_reloader {
            NeedsToReload::No(pipeline_id) => pipeline_id,
            NeedsToReload::Yes(pipeline_id, load_data) => {
                debug!("Reloading document {} in browsing context {}.", pipeline_id, browsing_context_id);

                // TODO: Save the sandbox state so it can be restored here.
                let sandbox = IFrameSandboxState::IFrameUnsandboxed;
                let new_pipeline_id = PipelineId::new();
                let (top_level_id, parent_info, window_size, is_private) =
                    match self.browsing_contexts.get(&browsing_context_id)
                {
                    Some(browsing_context) => match self.pipelines.get(&browsing_context.pipeline_id) {
                        Some(pipeline) => (
                            browsing_context.top_level_id,
                            pipeline.parent_info,
                            browsing_context.size,
                            pipeline.is_private
                        ),
                        None => (
                            browsing_context.top_level_id,
                            None,
                            browsing_context.size,
                            false
                        ),
                    },
                    None => return warn!("No browsing context to traverse!"),
                };
                self.new_pipeline(new_pipeline_id, browsing_context_id, top_level_id, parent_info,
                    window_size, load_data.clone(), sandbox, is_private);
                self.add_pending_change(SessionHistoryChange {
                    top_level_browsing_context_id: top_level_id,
                    browsing_context_id: browsing_context_id,
                    new_pipeline_id: new_pipeline_id,
                    replace: Some(NeedsToReload::Yes(pipeline_id, load_data.clone()))
                });
                return;
            },
        };

        let old_pipeline_id = match self.browsing_contexts.get_mut(&browsing_context_id) {
            Some(browsing_context) => {
                let old_pipeline_id = browsing_context.pipeline_id;
                browsing_context.update_current_entry(new_pipeline_id);
                old_pipeline_id
            },
            None => {
                return warn!("Browsing context {} was closed during traversal", browsing_context_id);
            }
        };

        let parent_info = self.pipelines.get(&old_pipeline_id).and_then(|pipeline| pipeline.parent_info);

        self.update_activity(old_pipeline_id);
        self.update_activity(new_pipeline_id);

        if let Some(parent_pipeline_id) = parent_info {
            let msg = ConstellationControlMsg::UpdatePipelineId(parent_pipeline_id, browsing_context_id,
                new_pipeline_id, UpdatePipelineIdReason::Traversal);
            let result = match self.pipelines.get(&parent_pipeline_id) {
                None => return warn!("Pipeline {} child traversed after closure", parent_pipeline_id),
                Some(pipeline) => pipeline.event_loop.send(msg),
            };
            if let Err(e) = result {
                self.handle_send_error(parent_pipeline_id, e);
            }
        }
    }

    fn update_pipeline(&mut self, pipeline_id: PipelineId, history_state_id: Option<HistoryStateId>, url: ServoUrl) {
        let result = match self.pipelines.get_mut(&pipeline_id) {
            None => return warn!("Pipeline {} history state updated after closure", pipeline_id),
            Some(pipeline) => {
                let msg = ConstellationControlMsg::UpdateHistoryState(pipeline_id, history_state_id, url.clone());
                pipeline.history_state_id = history_state_id;
                pipeline.url = url;
                pipeline.event_loop.send(msg)
            },
        };
        if let Err(e) = result {
            self.handle_send_error(pipeline_id, e);
        }
    }

    fn handle_joint_session_history_length(&self,
                                           top_level_browsing_context_id: TopLevelBrowsingContextId,
                                           sender: IpcSender<u32>)
    {
        let length = self.joint_session_histories.get(&top_level_browsing_context_id)
            .map(JointSessionHistory::history_length)
            .unwrap_or(1);
        let _ = sender.send(length as u32);
    }

    fn handle_push_history_state_msg(
        &mut self,
        pipeline_id: PipelineId,
        history_state_id: HistoryStateId,
        url: ServoUrl)
    {
        let (top_level_browsing_context_id, old_state_id, old_url) = match self.pipelines.get_mut(&pipeline_id) {
            Some(pipeline) => {
                let old_history_state_id = pipeline.history_state_id;
                let old_url = replace(&mut pipeline.url, url.clone());
                pipeline.history_state_id = Some(history_state_id);
                pipeline.history_states.insert(history_state_id);
                (pipeline.top_level_browsing_context_id, old_history_state_id, old_url)
            }
            None => return warn!("Push history state {} for closed pipeline {}", history_state_id, pipeline_id),
        };

        let session_history = self.get_joint_session_history(top_level_browsing_context_id);
        let diff = SessionHistoryDiff::PipelineDiff {
            pipeline_reloader: NeedsToReload::No(pipeline_id),
            new_history_state_id: history_state_id,
            new_url: url,
            old_history_state_id: old_state_id,
            old_url: old_url,
        };
        session_history.push_diff(diff);
    }

    fn handle_replace_history_state_msg(
        &mut self,
        pipeline_id: PipelineId,
        history_state_id: HistoryStateId,
        url: ServoUrl)
    {
        let top_level_browsing_context_id = match self.pipelines.get_mut(&pipeline_id) {
            Some(pipeline) => {
                pipeline.history_state_id = Some(history_state_id);
                pipeline.url = url.clone();
                pipeline.top_level_browsing_context_id
            }
            None => return warn!("Replace history state {} for closed pipeline {}", history_state_id, pipeline_id),
        };

        let session_history = self.get_joint_session_history(top_level_browsing_context_id);
        session_history.replace_history_state(pipeline_id, history_state_id, url);
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
        let parent_pipeline_id = match parent_info {
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
            .flat_map(|browsing_context| browsing_context.pipelines.iter().cloned()).collect();
        self.close_browsing_context(browsing_context_id, ExitPipelineMode::Normal);
        result
    }

    fn handle_set_visible_msg(&mut self, pipeline_id: PipelineId, visible: bool) {
        let browsing_context_id = match self.pipelines.get(&pipeline_id) {
            Some(pipeline) => pipeline.browsing_context_id,
            None => return warn!("No browsing context associated with pipeline {:?}", pipeline_id),
        };

        let child_pipeline_ids: Vec<PipelineId> = self.all_descendant_browsing_contexts_iter(browsing_context_id)
            .flat_map(|browsing_context| browsing_context.pipelines.iter().cloned()).collect();

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
        if let Some(parent_pipeline_id) = parent_pipeline_info {
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
            response_sender: IpcSender<(IpcSender<CanvasMsg>, CanvasId)>) {
        let webrender_api = self.webrender_api_sender.clone();
        let sender = self.canvas_chan.clone();
        let (canvas_id_sender, canvas_id_receiver) = ipc::channel::<CanvasId>().expect("ipc channel failure");

        if let Err(e) = sender.send(
            CanvasMsg::Create(
                canvas_id_sender,
                *size,
                webrender_api,
                opts::get().enable_canvas_antialiasing
            )
        ) {
            return warn!("Create canvas paint thread failed ({})", e);
        }
        let canvas_id = match canvas_id_receiver.recv() {
            Ok(canvas_id) => canvas_id,
            Err(e) => return warn!("Create canvas paint thread id response failed ({})", e),
        };
        if let Err(e) = response_sender.send((sender, canvas_id.clone())) {
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
                    Some(browsing_context) => match self.pipelines.get(&browsing_context.pipeline_id) {
                        Some(pipeline) => pipeline.load_data.clone(),
                        None => return warn!("Pipeline {} refresh after closure.", browsing_context.pipeline_id),
                    },
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

    fn notify_history_changed(&self, top_level_browsing_context_id: TopLevelBrowsingContextId) {
        // Send a flat projection of the history.
        // The final vector is a concatenation of the LoadData of the past entries,
        // the current entry and the future entries.
        // LoadData of inner frames are ignored and replaced with the LoadData of the parent.

        let session_history = match self.joint_session_histories.get(&top_level_browsing_context_id) {
            Some(session_history) => session_history,
            None => return warn!("Session history does not exist for {}", top_level_browsing_context_id),
        };

        let browsing_context_id = BrowsingContextId::from(top_level_browsing_context_id);
        let browsing_context = match self.browsing_contexts.get(&browsing_context_id) {
            Some(browsing_context) => browsing_context,
            None => return warn!("notify_history_changed error after top-level browsing context closed."),
        };

        let current_load_data = match self.pipelines.get(&browsing_context.pipeline_id) {
            Some(pipeline) => pipeline.load_data.clone(),
            None => return warn!("Pipeline {} refresh after closure.", browsing_context.pipeline_id),
        };

        // If LoadData was ignored, use the LoadData of the previous SessionHistoryEntry, which
        // is the LoadData of the parent browsing context.
        let resolve_load_data_future = |previous_load_data: &mut LoadData, diff: &SessionHistoryDiff| {
            match *diff {
                SessionHistoryDiff::BrowsingContextDiff { browsing_context_id, ref new_reloader, .. } => {
                    if browsing_context_id == top_level_browsing_context_id {
                        let load_data = match *new_reloader {
                            NeedsToReload::No(pipeline_id) => match self.pipelines.get(&pipeline_id) {
                                Some(pipeline) => pipeline.load_data.clone(),
                                None => previous_load_data.clone(),
                            },
                            NeedsToReload::Yes(_, ref load_data) => load_data.clone(),
                        };
                        *previous_load_data = load_data.clone();
                        Some(load_data)
                    } else {
                        Some(previous_load_data.clone())
                    }
                },
                SessionHistoryDiff::PipelineDiff { .. } => Some(previous_load_data.clone()),
            }
        };

        let resolve_load_data_past = |previous_load_data: &mut LoadData, diff: &SessionHistoryDiff| {
            match *diff {
                SessionHistoryDiff::BrowsingContextDiff { browsing_context_id, ref old_reloader, .. } => {
                    if browsing_context_id == top_level_browsing_context_id {
                        let load_data = match *old_reloader {
                            NeedsToReload::No(pipeline_id) => match self.pipelines.get(&pipeline_id) {
                                Some(pipeline) => pipeline.load_data.clone(),
                                None => previous_load_data.clone(),
                            },
                            NeedsToReload::Yes(_, ref load_data) => load_data.clone(),
                        };
                        *previous_load_data = load_data.clone();
                        Some(load_data)
                    } else {
                        Some(previous_load_data.clone())
                    }
                },
                SessionHistoryDiff::PipelineDiff { .. } => Some(previous_load_data.clone()),
            }
        };

        let mut entries: Vec<LoadData> = session_history.past.iter().rev()
            .scan(current_load_data.clone(), &resolve_load_data_past).collect();

        entries.reverse();

        let current_index = entries.len();

        entries.push(current_load_data.clone());

        entries.extend(session_history.future.iter().rev()
            .scan(current_load_data.clone(), &resolve_load_data_future));

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


        let (old_pipeline_id, top_level_id) = match self.browsing_contexts.get_mut(&change.browsing_context_id) {
            Some(browsing_context) => {
                debug!("Adding pipeline to existing browsing context.");
                let old_pipeline_id = browsing_context.pipeline_id;
                browsing_context.pipelines.insert(change.new_pipeline_id);
                browsing_context.update_current_entry(change.new_pipeline_id);
                (Some(old_pipeline_id), Some(browsing_context.top_level_id))
            },
            None => {
                debug!("Adding pipeline to new browsing context.");
                (None, None)
            }
        };

        match old_pipeline_id {
            None => {
                self.new_browsing_context(change.browsing_context_id,
                    change.top_level_browsing_context_id,
                    change.new_pipeline_id);
                self.update_activity(change.new_pipeline_id);
                self.notify_history_changed(change.top_level_browsing_context_id);
            },
            Some(old_pipeline_id) => {
                // https://html.spec.whatwg.org/multipage/#unload-a-document
                self.unload_document(old_pipeline_id);
                // Deactivate the old pipeline, and activate the new one.
                let (pipelines_to_close, states_to_close) = if let Some(replace_reloader) = change.replace {
                    let session_history = self.joint_session_histories
                        .entry(change.top_level_browsing_context_id).or_insert(JointSessionHistory::new());
                    session_history.replace_reloader(replace_reloader.clone(),
                        NeedsToReload::No(change.new_pipeline_id));

                    match replace_reloader {
                        NeedsToReload::No(pipeline_id) => (Some(vec![pipeline_id]), None),
                        NeedsToReload::Yes(..) => (None, None),
                    }
                } else {
                    let session_history = self.joint_session_histories
                        .entry(change.top_level_browsing_context_id).or_insert(JointSessionHistory::new());
                    let diff = SessionHistoryDiff::BrowsingContextDiff {
                        browsing_context_id: change.browsing_context_id,
                        new_reloader: NeedsToReload::No(change.new_pipeline_id),
                        old_reloader: NeedsToReload::No(old_pipeline_id),
                    };

                    let mut pipelines_to_close = vec![];
                    let mut states_to_close = HashMap::new();

                    let diffs_to_close = session_history.push_diff(diff);

                    for diff in diffs_to_close {
                        match diff {
                            SessionHistoryDiff::BrowsingContextDiff { new_reloader, .. } => {
                                if let Some(pipeline_id) = new_reloader.alive_pipeline_id() {
                                    pipelines_to_close.push(pipeline_id);
                                }
                            }
                            SessionHistoryDiff::PipelineDiff { pipeline_reloader, new_history_state_id, .. } => {
                                if let Some(pipeline_id) = pipeline_reloader.alive_pipeline_id() {
                                    let states = states_to_close.entry(pipeline_id).or_insert(Vec::new());
                                    states.push(new_history_state_id);
                                }
                            }
                        }
                    }

                    (Some(pipelines_to_close), Some(states_to_close))
                };

                self.update_activity(old_pipeline_id);
                self.update_activity(change.new_pipeline_id);

                if let Some(states_to_close) = states_to_close {
                    for (pipeline_id, states) in states_to_close {
                        let msg = ConstellationControlMsg::RemoveHistoryStates(pipeline_id, states);
                        let result = match self.pipelines.get(&pipeline_id) {
                            None => return warn!("Pipeline {} removed history states after closure", pipeline_id),
                            Some(pipeline) => pipeline.event_loop.send(msg),
                        };
                        if let Err(e) = result {
                            self.handle_send_error(pipeline_id, e);
                        }
                    }
                }

                if let Some(pipelines_to_close) = pipelines_to_close {
                    for pipeline_id in pipelines_to_close {
                        self.close_pipeline(pipeline_id, DiscardBrowsingContext::No, ExitPipelineMode::Normal);
                    }
                }

                self.notify_history_changed(change.top_level_browsing_context_id);
            }
        }

        if let Some(top_level_id) = top_level_id {
            self.trim_history(top_level_id);
        }

        self.notify_history_changed(change.top_level_browsing_context_id);
        self.update_frame_tree_if_active(change.top_level_browsing_context_id);
    }

    fn trim_history(&mut self, top_level_browsing_context_id: TopLevelBrowsingContextId) {
        let pipelines_to_evict = {
            let session_history = self.get_joint_session_history(top_level_browsing_context_id);

            let history_length = PREFS.get("session-history.max-length").as_u64().unwrap_or(20) as usize;

            // The past is stored with older entries at the front.
            // We reverse the iter so that newer entries are at the front and then
            // skip _n_ entries and evict the remaining entries.
            let mut pipelines_to_evict = session_history.past.iter().rev()
                .map(|diff| diff.alive_old_pipeline())
                .skip(history_length)
                .filter_map(|maybe_pipeline| maybe_pipeline)
                .collect::<Vec<_>>();

            // The future is stored with oldest entries front, so we must
            // reverse the iterator like we do for the `past`.
            pipelines_to_evict.extend(session_history.future.iter().rev()
                .map(|diff| diff.alive_new_pipeline())
                .skip(history_length)
                .filter_map(|maybe_pipeline| maybe_pipeline));

            pipelines_to_evict
        };

        let mut dead_pipelines = vec![];
        for evicted_id in pipelines_to_evict {
            let load_data = match self.pipelines.get(&evicted_id) {
                Some(pipeline) => {
                    let mut load_data = pipeline.load_data.clone();
                    load_data.url = pipeline.url.clone();
                    load_data
                },
                None => continue,
            };

            dead_pipelines.push((evicted_id, NeedsToReload::Yes(evicted_id, load_data)));
            self.close_pipeline(evicted_id, DiscardBrowsingContext::No, ExitPipelineMode::Normal);
        }

        let session_history = self.get_joint_session_history(top_level_browsing_context_id);

        for (alive_id, dead) in dead_pipelines {
            session_history.replace_reloader(NeedsToReload::No(alive_id), dead);
        }
    }

    fn handle_activate_document_msg(&mut self, pipeline_id: PipelineId) {
        debug!("Document ready to activate {}", pipeline_id);

        // Notify the parent (if there is one).
        if let Some(pipeline) = self.pipelines.get(&pipeline_id) {
            if let Some(parent_pipeline_id) = pipeline.parent_info {
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
                              top_level_browsing_context_id: Option<TopLevelBrowsingContextId>,
                              new_size: WindowSizeData,
                              size_type: WindowSizeType)
    {
        debug!("handle_window_size_msg: {:?}", new_size.initial_viewport.to_untyped());

        if let Some(top_level_browsing_context_id) = top_level_browsing_context_id {
            let browsing_context_id = BrowsingContextId::from(top_level_browsing_context_id);
            self.resize_browsing_context(new_size, size_type, browsing_context_id);
        }

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
                        if let Some(parent_id) = ancestor.parent_info {
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
            let pipelines = browsing_context.pipelines.iter()
                .filter(|pipeline_id| **pipeline_id != pipeline.id)
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

    // Close a browsing context (and all children)
    fn close_browsing_context(&mut self, browsing_context_id: BrowsingContextId, exit_mode: ExitPipelineMode) {
        debug!("Closing browsing context {}.", browsing_context_id);

        self.close_browsing_context_children(browsing_context_id, DiscardBrowsingContext::Yes, exit_mode);

        let browsing_context = match self.browsing_contexts.remove(&browsing_context_id) {
            None => return warn!("Closing browsing context {:?} twice.", browsing_context_id),
            Some(browsing_context) => browsing_context,
        };

        {
            let session_history = self.get_joint_session_history(browsing_context.top_level_id);
            session_history.remove_entries_for_browsing_context(browsing_context_id);
        }

        if BrowsingContextId::from(browsing_context.top_level_id) == browsing_context_id {
            self.event_loops.remove(&browsing_context.top_level_id);
        }

        let parent_info = self.pipelines.get(&browsing_context.pipeline_id)
            .and_then(|pipeline| pipeline.parent_info);

        if let Some(parent_pipeline_id) = parent_info {
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
            pipelines_to_close.extend(&browsing_context.pipelines)
        }

        for pipeline_id in pipelines_to_close {
            self.close_pipeline(pipeline_id, dbc, exit_mode);
        }

        debug!("Closed browsing context children {}.", browsing_context_id);
    }

    // Discard the pipeline for a given document, udpdate the joint session history.
    fn handle_discard_document(&mut self,
                               top_level_browsing_context_id: TopLevelBrowsingContextId,
                               pipeline_id: PipelineId) {
        let load_data = match self.pipelines.get(&pipeline_id) {
            Some(pipeline) => pipeline.load_data.clone(),
            None => return
        };
        self.joint_session_histories
            .entry(top_level_browsing_context_id).or_insert(JointSessionHistory::new())
            .replace_reloader(NeedsToReload::No(pipeline_id), NeedsToReload::Yes(pipeline_id, load_data));
       self.close_pipeline(pipeline_id, DiscardBrowsingContext::No, ExitPipelineMode::Normal);
    }

    // Send a message to script requesting the document associated with this pipeline runs the 'unload' algorithm.
    fn unload_document(&self, pipeline_id: PipelineId) {
        if let Some(pipeline) = self.pipelines.get(&pipeline_id) {
            let msg = ConstellationControlMsg::UnloadDocument(pipeline_id);
            let _ = pipeline.event_loop.send(msg);
        }
    }

    // Close all pipelines at and beneath a given browsing context
    fn close_pipeline(&mut self, pipeline_id: PipelineId, dbc: DiscardBrowsingContext, exit_mode: ExitPipelineMode) {
        debug!("Closing pipeline {:?}.", pipeline_id);

        // Sever connection to browsing context
        let browsing_context_id = self.pipelines.get(&pipeline_id).map(|pipeline| pipeline.browsing_context_id);
        if let Some(browsing_context) = browsing_context_id
            .and_then(|browsing_context_id| self.browsing_contexts.get_mut(&browsing_context_id))
        {
            browsing_context.pipelines.remove(&pipeline_id);
        }

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
                    if self.pending_changes.iter().any(|change| change.new_pipeline_id == pipeline.id) &&
                        probability <= rng.gen::<f32>() {
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

    fn get_joint_session_history(&mut self, top_level_id: TopLevelBrowsingContextId) -> &mut JointSessionHistory {
        self.joint_session_histories.entry(top_level_id).or_insert(JointSessionHistory::new())
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
    fn update_frame_tree_if_active(&mut self, top_level_browsing_context_id: TopLevelBrowsingContextId) {
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

    fn focused_pipeline_is_descendant_of(&self, browsing_context_id: BrowsingContextId) -> bool {
        self.focus_pipeline_id.map_or(false, |pipeline_id| {
            self.fully_active_descendant_browsing_contexts_iter(browsing_context_id)
                .any(|browsing_context| browsing_context.pipeline_id == pipeline_id)
        })
    }
}
