/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

use background_hang_monitor::HangMonitorRegister;
use background_hang_monitor_api::{
    BackgroundHangMonitorControlMsg, BackgroundHangMonitorRegister, HangMonitorAlert,
};
use base::id::{
    BrowsingContextId, HistoryStateId, PipelineId, PipelineNamespace, PipelineNamespaceId,
    PipelineNamespaceRequest, TopLevelBrowsingContextId,
};
use base::Epoch;
use bluetooth_traits::BluetoothRequest;
use canvas_traits::webgl::WebGLPipeline;
use compositing_traits::{CompositionPipeline, CompositorMsg, CompositorProxy};
use crossbeam_channel::{unbounded, Sender};
use devtools_traits::{DevtoolsControlMsg, ScriptToDevtoolsControlMsg};
use embedder_traits::EventLoopWaker;
use fonts::FontCacheThread;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use ipc_channel::Error;
use log::{debug, error, warn};
use media::WindowGLContext;
use net::image_cache::ImageCacheImpl;
use net_traits::image_cache::ImageCache;
use net_traits::ResourceThreads;
use profile_traits::{mem as profile_mem, time};
use script_layout_interface::{LayoutFactory, ScriptThreadFactory};
use script_traits::{
    AnimationState, ConstellationControlMsg, DiscardBrowsingContext, DocumentActivity,
    InitialScriptState, LayoutMsg, LoadData, NewLayoutInfo, SWManagerMsg,
    ScriptToConstellationChan, TimerSchedulerMsg, WindowSizeData,
};
use serde::{Deserialize, Serialize};
use servo_config::opts::{self, Opts};
use servo_config::prefs;
use servo_config::prefs::PrefValue;
use servo_url::ServoUrl;
use webrender_api::DocumentId;

use crate::event_loop::EventLoop;
use crate::sandboxing::{spawn_multiprocess, UnprivilegedContent};

/// A `Pipeline` is the constellation's view of a `Window`. Each pipeline has an event loop
/// (executed by a script thread). A script thread may be responsible for many pipelines.
pub struct Pipeline {
    /// The ID of the pipeline.
    pub id: PipelineId,

    /// The ID of the browsing context that contains this Pipeline.
    pub browsing_context_id: BrowsingContextId,

    /// The ID of the top-level browsing context that contains this Pipeline.
    pub top_level_browsing_context_id: TopLevelBrowsingContextId,

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
    pub top_level_browsing_context_id: TopLevelBrowsingContextId,

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

    /// A channel for the layout to send messages to the constellation.
    pub layout_to_constellation_chan: IpcSender<LayoutMsg>,

    /// A fatory for creating layouts to be used by the ScriptThread.
    pub layout_factory: Arc<dyn LayoutFactory>,

    /// A channel to schedule timer events.
    pub scheduler_chan: IpcSender<TimerSchedulerMsg>,

    /// A channel to the compositor.
    pub compositor_proxy: CompositorProxy,

    /// A channel to the developer tools, if applicable.
    pub devtools_sender: Option<Sender<DevtoolsControlMsg>>,

    /// A channel to the bluetooth thread.
    pub bluetooth_thread: IpcSender<BluetoothRequest>,

    /// A channel to the service worker manager thread
    pub swmanager_thread: IpcSender<SWManagerMsg>,

    /// A channel to the font cache thread.
    pub font_cache_thread: FontCacheThread,

    /// Channels to the resource-related threads.
    pub resource_threads: ResourceThreads,

    /// A channel to the time profiler thread.
    pub time_profiler_chan: time::ProfilerChan,

    /// A channel to the memory profiler thread.
    pub mem_profiler_chan: profile_mem::ProfilerChan,

    /// Information about the initial window size.
    pub window_size: WindowSizeData,

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

    /// Webrender api.
    pub webrender_image_api_sender: webrender_traits::WebRenderNetApi,

    /// Webrender api.
    pub webrender_api_sender: webrender_traits::WebRenderScriptApi,

    /// The ID of the document processed by this script thread.
    pub webrender_document: DocumentId,

    /// A channel to the WebGL thread.
    pub webgl_chan: Option<WebGLPipeline>,

    /// The XR device registry
    pub webxr_registry: webxr_api::Registry,

    /// Application window's GL Context for Media player
    pub player_context: WindowGLContext,

    /// Mechanism to force the compositor to process events.
    pub event_loop_waker: Option<Box<dyn EventLoopWaker>>,

    /// User agent string to report in network requests.
    pub user_agent: Cow<'static, str>,
}

pub struct NewPipeline {
    pub pipeline: Pipeline,
    pub bhm_control_chan: Option<IpcSender<BackgroundHangMonitorControlMsg>>,
}

impl Pipeline {
    /// Possibly starts a script thread, in a new process if requested.
    pub fn spawn<STF: ScriptThreadFactory>(
        state: InitialPipelineState,
    ) -> Result<NewPipeline, Error> {
        // Note: we allow channel creation to panic, since recovering from this
        // probably requires a general low-memory strategy.
        let (script_chan, bhm_control_chan) = match state.event_loop {
            Some(script_chan) => {
                let new_layout_info = NewLayoutInfo {
                    parent_info: state.parent_pipeline_id,
                    new_pipeline_id: state.id,
                    browsing_context_id: state.browsing_context_id,
                    top_level_browsing_context_id: state.top_level_browsing_context_id,
                    opener: state.opener,
                    load_data: state.load_data.clone(),
                    window_size: state.window_size,
                };

                if let Err(e) =
                    script_chan.send(ConstellationControlMsg::AttachLayout(new_layout_info))
                {
                    warn!("Sending to script during pipeline creation failed ({})", e);
                }
                (script_chan, None)
            },
            None => {
                let (script_chan, script_port) = ipc::channel().expect("Pipeline script chan");

                // Route messages coming from content to devtools as appropriate.
                let script_to_devtools_ipc_sender =
                    state.devtools_sender.as_ref().map(|devtools_sender| {
                        let (script_to_devtools_ipc_sender, script_to_devtools_ipc_receiver) =
                            ipc::channel().expect("Pipeline script to devtools chan");
                        let devtools_sender = (*devtools_sender).clone();
                        ROUTER.add_route(
                            script_to_devtools_ipc_receiver.to_opaque(),
                            Box::new(move |message| {
                                match message.to::<ScriptToDevtoolsControlMsg>() {
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
                                }
                            }),
                        );
                        script_to_devtools_ipc_sender
                    });

                let mut unprivileged_pipeline_content = UnprivilegedPipelineContent {
                    id: state.id,
                    browsing_context_id: state.browsing_context_id,
                    top_level_browsing_context_id: state.top_level_browsing_context_id,
                    parent_pipeline_id: state.parent_pipeline_id,
                    opener: state.opener,
                    script_to_constellation_chan: state.script_to_constellation_chan.clone(),
                    namespace_request_sender: state.namespace_request_sender,
                    background_hang_monitor_to_constellation_chan: state
                        .background_hang_monitor_to_constellation_chan
                        .clone(),
                    bhm_control_port: None,
                    scheduler_chan: state.scheduler_chan,
                    devtools_ipc_sender: script_to_devtools_ipc_sender,
                    bluetooth_thread: state.bluetooth_thread,
                    swmanager_thread: state.swmanager_thread,
                    font_cache_thread: state.font_cache_thread,
                    resource_threads: state.resource_threads,
                    time_profiler_chan: state.time_profiler_chan,
                    mem_profiler_chan: state.mem_profiler_chan,
                    window_size: state.window_size,
                    layout_to_constellation_chan: state.layout_to_constellation_chan,
                    script_chan: script_chan.clone(),
                    load_data: state.load_data.clone(),
                    script_port,
                    opts: (*opts::get()).clone(),
                    prefs: prefs::pref_map().iter().collect(),
                    pipeline_namespace_id: state.pipeline_namespace_id,
                    webrender_api_sender: state.webrender_api_sender,
                    webrender_image_api_sender: state.webrender_image_api_sender,
                    webrender_document: state.webrender_document,
                    webgl_chan: state.webgl_chan,
                    webxr_registry: state.webxr_registry,
                    player_context: state.player_context,
                    user_agent: state.user_agent,
                };

                // Spawn the child process.
                //
                // Yes, that's all there is to it!
                let bhm_control_chan = if opts::multiprocess() {
                    let (bhm_control_chan, bhm_control_port) =
                        ipc::channel().expect("Sampler chan");
                    unprivileged_pipeline_content.bhm_control_port = Some(bhm_control_port);
                    unprivileged_pipeline_content.spawn_multiprocess()?;
                    Some(bhm_control_chan)
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
                    None
                };

                (EventLoop::new(script_chan), bhm_control_chan)
            },
        };

        let pipeline = Pipeline::new(
            state.id,
            state.browsing_context_id,
            state.top_level_browsing_context_id,
            state.opener,
            script_chan,
            state.compositor_proxy,
            state.prev_throttled,
            state.load_data,
        );
        Ok(NewPipeline {
            pipeline,
            bhm_control_chan,
        })
    }

    /// Creates a new `Pipeline`, after the script has been spawned.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: PipelineId,
        browsing_context_id: BrowsingContextId,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
        opener: Option<BrowsingContextId>,
        event_loop: Rc<EventLoop>,
        compositor_proxy: CompositorProxy,
        throttled: bool,
        load_data: LoadData,
    ) -> Pipeline {
        let pipeline = Pipeline {
            id,
            browsing_context_id,
            top_level_browsing_context_id,
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
        };

        pipeline.set_throttled(throttled);

        pipeline
    }

    /// A normal exit of the pipeline, which waits for the compositor,
    /// and delegates layout shutdown to the script thread.
    pub fn exit(&self, discard_bc: DiscardBrowsingContext) {
        debug!("pipeline {:?} exiting", self.id);

        // The compositor wants to know when pipelines shut down too.
        // It may still have messages to process from these other threads
        // before they can be safely shut down.
        // It's OK for the constellation to block on the compositor,
        // since the compositor never blocks on the constellation.
        if let Ok((sender, receiver)) = ipc::channel() {
            self.compositor_proxy
                .send(CompositorMsg::PipelineExited(self.id, sender));
            if let Err(e) = receiver.recv() {
                warn!("Sending exit message failed ({:?}).", e);
            }
        }

        // Script thread handles shutting down layout, and layout handles shutting down the painter.
        // For now, if the script thread has failed, we give up on clean shutdown.
        let msg = ConstellationControlMsg::ExitPipeline(self.id, discard_bc);
        if let Err(e) = self.event_loop.send(msg) {
            warn!("Sending script exit message failed ({}).", e);
        }
    }

    /// A forced exit of the shutdown, which does not wait for the compositor,
    /// or for the script thread to shut down layout.
    pub fn force_exit(&self, discard_bc: DiscardBrowsingContext) {
        let msg = ConstellationControlMsg::ExitPipeline(self.id, discard_bc);
        if let Err(e) = self.event_loop.send(msg) {
            warn!("Sending script exit message failed ({}).", e);
        }
    }

    /// Notify this pipeline of its activity.
    pub fn set_activity(&self, activity: DocumentActivity) {
        let msg = ConstellationControlMsg::SetDocumentActivity(self.id, activity);
        if let Err(e) = self.event_loop.send(msg) {
            warn!("Sending activity message failed ({}).", e);
        }
    }

    /// The compositor's view of a pipeline.
    pub fn to_sendable(&self) -> CompositionPipeline {
        CompositionPipeline {
            id: self.id,
            top_level_browsing_context_id: self.top_level_browsing_context_id,
            script_chan: self.event_loop.sender(),
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
        let script_msg = ConstellationControlMsg::SetThrottled(self.id, throttled);
        let compositor_msg = CompositorMsg::SetThrottled(self.id, throttled);
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
    top_level_browsing_context_id: TopLevelBrowsingContextId,
    browsing_context_id: BrowsingContextId,
    parent_pipeline_id: Option<PipelineId>,
    opener: Option<BrowsingContextId>,
    namespace_request_sender: IpcSender<PipelineNamespaceRequest>,
    script_to_constellation_chan: ScriptToConstellationChan,
    background_hang_monitor_to_constellation_chan: IpcSender<HangMonitorAlert>,
    bhm_control_port: Option<IpcReceiver<BackgroundHangMonitorControlMsg>>,
    layout_to_constellation_chan: IpcSender<LayoutMsg>,
    scheduler_chan: IpcSender<TimerSchedulerMsg>,
    devtools_ipc_sender: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
    bluetooth_thread: IpcSender<BluetoothRequest>,
    swmanager_thread: IpcSender<SWManagerMsg>,
    font_cache_thread: FontCacheThread,
    resource_threads: ResourceThreads,
    time_profiler_chan: time::ProfilerChan,
    mem_profiler_chan: profile_mem::ProfilerChan,
    window_size: WindowSizeData,
    script_chan: IpcSender<ConstellationControlMsg>,
    load_data: LoadData,
    script_port: IpcReceiver<ConstellationControlMsg>,
    opts: Opts,
    prefs: HashMap<String, PrefValue>,
    pipeline_namespace_id: PipelineNamespaceId,
    webrender_api_sender: webrender_traits::WebRenderScriptApi,
    webrender_image_api_sender: webrender_traits::WebRenderNetApi,
    webrender_document: DocumentId,
    webgl_chan: Option<WebGLPipeline>,
    webxr_registry: webxr_api::Registry,
    player_context: WindowGLContext,
    user_agent: Cow<'static, str>,
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

        let image_cache = Arc::new(ImageCacheImpl::new(self.webrender_image_api_sender.clone()));
        let (content_process_shutdown_chan, content_process_shutdown_port) = unbounded();
        STF::create(
            InitialScriptState {
                id: self.id,
                browsing_context_id: self.browsing_context_id,
                top_level_browsing_context_id: self.top_level_browsing_context_id,
                parent_info: self.parent_pipeline_id,
                opener: self.opener,
                control_chan: self.script_chan.clone(),
                control_port: self.script_port,
                script_to_constellation_chan: self.script_to_constellation_chan.clone(),
                background_hang_monitor_register: background_hang_monitor_register.clone(),
                layout_to_constellation_chan: self.layout_to_constellation_chan.clone(),
                scheduler_chan: self.scheduler_chan,
                bluetooth_thread: self.bluetooth_thread,
                resource_threads: self.resource_threads,
                image_cache: image_cache.clone(),
                time_profiler_chan: self.time_profiler_chan.clone(),
                mem_profiler_chan: self.mem_profiler_chan.clone(),
                devtools_chan: self.devtools_ipc_sender,
                window_size: self.window_size,
                pipeline_namespace_id: self.pipeline_namespace_id,
                content_process_shutdown_chan,
                webgl_chan: self.webgl_chan,
                webxr_registry: self.webxr_registry,
                webrender_document: self.webrender_document,
                webrender_api_sender: self.webrender_api_sender.clone(),
                player_context: self.player_context.clone(),
                inherited_secure_context: self.load_data.inherited_secure_context,
            },
            layout_factory,
            self.font_cache_thread.clone(),
            self.load_data.clone(),
            self.user_agent,
        );

        if wait_for_completion {
            match content_process_shutdown_port.recv() {
                Ok(()) => {},
                Err(_) => error!("Script-thread shut-down unexpectedly"),
            }
        }
    }

    pub fn spawn_multiprocess(self) -> Result<(), Error> {
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

    pub fn prefs(&self) -> HashMap<String, PrefValue> {
        self.prefs.clone()
    }
}
