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
//!   layout.  Pipelines may share script threads.
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
//! ```text
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
//! * The script thread.
//! * The graphics compositor.
//! * The font cache, image cache, and resource manager, which load
//!   and cache shared fonts, images, or other resources.
//! * The service worker manager.
//! * The devtools and webdriver servers.
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
//! * Constellation can block on compositor
//! * Constellation can block on embedder
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
//! See <https://github.com/servo/servo/issues/14704>

use std::borrow::ToOwned;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet, VecDeque};
use std::marker::PhantomData;
use std::mem::replace;
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex};
use std::{process, thread};

use background_hang_monitor::HangMonitorRegister;
use background_hang_monitor_api::{
    BackgroundHangMonitorControlMsg, BackgroundHangMonitorRegister, HangMonitorAlert,
};
use base::Epoch;
use base::id::{
    BroadcastChannelRouterId, BrowsingContextGroupId, BrowsingContextId, HistoryStateId,
    MessagePortId, MessagePortRouterId, PipelineId, PipelineNamespace, PipelineNamespaceId,
    PipelineNamespaceRequest, WebViewId,
};
#[cfg(feature = "bluetooth")]
use bluetooth_traits::BluetoothRequest;
use canvas_traits::ConstellationCanvasMsg;
use canvas_traits::canvas::{CanvasId, CanvasMsg};
use canvas_traits::webgl::WebGLThreads;
use compositing_traits::{
    CompositorMsg, CompositorProxy, SendableFrameTree, WebrenderExternalImageRegistry,
};
use constellation_traits::{
    AuxiliaryWebViewCreationRequest, AuxiliaryWebViewCreationResponse, BroadcastMsg, DocumentState,
    EmbedderToConstellationMessage, IFrameLoadInfo, IFrameLoadInfoWithData, IFrameSandboxState,
    IFrameSizeMsg, Job, LoadData, LoadOrigin, LogEntry, MessagePortMsg, NavigationHistoryBehavior,
    PaintMetricEvent, PortMessageTask, PortTransferInfo, SWManagerMsg, SWManagerSenders,
    ScriptToConstellationChan, ScriptToConstellationMessage, ServiceWorkerManagerFactory,
    ServiceWorkerMsg, StructuredSerializedData, TraversalDirection, WindowSizeType,
};
use crossbeam_channel::{Receiver, Select, Sender, unbounded};
use devtools_traits::{
    ChromeToDevtoolsControlMsg, DevtoolsControlMsg, DevtoolsPageInfo, NavigationState,
    ScriptToDevtoolsControlMsg,
};
use embedder_traits::resources::{self, Resource};
use embedder_traits::user_content_manager::UserContentManager;
use embedder_traits::{
    AnimationState, CompositorHitTestResult, Cursor, EmbedderMsg, EmbedderProxy,
    FocusSequenceNumber, ImeEvent, InputEvent, JSValue, JavaScriptEvaluationError,
    JavaScriptEvaluationId, KeyboardEventWithWebDriverId, MediaSessionActionType,
    MediaSessionEvent, MediaSessionPlaybackState, MouseButton, MouseButtonAction, MouseButtonEvent,
    Theme, ViewportDetails, WebDriverCommandMsg, WebDriverCommandResponse, WebDriverLoadStatus,
};
use euclid::Size2D;
use euclid::default::Size2D as UntypedSize2D;
use fonts::SystemFontServiceProxy;
use ipc_channel::Error as IpcError;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use keyboard_types::webdriver::Event as WebDriverInputEvent;
use keyboard_types::{Key, KeyState, KeyboardEvent, Modifiers};
use log::{debug, error, info, trace, warn};
use media::WindowGLContext;
use net_traits::pub_domains::reg_host;
use net_traits::request::Referrer;
use net_traits::storage_thread::{StorageThreadMsg, StorageType};
use net_traits::{self, IpcSend, ReferrerPolicy, ResourceThreads};
use profile_traits::mem::ProfilerMsg;
use profile_traits::{mem, time};
use script_layout_interface::{LayoutFactory, ScriptThreadFactory};
use script_traits::{
    ConstellationInputEvent, DiscardBrowsingContext, DocumentActivity, ProgressiveWebMetricType,
    ScriptThreadMessage, UpdatePipelineIdReason,
};
use serde::{Deserialize, Serialize};
use servo_config::{opts, pref};
use servo_rand::{Rng, ServoRng, SliceRandom, random};
use servo_url::{Host, ImmutableOrigin, ServoUrl};
use style_traits::CSSPixel;
#[cfg(feature = "webgpu")]
use webgpu::swapchain::WGPUImageMap;
#[cfg(feature = "webgpu")]
use webgpu_traits::{WebGPU, WebGPURequest};
#[cfg(feature = "webgpu")]
use webrender::RenderApi;
use webrender::RenderApiSender;
use webrender_api::units::LayoutVector2D;
use webrender_api::{DocumentId, ExternalScrollId, ImageKey};

use crate::browsingcontext::{
    AllBrowsingContextsIterator, BrowsingContext, FullyActiveBrowsingContextsIterator,
    NewBrowsingContextInfo,
};
use crate::constellation_webview::ConstellationWebView;
use crate::event_loop::EventLoop;
use crate::pipeline::{InitialPipelineState, Pipeline};
use crate::process_manager::ProcessManager;
use crate::serviceworker::ServiceWorkerUnprivilegedContent;
use crate::session_history::{
    JointSessionHistory, NeedsToReload, SessionHistoryChange, SessionHistoryDiff,
};
use crate::webview_manager::WebViewManager;

type PendingApprovalNavigations = HashMap<PipelineId, (LoadData, NavigationHistoryBehavior)>;

#[derive(Debug)]
/// The state used by MessagePortInfo to represent the various states the port can be in.
enum TransferState {
    /// The port is currently managed by a given global,
    /// identified by its router id.
    Managed(MessagePortRouterId),
    /// The port is currently in-transfer,
    /// and incoming tasks should be buffered until it becomes managed again.
    TransferInProgress(VecDeque<PortMessageTask>),
    /// A global has requested the transfer to be completed,
    /// it's pending a confirmation of either failure or success to complete the transfer.
    CompletionInProgress(MessagePortRouterId),
    /// While a completion of a transfer was in progress, the port was shipped,
    /// hence the transfer failed to complete.
    /// We start buffering incoming messages,
    /// while awaiting the return of the previous buffer from the global
    /// that failed to complete the transfer.
    CompletionFailed(VecDeque<PortMessageTask>),
    /// While a completion failed, another global requested to complete the transfer.
    /// We are still buffering messages, and awaiting the return of the buffer from the global who failed.
    CompletionRequested(MessagePortRouterId, VecDeque<PortMessageTask>),
}

#[derive(Debug)]
/// Info related to a message-port tracked by the constellation.
struct MessagePortInfo {
    /// The current state of the messageport.
    state: TransferState,

    /// The id of the entangled port, if any.
    entangled_with: Option<MessagePortId>,
}

#[cfg(feature = "webgpu")]
/// Webrender related objects required by WebGPU threads
struct WebrenderWGPU {
    /// Webrender API.
    webrender_api: RenderApi,

    /// List of Webrender external images
    webrender_external_images: Arc<Mutex<WebrenderExternalImageRegistry>>,

    /// WebGPU data that supplied to Webrender for rendering
    wgpu_image_map: WGPUImageMap,
}

/// A browsing context group.
///
/// <https://html.spec.whatwg.org/multipage/#browsing-context-group>
#[derive(Clone, Default)]
struct BrowsingContextGroup {
    /// A browsing context group holds a set of top-level browsing contexts.
    top_level_browsing_context_set: HashSet<WebViewId>,

    /// The set of all event loops in this BrowsingContextGroup.
    /// We store the event loops in a map
    /// indexed by registered domain name (as a `Host`) to event loops.
    /// It is important that scripts with the same eTLD+1,
    /// who are part of the same browsing-context group
    /// share an event loop, since they can use `document.domain`
    /// to become same-origin, at which point they can share DOM objects.
    event_loops: HashMap<Host, Weak<EventLoop>>,

    /// The set of all WebGPU channels in this BrowsingContextGroup.
    #[cfg(feature = "webgpu")]
    webgpus: HashMap<Host, WebGPU>,
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
pub struct Constellation<STF, SWF> {
    /// An ipc-sender/threaded-receiver pair
    /// to facilitate installing pipeline namespaces in threads
    /// via a per-process installer.
    namespace_receiver: Receiver<Result<PipelineNamespaceRequest, IpcError>>,
    namespace_ipc_sender: IpcSender<PipelineNamespaceRequest>,

    /// An IPC channel for script threads to send messages to the constellation.
    /// This is the script threads' view of `script_receiver`.
    script_sender: IpcSender<(PipelineId, ScriptToConstellationMessage)>,

    /// A channel for the constellation to receive messages from script threads.
    /// This is the constellation's view of `script_sender`.
    script_receiver: Receiver<Result<(PipelineId, ScriptToConstellationMessage), IpcError>>,

    /// A handle to register components for hang monitoring.
    /// None when in multiprocess mode.
    background_monitor_register: Option<Box<dyn BackgroundHangMonitorRegister>>,

    /// Channels to control all background-hang monitors.
    /// TODO: store them on the relevant BrowsingContextGroup,
    /// so that they could be controlled on a "per-tab/event-loop" basis.
    background_monitor_control_senders: Vec<IpcSender<BackgroundHangMonitorControlMsg>>,

    /// A channel for the background hang monitor to send messages
    /// to the constellation.
    background_hang_monitor_sender: IpcSender<HangMonitorAlert>,

    /// A channel for the constellation to receiver messages
    /// from the background hang monitor.
    background_hang_monitor_receiver: Receiver<Result<HangMonitorAlert, IpcError>>,

    /// A factory for creating layouts. This allows customizing the kind
    /// of layout created for a [`Constellation`] and prevents a circular crate
    /// dependency between script and layout.
    layout_factory: Arc<dyn LayoutFactory>,

    /// A channel for the constellation to receive messages from the compositor thread.
    compositor_receiver: Receiver<EmbedderToConstellationMessage>,

    /// A channel through which messages can be sent to the embedder.
    embedder_proxy: EmbedderProxy,

    /// A channel (the implementation of which is port-specific) for the
    /// constellation to send messages to the compositor thread.
    compositor_proxy: CompositorProxy,

    /// Bookkeeping data for all webviews in the constellation.
    webviews: WebViewManager<ConstellationWebView>,

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
    system_font_service: Arc<SystemFontServiceProxy>,

    /// A channel for the constellation to send messages to the
    /// devtools thread.
    devtools_sender: Option<Sender<DevtoolsControlMsg>>,

    /// An IPC channel for the constellation to send messages to the
    /// bluetooth thread.
    #[cfg(feature = "bluetooth")]
    bluetooth_ipc_sender: IpcSender<BluetoothRequest>,

    /// A map of origin to sender to a Service worker manager.
    sw_managers: HashMap<ImmutableOrigin, IpcSender<ServiceWorkerMsg>>,

    /// An IPC channel for Service Worker Manager threads to send
    /// messages to the constellation.  This is the SW Manager thread's
    /// view of `swmanager_receiver`.
    swmanager_ipc_sender: IpcSender<SWManagerMsg>,

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

    /// A single WebRender document the constellation operates on.
    #[cfg(feature = "webgpu")]
    webrender_document: DocumentId,

    /// Webrender related objects required by WebGPU threads
    #[cfg(feature = "webgpu")]
    webrender_wgpu: WebrenderWGPU,

    /// A map of message-port Id to info.
    message_ports: HashMap<MessagePortId, MessagePortInfo>,

    /// A map of router-id to ipc-sender, to route messages to ports.
    message_port_routers: HashMap<MessagePortRouterId, IpcSender<MessagePortMsg>>,

    /// A map of broadcast routers to their IPC sender.
    broadcast_routers: HashMap<BroadcastChannelRouterId, IpcSender<BroadcastMsg>>,

    /// A map of origin to a map of channel-name to a list of relevant routers.
    broadcast_channels: HashMap<ImmutableOrigin, HashMap<String, Vec<BroadcastChannelRouterId>>>,

    /// The set of all the pipelines in the browser.  (See the `pipeline` module
    /// for more details.)
    pipelines: HashMap<PipelineId, Pipeline>,

    /// The set of all the browsing contexts in the browser.
    browsing_contexts: HashMap<BrowsingContextId, BrowsingContext>,

    /// A user agent holds a a set of browsing context groups.
    ///
    /// <https://html.spec.whatwg.org/multipage/#browsing-context-group-set>
    browsing_context_group_set: HashMap<BrowsingContextGroupId, BrowsingContextGroup>,

    /// The Id counter for BrowsingContextGroup.
    browsing_context_group_next_id: u32,

    /// When a navigation is performed, we do not immediately update
    /// the session history, instead we ask the event loop to begin loading
    /// the new document, and do not update the browsing context until the
    /// document is active. Between starting the load and it activating,
    /// we store a `SessionHistoryChange` object for the navigation in progress.
    pending_changes: Vec<SessionHistoryChange>,

    /// Pipeline IDs are namespaced in order to avoid name collisions,
    /// and the namespaces are allocated by the constellation.
    next_pipeline_namespace_id: PipelineNamespaceId,

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
    phantom: PhantomData<(STF, SWF)>,

    /// Entry point to create and get channels to a WebGLThread.
    webgl_threads: Option<WebGLThreads>,

    /// The XR device registry
    webxr_registry: Option<webxr_api::Registry>,

    /// A channel through which messages can be sent to the canvas paint thread.
    canvas_sender: Sender<ConstellationCanvasMsg>,

    canvas_ipc_sender: IpcSender<CanvasMsg>,

    /// Navigation requests from script awaiting approval from the embedder.
    pending_approval_navigations: PendingApprovalNavigations,

    /// Bitmask which indicates which combination of mouse buttons are
    /// currently being pressed.
    pressed_mouse_buttons: u16,

    /// The currently activated keyboard modifiers.
    active_keyboard_modifiers: Modifiers,

    /// If True, exits on thread failure instead of displaying about:failure
    hard_fail: bool,

    /// Pipeline ID of the active media session.
    active_media_session: Option<PipelineId>,

    /// The image bytes associated with the RippyPNG embedder resource.
    /// Read during startup and provided to image caches that are created
    /// on an as-needed basis, rather than retrieving it every time.
    rippy_data: Vec<u8>,

    /// User content manager
    user_content_manager: UserContentManager,

    /// The process manager.
    process_manager: ProcessManager,
}

/// State needed to construct a constellation.
pub struct InitialConstellationState {
    /// A channel through which messages can be sent to the embedder.
    pub embedder_proxy: EmbedderProxy,

    /// A channel through which messages can be sent to the compositor in-process.
    pub compositor_proxy: CompositorProxy,

    /// A channel to the developer tools, if applicable.
    pub devtools_sender: Option<Sender<DevtoolsControlMsg>>,

    /// A channel to the bluetooth thread.
    #[cfg(feature = "bluetooth")]
    pub bluetooth_thread: IpcSender<BluetoothRequest>,

    /// A proxy to the `SystemFontService` which manages the list of system fonts.
    pub system_font_service: Arc<SystemFontServiceProxy>,

    /// A channel to the resource thread.
    pub public_resource_threads: ResourceThreads,

    /// A channel to the resource thread.
    pub private_resource_threads: ResourceThreads,

    /// A channel to the time profiler thread.
    pub time_profiler_chan: time::ProfilerChan,

    /// A channel to the memory profiler thread.
    pub mem_profiler_chan: mem::ProfilerChan,

    /// Webrender document ID.
    pub webrender_document: DocumentId,

    /// Webrender API.
    pub webrender_api_sender: RenderApiSender,

    /// Webrender external images
    pub webrender_external_images: Arc<Mutex<WebrenderExternalImageRegistry>>,

    /// Entry point to create and get channels to a WebGLThread.
    pub webgl_threads: Option<WebGLThreads>,

    /// The XR device registry
    pub webxr_registry: Option<webxr_api::Registry>,

    #[cfg(feature = "webgpu")]
    pub wgpu_image_map: WGPUImageMap,

    /// User content manager
    pub user_content_manager: UserContentManager,
}

/// Data needed for webdriver
struct WebDriverData {
    load_channel: Option<(PipelineId, IpcSender<WebDriverLoadStatus>)>,
    resize_channel: Option<IpcSender<Size2D<f32, CSSPixel>>>,
    // Forward responses from the script thread to the webdriver server.
    input_command_response_sender: Option<IpcSender<WebDriverCommandResponse>>,
}

impl WebDriverData {
    fn new() -> WebDriverData {
        WebDriverData {
            load_channel: None,
            resize_channel: None,
            input_command_response_sender: None,
        }
    }
}

/// When we are running reftests, we save an image to compare against a reference.
/// This enum gives the possible states of preparing such an image.
#[derive(Debug, PartialEq)]
enum ReadyToSave {
    NoTopLevelBrowsingContext,
    PendingChanges,
    DocumentLoading,
    EpochMismatch,
    PipelineUnknown,
    Ready,
}

/// When we are exiting a pipeline, we can either force exiting or not.
/// A normal exit waits for the compositor to update its state before
/// exiting, and delegates layout exit to script. A forced exit does
/// not notify the compositor, and exits layout without involving script.
#[derive(Clone, Copy, Debug)]
enum ExitPipelineMode {
    Normal,
    Force,
}

/// The number of warnings to include in each crash report.
const WARNINGS_BUFFER_SIZE: usize = 32;

/// Route an ipc receiver to an crossbeam receiver, preserving any errors.
fn route_ipc_receiver_to_new_crossbeam_receiver_preserving_errors<T>(
    ipc_receiver: IpcReceiver<T>,
) -> Receiver<Result<T, IpcError>>
where
    T: for<'de> Deserialize<'de> + Serialize + Send + 'static,
{
    let (crossbeam_sender, crossbeam_receiver) = unbounded();
    ROUTER.add_typed_route(
        ipc_receiver,
        Box::new(move |message| {
            let _ = crossbeam_sender.send(message);
        }),
    );
    crossbeam_receiver
}

impl<STF, SWF> Constellation<STF, SWF>
where
    STF: ScriptThreadFactory,
    SWF: ServiceWorkerManagerFactory,
{
    /// Create a new constellation thread.
    #[allow(clippy::too_many_arguments)]
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip(state, layout_factory),
            fields(servo_profiling = true),
            level = "trace",
        )
    )]
    pub fn start(
        state: InitialConstellationState,
        layout_factory: Arc<dyn LayoutFactory>,
        random_pipeline_closure_probability: Option<f32>,
        random_pipeline_closure_seed: Option<usize>,
        hard_fail: bool,
        canvas_create_sender: Sender<ConstellationCanvasMsg>,
        canvas_ipc_sender: IpcSender<CanvasMsg>,
    ) -> Sender<EmbedderToConstellationMessage> {
        let (compositor_sender, compositor_receiver) = unbounded();

        // service worker manager to communicate with constellation
        let (swmanager_ipc_sender, swmanager_ipc_receiver) =
            ipc::channel().expect("ipc channel failure");

        thread::Builder::new()
            .name("Constellation".to_owned())
            .spawn(move || {
                let (script_ipc_sender, script_ipc_receiver) =
                    ipc::channel().expect("ipc channel failure");
                let script_receiver =
                    route_ipc_receiver_to_new_crossbeam_receiver_preserving_errors(
                        script_ipc_receiver,
                    );

                let (namespace_ipc_sender, namespace_ipc_receiver) =
                    ipc::channel().expect("ipc channel failure");
                let namespace_receiver =
                    route_ipc_receiver_to_new_crossbeam_receiver_preserving_errors(
                        namespace_ipc_receiver,
                    );

                let (background_hang_monitor_ipc_sender, background_hang_monitor_ipc_receiver) =
                    ipc::channel().expect("ipc channel failure");
                let background_hang_monitor_receiver =
                    route_ipc_receiver_to_new_crossbeam_receiver_preserving_errors(
                        background_hang_monitor_ipc_receiver,
                    );

                // If we are in multiprocess mode,
                // a dedicated per-process hang monitor will be initialized later inside the content process.
                // See run_content_process in servo/lib.rs
                let (background_monitor_register, background_hang_monitor_control_ipc_senders) =
                    if opts::get().multiprocess {
                        (None, vec![])
                    } else {
                        let (
                            background_hang_monitor_control_ipc_sender,
                            background_hang_monitor_control_ipc_receiver,
                        ) = ipc::channel().expect("ipc channel failure");
                        (
                            Some(HangMonitorRegister::init(
                                background_hang_monitor_ipc_sender.clone(),
                                background_hang_monitor_control_ipc_receiver,
                                opts::get().background_hang_monitor,
                            )),
                            vec![background_hang_monitor_control_ipc_sender],
                        )
                    };

                let swmanager_receiver =
                    route_ipc_receiver_to_new_crossbeam_receiver_preserving_errors(
                        swmanager_ipc_receiver,
                    );

                // Zero is reserved for the embedder.
                PipelineNamespace::install(PipelineNamespaceId(1));

                #[cfg(feature = "webgpu")]
                let webrender_wgpu = WebrenderWGPU {
                    webrender_api: state.webrender_api_sender.create_api(),
                    webrender_external_images: state.webrender_external_images,
                    wgpu_image_map: state.wgpu_image_map,
                };

                let rippy_data = resources::read_bytes(Resource::RippyPNG);

                let mut constellation: Constellation<STF, SWF> = Constellation {
                    namespace_receiver,
                    namespace_ipc_sender,
                    script_sender: script_ipc_sender,
                    background_hang_monitor_sender: background_hang_monitor_ipc_sender,
                    background_hang_monitor_receiver,
                    background_monitor_register,
                    background_monitor_control_senders: background_hang_monitor_control_ipc_senders,
                    script_receiver,
                    compositor_receiver,
                    layout_factory,
                    embedder_proxy: state.embedder_proxy,
                    compositor_proxy: state.compositor_proxy,
                    webviews: WebViewManager::default(),
                    devtools_sender: state.devtools_sender,
                    #[cfg(feature = "bluetooth")]
                    bluetooth_ipc_sender: state.bluetooth_thread,
                    public_resource_threads: state.public_resource_threads,
                    private_resource_threads: state.private_resource_threads,
                    system_font_service: state.system_font_service,
                    sw_managers: Default::default(),
                    swmanager_receiver,
                    swmanager_ipc_sender,
                    browsing_context_group_set: Default::default(),
                    browsing_context_group_next_id: Default::default(),
                    message_ports: HashMap::new(),
                    message_port_routers: HashMap::new(),
                    broadcast_routers: HashMap::new(),
                    broadcast_channels: HashMap::new(),
                    pipelines: HashMap::new(),
                    browsing_contexts: HashMap::new(),
                    pending_changes: vec![],
                    // We initialize the namespace at 2, since we reserved
                    // namespace 0 for the embedder, and 0 for the constellation
                    next_pipeline_namespace_id: PipelineNamespaceId(2),
                    time_profiler_chan: state.time_profiler_chan,
                    mem_profiler_chan: state.mem_profiler_chan.clone(),
                    phantom: PhantomData,
                    webdriver: WebDriverData::new(),
                    document_states: HashMap::new(),
                    #[cfg(feature = "webgpu")]
                    webrender_document: state.webrender_document,
                    #[cfg(feature = "webgpu")]
                    webrender_wgpu,
                    shutting_down: false,
                    handled_warnings: VecDeque::new(),
                    random_pipeline_closure: random_pipeline_closure_probability.map(|prob| {
                        let seed = random_pipeline_closure_seed.unwrap_or_else(random);
                        let rng = ServoRng::new_manually_reseeded(seed as u64);
                        warn!("Randomly closing pipelines.");
                        info!("Using seed {} for random pipeline closure.", seed);
                        (rng, prob)
                    }),
                    webgl_threads: state.webgl_threads,
                    webxr_registry: state.webxr_registry,
                    canvas_sender: canvas_create_sender,
                    canvas_ipc_sender,
                    pending_approval_navigations: HashMap::new(),
                    pressed_mouse_buttons: 0,
                    active_keyboard_modifiers: Modifiers::empty(),
                    hard_fail,
                    active_media_session: None,
                    rippy_data,
                    user_content_manager: state.user_content_manager,
                    process_manager: ProcessManager::new(state.mem_profiler_chan),
                };

                constellation.run();
            })
            .expect("Thread spawning failed");

        compositor_sender
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

    fn next_browsing_context_group_id(&mut self) -> BrowsingContextGroupId {
        let id = self.browsing_context_group_next_id;
        self.browsing_context_group_next_id += 1;
        BrowsingContextGroupId(id)
    }

    fn get_event_loop(
        &mut self,
        host: &Host,
        webview_id: &WebViewId,
        opener: &Option<BrowsingContextId>,
    ) -> Result<Weak<EventLoop>, &'static str> {
        let bc_group = match opener {
            Some(browsing_context_id) => {
                let opener = self
                    .browsing_contexts
                    .get(browsing_context_id)
                    .ok_or("Opener was closed before the openee started")?;
                self.browsing_context_group_set
                    .get(&opener.bc_group_id)
                    .ok_or("Opener belongs to an unknown browsing context group")?
            },
            None => self
                .browsing_context_group_set
                .iter()
                .filter_map(|(_, bc_group)| {
                    if bc_group
                        .top_level_browsing_context_set
                        .contains(webview_id)
                    {
                        Some(bc_group)
                    } else {
                        None
                    }
                })
                .last()
                .ok_or(
                    "Trying to get an event-loop for a top-level belonging to an unknown browsing context group",
                )?,
        };
        bc_group
            .event_loops
            .get(host)
            .ok_or("Trying to get an event-loop from an unknown browsing context group")
            .cloned()
    }

    fn set_event_loop(
        &mut self,
        event_loop: Weak<EventLoop>,
        host: Host,
        webview_id: WebViewId,
        opener: Option<BrowsingContextId>,
    ) {
        let relevant_top_level = if let Some(opener) = opener {
            match self.browsing_contexts.get(&opener) {
                Some(opener) => opener.top_level_id,
                None => {
                    warn!("Setting event-loop for an unknown auxiliary");
                    return;
                },
            }
        } else {
            webview_id
        };
        let maybe_bc_group_id = self
            .browsing_context_group_set
            .iter()
            .filter_map(|(id, bc_group)| {
                if bc_group
                    .top_level_browsing_context_set
                    .contains(&webview_id)
                {
                    Some(*id)
                } else {
                    None
                }
            })
            .last();
        let bc_group_id = match maybe_bc_group_id {
            Some(id) => id,
            None => {
                warn!("Trying to add an event-loop to an unknown browsing context group");
                return;
            },
        };
        if let Some(bc_group) = self.browsing_context_group_set.get_mut(&bc_group_id) {
            if bc_group
                .event_loops
                .insert(host.clone(), event_loop)
                .is_some()
            {
                warn!(
                    "Double-setting an event-loop for {:?} at {:?}",
                    host, relevant_top_level
                );
            }
        }
    }

    /// Helper function for creating a pipeline
    #[allow(clippy::too_many_arguments)]
    fn new_pipeline(
        &mut self,
        pipeline_id: PipelineId,
        browsing_context_id: BrowsingContextId,
        webview_id: WebViewId,
        parent_pipeline_id: Option<PipelineId>,
        opener: Option<BrowsingContextId>,
        initial_viewport_details: ViewportDetails,
        // TODO: we have to provide ownership of the LoadData
        // here, because it will be send on an ipc channel,
        // and ipc channels take onership of their data.
        // https://github.com/servo/ipc-channel/issues/138
        load_data: LoadData,
        sandbox: IFrameSandboxState,
        is_private: bool,
        throttled: bool,
    ) {
        if self.shutting_down {
            return;
        }

        let Some(theme) = self
            .webviews
            .get(webview_id)
            .map(ConstellationWebView::theme)
        else {
            warn!("Tried to create Pipeline for uknown WebViewId: {webview_id:?}");
            return;
        };

        debug!(
            "{}: Creating new pipeline in {}",
            pipeline_id, browsing_context_id
        );

        let (event_loop, host) = match sandbox {
            IFrameSandboxState::IFrameSandboxed => (None, None),
            IFrameSandboxState::IFrameUnsandboxed => {
                // If this is an about:blank or about:srcdoc load, it must share the creator's
                // event loop. This must match the logic in the script thread when determining
                // the proper origin.
                if load_data.url.as_str() != "about:blank" &&
                    load_data.url.as_str() != "about:srcdoc"
                {
                    match reg_host(&load_data.url) {
                        None => (None, None),
                        Some(host) => match self.get_event_loop(&host, &webview_id, &opener) {
                            Err(err) => {
                                warn!("{}", err);
                                (None, Some(host))
                            },
                            Ok(event_loop) => {
                                if let Some(event_loop) = event_loop.upgrade() {
                                    (Some(event_loop), None)
                                } else {
                                    (None, Some(host))
                                }
                            },
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

        let result = Pipeline::spawn::<STF>(InitialPipelineState {
            id: pipeline_id,
            browsing_context_id,
            webview_id,
            parent_pipeline_id,
            opener,
            script_to_constellation_chan: ScriptToConstellationChan {
                sender: self.script_sender.clone(),
                pipeline_id,
            },
            namespace_request_sender: self.namespace_ipc_sender.clone(),
            pipeline_namespace_id: self.next_pipeline_namespace_id(),
            background_monitor_register: self.background_monitor_register.clone(),
            background_hang_monitor_to_constellation_chan: self
                .background_hang_monitor_sender
                .clone(),
            layout_factory: self.layout_factory.clone(),
            compositor_proxy: self.compositor_proxy.clone(),
            devtools_sender: self.devtools_sender.clone(),
            #[cfg(feature = "bluetooth")]
            bluetooth_thread: self.bluetooth_ipc_sender.clone(),
            swmanager_thread: self.swmanager_ipc_sender.clone(),
            system_font_service: self.system_font_service.clone(),
            resource_threads,
            time_profiler_chan: self.time_profiler_chan.clone(),
            mem_profiler_chan: self.mem_profiler_chan.clone(),
            viewport_details: initial_viewport_details,
            theme,
            event_loop,
            load_data,
            prev_throttled: throttled,
            webgl_chan: self
                .webgl_threads
                .as_ref()
                .map(|threads| threads.pipeline()),
            webxr_registry: self.webxr_registry.clone(),
            player_context: WindowGLContext::get(),
            rippy_data: self.rippy_data.clone(),
            user_content_manager: self.user_content_manager.clone(),
        });

        let pipeline = match result {
            Ok(result) => result,
            Err(e) => return self.handle_send_error(pipeline_id, e),
        };

        if let Some(chan) = pipeline.bhm_control_chan {
            self.background_monitor_control_senders.push(chan);
        }

        if let Some(host) = host {
            debug!("{}: Adding new host entry {}", webview_id, host,);
            self.set_event_loop(
                Rc::downgrade(&pipeline.pipeline.event_loop),
                host,
                webview_id,
                opener,
            );
        }

        if let Some((lifeline_receiver, process)) = pipeline.lifeline {
            let crossbeam_receiver =
                route_ipc_receiver_to_new_crossbeam_receiver_preserving_errors(lifeline_receiver);
            self.process_manager.add(crossbeam_receiver, process);
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
        webview_id: WebViewId,
    ) -> FullyActiveBrowsingContextsIterator {
        self.fully_active_descendant_browsing_contexts_iter(BrowsingContextId::from(webview_id))
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

    /// Enumerate the specified browsing context's ancestor pipelines up to
    /// the top-level pipeline.
    fn ancestor_pipelines_of_browsing_context_iter(
        &self,
        browsing_context_id: BrowsingContextId,
    ) -> impl Iterator<Item = &Pipeline> + '_ {
        let mut state: Option<PipelineId> = self
            .browsing_contexts
            .get(&browsing_context_id)
            .and_then(|browsing_context| browsing_context.parent_pipeline_id);
        std::iter::from_fn(move || {
            if let Some(pipeline_id) = state {
                let pipeline = self.pipelines.get(&pipeline_id)?;
                let browsing_context = self.browsing_contexts.get(&pipeline.browsing_context_id)?;
                state = browsing_context.parent_pipeline_id;
                Some(pipeline)
            } else {
                None
            }
        })
    }

    /// Enumerate the specified browsing context's ancestor-or-self pipelines up
    /// to the top-level pipeline.
    fn ancestor_or_self_pipelines_of_browsing_context_iter(
        &self,
        browsing_context_id: BrowsingContextId,
    ) -> impl Iterator<Item = &Pipeline> + '_ {
        let this_pipeline = self
            .browsing_contexts
            .get(&browsing_context_id)
            .map(|browsing_context| browsing_context.pipeline_id)
            .and_then(|pipeline_id| self.pipelines.get(&pipeline_id));
        this_pipeline
            .into_iter()
            .chain(self.ancestor_pipelines_of_browsing_context_iter(browsing_context_id))
    }

    /// Create a new browsing context and update the internal bookkeeping.
    #[allow(clippy::too_many_arguments)]
    fn new_browsing_context(
        &mut self,
        browsing_context_id: BrowsingContextId,
        top_level_id: WebViewId,
        pipeline_id: PipelineId,
        parent_pipeline_id: Option<PipelineId>,
        viewport_details: ViewportDetails,
        is_private: bool,
        inherited_secure_context: Option<bool>,
        throttled: bool,
    ) {
        debug!("{}: Creating new browsing context", browsing_context_id);
        let bc_group_id = match self
            .browsing_context_group_set
            .iter_mut()
            .filter_map(|(id, bc_group)| {
                if bc_group
                    .top_level_browsing_context_set
                    .contains(&top_level_id)
                {
                    Some(id)
                } else {
                    None
                }
            })
            .last()
        {
            Some(id) => *id,
            None => {
                warn!("Top-level was unexpectedly removed from its top_level_browsing_context_set");
                return;
            },
        };
        let browsing_context = BrowsingContext::new(
            bc_group_id,
            browsing_context_id,
            top_level_id,
            pipeline_id,
            parent_pipeline_id,
            viewport_details,
            is_private,
            inherited_secure_context,
            throttled,
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
        self.pending_changes.push(change);
    }

    /// Handles loading pages, navigation, and granting access to the compositor
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_request(&mut self) {
        #[derive(Debug)]
        enum Request {
            PipelineNamespace(PipelineNamespaceRequest),
            Script((PipelineId, ScriptToConstellationMessage)),
            BackgroundHangMonitor(HangMonitorAlert),
            Compositor(EmbedderToConstellationMessage),
            FromSWManager(SWManagerMsg),
            RemoveProcess(usize),
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
        let mut sel = Select::new();
        sel.recv(&self.namespace_receiver);
        sel.recv(&self.script_receiver);
        sel.recv(&self.background_hang_monitor_receiver);
        sel.recv(&self.compositor_receiver);
        sel.recv(&self.swmanager_receiver);

        self.process_manager.register(&mut sel);

        let request = {
            let oper = sel.select();
            let index = oper.index();

            #[cfg(feature = "tracing")]
            let _span =
                tracing::trace_span!("handle_request::select", servo_profiling = true).entered();
            match index {
                0 => oper
                    .recv(&self.namespace_receiver)
                    .expect("Unexpected script channel panic in constellation")
                    .map(Request::PipelineNamespace),
                1 => oper
                    .recv(&self.script_receiver)
                    .expect("Unexpected script channel panic in constellation")
                    .map(Request::Script),
                2 => oper
                    .recv(&self.background_hang_monitor_receiver)
                    .expect("Unexpected BHM channel panic in constellation")
                    .map(Request::BackgroundHangMonitor),
                3 => Ok(Request::Compositor(
                    oper.recv(&self.compositor_receiver)
                        .expect("Unexpected compositor channel panic in constellation"),
                )),
                4 => oper
                    .recv(&self.swmanager_receiver)
                    .expect("Unexpected SW channel panic in constellation")
                    .map(Request::FromSWManager),
                _ => {
                    // This can only be a error reading on a closed lifeline receiver.
                    let process_index = index - 5;
                    let _ = oper.recv(self.process_manager.receiver_at(process_index));
                    Ok(Request::RemoveProcess(process_index))
                },
            }
        };

        let request = match request {
            Ok(request) => request,
            Err(err) => return error!("Deserialization failed ({}).", err),
        };

        match request {
            Request::PipelineNamespace(message) => {
                self.handle_request_for_pipeline_namespace(message)
            },
            Request::Compositor(message) => self.handle_request_from_compositor(message),
            Request::Script(message) => {
                self.handle_request_from_script(message);
            },
            Request::BackgroundHangMonitor(message) => {
                self.handle_request_from_background_hang_monitor(message);
            },
            Request::FromSWManager(message) => {
                self.handle_request_from_swmanager(message);
            },
            Request::RemoveProcess(index) => self.process_manager.remove(index),
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_request_for_pipeline_namespace(&mut self, request: PipelineNamespaceRequest) {
        let PipelineNamespaceRequest(sender) = request;
        let _ = sender.send(self.next_pipeline_namespace_id());
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_request_from_background_hang_monitor(&self, message: HangMonitorAlert) {
        match message {
            HangMonitorAlert::Profile(bytes) => {
                self.embedder_proxy.send(EmbedderMsg::ReportProfile(bytes))
            },
            HangMonitorAlert::Hang(hang) => {
                // TODO: In case of a permanent hang being reported, add a "kill script" workflow,
                // via the embedder?
                warn!("Component hang alert: {:?}", hang);
            },
        }
    }

    fn handle_request_from_swmanager(&mut self, message: SWManagerMsg) {
        match message {
            SWManagerMsg::PostMessageToClient => {
                // TODO: implement posting a message to a SW client.
                // https://github.com/servo/servo/issues/24660
            },
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_request_from_compositor(&mut self, message: EmbedderToConstellationMessage) {
        trace_msg_from_compositor!(message, "{message:?}");
        match message {
            EmbedderToConstellationMessage::Exit => {
                self.handle_exit();
            },
            EmbedderToConstellationMessage::GetFocusTopLevelBrowsingContext(resp_chan) => {
                let focused_context = self
                    .webviews
                    .focused_webview()
                    .filter(|(_, webview)| {
                        self.browsing_contexts
                            .contains_key(&webview.focused_browsing_context_id)
                    })
                    .map(|(id, _)| id);
                let _ = resp_chan.send(focused_context);
            },
            // Perform a navigation previously requested by script, if approved by the embedder.
            // If there is already a pending page (self.pending_changes), it will not be overridden;
            // However, if the id is not encompassed by another change, it will be.
            EmbedderToConstellationMessage::AllowNavigationResponse(pipeline_id, allowed) => {
                let pending = self.pending_approval_navigations.remove(&pipeline_id);

                let webview_id = match self.pipelines.get(&pipeline_id) {
                    Some(pipeline) => pipeline.webview_id,
                    None => return warn!("{}: Attempted to navigate after closure", pipeline_id),
                };

                match pending {
                    Some((load_data, history_handling)) => {
                        if allowed {
                            self.load_url(webview_id, pipeline_id, load_data, history_handling);
                        } else {
                            let pipeline_is_top_level_pipeline = self
                                .browsing_contexts
                                .get(&BrowsingContextId::from(webview_id))
                                .map(|ctx| ctx.pipeline_id == pipeline_id)
                                .unwrap_or(false);
                            // If the navigation is refused, and this concerns an iframe,
                            // we need to take it out of it's "delaying-load-events-mode".
                            // https://html.spec.whatwg.org/multipage/#delaying-load-events-mode
                            if !pipeline_is_top_level_pipeline {
                                let msg =
                                    ScriptThreadMessage::StopDelayingLoadEventsMode(pipeline_id);
                                let result = match self.pipelines.get(&pipeline_id) {
                                    Some(pipeline) => pipeline.event_loop.send(msg),
                                    None => {
                                        return warn!(
                                            "{}: Attempted to navigate after closure",
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
                        warn!(
                            "{}: AllowNavigationResponse for unknown request",
                            pipeline_id
                        )
                    },
                }
            },
            EmbedderToConstellationMessage::ClearCache => {
                self.public_resource_threads.clear_cache();
                self.private_resource_threads.clear_cache();
            },
            // Load a new page from a typed url
            // If there is already a pending page (self.pending_changes), it will not be overridden;
            // However, if the id is not encompassed by another change, it will be.
            EmbedderToConstellationMessage::LoadUrl(webview_id, url) => {
                let load_data = LoadData::new(
                    LoadOrigin::Constellation,
                    url,
                    None,
                    Referrer::NoReferrer,
                    ReferrerPolicy::EmptyString,
                    None,
                    None,
                    false,
                );
                let ctx_id = BrowsingContextId::from(webview_id);
                let pipeline_id = match self.browsing_contexts.get(&ctx_id) {
                    Some(ctx) => ctx.pipeline_id,
                    None => {
                        return warn!("{}: LoadUrl for unknown browsing context", webview_id);
                    },
                };
                // Since this is a top-level load, initiated by the embedder, go straight to load_url,
                // bypassing schedule_navigation.
                self.load_url(
                    webview_id,
                    pipeline_id,
                    load_data,
                    NavigationHistoryBehavior::Push,
                );
            },
            EmbedderToConstellationMessage::IsReadyToSaveImage(pipeline_states) => {
                let is_ready = self.handle_is_ready_to_save_image(pipeline_states);
                debug!("Ready to save image {:?}.", is_ready);
                self.compositor_proxy
                    .send(CompositorMsg::IsReadyToSaveImageReply(
                        is_ready == ReadyToSave::Ready,
                    ));
            },
            // Create a new top level browsing context. Will use response_chan to return
            // the browsing context id.
            EmbedderToConstellationMessage::NewWebView(url, webview_id, viewport_details) => {
                self.handle_new_top_level_browsing_context(url, webview_id, viewport_details, None);
            },
            // Close a top level browsing context.
            EmbedderToConstellationMessage::CloseWebView(webview_id) => {
                self.handle_close_top_level_browsing_context(webview_id);
            },
            // Panic a top level browsing context.
            EmbedderToConstellationMessage::SendError(webview_id, error) => {
                debug!("constellation got SendError message");
                if webview_id.is_none() {
                    warn!("constellation got a SendError message without top level id");
                }
                self.handle_panic(webview_id, error, None);
            },
            EmbedderToConstellationMessage::FocusWebView(webview_id) => {
                self.handle_focus_web_view(webview_id);
            },
            EmbedderToConstellationMessage::BlurWebView => {
                self.webviews.unfocus();
                self.embedder_proxy.send(EmbedderMsg::WebViewBlurred);
            },
            // Handle a forward or back request
            EmbedderToConstellationMessage::TraverseHistory(webview_id, direction) => {
                self.handle_traverse_history_msg(webview_id, direction);
            },
            EmbedderToConstellationMessage::ChangeViewportDetails(
                webview_id,
                new_viewport_details,
                size_type,
            ) => {
                self.handle_change_viewport_details_msg(
                    webview_id,
                    new_viewport_details,
                    size_type,
                );
            },
            EmbedderToConstellationMessage::ThemeChange(webview_id, theme) => {
                self.handle_theme_change(webview_id, theme);
            },
            EmbedderToConstellationMessage::TickAnimation(webview_ids) => {
                self.handle_tick_animation(webview_ids)
            },
            EmbedderToConstellationMessage::WebDriverCommand(command) => {
                self.handle_webdriver_msg(command);
            },
            EmbedderToConstellationMessage::Reload(webview_id) => {
                self.handle_reload_msg(webview_id);
            },
            EmbedderToConstellationMessage::LogEntry(webview_id, thread_name, entry) => {
                self.handle_log_entry(webview_id, thread_name, entry);
            },
            EmbedderToConstellationMessage::ForwardInputEvent(webview_id, event, hit_test) => {
                self.forward_input_event(webview_id, event, hit_test);
            },
            EmbedderToConstellationMessage::SetCursor(webview_id, cursor) => {
                self.handle_set_cursor_msg(webview_id, cursor)
            },
            EmbedderToConstellationMessage::ToggleProfiler(rate, max_duration) => {
                for background_monitor_control_sender in &self.background_monitor_control_senders {
                    if let Err(e) = background_monitor_control_sender.send(
                        BackgroundHangMonitorControlMsg::ToggleSampler(rate, max_duration),
                    ) {
                        warn!("error communicating with sampling profiler: {}", e);
                    }
                }
            },
            EmbedderToConstellationMessage::ExitFullScreen(webview_id) => {
                self.handle_exit_fullscreen_msg(webview_id);
            },
            EmbedderToConstellationMessage::MediaSessionAction(action) => {
                self.handle_media_session_action_msg(action);
            },
            EmbedderToConstellationMessage::SetWebViewThrottled(webview_id, throttled) => {
                self.set_webview_throttled(webview_id, throttled);
            },
            EmbedderToConstellationMessage::SetScrollStates(pipeline_id, scroll_states) => {
                self.handle_set_scroll_states(pipeline_id, scroll_states)
            },
            EmbedderToConstellationMessage::PaintMetric(pipeline_id, paint_metric_event) => {
                self.handle_paint_metric(pipeline_id, paint_metric_event);
            },
            EmbedderToConstellationMessage::EvaluateJavaScript(
                webview_id,
                evaluation_id,
                script,
            ) => {
                self.handle_evaluate_javascript(webview_id, evaluation_id, script);
            },
            EmbedderToConstellationMessage::CreateMemoryReport(sender) => {
                self.mem_profiler_chan.send(ProfilerMsg::Report(sender));
            },
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_evaluate_javascript(
        &mut self,
        webview_id: WebViewId,
        evaluation_id: JavaScriptEvaluationId,
        script: String,
    ) {
        let browsing_context_id = BrowsingContextId::from(webview_id);
        let Some(pipeline) = self
            .browsing_contexts
            .get(&browsing_context_id)
            .and_then(|browsing_context| self.pipelines.get(&browsing_context.pipeline_id))
        else {
            self.handle_finish_javascript_evaluation(
                evaluation_id,
                Err(JavaScriptEvaluationError::InternalError),
            );
            return;
        };

        if pipeline
            .event_loop
            .send(ScriptThreadMessage::EvaluateJavaScript(
                pipeline.id,
                evaluation_id,
                script,
            ))
            .is_err()
        {
            self.handle_finish_javascript_evaluation(
                evaluation_id,
                Err(JavaScriptEvaluationError::InternalError),
            );
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_request_from_script(&mut self, message: (PipelineId, ScriptToConstellationMessage)) {
        let (source_pipeline_id, content) = message;
        trace_script_msg!(content, "{source_pipeline_id}: {content:?}");

        let webview_id = match self
            .pipelines
            .get(&source_pipeline_id)
            .map(|pipeline| pipeline.webview_id)
        {
            None => return warn!("{}: ScriptMsg from closed pipeline", source_pipeline_id),
            Some(ctx) => ctx,
        };

        match content {
            ScriptToConstellationMessage::CompleteMessagePortTransfer(router_id, ports) => {
                self.handle_complete_message_port_transfer(router_id, ports);
            },
            ScriptToConstellationMessage::MessagePortTransferResult(
                router_id,
                succeeded,
                failed,
            ) => {
                self.handle_message_port_transfer_completed(router_id, succeeded);
                self.handle_message_port_transfer_failed(failed);
            },
            ScriptToConstellationMessage::RerouteMessagePort(port_id, task) => {
                self.handle_reroute_messageport(port_id, task);
            },
            ScriptToConstellationMessage::MessagePortShipped(port_id) => {
                self.handle_messageport_shipped(port_id);
            },
            ScriptToConstellationMessage::NewMessagePortRouter(router_id, ipc_sender) => {
                self.handle_new_messageport_router(router_id, ipc_sender);
            },
            ScriptToConstellationMessage::RemoveMessagePortRouter(router_id) => {
                self.handle_remove_messageport_router(router_id);
            },
            ScriptToConstellationMessage::NewMessagePort(router_id, port_id) => {
                self.handle_new_messageport(router_id, port_id);
            },
            ScriptToConstellationMessage::EntanglePorts(port1, port2) => {
                self.handle_entangle_messageports(port1, port2);
            },
            ScriptToConstellationMessage::DisentanglePorts(port1, port2) => {
                self.handle_disentangle_messageports(port1, port2);
            },
            ScriptToConstellationMessage::NewBroadcastChannelRouter(
                router_id,
                response_sender,
                origin,
            ) => {
                self.handle_new_broadcast_channel_router(
                    source_pipeline_id,
                    router_id,
                    response_sender,
                    origin,
                );
            },
            ScriptToConstellationMessage::NewBroadcastChannelNameInRouter(
                router_id,
                channel_name,
                origin,
            ) => {
                self.handle_new_broadcast_channel_name_in_router(
                    source_pipeline_id,
                    router_id,
                    channel_name,
                    origin,
                );
            },
            ScriptToConstellationMessage::RemoveBroadcastChannelNameInRouter(
                router_id,
                channel_name,
                origin,
            ) => {
                self.handle_remove_broadcast_channel_name_in_router(
                    source_pipeline_id,
                    router_id,
                    channel_name,
                    origin,
                );
            },
            ScriptToConstellationMessage::RemoveBroadcastChannelRouter(router_id, origin) => {
                self.handle_remove_broadcast_channel_router(source_pipeline_id, router_id, origin);
            },
            ScriptToConstellationMessage::ScheduleBroadcast(router_id, message) => {
                self.handle_schedule_broadcast(source_pipeline_id, router_id, message);
            },
            ScriptToConstellationMessage::ForwardToEmbedder(embedder_msg) => {
                self.embedder_proxy.send(embedder_msg);
            },
            ScriptToConstellationMessage::PipelineExited => {
                self.handle_pipeline_exited(source_pipeline_id);
            },
            ScriptToConstellationMessage::DiscardDocument => {
                self.handle_discard_document(webview_id, source_pipeline_id);
            },
            ScriptToConstellationMessage::DiscardTopLevelBrowsingContext => {
                self.handle_close_top_level_browsing_context(webview_id);
            },
            ScriptToConstellationMessage::ScriptLoadedURLInIFrame(load_info) => {
                self.handle_script_loaded_url_in_iframe_msg(load_info);
            },
            ScriptToConstellationMessage::ScriptNewIFrame(load_info) => {
                self.handle_script_new_iframe(load_info);
            },
            ScriptToConstellationMessage::CreateAuxiliaryWebView(load_info) => {
                self.handle_script_new_auxiliary(load_info);
            },
            ScriptToConstellationMessage::ChangeRunningAnimationsState(animation_state) => {
                self.handle_change_running_animations_state(source_pipeline_id, animation_state)
            },
            // Ask the embedder for permission to load a new page.
            ScriptToConstellationMessage::LoadUrl(load_data, history_handling) => {
                self.schedule_navigation(
                    webview_id,
                    source_pipeline_id,
                    load_data,
                    history_handling,
                );
            },
            ScriptToConstellationMessage::AbortLoadUrl => {
                self.handle_abort_load_url_msg(source_pipeline_id);
            },
            // A page loaded has completed all parsing, script, and reflow messages have been sent.
            ScriptToConstellationMessage::LoadComplete => {
                self.handle_load_complete_msg(webview_id, source_pipeline_id)
            },
            // Handle navigating to a fragment
            ScriptToConstellationMessage::NavigatedToFragment(new_url, replacement_enabled) => {
                self.handle_navigated_to_fragment(source_pipeline_id, new_url, replacement_enabled);
            },
            // Handle a forward or back request
            ScriptToConstellationMessage::TraverseHistory(direction) => {
                self.handle_traverse_history_msg(webview_id, direction);
            },
            // Handle a push history state request.
            ScriptToConstellationMessage::PushHistoryState(history_state_id, url) => {
                self.handle_push_history_state_msg(source_pipeline_id, history_state_id, url);
            },
            ScriptToConstellationMessage::ReplaceHistoryState(history_state_id, url) => {
                self.handle_replace_history_state_msg(source_pipeline_id, history_state_id, url);
            },
            // Handle a joint session history length request.
            ScriptToConstellationMessage::JointSessionHistoryLength(response_sender) => {
                self.handle_joint_session_history_length(webview_id, response_sender);
            },
            // Notification that the new document is ready to become active
            ScriptToConstellationMessage::ActivateDocument => {
                self.handle_activate_document_msg(source_pipeline_id);
            },
            // Update pipeline url after redirections
            ScriptToConstellationMessage::SetFinalUrl(final_url) => {
                // The script may have finished loading after we already started shutting down.
                if let Some(ref mut pipeline) = self.pipelines.get_mut(&source_pipeline_id) {
                    pipeline.url = final_url;
                } else {
                    warn!("constellation got set final url message for dead pipeline");
                }
            },
            ScriptToConstellationMessage::PostMessage {
                target: browsing_context_id,
                source: source_pipeline_id,
                target_origin: origin,
                source_origin,
                data,
            } => {
                self.handle_post_message_msg(
                    browsing_context_id,
                    source_pipeline_id,
                    origin,
                    source_origin,
                    data,
                );
            },
            ScriptToConstellationMessage::Focus(focused_child_browsing_context_id, sequence) => {
                self.handle_focus_msg(
                    source_pipeline_id,
                    focused_child_browsing_context_id,
                    sequence,
                );
            },
            ScriptToConstellationMessage::FocusRemoteDocument(focused_browsing_context_id) => {
                self.handle_focus_remote_document_msg(focused_browsing_context_id);
            },
            ScriptToConstellationMessage::SetThrottledComplete(throttled) => {
                self.handle_set_throttled_complete(source_pipeline_id, throttled);
            },
            ScriptToConstellationMessage::RemoveIFrame(browsing_context_id, response_sender) => {
                let removed_pipeline_ids = self.handle_remove_iframe_msg(browsing_context_id);
                if let Err(e) = response_sender.send(removed_pipeline_ids) {
                    warn!("Error replying to remove iframe ({})", e);
                }
            },
            ScriptToConstellationMessage::CreateCanvasPaintThread(size, response_sender) => {
                self.handle_create_canvas_paint_thread_msg(size, response_sender)
            },
            ScriptToConstellationMessage::SetDocumentState(state) => {
                self.document_states.insert(source_pipeline_id, state);
            },
            ScriptToConstellationMessage::SetLayoutEpoch(epoch, response_sender) => {
                if let Some(pipeline) = self.pipelines.get_mut(&source_pipeline_id) {
                    pipeline.layout_epoch = epoch;
                }

                response_sender.send(true).unwrap_or_default();
            },
            ScriptToConstellationMessage::LogEntry(thread_name, entry) => {
                self.handle_log_entry(Some(webview_id), thread_name, entry);
            },
            ScriptToConstellationMessage::TouchEventProcessed(result) => self
                .compositor_proxy
                .send(CompositorMsg::TouchEventProcessed(webview_id, result)),
            ScriptToConstellationMessage::GetBrowsingContextInfo(pipeline_id, response_sender) => {
                let result = self
                    .pipelines
                    .get(&pipeline_id)
                    .and_then(|pipeline| self.browsing_contexts.get(&pipeline.browsing_context_id))
                    .map(|ctx| (ctx.id, ctx.parent_pipeline_id));
                if let Err(e) = response_sender.send(result) {
                    warn!(
                        "Sending reply to get browsing context info failed ({:?}).",
                        e
                    );
                }
            },
            ScriptToConstellationMessage::GetTopForBrowsingContext(
                browsing_context_id,
                response_sender,
            ) => {
                let result = self
                    .browsing_contexts
                    .get(&browsing_context_id)
                    .map(|bc| bc.top_level_id);
                if let Err(e) = response_sender.send(result) {
                    warn!(
                        "Sending reply to get top for browsing context info failed ({:?}).",
                        e
                    );
                }
            },
            ScriptToConstellationMessage::GetChildBrowsingContextId(
                browsing_context_id,
                index,
                response_sender,
            ) => {
                let result = self
                    .browsing_contexts
                    .get(&browsing_context_id)
                    .and_then(|bc| self.pipelines.get(&bc.pipeline_id))
                    .and_then(|pipeline| pipeline.children.get(index))
                    .copied();
                if let Err(e) = response_sender.send(result) {
                    warn!(
                        "Sending reply to get child browsing context ID failed ({:?}).",
                        e
                    );
                }
            },
            ScriptToConstellationMessage::ScheduleJob(job) => {
                self.handle_schedule_serviceworker_job(source_pipeline_id, job);
            },
            ScriptToConstellationMessage::ForwardDOMMessage(msg_vec, scope_url) => {
                if let Some(mgr) = self.sw_managers.get(&scope_url.origin()) {
                    let _ = mgr.send(ServiceWorkerMsg::ForwardDOMMessage(msg_vec, scope_url));
                } else {
                    warn!("Unable to forward DOMMessage for postMessage call");
                }
            },
            ScriptToConstellationMessage::BroadcastStorageEvent(
                storage,
                url,
                key,
                old_value,
                new_value,
            ) => {
                self.handle_broadcast_storage_event(
                    source_pipeline_id,
                    storage,
                    url,
                    key,
                    old_value,
                    new_value,
                );
            },
            ScriptToConstellationMessage::MediaSessionEvent(pipeline_id, event) => {
                // Unlikely at this point, but we may receive events coming from
                // different media sessions, so we set the active media session based
                // on Playing events.
                // The last media session claiming to be in playing state is set to
                // the active media session.
                // Events coming from inactive media sessions are discarded.
                if self.active_media_session.is_some() {
                    if let MediaSessionEvent::PlaybackStateChange(ref state) = event {
                        if !matches!(
                            state,
                            MediaSessionPlaybackState::Playing | MediaSessionPlaybackState::Paused
                        ) {
                            return;
                        }
                    };
                }
                self.active_media_session = Some(pipeline_id);
                self.embedder_proxy
                    .send(EmbedderMsg::MediaSessionEvent(webview_id, event));
            },
            #[cfg(feature = "webgpu")]
            ScriptToConstellationMessage::RequestAdapter(response_sender, options, ids) => self
                .handle_wgpu_request(
                    source_pipeline_id,
                    BrowsingContextId::from(webview_id),
                    ScriptToConstellationMessage::RequestAdapter(response_sender, options, ids),
                ),
            #[cfg(feature = "webgpu")]
            ScriptToConstellationMessage::GetWebGPUChan(response_sender) => self
                .handle_wgpu_request(
                    source_pipeline_id,
                    BrowsingContextId::from(webview_id),
                    ScriptToConstellationMessage::GetWebGPUChan(response_sender),
                ),
            ScriptToConstellationMessage::TitleChanged(pipeline, title) => {
                if let Some(pipeline) = self.pipelines.get_mut(&pipeline) {
                    pipeline.title = title;
                }
            },
            ScriptToConstellationMessage::IFrameSizes(iframe_sizes) => {
                self.handle_iframe_size_msg(iframe_sizes)
            },
            ScriptToConstellationMessage::ReportMemory(sender) => {
                // get memory report and send it back.
                self.mem_profiler_chan
                    .send(mem::ProfilerMsg::Report(sender));
            },
            ScriptToConstellationMessage::FinishJavaScriptEvaluation(evaluation_id, result) => {
                self.handle_finish_javascript_evaluation(evaluation_id, result)
            },
            ScriptToConstellationMessage::WebDriverInputComplete(msg_id) => {
                if let Some(ref reply_sender) = self.webdriver.input_command_response_sender {
                    reply_sender
                        .send(WebDriverCommandResponse { id: msg_id })
                        .unwrap_or_else(|_| {
                            warn!("Failed to send WebDriverInputComplete {:?}", msg_id);
                            self.webdriver.input_command_response_sender = None;
                        });
                } else {
                    warn!("No WebDriver input_command_response_sender");
                }
            },
        }
    }

    /// Check the origin of a message against that of the pipeline it came from.
    /// Note: this is still limited as a security check,
    /// see <https://github.com/servo/servo/issues/11722>
    fn check_origin_against_pipeline(
        &self,
        pipeline_id: &PipelineId,
        origin: &ImmutableOrigin,
    ) -> Result<(), ()> {
        let pipeline_origin = match self.pipelines.get(pipeline_id) {
            Some(pipeline) => pipeline.load_data.url.origin(),
            None => {
                warn!("Received message from closed or unknown pipeline.");
                return Err(());
            },
        };
        if &pipeline_origin == origin {
            return Ok(());
        }
        Err(())
    }

    /// Broadcast a message via routers in various event-loops.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_schedule_broadcast(
        &self,
        pipeline_id: PipelineId,
        router_id: BroadcastChannelRouterId,
        message: BroadcastMsg,
    ) {
        if self
            .check_origin_against_pipeline(&pipeline_id, &message.origin)
            .is_err()
        {
            return warn!(
                "Attempt to schedule broadcast from an origin not matching the origin of the msg."
            );
        }
        if let Some(channels) = self.broadcast_channels.get(&message.origin) {
            let routers = match channels.get(&message.channel_name) {
                Some(routers) => routers,
                None => return warn!("Broadcast to channel name without active routers."),
            };
            for router in routers {
                // Exclude the sender of the broadcast.
                // Broadcasting locally is done at the point of sending.
                if router == &router_id {
                    continue;
                }

                if let Some(broadcast_ipc_sender) = self.broadcast_routers.get(router) {
                    if broadcast_ipc_sender.send(message.clone()).is_err() {
                        warn!("Failed to broadcast message to router: {:?}", router);
                    }
                } else {
                    warn!("No sender for broadcast router: {:?}", router);
                }
            }
        } else {
            warn!(
                "Attempt to schedule a broadcast for an origin without routers {:?}",
                message.origin
            );
        }
    }

    /// Remove a channel-name for a given broadcast router.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_remove_broadcast_channel_name_in_router(
        &mut self,
        pipeline_id: PipelineId,
        router_id: BroadcastChannelRouterId,
        channel_name: String,
        origin: ImmutableOrigin,
    ) {
        if self
            .check_origin_against_pipeline(&pipeline_id, &origin)
            .is_err()
        {
            return warn!("Attempt to remove channel name from an unexpected origin.");
        }
        if let Some(channels) = self.broadcast_channels.get_mut(&origin) {
            let is_empty = if let Some(routers) = channels.get_mut(&channel_name) {
                routers.retain(|router| router != &router_id);
                routers.is_empty()
            } else {
                return warn!(
                    "Multiple attempts to remove name for broadcast-channel {:?} at {:?}",
                    channel_name, origin
                );
            };
            if is_empty {
                channels.remove(&channel_name);
            }
        } else {
            warn!(
                "Attempt to remove a channel-name for an origin without channels {:?}",
                origin
            );
        }
    }

    /// Note a new channel-name relevant to a given broadcast router.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_new_broadcast_channel_name_in_router(
        &mut self,
        pipeline_id: PipelineId,
        router_id: BroadcastChannelRouterId,
        channel_name: String,
        origin: ImmutableOrigin,
    ) {
        if self
            .check_origin_against_pipeline(&pipeline_id, &origin)
            .is_err()
        {
            return warn!("Attempt to add channel name from an unexpected origin.");
        }
        let channels = self.broadcast_channels.entry(origin).or_default();

        let routers = channels.entry(channel_name).or_default();

        routers.push(router_id);
    }

    /// Remove a broadcast router.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_remove_broadcast_channel_router(
        &mut self,
        pipeline_id: PipelineId,
        router_id: BroadcastChannelRouterId,
        origin: ImmutableOrigin,
    ) {
        if self
            .check_origin_against_pipeline(&pipeline_id, &origin)
            .is_err()
        {
            return warn!("Attempt to remove broadcast router from an unexpected origin.");
        }
        if self.broadcast_routers.remove(&router_id).is_none() {
            warn!("Attempt to remove unknown broadcast-channel router.");
        }
    }

    /// Add a new broadcast router.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_new_broadcast_channel_router(
        &mut self,
        pipeline_id: PipelineId,
        router_id: BroadcastChannelRouterId,
        broadcast_ipc_sender: IpcSender<BroadcastMsg>,
        origin: ImmutableOrigin,
    ) {
        if self
            .check_origin_against_pipeline(&pipeline_id, &origin)
            .is_err()
        {
            return warn!("Attempt to add broadcast router from an unexpected origin.");
        }
        if self
            .broadcast_routers
            .insert(router_id, broadcast_ipc_sender)
            .is_some()
        {
            warn!("Multple attempt to add broadcast-channel router.");
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    #[cfg(feature = "webgpu")]
    fn handle_wgpu_request(
        &mut self,
        source_pipeline_id: PipelineId,
        browsing_context_id: BrowsingContextId,
        request: ScriptToConstellationMessage,
    ) {
        use webgpu::start_webgpu_thread;

        let browsing_context_group_id = match self.browsing_contexts.get(&browsing_context_id) {
            Some(bc) => &bc.bc_group_id,
            None => return warn!("Browsing context not found"),
        };
        let source_pipeline = match self.pipelines.get(&source_pipeline_id) {
            Some(pipeline) => pipeline,
            None => return warn!("{}: ScriptMsg from closed pipeline", source_pipeline_id),
        };
        let host = match reg_host(&source_pipeline.url) {
            Some(host) => host,
            None => return warn!("Invalid host url"),
        };
        let browsing_context_group = if let Some(bcg) = self
            .browsing_context_group_set
            .get_mut(browsing_context_group_id)
        {
            bcg
        } else {
            return warn!("Browsing context group not found");
        };
        let webgpu_chan = match browsing_context_group.webgpus.entry(host) {
            Entry::Vacant(v) => start_webgpu_thread(
                self.webrender_wgpu.webrender_api.create_sender(),
                self.webrender_document,
                self.webrender_wgpu.webrender_external_images.clone(),
                self.webrender_wgpu.wgpu_image_map.clone(),
            )
            .map(|webgpu| {
                let msg = ScriptThreadMessage::SetWebGPUPort(webgpu.1);
                if let Err(e) = source_pipeline.event_loop.send(msg) {
                    warn!(
                        "{}: Failed to send SetWebGPUPort to pipeline ({:?})",
                        source_pipeline_id, e
                    );
                }
                v.insert(webgpu.0).clone()
            }),
            Entry::Occupied(o) => Some(o.get().clone()),
        };
        match request {
            ScriptToConstellationMessage::RequestAdapter(response_sender, options, adapter_id) => {
                match webgpu_chan {
                    None => {
                        if let Err(e) = response_sender.send(None) {
                            warn!("Failed to send request adapter message: {}", e)
                        }
                    },
                    Some(webgpu_chan) => {
                        let adapter_request = WebGPURequest::RequestAdapter {
                            sender: response_sender,
                            options,
                            adapter_id,
                        };
                        if webgpu_chan.0.send(adapter_request).is_err() {
                            warn!("Failed to send request adapter message on WebGPU channel");
                        }
                    },
                }
            },
            ScriptToConstellationMessage::GetWebGPUChan(response_sender) => {
                if response_sender.send(webgpu_chan).is_err() {
                    warn!(
                        "{}: Failed to send WebGPU channel to pipeline",
                        source_pipeline_id
                    )
                }
            },
            _ => warn!("Wrong message type in handle_wgpu_request"),
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_message_port_transfer_completed(
        &mut self,
        router_id: Option<MessagePortRouterId>,
        ports: Vec<MessagePortId>,
    ) {
        let router_id = match router_id {
            Some(router_id) => router_id,
            None => {
                if !ports.is_empty() {
                    warn!(
                        "Constellation unable to process port transfer successes, since no router id was received"
                    );
                }
                return;
            },
        };
        for port_id in ports.into_iter() {
            let mut entry = match self.message_ports.entry(port_id) {
                Entry::Vacant(_) => {
                    warn!(
                        "Constellation received a port transfer completed msg for unknown messageport {:?}",
                        port_id
                    );
                    continue;
                },
                Entry::Occupied(entry) => entry,
            };
            match entry.get().state {
                TransferState::CompletionInProgress(expected_router_id) => {
                    // Here, the transfer was normally completed.

                    if expected_router_id != router_id {
                        return warn!(
                            "Transfer completed by an unexpected router: {:?}",
                            router_id
                        );
                    }
                    // Update the state to managed.
                    let new_info = MessagePortInfo {
                        state: TransferState::Managed(router_id),
                        entangled_with: entry.get().entangled_with,
                    };
                    entry.insert(new_info);
                },
                _ => warn!("Constellation received unexpected port transfer completed message"),
            }
        }
    }

    fn handle_message_port_transfer_failed(
        &mut self,
        ports: HashMap<MessagePortId, PortTransferInfo>,
    ) {
        for (port_id, mut transfer_info) in ports.into_iter() {
            let entry = match self.message_ports.remove(&port_id) {
                None => {
                    warn!(
                        "Constellation received a port transfer completed msg for unknown messageport {:?}",
                        port_id
                    );
                    continue;
                },
                Some(entry) => entry,
            };
            let new_info = match entry.state {
                TransferState::CompletionFailed(mut current_buffer) => {
                    // The transfer failed,
                    // and now the global has returned us the buffer we previously sent.
                    // So the next update is back to a "normal" transfer in progress.

                    // Tasks in the previous buffer are older,
                    // hence need to be added to the front of the current one.
                    while let Some(task) = transfer_info.port_message_queue.pop_back() {
                        current_buffer.push_front(task);
                    }
                    // Update the state to transfer-in-progress.
                    MessagePortInfo {
                        state: TransferState::TransferInProgress(current_buffer),
                        entangled_with: entry.entangled_with,
                    }
                },
                TransferState::CompletionRequested(target_router_id, mut current_buffer) => {
                    // Here, before the global who failed the last transfer could return us the buffer,
                    // another global already sent us a request to complete a new transfer.
                    // So we use the returned buffer to update
                    // the current-buffer(of new incoming messages),
                    // and we send everything to the global
                    // who is waiting for completion of the current transfer.

                    // Tasks in the previous buffer are older,
                    // hence need to be added to the front of the current one.
                    while let Some(task) = transfer_info.port_message_queue.pop_back() {
                        current_buffer.push_front(task);
                    }
                    // Forward the buffered message-queue to complete the current transfer.
                    if let Some(ipc_sender) = self.message_port_routers.get(&target_router_id) {
                        if ipc_sender
                            .send(MessagePortMsg::CompletePendingTransfer(
                                port_id,
                                PortTransferInfo {
                                    port_message_queue: current_buffer,
                                    disentangled: entry.entangled_with.is_none(),
                                },
                            ))
                            .is_err()
                        {
                            warn!("Constellation failed to send complete port transfer response.");
                        }
                    } else {
                        warn!("No message-port sender for {:?}", target_router_id);
                    }

                    // Update the state to completion-in-progress.
                    MessagePortInfo {
                        state: TransferState::CompletionInProgress(target_router_id),
                        entangled_with: entry.entangled_with,
                    }
                },
                _ => {
                    warn!("Unexpected port transfer failed message received");
                    continue;
                },
            };
            self.message_ports.insert(port_id, new_info);
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_complete_message_port_transfer(
        &mut self,
        router_id: MessagePortRouterId,
        ports: Vec<MessagePortId>,
    ) {
        let mut response = HashMap::new();
        for port_id in ports.into_iter() {
            let entry = match self.message_ports.remove(&port_id) {
                None => {
                    warn!(
                        "Constellation asked to complete transfer for unknown messageport {:?}",
                        port_id
                    );
                    continue;
                },
                Some(entry) => entry,
            };
            let new_info = match entry.state {
                TransferState::TransferInProgress(buffer) => {
                    response.insert(
                        port_id,
                        PortTransferInfo {
                            port_message_queue: buffer,
                            disentangled: entry.entangled_with.is_none(),
                        },
                    );

                    // If the port was in transfer, and a global is requesting completion,
                    // we note the start of the completion.
                    MessagePortInfo {
                        state: TransferState::CompletionInProgress(router_id),
                        entangled_with: entry.entangled_with,
                    }
                },
                TransferState::CompletionFailed(buffer) |
                TransferState::CompletionRequested(_, buffer) => {
                    // If the completion had already failed,
                    // this is a request coming from a global to complete a new transfer,
                    // but we're still awaiting the return of the buffer
                    // from the first global who failed.
                    //
                    // So we note the request from the new global,
                    // and continue to buffer incoming messages
                    // and wait for the buffer used in the previous transfer to be returned.
                    //
                    // If another global requests completion in the CompletionRequested state,
                    // we simply swap the target router-id for the new one,
                    // keeping the buffer.
                    MessagePortInfo {
                        state: TransferState::CompletionRequested(router_id, buffer),
                        entangled_with: entry.entangled_with,
                    }
                },
                _ => {
                    warn!("Unexpected complete port transfer message received");
                    continue;
                },
            };
            self.message_ports.insert(port_id, new_info);
        }

        if !response.is_empty() {
            // Forward the buffered message-queue.
            if let Some(ipc_sender) = self.message_port_routers.get(&router_id) {
                if ipc_sender
                    .send(MessagePortMsg::CompleteTransfer(response))
                    .is_err()
                {
                    warn!("Constellation failed to send complete port transfer response.");
                }
            } else {
                warn!("No message-port sender for {:?}", router_id);
            }
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_reroute_messageport(&mut self, port_id: MessagePortId, task: PortMessageTask) {
        let info = match self.message_ports.get_mut(&port_id) {
            Some(info) => info,
            None => {
                return warn!(
                    "Constellation asked to re-route msg to unknown messageport {:?}",
                    port_id
                );
            },
        };
        match &mut info.state {
            TransferState::Managed(router_id) | TransferState::CompletionInProgress(router_id) => {
                // In both the managed and completion of a transfer case, we forward the message.
                // Note that in both cases, if the port is transferred before the message is handled,
                // it will be sent back here and buffered while the transfer is ongoing.
                if let Some(ipc_sender) = self.message_port_routers.get(router_id) {
                    let _ = ipc_sender.send(MessagePortMsg::NewTask(port_id, task));
                } else {
                    warn!("No message-port sender for {:?}", router_id);
                }
            },
            TransferState::TransferInProgress(queue) => queue.push_back(task),
            TransferState::CompletionFailed(queue) => queue.push_back(task),
            TransferState::CompletionRequested(_, queue) => queue.push_back(task),
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_messageport_shipped(&mut self, port_id: MessagePortId) {
        if let Some(info) = self.message_ports.get_mut(&port_id) {
            match info.state {
                TransferState::Managed(_) => {
                    // If shipped while managed, note the start of a transfer.
                    info.state = TransferState::TransferInProgress(VecDeque::new());
                },
                TransferState::CompletionInProgress(_) => {
                    // If shipped while completion of a transfer was in progress,
                    // the completion failed.
                    // This will be followed by a MessagePortTransferFailed message,
                    // containing the buffer we previously sent.
                    info.state = TransferState::CompletionFailed(VecDeque::new());
                },
                _ => warn!("Unexpected messageport shipped received"),
            }
        } else {
            warn!(
                "Constellation asked to mark unknown messageport as shipped {:?}",
                port_id
            );
        }
    }

    fn handle_new_messageport_router(
        &mut self,
        router_id: MessagePortRouterId,
        message_port_ipc_sender: IpcSender<MessagePortMsg>,
    ) {
        self.message_port_routers
            .insert(router_id, message_port_ipc_sender);
    }

    fn handle_remove_messageport_router(&mut self, router_id: MessagePortRouterId) {
        self.message_port_routers.remove(&router_id);
    }

    fn handle_new_messageport(&mut self, router_id: MessagePortRouterId, port_id: MessagePortId) {
        match self.message_ports.entry(port_id) {
            // If it's a new port, we should not know about it.
            Entry::Occupied(_) => warn!(
                "Constellation asked to start tracking an existing messageport {:?}",
                port_id
            ),
            Entry::Vacant(entry) => {
                let info = MessagePortInfo {
                    state: TransferState::Managed(router_id),
                    entangled_with: None,
                };
                entry.insert(info);
            },
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_entangle_messageports(&mut self, port1: MessagePortId, port2: MessagePortId) {
        if let Some(info) = self.message_ports.get_mut(&port1) {
            info.entangled_with = Some(port2);
        } else {
            warn!(
                "Constellation asked to entangle unknown messageport: {:?}",
                port1
            );
        }
        if let Some(info) = self.message_ports.get_mut(&port2) {
            info.entangled_with = Some(port1);
        } else {
            warn!(
                "Constellation asked to entangle unknown messageport: {:?}",
                port2
            );
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    /// <https://html.spec.whatwg.org/multipage/#disentangle>
    fn handle_disentangle_messageports(
        &mut self,
        port1: MessagePortId,
        port2: Option<MessagePortId>,
    ) {
        // Disentangle initiatorPort and otherPort,
        // so that they are no longer entangled or associated with each other.
        // Note: If `port2` is some, then this is the first message
        // and `port1` is the initiatorPort, `port2` is the otherPort.
        // We can immediately remove the initiator.
        let _ = self.message_ports.remove(&port1);

        // Note: the none case is when otherPort sent this message
        // in response to completing its own local disentanglement.
        let Some(port2) = port2 else {
            return;
        };

        // Start disentanglement of the other port.
        if let Some(info) = self.message_ports.get_mut(&port2) {
            info.entangled_with = None;
            match &mut info.state {
                TransferState::Managed(router_id) |
                TransferState::CompletionInProgress(router_id) => {
                    // We try to disentangle the other port now,
                    // and if it has been transfered out by the time the message is received,
                    // it will be ignored,
                    // and disentanglement will be completed as part of the transfer.
                    if let Some(ipc_sender) = self.message_port_routers.get(router_id) {
                        let _ = ipc_sender.send(MessagePortMsg::CompleteDisentanglement(port2));
                    } else {
                        warn!("No message-port sender for {:?}", router_id);
                    }
                },
                _ => {
                    // Note: the port is in transfer, disentanglement will complete along with it.
                },
            }
        } else {
            warn!(
                "Constellation asked to disentangle unknown messageport: {:?}",
                port2
            );
        }
    }

    /// <https://w3c.github.io/ServiceWorker/#schedule-job-algorithm>
    /// and
    /// <https://w3c.github.io/ServiceWorker/#dfn-job-queue>
    ///
    /// The Job Queue is essentially the channel to a SW manager,
    /// which are scoped per origin.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_schedule_serviceworker_job(&mut self, pipeline_id: PipelineId, job: Job) {
        let origin = job.scope_url.origin();

        if self
            .check_origin_against_pipeline(&pipeline_id, &origin)
            .is_err()
        {
            return warn!(
                "Attempt to schedule a serviceworker job from an origin not matching the origin of the job."
            );
        }

        // This match is equivalent to Entry.or_insert_with but allows for early return.
        let sw_manager = match self.sw_managers.entry(origin.clone()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let (own_sender, receiver) = ipc::channel().expect("Failed to create IPC channel!");

                let sw_senders = SWManagerSenders {
                    swmanager_sender: self.swmanager_ipc_sender.clone(),
                    resource_sender: self.public_resource_threads.sender(),
                    own_sender: own_sender.clone(),
                    receiver,
                };

                if opts::get().multiprocess {
                    let (sender, receiver) =
                        ipc::channel().expect("Failed to create lifeline channel for sw");
                    let content =
                        ServiceWorkerUnprivilegedContent::new(sw_senders, origin, Some(sender));

                    if let Ok(process) = content.spawn_multiprocess() {
                        let crossbeam_receiver =
                            route_ipc_receiver_to_new_crossbeam_receiver_preserving_errors(
                                receiver,
                            );
                        self.process_manager.add(crossbeam_receiver, process);
                    } else {
                        return warn!("Failed to spawn process for SW manager.");
                    }
                } else {
                    let content = ServiceWorkerUnprivilegedContent::new(sw_senders, origin, None);
                    content.start::<SWF>();
                }
                entry.insert(own_sender)
            },
        };
        let _ = sw_manager.send(ServiceWorkerMsg::ScheduleJob(job));
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
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
                let msg = ScriptThreadMessage::DispatchStorageEvent(
                    pipeline.id,
                    storage,
                    url.clone(),
                    key.clone(),
                    old_value.clone(),
                    new_value.clone(),
                );
                if let Err(err) = pipeline.event_loop.send(msg) {
                    warn!(
                        "{}: Failed to broadcast storage event to pipeline ({:?}).",
                        pipeline.id, err
                    );
                }
            }
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_exit(&mut self) {
        debug!("Handling exit.");

        // TODO: add a timer, which forces shutdown if threads aren't responsive.
        if self.shutting_down {
            return;
        }
        self.shutting_down = true;

        self.mem_profiler_chan.send(mem::ProfilerMsg::Exit);

        // Tell all BHMs to exit,
        // and to ensure their monitored components exit
        // even when currently hanging(on JS or sync XHR).
        // This must be done before starting the process of closing all pipelines.
        for chan in &self.background_monitor_control_senders {
            let (exit_ipc_sender, exit_ipc_receiver) =
                ipc::channel().expect("Failed to create IPC channel!");
            if let Err(e) = chan.send(BackgroundHangMonitorControlMsg::Exit(exit_ipc_sender)) {
                warn!("error communicating with bhm: {}", e);
                continue;
            }
            if exit_ipc_receiver.recv().is_err() {
                warn!("Failed to receive exit confirmation from BHM.");
            }
        }

        // Close the top-level browsing contexts
        let browsing_context_ids: Vec<BrowsingContextId> = self
            .browsing_contexts
            .values()
            .filter(|browsing_context| browsing_context.is_top_level())
            .map(|browsing_context| browsing_context.id)
            .collect();
        for browsing_context_id in browsing_context_ids {
            debug!(
                "{}: Removing top-level browsing context",
                browsing_context_id
            );
            self.close_browsing_context(browsing_context_id, ExitPipelineMode::Normal);
        }

        // Close any pending changes and pipelines
        while let Some(pending) = self.pending_changes.pop() {
            debug!(
                "{}: Removing pending browsing context",
                pending.browsing_context_id
            );
            self.close_browsing_context(pending.browsing_context_id, ExitPipelineMode::Normal);
            debug!("{}: Removing pending pipeline", pending.new_pipeline_id);
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
                "{}: Removing detached browsing context",
                browsing_context_id
            );
            self.close_browsing_context(browsing_context_id, ExitPipelineMode::Normal);
        }

        // In case there are pipelines which weren't attached to the pipeline tree, we close them.
        let pipeline_ids: Vec<PipelineId> = self.pipelines.keys().cloned().collect();
        for pipeline_id in pipeline_ids {
            debug!("{}: Removing detached pipeline", pipeline_id);
            self.close_pipeline(
                pipeline_id,
                DiscardBrowsingContext::Yes,
                ExitPipelineMode::Normal,
            );
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_shutdown(&mut self) {
        debug!("Handling shutdown.");

        // At this point, there are no active pipelines,
        // so we can safely block on other threads, without worrying about deadlock.
        // Channels to receive signals when threads are done exiting.
        let (core_ipc_sender, core_ipc_receiver) =
            ipc::channel().expect("Failed to create IPC channel!");
        let (storage_ipc_sender, storage_ipc_receiver) =
            ipc::channel().expect("Failed to create IPC channel!");
        let mut webgl_threads_receiver = None;

        debug!("Exiting core resource threads.");
        if let Err(e) = self
            .public_resource_threads
            .send(net_traits::CoreResourceMsg::Exit(core_ipc_sender))
        {
            warn!("Exit resource thread failed ({})", e);
        }

        if let Some(ref chan) = self.devtools_sender {
            debug!("Exiting devtools.");
            let msg = DevtoolsControlMsg::FromChrome(ChromeToDevtoolsControlMsg::ServerExitMsg);
            if let Err(e) = chan.send(msg) {
                warn!("Exit devtools failed ({:?})", e);
            }
        }

        debug!("Exiting storage resource threads.");
        if let Err(e) = self
            .public_resource_threads
            .send(StorageThreadMsg::Exit(storage_ipc_sender))
        {
            warn!("Exit storage thread failed ({})", e);
        }

        #[cfg(feature = "bluetooth")]
        {
            debug!("Exiting bluetooth thread.");
            if let Err(e) = self.bluetooth_ipc_sender.send(BluetoothRequest::Exit) {
                warn!("Exit bluetooth thread failed ({})", e);
            }
        }

        debug!("Exiting service worker manager thread.");
        for (_, mgr) in self.sw_managers.drain() {
            if let Err(e) = mgr.send(ServiceWorkerMsg::Exit) {
                warn!("Exit service worker manager failed ({})", e);
            }
        }

        debug!("Exiting Canvas Paint thread.");
        let (canvas_exit_sender, canvas_exit_receiver) = unbounded();
        if let Err(e) = self
            .canvas_sender
            .send(ConstellationCanvasMsg::Exit(canvas_exit_sender))
        {
            warn!("Exit Canvas Paint thread failed ({})", e);
        }

        debug!("Exiting WebGPU threads.");
        #[cfg(feature = "webgpu")]
        let receivers = self
            .browsing_context_group_set
            .values()
            .flat_map(|browsing_context_group| {
                browsing_context_group.webgpus.values().map(|webgpu| {
                    let (sender, receiver) = ipc::channel().expect("Failed to create IPC channel!");
                    if let Err(e) = webgpu.exit(sender) {
                        warn!("Exit WebGPU Thread failed ({})", e);
                        None
                    } else {
                        Some(receiver)
                    }
                })
            })
            .flatten();

        #[cfg(feature = "webgpu")]
        for receiver in receivers {
            if let Err(e) = receiver.recv() {
                warn!("Failed to receive exit response from WebGPU ({:?})", e);
            }
        }

        if let Some(webgl_threads) = self.webgl_threads.as_ref() {
            let (sender, receiver) = ipc::channel().expect("Failed to create IPC channel!");
            webgl_threads_receiver = Some(receiver);
            debug!("Exiting WebGL thread.");

            if let Err(e) = webgl_threads.exit(sender) {
                warn!("Exit WebGL Thread failed ({e})");
            }
        }

        debug!("Exiting GLPlayer thread.");
        WindowGLContext::get().exit();

        // Wait for the canvas thread to exit before shutting down the font service, as
        // canvas might still be using the system font service before shutting down.
        let _ = canvas_exit_receiver.recv();

        debug!("Exiting the system font service thread.");
        self.system_font_service.exit();

        // Receive exit signals from threads.
        if let Err(e) = core_ipc_receiver.recv() {
            warn!("Exit resource thread failed ({:?})", e);
        }
        if let Err(e) = storage_ipc_receiver.recv() {
            warn!("Exit storage thread failed ({:?})", e);
        }
        if self.webgl_threads.is_some() {
            if let Err(e) = webgl_threads_receiver
                .expect("webgl_threads_receiver to be Some")
                .recv()
            {
                warn!("Exit WebGL thread failed ({:?})", e);
            }
        }

        debug!("Asking embedding layer to complete shutdown.");
        self.embedder_proxy.send(EmbedderMsg::ShutdownComplete);

        debug!("Shutting-down IPC router thread in constellation.");
        ROUTER.shutdown();
    }

    fn handle_pipeline_exited(&mut self, pipeline_id: PipelineId) {
        debug!("{}: Exited", pipeline_id);
        self.pipelines.remove(&pipeline_id);
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_send_error(&mut self, pipeline_id: PipelineId, err: IpcError) {
        // Treat send error the same as receiving a panic message
        error!("{}: Send error ({})", pipeline_id, err);
        let webview_id = self
            .pipelines
            .get(&pipeline_id)
            .map(|pipeline| pipeline.webview_id);
        let reason = format!("Send failed ({})", err);
        self.handle_panic(webview_id, reason, None);
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_panic(
        &mut self,
        webview_id: Option<WebViewId>,
        reason: String,
        backtrace: Option<String>,
    ) {
        if self.hard_fail {
            // It's quite difficult to make Servo exit cleanly if some threads have failed.
            // Hard fail exists for test runners so we crash and that's good enough.
            println!("Pipeline failed in hard-fail mode.  Crashing!");
            process::exit(1);
        }

        let webview_id = match webview_id {
            Some(id) => id,
            None => return,
        };

        debug!(
            "{}: Panic handler for top-level browsing context: {}",
            webview_id, reason
        );

        let browsing_context_id = BrowsingContextId::from(webview_id);

        self.embedder_proxy.send(EmbedderMsg::Panic(
            webview_id,
            reason.clone(),
            backtrace.clone(),
        ));

        let browsing_context = match self.browsing_contexts.get(&browsing_context_id) {
            Some(context) => context,
            None => return warn!("failed browsing context is missing"),
        };
        let viewport_details = browsing_context.viewport_details;
        let pipeline_id = browsing_context.pipeline_id;
        let throttled = browsing_context.throttled;

        let pipeline = match self.pipelines.get(&pipeline_id) {
            Some(p) => p,
            None => return warn!("failed pipeline is missing"),
        };
        let opener = pipeline.opener;

        self.close_browsing_context_children(
            browsing_context_id,
            DiscardBrowsingContext::No,
            ExitPipelineMode::Force,
        );

        let old_pipeline_id = pipeline_id;
        let old_load_data = match self.pipelines.get(&pipeline_id) {
            Some(pipeline) => pipeline.load_data.clone(),
            None => return warn!("failed pipeline is missing"),
        };
        if old_load_data.crash.is_some() {
            return error!("crash page crashed");
        }

        warn!("creating replacement pipeline for crash page");

        let new_pipeline_id = PipelineId::new();
        let new_load_data = LoadData {
            crash: Some(
                backtrace
                    .map(|b| format!("{}\n{}", reason, b))
                    .unwrap_or(reason),
            ),
            ..old_load_data.clone()
        };

        let sandbox = IFrameSandboxState::IFrameSandboxed;
        let is_private = false;
        self.new_pipeline(
            new_pipeline_id,
            browsing_context_id,
            webview_id,
            None,
            opener,
            viewport_details,
            new_load_data,
            sandbox,
            is_private,
            throttled,
        );
        self.add_pending_change(SessionHistoryChange {
            webview_id,
            browsing_context_id,
            new_pipeline_id,
            // Pipeline already closed by close_browsing_context_children, so we can pass Yes here
            // to avoid closing again in handle_activate_document_msg (though it would be harmless)
            replace: Some(NeedsToReload::Yes(old_pipeline_id, old_load_data)),
            new_browsing_context_info: None,
            viewport_details,
        });
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_focus_web_view(&mut self, webview_id: WebViewId) {
        if self.webviews.get(webview_id).is_none() {
            return warn!("{webview_id}: FocusWebView on unknown top-level browsing context");
        }
        self.webviews.focus(webview_id);
        self.embedder_proxy
            .send(EmbedderMsg::WebViewFocused(webview_id));
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true))
    )]
    fn handle_log_entry(
        &mut self,
        webview_id: Option<WebViewId>,
        thread_name: Option<String>,
        entry: LogEntry,
    ) {
        if let LogEntry::Panic(ref reason, ref backtrace) = entry {
            self.handle_panic(webview_id, reason.clone(), Some(backtrace.clone()));
        }

        match entry {
            LogEntry::Panic(reason, _) | LogEntry::Error(reason) | LogEntry::Warn(reason) => {
                // VecDeque::truncate is unstable
                if WARNINGS_BUFFER_SIZE <= self.handled_warnings.len() {
                    self.handled_warnings.pop_front();
                }
                self.handled_warnings.push_back((thread_name, reason));
            },
        }
    }

    fn update_pressed_mouse_buttons(&mut self, event: &MouseButtonEvent) {
        // This value is ultimately used for a DOM mouse event, and the specification says that
        // the pressed buttons should be represented as a bitmask with values defined at
        // <https://w3c.github.io/uievents/#dom-mouseevent-buttons>.
        let button_as_bitmask = match event.button {
            MouseButton::Left => 1,
            MouseButton::Right => 2,
            MouseButton::Middle => 4,
            MouseButton::Back => 8,
            MouseButton::Forward => 16,
            MouseButton::Other(_) => return,
        };

        match event.action {
            MouseButtonAction::Click | MouseButtonAction::Down => {
                self.pressed_mouse_buttons |= button_as_bitmask;
            },
            MouseButtonAction::Up => {
                self.pressed_mouse_buttons &= !(button_as_bitmask);
            },
        }
    }

    fn update_active_keybord_modifiers(&mut self, event: &KeyboardEvent) {
        self.active_keyboard_modifiers = event.modifiers;

        // `KeyboardEvent::modifiers` contains the pre-existing modifiers before this key was
        // either pressed or released, but `active_keyboard_modifiers` should track the subsequent
        // state. If this event will update that state, we need to ensure that we are tracking what
        // the event changes.
        let modified_modifier = match event.key {
            Key::Alt => Modifiers::ALT,
            Key::AltGraph => Modifiers::ALT_GRAPH,
            Key::CapsLock => Modifiers::CAPS_LOCK,
            Key::Control => Modifiers::CONTROL,
            Key::Fn => Modifiers::FN,
            Key::FnLock => Modifiers::FN_LOCK,
            Key::Meta => Modifiers::META,
            Key::NumLock => Modifiers::NUM_LOCK,
            Key::ScrollLock => Modifiers::SCROLL_LOCK,
            Key::Shift => Modifiers::SHIFT,
            Key::Symbol => Modifiers::SYMBOL,
            Key::SymbolLock => Modifiers::SYMBOL_LOCK,
            Key::Hyper => Modifiers::HYPER,
            // The web doesn't make a distinction between these keys (there is only
            // "meta") so map "super" to "meta".
            Key::Super => Modifiers::META,
            _ => return,
        };
        match event.state {
            KeyState::Down => self.active_keyboard_modifiers.insert(modified_modifier),
            KeyState::Up => self.active_keyboard_modifiers.remove(modified_modifier),
        }
    }

    fn forward_input_event(
        &mut self,
        webview_id: WebViewId,
        event: InputEvent,
        hit_test_result: Option<CompositorHitTestResult>,
    ) {
        if let InputEvent::MouseButton(event) = &event {
            self.update_pressed_mouse_buttons(event);
        }

        if let InputEvent::Keyboard(event) = &event {
            self.update_active_keybord_modifiers(&event.event);
        }

        // The constellation tracks the state of pressed mouse buttons and keyboard
        // modifiers and updates the event here to reflect the current state.
        let pressed_mouse_buttons = self.pressed_mouse_buttons;
        let active_keyboard_modifiers = self.active_keyboard_modifiers;

        // TODO: Click should be handled internally in the `Document`.
        if let InputEvent::MouseButton(event) = &event {
            if event.action == MouseButtonAction::Click {
                self.pressed_mouse_buttons = 0;
            }
        }

        let pipeline_id = match &hit_test_result {
            Some(hit_test) => hit_test.pipeline_id,
            None => {
                // If there's no hit test, send to the focused browsing context of the given webview.
                let Some(browsing_context_id) = self
                    .webviews
                    .get(webview_id)
                    .map(|webview| webview.focused_browsing_context_id)
                else {
                    warn!("Handling InputEvent for an unknown webview: {webview_id}");
                    return;
                };

                let Some(pipeline_id) = self
                    .browsing_contexts
                    .get(&browsing_context_id)
                    .map(|context| context.pipeline_id)
                else {
                    warn!("{browsing_context_id}: Got InputEvent for nonexistent browsing context");
                    return;
                };

                pipeline_id
            },
        };

        let Some(pipeline) = self.pipelines.get(&pipeline_id) else {
            debug!("Got event for pipeline ({pipeline_id}) after closure");
            return;
        };

        let event = ConstellationInputEvent {
            hit_test_result,
            pressed_mouse_buttons,
            active_keyboard_modifiers,
            event,
        };

        if let Err(error) = pipeline
            .event_loop
            .send(ScriptThreadMessage::SendInputEvent(pipeline_id, event))
        {
            self.handle_send_error(pipeline_id, error);
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_new_top_level_browsing_context(
        &mut self,
        url: ServoUrl,
        webview_id: WebViewId,
        viewport_details: ViewportDetails,
        response_sender: Option<IpcSender<WebDriverLoadStatus>>,
    ) {
        let pipeline_id = PipelineId::new();
        let browsing_context_id = BrowsingContextId::from(webview_id);
        let load_data = LoadData::new(
            LoadOrigin::Constellation,
            url,
            None,
            Referrer::NoReferrer,
            ReferrerPolicy::EmptyString,
            None,
            None,
            false,
        );
        let sandbox = IFrameSandboxState::IFrameUnsandboxed;
        let is_private = false;
        let throttled = false;

        // Register this new top-level browsing context id as a webview and set
        // its focused browsing context to be itself.
        self.webviews
            .add(webview_id, ConstellationWebView::new(browsing_context_id));

        // https://html.spec.whatwg.org/multipage/#creating-a-new-browsing-context-group
        let mut new_bc_group: BrowsingContextGroup = Default::default();
        let new_bc_group_id = self.next_browsing_context_group_id();
        new_bc_group
            .top_level_browsing_context_set
            .insert(webview_id);
        self.browsing_context_group_set
            .insert(new_bc_group_id, new_bc_group);

        self.new_pipeline(
            pipeline_id,
            browsing_context_id,
            webview_id,
            None,
            None,
            viewport_details,
            load_data,
            sandbox,
            is_private,
            throttled,
        );
        self.add_pending_change(SessionHistoryChange {
            webview_id,
            browsing_context_id,
            new_pipeline_id: pipeline_id,
            replace: None,
            new_browsing_context_info: Some(NewBrowsingContextInfo {
                parent_pipeline_id: None,
                is_private,
                inherited_secure_context: None,
                throttled,
            }),
            viewport_details,
        });

        if let Some(response_sender) = response_sender {
            self.webdriver.load_channel = Some((pipeline_id, response_sender));
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_close_top_level_browsing_context(&mut self, webview_id: WebViewId) {
        debug!("{webview_id}: Closing");
        let browsing_context_id = BrowsingContextId::from(webview_id);
        let browsing_context =
            self.close_browsing_context(browsing_context_id, ExitPipelineMode::Normal);
        if self.webviews.focused_webview().map(|(id, _)| id) == Some(webview_id) {
            self.embedder_proxy.send(EmbedderMsg::WebViewBlurred);
        }
        self.webviews.remove(webview_id);
        self.compositor_proxy
            .send(CompositorMsg::RemoveWebView(webview_id));
        self.embedder_proxy
            .send(EmbedderMsg::WebViewClosed(webview_id));

        let Some(browsing_context) = browsing_context else {
            return;
        };
        // https://html.spec.whatwg.org/multipage/#bcg-remove
        let bc_group_id = browsing_context.bc_group_id;
        let Some(bc_group) = self.browsing_context_group_set.get_mut(&bc_group_id) else {
            warn!("{}: Browsing context group not found!", bc_group_id);
            return;
        };
        if !bc_group.top_level_browsing_context_set.remove(&webview_id) {
            warn!("{webview_id}: Top-level browsing context not found in {bc_group_id}",);
        }
        if bc_group.top_level_browsing_context_set.is_empty() {
            self.browsing_context_group_set
                .remove(&browsing_context.bc_group_id);
        }

        debug!("{webview_id}: Closed");
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_iframe_size_msg(&mut self, iframe_sizes: Vec<IFrameSizeMsg>) {
        for IFrameSizeMsg {
            browsing_context_id,
            size,
            type_,
        } in iframe_sizes
        {
            self.resize_browsing_context(size, type_, browsing_context_id);
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_finish_javascript_evaluation(
        &mut self,
        evaluation_id: JavaScriptEvaluationId,
        result: Result<JSValue, JavaScriptEvaluationError>,
    ) {
        self.embedder_proxy
            .send(EmbedderMsg::FinishJavaScriptEvaluation(
                evaluation_id,
                result,
            ));
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_subframe_loaded(&mut self, pipeline_id: PipelineId) {
        let browsing_context_id = match self.pipelines.get(&pipeline_id) {
            Some(pipeline) => pipeline.browsing_context_id,
            None => return warn!("{}: Subframe loaded after closure", pipeline_id),
        };
        let parent_pipeline_id = match self.browsing_contexts.get(&browsing_context_id) {
            Some(browsing_context) => browsing_context.parent_pipeline_id,
            None => {
                return warn!(
                    "{}: Subframe loaded in closed {}",
                    pipeline_id, browsing_context_id,
                );
            },
        };
        let parent_pipeline_id = match parent_pipeline_id {
            Some(parent_pipeline_id) => parent_pipeline_id,
            None => return warn!("{}: Subframe has no parent", pipeline_id),
        };
        // https://html.spec.whatwg.org/multipage/#the-iframe-element:completely-loaded
        // When a Document in an iframe is marked as completely loaded,
        // the user agent must run the iframe load event steps.
        let msg = ScriptThreadMessage::DispatchIFrameLoadEvent {
            target: browsing_context_id,
            parent: parent_pipeline_id,
            child: pipeline_id,
        };
        let result = match self.pipelines.get(&parent_pipeline_id) {
            Some(parent) => parent.event_loop.send(msg),
            None => {
                return warn!(
                    "{}: Parent pipeline browsing context loaded after closure",
                    parent_pipeline_id
                );
            },
        };
        if let Err(e) = result {
            self.handle_send_error(parent_pipeline_id, e);
        }
    }

    // The script thread associated with pipeline_id has loaded a URL in an
    // iframe via script. This will result in a new pipeline being spawned and
    // a child being added to the parent browsing context. This message is never
    // the result of a page navigation.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_script_loaded_url_in_iframe_msg(&mut self, load_info: IFrameLoadInfoWithData) {
        let IFrameLoadInfo {
            parent_pipeline_id,
            browsing_context_id,
            webview_id,
            new_pipeline_id,
            is_private,
            mut history_handling,
            ..
        } = load_info.info;

        // If no url is specified, reload.
        let old_pipeline = load_info
            .old_pipeline_id
            .and_then(|id| self.pipelines.get(&id));

        // Replacement enabled also takes into account whether the document is "completely loaded",
        // see https://html.spec.whatwg.org/multipage/#the-iframe-element:completely-loaded
        if let Some(old_pipeline) = old_pipeline {
            if !old_pipeline.completely_loaded {
                history_handling = NavigationHistoryBehavior::Replace;
            }
            debug!(
                "{:?}: Old pipeline is {}completely loaded",
                load_info.old_pipeline_id,
                if old_pipeline.completely_loaded {
                    ""
                } else {
                    "not "
                }
            );
        }

        let is_parent_private = {
            let parent_browsing_context_id = match self.pipelines.get(&parent_pipeline_id) {
                Some(pipeline) => pipeline.browsing_context_id,
                None => {
                    return warn!(
                        "{}: Script loaded url in iframe {} in closed parent pipeline",
                        parent_pipeline_id, browsing_context_id,
                    );
                },
            };
            let is_parent_private = match self.browsing_contexts.get(&parent_browsing_context_id) {
                Some(ctx) => ctx.is_private,
                None => {
                    return warn!(
                        "{}: Script loaded url in iframe {} in closed parent browsing context",
                        parent_browsing_context_id, browsing_context_id,
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
                    "{}: Script loaded url in iframe with closed browsing context",
                    browsing_context_id,
                );
            },
        };

        let replace = if history_handling == NavigationHistoryBehavior::Replace {
            Some(NeedsToReload::No(browsing_context.pipeline_id))
        } else {
            None
        };

        let browsing_context_size = browsing_context.viewport_details;
        let browsing_context_throttled = browsing_context.throttled;
        // TODO(servo#30571) revert to debug_assert_eq!() once underlying bug is fixed
        #[cfg(debug_assertions)]
        if !(browsing_context_size == load_info.viewport_details) {
            log::warn!(
                "debug assertion failed! browsing_context_size == load_info.viewport_details.initial_viewport"
            );
        }

        // Create the new pipeline, attached to the parent and push to pending changes
        self.new_pipeline(
            new_pipeline_id,
            browsing_context_id,
            webview_id,
            Some(parent_pipeline_id),
            None,
            browsing_context_size,
            load_info.load_data,
            load_info.sandbox,
            is_private,
            browsing_context_throttled,
        );
        self.add_pending_change(SessionHistoryChange {
            webview_id,
            browsing_context_id,
            new_pipeline_id,
            replace,
            // Browsing context for iframe already exists.
            new_browsing_context_info: None,
            viewport_details: load_info.viewport_details,
        });
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_script_new_iframe(&mut self, load_info: IFrameLoadInfoWithData) {
        let IFrameLoadInfo {
            parent_pipeline_id,
            new_pipeline_id,
            browsing_context_id,
            webview_id,
            is_private,
            ..
        } = load_info.info;

        let (script_sender, parent_browsing_context_id) =
            match self.pipelines.get(&parent_pipeline_id) {
                Some(pipeline) => (pipeline.event_loop.clone(), pipeline.browsing_context_id),
                None => {
                    return warn!(
                        "{}: Script loaded url in closed iframe pipeline",
                        parent_pipeline_id
                    );
                },
            };
        let (is_parent_private, is_parent_throttled, is_parent_secure) =
            match self.browsing_contexts.get(&parent_browsing_context_id) {
                Some(ctx) => (ctx.is_private, ctx.throttled, ctx.inherited_secure_context),
                None => {
                    return warn!(
                        "{}: New iframe {} loaded in closed parent browsing context",
                        parent_browsing_context_id, browsing_context_id,
                    );
                },
            };
        let is_private = is_private || is_parent_private;
        let pipeline = Pipeline::new(
            new_pipeline_id,
            browsing_context_id,
            webview_id,
            None,
            script_sender,
            self.compositor_proxy.clone(),
            is_parent_throttled,
            load_info.load_data,
        );

        assert!(!self.pipelines.contains_key(&new_pipeline_id));
        self.pipelines.insert(new_pipeline_id, pipeline);
        self.add_pending_change(SessionHistoryChange {
            webview_id,
            browsing_context_id,
            new_pipeline_id,
            replace: None,
            // Browsing context for iframe doesn't exist yet.
            new_browsing_context_info: Some(NewBrowsingContextInfo {
                parent_pipeline_id: Some(parent_pipeline_id),
                is_private,
                inherited_secure_context: is_parent_secure,
                throttled: is_parent_throttled,
            }),
            viewport_details: load_info.viewport_details,
        });
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_script_new_auxiliary(&mut self, load_info: AuxiliaryWebViewCreationRequest) {
        let AuxiliaryWebViewCreationRequest {
            load_data,
            opener_webview_id,
            opener_pipeline_id,
            response_sender,
        } = load_info;

        let (webview_id_sender, webview_id_receiver) = match ipc::channel() {
            Ok(result) => result,
            Err(error) => {
                warn!("Failed to create channel: {error:?}");
                let _ = response_sender.send(None);
                return;
            },
        };
        self.embedder_proxy.send(EmbedderMsg::AllowOpeningWebView(
            opener_webview_id,
            webview_id_sender,
        ));
        let (new_webview_id, viewport_details) = match webview_id_receiver.recv() {
            Ok(Some((webview_id, viewport_details))) => (webview_id, viewport_details),
            Ok(None) | Err(_) => {
                let _ = response_sender.send(None);
                return;
            },
        };
        let new_browsing_context_id = BrowsingContextId::from(new_webview_id);

        let (script_sender, opener_browsing_context_id) =
            match self.pipelines.get(&opener_pipeline_id) {
                Some(pipeline) => (pipeline.event_loop.clone(), pipeline.browsing_context_id),
                None => {
                    return warn!(
                        "{}: Auxiliary loaded url in closed iframe pipeline",
                        opener_pipeline_id
                    );
                },
            };
        let (is_opener_private, is_opener_throttled, is_opener_secure) =
            match self.browsing_contexts.get(&opener_browsing_context_id) {
                Some(ctx) => (ctx.is_private, ctx.throttled, ctx.inherited_secure_context),
                None => {
                    return warn!(
                        "{}: New auxiliary {} loaded in closed opener browsing context",
                        opener_browsing_context_id, new_browsing_context_id,
                    );
                },
            };
        let new_pipeline_id = PipelineId::new();
        let pipeline = Pipeline::new(
            new_pipeline_id,
            new_browsing_context_id,
            new_webview_id,
            Some(opener_browsing_context_id),
            script_sender,
            self.compositor_proxy.clone(),
            is_opener_throttled,
            load_data,
        );
        let _ = response_sender.send(Some(AuxiliaryWebViewCreationResponse {
            new_webview_id,
            new_pipeline_id,
        }));

        assert!(!self.pipelines.contains_key(&new_pipeline_id));
        self.pipelines.insert(new_pipeline_id, pipeline);
        self.webviews.add(
            new_webview_id,
            ConstellationWebView::new(new_browsing_context_id),
        );

        // https://html.spec.whatwg.org/multipage/#bcg-append
        let opener = match self.browsing_contexts.get(&opener_browsing_context_id) {
            Some(id) => id,
            None => {
                warn!("Trying to append an unknown auxiliary to a browsing context group");
                return;
            },
        };
        let bc_group = match self.browsing_context_group_set.get_mut(&opener.bc_group_id) {
            Some(bc_group) => bc_group,
            None => {
                warn!("Trying to add a top-level to an unknown group.");
                return;
            },
        };
        bc_group
            .top_level_browsing_context_set
            .insert(new_webview_id);

        self.add_pending_change(SessionHistoryChange {
            webview_id: new_webview_id,
            browsing_context_id: new_browsing_context_id,
            new_pipeline_id,
            replace: None,
            new_browsing_context_info: Some(NewBrowsingContextInfo {
                // Auxiliary browsing contexts are always top-level.
                parent_pipeline_id: None,
                is_private: is_opener_private,
                inherited_secure_context: is_opener_secure,
                throttled: is_opener_throttled,
            }),
            viewport_details,
        });
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_set_cursor_msg(&mut self, webview_id: WebViewId, cursor: Cursor) {
        self.embedder_proxy
            .send(EmbedderMsg::SetCursor(webview_id, cursor));
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_change_running_animations_state(
        &mut self,
        pipeline_id: PipelineId,
        animation_state: AnimationState,
    ) {
        if let Some(pipeline) = self.pipelines.get_mut(&pipeline_id) {
            if pipeline.animation_state != animation_state {
                pipeline.animation_state = animation_state;
                self.compositor_proxy
                    .send(CompositorMsg::ChangeRunningAnimationsState(
                        pipeline.webview_id,
                        pipeline_id,
                        animation_state,
                    ))
            }
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_tick_animation(&mut self, webview_ids: Vec<WebViewId>) {
        let mut animating_event_loops = HashSet::new();

        for webview_id in webview_ids.iter() {
            for browsing_context in self.fully_active_browsing_contexts_iter(*webview_id) {
                let Some(pipeline) = self.pipelines.get(&browsing_context.pipeline_id) else {
                    continue;
                };
                animating_event_loops.insert(pipeline.event_loop.clone());
            }
        }

        for event_loop in animating_event_loops {
            // No error handling here. It's unclear what to do when this fails as the error isn't associated
            // with a particular pipeline. In addition, the danger of not progressing animations is pretty
            // low, so it's probably safe to ignore this error and handle the crashed ScriptThread on
            // some other message.
            let _ = event_loop.send(ScriptThreadMessage::TickAllAnimations(webview_ids.clone()));
        }
    }

    /// Schedule a navigation(via load_url).
    /// 1: Ask the embedder for permission.
    /// 2: Store the details of the navigation, pending approval from the embedder.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn schedule_navigation(
        &mut self,
        webview_id: WebViewId,
        source_id: PipelineId,
        load_data: LoadData,
        history_handling: NavigationHistoryBehavior,
    ) {
        match self.pending_approval_navigations.entry(source_id) {
            Entry::Occupied(_) => {
                return warn!(
                    "{}: Tried to schedule a navigation while one is already pending",
                    source_id
                );
            },
            Entry::Vacant(entry) => {
                let _ = entry.insert((load_data.clone(), history_handling));
            },
        };
        // Allow the embedder to handle the url itself
        self.embedder_proxy
            .send(EmbedderMsg::AllowNavigationRequest(
                webview_id,
                source_id,
                load_data.url.clone(),
            ));
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn load_url(
        &mut self,
        webview_id: WebViewId,
        source_id: PipelineId,
        load_data: LoadData,
        history_handling: NavigationHistoryBehavior,
    ) -> Option<PipelineId> {
        debug!(
            "{}: Loading ({}replacing): {}",
            source_id,
            match history_handling {
                NavigationHistoryBehavior::Push => "",
                NavigationHistoryBehavior::Replace => "not ",
                NavigationHistoryBehavior::Auto => "unsure if ",
            },
            load_data.url,
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
                warn!("{}: Loaded after closure", source_id);
                return None;
            },
        };
        let (viewport_details, pipeline_id, parent_pipeline_id, is_private, is_throttled) =
            match self.browsing_contexts.get(&browsing_context_id) {
                Some(ctx) => (
                    ctx.viewport_details,
                    ctx.pipeline_id,
                    ctx.parent_pipeline_id,
                    ctx.is_private,
                    ctx.throttled,
                ),
                None => {
                    // This should technically never happen (since `load_url` is
                    // only called on existing browsing contexts), but we prefer to
                    // avoid `expect`s or `unwrap`s in `Constellation` to ward
                    // against future changes that might break things.
                    warn!(
                        "{}: Loaded url in closed {}",
                        source_id, browsing_context_id,
                    );
                    return None;
                },
            };

        match parent_pipeline_id {
            Some(parent_pipeline_id) => {
                // Find the script thread for the pipeline containing the iframe
                // and issue an iframe load through there.
                let msg = ScriptThreadMessage::NavigateIframe(
                    parent_pipeline_id,
                    browsing_context_id,
                    load_data,
                    history_handling,
                );
                let result = match self.pipelines.get(&parent_pipeline_id) {
                    Some(parent_pipeline) => parent_pipeline.event_loop.send(msg),
                    None => {
                        warn!("{}: Child loaded after closure", parent_pipeline_id);
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

                let replace = if history_handling == NavigationHistoryBehavior::Replace {
                    Some(NeedsToReload::No(pipeline_id))
                } else {
                    None
                };

                let new_pipeline_id = PipelineId::new();
                let sandbox = IFrameSandboxState::IFrameUnsandboxed;
                self.new_pipeline(
                    new_pipeline_id,
                    browsing_context_id,
                    webview_id,
                    None,
                    opener,
                    viewport_details,
                    load_data,
                    sandbox,
                    is_private,
                    is_throttled,
                );
                self.add_pending_change(SessionHistoryChange {
                    webview_id,
                    browsing_context_id,
                    new_pipeline_id,
                    replace,
                    // `load_url` is always invoked on an existing browsing context.
                    new_browsing_context_info: None,
                    viewport_details,
                });
                Some(new_pipeline_id)
            },
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
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

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_load_complete_msg(&mut self, webview_id: WebViewId, pipeline_id: PipelineId) {
        let mut webdriver_reset = false;
        if let Some((expected_pipeline_id, ref reply_chan)) = self.webdriver.load_channel {
            if expected_pipeline_id == pipeline_id {
                debug!("Sending load for {:?} to WebDriver", expected_pipeline_id);
                let _ = reply_chan.send(WebDriverLoadStatus::Complete);
                webdriver_reset = true;
            }
        }
        if webdriver_reset {
            self.webdriver.load_channel = None;
        }

        if let Some(pipeline) = self.pipelines.get_mut(&pipeline_id) {
            debug!("{}: Marking as loaded", pipeline_id);
            pipeline.completely_loaded = true;
        }

        // Notify the embedder that the TopLevelBrowsingContext current document
        // has finished loading.
        // We need to make sure the pipeline that has finished loading is the current
        // pipeline and that no pending pipeline will replace the current one.
        let pipeline_is_top_level_pipeline = self
            .browsing_contexts
            .get(&BrowsingContextId::from(webview_id))
            .map(|ctx| ctx.pipeline_id == pipeline_id)
            .unwrap_or(false);
        if pipeline_is_top_level_pipeline {
            // Is there any pending pipeline that will replace the current top level pipeline
            let current_top_level_pipeline_will_be_replaced = self
                .pending_changes
                .iter()
                .any(|change| change.browsing_context_id == webview_id);

            if !current_top_level_pipeline_will_be_replaced {
                // Notify embedder and compositor top level document finished loading.
                self.compositor_proxy
                    .send(CompositorMsg::LoadComplete(webview_id));
            }
        } else {
            self.handle_subframe_loaded(pipeline_id);
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_navigated_to_fragment(
        &mut self,
        pipeline_id: PipelineId,
        new_url: ServoUrl,
        history_handling: NavigationHistoryBehavior,
    ) {
        let (webview_id, old_url) = match self.pipelines.get_mut(&pipeline_id) {
            Some(pipeline) => {
                let old_url = replace(&mut pipeline.url, new_url.clone());
                (pipeline.webview_id, old_url)
            },
            None => {
                return warn!("{}: Navigated to fragment after closure", pipeline_id);
            },
        };

        match history_handling {
            NavigationHistoryBehavior::Replace => {},
            _ => {
                let diff = SessionHistoryDiff::Hash {
                    pipeline_reloader: NeedsToReload::No(pipeline_id),
                    new_url,
                    old_url,
                };

                self.get_joint_session_history(webview_id).push_diff(diff);

                self.notify_history_changed(webview_id);
            },
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_traverse_history_msg(
        &mut self,
        webview_id: WebViewId,
        direction: TraversalDirection,
    ) {
        let mut browsing_context_changes = HashMap::<BrowsingContextId, NeedsToReload>::new();
        let mut pipeline_changes = HashMap::<PipelineId, (Option<HistoryStateId>, ServoUrl)>::new();
        let mut url_to_load = HashMap::<PipelineId, ServoUrl>::new();
        {
            let session_history = self.get_joint_session_history(webview_id);
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
                            SessionHistoryDiff::BrowsingContext {
                                browsing_context_id,
                                ref new_reloader,
                                ..
                            } => {
                                browsing_context_changes
                                    .insert(browsing_context_id, new_reloader.clone());
                            },
                            SessionHistoryDiff::Pipeline {
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
                            SessionHistoryDiff::Hash {
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
                            SessionHistoryDiff::BrowsingContext {
                                browsing_context_id,
                                ref old_reloader,
                                ..
                            } => {
                                browsing_context_changes
                                    .insert(browsing_context_id, old_reloader.clone());
                            },
                            SessionHistoryDiff::Pipeline {
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
                            SessionHistoryDiff::Hash {
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

        self.notify_history_changed(webview_id);

        self.trim_history(webview_id);
        self.update_webview_in_compositor(webview_id);
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn update_browsing_context(
        &mut self,
        browsing_context_id: BrowsingContextId,
        new_reloader: NeedsToReload,
    ) {
        let new_pipeline_id = match new_reloader {
            NeedsToReload::No(pipeline_id) => pipeline_id,
            NeedsToReload::Yes(pipeline_id, load_data) => {
                debug!(
                    "{}: Reloading document {}",
                    browsing_context_id, pipeline_id,
                );

                // TODO: Save the sandbox state so it can be restored here.
                let sandbox = IFrameSandboxState::IFrameUnsandboxed;
                let (
                    top_level_id,
                    old_pipeline_id,
                    parent_pipeline_id,
                    viewport_details,
                    is_private,
                    throttled,
                ) = match self.browsing_contexts.get(&browsing_context_id) {
                    Some(ctx) => (
                        ctx.top_level_id,
                        ctx.pipeline_id,
                        ctx.parent_pipeline_id,
                        ctx.viewport_details,
                        ctx.is_private,
                        ctx.throttled,
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
                    viewport_details,
                    load_data.clone(),
                    sandbox,
                    is_private,
                    throttled,
                );
                self.add_pending_change(SessionHistoryChange {
                    webview_id: top_level_id,
                    browsing_context_id,
                    new_pipeline_id,
                    replace: Some(NeedsToReload::Yes(pipeline_id, load_data)),
                    // Browsing context must exist at this point.
                    new_browsing_context_info: None,
                    viewport_details,
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
                    return warn!("{}: Closed during traversal", browsing_context_id);
                },
            };

        if let Some(old_pipeline) = self.pipelines.get(&old_pipeline_id) {
            old_pipeline.set_throttled(true);
        }
        if let Some(new_pipeline) = self.pipelines.get(&new_pipeline_id) {
            if let Some(ref chan) = self.devtools_sender {
                let state = NavigationState::Start(new_pipeline.url.clone());
                let _ = chan.send(DevtoolsControlMsg::FromScript(
                    ScriptToDevtoolsControlMsg::Navigate(browsing_context_id, state),
                ));
                let page_info = DevtoolsPageInfo {
                    title: new_pipeline.title.clone(),
                    url: new_pipeline.url.clone(),
                    is_top_level_global: top_level_id == browsing_context_id,
                };
                let state = NavigationState::Stop(new_pipeline.id, page_info);
                let _ = chan.send(DevtoolsControlMsg::FromScript(
                    ScriptToDevtoolsControlMsg::Navigate(browsing_context_id, state),
                ));
            }

            new_pipeline.set_throttled(false);
            self.notify_focus_state(new_pipeline_id);
        }

        self.update_activity(old_pipeline_id);
        self.update_activity(new_pipeline_id);

        if let Some(parent_pipeline_id) = parent_pipeline_id {
            let msg = ScriptThreadMessage::UpdatePipelineId(
                parent_pipeline_id,
                browsing_context_id,
                top_level_id,
                new_pipeline_id,
                UpdatePipelineIdReason::Traversal,
            );
            let result = match self.pipelines.get(&parent_pipeline_id) {
                None => {
                    return warn!("{}: Child traversed after closure", parent_pipeline_id);
                },
                Some(pipeline) => pipeline.event_loop.send(msg),
            };
            if let Err(e) = result {
                self.handle_send_error(parent_pipeline_id, e);
            }
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn update_pipeline(
        &mut self,
        pipeline_id: PipelineId,
        history_state_id: Option<HistoryStateId>,
        url: ServoUrl,
    ) {
        let result = match self.pipelines.get_mut(&pipeline_id) {
            None => {
                return warn!("{}: History state updated after closure", pipeline_id);
            },
            Some(pipeline) => {
                let msg = ScriptThreadMessage::UpdateHistoryState(
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

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_joint_session_history_length(
        &self,
        webview_id: WebViewId,
        response_sender: IpcSender<u32>,
    ) {
        let length = self
            .webviews
            .get(webview_id)
            .map(|webview| webview.session_history.history_length())
            .unwrap_or(1);
        let _ = response_sender.send(length as u32);
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_push_history_state_msg(
        &mut self,
        pipeline_id: PipelineId,
        history_state_id: HistoryStateId,
        url: ServoUrl,
    ) {
        let (webview_id, old_state_id, old_url) = match self.pipelines.get_mut(&pipeline_id) {
            Some(pipeline) => {
                let old_history_state_id = pipeline.history_state_id;
                let old_url = replace(&mut pipeline.url, url.clone());
                pipeline.history_state_id = Some(history_state_id);
                pipeline.history_states.insert(history_state_id);
                (pipeline.webview_id, old_history_state_id, old_url)
            },
            None => {
                return warn!(
                    "{}: Push history state {} for closed pipeline",
                    pipeline_id, history_state_id,
                );
            },
        };

        let diff = SessionHistoryDiff::Pipeline {
            pipeline_reloader: NeedsToReload::No(pipeline_id),
            new_history_state_id: history_state_id,
            new_url: url,
            old_history_state_id: old_state_id,
            old_url,
        };
        self.get_joint_session_history(webview_id).push_diff(diff);
        self.notify_history_changed(webview_id);
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_replace_history_state_msg(
        &mut self,
        pipeline_id: PipelineId,
        history_state_id: HistoryStateId,
        url: ServoUrl,
    ) {
        let webview_id = match self.pipelines.get_mut(&pipeline_id) {
            Some(pipeline) => {
                pipeline.history_state_id = Some(history_state_id);
                pipeline.url = url.clone();
                pipeline.webview_id
            },
            None => {
                return warn!(
                    "{}: Replace history state {} for closed pipeline",
                    history_state_id, pipeline_id
                );
            },
        };

        let session_history = self.get_joint_session_history(webview_id);
        session_history.replace_history_state(pipeline_id, history_state_id, url);
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_reload_msg(&mut self, webview_id: WebViewId) {
        let browsing_context_id = BrowsingContextId::from(webview_id);
        let pipeline_id = match self.browsing_contexts.get(&browsing_context_id) {
            Some(browsing_context) => browsing_context.pipeline_id,
            None => {
                return warn!("{}: Got reload event after closure", browsing_context_id);
            },
        };
        let msg = ScriptThreadMessage::Reload(pipeline_id);
        let result = match self.pipelines.get(&pipeline_id) {
            None => return warn!("{}: Got reload event after closure", pipeline_id),
            Some(pipeline) => pipeline.event_loop.send(msg),
        };
        if let Err(e) = result {
            self.handle_send_error(pipeline_id, e);
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_post_message_msg(
        &mut self,
        browsing_context_id: BrowsingContextId,
        source_pipeline: PipelineId,
        origin: Option<ImmutableOrigin>,
        source_origin: ImmutableOrigin,
        data: StructuredSerializedData,
    ) {
        let pipeline_id = match self.browsing_contexts.get(&browsing_context_id) {
            None => {
                return warn!(
                    "{}: PostMessage to closed browsing context",
                    browsing_context_id
                );
            },
            Some(browsing_context) => browsing_context.pipeline_id,
        };
        let source_browsing_context = match self.pipelines.get(&source_pipeline) {
            Some(pipeline) => pipeline.webview_id,
            None => return warn!("{}: PostMessage from closed pipeline", source_pipeline),
        };
        let msg = ScriptThreadMessage::PostMessage {
            target: pipeline_id,
            source: source_pipeline,
            source_browsing_context,
            target_origin: origin,
            source_origin,
            data,
        };
        let result = match self.pipelines.get(&pipeline_id) {
            Some(pipeline) => pipeline.event_loop.send(msg),
            None => return warn!("{}: PostMessage to closed pipeline", pipeline_id),
        };
        if let Err(e) = result {
            self.handle_send_error(pipeline_id, e);
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_focus_msg(
        &mut self,
        pipeline_id: PipelineId,
        focused_child_browsing_context_id: Option<BrowsingContextId>,
        sequence: FocusSequenceNumber,
    ) {
        let (browsing_context_id, webview_id) = match self.pipelines.get_mut(&pipeline_id) {
            Some(pipeline) => {
                pipeline.focus_sequence = sequence;
                (pipeline.browsing_context_id, pipeline.webview_id)
            },
            None => return warn!("{}: Focus parent after closure", pipeline_id),
        };

        // Ignore if the pipeline isn't fully active.
        if self.get_activity(pipeline_id) != DocumentActivity::FullyActive {
            debug!(
                "Ignoring the focus request because pipeline {} is not \
                fully active",
                pipeline_id
            );
            return;
        }

        // Focus the top-level browsing context.
        self.webviews.focus(webview_id);
        self.embedder_proxy
            .send(EmbedderMsg::WebViewFocused(webview_id));

        // If a container with a non-null nested browsing context is focused,
        // the nested browsing context's active document becomes the focused
        // area of the top-level browsing context instead.
        let focused_browsing_context_id =
            focused_child_browsing_context_id.unwrap_or(browsing_context_id);

        // Send focus messages to the affected pipelines, except
        // `pipeline_id`, which has already its local focus state
        // updated.
        self.focus_browsing_context(Some(pipeline_id), focused_browsing_context_id);
    }

    fn handle_focus_remote_document_msg(&mut self, focused_browsing_context_id: BrowsingContextId) {
        let pipeline_id = match self.browsing_contexts.get(&focused_browsing_context_id) {
            Some(browsing_context) => browsing_context.pipeline_id,
            None => return warn!("Browsing context {} not found", focused_browsing_context_id),
        };

        // Ignore if its active document isn't fully active.
        if self.get_activity(pipeline_id) != DocumentActivity::FullyActive {
            debug!(
                "Ignoring the remote focus request because pipeline {} of \
                browsing context {} is not fully active",
                pipeline_id, focused_browsing_context_id,
            );
            return;
        }

        self.focus_browsing_context(None, focused_browsing_context_id);
    }

    /// Perform [the focusing steps][1] for the active document of
    /// `focused_browsing_context_id`.
    ///
    /// If `initiator_pipeline_id` is specified, this method avoids sending
    /// a message to `initiator_pipeline_id`, assuming its local focus state has
    /// already been updated. This is necessary for performing the focusing
    /// steps for an object that is not the document itself but something that
    /// belongs to the document.
    ///
    /// [1]: https://html.spec.whatwg.org/multipage/#focusing-steps
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn focus_browsing_context(
        &mut self,
        initiator_pipeline_id: Option<PipelineId>,
        focused_browsing_context_id: BrowsingContextId,
    ) {
        let webview_id = match self.browsing_contexts.get(&focused_browsing_context_id) {
            Some(browsing_context) => browsing_context.top_level_id,
            None => return warn!("Browsing context {} not found", focused_browsing_context_id),
        };

        // Update the webview’s focused browsing context.
        let old_focused_browsing_context_id = match self.webviews.get_mut(webview_id) {
            Some(browser) => replace(
                &mut browser.focused_browsing_context_id,
                focused_browsing_context_id,
            ),
            None => {
                return warn!(
                    "{}: Browsing context for focus msg does not exist",
                    webview_id
                );
            },
        };

        // The following part is similar to [the focus update steps][1] except
        // that only `Document`s in the given focus chains are considered. It's
        // ultimately up to the script threads to fire focus events at the
        // affected objects.
        //
        // [1]: https://html.spec.whatwg.org/multipage/#focus-update-steps
        let mut old_focus_chain_pipelines: Vec<&Pipeline> = self
            .ancestor_or_self_pipelines_of_browsing_context_iter(old_focused_browsing_context_id)
            .collect();
        let mut new_focus_chain_pipelines: Vec<&Pipeline> = self
            .ancestor_or_self_pipelines_of_browsing_context_iter(focused_browsing_context_id)
            .collect();

        debug!(
            "old_focus_chain_pipelines = {:?}",
            old_focus_chain_pipelines
                .iter()
                .map(|p| p.id.to_string())
                .collect::<Vec<_>>()
        );
        debug!(
            "new_focus_chain_pipelines = {:?}",
            new_focus_chain_pipelines
                .iter()
                .map(|p| p.id.to_string())
                .collect::<Vec<_>>()
        );

        // At least the last entries should match. Otherwise something is wrong,
        // and we don't want to proceed and crash the top-level pipeline by
        // sending an impossible `Unfocus` message to it.
        match (
            &old_focus_chain_pipelines[..],
            &new_focus_chain_pipelines[..],
        ) {
            ([.., p1], [.., p2]) if p1.id == p2.id => {},
            _ => {
                warn!("Aborting the focus operation - focus chain sanity check failed");
                return;
            },
        }

        // > If the last entry in `old chain` and the last entry in `new chain`
        // > are the same, pop the last entry from `old chain` and the last
        // > entry from `new chain` and redo this step.
        let mut first_common_pipeline_in_chain = None;
        while let ([.., p1], [.., p2]) = (
            &old_focus_chain_pipelines[..],
            &new_focus_chain_pipelines[..],
        ) {
            if p1.id != p2.id {
                break;
            }
            old_focus_chain_pipelines.pop();
            first_common_pipeline_in_chain = new_focus_chain_pipelines.pop();
        }

        let mut send_errors = Vec::new();

        // > For each entry `entry` in `old chain`, in order, run these
        // > substeps: [...]
        for &pipeline in old_focus_chain_pipelines.iter() {
            if Some(pipeline.id) != initiator_pipeline_id {
                let msg = ScriptThreadMessage::Unfocus(pipeline.id, pipeline.focus_sequence);
                trace!("Sending {:?} to {}", msg, pipeline.id);
                if let Err(e) = pipeline.event_loop.send(msg) {
                    send_errors.push((pipeline.id, e));
                }
            } else {
                trace!(
                    "Not notifying {} - it's the initiator of this focus operation",
                    pipeline.id
                );
            }
        }

        // > For each entry entry in `new chain`, in reverse order, run these
        // > substeps: [...]
        let mut child_browsing_context_id = None;
        for &pipeline in new_focus_chain_pipelines.iter().rev() {
            // Don't send a message to the browsing context that initiated this
            // focus operation. It already knows that it has gotten focus.
            if Some(pipeline.id) != initiator_pipeline_id {
                let msg = if let Some(child_browsing_context_id) = child_browsing_context_id {
                    // Focus the container element of `child_browsing_context_id`.
                    ScriptThreadMessage::FocusIFrame(
                        pipeline.id,
                        child_browsing_context_id,
                        pipeline.focus_sequence,
                    )
                } else {
                    // Focus the document.
                    ScriptThreadMessage::FocusDocument(pipeline.id, pipeline.focus_sequence)
                };
                trace!("Sending {:?} to {}", msg, pipeline.id);
                if let Err(e) = pipeline.event_loop.send(msg) {
                    send_errors.push((pipeline.id, e));
                }
            } else {
                trace!(
                    "Not notifying {} - it's the initiator of this focus operation",
                    pipeline.id
                );
            }
            child_browsing_context_id = Some(pipeline.browsing_context_id);
        }

        if let (Some(pipeline), Some(child_browsing_context_id)) =
            (first_common_pipeline_in_chain, child_browsing_context_id)
        {
            if Some(pipeline.id) != initiator_pipeline_id {
                // Focus the container element of `child_browsing_context_id`.
                let msg = ScriptThreadMessage::FocusIFrame(
                    pipeline.id,
                    child_browsing_context_id,
                    pipeline.focus_sequence,
                );
                trace!("Sending {:?} to {}", msg, pipeline.id);
                if let Err(e) = pipeline.event_loop.send(msg) {
                    send_errors.push((pipeline.id, e));
                }
            }
        }

        for (pipeline_id, e) in send_errors {
            self.handle_send_error(pipeline_id, e);
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
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

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_set_throttled_complete(&mut self, pipeline_id: PipelineId, throttled: bool) {
        let browsing_context_id = match self.pipelines.get(&pipeline_id) {
            Some(pipeline) => pipeline.browsing_context_id,
            None => {
                return warn!(
                    "{}: Visibility change for closed browsing context",
                    pipeline_id
                );
            },
        };
        let parent_pipeline_id = match self.browsing_contexts.get(&browsing_context_id) {
            Some(ctx) => ctx.parent_pipeline_id,
            None => {
                return warn!("{}: Visibility change for closed pipeline", pipeline_id);
            },
        };

        if let Some(parent_pipeline_id) = parent_pipeline_id {
            let msg = ScriptThreadMessage::SetThrottledInContainingIframe(
                parent_pipeline_id,
                browsing_context_id,
                throttled,
            );
            let result = match self.pipelines.get(&parent_pipeline_id) {
                None => return warn!("{}: Parent pipeline closed", parent_pipeline_id),
                Some(parent_pipeline) => parent_pipeline.event_loop.send(msg),
            };

            if let Err(e) = result {
                self.handle_send_error(parent_pipeline_id, e);
            }
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_create_canvas_paint_thread_msg(
        &mut self,
        size: UntypedSize2D<u64>,
        response_sender: IpcSender<(IpcSender<CanvasMsg>, CanvasId, ImageKey)>,
    ) {
        let (canvas_data_sender, canvas_data_receiver) = unbounded();

        if let Err(e) = self.canvas_sender.send(ConstellationCanvasMsg::Create {
            sender: canvas_data_sender,
            size,
        }) {
            return warn!("Create canvas paint thread failed ({})", e);
        }
        let (canvas_id, image_key) = match canvas_data_receiver.recv() {
            Ok(canvas_data) => canvas_data,
            Err(e) => return warn!("Create canvas paint thread id response failed ({})", e),
        };
        if let Err(e) = response_sender.send((self.canvas_ipc_sender.clone(), canvas_id, image_key))
        {
            warn!("Create canvas paint thread response failed ({})", e);
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_webdriver_msg(&mut self, msg: WebDriverCommandMsg) {
        // Find the script channel for the given parent pipeline,
        // and pass the event to that script thread.
        match msg {
            WebDriverCommandMsg::CloseWebView(webview_id) => {
                self.handle_close_top_level_browsing_context(webview_id);
            },
            WebDriverCommandMsg::NewWebView(
                originating_webview_id,
                response_sender,
                load_status_sender,
            ) => {
                let (embedder_sender, receiver) = match ipc::channel() {
                    Ok(result) => result,
                    Err(error) => return warn!("Failed to create channel: {error:?}"),
                };
                self.embedder_proxy.send(EmbedderMsg::AllowOpeningWebView(
                    originating_webview_id,
                    embedder_sender,
                ));
                let (new_webview_id, viewport_details) = match receiver.recv() {
                    Ok(Some((new_webview_id, viewport_details))) => {
                        (new_webview_id, viewport_details)
                    },
                    Ok(None) => return warn!("Embedder refused to allow opening webview"),
                    Err(error) => return warn!("Failed to receive webview id: {error:?}"),
                };
                self.handle_new_top_level_browsing_context(
                    ServoUrl::parse_with_base(None, "about:blank").expect("Infallible parse"),
                    new_webview_id,
                    viewport_details,
                    Some(load_status_sender),
                );
                if let Err(error) = response_sender.send(new_webview_id) {
                    error!(
                        "WebDriverCommandMsg::NewWebView: IPC error when sending new_webview_id \
                        to webdriver server: {error}"
                    );
                }
            },
            WebDriverCommandMsg::FocusWebView(webview_id) => {
                self.handle_focus_web_view(webview_id);
            },
            WebDriverCommandMsg::IsWebViewOpen(webview_id, response_sender) => {
                let is_open = self.webviews.get(webview_id).is_some();
                let _ = response_sender.send(is_open);
            },
            WebDriverCommandMsg::IsBrowsingContextOpen(browsing_context_id, response_sender) => {
                let is_open = self.browsing_contexts.contains_key(&browsing_context_id);
                let _ = response_sender.send(is_open);
            },
            WebDriverCommandMsg::GetWindowSize(webview_id, response_sender) => {
                let browsing_context_id = BrowsingContextId::from(webview_id);
                let size = self
                    .browsing_contexts
                    .get(&browsing_context_id)
                    .map(|browsing_context| browsing_context.viewport_details.size)
                    .unwrap_or_default();
                let _ = response_sender.send(size);
            },
            WebDriverCommandMsg::SetWindowSize(webview_id, size, response_sender) => {
                self.webdriver.resize_channel = Some(response_sender);
                self.embedder_proxy
                    .send(EmbedderMsg::ResizeTo(webview_id, size));
            },
            WebDriverCommandMsg::LoadUrl(webview_id, url, response_sender) => {
                let load_data = LoadData::new(
                    LoadOrigin::WebDriver,
                    url,
                    None,
                    Referrer::NoReferrer,
                    ReferrerPolicy::EmptyString,
                    None,
                    None,
                    false,
                );
                self.load_url_for_webdriver(
                    webview_id,
                    load_data,
                    response_sender,
                    NavigationHistoryBehavior::Push,
                );
            },
            WebDriverCommandMsg::Refresh(webview_id, response_sender) => {
                let browsing_context_id = BrowsingContextId::from(webview_id);
                let pipeline_id = self
                    .browsing_contexts
                    .get(&browsing_context_id)
                    .expect("Refresh: Browsing context must exist at this point")
                    .pipeline_id;
                let load_data = match self.pipelines.get(&pipeline_id) {
                    Some(pipeline) => pipeline.load_data.clone(),
                    None => return warn!("{}: Refresh after closure", pipeline_id),
                };
                self.load_url_for_webdriver(
                    webview_id,
                    load_data,
                    response_sender,
                    NavigationHistoryBehavior::Replace,
                );
            },
            // TODO: This should use the ScriptThreadMessage::EvaluateJavaScript command
            WebDriverCommandMsg::ScriptCommand(browsing_context_id, cmd) => {
                let pipeline_id = self
                    .browsing_contexts
                    .get(&browsing_context_id)
                    .expect("ScriptCommand: Browsing context must exist at this point")
                    .pipeline_id;
                let control_msg = ScriptThreadMessage::WebDriverScriptCommand(pipeline_id, cmd);
                let result = match self.pipelines.get(&pipeline_id) {
                    Some(pipeline) => pipeline.event_loop.send(control_msg),
                    None => return warn!("{}: ScriptCommand after closure", pipeline_id),
                };
                if let Err(e) = result {
                    self.handle_send_error(pipeline_id, e);
                }
            },
            WebDriverCommandMsg::SendKeys(browsing_context_id, cmd) => {
                let pipeline_id = self
                    .browsing_contexts
                    .get(&browsing_context_id)
                    .expect("SendKeys: Browsing context must exist at this point")
                    .pipeline_id;
                let event_loop = match self.pipelines.get(&pipeline_id) {
                    Some(pipeline) => pipeline.event_loop.clone(),
                    None => return warn!("{}: SendKeys after closure", pipeline_id),
                };
                for event in cmd {
                    let event = match event {
                        WebDriverInputEvent::Keyboard(event) => ConstellationInputEvent {
                            pressed_mouse_buttons: self.pressed_mouse_buttons,
                            active_keyboard_modifiers: event.modifiers,
                            hit_test_result: None,
                            event: InputEvent::Keyboard(KeyboardEventWithWebDriverId::new(event)),
                        },
                        WebDriverInputEvent::Composition(event) => ConstellationInputEvent {
                            pressed_mouse_buttons: self.pressed_mouse_buttons,
                            active_keyboard_modifiers: self.active_keyboard_modifiers,
                            hit_test_result: None,
                            event: InputEvent::Ime(ImeEvent::Composition(event)),
                        },
                    };
                    let control_msg = ScriptThreadMessage::SendInputEvent(pipeline_id, event);
                    if let Err(e) = event_loop.send(control_msg) {
                        return self.handle_send_error(pipeline_id, e);
                    }
                }
            },
            WebDriverCommandMsg::KeyboardAction(
                browsing_context_id,
                key_event,
                msg_id,
                response_sender,
            ) => {
                self.webdriver.input_command_response_sender = Some(response_sender);

                let pipeline_id = match self.browsing_contexts.get(&browsing_context_id) {
                    Some(browsing_context) => browsing_context.pipeline_id,
                    None => {
                        return warn!("{}: KeyboardAction after closure", browsing_context_id);
                    },
                };
                let event_loop = match self.pipelines.get(&pipeline_id) {
                    Some(pipeline) => pipeline.event_loop.clone(),
                    None => return warn!("{}: KeyboardAction after closure", pipeline_id),
                };
                let event =
                    InputEvent::Keyboard(KeyboardEventWithWebDriverId::new(key_event.clone()))
                        .with_webdriver_message_id(msg_id);
                let control_msg = ScriptThreadMessage::SendInputEvent(
                    pipeline_id,
                    ConstellationInputEvent {
                        pressed_mouse_buttons: self.pressed_mouse_buttons,
                        active_keyboard_modifiers: key_event.modifiers,
                        hit_test_result: None,
                        event,
                    },
                );
                if let Err(e) = event_loop.send(control_msg) {
                    self.handle_send_error(pipeline_id, e)
                }
            },
            WebDriverCommandMsg::MouseButtonAction(
                webview_id,
                mouse_event_type,
                mouse_button,
                x,
                y,
                msg_id,
                response_sender,
            ) => {
                self.webdriver.input_command_response_sender = Some(response_sender);

                self.compositor_proxy
                    .send(CompositorMsg::WebDriverMouseButtonEvent(
                        webview_id,
                        mouse_event_type,
                        mouse_button,
                        x,
                        y,
                        msg_id,
                    ));
            },
            WebDriverCommandMsg::MouseMoveAction(webview_id, x, y, msg_id, response_sender) => {
                self.webdriver.input_command_response_sender = Some(response_sender);

                self.compositor_proxy
                    .send(CompositorMsg::WebDriverMouseMoveEvent(
                        webview_id, x, y, msg_id,
                    ));
            },
            WebDriverCommandMsg::WheelScrollAction(
                webview_id,
                x,
                y,
                delta_x,
                delta_y,
                msg_id,
                response_sender,
            ) => {
                self.webdriver.input_command_response_sender = Some(response_sender);

                self.compositor_proxy
                    .send(CompositorMsg::WebDriverWheelScrollEvent(
                        webview_id, x, y, delta_x, delta_y, msg_id,
                    ));
            },
            WebDriverCommandMsg::TakeScreenshot(webview_id, rect, response_sender) => {
                self.compositor_proxy.send(CompositorMsg::CreatePng(
                    webview_id,
                    rect,
                    response_sender,
                ));
            },
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn set_webview_throttled(&mut self, webview_id: WebViewId, throttled: bool) {
        let browsing_context_id = BrowsingContextId::from(webview_id);
        let pipeline_id = match self.browsing_contexts.get(&browsing_context_id) {
            Some(browsing_context) => browsing_context.pipeline_id,
            None => {
                return warn!("{browsing_context_id}: Tried to SetWebViewThrottled after closure");
            },
        };
        match self.pipelines.get(&pipeline_id) {
            None => warn!("{pipeline_id}: Tried to SetWebViewThrottled after closure"),
            Some(pipeline) => pipeline.set_throttled(throttled),
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn notify_history_changed(&self, webview_id: WebViewId) {
        // Send a flat projection of the history to embedder.
        // The final vector is a concatenation of the URLs of the past
        // entries, the current entry and the future entries.
        // URLs of inner frames are ignored and replaced with the URL
        // of the parent.

        let session_history = match self.webviews.get(webview_id) {
            Some(webview) => &webview.session_history,
            None => {
                return warn!(
                    "{}: Session history does not exist for browsing context",
                    webview_id
                );
            },
        };

        let browsing_context_id = BrowsingContextId::from(webview_id);
        let browsing_context = match self.browsing_contexts.get(&browsing_context_id) {
            Some(browsing_context) => browsing_context,
            None => {
                return warn!(
                    "notify_history_changed error after top-level browsing context closed."
                );
            },
        };

        let current_url = match self.pipelines.get(&browsing_context.pipeline_id) {
            Some(pipeline) => pipeline.url.clone(),
            None => {
                return warn!("{}: Refresh after closure", browsing_context.pipeline_id);
            },
        };

        // If URL was ignored, use the URL of the previous SessionHistoryEntry, which
        // is the URL of the parent browsing context.
        let resolve_url_future =
            |previous_url: &mut ServoUrl, diff: &SessionHistoryDiff| match *diff {
                SessionHistoryDiff::BrowsingContext {
                    browsing_context_id,
                    ref new_reloader,
                    ..
                } => {
                    if browsing_context_id == webview_id {
                        let url = match *new_reloader {
                            NeedsToReload::No(pipeline_id) => {
                                match self.pipelines.get(&pipeline_id) {
                                    Some(pipeline) => pipeline.url.clone(),
                                    None => previous_url.clone(),
                                }
                            },
                            NeedsToReload::Yes(_, ref load_data) => load_data.url.clone(),
                        };
                        *previous_url = url.clone();
                        Some(url)
                    } else {
                        Some(previous_url.clone())
                    }
                },
                _ => Some(previous_url.clone()),
            };

        let resolve_url_past = |previous_url: &mut ServoUrl, diff: &SessionHistoryDiff| match *diff
        {
            SessionHistoryDiff::BrowsingContext {
                browsing_context_id,
                ref old_reloader,
                ..
            } => {
                if browsing_context_id == webview_id {
                    let url = match *old_reloader {
                        NeedsToReload::No(pipeline_id) => match self.pipelines.get(&pipeline_id) {
                            Some(pipeline) => pipeline.url.clone(),
                            None => previous_url.clone(),
                        },
                        NeedsToReload::Yes(_, ref load_data) => load_data.url.clone(),
                    };
                    *previous_url = url.clone();
                    Some(url)
                } else {
                    Some(previous_url.clone())
                }
            },
            _ => Some(previous_url.clone()),
        };

        let mut entries: Vec<ServoUrl> = session_history
            .past
            .iter()
            .rev()
            .scan(current_url.clone(), &resolve_url_past)
            .collect();

        entries.reverse();

        let current_index = entries.len();

        entries.push(current_url.clone());

        entries.extend(
            session_history
                .future
                .iter()
                .rev()
                .scan(current_url, &resolve_url_future),
        );
        self.embedder_proxy.send(EmbedderMsg::HistoryChanged(
            webview_id,
            entries,
            current_index,
        ));
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn load_url_for_webdriver(
        &mut self,
        webview_id: WebViewId,
        load_data: LoadData,
        response_sender: IpcSender<WebDriverLoadStatus>,
        history_handling: NavigationHistoryBehavior,
    ) {
        let browsing_context_id = BrowsingContextId::from(webview_id);
        let pipeline_id = match self.browsing_contexts.get(&browsing_context_id) {
            Some(browsing_context) => browsing_context.pipeline_id,
            None => {
                return warn!(
                    "{}: Webdriver load for closed browsing context",
                    browsing_context_id
                );
            },
        };

        if let Some(new_pipeline_id) =
            self.load_url(webview_id, pipeline_id, load_data, history_handling)
        {
            debug!(
                "Setting up webdriver load notification for {:?}",
                new_pipeline_id
            );
            self.webdriver.load_channel = Some((new_pipeline_id, response_sender));
        } else {
            let _ = response_sender.send(WebDriverLoadStatus::Canceled);
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn change_session_history(&mut self, change: SessionHistoryChange) {
        debug!(
            "{}: Setting to {}",
            change.browsing_context_id, change.new_pipeline_id
        );

        // If the currently focused browsing context is a child of the browsing
        // context in which the page is being loaded, then update the focused
        // browsing context to be the one where the page is being loaded.
        if self.focused_browsing_context_is_descendant_of(change.browsing_context_id) {
            if let Some(webview) = self.webviews.get_mut(change.webview_id) {
                webview.focused_browsing_context_id = change.browsing_context_id;
            }
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
                            "{}: No NewBrowsingContextInfo for browsing context",
                            change.browsing_context_id,
                        );
                    },
                };
                self.new_browsing_context(
                    change.browsing_context_id,
                    change.webview_id,
                    change.new_pipeline_id,
                    new_context_info.parent_pipeline_id,
                    change.viewport_details,
                    new_context_info.is_private,
                    new_context_info.inherited_secure_context,
                    new_context_info.throttled,
                );
                self.update_activity(change.new_pipeline_id);
            },
            Some(old_pipeline_id) => {
                if let Some(pipeline) = self.pipelines.get(&old_pipeline_id) {
                    pipeline.set_throttled(true);
                }

                // https://html.spec.whatwg.org/multipage/#unload-a-document
                self.unload_document(old_pipeline_id);
                // Deactivate the old pipeline, and activate the new one.
                let (pipelines_to_close, states_to_close) = if let Some(replace_reloader) =
                    change.replace
                {
                    self.get_joint_session_history(change.webview_id)
                        .replace_reloader(
                            replace_reloader.clone(),
                            NeedsToReload::No(change.new_pipeline_id),
                        );

                    match replace_reloader {
                        NeedsToReload::No(pipeline_id) => (Some(vec![pipeline_id]), None),
                        NeedsToReload::Yes(..) => (None, None),
                    }
                } else {
                    let diff = SessionHistoryDiff::BrowsingContext {
                        browsing_context_id: change.browsing_context_id,
                        new_reloader: NeedsToReload::No(change.new_pipeline_id),
                        old_reloader: NeedsToReload::No(old_pipeline_id),
                    };

                    let mut pipelines_to_close = vec![];
                    let mut states_to_close = HashMap::new();

                    let diffs_to_close = self
                        .get_joint_session_history(change.webview_id)
                        .push_diff(diff);

                    for diff in diffs_to_close {
                        match diff {
                            SessionHistoryDiff::BrowsingContext { new_reloader, .. } => {
                                if let Some(pipeline_id) = new_reloader.alive_pipeline_id() {
                                    pipelines_to_close.push(pipeline_id);
                                }
                            },
                            SessionHistoryDiff::Pipeline {
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
                        let msg = ScriptThreadMessage::RemoveHistoryStates(pipeline_id, states);
                        let result = match self.pipelines.get(&pipeline_id) {
                            None => {
                                return warn!(
                                    "{}: Removed history states after closure",
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
            },
        }

        if let Some(top_level_id) = top_level_id {
            self.trim_history(top_level_id);
        }

        self.notify_focus_state(change.new_pipeline_id);

        self.notify_history_changed(change.webview_id);
        self.update_webview_in_compositor(change.webview_id);
    }

    /// Update the focus state of the specified pipeline that recently became
    /// active (thus doesn't have a focused container element) and may have
    /// out-dated information.
    fn notify_focus_state(&mut self, pipeline_id: PipelineId) {
        let pipeline = match self.pipelines.get(&pipeline_id) {
            Some(pipeline) => pipeline,
            None => return warn!("Pipeline {} is closed", pipeline_id),
        };

        let is_focused = match self.webviews.get(pipeline.webview_id) {
            Some(webview) => webview.focused_browsing_context_id == pipeline.browsing_context_id,
            None => {
                return warn!(
                    "Pipeline {}'s top-level browsing context {} is closed",
                    pipeline_id, pipeline.webview_id
                );
            },
        };

        // If the browsing context is focused, focus the document
        let msg = if is_focused {
            ScriptThreadMessage::FocusDocument(pipeline_id, pipeline.focus_sequence)
        } else {
            ScriptThreadMessage::Unfocus(pipeline_id, pipeline.focus_sequence)
        };
        if let Err(e) = pipeline.event_loop.send(msg) {
            self.handle_send_error(pipeline_id, e);
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn focused_browsing_context_is_descendant_of(
        &self,
        browsing_context_id: BrowsingContextId,
    ) -> bool {
        let focused_browsing_context_id = self
            .webviews
            .focused_webview()
            .map(|(_, webview)| webview.focused_browsing_context_id);
        focused_browsing_context_id.is_some_and(|focus_ctx_id| {
            focus_ctx_id == browsing_context_id ||
                self.fully_active_descendant_browsing_contexts_iter(browsing_context_id)
                    .any(|nested_ctx| nested_ctx.id == focus_ctx_id)
        })
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn trim_history(&mut self, webview_id: WebViewId) {
        let pipelines_to_evict = {
            let session_history = self.get_joint_session_history(webview_id);

            let history_length = pref!(session_history_max_length) as usize;

            // The past is stored with older entries at the front.
            // We reverse the iter so that newer entries are at the front and then
            // skip _n_ entries and evict the remaining entries.
            let mut pipelines_to_evict = session_history
                .past
                .iter()
                .rev()
                .map(|diff| diff.alive_old_pipeline())
                .skip(history_length)
                .flatten()
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
                    .flatten(),
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

        let session_history = self.get_joint_session_history(webview_id);

        for (alive_id, dead) in dead_pipelines {
            session_history.replace_reloader(NeedsToReload::No(alive_id), dead);
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_activate_document_msg(&mut self, pipeline_id: PipelineId) {
        debug!("{}: Document ready to activate", pipeline_id);

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
                            "{}: Activated document after closure of {}",
                            change.new_pipeline_id, change.browsing_context_id,
                        );
                    },
                },
            };
            if let Some(parent_pipeline_id) = parent_pipeline_id {
                if let Some(parent_pipeline) = self.pipelines.get(&parent_pipeline_id) {
                    let msg = ScriptThreadMessage::UpdatePipelineId(
                        parent_pipeline_id,
                        change.browsing_context_id,
                        change.webview_id,
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
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_change_viewport_details_msg(
        &mut self,
        webview_id: WebViewId,
        new_viewport_details: ViewportDetails,
        size_type: WindowSizeType,
    ) {
        debug!(
            "handle_change_viewport_details_msg: {:?}",
            new_viewport_details.size.to_untyped()
        );

        let browsing_context_id = BrowsingContextId::from(webview_id);
        self.resize_browsing_context(new_viewport_details, size_type, browsing_context_id);

        if let Some(response_sender) = self.webdriver.resize_channel.take() {
            let _ = response_sender.send(new_viewport_details.size);
        }
    }

    /// Called when the window exits from fullscreen mode
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_exit_fullscreen_msg(&mut self, webview_id: WebViewId) {
        let browsing_context_id = BrowsingContextId::from(webview_id);
        self.switch_fullscreen_mode(browsing_context_id);
    }

    /// Checks the state of all script and layout pipelines to see if they are idle
    /// and compares the current layout state to what the compositor has. This is used
    /// to check if the output image is "stable" and can be written as a screenshot
    /// for reftests.
    /// Since this function is only used in reftests, we do not harden it against panic.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
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
        let Some(webview_id) = self.webviews.focused_webview().map(|(id, _)| id) else {
            return ReadyToSave::NoTopLevelBrowsingContext;
        };

        // If there are pending loads, wait for those to complete.
        if !self.pending_changes.is_empty() {
            return ReadyToSave::PendingChanges;
        }

        // Step through the fully active browsing contexts, checking that the script thread is idle,
        // and that the current epoch of the layout matches what the compositor has painted. If all
        // these conditions are met, then the output image should not change and a reftest
        // screenshot can safely be written.
        for browsing_context in self.fully_active_browsing_contexts_iter(webview_id) {
            let pipeline_id = browsing_context.pipeline_id;
            trace!(
                "{}: Checking readiness of {}",
                browsing_context.id, pipeline_id
            );

            let pipeline = match self.pipelines.get(&pipeline_id) {
                None => {
                    warn!("{}: Screenshot while closing", pipeline_id);
                    continue;
                },
                Some(pipeline) => pipeline,
            };

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
            if browsing_context.viewport_details.size == Size2D::zero() {
                continue;
            }

            // Get the epoch that the compositor has drawn for this pipeline and then check if the
            // last laid out epoch matches what the compositor has drawn. If they match (and script
            // is idle) then this pipeline won't change again and can be considered stable.
            let compositor_epoch = pipeline_states.get(&browsing_context.pipeline_id);
            match compositor_epoch {
                Some(compositor_epoch) => {
                    if pipeline.layout_epoch != *compositor_epoch {
                        return ReadyToSave::EpochMismatch;
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
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
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
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn set_activity(&self, pipeline_id: PipelineId, activity: DocumentActivity) {
        debug!("{}: Setting activity to {:?}", pipeline_id, activity);
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
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn update_activity(&self, pipeline_id: PipelineId) {
        self.set_activity(pipeline_id, self.get_activity(pipeline_id));
    }

    /// Handle updating the size of a browsing context.
    /// This notifies every pipeline in the context of the new size.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn resize_browsing_context(
        &mut self,
        new_viewport_details: ViewportDetails,
        size_type: WindowSizeType,
        browsing_context_id: BrowsingContextId,
    ) {
        if let Some(browsing_context) = self.browsing_contexts.get_mut(&browsing_context_id) {
            browsing_context.viewport_details = new_viewport_details;
            // Send Resize (or ResizeInactive) messages to each pipeline in the frame tree.
            let pipeline_id = browsing_context.pipeline_id;
            let pipeline = match self.pipelines.get(&pipeline_id) {
                None => return warn!("{}: Resized after closing", pipeline_id),
                Some(pipeline) => pipeline,
            };
            let _ = pipeline.event_loop.send(ScriptThreadMessage::Resize(
                pipeline.id,
                new_viewport_details,
                size_type,
            ));
            let pipeline_ids = browsing_context
                .pipelines
                .iter()
                .filter(|pipeline_id| **pipeline_id != pipeline.id);
            for id in pipeline_ids {
                if let Some(pipeline) = self.pipelines.get(id) {
                    let _ = pipeline
                        .event_loop
                        .send(ScriptThreadMessage::ResizeInactive(
                            pipeline.id,
                            new_viewport_details,
                        ));
                }
            }
        }

        // Send resize message to any pending pipelines that aren't loaded yet.
        for change in &self.pending_changes {
            let pipeline_id = change.new_pipeline_id;
            let pipeline = match self.pipelines.get(&pipeline_id) {
                None => {
                    warn!("{}: Pending pipeline is closed", pipeline_id);
                    continue;
                },
                Some(pipeline) => pipeline,
            };
            if pipeline.browsing_context_id == browsing_context_id {
                let _ = pipeline.event_loop.send(ScriptThreadMessage::Resize(
                    pipeline.id,
                    new_viewport_details,
                    size_type,
                ));
            }
        }
    }

    /// Handle theme change events from the embedder and forward them to all appropriate `ScriptThread`s.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_theme_change(&mut self, webview_id: WebViewId, theme: Theme) {
        let Some(webview) = self.webviews.get_mut(webview_id) else {
            warn!("Received theme change request for uknown WebViewId: {webview_id:?}");
            return;
        };
        if !webview.set_theme(theme) {
            return;
        }

        for pipeline in self.pipelines.values() {
            if pipeline.webview_id != webview_id {
                continue;
            }
            if let Err(error) = pipeline
                .event_loop
                .send(ScriptThreadMessage::ThemeChange(pipeline.id, theme))
            {
                warn!(
                    "{}: Failed to send theme change event to pipeline ({error:?}).",
                    pipeline.id,
                );
            }
        }
    }

    // Handle switching from fullscreen mode
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn switch_fullscreen_mode(&mut self, browsing_context_id: BrowsingContextId) {
        if let Some(browsing_context) = self.browsing_contexts.get(&browsing_context_id) {
            let pipeline_id = browsing_context.pipeline_id;
            let pipeline = match self.pipelines.get(&pipeline_id) {
                None => {
                    return warn!(
                        "{}: Switched from fullscreen mode after closing",
                        pipeline_id
                    );
                },
                Some(pipeline) => pipeline,
            };
            let _ = pipeline
                .event_loop
                .send(ScriptThreadMessage::ExitFullScreen(pipeline.id));
        }
    }

    // Close and return the browsing context with the given id (and its children), if it exists.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn close_browsing_context(
        &mut self,
        browsing_context_id: BrowsingContextId,
        exit_mode: ExitPipelineMode,
    ) -> Option<BrowsingContext> {
        debug!("{}: Closing", browsing_context_id);

        self.close_browsing_context_children(
            browsing_context_id,
            DiscardBrowsingContext::Yes,
            exit_mode,
        );

        let browsing_context = match self.browsing_contexts.remove(&browsing_context_id) {
            Some(ctx) => ctx,
            None => {
                warn!("{browsing_context_id}: Closing twice");
                return None;
            },
        };

        {
            let session_history = self.get_joint_session_history(browsing_context.top_level_id);
            session_history.remove_entries_for_browsing_context(browsing_context_id);
        }

        if let Some(parent_pipeline_id) = browsing_context.parent_pipeline_id {
            match self.pipelines.get_mut(&parent_pipeline_id) {
                None => {
                    warn!("{parent_pipeline_id}: Child closed after parent");
                },
                Some(parent_pipeline) => {
                    parent_pipeline.remove_child(browsing_context_id);

                    // If `browsing_context_id` has focus, focus the parent
                    // browsing context
                    if let Some(webview) = self.webviews.get_mut(browsing_context.top_level_id) {
                        if webview.focused_browsing_context_id == browsing_context_id {
                            trace!(
                                "About-to-be-closed browsing context {} is currently focused, so \
                                focusing its parent {}",
                                browsing_context_id, parent_pipeline.browsing_context_id
                            );
                            webview.focused_browsing_context_id =
                                parent_pipeline.browsing_context_id;
                        }
                    } else {
                        warn!(
                            "Browsing context {} contains a reference to \
                                a non-existent top-level browsing context {}",
                            browsing_context_id, browsing_context.top_level_id
                        );
                    }
                },
            };
        }
        debug!("{}: Closed", browsing_context_id);
        Some(browsing_context)
    }

    // Close the children of a browsing context
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn close_browsing_context_children(
        &mut self,
        browsing_context_id: BrowsingContextId,
        dbc: DiscardBrowsingContext,
        exit_mode: ExitPipelineMode,
    ) {
        debug!("{}: Closing browsing context children", browsing_context_id);
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

        debug!("{}: Closed browsing context children", browsing_context_id);
    }

    // Discard the pipeline for a given document, udpdate the joint session history.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_discard_document(&mut self, webview_id: WebViewId, pipeline_id: PipelineId) {
        match self.webviews.get_mut(webview_id) {
            Some(webview) => {
                let load_data = match self.pipelines.get(&pipeline_id) {
                    Some(pipeline) => pipeline.load_data.clone(),
                    None => return warn!("{}: Discarding closed pipeline", pipeline_id),
                };
                webview.session_history.replace_reloader(
                    NeedsToReload::No(pipeline_id),
                    NeedsToReload::Yes(pipeline_id, load_data),
                );
            },
            None => {
                return warn!(
                    "{}: Discarding after closure of {}",
                    pipeline_id, webview_id,
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
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn unload_document(&self, pipeline_id: PipelineId) {
        if let Some(pipeline) = self.pipelines.get(&pipeline_id) {
            let msg = ScriptThreadMessage::UnloadDocument(pipeline_id);
            let _ = pipeline.event_loop.send(msg);
        }
    }

    // Close all pipelines at and beneath a given browsing context
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn close_pipeline(
        &mut self,
        pipeline_id: PipelineId,
        dbc: DiscardBrowsingContext,
        exit_mode: ExitPipelineMode,
    ) {
        debug!("{}: Closing", pipeline_id);

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
            None => return warn!("{}: Closing twice", pipeline_id),
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
        debug!("{}: Closed", pipeline_id);
    }

    // Randomly close a pipeline -if --random-pipeline-closure-probability is set
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn maybe_close_random_pipeline(&mut self) {
        match self.random_pipeline_closure {
            Some((ref mut rng, probability)) => {
                if probability <= rng.r#gen::<f32>() {
                    return;
                }
            },
            _ => return,
        };
        // In order to get repeatability, we sort the pipeline ids.
        let mut pipeline_ids: Vec<&PipelineId> = self.pipelines.keys().collect();
        pipeline_ids.sort();
        if let Some((ref mut rng, probability)) = self.random_pipeline_closure {
            if let Some(pipeline_id) = pipeline_ids.choose(rng) {
                if let Some(pipeline) = self.pipelines.get(pipeline_id) {
                    if self
                        .pending_changes
                        .iter()
                        .any(|change| change.new_pipeline_id == pipeline.id) &&
                        probability <= rng.r#gen::<f32>()
                    {
                        // We tend not to close pending pipelines, as that almost always
                        // results in pipelines being closed early in their lifecycle,
                        // and not stressing the constellation as much.
                        // https://github.com/servo/servo/issues/18852
                        info!("{}: Not closing pending pipeline", pipeline_id);
                    } else {
                        // Note that we deliberately do not do any of the tidying up
                        // associated with closing a pipeline. The constellation should cope!
                        warn!("{}: Randomly closing pipeline", pipeline_id);
                        pipeline.force_exit(DiscardBrowsingContext::No);
                    }
                }
            }
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn get_joint_session_history(&mut self, top_level_id: WebViewId) -> &mut JointSessionHistory {
        self.webviews
            .get_mut(top_level_id)
            .map(|webview| &mut webview.session_history)
            .expect("Unknown top-level browsing context")
    }

    // Convert a browsing context to a sendable form to pass to the compositor
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
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

    /// Send the frame tree for the given webview to the compositor.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn update_webview_in_compositor(&mut self, webview_id: WebViewId) {
        // Note that this function can panic, due to ipc-channel creation failure.
        // avoiding this panic would require a mechanism for dealing
        // with low-resource scenarios.
        let browsing_context_id = BrowsingContextId::from(webview_id);
        if let Some(frame_tree) = self.browsing_context_to_sendable(browsing_context_id) {
            debug!("{}: Sending frame tree", browsing_context_id);
            self.compositor_proxy
                .send(CompositorMsg::CreateOrUpdateWebView(frame_tree));
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_media_session_action_msg(&mut self, action: MediaSessionActionType) {
        if let Some(media_session_pipeline_id) = self.active_media_session {
            let result = match self.pipelines.get(&media_session_pipeline_id) {
                None => {
                    return warn!(
                        "{}: Got media session action request after closure",
                        media_session_pipeline_id,
                    );
                },
                Some(pipeline) => {
                    let msg =
                        ScriptThreadMessage::MediaSessionAction(media_session_pipeline_id, action);
                    pipeline.event_loop.send(msg)
                },
            };
            if let Err(e) = result {
                self.handle_send_error(media_session_pipeline_id, e);
            }
        } else {
            error!("Got a media session action but no active media session is registered");
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_set_scroll_states(
        &self,
        pipeline_id: PipelineId,
        scroll_states: HashMap<ExternalScrollId, LayoutVector2D>,
    ) {
        let Some(pipeline) = self.pipelines.get(&pipeline_id) else {
            warn!("Discarding scroll offset update for unknown pipeline");
            return;
        };
        if let Err(error) = pipeline
            .event_loop
            .send(ScriptThreadMessage::SetScrollStates(
                pipeline_id,
                scroll_states,
            ))
        {
            warn!("Could not send scroll offsets to pipeline: {pipeline_id:?}: {error:?}");
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(servo_profiling = true), level = "trace")
    )]
    fn handle_paint_metric(&mut self, pipeline_id: PipelineId, event: PaintMetricEvent) {
        let Some(pipeline) = self.pipelines.get(&pipeline_id) else {
            warn!("Discarding paint metric event for unknown pipeline");
            return;
        };
        let (metric_type, metric_value, first_reflow) = match event {
            PaintMetricEvent::FirstPaint(metric_value, first_reflow) => (
                ProgressiveWebMetricType::FirstPaint,
                metric_value,
                first_reflow,
            ),
            PaintMetricEvent::FirstContentfulPaint(metric_value, first_reflow) => (
                ProgressiveWebMetricType::FirstContentfulPaint,
                metric_value,
                first_reflow,
            ),
        };
        if let Err(error) = pipeline.event_loop.send(ScriptThreadMessage::PaintMetric(
            pipeline_id,
            metric_type,
            metric_value,
            first_reflow,
        )) {
            warn!("Could not sent paint metric event to pipeline: {pipeline_id:?}: {error:?}");
        }
    }
}
