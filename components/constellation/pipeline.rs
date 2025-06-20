/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashSet;
use std::rc::Rc;
use std::sync::Arc;

use background_hang_monitor::HangMonitorRegister;
use background_hang_monitor_api::{
    BackgroundHangMonitorControlMsg, BackgroundHangMonitorRegister, HangMonitorAlert,
};
use base::Epoch;
use base::id::{
    BrowsingContextId, HistoryStateId, PipelineId, PipelineNamespace, PipelineNamespaceId,
    PipelineNamespaceRequest, WebViewId,
};
#[cfg(feature = "bluetooth")]
use bluetooth_traits::BluetoothRequest;
use canvas_traits::webgl::WebGLPipeline;
use compositing_traits::{
    CompositionPipeline, CompositorMsg, CompositorProxy, CrossProcessCompositorApi,
};
use constellation_traits::{LoadData, SWManagerMsg, ScriptToConstellationChan};
use crossbeam_channel::{Sender, unbounded};
use devtools_traits::{DevtoolsControlMsg, ScriptToDevtoolsControlMsg};
use embedder_traits::user_content_manager::UserContentManager;
use embedder_traits::{AnimationState, FocusSequenceNumber, Theme, ViewportDetails};
use fonts::{SystemFontServiceProxy, SystemFontServiceProxySender};
use ipc_channel::Error;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use layout_api::{LayoutFactory, ScriptThreadFactory};
use log::{debug, error, warn};
use media::WindowGLContext;
use net::image_cache::ImageCacheImpl;
use net_traits::ResourceThreads;
use net_traits::image_cache::ImageCache;
use profile::system_reporter;
use profile_traits::mem::{ProfilerMsg, Reporter};
use profile_traits::{mem as profile_mem, time};
use script_traits::{
    DiscardBrowsingContext, DocumentActivity, InitialScriptState, NewLayoutInfo,
    ScriptThreadMessage,
};
use serde::{Deserialize, Serialize};
use servo_config::opts::{self, Opts};
use servo_config::prefs::{self, Preferences};
use servo_url::ServoUrl;

use crate::event_loop::EventLoop;
use crate::process_manager::Process;
use crate::sandboxing::{UnprivilegedContent, spawn_multiprocess};

/// A `Pipeline` is the constellation's view of a `Window`. Each pipeline has an event loop
/// (executed by a script thread). A script thread may be responsible for many pipelines.
pub struct Pipeline {
    /// The ID of the pipeline.
    pub id: PipelineId,

    /// The ID of the browsing context that contains this Pipeline.
    pub browsing_context_id: BrowsingContextId,

    /// The [`WebViewId`] of the `WebView` that contains this Pipeline.
    pub webview_id: WebViewId,

    pub opener: Option<BrowsingContextId>,

    /// The event loop handling this pipeline.
    pub event_loop: Rc<EventLoop>,

    /// A channel to the compositor.
    pub compositor_proxy: CompositorProxy,

    /// The most recently loaded URL in this pipeline.
    /// Note that this URL can change, for example if the page navigates
    /// to a hash URL.
    pub url: ServoUrl,

    /// Whether this pipeline is currently running animations. Pipelines that are running
    /// animations cause composites to be continually scheduled.
    pub animation_state: AnimationState,

    /// The child browsing contexts of this pipeline (these are iframes in the document).
    pub children: Vec<BrowsingContextId>,

    /// The Load Data used to create this pipeline.
    pub load_data: LoadData,

    /// The active history state for this pipeline.
    pub history_state_id: Option<HistoryStateId>,

    /// The history states owned by this pipeline.
    pub history_states: HashSet<HistoryStateId>,

    /// Has this pipeline received a notification that it is completely loaded?
    pub completely_loaded: bool,

    /// The title of this pipeline's document.
    pub title: String,

    /// The last compositor [`Epoch`] that was laid out in this pipeline if "exit after load" is
    /// enabled.
    pub layout_epoch: Epoch,

    pub focus_sequence: FocusSequenceNumber,
}

/// Initial setup data needed to construct a pipeline.
///
/// *DO NOT* add any Senders to this unless you absolutely know what you're doing, or pcwalton will
/// have to rewrite your code. Use IPC senders instead.
pub struct InitialPipelineState {
    /// The ID of the pipeline to create.
    pub id: PipelineId,

    /// The ID of the browsing context that contains this Pipeline.
    pub browsing_context_id: BrowsingContextId,

    /// The ID of the top-level browsing context that contains this Pipeline.
    pub webview_id: WebViewId,

    /// The ID of the parent pipeline and frame type, if any.
    /// If `None`, this is the root.
    pub parent_pipeline_id: Option<PipelineId>,

    pub opener: Option<BrowsingContextId>,

    /// A channel to the associated constellation.
    pub script_to_constellation_chan: ScriptToConstellationChan,

    /// A sender to request pipeline namespace ids.
    pub namespace_request_sender: IpcSender<PipelineNamespaceRequest>,

    /// A handle to register components for hang monitoring.
    /// None when in multiprocess mode.
    pub background_monitor_register: Option<Box<dyn BackgroundHangMonitorRegister>>,

    /// A channel for the background hang monitor to send messages to the constellation.
    pub background_hang_monitor_to_constellation_chan: IpcSender<HangMonitorAlert>,

    /// A fatory for creating layouts to be used by the ScriptThread.
    pub layout_factory: Arc<dyn LayoutFactory>,

    /// A channel to the compositor.
    pub compositor_proxy: CompositorProxy,

    /// A channel to the developer tools, if applicable.
    pub devtools_sender: Option<Sender<DevtoolsControlMsg>>,

    /// A channel to the bluetooth thread.
    #[cfg(feature = "bluetooth")]
    pub bluetooth_thread: IpcSender<BluetoothRequest>,

    /// A channel to the service worker manager thread
    pub swmanager_thread: IpcSender<SWManagerMsg>,

    /// A proxy to the system font service, responsible for managing the list of system fonts.
    pub system_font_service: Arc<SystemFontServiceProxy>,

    /// Channels to the resource-related threads.
    pub resource_threads: ResourceThreads,

    /// A channel to the time profiler thread.
    pub time_profiler_chan: time::ProfilerChan,

    /// A channel to the memory profiler thread.
    pub mem_profiler_chan: profile_mem::ProfilerChan,

    /// The initial [`ViewportDetails`] to use when starting this new [`Pipeline`].
    pub viewport_details: ViewportDetails,

    /// The initial [`Theme`] to use when starting this new [`Pipeline`].
    pub theme: Theme,

    /// The ID of the pipeline namespace for this script thread.
    pub pipeline_namespace_id: PipelineNamespaceId,

    /// The event loop to run in, if applicable.
    pub event_loop: Option<Rc<EventLoop>>,

    /// Information about the page to load.
    pub load_data: LoadData,

    /// Whether the browsing context in which pipeline is embedded is throttled,
    /// using less resources by stopping animations and running timers at a
    /// heavily limited rate. This field is only used to notify script and
    /// compositor threads after spawning a pipeline.
    pub prev_throttled: bool,

    /// A channel to the WebGL thread.
    pub webgl_chan: Option<WebGLPipeline>,

    /// The XR device registry
    pub webxr_registry: Option<webxr_api::Registry>,

    /// Application window's GL Context for Media player
    pub player_context: WindowGLContext,

    /// The image bytes associated with the RippyPNG embedder resource.
    pub rippy_data: Vec<u8>,

    /// User content manager
    pub user_content_manager: UserContentManager,
}

pub struct NewPipeline {
    pub pipeline: Pipeline,
    pub bhm_control_chan: Option<IpcSender<BackgroundHangMonitorControlMsg>>,
    pub lifeline: Option<(IpcReceiver<()>, Process)>,
}

impl Pipeline {
    /// Possibly starts a script thread, in a new process if requested.
    pub fn spawn<STF: ScriptThreadFactory>(
        state: InitialPipelineState,
    ) -> Result<NewPipeline, Error> {
        // Note: we allow channel creation to panic, since recovering from this
        // probably requires a general low-memory strategy.
        let (script_chan, (bhm_control_chan, lifeline)) = match state.event_loop {
            Some(script_chan) => {
                let new_layout_info = NewLayoutInfo {
                    parent_info: state.parent_pipeline_id,
                    new_pipeline_id: state.id,
                    browsing_context_id: state.browsing_context_id,
                    webview_id: state.webview_id,
                    opener: state.opener,
                    load_data: state.load_data.clone(),
                    viewport_details: state.viewport_details,
                    theme: state.theme,
                };

                if let Err(e) = script_chan.send(ScriptThreadMessage::AttachLayout(new_layout_info))
                {
                    warn!("Sending to script during pipeline creation failed ({})", e);
                }
                (script_chan, (None, None))
            },
            None => {
                let (script_chan, script_port) = ipc::channel().expect("Pipeline script chan");

                // Route messages coming from content to devtools as appropriate.
                let script_to_devtools_ipc_sender =
                    state.devtools_sender.as_ref().map(|devtools_sender| {
                        let (script_to_devtools_ipc_sender, script_to_devtools_ipc_receiver) =
                            ipc::channel().expect("Pipeline script to devtools chan");
                        let devtools_sender = (*devtools_sender).clone();
                        ROUTER.add_typed_route(
                            script_to_devtools_ipc_receiver,
                            Box::new(move |message| match message {
                                Err(e) => {
                                    error!("Cast to ScriptToDevtoolsControlMsg failed ({}).", e)
                                },
                                Ok(message) => {
                                    if let Err(e) = devtools_sender
                                        .send(DevtoolsControlMsg::FromScript(message))
                                    {
                                        warn!("Sending to devtools failed ({:?})", e)
                                    }
                                },
                            }),
                        );
                        script_to_devtools_ipc_sender
                    });

                let mut unprivileged_pipeline_content = UnprivilegedPipelineContent {
                    id: state.id,
                    browsing_context_id: state.browsing_context_id,
                    webview_id: state.webview_id,
                    parent_pipeline_id: state.parent_pipeline_id,
                    opener: state.opener,
                    script_to_constellation_chan: state.script_to_constellation_chan.clone(),
                    namespace_request_sender: state.namespace_request_sender,
                    background_hang_monitor_to_constellation_chan: state
                        .background_hang_monitor_to_constellation_chan
                        .clone(),
                    bhm_control_port: None,
                    devtools_ipc_sender: script_to_devtools_ipc_sender,
                    #[cfg(feature = "bluetooth")]
                    bluetooth_thread: state.bluetooth_thread,
                    swmanager_thread: state.swmanager_thread,
                    system_font_service: state.system_font_service.to_sender(),
                    resource_threads: state.resource_threads,
                    time_profiler_chan: state.time_profiler_chan,
                    mem_profiler_chan: state.mem_profiler_chan,
                    viewport_details: state.viewport_details,
                    theme: state.theme,
                    script_chan: script_chan.clone(),
                    load_data: state.load_data.clone(),
                    script_port,
                    opts: (*opts::get()).clone(),
                    prefs: Box::new(prefs::get().clone()),
                    pipeline_namespace_id: state.pipeline_namespace_id,
                    cross_process_compositor_api: state
                        .compositor_proxy
                        .cross_process_compositor_api
                        .clone(),
                    webgl_chan: state.webgl_chan,
                    webxr_registry: state.webxr_registry,
                    player_context: state.player_context,
                    rippy_data: state.rippy_data,
                    user_content_manager: state.user_content_manager,
                    lifeline_sender: None,
                };

                // Spawn the child process.
                //
                // Yes, that's all there is to it!
                let multiprocess_data = if opts::get().multiprocess {
                    let (bhm_control_chan, bhm_control_port) =
                        ipc::channel().expect("Sampler chan");
                    unprivileged_pipeline_content.bhm_control_port = Some(bhm_control_port);
                    let (sender, receiver) =
                        ipc::channel().expect("Failed to create lifeline channel");
                    unprivileged_pipeline_content.lifeline_sender = Some(sender);
                    let process = unprivileged_pipeline_content.spawn_multiprocess()?;
                    (Some(bhm_control_chan), Some((receiver, process)))
                } else {
                    // Should not be None in single-process mode.
                    let register = state
                        .background_monitor_register
                        .expect("Couldn't start content, no background monitor has been initiated");
                    unprivileged_pipeline_content.start_all::<STF>(
                        false,
                        state.layout_factory,
                        register,
                    );
                    (None, None)
                };

                (EventLoop::new(script_chan), multiprocess_data)
            },
        };

        let pipeline = Pipeline::new(
            state.id,
            state.browsing_context_id,
            state.webview_id,
            state.opener,
            script_chan,
            state.compositor_proxy,
            state.prev_throttled,
            state.load_data,
        );
        Ok(NewPipeline {
            pipeline,
            bhm_control_chan,
            lifeline,
        })
    }

    /// Creates a new `Pipeline`, after the script has been spawned.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: PipelineId,
        browsing_context_id: BrowsingContextId,
        webview_id: WebViewId,
        opener: Option<BrowsingContextId>,
        event_loop: Rc<EventLoop>,
        compositor_proxy: CompositorProxy,
        throttled: bool,
        load_data: LoadData,
    ) -> Pipeline {
        let pipeline = Pipeline {
            id,
            browsing_context_id,
            webview_id,
            opener,
            event_loop,
            compositor_proxy,
            url: load_data.url.clone(),
            children: vec![],
            animation_state: AnimationState::NoAnimationsPresent,
            load_data,
            history_state_id: None,
            history_states: HashSet::new(),
            completely_loaded: false,
            title: String::new(),
            layout_epoch: Epoch(0),
            focus_sequence: FocusSequenceNumber::default(),
        };

        pipeline.set_throttled(throttled);

        pipeline
    }

    /// Let the `ScriptThread` for this [`Pipeline`] know that it has exited. If the `ScriptThread` hasn't
    /// panicked and is still alive, it will send a `PipelineExited` message back to the `Constellation`
    /// when it finishes cleaning up.
    pub fn send_exit_message_to_script(&self, discard_bc: DiscardBrowsingContext) {
        debug!("{:?} Sending exit message to script", self.id);

        // Script thread handles shutting down layout, and layout handles shutting down the painter.
        // For now, if the script thread has failed, we give up on clean shutdown.
        if let Err(error) = self.event_loop.send(ScriptThreadMessage::ExitPipeline(
            self.webview_id,
            self.id,
            discard_bc,
        )) {
            warn!("Sending script exit message failed ({error}).");
        }
    }

    /// Notify this pipeline of its activity.
    pub fn set_activity(&self, activity: DocumentActivity) {
        let msg = ScriptThreadMessage::SetDocumentActivity(self.id, activity);
        if let Err(e) = self.event_loop.send(msg) {
            warn!("Sending activity message failed ({}).", e);
        }
    }

    /// The compositor's view of a pipeline.
    pub fn to_sendable(&self) -> CompositionPipeline {
        CompositionPipeline {
            id: self.id,
            webview_id: self.webview_id,
        }
    }

    /// Add a new child browsing context.
    pub fn add_child(&mut self, browsing_context_id: BrowsingContextId) {
        self.children.push(browsing_context_id);
    }

    /// Remove a child browsing context.
    pub fn remove_child(&mut self, browsing_context_id: BrowsingContextId) {
        match self
            .children
            .iter()
            .position(|id| *id == browsing_context_id)
        {
            None => {
                warn!(
                    "Pipeline remove child already removed ({:?}).",
                    browsing_context_id
                )
            },
            Some(index) => {
                self.children.remove(index);
            },
        }
    }

    /// Set whether to make pipeline use less resources, by stopping animations and
    /// running timers at a heavily limited rate.
    pub fn set_throttled(&self, throttled: bool) {
        let script_msg = ScriptThreadMessage::SetThrottled(self.id, throttled);
        let compositor_msg = CompositorMsg::SetThrottled(self.webview_id, self.id, throttled);
        let err = self.event_loop.send(script_msg);
        if let Err(e) = err {
            warn!("Sending SetThrottled to script failed ({}).", e);
        }
        self.compositor_proxy.send(compositor_msg);
    }
}

/// Creating a new pipeline may require creating a new event loop.
/// This is the data used to initialize the event loop.
/// TODO: simplify this, and unify it with `InitialPipelineState` if possible.
#[derive(Deserialize, Serialize)]
pub struct UnprivilegedPipelineContent {
    id: PipelineId,
    webview_id: WebViewId,
    browsing_context_id: BrowsingContextId,
    parent_pipeline_id: Option<PipelineId>,
    opener: Option<BrowsingContextId>,
    namespace_request_sender: IpcSender<PipelineNamespaceRequest>,
    script_to_constellation_chan: ScriptToConstellationChan,
    background_hang_monitor_to_constellation_chan: IpcSender<HangMonitorAlert>,
    bhm_control_port: Option<IpcReceiver<BackgroundHangMonitorControlMsg>>,
    devtools_ipc_sender: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
    #[cfg(feature = "bluetooth")]
    bluetooth_thread: IpcSender<BluetoothRequest>,
    swmanager_thread: IpcSender<SWManagerMsg>,
    system_font_service: SystemFontServiceProxySender,
    resource_threads: ResourceThreads,
    time_profiler_chan: time::ProfilerChan,
    mem_profiler_chan: profile_mem::ProfilerChan,
    viewport_details: ViewportDetails,
    theme: Theme,
    script_chan: IpcSender<ScriptThreadMessage>,
    load_data: LoadData,
    script_port: IpcReceiver<ScriptThreadMessage>,
    opts: Opts,
    prefs: Box<Preferences>,
    pipeline_namespace_id: PipelineNamespaceId,
    cross_process_compositor_api: CrossProcessCompositorApi,
    webgl_chan: Option<WebGLPipeline>,
    webxr_registry: Option<webxr_api::Registry>,
    player_context: WindowGLContext,
    rippy_data: Vec<u8>,
    user_content_manager: UserContentManager,
    lifeline_sender: Option<IpcSender<()>>,
}

impl UnprivilegedPipelineContent {
    pub fn start_all<STF: ScriptThreadFactory>(
        self,
        wait_for_completion: bool,
        layout_factory: Arc<dyn LayoutFactory>,
        background_hang_monitor_register: Box<dyn BackgroundHangMonitorRegister>,
    ) {
        // Setup pipeline-namespace-installing for all threads in this process.
        // Idempotent in single-process mode.
        PipelineNamespace::set_installer_sender(self.namespace_request_sender);

        let image_cache = Arc::new(ImageCacheImpl::new(
            self.cross_process_compositor_api.clone(),
            self.rippy_data,
        ));
        let (content_process_shutdown_chan, content_process_shutdown_port) = unbounded();
        STF::create(
            InitialScriptState {
                id: self.id,
                browsing_context_id: self.browsing_context_id,
                webview_id: self.webview_id,
                parent_info: self.parent_pipeline_id,
                opener: self.opener,
                constellation_sender: self.script_chan.clone(),
                constellation_receiver: self.script_port,
                pipeline_to_constellation_sender: self.script_to_constellation_chan.clone(),
                background_hang_monitor_register: background_hang_monitor_register.clone(),
                #[cfg(feature = "bluetooth")]
                bluetooth_sender: self.bluetooth_thread,
                resource_threads: self.resource_threads,
                image_cache: image_cache.clone(),
                time_profiler_sender: self.time_profiler_chan.clone(),
                memory_profiler_sender: self.mem_profiler_chan.clone(),
                devtools_server_sender: self.devtools_ipc_sender,
                viewport_details: self.viewport_details,
                theme: self.theme,
                pipeline_namespace_id: self.pipeline_namespace_id,
                content_process_shutdown_sender: content_process_shutdown_chan,
                webgl_chan: self.webgl_chan,
                webxr_registry: self.webxr_registry,
                compositor_api: self.cross_process_compositor_api.clone(),
                player_context: self.player_context.clone(),
                inherited_secure_context: self.load_data.inherited_secure_context,
                user_content_manager: self.user_content_manager,
            },
            layout_factory,
            Arc::new(self.system_font_service.to_proxy()),
            self.load_data.clone(),
        );

        if wait_for_completion {
            match content_process_shutdown_port.recv() {
                Ok(()) => {},
                Err(_) => error!("Script-thread shut-down unexpectedly"),
            }
        }
    }

    pub fn spawn_multiprocess(self) -> Result<Process, Error> {
        spawn_multiprocess(UnprivilegedContent::Pipeline(self))
    }

    pub fn register_with_background_hang_monitor(
        &mut self,
    ) -> Box<dyn BackgroundHangMonitorRegister> {
        HangMonitorRegister::init(
            self.background_hang_monitor_to_constellation_chan.clone(),
            self.bhm_control_port.take().expect("no sampling profiler?"),
            opts::get().background_hang_monitor,
        )
    }

    pub fn script_to_constellation_chan(&self) -> &ScriptToConstellationChan {
        &self.script_to_constellation_chan
    }

    pub fn opts(&self) -> Opts {
        self.opts.clone()
    }

    pub fn prefs(&self) -> &Preferences {
        &self.prefs
    }

    pub fn register_system_memory_reporter(&self) {
        // Register the system memory reporter, which will run on its own thread. It never needs to
        // be unregistered, because as long as the memory profiler is running the system memory
        // reporter can make measurements.
        let (system_reporter_sender, system_reporter_receiver) =
            ipc::channel().expect("failed to create ipc channel");
        ROUTER.add_typed_route(
            system_reporter_receiver,
            Box::new(|message| {
                if let Ok(request) = message {
                    system_reporter::collect_reports(request);
                }
            }),
        );
        self.mem_profiler_chan.send(ProfilerMsg::RegisterReporter(
            format!("system-content-{}", std::process::id()),
            Reporter(system_reporter_sender),
        ));
    }
}
