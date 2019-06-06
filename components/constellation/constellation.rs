/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

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

use crate::browsingcontext::NewBrowsingContextInfo;
use crate::browsingcontext::{
    AllBrowsingContextsIterator, BrowsingContext, FullyActiveBrowsingContextsIterator,
};
use crate::event_loop::EventLoop;
use crate::network_listener::NetworkListener;
use crate::pipeline::{InitialPipelineState, Pipeline};
use crate::session_history::{
    JointSessionHistory, NeedsToReload, SessionHistoryChange, SessionHistoryDiff,
};
use crate::timer_scheduler::TimerScheduler;
use background_hang_monitor::HangMonitorRegister;
use backtrace::Backtrace;
use bluetooth_traits::BluetoothRequest;
use canvas::canvas_paint_thread::CanvasPaintThread;
use canvas::webgl_thread::WebGLThreads;
use canvas_traits::canvas::CanvasId;
use canvas_traits::canvas::CanvasMsg;
use clipboard::{ClipboardContext, ClipboardProvider};
use compositing::compositor_thread::CompositorProxy;
use compositing::compositor_thread::Msg as ToCompositorMsg;
use compositing::SendableFrameTree;
use crossbeam_channel::{unbounded, Receiver, Sender};
use devtools_traits::{ChromeToDevtoolsControlMsg, DevtoolsControlMsg};
use embedder_traits::{Cursor, EmbedderMsg, EmbedderProxy};
use euclid::{Size2D, TypedScale, TypedSize2D};
use gfx::font_cache_thread::FontCacheThread;
use gfx_traits::Epoch;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use ipc_channel::Error as IpcError;
use keyboard_types::webdriver::Event as WebDriverInputEvent;
use keyboard_types::KeyboardEvent;
use layout_traits::LayoutThreadFactory;
use log::{Level, LevelFilter, Log, Metadata, Record};
use msg::constellation_msg::{BackgroundHangMonitorRegister, HangMonitorAlert, SamplerControlMsg};
use msg::constellation_msg::{
    BrowsingContextId, HistoryStateId, PipelineId, TopLevelBrowsingContextId,
};
use msg::constellation_msg::{PipelineNamespace, PipelineNamespaceId, TraversalDirection};
use net_traits::pub_domains::reg_host;
use net_traits::request::RequestBuilder;
use net_traits::storage_thread::{StorageThreadMsg, StorageType};
use net_traits::{self, FetchResponseMsg, IpcSend, ResourceThreads};
use profile_traits::mem;
use profile_traits::time;
use script_traits::CompositorEvent::{MouseButtonEvent, MouseMoveEvent};
use script_traits::MouseEventType;
use script_traits::{webdriver_msg, LogEntry, ScriptToConstellationChan, ServiceWorkerMsg};
use script_traits::{
    AnimationState, AnimationTickType, AuxiliaryBrowsingContextLoadInfo, CompositorEvent,
};
use script_traits::{
    ConstellationControlMsg, ConstellationMsg as FromCompositorMsg, DiscardBrowsingContext,
};
use script_traits::{DocumentActivity, DocumentState, LayoutControlMsg, LoadData};
use script_traits::{HistoryEntryReplacement, IFrameSizeMsg, WindowSizeData, WindowSizeType};
use script_traits::{
    IFrameLoadInfo, IFrameLoadInfoWithData, IFrameSandboxState, TimerSchedulerMsg,
};
use script_traits::{LayoutMsg as FromLayoutMsg, ScriptMsg as FromScriptMsg, ScriptThreadFactory};
use script_traits::{SWManagerMsg, ScopeThings, UpdatePipelineIdReason, WebDriverCommandMsg};
use serde::{Deserialize, Serialize};
use servo_config::{opts, pref};
use servo_geometry::DeviceIndependentPixel;
use servo_rand::{random, Rng, SeedableRng, ServoRng};
use servo_remutex::ReentrantMutex;
use servo_url::{Host, ImmutableOrigin, ServoUrl};
use std::borrow::ToOwned;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, VecDeque};
use std::marker::PhantomData;
use std::mem::replace;
use std::process;
use std::rc::{Rc, Weak};
use std::sync::Arc;
use std::thread;
use style_traits::viewport::ViewportConstraints;
use style_traits::CSSPixel;
use webvr_traits::{WebVREvent, WebVRMsg};

type PendingApprovalNavigations = HashMap<PipelineId, (LoadData, HistoryEntryReplacement)>;

/// Servo supports tabs (referred to as browsers), so `Constellation` needs to
/// store browser specific data for bookkeeping.
struct Browser {
    /// The currently focused browsing context in this browser for key events.
    /// The focused pipeline is the current entry of the focused browsing
    /// context.
    focused_browsing_context_id: BrowsingContextId,

    /// The joint session history for this browser.
    session_history: JointSessionHistory,
}

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

    /// A handle to register components for hang monitoring.
    /// None when in multiprocess mode.
    background_monitor_register: Option<Box<BackgroundHangMonitorRegister>>,

    /// Channels to control all sampling profilers.
    sampling_profiler_control: Vec<IpcSender<SamplerControlMsg>>,

    /// A channel for the background hang monitor to send messages
    /// to the constellation.
    background_hang_monitor_sender: IpcSender<HangMonitorAlert>,

    /// A channel for the constellation to receiver messages
    /// from the background hang monitor.
    background_hang_monitor_receiver: Receiver<Result<HangMonitorAlert, IpcError>>,

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

    /// The last frame tree sent to WebRender, denoting the browser (tab) user
    /// has currently selected. This also serves as the key to retrieve data
    /// about the current active browser from `browsers`.
    active_browser_id: Option<TopLevelBrowsingContextId>,

    /// Bookkeeping data for all browsers in constellation.
    browsers: HashMap<TopLevelBrowsingContextId, Browser>,

    /// Channels for the constellation to send messages to the public
    /// resource-related threads. There are two groups of resource threads: one
    /// for public browsing, and one for private browsing.
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

    /// The set of all event loops in the browser.
    /// We store the event loops in a map
    /// indexed by registered domain name (as a `Host`) to event loops.
    /// It is important that scripts with the same eTLD+1
    /// share an event loop, since they can use `document.domain`
    /// to become same-origin, at which point they can share DOM objects.
    event_loops: HashMap<Host, Weak<EventLoop>>,

    /// The set of all the pipelines in the browser.  (See the `pipeline` module
    /// for more details.)
    pipelines: HashMap<PipelineId, Pipeline>,

    /// The set of all the browsing contexts in the browser.
    browsing_contexts: HashMap<BrowsingContextId, BrowsingContext>,

    /// When a navigation is performed, we do not immediately update
    /// the session history, instead we ask the event loop to begin loading
    /// the new document, and do not update the browsing context until the
    /// document is active. Between starting the load and it activating,
    /// we store a `SessionHistoryChange` object for the navigation in progress.
    pending_changes: Vec<SessionHistoryChange>,

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

    /// Navigation requests from script awaiting approval from the embedder.
    pending_approval_navigations: PendingApprovalNavigations,

    /// Bitmask which indicates which combination of mouse buttons are
    /// currently being pressed.
    pressed_mouse_buttons: u16,

    is_running_problem_test: bool,

    /// If True, exits on thread failure instead of displaying about:failure
    hard_fail: bool,

    /// If set with --disable-canvas-aa, disable antialiasing on the HTML
    /// canvas element.
    /// Like --disable-text-aa, this is useful for reftests where pixel perfect
    /// results are required.
    enable_canvas_antialiasing: bool,
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
            script_to_constellation_chan: Arc::new(ReentrantMutex::new(
                script_to_constellation_chan,
            )),
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
            let chan = self
                .script_to_constellation_chan
                .lock()
                .unwrap_or_else(|err| err.into_inner());
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
            constellation_chan: Arc::new(ReentrantMutex::new(constellation_chan)),
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
            let chan = self
                .constellation_chan
                .lock()
                .unwrap_or_else(|err| err.into_inner());
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
            format!("{:?}", Backtrace::new()),
        )),
        Level::Error => Some(LogEntry::Error(format!("{}", record.args()))),
        Level::Warn => Some(LogEntry::Warn(format!("{}", record.args()))),
        _ => None,
    }
}

/// The number of warnings to include in each crash report.
const WARNINGS_BUFFER_SIZE: usize = 32;

/// Route an ipc receiver to an mpsc receiver, preserving any errors.
/// This is the same as `route_ipc_receiver_to_new_mpsc_receiver`,
/// but does not panic on deserializtion errors.
fn route_ipc_receiver_to_new_mpsc_receiver_preserving_errors<T>(
    ipc_receiver: IpcReceiver<T>,
) -> Receiver<Result<T, IpcError>>
where
    T: for<'de> Deserialize<'de> + Serialize + Send + 'static,
{
    let (mpsc_sender, mpsc_receiver) = unbounded();
    ROUTER.add_route(
        ipc_receiver.to_opaque(),
        Box::new(move |message| drop(mpsc_sender.send(message.to::<T>()))),
    );
    mpsc_receiver
}

impl<Message, LTF, STF> Constellation<Message, LTF, STF>
where
    LTF: LayoutThreadFactory<Message = Message>,
    STF: ScriptThreadFactory<Message = Message>,
{
    /// Create a new constellation thread.
    pub fn start(
        state: InitialConstellationState,
        initial_window_size: TypedSize2D<u32, DeviceIndependentPixel>,
        device_pixels_per_px: Option<f32>,
        random_pipeline_closure_probability: Option<f32>,
        random_pipeline_closure_seed: Option<usize>,
        is_running_problem_test: bool,
        hard_fail: bool,
        enable_canvas_antialiasing: bool,
    ) -> (Sender<FromCompositorMsg>, IpcSender<SWManagerMsg>) {
        let (compositor_sender, compositor_receiver) = unbounded();

        // service worker manager to communicate with constellation
        let (swmanager_sender, swmanager_receiver) = ipc::channel().expect("ipc channel failure");
        let sw_mgr_clone = swmanager_sender.clone();

        thread::Builder::new()
            .name("Constellation".to_owned())
            .spawn(move || {
                let (ipc_script_sender, ipc_script_receiver) =
                    ipc::channel().expect("ipc channel failure");
                let script_receiver =
                    route_ipc_receiver_to_new_mpsc_receiver_preserving_errors(ipc_script_receiver);

                let (background_hang_monitor_sender, ipc_bhm_receiver) =
                    ipc::channel().expect("ipc channel failure");
                let background_hang_monitor_receiver =
                    route_ipc_receiver_to_new_mpsc_receiver_preserving_errors(ipc_bhm_receiver);

                // If we are in multiprocess mode,
                // a dedicated per-process hang monitor will be initialized later inside the content process.
                // See run_content_process in servo/lib.rs
                let (background_monitor_register, sampler_chan) = if opts::multiprocess() {
                    (None, vec![])
                } else {
                    let (sampling_profiler_control, sampling_profiler_port) =
                        ipc::channel().expect("ipc channel failure");

                    (
                        Some(HangMonitorRegister::init(
                            background_hang_monitor_sender.clone(),
                            sampling_profiler_port,
                        )),
                        vec![sampling_profiler_control],
                    )
                };

                let (ipc_layout_sender, ipc_layout_receiver) =
                    ipc::channel().expect("ipc channel failure");
                let layout_receiver =
                    route_ipc_receiver_to_new_mpsc_receiver_preserving_errors(ipc_layout_receiver);

                let (network_listener_sender, network_listener_receiver) = unbounded();

                let swmanager_receiver =
                    route_ipc_receiver_to_new_mpsc_receiver_preserving_errors(swmanager_receiver);

                // Zero is reserved for the embedder.
                PipelineNamespace::install(PipelineNamespaceId(1));

                let mut constellation: Constellation<Message, LTF, STF> = Constellation {
                    script_sender: ipc_script_sender,
                    background_hang_monitor_sender,
                    background_hang_monitor_receiver,
                    background_monitor_register,
                    sampling_profiler_control: sampler_chan,
                    layout_sender: ipc_layout_sender,
                    script_receiver: script_receiver,
                    compositor_receiver: compositor_receiver,
                    layout_receiver: layout_receiver,
                    network_listener_sender: network_listener_sender,
                    network_listener_receiver: network_listener_receiver,
                    embedder_proxy: state.embedder_proxy,
                    compositor_proxy: state.compositor_proxy,
                    active_browser_id: None,
                    browsers: HashMap::new(),
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
                    pending_changes: vec![],
                    // We initialize the namespace at 2, since we reserved
                    // namespace 0 for the embedder, and 0 for the constellation
                    next_pipeline_namespace_id: PipelineNamespaceId(2),
                    time_profiler_chan: state.time_profiler_chan,
                    mem_profiler_chan: state.mem_profiler_chan,
                    window_size: WindowSizeData {
                        initial_viewport: initial_window_size.to_f32() * TypedScale::new(1.0),
                        device_pixel_ratio: TypedScale::new(device_pixels_per_px.unwrap_or(1.0)),
                    },
                    phantom: PhantomData,
                    clipboard_ctx: match ClipboardContext::new() {
                        Ok(c) => Some(c),
                        Err(e) => {
                            warn!("Error creating clipboard context ({})", e);
                            None
                        },
                    },
                    webdriver: WebDriverData::new(),
                    scheduler_chan: TimerScheduler::start(),
                    document_states: HashMap::new(),
                    webrender_document: state.webrender_document,
                    webrender_api_sender: state.webrender_api_sender,
                    shutting_down: false,
                    handled_warnings: VecDeque::new(),
                    random_pipeline_closure: random_pipeline_closure_probability.map(|prob| {
                        let seed = random_pipeline_closure_seed.unwrap_or_else(random);
                        let rng = ServoRng::from_seed(&[seed]);
                        warn!("Randomly closing pipelines.");
                        info!("Using seed {} for random pipeline closure.", seed);
                        (rng, prob)
                    }),
                    webgl_threads: state.webgl_threads,
                    webvr_chan: state.webvr_chan,
                    canvas_chan: CanvasPaintThread::start(),
                    pending_approval_navigations: HashMap::new(),
                    pressed_mouse_buttons: 0,
                    is_running_problem_test,
                    hard_fail,
                    enable_canvas_antialiasing,
                };

                constellation.run();
            })
            .expect("Thread spawning failed");

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
    fn new_pipeline(
        &mut self,
        pipeline_id: PipelineId,
        browsing_context_id: BrowsingContextId,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
        parent_pipeline_id: Option<PipelineId>,
        opener: Option<BrowsingContextId>,
        initial_window_size: TypedSize2D<f32, CSSPixel>,
        // TODO: we have to provide ownership of the LoadData
        // here, because it will be send on an ipc channel,
        // and ipc channels take onership of their data.
        // https://github.com/servo/ipc-channel/issues/138
        load_data: LoadData,
        sandbox: IFrameSandboxState,
        is_private: bool,
        is_visible: bool,
    ) {
        if self.shutting_down {
            return;
        }
        debug!(
            "Creating new pipeline {} in browsing context {}.",
            pipeline_id, browsing_context_id
        );

        let (event_loop, host) = match sandbox {
            IFrameSandboxState::IFrameSandboxed => (None, None),
            IFrameSandboxState::IFrameUnsandboxed => {
                // If this is an about:blank load, it must share the creator's event loop.
                // This must match the logic in the script thread when determining the proper origin.
                if load_data.url.as_str() != "about:blank" {
                    match reg_host(&load_data.url) {
                        None => (None, None),
                        Some(host) => {
                            let event_loop =
                                self.event_loops.get(&host).and_then(|weak| weak.upgrade());
                            match event_loop {
                                None => (None, Some(host)),
                                Some(event_loop) => (Some(event_loop), None),
                            }
                        },
                    }
                } else if let Some(parent) =
                    parent_pipeline_id.and_then(|pipeline_id| self.pipelines.get(&pipeline_id))
                {
                    (Some(parent.event_loop.clone()), None)
                } else if let Some(creator) = load_data
                    .creator_pipeline_id
                    .and_then(|pipeline_id| self.pipelines.get(&pipeline_id))
                {
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

        let result = Pipeline::spawn::<Message, LTF, STF>(InitialPipelineState {
            id: pipeline_id,
            browsing_context_id,
            top_level_browsing_context_id,
            parent_pipeline_id,
            opener,
            script_to_constellation_chan: ScriptToConstellationChan {
                sender: self.script_sender.clone(),
                pipeline_id: pipeline_id,
            },
            background_monitor_register: self.background_monitor_register.clone(),
            background_hang_monitor_to_constellation_chan: self
                .background_hang_monitor_sender
                .clone(),
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
            prev_visibility: is_visible,
            webrender_api_sender: self.webrender_api_sender.clone(),
            webrender_document: self.webrender_document,
            webgl_chan: self
                .webgl_threads
                .as_ref()
                .map(|threads| threads.pipeline()),
            webvr_chan: self.webvr_chan.clone(),
        });

        let pipeline = match result {
            Ok(result) => result,
            Err(e) => return self.handle_send_error(pipeline_id, e),
        };

        if let Some(sampler_chan) = pipeline.sampler_control_chan {
            self.sampling_profiler_control.push(sampler_chan);
        }

        if let Some(host) = host {
            debug!(
                "Adding new host entry {} for top-level browsing context {}.",
                host, top_level_browsing_context_id
            );
            let _ = self
                .event_loops
                .insert(host, Rc::downgrade(&pipeline.pipeline.event_loop));
        }

        assert!(!self.pipelines.contains_key(&pipeline_id));
        self.pipelines.insert(pipeline_id, pipeline.pipeline);
    }

    /// Get an iterator for the fully active browsing contexts in a subtree.
    fn fully_active_descendant_browsing_contexts_iter(
        &self,
        browsing_context_id: BrowsingContextId,
    ) -> FullyActiveBrowsingContextsIterator {
        FullyActiveBrowsingContextsIterator {
            stack: vec![browsing_context_id],
            pipelines: &self.pipelines,
            browsing_contexts: &self.browsing_contexts,
        }
    }

    /// Get an iterator for the fully active browsing contexts in a tree.
    fn fully_active_browsing_contexts_iter(
        &self,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
    ) -> FullyActiveBrowsingContextsIterator {
        self.fully_active_descendant_browsing_contexts_iter(BrowsingContextId::from(
            top_level_browsing_context_id,
        ))
    }

    /// Get an iterator for the browsing contexts in a subtree.
    fn all_descendant_browsing_contexts_iter(
        &self,
        browsing_context_id: BrowsingContextId,
    ) -> AllBrowsingContextsIterator {
        AllBrowsingContextsIterator {
            stack: vec![browsing_context_id],
            pipelines: &self.pipelines,
            browsing_contexts: &self.browsing_contexts,
        }
    }

    /// Create a new browsing context and update the internal bookkeeping.
    fn new_browsing_context(
        &mut self,
        browsing_context_id: BrowsingContextId,
        top_level_id: TopLevelBrowsingContextId,
        pipeline_id: PipelineId,
        parent_pipeline_id: Option<PipelineId>,
        size: TypedSize2D<f32, CSSPixel>,
        is_private: bool,
        is_visible: bool,
    ) {
        debug!("Creating new browsing context {}", browsing_context_id);
        let browsing_context = BrowsingContext::new(
            browsing_context_id,
            top_level_id,
            pipeline_id,
            parent_pipeline_id,
            size,
            is_private,
            is_visible,
        );
        self.browsing_contexts
            .insert(browsing_context_id, browsing_context);

        // If this context is a nested container, attach it to parent pipeline.
        if let Some(parent_pipeline_id) = parent_pipeline_id {
            if let Some(parent) = self.pipelines.get_mut(&parent_pipeline_id) {
                parent.add_child(browsing_context_id);
            }
        }
    }

    fn add_pending_change(&mut self, change: SessionHistoryChange) {
        debug!(
            "adding pending session history change with {}",
            if change.replace.is_some() {
                "replacement"
            } else {
                "no replacement"
            },
        );
        self.handle_load_start_msg(
            change.top_level_browsing_context_id,
            change.browsing_context_id,
        );
        self.pending_changes.push(change);
    }

    /// Handles loading pages, navigation, and granting access to the compositor
    fn handle_request(&mut self) {
        #[derive(Debug)]
        enum Request {
            Script((PipelineId, FromScriptMsg)),
            BackgroundHangMonitor(HangMonitorAlert),
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
        let request = select! {
            recv(self.script_receiver) -> msg => {
                msg.expect("Unexpected script channel panic in constellation").map(Request::Script)
            }
            recv(self.background_hang_monitor_receiver) -> msg => {
                msg.expect("Unexpected BHM channel panic in constellation").map(Request::BackgroundHangMonitor)
            }
            recv(self.compositor_receiver) -> msg => {
                Ok(Request::Compositor(msg.expect("Unexpected compositor channel panic in constellation")))
            }
            recv(self.layout_receiver) -> msg => {
                msg.expect("Unexpected layout channel panic in constellation").map(Request::Layout)
            }
            recv(self.network_listener_receiver) -> msg => {
                Ok(Request::NetworkListener(
                    msg.expect("Unexpected network listener channel panic in constellation")
                ))
            }
            recv(self.swmanager_receiver) -> msg => {
                msg.expect("Unexpected panic channel panic in constellation").map(Request::FromSWManager)
            }
        };

        let request = match request {
            Ok(request) => request,
            Err(err) => return error!("Deserialization failed ({}).", err),
        };

        match request {
            Request::Compositor(message) => self.handle_request_from_compositor(message),
            Request::Script(message) => {
                self.handle_request_from_script(message);
            },
            Request::BackgroundHangMonitor(message) => {
                self.handle_request_from_background_hang_monitor(message);
            },
            Request::Layout(message) => {
                self.handle_request_from_layout(message);
            },
            Request::NetworkListener(message) => {
                self.handle_request_from_network_listener(message);
            },
            Request::FromSWManager(message) => {
                self.handle_request_from_swmanager(message);
            },
        }
    }

    fn handle_request_from_background_hang_monitor(&self, message: HangMonitorAlert) {
        match message {
            HangMonitorAlert::Profile(bytes) => self
                .embedder_proxy
                .send((None, EmbedderMsg::ReportProfile(bytes))),
            HangMonitorAlert::Hang(hang) => {
                // TODO: In case of a permanent hang being reported, add a "kill script" workflow,
                // via the embedder?
                warn!("Component hang alert: {:?}", hang);
            },
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
            },
        }
    }

    fn handle_request_from_compositor(&mut self, message: FromCompositorMsg) {
        debug!("constellation got {:?} message", message);
        match message {
            FromCompositorMsg::Exit => {
                self.handle_exit();
            },
            FromCompositorMsg::GetBrowsingContext(pipeline_id, resp_chan) => {
                self.handle_get_browsing_context(pipeline_id, resp_chan);
            },
            FromCompositorMsg::GetPipeline(browsing_context_id, resp_chan) => {
                self.handle_get_pipeline(browsing_context_id, resp_chan);
            },
            FromCompositorMsg::GetFocusTopLevelBrowsingContext(resp_chan) => {
                // The focused browsing context's top-level browsing context is
                // the active browser's id itself.
                let _ = resp_chan.send(self.active_browser_id);
            },
            FromCompositorMsg::Keyboard(key_event) => {
                self.handle_key_msg(key_event);
            },
            // Perform a navigation previously requested by script, if approved by the embedder.
            // If there is already a pending page (self.pending_changes), it will not be overridden;
            // However, if the id is not encompassed by another change, it will be.
            FromCompositorMsg::AllowNavigationResponse(pipeline_id, allowed) => {
                let pending = self.pending_approval_navigations.remove(&pipeline_id);

                let top_level_browsing_context_id = match self.pipelines.get(&pipeline_id) {
                    Some(pipeline) => pipeline.top_level_browsing_context_id,
                    None => return warn!("Attempted to navigate {} after closure.", pipeline_id),
                };

                match pending {
                    Some((load_data, replace)) => {
                        if allowed {
                            self.load_url(
                                top_level_browsing_context_id,
                                pipeline_id,
                                load_data,
                                replace,
                            );
                        } else {
                            let pipeline_is_top_level_pipeline = self
                                .browsing_contexts
                                .get(&BrowsingContextId::from(top_level_browsing_context_id))
                                .map(|ctx| ctx.pipeline_id == pipeline_id)
                                .unwrap_or(false);
                            // If the navigation is refused, and this concerns an iframe,
                            // we need to take it out of it's "delaying-load-events-mode".
                            // https://html.spec.whatwg.org/multipage/#delaying-load-events-mode
                            if !pipeline_is_top_level_pipeline {
                                let msg = ConstellationControlMsg::StopDelayingLoadEventsMode(
                                    pipeline_id,
                                );
                                let result = match self.pipelines.get(&pipeline_id) {
                                    Some(pipeline) => pipeline.event_loop.send(msg),
                                    None => {
                                        return warn!(
                                            "Attempted to navigate {} after closure.",
                                            pipeline_id
                                        );
                                    },
                                };
                                if let Err(e) = result {
                                    self.handle_send_error(pipeline_id, e);
                                }
                            }
                        }
                    },
                    None => {
                        return warn!(
                            "AllowNavigationReqsponse for unknow request: {:?}",
                            pipeline_id
                        );
                    },
                };
            },
            // Load a new page from a typed url
            // If there is already a pending page (self.pending_changes), it will not be overridden;
            // However, if the id is not encompassed by another change, it will be.
            FromCompositorMsg::LoadUrl(top_level_browsing_context_id, url) => {
                let load_data = LoadData::new(url.origin(), url, None, None, None);
                let ctx_id = BrowsingContextId::from(top_level_browsing_context_id);
                let pipeline_id = match self.browsing_contexts.get(&ctx_id) {
                    Some(ctx) => ctx.pipeline_id,
                    None => {
                        return warn!(
                            "LoadUrl for unknow browsing context: {:?}",
                            top_level_browsing_context_id
                        );
                    },
                };
                // Since this is a top-level load, initiated by the embedder, go straight to load_url,
                // bypassing schedule_navigation.
                self.load_url(
                    top_level_browsing_context_id,
                    pipeline_id,
                    load_data,
                    HistoryEntryReplacement::Disabled,
                );
            },
            FromCompositorMsg::IsReadyToSaveImage(pipeline_states) => {
                let is_ready = self.handle_is_ready_to_save_image(pipeline_states);
                debug!("Ready to save image {:?}.", is_ready);
                if self.is_running_problem_test {
                    println!("got ready to save image query, result is {:?}", is_ready);
                }
                let is_ready = is_ready == ReadyToSave::Ready;
                self.compositor_proxy
                    .send(ToCompositorMsg::IsReadyToSaveImageReply(is_ready));
                if self.is_running_problem_test {
                    println!("sent response");
                }
            },
            // Create a new top level browsing context. Will use response_chan to return
            // the browsing context id.
            FromCompositorMsg::NewBrowser(url, top_level_browsing_context_id) => {
                self.handle_new_top_level_browsing_context(url, top_level_browsing_context_id);
            },
            // Close a top level browsing context.
            FromCompositorMsg::CloseBrowser(top_level_browsing_context_id) => {
                self.handle_close_top_level_browsing_context(top_level_browsing_context_id);
            },
            // Panic a top level browsing context.
            FromCompositorMsg::SendError(top_level_browsing_context_id, error) => {
                debug!("constellation got SendError message");
                if let Some(id) = top_level_browsing_context_id {
                    self.handle_panic(id, error, None);
                } else {
                    warn!("constellation got a SendError message without top level id");
                }
            },
            // Send frame tree to WebRender. Make it visible.
            FromCompositorMsg::SelectBrowser(top_level_browsing_context_id) => {
                self.send_frame_tree(top_level_browsing_context_id);
            },
            // Handle a forward or back request
            FromCompositorMsg::TraverseHistory(top_level_browsing_context_id, direction) => {
                self.handle_traverse_history_msg(top_level_browsing_context_id, direction);
            },
            FromCompositorMsg::WindowSize(top_level_browsing_context_id, new_size, size_type) => {
                self.handle_window_size_msg(top_level_browsing_context_id, new_size, size_type);
            },
            FromCompositorMsg::TickAnimation(pipeline_id, tick_type) => {
                self.handle_tick_animation(pipeline_id, tick_type)
            },
            FromCompositorMsg::WebDriverCommand(command) => {
                self.handle_webdriver_msg(command);
            },
            FromCompositorMsg::Reload(top_level_browsing_context_id) => {
                self.handle_reload_msg(top_level_browsing_context_id);
            },
            FromCompositorMsg::LogEntry(top_level_browsing_context_id, thread_name, entry) => {
                self.handle_log_entry(top_level_browsing_context_id, thread_name, entry);
            },
            FromCompositorMsg::WebVREvents(pipeline_ids, events) => {
                self.handle_webvr_events(pipeline_ids, events);
            },
            FromCompositorMsg::ForwardEvent(destination_pipeline_id, event) => {
                self.forward_event(destination_pipeline_id, event);
            },
            FromCompositorMsg::SetCursor(cursor) => self.handle_set_cursor_msg(cursor),
            FromCompositorMsg::EnableProfiler(rate, max_duration) => {
                for chan in &self.sampling_profiler_control {
                    if let Err(e) = chan.send(SamplerControlMsg::Enable(rate, max_duration)) {
                        warn!("error communicating with sampling profiler: {}", e);
                    }
                }
            },
            FromCompositorMsg::DisableProfiler => {
                for chan in &self.sampling_profiler_control {
                    if let Err(e) = chan.send(SamplerControlMsg::Disable) {
                        warn!("error communicating with sampling profiler: {}", e);
                    }
                }
            },
            FromCompositorMsg::ExitFullScreen(top_level_browsing_context_id) => {
                self.handle_exit_fullscreen_msg(top_level_browsing_context_id);
            },
        }
    }

    fn handle_request_from_script(&mut self, message: (PipelineId, FromScriptMsg)) {
        let (source_pipeline_id, content) = message;
        debug!(
            "constellation got {:?} message from pipeline {}",
            content, source_pipeline_id
        );

        let source_top_ctx_id = match self
            .pipelines
            .get(&source_pipeline_id)
            .map(|pipeline| pipeline.top_level_browsing_context_id)
        {
            None => return warn!("ScriptMsg from closed pipeline {:?}.", source_pipeline_id),
            Some(ctx) => ctx,
        };

        match content {
            FromScriptMsg::ForwardToEmbedder(embedder_msg) => {
                self.embedder_proxy
                    .send((Some(source_top_ctx_id), embedder_msg));
            },
            FromScriptMsg::PipelineExited => {
                self.handle_pipeline_exited(source_pipeline_id);
            },
            FromScriptMsg::DiscardDocument => {
                self.handle_discard_document(source_top_ctx_id, source_pipeline_id);
            },
            FromScriptMsg::DiscardTopLevelBrowsingContext => {
                self.handle_close_top_level_browsing_context(source_top_ctx_id);
            },

            FromScriptMsg::InitiateNavigateRequest(req_init, cancel_chan) => {
                self.handle_navigate_request(source_pipeline_id, req_init, cancel_chan);
            },
            FromScriptMsg::ScriptLoadedURLInIFrame(load_info) => {
                self.handle_script_loaded_url_in_iframe_msg(load_info);
            },
            FromScriptMsg::ScriptNewIFrame(load_info, layout_sender) => {
                self.handle_script_new_iframe(load_info, layout_sender);
            },
            FromScriptMsg::ScriptNewAuxiliary(load_info, layout_sender) => {
                self.handle_script_new_auxiliary(load_info, layout_sender);
            },
            FromScriptMsg::ChangeRunningAnimationsState(animation_state) => {
                self.handle_change_running_animations_state(source_pipeline_id, animation_state)
            },
            // Ask the embedder for permission to load a new page.
            FromScriptMsg::LoadUrl(load_data, replace) => {
                self.schedule_navigation(source_top_ctx_id, source_pipeline_id, load_data, replace);
            },
            FromScriptMsg::AbortLoadUrl => {
                self.handle_abort_load_url_msg(source_pipeline_id);
            },
            // A page loaded has completed all parsing, script, and reflow messages have been sent.
            FromScriptMsg::LoadComplete => {
                self.handle_load_complete_msg(source_top_ctx_id, source_pipeline_id)
            },
            // Handle navigating to a fragment
            FromScriptMsg::NavigatedToFragment(new_url, replacement_enabled) => {
                self.handle_navigated_to_fragment(source_pipeline_id, new_url, replacement_enabled);
            },
            // Handle a forward or back request
            FromScriptMsg::TraverseHistory(direction) => {
                self.handle_traverse_history_msg(source_top_ctx_id, direction);
            },
            // Handle a push history state request.
            FromScriptMsg::PushHistoryState(history_state_id, url) => {
                self.handle_push_history_state_msg(source_pipeline_id, history_state_id, url);
            },
            FromScriptMsg::ReplaceHistoryState(history_state_id, url) => {
                self.handle_replace_history_state_msg(source_pipeline_id, history_state_id, url);
            },
            // Handle a joint session history length request.
            FromScriptMsg::JointSessionHistoryLength(sender) => {
                self.handle_joint_session_history_length(source_top_ctx_id, sender);
            },
            // Notification that the new document is ready to become active
            FromScriptMsg::ActivateDocument => {
                self.handle_activate_document_msg(source_pipeline_id);
            },
            // Update pipeline url after redirections
            FromScriptMsg::SetFinalUrl(final_url) => {
                // The script may have finished loading after we already started shutting down.
                if let Some(ref mut pipeline) = self.pipelines.get_mut(&source_pipeline_id) {
                    pipeline.url = final_url;
                } else {
                    warn!("constellation got set final url message for dead pipeline");
                }
            },
            FromScriptMsg::PostMessage {
                target: browsing_context_id,
                source: source_pipeline_id,
                target_origin: origin,
                data,
            } => {
                self.handle_post_message_msg(browsing_context_id, source_pipeline_id, origin, data);
            },
            FromScriptMsg::Focus => {
                self.handle_focus_msg(source_pipeline_id);
            },
            FromScriptMsg::GetClipboardContents(sender) => {
                let contents = match self.clipboard_ctx {
                    Some(ref mut ctx) => {
                        match ctx.get_contents() {
                            Ok(c) => c,
                            Err(e) => {
                                warn!("Error getting clipboard contents ({}), defaulting to empty string", e);
                                "".to_owned()
                            },
                        }
                    },
                    None => "".to_owned(),
                };
                if let Err(e) = sender.send(contents) {
                    warn!("Failed to send clipboard ({})", e);
                }
            },
            FromScriptMsg::SetClipboardContents(s) => {
                if let Some(ref mut ctx) = self.clipboard_ctx {
                    if let Err(e) = ctx.set_contents(s) {
                        warn!("Error setting clipboard contents ({})", e);
                    }
                }
            },
            FromScriptMsg::VisibilityChangeComplete(is_visible) => {
                self.handle_visibility_change_complete(source_pipeline_id, is_visible);
            },
            FromScriptMsg::RemoveIFrame(browsing_context_id, sender) => {
                let removed_pipeline_ids = self.handle_remove_iframe_msg(browsing_context_id);
                if let Err(e) = sender.send(removed_pipeline_ids) {
                    warn!("Error replying to remove iframe ({})", e);
                }
            },
            FromScriptMsg::CreateCanvasPaintThread(size, sender) => {
                self.handle_create_canvas_paint_thread_msg(size, sender)
            },
            FromScriptMsg::SetDocumentState(state) => {
                self.document_states.insert(source_pipeline_id, state);
            },
            FromScriptMsg::GetClientWindow(send) => {
                self.compositor_proxy
                    .send(ToCompositorMsg::GetClientWindow(send));
            },
            FromScriptMsg::GetScreenSize(send) => {
                self.compositor_proxy
                    .send(ToCompositorMsg::GetScreenSize(send));
            },
            FromScriptMsg::GetScreenAvailSize(send) => {
                self.compositor_proxy
                    .send(ToCompositorMsg::GetScreenAvailSize(send));
            },
            FromScriptMsg::LogEntry(thread_name, entry) => {
                self.handle_log_entry(Some(source_top_ctx_id), thread_name, entry);
            },
            FromScriptMsg::TouchEventProcessed(result) => self
                .compositor_proxy
                .send(ToCompositorMsg::TouchEventProcessed(result)),
            FromScriptMsg::GetBrowsingContextInfo(pipeline_id, sender) => {
                let result = self
                    .pipelines
                    .get(&pipeline_id)
                    .and_then(|pipeline| self.browsing_contexts.get(&pipeline.browsing_context_id))
                    .map(|ctx| (ctx.id, ctx.parent_pipeline_id));
                if let Err(e) = sender.send(result) {
                    warn!(
                        "Sending reply to get browsing context info failed ({:?}).",
                        e
                    );
                }
            },
            FromScriptMsg::GetTopForBrowsingContext(browsing_context_id, sender) => {
                let result = self
                    .browsing_contexts
                    .get(&browsing_context_id)
                    .and_then(|bc| Some(bc.top_level_id));
                if let Err(e) = sender.send(result) {
                    warn!(
                        "Sending reply to get top for browsing context info failed ({:?}).",
                        e
                    );
                }
            },
            FromScriptMsg::GetChildBrowsingContextId(browsing_context_id, index, sender) => {
                let result = self
                    .browsing_contexts
                    .get(&browsing_context_id)
                    .and_then(|bc| self.pipelines.get(&bc.pipeline_id))
                    .and_then(|pipeline| pipeline.children.get(index))
                    .map(|maybe_bcid| *maybe_bcid);
                if let Err(e) = sender.send(result) {
                    warn!(
                        "Sending reply to get child browsing context ID failed ({:?}).",
                        e
                    );
                }
            },
            FromScriptMsg::RegisterServiceWorker(scope_things, scope) => {
                self.handle_register_serviceworker(scope_things, scope);
            },
            FromScriptMsg::ForwardDOMMessage(msg_vec, scope_url) => {
                if let Some(ref mgr) = self.swmanager_chan {
                    let _ = mgr.send(ServiceWorkerMsg::ForwardDOMMessage(msg_vec, scope_url));
                } else {
                    warn!("Unable to forward DOMMessage for postMessage call");
                }
            },
            FromScriptMsg::BroadcastStorageEvent(storage, url, key, old_value, new_value) => {
                self.handle_broadcast_storage_event(
                    source_pipeline_id,
                    storage,
                    url,
                    key,
                    old_value,
                    new_value,
                );
            },
        }
    }

    fn handle_request_from_layout(&mut self, message: FromLayoutMsg) {
        debug!("Constellation got {:?} message", message);
        match message {
            FromLayoutMsg::ChangeRunningAnimationsState(pipeline_id, animation_state) => {
                self.handle_change_running_animations_state(pipeline_id, animation_state)
            },
            // Layout sends new sizes for all subframes. This needs to be reflected by all
            // frame trees in the navigation context containing the subframe.
            FromLayoutMsg::IFrameSizes(iframe_sizes) => {
                self.handle_iframe_size_msg(iframe_sizes);
            },
            FromLayoutMsg::PendingPaintMetric(pipeline_id, epoch) => {
                self.handle_pending_paint_metric(pipeline_id, epoch);
            },
            FromLayoutMsg::ViewportConstrained(pipeline_id, constraints) => {
                self.handle_viewport_constrained_msg(pipeline_id, constraints);
            },
        }
    }

    fn handle_register_serviceworker(&self, scope_things: ScopeThings, scope: ServoUrl) {
        if let Some(ref mgr) = self.swmanager_chan {
            let _ = mgr.send(ServiceWorkerMsg::RegisterServiceWorker(scope_things, scope));
        } else {
            warn!("sending scope info to service worker manager failed");
        }
    }

    fn handle_broadcast_storage_event(
        &self,
        pipeline_id: PipelineId,
        storage: StorageType,
        url: ServoUrl,
        key: Option<String>,
        old_value: Option<String>,
        new_value: Option<String>,
    ) {
        let origin = url.origin();
        for pipeline in self.pipelines.values() {
            if (pipeline.id != pipeline_id) && (pipeline.url.origin() == origin) {
                let msg = ConstellationControlMsg::DispatchStorageEvent(
                    pipeline.id,
                    storage,
                    url.clone(),
                    key.clone(),
                    old_value.clone(),
                    new_value.clone(),
                );
                if let Err(err) = pipeline.event_loop.send(msg) {
                    warn!(
                        "Failed to broadcast storage event to pipeline {} ({:?}).",
                        pipeline.id, err
                    );
                }
            }
        }
    }

    fn handle_exit(&mut self) {
        // TODO: add a timer, which forces shutdown if threads aren't responsive.
        if self.shutting_down {
            return;
        }
        self.shutting_down = true;

        self.mem_profiler_chan.send(mem::ProfilerMsg::Exit);

        // Close the top-level browsing contexts
        let browsing_context_ids: Vec<BrowsingContextId> = self
            .browsing_contexts
            .values()
            .filter(|browsing_context| browsing_context.is_top_level())
            .map(|browsing_context| browsing_context.id)
            .collect();
        for browsing_context_id in browsing_context_ids {
            debug!(
                "Removing top-level browsing context {}.",
                browsing_context_id
            );
            self.close_browsing_context(browsing_context_id, ExitPipelineMode::Normal);
        }

        // Close any pending changes and pipelines
        while let Some(pending) = self.pending_changes.pop() {
            debug!(
                "Removing pending browsing context {}.",
                pending.browsing_context_id
            );
            self.close_browsing_context(pending.browsing_context_id, ExitPipelineMode::Normal);
            debug!("Removing pending pipeline {}.", pending.new_pipeline_id);
            self.close_pipeline(
                pending.new_pipeline_id,
                DiscardBrowsingContext::Yes,
                ExitPipelineMode::Normal,
            );
        }

        // In case there are browsing contexts which weren't attached, we close them.
        let browsing_context_ids: Vec<BrowsingContextId> =
            self.browsing_contexts.keys().cloned().collect();
        for browsing_context_id in browsing_context_ids {
            debug!(
                "Removing detached browsing context {}.",
                browsing_context_id
            );
            self.close_browsing_context(browsing_context_id, ExitPipelineMode::Normal);
        }

        // In case there are pipelines which weren't attached to the pipeline tree, we close them.
        let pipeline_ids: Vec<PipelineId> = self.pipelines.keys().cloned().collect();
        for pipeline_id in pipeline_ids {
            debug!("Removing detached pipeline {}.", pipeline_id);
            self.close_pipeline(
                pipeline_id,
                DiscardBrowsingContext::Yes,
                ExitPipelineMode::Normal,
            );
        }
    }

    fn handle_shutdown(&mut self) {
        // At this point, there are no active pipelines,
        // so we can safely block on other threads, without worrying about deadlock.
        // Channels to receive signals when threads are done exiting.
        let (core_sender, core_receiver) = ipc::channel().expect("Failed to create IPC channel!");
        let (storage_sender, storage_receiver) =
            ipc::channel().expect("Failed to create IPC channel!");

        debug!("Exiting core resource threads.");
        if let Err(e) = self
            .public_resource_threads
            .send(net_traits::CoreResourceMsg::Exit(core_sender))
        {
            warn!("Exit resource thread failed ({})", e);
        }

        if let Some(ref chan) = self.debugger_chan {
            debugger::shutdown_server(chan);
        }

        if let Some(ref chan) = self.devtools_chan {
            debug!("Exiting devtools.");
            let msg = DevtoolsControlMsg::FromChrome(ChromeToDevtoolsControlMsg::ServerExitMsg);
            if let Err(e) = chan.send(msg) {
                warn!("Exit devtools failed ({:?})", e);
            }
        }

        debug!("Exiting storage resource threads.");
        if let Err(e) = self
            .public_resource_threads
            .send(StorageThreadMsg::Exit(storage_sender))
        {
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

        debug!("Exiting Canvas Paint thread.");
        if let Err(e) = self.canvas_chan.send(CanvasMsg::Exit) {
            warn!("Exit Canvas Paint thread failed ({})", e);
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
        self.compositor_proxy
            .send(ToCompositorMsg::ShutdownComplete);
    }

    fn handle_pipeline_exited(&mut self, pipeline_id: PipelineId) {
        debug!("Pipeline {:?} exited.", pipeline_id);
        self.pipelines.remove(&pipeline_id);
    }

    fn handle_send_error(&mut self, pipeline_id: PipelineId, err: IpcError) {
        // Treat send error the same as receiving a panic message
        error!("Pipeline {} send error ({}).", pipeline_id, err);
        let top_level_browsing_context_id = self
            .pipelines
            .get(&pipeline_id)
            .map(|pipeline| pipeline.top_level_browsing_context_id);
        if let Some(top_level_browsing_context_id) = top_level_browsing_context_id {
            let reason = format!("Send failed ({})", err);
            self.handle_panic(top_level_browsing_context_id, reason, None);
        }
    }

    fn handle_panic(
        &mut self,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
        reason: String,
        backtrace: Option<String>,
    ) {
        if self.hard_fail {
            // It's quite difficult to make Servo exit cleanly if some threads have failed.
            // Hard fail exists for test runners so we crash and that's good enough.
            println!("Pipeline failed in hard-fail mode.  Crashing!");
            process::exit(1);
        }

        debug!(
            "Panic handler for top-level browsing context {}: {}.",
            top_level_browsing_context_id, reason
        );

        let browsing_context_id = BrowsingContextId::from(top_level_browsing_context_id);

        self.embedder_proxy.send((
            Some(top_level_browsing_context_id),
            EmbedderMsg::Panic(reason, backtrace),
        ));

        let browsing_context = match self.browsing_contexts.get(&browsing_context_id) {
            Some(context) => context,
            None => return warn!("failed browsing context is missing"),
        };
        let window_size = browsing_context.size;
        let pipeline_id = browsing_context.pipeline_id;
        let is_visible = browsing_context.is_visible;

        let pipeline = match self.pipelines.get(&pipeline_id) {
            Some(p) => p,
            None => return warn!("failed pipeline is missing"),
        };
        let pipeline_url = pipeline.url.clone();
        let opener = pipeline.opener;

        self.close_browsing_context_children(
            browsing_context_id,
            DiscardBrowsingContext::No,
            ExitPipelineMode::Force,
        );

        let failure_url = ServoUrl::parse("about:failure").expect("infallible");

        if pipeline_url == failure_url {
            return error!("about:failure failed");
        }

        warn!("creating replacement pipeline for about:failure");

        let new_pipeline_id = PipelineId::new();
        let load_data = LoadData::new(failure_url.origin(), failure_url, None, None, None);
        let sandbox = IFrameSandboxState::IFrameSandboxed;
        let is_private = false;
        self.new_pipeline(
            new_pipeline_id,
            browsing_context_id,
            top_level_browsing_context_id,
            None,
            opener,
            window_size,
            load_data,
            sandbox,
            is_private,
            is_visible,
        );
        self.add_pending_change(SessionHistoryChange {
            top_level_browsing_context_id: top_level_browsing_context_id,
            browsing_context_id: browsing_context_id,
            new_pipeline_id: new_pipeline_id,
            replace: None,
            new_browsing_context_info: None,
        });
    }

    fn handle_log_entry(
        &mut self,
        top_level_browsing_context_id: Option<TopLevelBrowsingContextId>,
        thread_name: Option<String>,
        entry: LogEntry,
    ) {
        debug!("Received log entry {:?}.", entry);
        match (entry, top_level_browsing_context_id) {
            (LogEntry::Panic(reason, backtrace), Some(top_level_browsing_context_id)) => {
                self.handle_panic(top_level_browsing_context_id, reason, Some(backtrace));
            },
            (LogEntry::Panic(reason, _), _) |
            (LogEntry::Error(reason), _) |
            (LogEntry::Warn(reason), _) => {
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
                    let _ = pipeline
                        .event_loop
                        .send(ConstellationControlMsg::WebVREvents(id, events.clone()));
                },
                None => warn!("constellation got webvr event for dead pipeline"),
            }
        }
    }

    fn forward_event(&mut self, destination_pipeline_id: PipelineId, event: CompositorEvent) {
        if let MouseButtonEvent(event_type, button, ..) = &event {
            match event_type {
                MouseEventType::MouseDown | MouseEventType::Click => {
                    self.pressed_mouse_buttons |= *button as u16;
                },
                MouseEventType::MouseUp => {
                    self.pressed_mouse_buttons &= !(*button as u16);
                },
            }
        }

        let event = match event {
            MouseButtonEvent(event_type, button, point, node_address, point_in_node, _) => {
                MouseButtonEvent(
                    event_type,
                    button,
                    point,
                    node_address,
                    point_in_node,
                    self.pressed_mouse_buttons,
                )
            },
            MouseMoveEvent(point, node_address, _) => {
                MouseMoveEvent(point, node_address, self.pressed_mouse_buttons)
            },
            _ => event,
        };

        let msg = ConstellationControlMsg::SendEvent(destination_pipeline_id, event);
        let result = match self.pipelines.get(&destination_pipeline_id) {
            None => {
                debug!(
                    "Pipeline {:?} got event after closure.",
                    destination_pipeline_id
                );
                return;
            },
            Some(pipeline) => pipeline.event_loop.send(msg),
        };
        if let Err(e) = result {
            self.handle_send_error(destination_pipeline_id, e);
        }
    }

    fn handle_new_top_level_browsing_context(
        &mut self,
        url: ServoUrl,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
    ) {
        let window_size = self.window_size.initial_viewport;
        let pipeline_id = PipelineId::new();
        let msg = (
            Some(top_level_browsing_context_id),
            EmbedderMsg::BrowserCreated(top_level_browsing_context_id),
        );
        self.embedder_proxy.send(msg);
        let browsing_context_id = BrowsingContextId::from(top_level_browsing_context_id);
        let load_data = LoadData::new(url.origin(), url, None, None, None);
        let sandbox = IFrameSandboxState::IFrameUnsandboxed;
        let is_private = false;
        let is_visible = true;

        // Register this new top-level browsing context id as a browser and set
        // its focused browsing context to be itself.
        self.browsers.insert(
            top_level_browsing_context_id,
            Browser {
                focused_browsing_context_id: browsing_context_id,
                session_history: JointSessionHistory::new(),
            },
        );

        self.new_pipeline(
            pipeline_id,
            browsing_context_id,
            top_level_browsing_context_id,
            None,
            None,
            window_size,
            load_data,
            sandbox,
            is_private,
            is_visible,
        );
        self.add_pending_change(SessionHistoryChange {
            top_level_browsing_context_id: top_level_browsing_context_id,
            browsing_context_id: browsing_context_id,
            new_pipeline_id: pipeline_id,
            replace: None,
            new_browsing_context_info: Some(NewBrowsingContextInfo {
                parent_pipeline_id: None,
                is_private: is_private,
                is_visible: is_visible,
            }),
        });
    }

    fn handle_close_top_level_browsing_context(
        &mut self,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
    ) {
        let browsing_context_id = BrowsingContextId::from(top_level_browsing_context_id);
        self.close_browsing_context(browsing_context_id, ExitPipelineMode::Normal);
        self.browsers.remove(&top_level_browsing_context_id);
        if self.active_browser_id == Some(top_level_browsing_context_id) {
            self.active_browser_id = None;
        }
    }

    fn handle_iframe_size_msg(&mut self, iframe_sizes: Vec<IFrameSizeMsg>) {
        for IFrameSizeMsg { data, type_ } in iframe_sizes {
            let window_size = WindowSizeData {
                initial_viewport: data.size,
                device_pixel_ratio: self.window_size.device_pixel_ratio,
            };

            self.resize_browsing_context(window_size, type_, data.id);
        }
    }

    fn handle_subframe_loaded(&mut self, pipeline_id: PipelineId) {
        let browsing_context_id = match self.pipelines.get(&pipeline_id) {
            Some(pipeline) => pipeline.browsing_context_id,
            None => return warn!("Subframe {} loaded after closure.", pipeline_id),
        };
        let parent_pipeline_id = match self.browsing_contexts.get(&browsing_context_id) {
            Some(browsing_context) => browsing_context.parent_pipeline_id,
            None => {
                return warn!(
                    "Subframe {} loaded in closed browsing context {}.",
                    pipeline_id, browsing_context_id,
                );
            },
        };
        let parent_pipeline_id = match parent_pipeline_id {
            Some(parent_pipeline_id) => parent_pipeline_id,
            None => return warn!("Subframe {} has no parent.", pipeline_id),
        };
        // https://html.spec.whatwg.org/multipage/#the-iframe-element:completely-loaded
        // When a Document in an iframe is marked as completely loaded,
        // the user agent must run the iframe load event steps.
        let msg = ConstellationControlMsg::DispatchIFrameLoadEvent {
            target: browsing_context_id,
            parent: parent_pipeline_id,
            child: pipeline_id,
        };
        let result = match self.pipelines.get(&parent_pipeline_id) {
            Some(parent) => parent.event_loop.send(msg),
            None => {
                return warn!(
                    "Parent {} browsing context loaded after closure.",
                    parent_pipeline_id
                );
            },
        };
        if let Err(e) = result {
            self.handle_send_error(parent_pipeline_id, e);
        }
    }

    fn handle_navigate_request(
        &self,
        id: PipelineId,
        request_builder: RequestBuilder,
        cancel_chan: IpcReceiver<()>,
    ) {
        let listener = NetworkListener::new(
            request_builder,
            id,
            self.public_resource_threads.clone(),
            self.network_listener_sender.clone(),
        );

        listener.initiate_fetch(Some(cancel_chan));
    }

    // The script thread associated with pipeline_id has loaded a URL in an
    // iframe via script. This will result in a new pipeline being spawned and
    // a child being added to the parent browsing context. This message is never
    // the result of a page navigation.
    fn handle_script_loaded_url_in_iframe_msg(&mut self, load_info: IFrameLoadInfoWithData) {
        let IFrameLoadInfo {
            parent_pipeline_id,
            browsing_context_id,
            top_level_browsing_context_id,
            new_pipeline_id,
            is_private,
            mut replace,
        } = load_info.info;

        // If no url is specified, reload.
        let old_pipeline = load_info
            .old_pipeline_id
            .and_then(|id| self.pipelines.get(&id));

        // Replacement enabled also takes into account whether the document is "completely loaded",
        // see https://html.spec.whatwg.org/multipage/#the-iframe-element:completely-loaded
        debug!("checking old pipeline? {:?}", load_info.old_pipeline_id);
        if let Some(old_pipeline) = old_pipeline {
            if !old_pipeline.completely_loaded {
                replace = HistoryEntryReplacement::Enabled;
            }
            debug!(
                "old pipeline is {}completely loaded",
                if old_pipeline.completely_loaded {
                    ""
                } else {
                    "not "
                }
            );
        }

        let load_data = load_info.load_data.unwrap_or_else(|| {
            let url = match old_pipeline {
                Some(old_pipeline) => old_pipeline.url.clone(),
                None => ServoUrl::parse("about:blank").expect("infallible"),
            };

            // TODO - loaddata here should have referrer info (not None, None)
            LoadData::new(url.origin(), url, Some(parent_pipeline_id), None, None)
        });

        let is_parent_private = {
            let parent_browsing_context_id = match self.pipelines.get(&parent_pipeline_id) {
                Some(pipeline) => pipeline.browsing_context_id,
                None => {
                    return warn!(
                        "Script loaded url in iframe {} in closed parent pipeline {}.",
                        browsing_context_id, parent_pipeline_id,
                    );
                },
            };
            let is_parent_private = match self.browsing_contexts.get(&parent_browsing_context_id) {
                Some(ctx) => ctx.is_private,
                None => {
                    return warn!(
                        "Script loaded url in iframe {} in closed parent browsing context {}.",
                        browsing_context_id, parent_browsing_context_id,
                    );
                },
            };
            is_parent_private
        };
        let is_private = is_private || is_parent_private;

        let browsing_context = match self.browsing_contexts.get(&browsing_context_id) {
            Some(ctx) => ctx,
            None => {
                return warn!(
                    "Script loaded url in iframe with closed browsing context {}.",
                    browsing_context_id,
                );
            },
        };

        let replace = match replace {
            HistoryEntryReplacement::Enabled => {
                Some(NeedsToReload::No(browsing_context.pipeline_id))
            },
            HistoryEntryReplacement::Disabled => None,
        };

        // https://github.com/rust-lang/rust/issues/59159
        let browsing_context_size = browsing_context.size;
        let browsing_context_is_visible = browsing_context.is_visible;

        // Create the new pipeline, attached to the parent and push to pending changes
        self.new_pipeline(
            new_pipeline_id,
            browsing_context_id,
            top_level_browsing_context_id,
            Some(parent_pipeline_id),
            None,
            browsing_context_size,
            load_data,
            load_info.sandbox,
            is_private,
            browsing_context_is_visible,
        );
        self.add_pending_change(SessionHistoryChange {
            top_level_browsing_context_id: top_level_browsing_context_id,
            browsing_context_id: browsing_context_id,
            new_pipeline_id: new_pipeline_id,
            replace: replace,
            // Browsing context for iframe already exists.
            new_browsing_context_info: None,
        });
    }

    fn handle_script_new_iframe(
        &mut self,
        load_info: IFrameLoadInfo,
        layout_sender: IpcSender<LayoutControlMsg>,
    ) {
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
        let load_data = LoadData::new(
            url.origin(),
            url.clone(),
            Some(parent_pipeline_id),
            None,
            None,
        );

        let (script_sender, parent_browsing_context_id) =
            match self.pipelines.get(&parent_pipeline_id) {
                Some(pipeline) => (pipeline.event_loop.clone(), pipeline.browsing_context_id),
                None => return warn!("Script loaded url in closed iframe {}.", parent_pipeline_id),
            };
        let (is_parent_private, is_parent_visible) =
            match self.browsing_contexts.get(&parent_browsing_context_id) {
                Some(ctx) => (ctx.is_private, ctx.is_visible),
                None => {
                    return warn!(
                        "New iframe {} loaded in closed parent browsing context {}.",
                        browsing_context_id, parent_browsing_context_id,
                    );
                },
            };
        let is_private = is_private || is_parent_private;
        let pipeline = Pipeline::new(
            new_pipeline_id,
            browsing_context_id,
            top_level_browsing_context_id,
            None,
            script_sender,
            layout_sender,
            self.compositor_proxy.clone(),
            url,
            is_parent_visible,
            load_data,
        );

        assert!(!self.pipelines.contains_key(&new_pipeline_id));
        self.pipelines.insert(new_pipeline_id, pipeline);
        self.add_pending_change(SessionHistoryChange {
            top_level_browsing_context_id: top_level_browsing_context_id,
            browsing_context_id: browsing_context_id,
            new_pipeline_id: new_pipeline_id,
            replace: None,
            // Browsing context for iframe doesn't exist yet.
            new_browsing_context_info: Some(NewBrowsingContextInfo {
                parent_pipeline_id: Some(parent_pipeline_id),
                is_private: is_private,
                is_visible: is_parent_visible,
            }),
        });
    }

    fn handle_script_new_auxiliary(
        &mut self,
        load_info: AuxiliaryBrowsingContextLoadInfo,
        layout_sender: IpcSender<LayoutControlMsg>,
    ) {
        let AuxiliaryBrowsingContextLoadInfo {
            opener_pipeline_id,
            new_top_level_browsing_context_id,
            new_browsing_context_id,
            new_pipeline_id,
        } = load_info;

        let url = ServoUrl::parse("about:blank").expect("infallible");

        // TODO: Referrer?
        let load_data = LoadData::new(url.origin(), url.clone(), None, None, None);

        let (script_sender, opener_browsing_context_id) =
            match self.pipelines.get(&opener_pipeline_id) {
                Some(pipeline) => (pipeline.event_loop.clone(), pipeline.browsing_context_id),
                None => {
                    return warn!(
                        "Auxiliary loaded url in closed iframe {}.",
                        opener_pipeline_id
                    );
                },
            };
        let (is_opener_private, is_opener_visible) =
            match self.browsing_contexts.get(&opener_browsing_context_id) {
                Some(ctx) => (ctx.is_private, ctx.is_visible),
                None => {
                    return warn!(
                        "New auxiliary {} loaded in closed opener browsing context {}.",
                        new_browsing_context_id, opener_browsing_context_id,
                    );
                },
            };
        let pipeline = Pipeline::new(
            new_pipeline_id,
            new_browsing_context_id,
            new_top_level_browsing_context_id,
            Some(opener_browsing_context_id),
            script_sender,
            layout_sender,
            self.compositor_proxy.clone(),
            url,
            is_opener_visible,
            load_data,
        );

        assert!(!self.pipelines.contains_key(&new_pipeline_id));
        self.pipelines.insert(new_pipeline_id, pipeline);
        self.browsers.insert(
            new_top_level_browsing_context_id,
            Browser {
                focused_browsing_context_id: new_browsing_context_id,
                session_history: JointSessionHistory::new(),
            },
        );
        self.add_pending_change(SessionHistoryChange {
            top_level_browsing_context_id: new_top_level_browsing_context_id,
            browsing_context_id: new_browsing_context_id,
            new_pipeline_id: new_pipeline_id,
            replace: None,
            new_browsing_context_info: Some(NewBrowsingContextInfo {
                // Auxiliary browsing contexts are always top-level.
                parent_pipeline_id: None,
                is_private: is_opener_private,
                is_visible: is_opener_visible,
            }),
        });
    }

    fn handle_pending_paint_metric(&self, pipeline_id: PipelineId, epoch: Epoch) {
        self.compositor_proxy
            .send(ToCompositorMsg::PendingPaintMetric(pipeline_id, epoch))
    }

    fn handle_set_cursor_msg(&mut self, cursor: Cursor) {
        self.embedder_proxy
            .send((None, EmbedderMsg::SetCursor(cursor)))
    }

    fn handle_change_running_animations_state(
        &mut self,
        pipeline_id: PipelineId,
        animation_state: AnimationState,
    ) {
        self.compositor_proxy
            .send(ToCompositorMsg::ChangeRunningAnimationsState(
                pipeline_id,
                animation_state,
            ))
    }

    fn handle_tick_animation(&mut self, pipeline_id: PipelineId, tick_type: AnimationTickType) {
        let result = match tick_type {
            AnimationTickType::Script => {
                let msg = ConstellationControlMsg::TickAllAnimations(pipeline_id);
                match self.pipelines.get(&pipeline_id) {
                    Some(pipeline) => pipeline.event_loop.send(msg),
                    None => {
                        return warn!("Pipeline {:?} got script tick after closure.", pipeline_id);
                    },
                }
            },
            AnimationTickType::Layout => {
                let msg = LayoutControlMsg::TickAnimations;
                match self.pipelines.get(&pipeline_id) {
                    Some(pipeline) => pipeline.layout_chan.send(msg),
                    None => {
                        return warn!("Pipeline {:?} got layout tick after closure.", pipeline_id);
                    },
                }
            },
        };
        if let Err(e) = result {
            self.handle_send_error(pipeline_id, e);
        }
    }

    /// Schedule a navigation(via load_url).
    /// 1: Ask the embedder for permission.
    /// 2: Store the details of the navigation, pending approval from the embedder.
    fn schedule_navigation(
        &mut self,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
        source_id: PipelineId,
        load_data: LoadData,
        replace: HistoryEntryReplacement,
    ) {
        match self.pending_approval_navigations.entry(source_id) {
            Entry::Occupied(_) => {
                return warn!(
                    "Pipeline {:?} tried to schedule a navigation while one is already pending.",
                    source_id
                );
            },
            Entry::Vacant(entry) => {
                let _ = entry.insert((load_data.clone(), replace));
            },
        };
        // Allow the embedder to handle the url itself
        let msg = (
            Some(top_level_browsing_context_id),
            EmbedderMsg::AllowNavigationRequest(source_id, load_data.url.clone()),
        );
        self.embedder_proxy.send(msg);
    }

    fn load_url(
        &mut self,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
        source_id: PipelineId,
        load_data: LoadData,
        replace: HistoryEntryReplacement,
    ) -> Option<PipelineId> {
        let replace_debug = match replace {
            HistoryEntryReplacement::Enabled => "",
            HistoryEntryReplacement::Disabled => "not",
        };
        debug!(
            "Loading {} in pipeline {}, {}replacing.",
            load_data.url, source_id, replace_debug
        );
        // If this load targets an iframe, its framing element may exist
        // in a separate script thread than the framed document that initiated
        // the new load. The framing element must be notified about the
        // requested change so it can update its internal state.
        //
        // If replace is true, the current entry is replaced instead of a new entry being added.
        let (browsing_context_id, opener) = match self.pipelines.get(&source_id) {
            Some(pipeline) => (pipeline.browsing_context_id, pipeline.opener),
            None => {
                warn!("Pipeline {} loaded after closure.", source_id);
                return None;
            },
        };
        let (window_size, pipeline_id, parent_pipeline_id, is_private, is_visible) =
            match self.browsing_contexts.get(&browsing_context_id) {
                Some(ctx) => (
                    ctx.size,
                    ctx.pipeline_id,
                    ctx.parent_pipeline_id,
                    ctx.is_private,
                    ctx.is_visible,
                ),
                None => {
                    // This should technically never happen (since `load_url` is
                    // only called on existing browsing contexts), but we prefer to
                    // avoid `expect`s or `unwrap`s in `Constellation` to ward
                    // against future changes that might break things.
                    warn!(
                        "Pipeline {} loaded url in closed browsing context {}.",
                        source_id, browsing_context_id,
                    );
                    return None;
                },
            };

        match parent_pipeline_id {
            Some(parent_pipeline_id) => {
                // Find the script thread for the pipeline containing the iframe
                // and issue an iframe load through there.
                let msg = ConstellationControlMsg::NavigateIframe(
                    parent_pipeline_id,
                    browsing_context_id,
                    load_data,
                    replace,
                );
                let result = match self.pipelines.get(&parent_pipeline_id) {
                    Some(parent_pipeline) => parent_pipeline.event_loop.send(msg),
                    None => {
                        warn!(
                            "Pipeline {:?} child loaded after closure",
                            parent_pipeline_id
                        );
                        return None;
                    },
                };
                if let Err(e) = result {
                    self.handle_send_error(parent_pipeline_id, e);
                }
                None
            },
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

                let replace = match replace {
                    HistoryEntryReplacement::Enabled => Some(NeedsToReload::No(pipeline_id)),
                    HistoryEntryReplacement::Disabled => None,
                };

                let new_pipeline_id = PipelineId::new();
                let sandbox = IFrameSandboxState::IFrameUnsandboxed;
                self.new_pipeline(
                    new_pipeline_id,
                    browsing_context_id,
                    top_level_browsing_context_id,
                    None,
                    opener,
                    window_size,
                    load_data,
                    sandbox,
                    is_private,
                    is_visible,
                );
                self.add_pending_change(SessionHistoryChange {
                    top_level_browsing_context_id: top_level_browsing_context_id,
                    browsing_context_id: browsing_context_id,
                    new_pipeline_id: new_pipeline_id,
                    replace,
                    // `load_url` is always invoked on an existing browsing context.
                    new_browsing_context_info: None,
                });
                Some(new_pipeline_id)
            },
        }
    }

    fn handle_abort_load_url_msg(&mut self, new_pipeline_id: PipelineId) {
        let pending_index = self
            .pending_changes
            .iter()
            .rposition(|change| change.new_pipeline_id == new_pipeline_id);

        // If it is found, remove it from the pending changes.
        if let Some(pending_index) = pending_index {
            self.pending_changes.remove(pending_index);
            self.close_pipeline(
                new_pipeline_id,
                DiscardBrowsingContext::No,
                ExitPipelineMode::Normal,
            );
        }
    }

    fn handle_load_start_msg(
        &mut self,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
        browsing_context_id: BrowsingContextId,
    ) {
        if browsing_context_id == top_level_browsing_context_id {
            // Notify embedder top level document started loading.
            self.embedder_proxy
                .send((Some(top_level_browsing_context_id), EmbedderMsg::LoadStart));
        }
    }

    fn handle_load_complete_msg(
        &mut self,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
        pipeline_id: PipelineId,
    ) {
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

        if let Some(pipeline) = self.pipelines.get_mut(&pipeline_id) {
            debug!("marking pipeline {:?} as loaded", pipeline_id);
            pipeline.completely_loaded = true;
        }

        // Notify the embedder that the TopLevelBrowsingContext current document
        // has finished loading.
        // We need to make sure the pipeline that has finished loading is the current
        // pipeline and that no pending pipeline will replace the current one.
        let pipeline_is_top_level_pipeline = self
            .browsing_contexts
            .get(&BrowsingContextId::from(top_level_browsing_context_id))
            .map(|ctx| ctx.pipeline_id == pipeline_id)
            .unwrap_or(false);
        if pipeline_is_top_level_pipeline {
            // Is there any pending pipeline that will replace the current top level pipeline
            let current_top_level_pipeline_will_be_replaced = self
                .pending_changes
                .iter()
                .any(|change| change.browsing_context_id == top_level_browsing_context_id);

            if !current_top_level_pipeline_will_be_replaced {
                // Notify embedder and compositor top level document finished loading.
                self.compositor_proxy
                    .send(ToCompositorMsg::LoadComplete(top_level_browsing_context_id));
                self.embedder_proxy.send((
                    Some(top_level_browsing_context_id),
                    EmbedderMsg::LoadComplete,
                ));
            }
        } else {
            self.handle_subframe_loaded(pipeline_id);
        }
    }

    fn handle_navigated_to_fragment(
        &mut self,
        pipeline_id: PipelineId,
        new_url: ServoUrl,
        replacement_enabled: HistoryEntryReplacement,
    ) {
        let (top_level_browsing_context_id, old_url) = match self.pipelines.get_mut(&pipeline_id) {
            Some(pipeline) => {
                let old_url = replace(&mut pipeline.url, new_url.clone());
                (pipeline.top_level_browsing_context_id, old_url)
            },
            None => {
                return warn!(
                    "Pipeline {} navigated to fragment after closure",
                    pipeline_id
                );
            },
        };

        match replacement_enabled {
            HistoryEntryReplacement::Disabled => {
                let diff = SessionHistoryDiff::HashDiff {
                    pipeline_reloader: NeedsToReload::No(pipeline_id),
                    new_url,
                    old_url,
                };
                self.get_joint_session_history(top_level_browsing_context_id)
                    .push_diff(diff);
                self.notify_history_changed(top_level_browsing_context_id);
            },
            HistoryEntryReplacement::Enabled => {},
        }
    }

    fn handle_traverse_history_msg(
        &mut self,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
        direction: TraversalDirection,
    ) {
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

                    for diff in session_history
                        .future
                        .drain(future_length - forward..)
                        .rev()
                    {
                        match diff {
                            SessionHistoryDiff::BrowsingContextDiff {
                                browsing_context_id,
                                ref new_reloader,
                                ..
                            } => {
                                browsing_context_changes
                                    .insert(browsing_context_id, new_reloader.clone());
                            },
                            SessionHistoryDiff::PipelineDiff {
                                ref pipeline_reloader,
                                new_history_state_id,
                                ref new_url,
                                ..
                            } => match *pipeline_reloader {
                                NeedsToReload::No(pipeline_id) => {
                                    pipeline_changes.insert(
                                        pipeline_id,
                                        (Some(new_history_state_id), new_url.clone()),
                                    );
                                },
                                NeedsToReload::Yes(pipeline_id, ..) => {
                                    url_to_load.insert(pipeline_id, new_url.clone());
                                },
                            },
                            SessionHistoryDiff::HashDiff {
                                ref pipeline_reloader,
                                ref new_url,
                                ..
                            } => match *pipeline_reloader {
                                NeedsToReload::No(pipeline_id) => {
                                    let state = pipeline_changes
                                        .get(&pipeline_id)
                                        .and_then(|change| change.0);
                                    pipeline_changes.insert(pipeline_id, (state, new_url.clone()));
                                },
                                NeedsToReload::Yes(pipeline_id, ..) => {
                                    url_to_load.insert(pipeline_id, new_url.clone());
                                },
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
                            SessionHistoryDiff::BrowsingContextDiff {
                                browsing_context_id,
                                ref old_reloader,
                                ..
                            } => {
                                browsing_context_changes
                                    .insert(browsing_context_id, old_reloader.clone());
                            },
                            SessionHistoryDiff::PipelineDiff {
                                ref pipeline_reloader,
                                old_history_state_id,
                                ref old_url,
                                ..
                            } => match *pipeline_reloader {
                                NeedsToReload::No(pipeline_id) => {
                                    pipeline_changes.insert(
                                        pipeline_id,
                                        (old_history_state_id, old_url.clone()),
                                    );
                                },
                                NeedsToReload::Yes(pipeline_id, ..) => {
                                    url_to_load.insert(pipeline_id, old_url.clone());
                                },
                            },
                            SessionHistoryDiff::HashDiff {
                                ref pipeline_reloader,
                                ref old_url,
                                ..
                            } => match *pipeline_reloader {
                                NeedsToReload::No(pipeline_id) => {
                                    let state = pipeline_changes
                                        .get(&pipeline_id)
                                        .and_then(|change| change.0);
                                    pipeline_changes.insert(pipeline_id, (state, old_url.clone()));
                                },
                                NeedsToReload::Yes(pipeline_id, ..) => {
                                    url_to_load.insert(pipeline_id, old_url.clone());
                                },
                            },
                        }
                        session_history.future.push(diff);
                    }
                },
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

    fn update_browsing_context(
        &mut self,
        browsing_context_id: BrowsingContextId,
        new_reloader: NeedsToReload,
    ) {
        let new_pipeline_id = match new_reloader {
            NeedsToReload::No(pipeline_id) => pipeline_id,
            NeedsToReload::Yes(pipeline_id, load_data) => {
                debug!(
                    "Reloading document {} in browsing context {}.",
                    pipeline_id, browsing_context_id
                );

                // TODO: Save the sandbox state so it can be restored here.
                let sandbox = IFrameSandboxState::IFrameUnsandboxed;
                let (
                    top_level_id,
                    old_pipeline_id,
                    parent_pipeline_id,
                    window_size,
                    is_private,
                    is_visible,
                ) = match self.browsing_contexts.get(&browsing_context_id) {
                    Some(ctx) => (
                        ctx.top_level_id,
                        ctx.pipeline_id,
                        ctx.parent_pipeline_id,
                        ctx.size,
                        ctx.is_private,
                        ctx.is_visible,
                    ),
                    None => return warn!("No browsing context to traverse!"),
                };
                let opener = match self.pipelines.get(&old_pipeline_id) {
                    Some(pipeline) => pipeline.opener,
                    None => None,
                };
                let new_pipeline_id = PipelineId::new();
                self.new_pipeline(
                    new_pipeline_id,
                    browsing_context_id,
                    top_level_id,
                    parent_pipeline_id,
                    opener,
                    window_size,
                    load_data.clone(),
                    sandbox,
                    is_private,
                    is_visible,
                );
                self.add_pending_change(SessionHistoryChange {
                    top_level_browsing_context_id: top_level_id,
                    browsing_context_id: browsing_context_id,
                    new_pipeline_id: new_pipeline_id,
                    replace: Some(NeedsToReload::Yes(pipeline_id, load_data)),
                    // Browsing context must exist at this point.
                    new_browsing_context_info: None,
                });
                return;
            },
        };

        let (old_pipeline_id, parent_pipeline_id, top_level_id) =
            match self.browsing_contexts.get_mut(&browsing_context_id) {
                Some(browsing_context) => {
                    let old_pipeline_id = browsing_context.pipeline_id;
                    browsing_context.update_current_entry(new_pipeline_id);
                    (
                        old_pipeline_id,
                        browsing_context.parent_pipeline_id,
                        browsing_context.top_level_id,
                    )
                },
                None => {
                    return warn!(
                        "Browsing context {} was closed during traversal",
                        browsing_context_id
                    );
                },
            };

        if let Some(old_pipeline) = self.pipelines.get(&old_pipeline_id) {
            old_pipeline.notify_visibility(false);
        }
        if let Some(new_pipeline) = self.pipelines.get(&new_pipeline_id) {
            new_pipeline.notify_visibility(true);
        }

        self.update_activity(old_pipeline_id);
        self.update_activity(new_pipeline_id);

        if let Some(parent_pipeline_id) = parent_pipeline_id {
            let msg = ConstellationControlMsg::UpdatePipelineId(
                parent_pipeline_id,
                browsing_context_id,
                top_level_id,
                new_pipeline_id,
                UpdatePipelineIdReason::Traversal,
            );
            let result = match self.pipelines.get(&parent_pipeline_id) {
                None => {
                    return warn!(
                        "Pipeline {} child traversed after closure",
                        parent_pipeline_id
                    );
                },
                Some(pipeline) => pipeline.event_loop.send(msg),
            };
            if let Err(e) = result {
                self.handle_send_error(parent_pipeline_id, e);
            }
        }
    }

    fn update_pipeline(
        &mut self,
        pipeline_id: PipelineId,
        history_state_id: Option<HistoryStateId>,
        url: ServoUrl,
    ) {
        let result = match self.pipelines.get_mut(&pipeline_id) {
            None => {
                return warn!(
                    "Pipeline {} history state updated after closure",
                    pipeline_id
                );
            },
            Some(pipeline) => {
                let msg = ConstellationControlMsg::UpdateHistoryState(
                    pipeline_id,
                    history_state_id,
                    url.clone(),
                );
                pipeline.history_state_id = history_state_id;
                pipeline.url = url;
                pipeline.event_loop.send(msg)
            },
        };
        if let Err(e) = result {
            self.handle_send_error(pipeline_id, e);
        }
    }

    fn handle_joint_session_history_length(
        &self,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
        sender: IpcSender<u32>,
    ) {
        let length = self
            .browsers
            .get(&top_level_browsing_context_id)
            .map(|browser| browser.session_history.history_length())
            .unwrap_or(1);
        let _ = sender.send(length as u32);
    }

    fn handle_push_history_state_msg(
        &mut self,
        pipeline_id: PipelineId,
        history_state_id: HistoryStateId,
        url: ServoUrl,
    ) {
        let (top_level_browsing_context_id, old_state_id, old_url) =
            match self.pipelines.get_mut(&pipeline_id) {
                Some(pipeline) => {
                    let old_history_state_id = pipeline.history_state_id;
                    let old_url = replace(&mut pipeline.url, url.clone());
                    pipeline.history_state_id = Some(history_state_id);
                    pipeline.history_states.insert(history_state_id);
                    (
                        pipeline.top_level_browsing_context_id,
                        old_history_state_id,
                        old_url,
                    )
                },
                None => {
                    return warn!(
                        "Push history state {} for closed pipeline {}",
                        history_state_id, pipeline_id
                    );
                },
            };

        let diff = SessionHistoryDiff::PipelineDiff {
            pipeline_reloader: NeedsToReload::No(pipeline_id),
            new_history_state_id: history_state_id,
            new_url: url,
            old_history_state_id: old_state_id,
            old_url: old_url,
        };
        self.get_joint_session_history(top_level_browsing_context_id)
            .push_diff(diff);
        self.notify_history_changed(top_level_browsing_context_id);
    }

    fn handle_replace_history_state_msg(
        &mut self,
        pipeline_id: PipelineId,
        history_state_id: HistoryStateId,
        url: ServoUrl,
    ) {
        let top_level_browsing_context_id = match self.pipelines.get_mut(&pipeline_id) {
            Some(pipeline) => {
                pipeline.history_state_id = Some(history_state_id);
                pipeline.url = url.clone();
                pipeline.top_level_browsing_context_id
            },
            None => {
                return warn!(
                    "Replace history state {} for closed pipeline {}",
                    history_state_id, pipeline_id
                );
            },
        };

        let session_history = self.get_joint_session_history(top_level_browsing_context_id);
        session_history.replace_history_state(pipeline_id, history_state_id, url);
    }

    fn handle_key_msg(&mut self, event: KeyboardEvent) {
        // Send to the focused browsing contexts' current pipeline.  If it
        // doesn't exist, fall back to sending to the compositor.
        let focused_browsing_context_id = self
            .active_browser_id
            .and_then(|browser_id| self.browsers.get(&browser_id))
            .map(|browser| browser.focused_browsing_context_id);
        match focused_browsing_context_id {
            Some(browsing_context_id) => {
                let event = CompositorEvent::KeyboardEvent(event);
                let pipeline_id = match self.browsing_contexts.get(&browsing_context_id) {
                    Some(ctx) => ctx.pipeline_id,
                    None => {
                        return warn!(
                            "Got key event for nonexistent browsing context {}.",
                            browsing_context_id,
                        );
                    },
                };
                let msg = ConstellationControlMsg::SendEvent(pipeline_id, event);
                let result = match self.pipelines.get(&pipeline_id) {
                    Some(pipeline) => pipeline.event_loop.send(msg),
                    None => {
                        return debug!("Pipeline {:?} got key event after closure.", pipeline_id);
                    },
                };
                if let Err(e) = result {
                    self.handle_send_error(pipeline_id, e);
                }
            },
            None => {
                let event = (None, EmbedderMsg::Keyboard(event));
                self.embedder_proxy.send(event);
            },
        }
    }

    fn handle_reload_msg(&mut self, top_level_browsing_context_id: TopLevelBrowsingContextId) {
        let browsing_context_id = BrowsingContextId::from(top_level_browsing_context_id);
        let pipeline_id = match self.browsing_contexts.get(&browsing_context_id) {
            Some(browsing_context) => browsing_context.pipeline_id,
            None => {
                return warn!(
                    "Browsing context {} got reload event after closure.",
                    browsing_context_id
                );
            },
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

    fn handle_post_message_msg(
        &mut self,
        browsing_context_id: BrowsingContextId,
        source_pipeline: PipelineId,
        origin: Option<ImmutableOrigin>,
        data: Vec<u8>,
    ) {
        let pipeline_id = match self.browsing_contexts.get(&browsing_context_id) {
            None => {
                return warn!(
                    "PostMessage to closed browsing_context {}.",
                    browsing_context_id
                );
            },
            Some(browsing_context) => browsing_context.pipeline_id,
        };
        let source_browsing_context = match self.pipelines.get(&source_pipeline) {
            Some(pipeline) => pipeline.top_level_browsing_context_id,
            None => return warn!("PostMessage from closed pipeline {:?}", source_pipeline),
        };
        let msg = ConstellationControlMsg::PostMessage {
            target: pipeline_id,
            source: source_pipeline,
            source_browsing_context: source_browsing_context,
            target_origin: origin,
            data,
        };
        let result = match self.pipelines.get(&pipeline_id) {
            Some(pipeline) => pipeline.event_loop.send(msg),
            None => return warn!("postMessage to closed pipeline {}.", pipeline_id),
        };
        if let Err(e) = result {
            self.handle_send_error(pipeline_id, e);
        }
    }

    fn handle_get_pipeline(
        &mut self,
        browsing_context_id: BrowsingContextId,
        resp_chan: IpcSender<Option<PipelineId>>,
    ) {
        let current_pipeline_id = self
            .browsing_contexts
            .get(&browsing_context_id)
            .map(|browsing_context| browsing_context.pipeline_id);
        let pipeline_id_loaded = self
            .pending_changes
            .iter()
            .rev()
            .find(|x| x.browsing_context_id == browsing_context_id)
            .map(|x| x.new_pipeline_id)
            .or(current_pipeline_id);
        if let Err(e) = resp_chan.send(pipeline_id_loaded) {
            warn!("Failed get_pipeline response ({}).", e);
        }
    }

    fn handle_get_browsing_context(
        &mut self,
        pipeline_id: PipelineId,
        resp_chan: IpcSender<Option<BrowsingContextId>>,
    ) {
        let browsing_context_id = self
            .pipelines
            .get(&pipeline_id)
            .map(|pipeline| pipeline.browsing_context_id);
        if let Err(e) = resp_chan.send(browsing_context_id) {
            warn!("Failed get_browsing_context response ({}).", e);
        }
    }

    fn handle_focus_msg(&mut self, pipeline_id: PipelineId) {
        let (browsing_context_id, top_level_browsing_context_id) =
            match self.pipelines.get(&pipeline_id) {
                Some(pipeline) => (
                    pipeline.browsing_context_id,
                    pipeline.top_level_browsing_context_id,
                ),
                None => return warn!("Pipeline {:?} focus parent after closure.", pipeline_id),
            };

        // Update the focused browsing context in its browser in `browsers`.
        match self.browsers.get_mut(&top_level_browsing_context_id) {
            Some(browser) => {
                browser.focused_browsing_context_id = browsing_context_id;
            },
            None => {
                return warn!(
                    "Browser {} for focus msg does not exist",
                    top_level_browsing_context_id
                );
            },
        };

        // Focus parent iframes recursively
        self.focus_parent_pipeline(browsing_context_id);
    }

    fn focus_parent_pipeline(&mut self, browsing_context_id: BrowsingContextId) {
        let parent_pipeline_id = match self.browsing_contexts.get(&browsing_context_id) {
            Some(ctx) => ctx.parent_pipeline_id,
            None => {
                return warn!(
                    "Browsing context {:?} focus parent after closure.",
                    browsing_context_id
                );
            },
        };
        let parent_pipeline_id = match parent_pipeline_id {
            Some(parent_id) => parent_id,
            None => {
                return debug!(
                    "Browsing context {:?} focus has no parent.",
                    browsing_context_id
                );
            },
        };

        // Send a message to the parent of the provided browsing context (if it
        // exists) telling it to mark the iframe element as focused.
        let msg = ConstellationControlMsg::FocusIFrame(parent_pipeline_id, browsing_context_id);
        let (result, parent_browsing_context_id) = match self.pipelines.get(&parent_pipeline_id) {
            Some(pipeline) => {
                let result = pipeline.event_loop.send(msg);
                (result, pipeline.browsing_context_id)
            },
            None => return warn!("Pipeline {:?} focus after closure.", parent_pipeline_id),
        };
        if let Err(e) = result {
            self.handle_send_error(parent_pipeline_id, e);
        }
        self.focus_parent_pipeline(parent_browsing_context_id);
    }

    fn handle_remove_iframe_msg(
        &mut self,
        browsing_context_id: BrowsingContextId,
    ) -> Vec<PipelineId> {
        let result = self
            .all_descendant_browsing_contexts_iter(browsing_context_id)
            .flat_map(|browsing_context| browsing_context.pipelines.iter().cloned())
            .collect();
        self.close_browsing_context(browsing_context_id, ExitPipelineMode::Normal);
        result
    }

    fn handle_visibility_change_complete(&mut self, pipeline_id: PipelineId, visibility: bool) {
        let browsing_context_id = match self.pipelines.get(&pipeline_id) {
            Some(pipeline) => pipeline.browsing_context_id,
            None => return warn!("Visibity change for closed pipeline {:?}.", pipeline_id),
        };
        let parent_pipeline_id = match self.browsing_contexts.get(&browsing_context_id) {
            Some(ctx) => ctx.parent_pipeline_id,
            None => {
                return warn!(
                    "Visibility change for closed browsing context {:?}.",
                    pipeline_id
                );
            },
        };

        if let Some(parent_pipeline_id) = parent_pipeline_id {
            let visibility_msg = ConstellationControlMsg::NotifyVisibilityChange(
                parent_pipeline_id,
                browsing_context_id,
                visibility,
            );
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
        size: Size2D<u64>,
        response_sender: IpcSender<(IpcSender<CanvasMsg>, CanvasId)>,
    ) {
        let webrender_api = self.webrender_api_sender.clone();
        let sender = self.canvas_chan.clone();
        let (canvas_id_sender, canvas_id_receiver) =
            ipc::channel::<CanvasId>().expect("ipc channel failure");

        if let Err(e) = sender.send(CanvasMsg::Create(
            canvas_id_sender,
            size,
            webrender_api,
            self.enable_canvas_antialiasing,
        )) {
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
                self.embedder_proxy.send((
                    Some(top_level_browsing_context_id),
                    EmbedderMsg::ResizeTo(size),
                ));
            },
            WebDriverCommandMsg::LoadUrl(top_level_browsing_context_id, load_data, reply) => {
                self.load_url_for_webdriver(
                    top_level_browsing_context_id,
                    load_data,
                    reply,
                    HistoryEntryReplacement::Disabled,
                );
            },
            WebDriverCommandMsg::Refresh(top_level_browsing_context_id, reply) => {
                let browsing_context_id = BrowsingContextId::from(top_level_browsing_context_id);
                let pipeline_id = match self.browsing_contexts.get(&browsing_context_id) {
                    Some(browsing_context) => browsing_context.pipeline_id,
                    None => {
                        return warn!(
                            "Browsing context {} Refresh after closure.",
                            browsing_context_id
                        );
                    },
                };
                let load_data = match self.pipelines.get(&pipeline_id) {
                    Some(pipeline) => pipeline.load_data.clone(),
                    None => return warn!("Pipeline {} refresh after closure.", pipeline_id),
                };
                self.load_url_for_webdriver(
                    top_level_browsing_context_id,
                    load_data,
                    reply,
                    HistoryEntryReplacement::Enabled,
                );
            },
            WebDriverCommandMsg::ScriptCommand(browsing_context_id, cmd) => {
                let pipeline_id = match self.browsing_contexts.get(&browsing_context_id) {
                    Some(browsing_context) => browsing_context.pipeline_id,
                    None => {
                        return warn!(
                            "Browsing context {} ScriptCommand after closure.",
                            browsing_context_id
                        );
                    },
                };
                let control_msg = ConstellationControlMsg::WebDriverScriptCommand(pipeline_id, cmd);
                let result = match self.pipelines.get(&pipeline_id) {
                    Some(pipeline) => pipeline.event_loop.send(control_msg),
                    None => {
                        return warn!("Pipeline {:?} ScriptCommand after closure.", pipeline_id)
                    },
                };
                if let Err(e) = result {
                    self.handle_send_error(pipeline_id, e);
                }
            },
            WebDriverCommandMsg::SendKeys(browsing_context_id, cmd) => {
                let pipeline_id = match self.browsing_contexts.get(&browsing_context_id) {
                    Some(browsing_context) => browsing_context.pipeline_id,
                    None => {
                        return warn!(
                            "Browsing context {} SendKeys after closure.",
                            browsing_context_id
                        );
                    },
                };
                let event_loop = match self.pipelines.get(&pipeline_id) {
                    Some(pipeline) => pipeline.event_loop.clone(),
                    None => return warn!("Pipeline {} SendKeys after closure.", pipeline_id),
                };
                for event in cmd {
                    let event = match event {
                        WebDriverInputEvent::Keyboard(event) => {
                            CompositorEvent::KeyboardEvent(event)
                        },
                        WebDriverInputEvent::Composition(event) => {
                            CompositorEvent::CompositionEvent(event)
                        },
                    };
                    let control_msg = ConstellationControlMsg::SendEvent(pipeline_id, event);
                    if let Err(e) = event_loop.send(control_msg) {
                        return self.handle_send_error(pipeline_id, e);
                    }
                }
            },
            WebDriverCommandMsg::TakeScreenshot(_, reply) => {
                self.compositor_proxy
                    .send(ToCompositorMsg::CreatePng(reply));
            },
        }
    }

    fn notify_history_changed(&self, top_level_browsing_context_id: TopLevelBrowsingContextId) {
        // Send a flat projection of the history to embedder.
        // The final vector is a concatenation of the LoadData of the past
        // entries, the current entry and the future entries.
        // LoadData of inner frames are ignored and replaced with the LoadData
        // of the parent.

        let session_history = match self.browsers.get(&top_level_browsing_context_id) {
            Some(browser) => &browser.session_history,
            None => {
                return warn!(
                    "Session history does not exist for {}",
                    top_level_browsing_context_id
                );
            },
        };

        let browsing_context_id = BrowsingContextId::from(top_level_browsing_context_id);
        let browsing_context = match self.browsing_contexts.get(&browsing_context_id) {
            Some(browsing_context) => browsing_context,
            None => {
                return warn!(
                    "notify_history_changed error after top-level browsing context closed."
                );
            },
        };

        let current_load_data = match self.pipelines.get(&browsing_context.pipeline_id) {
            Some(pipeline) => pipeline.load_data.clone(),
            None => {
                return warn!(
                    "Pipeline {} refresh after closure.",
                    browsing_context.pipeline_id
                );
            },
        };

        // If LoadData was ignored, use the LoadData of the previous SessionHistoryEntry, which
        // is the LoadData of the parent browsing context.
        let resolve_load_data_future =
            |previous_load_data: &mut LoadData, diff: &SessionHistoryDiff| match *diff {
                SessionHistoryDiff::BrowsingContextDiff {
                    browsing_context_id,
                    ref new_reloader,
                    ..
                } => {
                    if browsing_context_id == top_level_browsing_context_id {
                        let load_data = match *new_reloader {
                            NeedsToReload::No(pipeline_id) => {
                                match self.pipelines.get(&pipeline_id) {
                                    Some(pipeline) => pipeline.load_data.clone(),
                                    None => previous_load_data.clone(),
                                }
                            },
                            NeedsToReload::Yes(_, ref load_data) => load_data.clone(),
                        };
                        *previous_load_data = load_data.clone();
                        Some(load_data)
                    } else {
                        Some(previous_load_data.clone())
                    }
                },
                _ => Some(previous_load_data.clone()),
            };

        let resolve_load_data_past =
            |previous_load_data: &mut LoadData, diff: &SessionHistoryDiff| match *diff {
                SessionHistoryDiff::BrowsingContextDiff {
                    browsing_context_id,
                    ref old_reloader,
                    ..
                } => {
                    if browsing_context_id == top_level_browsing_context_id {
                        let load_data = match *old_reloader {
                            NeedsToReload::No(pipeline_id) => {
                                match self.pipelines.get(&pipeline_id) {
                                    Some(pipeline) => pipeline.load_data.clone(),
                                    None => previous_load_data.clone(),
                                }
                            },
                            NeedsToReload::Yes(_, ref load_data) => load_data.clone(),
                        };
                        *previous_load_data = load_data.clone();
                        Some(load_data)
                    } else {
                        Some(previous_load_data.clone())
                    }
                },
                _ => Some(previous_load_data.clone()),
            };

        let mut entries: Vec<LoadData> = session_history
            .past
            .iter()
            .rev()
            .scan(current_load_data.clone(), &resolve_load_data_past)
            .collect();

        entries.reverse();

        let current_index = entries.len();

        entries.push(current_load_data.clone());

        entries.extend(
            session_history
                .future
                .iter()
                .rev()
                .scan(current_load_data, &resolve_load_data_future),
        );
        let urls = entries.iter().map(|entry| entry.url.clone()).collect();
        let msg = (
            Some(top_level_browsing_context_id),
            EmbedderMsg::HistoryChanged(urls, current_index),
        );
        self.embedder_proxy.send(msg);
    }

    fn load_url_for_webdriver(
        &mut self,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
        load_data: LoadData,
        reply: IpcSender<webdriver_msg::LoadStatus>,
        replace: HistoryEntryReplacement,
    ) {
        let browsing_context_id = BrowsingContextId::from(top_level_browsing_context_id);
        let pipeline_id = match self.browsing_contexts.get(&browsing_context_id) {
            Some(browsing_context) => browsing_context.pipeline_id,
            None => {
                return warn!(
                    "Webdriver load for closed browsing context {}.",
                    browsing_context_id
                );
            },
        };
        if let Some(new_pipeline_id) = self.load_url(
            top_level_browsing_context_id,
            pipeline_id,
            load_data,
            replace,
        ) {
            self.webdriver.load_channel = Some((new_pipeline_id, reply));
        }
    }

    fn change_session_history(&mut self, change: SessionHistoryChange) {
        debug!(
            "Setting browsing context {} to be pipeline {}.",
            change.browsing_context_id, change.new_pipeline_id
        );

        // If the currently focused browsing context is a child of the browsing
        // context in which the page is being loaded, then update the focused
        // browsing context to be the one where the page is being loaded.
        if self.focused_browsing_context_is_descendant_of(change.browsing_context_id) {
            self.browsers
                .entry(change.top_level_browsing_context_id)
                .and_modify(|browser| {
                    browser.focused_browsing_context_id = change.browsing_context_id
                });
        }

        let (old_pipeline_id, top_level_id) =
            match self.browsing_contexts.get_mut(&change.browsing_context_id) {
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
                },
            };

        match old_pipeline_id {
            None => {
                let new_context_info = match change.new_browsing_context_info {
                    Some(info) => info,
                    None => {
                        return warn!(
                            "No NewBrowsingContextInfo for browsing context {}",
                            change.browsing_context_id,
                        );
                    },
                };
                self.new_browsing_context(
                    change.browsing_context_id,
                    change.top_level_browsing_context_id,
                    change.new_pipeline_id,
                    new_context_info.parent_pipeline_id,
                    self.window_size.initial_viewport, //XXXjdm is this valid?
                    new_context_info.is_private,
                    new_context_info.is_visible,
                );
                self.update_activity(change.new_pipeline_id);
                self.notify_history_changed(change.top_level_browsing_context_id);
            },
            Some(old_pipeline_id) => {
                if let Some(pipeline) = self.pipelines.get(&old_pipeline_id) {
                    pipeline.notify_visibility(false);
                }

                // https://html.spec.whatwg.org/multipage/#unload-a-document
                self.unload_document(old_pipeline_id);
                // Deactivate the old pipeline, and activate the new one.
                let (pipelines_to_close, states_to_close) = if let Some(replace_reloader) =
                    change.replace
                {
                    self.get_joint_session_history(change.top_level_browsing_context_id)
                        .replace_reloader(
                            replace_reloader.clone(),
                            NeedsToReload::No(change.new_pipeline_id),
                        );

                    match replace_reloader {
                        NeedsToReload::No(pipeline_id) => (Some(vec![pipeline_id]), None),
                        NeedsToReload::Yes(..) => (None, None),
                    }
                } else {
                    let diff = SessionHistoryDiff::BrowsingContextDiff {
                        browsing_context_id: change.browsing_context_id,
                        new_reloader: NeedsToReload::No(change.new_pipeline_id),
                        old_reloader: NeedsToReload::No(old_pipeline_id),
                    };

                    let mut pipelines_to_close = vec![];
                    let mut states_to_close = HashMap::new();

                    let diffs_to_close = self
                        .get_joint_session_history(change.top_level_browsing_context_id)
                        .push_diff(diff);

                    for diff in diffs_to_close {
                        match diff {
                            SessionHistoryDiff::BrowsingContextDiff { new_reloader, .. } => {
                                if let Some(pipeline_id) = new_reloader.alive_pipeline_id() {
                                    pipelines_to_close.push(pipeline_id);
                                }
                            },
                            SessionHistoryDiff::PipelineDiff {
                                pipeline_reloader,
                                new_history_state_id,
                                ..
                            } => {
                                if let Some(pipeline_id) = pipeline_reloader.alive_pipeline_id() {
                                    let states =
                                        states_to_close.entry(pipeline_id).or_insert(Vec::new());
                                    states.push(new_history_state_id);
                                }
                            },
                            _ => {},
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
                            None => {
                                return warn!(
                                    "Pipeline {} removed history states after closure",
                                    pipeline_id
                                );
                            },
                            Some(pipeline) => pipeline.event_loop.send(msg),
                        };
                        if let Err(e) = result {
                            self.handle_send_error(pipeline_id, e);
                        }
                    }
                }

                if let Some(pipelines_to_close) = pipelines_to_close {
                    for pipeline_id in pipelines_to_close {
                        self.close_pipeline(
                            pipeline_id,
                            DiscardBrowsingContext::No,
                            ExitPipelineMode::Normal,
                        );
                    }
                }

                self.notify_history_changed(change.top_level_browsing_context_id);
            },
        }

        if let Some(top_level_id) = top_level_id {
            self.trim_history(top_level_id);
        }

        self.notify_history_changed(change.top_level_browsing_context_id);
        self.update_frame_tree_if_active(change.top_level_browsing_context_id);
    }

    fn focused_browsing_context_is_descendant_of(
        &self,
        browsing_context_id: BrowsingContextId,
    ) -> bool {
        let focused_browsing_context_id = self
            .active_browser_id
            .and_then(|browser_id| self.browsers.get(&browser_id))
            .map(|browser| browser.focused_browsing_context_id);
        focused_browsing_context_id.map_or(false, |focus_ctx_id| {
            focus_ctx_id == browsing_context_id ||
                self.fully_active_descendant_browsing_contexts_iter(browsing_context_id)
                    .any(|nested_ctx| nested_ctx.id == focus_ctx_id)
        })
    }

    fn trim_history(&mut self, top_level_browsing_context_id: TopLevelBrowsingContextId) {
        let pipelines_to_evict = {
            let session_history = self.get_joint_session_history(top_level_browsing_context_id);

            let history_length = pref!(session_history.max_length) as usize;

            // The past is stored with older entries at the front.
            // We reverse the iter so that newer entries are at the front and then
            // skip _n_ entries and evict the remaining entries.
            let mut pipelines_to_evict = session_history
                .past
                .iter()
                .rev()
                .map(|diff| diff.alive_old_pipeline())
                .skip(history_length)
                .filter_map(|maybe_pipeline| maybe_pipeline)
                .collect::<Vec<_>>();

            // The future is stored with oldest entries front, so we must
            // reverse the iterator like we do for the `past`.
            pipelines_to_evict.extend(
                session_history
                    .future
                    .iter()
                    .rev()
                    .map(|diff| diff.alive_new_pipeline())
                    .skip(history_length)
                    .filter_map(|maybe_pipeline| maybe_pipeline),
            );

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
            self.close_pipeline(
                evicted_id,
                DiscardBrowsingContext::No,
                ExitPipelineMode::Normal,
            );
        }

        let session_history = self.get_joint_session_history(top_level_browsing_context_id);

        for (alive_id, dead) in dead_pipelines {
            session_history.replace_reloader(NeedsToReload::No(alive_id), dead);
        }
    }

    fn handle_activate_document_msg(&mut self, pipeline_id: PipelineId) {
        debug!("Document ready to activate {}", pipeline_id);

        // Find the pending change whose new pipeline id is pipeline_id.
        let pending_index = self
            .pending_changes
            .iter()
            .rposition(|change| change.new_pipeline_id == pipeline_id);

        // If it is found, remove it from the pending changes, and make it
        // the active document of its frame.
        if let Some(pending_index) = pending_index {
            let change = self.pending_changes.swap_remove(pending_index);
            // Notify the parent (if there is one).
            let parent_pipeline_id = match change.new_browsing_context_info {
                // This will be a new browsing context.
                Some(ref info) => info.parent_pipeline_id,
                // This is an existing browsing context.
                None => match self.browsing_contexts.get(&change.browsing_context_id) {
                    Some(ctx) => ctx.parent_pipeline_id,
                    None => {
                        return warn!(
                            "Activated document {} after browsing context {} closure.",
                            change.new_pipeline_id, change.browsing_context_id,
                        );
                    },
                },
            };
            if let Some(parent_pipeline_id) = parent_pipeline_id {
                if let Some(parent_pipeline) = self.pipelines.get(&parent_pipeline_id) {
                    let msg = ConstellationControlMsg::UpdatePipelineId(
                        parent_pipeline_id,
                        change.browsing_context_id,
                        change.top_level_browsing_context_id,
                        pipeline_id,
                        UpdatePipelineIdReason::Navigation,
                    );
                    let _ = parent_pipeline.event_loop.send(msg);
                }
            }
            self.change_session_history(change);
        }
    }

    /// Called when the window is resized.
    fn handle_window_size_msg(
        &mut self,
        top_level_browsing_context_id: Option<TopLevelBrowsingContextId>,
        new_size: WindowSizeData,
        size_type: WindowSizeType,
    ) {
        debug!(
            "handle_window_size_msg: {:?}",
            new_size.initial_viewport.to_untyped()
        );

        if let Some(top_level_browsing_context_id) = top_level_browsing_context_id {
            let browsing_context_id = BrowsingContextId::from(top_level_browsing_context_id);
            self.resize_browsing_context(new_size, size_type, browsing_context_id);
        }

        if let Some(resize_channel) = self.webdriver.resize_channel.take() {
            let _ = resize_channel.send(new_size);
        }

        self.window_size = new_size;
    }

    /// Called when the window exits from fullscreen mode
    fn handle_exit_fullscreen_msg(
        &mut self,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
    ) {
        let browsing_context_id = BrowsingContextId::from(top_level_browsing_context_id);
        self.switch_fullscreen_mode(browsing_context_id);
    }

    /// Handle updating actual viewport / zoom due to @viewport rules
    fn handle_viewport_constrained_msg(
        &mut self,
        pipeline_id: PipelineId,
        constraints: ViewportConstraints,
    ) {
        self.compositor_proxy
            .send(ToCompositorMsg::ViewportConstrained(
                pipeline_id,
                constraints,
            ));
    }

    /// Checks the state of all script and layout pipelines to see if they are idle
    /// and compares the current layout state to what the compositor has. This is used
    /// to check if the output image is "stable" and can be written as a screenshot
    /// for reftests.
    /// Since this function is only used in reftests, we do not harden it against panic.
    fn handle_is_ready_to_save_image(
        &mut self,
        pipeline_states: HashMap<PipelineId, Epoch>,
    ) -> ReadyToSave {
        // Note that this function can panic, due to ipc-channel creation
        // failure. Avoiding this panic would require a mechanism for dealing
        // with low-resource scenarios.
        //
        // If there is no focus browsing context yet, the initial page has
        // not loaded, so there is nothing to save yet.
        let top_level_browsing_context_id = match self.active_browser_id {
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
        for browsing_context in
            self.fully_active_browsing_contexts_iter(top_level_browsing_context_id)
        {
            let pipeline_id = browsing_context.pipeline_id;
            debug!(
                "Checking readiness of browsing context {}, pipeline {}.",
                browsing_context.id, pipeline_id
            );

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
                Some(&DocumentState::Idle) => {},
                Some(&DocumentState::Pending) | None => {
                    return ReadyToSave::DocumentLoading;
                },
            }

            // Check the visible rectangle for this pipeline. If the constellation has received a
            // size for the pipeline, then its painting should be up to date.
            //
            // If the rectangle for this pipeline is zero sized, it will
            // never be painted. In this case, don't query the layout
            // thread as it won't contribute to the final output image.
            if browsing_context.size == TypedSize2D::zero() {
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
                        Ok(layout_thread_epoch) => {
                            if layout_thread_epoch != *compositor_epoch {
                                return ReadyToSave::EpochMismatch;
                            }
                        },
                    }
                },
                None => {
                    // The compositor doesn't know about this pipeline yet.
                    // Assume it hasn't rendered yet.
                    return ReadyToSave::PipelineUnknown;
                },
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
                if let Some(browsing_context) =
                    self.browsing_contexts.get(&ancestor.browsing_context_id)
                {
                    if browsing_context.pipeline_id == ancestor_id {
                        if let Some(parent_pipeline_id) = browsing_context.parent_pipeline_id {
                            ancestor_id = parent_pipeline_id;
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
    fn resize_browsing_context(
        &mut self,
        new_size: WindowSizeData,
        size_type: WindowSizeType,
        browsing_context_id: BrowsingContextId,
    ) {
        if let Some(browsing_context) = self.browsing_contexts.get_mut(&browsing_context_id) {
            browsing_context.size = new_size.initial_viewport;
            // Send Resize (or ResizeInactive) messages to each pipeline in the frame tree.
            let pipeline_id = browsing_context.pipeline_id;
            let pipeline = match self.pipelines.get(&pipeline_id) {
                None => return warn!("Pipeline {:?} resized after closing.", pipeline_id),
                Some(pipeline) => pipeline,
            };
            let _ = pipeline.event_loop.send(ConstellationControlMsg::Resize(
                pipeline.id,
                new_size,
                size_type,
            ));
            let pipeline_ids = browsing_context
                .pipelines
                .iter()
                .filter(|pipeline_id| **pipeline_id != pipeline.id);
            for id in pipeline_ids {
                if let Some(pipeline) = self.pipelines.get(&id) {
                    let _ = pipeline
                        .event_loop
                        .send(ConstellationControlMsg::ResizeInactive(
                            pipeline.id,
                            new_size,
                        ));
                }
            }
        }

        // Send resize message to any pending pipelines that aren't loaded yet.
        for change in &self.pending_changes {
            let pipeline_id = change.new_pipeline_id;
            let pipeline = match self.pipelines.get(&pipeline_id) {
                None => {
                    warn!("Pending pipeline {:?} is closed", pipeline_id);
                    continue;
                },
                Some(pipeline) => pipeline,
            };
            if pipeline.browsing_context_id == browsing_context_id {
                let _ = pipeline.event_loop.send(ConstellationControlMsg::Resize(
                    pipeline.id,
                    new_size,
                    size_type,
                ));
            }
        }
    }

    // Handle switching from fullscreen mode
    fn switch_fullscreen_mode(&mut self, browsing_context_id: BrowsingContextId) {
        if let Some(browsing_context) = self.browsing_contexts.get(&browsing_context_id) {
            let pipeline_id = browsing_context.pipeline_id;
            let pipeline = match self.pipelines.get(&pipeline_id) {
                None => {
                    return warn!(
                        "Pipeline {:?} switched from fullscreen mode after closing.",
                        pipeline_id
                    )
                },
                Some(pipeline) => pipeline,
            };
            let _ = pipeline
                .event_loop
                .send(ConstellationControlMsg::ExitFullScreen(pipeline.id));
        }
    }

    // Close a browsing context (and all children)
    fn close_browsing_context(
        &mut self,
        browsing_context_id: BrowsingContextId,
        exit_mode: ExitPipelineMode,
    ) {
        debug!("Closing browsing context {}.", browsing_context_id);

        self.close_browsing_context_children(
            browsing_context_id,
            DiscardBrowsingContext::Yes,
            exit_mode,
        );

        let browsing_context = match self.browsing_contexts.remove(&browsing_context_id) {
            Some(ctx) => ctx,
            None => return warn!("Closing browsing context {:?} twice.", browsing_context_id),
        };

        {
            let session_history = self.get_joint_session_history(browsing_context.top_level_id);
            session_history.remove_entries_for_browsing_context(browsing_context_id);
        }

        if let Some(parent_pipeline_id) = browsing_context.parent_pipeline_id {
            match self.pipelines.get_mut(&parent_pipeline_id) {
                None => {
                    return warn!(
                        "Pipeline {:?} child closed after parent.",
                        parent_pipeline_id
                    );
                },
                Some(parent_pipeline) => parent_pipeline.remove_child(browsing_context_id),
            };
        }
        debug!("Closed browsing context {:?}.", browsing_context_id);
    }

    // Close the children of a browsing context
    fn close_browsing_context_children(
        &mut self,
        browsing_context_id: BrowsingContextId,
        dbc: DiscardBrowsingContext,
        exit_mode: ExitPipelineMode,
    ) {
        debug!("Closing browsing context children {}.", browsing_context_id);
        // Store information about the pipelines to be closed. Then close the
        // pipelines, before removing ourself from the browsing_contexts hash map. This
        // ordering is vital - so that if close_pipeline() ends up closing
        // any child browsing contexts, they can be removed from the parent browsing context correctly.
        let mut pipelines_to_close: Vec<PipelineId> = self
            .pending_changes
            .iter()
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
    fn handle_discard_document(
        &mut self,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
        pipeline_id: PipelineId,
    ) {
        match self.browsers.get_mut(&top_level_browsing_context_id) {
            Some(browser) => {
                let load_data = match self.pipelines.get(&pipeline_id) {
                    Some(pipeline) => pipeline.load_data.clone(),
                    None => return warn!("Discarding closed pipeline {}", pipeline_id),
                };
                browser.session_history.replace_reloader(
                    NeedsToReload::No(pipeline_id),
                    NeedsToReload::Yes(pipeline_id, load_data),
                );
            },
            None => {
                return warn!(
                    "Discarding pipeline {} after browser {} closure",
                    pipeline_id, top_level_browsing_context_id,
                );
            },
        };
        self.close_pipeline(
            pipeline_id,
            DiscardBrowsingContext::No,
            ExitPipelineMode::Normal,
        );
    }

    // Send a message to script requesting the document associated with this pipeline runs the 'unload' algorithm.
    fn unload_document(&self, pipeline_id: PipelineId) {
        if let Some(pipeline) = self.pipelines.get(&pipeline_id) {
            let msg = ConstellationControlMsg::UnloadDocument(pipeline_id);
            let _ = pipeline.event_loop.send(msg);
        }
    }

    // Close all pipelines at and beneath a given browsing context
    fn close_pipeline(
        &mut self,
        pipeline_id: PipelineId,
        dbc: DiscardBrowsingContext,
        exit_mode: ExitPipelineMode,
    ) {
        debug!("Closing pipeline {:?}.", pipeline_id);

        // Sever connection to browsing context
        let browsing_context_id = self
            .pipelines
            .get(&pipeline_id)
            .map(|pipeline| pipeline.browsing_context_id);
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
            let mut browsing_contexts_to_close = vec![];

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
        let pending_index = self
            .pending_changes
            .iter()
            .position(|change| change.new_pipeline_id == pipeline_id);
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
            Some((ref mut rng, probability)) => {
                if probability <= rng.gen::<f32>() {
                    return;
                }
            },
            _ => return,
        };
        // In order to get repeatability, we sort the pipeline ids.
        let mut pipeline_ids: Vec<&PipelineId> = self.pipelines.keys().collect();
        pipeline_ids.sort();
        if let Some((ref mut rng, probability)) = self.random_pipeline_closure {
            if let Some(pipeline_id) = rng.choose(&*pipeline_ids) {
                if let Some(pipeline) = self.pipelines.get(pipeline_id) {
                    if self
                        .pending_changes
                        .iter()
                        .any(|change| change.new_pipeline_id == pipeline.id) &&
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

    fn get_joint_session_history(
        &mut self,
        top_level_id: TopLevelBrowsingContextId,
    ) -> &mut JointSessionHistory {
        &mut self
            .browsers
            .entry(top_level_id)
            // This shouldn't be necessary since `get_joint_session_history` is
            // invoked for existing browsers but we need this to satisfy the
            // type system.
            .or_insert_with(|| Browser {
                focused_browsing_context_id: BrowsingContextId::from(top_level_id),
                session_history: JointSessionHistory::new(),
            })
            .session_history
    }

    // Convert a browsing context to a sendable form to pass to the compositor
    fn browsing_context_to_sendable(
        &self,
        browsing_context_id: BrowsingContextId,
    ) -> Option<SendableFrameTree> {
        self.browsing_contexts
            .get(&browsing_context_id)
            .and_then(|browsing_context| {
                self.pipelines
                    .get(&browsing_context.pipeline_id)
                    .map(|pipeline| {
                        let mut frame_tree = SendableFrameTree {
                            pipeline: pipeline.to_sendable(),
                            children: vec![],
                        };

                        for child_browsing_context_id in &pipeline.children {
                            if let Some(child) =
                                self.browsing_context_to_sendable(*child_browsing_context_id)
                            {
                                frame_tree.children.push(child);
                            }
                        }

                        frame_tree
                    })
            })
    }

    /// Re-send the frame tree to the compositor.
    fn update_frame_tree_if_active(
        &mut self,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
    ) {
        // Only send the frame tree if it's the active one or if no frame tree
        // has been sent yet.
        if self.active_browser_id.is_none() ||
            Some(top_level_browsing_context_id) == self.active_browser_id
        {
            self.send_frame_tree(top_level_browsing_context_id);
        }
    }

    /// Send the current frame tree to compositor
    fn send_frame_tree(&mut self, top_level_browsing_context_id: TopLevelBrowsingContextId) {
        // Note that this function can panic, due to ipc-channel creation failure.
        // avoiding this panic would require a mechanism for dealing
        // with low-resource scenarios.
        let browsing_context_id = BrowsingContextId::from(top_level_browsing_context_id);
        if let Some(frame_tree) = self.browsing_context_to_sendable(browsing_context_id) {
            debug!(
                "Sending frame tree for browsing context {}.",
                browsing_context_id
            );
            self.active_browser_id = Some(top_level_browsing_context_id);
            self.compositor_proxy
                .send(ToCompositorMsg::SetFrameTree(frame_tree));
        }
    }
}
