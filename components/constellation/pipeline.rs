/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::event_loop::EventLoop;
use background_hang_monitor::HangMonitorRegister;
use bluetooth_traits::BluetoothRequest;
use canvas_traits::webgl::WebGLPipeline;
use compositing::compositor_thread::Msg as CompositorMsg;
use compositing::CompositionPipeline;
use compositing::CompositorProxy;
use crossbeam_channel::Sender;
use devtools_traits::{DevtoolsControlMsg, ScriptToDevtoolsControlMsg};
use euclid::{TypedScale, TypedSize2D};
use gfx::font_cache_thread::FontCacheThread;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use ipc_channel::Error;
use layout_traits::LayoutThreadFactory;
use metrics::PaintTimeMetrics;
use msg::constellation_msg::TopLevelBrowsingContextId;
use msg::constellation_msg::{BackgroundHangMonitorRegister, HangMonitorAlert, SamplerControlMsg};
use msg::constellation_msg::{BrowsingContextId, HistoryStateId, PipelineId, PipelineNamespaceId};
use net::image_cache::ImageCacheImpl;
use net_traits::image_cache::ImageCache;
use net_traits::{IpcSend, ResourceThreads};
use profile_traits::mem as profile_mem;
use profile_traits::time;
use script_traits::{ConstellationControlMsg, DiscardBrowsingContext, ScriptToConstellationChan};
use script_traits::{DocumentActivity, InitialScriptState};
use script_traits::{LayoutControlMsg, LayoutMsg, LoadData};
use script_traits::{NewLayoutInfo, SWManagerMsg, SWManagerSenders};
use script_traits::{ScriptThreadFactory, TimerSchedulerMsg, WindowSizeData};
use servo_config::opts::{self, Opts};
use servo_config::{prefs, prefs::PrefValue};
use servo_url::ServoUrl;
use std::collections::{HashMap, HashSet};
#[cfg(not(windows))]
use std::env;
use std::ffi::OsStr;
use std::process;
use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use style_traits::CSSPixel;
use style_traits::DevicePixel;
use webvr_traits::WebVRMsg;

/// A `Pipeline` is the constellation's view of a `Document`. Each pipeline has an
/// event loop (executed by a script thread) and a layout thread. A script thread
/// may be responsible for many pipelines, but a layout thread is only responsible
/// for one.
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

    /// A channel to layout, for performing reflows and shutdown.
    pub layout_chan: IpcSender<LayoutControlMsg>,

    /// A channel to the compositor.
    pub compositor_proxy: CompositorProxy,

    /// The most recently loaded URL in this pipeline.
    /// Note that this URL can change, for example if the page navigates
    /// to a hash URL.
    pub url: ServoUrl,

    /// Whether this pipeline is currently running animations. Pipelines that are running
    /// animations cause composites to be continually scheduled.
    pub running_animations: bool,

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

    /// A handle to register components for hang monitoring.
    /// None when in multiprocess mode.
    pub background_monitor_register: Option<Box<BackgroundHangMonitorRegister>>,

    /// A channel for the background hang monitor to send messages to the constellation.
    pub background_hang_monitor_to_constellation_chan: IpcSender<HangMonitorAlert>,

    /// A channel for the layout thread to send messages to the constellation.
    pub layout_to_constellation_chan: IpcSender<LayoutMsg>,

    /// A channel to schedule timer events.
    pub scheduler_chan: IpcSender<TimerSchedulerMsg>,

    /// A channel to the compositor.
    pub compositor_proxy: CompositorProxy,

    /// A channel to the developer tools, if applicable.
    pub devtools_chan: Option<Sender<DevtoolsControlMsg>>,

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
    pub window_size: TypedSize2D<f32, CSSPixel>,

    /// Information about the device pixel ratio.
    pub device_pixel_ratio: TypedScale<f32, CSSPixel, DevicePixel>,

    /// The event loop to run in, if applicable.
    pub event_loop: Option<Rc<EventLoop>>,

    /// Information about the page to load.
    pub load_data: LoadData,

    /// The ID of the pipeline namespace for this script thread.
    pub pipeline_namespace_id: PipelineNamespaceId,

    /// Whether the browsing context in which pipeline is embedded is visible
    /// for the purposes of scheduling and resource management. This field is
    /// only used to notify script and compositor threads after spawning
    /// a pipeline.
    pub prev_visibility: bool,

    /// Webrender api.
    pub webrender_api_sender: webrender_api::RenderApiSender,

    /// The ID of the document processed by this script thread.
    pub webrender_document: webrender_api::DocumentId,

    /// A channel to the WebGL thread.
    pub webgl_chan: Option<WebGLPipeline>,

    /// A channel to the webvr thread.
    pub webvr_chan: Option<IpcSender<WebVRMsg>>,
}

pub struct NewPipeline {
    pub pipeline: Pipeline,
    pub sampler_control_chan: Option<IpcSender<SamplerControlMsg>>,
}

impl Pipeline {
    /// Starts a layout thread, and possibly a script thread, in
    /// a new process if requested.
    pub fn spawn<Message, LTF, STF>(state: InitialPipelineState) -> Result<NewPipeline, Error>
    where
        LTF: LayoutThreadFactory<Message = Message>,
        STF: ScriptThreadFactory<Message = Message>,
    {
        // Note: we allow channel creation to panic, since recovering from this
        // probably requires a general low-memory strategy.
        let (pipeline_chan, pipeline_port) = ipc::channel().expect("Pipeline main chan");

        let (layout_content_process_shutdown_chan, layout_content_process_shutdown_port) =
            ipc::channel().expect("Pipeline layout content shutdown chan");

        let window_size = WindowSizeData {
            initial_viewport: state.window_size,
            device_pixel_ratio: state.device_pixel_ratio,
        };

        let url = state.load_data.url.clone();

        let (script_chan, sampler_chan) = match state.event_loop {
            Some(script_chan) => {
                let new_layout_info = NewLayoutInfo {
                    parent_info: state.parent_pipeline_id,
                    new_pipeline_id: state.id,
                    browsing_context_id: state.browsing_context_id,
                    top_level_browsing_context_id: state.top_level_browsing_context_id,
                    opener: state.opener,
                    load_data: state.load_data.clone(),
                    window_size: window_size,
                    pipeline_port: pipeline_port,
                    content_process_shutdown_chan: Some(layout_content_process_shutdown_chan),
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
                let script_to_devtools_chan = state.devtools_chan.as_ref().map(|devtools_chan| {
                    let (script_to_devtools_chan, script_to_devtools_port) =
                        ipc::channel().expect("Pipeline script to devtools chan");
                    let devtools_chan = (*devtools_chan).clone();
                    ROUTER.add_route(
                        script_to_devtools_port.to_opaque(),
                        Box::new(
                            move |message| match message.to::<ScriptToDevtoolsControlMsg>() {
                                Err(e) => {
                                    error!("Cast to ScriptToDevtoolsControlMsg failed ({}).", e)
                                },
                                Ok(message) => {
                                    if let Err(e) =
                                        devtools_chan.send(DevtoolsControlMsg::FromScript(message))
                                    {
                                        warn!("Sending to devtools failed ({:?})", e)
                                    }
                                },
                            },
                        ),
                    );
                    script_to_devtools_chan
                });

                let (script_content_process_shutdown_chan, script_content_process_shutdown_port) =
                    ipc::channel().expect("Pipeline script content process shutdown chan");

                let mut unprivileged_pipeline_content = UnprivilegedPipelineContent {
                    id: state.id,
                    browsing_context_id: state.browsing_context_id,
                    top_level_browsing_context_id: state.top_level_browsing_context_id,
                    parent_pipeline_id: state.parent_pipeline_id,
                    opener: state.opener,
                    script_to_constellation_chan: state.script_to_constellation_chan.clone(),
                    background_hang_monitor_to_constellation_chan: state
                        .background_hang_monitor_to_constellation_chan
                        .clone(),
                    sampling_profiler_port: None,
                    scheduler_chan: state.scheduler_chan,
                    devtools_chan: script_to_devtools_chan,
                    bluetooth_thread: state.bluetooth_thread,
                    swmanager_thread: state.swmanager_thread,
                    font_cache_thread: state.font_cache_thread,
                    resource_threads: state.resource_threads,
                    time_profiler_chan: state.time_profiler_chan,
                    mem_profiler_chan: state.mem_profiler_chan,
                    window_size: window_size,
                    layout_to_constellation_chan: state.layout_to_constellation_chan,
                    script_chan: script_chan.clone(),
                    load_data: state.load_data.clone(),
                    script_port: script_port,
                    opts: (*opts::get()).clone(),
                    prefs: prefs::pref_map().iter().collect(),
                    pipeline_port: pipeline_port,
                    pipeline_namespace_id: state.pipeline_namespace_id,
                    layout_content_process_shutdown_chan: layout_content_process_shutdown_chan,
                    layout_content_process_shutdown_port: layout_content_process_shutdown_port,
                    script_content_process_shutdown_chan: script_content_process_shutdown_chan,
                    script_content_process_shutdown_port: script_content_process_shutdown_port,
                    webrender_api_sender: state.webrender_api_sender,
                    webrender_document: state.webrender_document,
                    webgl_chan: state.webgl_chan,
                    webvr_chan: state.webvr_chan,
                };

                // Spawn the child process.
                //
                // Yes, that's all there is to it!
                let sampler_chan = if opts::multiprocess() {
                    let (sampler_chan, sampler_port) = ipc::channel().expect("Sampler chan");
                    unprivileged_pipeline_content.sampling_profiler_port = Some(sampler_port);
                    let _ = unprivileged_pipeline_content.spawn_multiprocess()?;
                    Some(sampler_chan)
                } else {
                    // Should not be None in single-process mode.
                    let register = state
                        .background_monitor_register
                        .expect("Couldn't start content, no background monitor has been initiated");
                    unprivileged_pipeline_content.start_all::<Message, LTF, STF>(false, register);
                    None
                };

                (EventLoop::new(script_chan), sampler_chan)
            },
        };

        let pipeline = Pipeline::new(
            state.id,
            state.browsing_context_id,
            state.top_level_browsing_context_id,
            state.opener,
            script_chan,
            pipeline_chan,
            state.compositor_proxy,
            url,
            state.prev_visibility,
            state.load_data,
        );
        Ok(NewPipeline {
            pipeline,
            sampler_control_chan: sampler_chan,
        })
    }

    /// Creates a new `Pipeline`, after the script and layout threads have been
    /// spawned.
    pub fn new(
        id: PipelineId,
        browsing_context_id: BrowsingContextId,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
        opener: Option<BrowsingContextId>,
        event_loop: Rc<EventLoop>,
        layout_chan: IpcSender<LayoutControlMsg>,
        compositor_proxy: CompositorProxy,
        url: ServoUrl,
        is_visible: bool,
        load_data: LoadData,
    ) -> Pipeline {
        let pipeline = Pipeline {
            id: id,
            browsing_context_id: browsing_context_id,
            top_level_browsing_context_id: top_level_browsing_context_id,
            opener: opener,
            event_loop: event_loop,
            layout_chan: layout_chan,
            compositor_proxy: compositor_proxy,
            url: url,
            children: vec![],
            running_animations: false,
            load_data: load_data,
            history_state_id: None,
            history_states: HashSet::new(),
            completely_loaded: false,
        };

        pipeline.notify_visibility(is_visible);

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
                warn!("Sending exit message failed ({}).", e);
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
        if let Err(e) = self.layout_chan.send(LayoutControlMsg::ExitNow) {
            warn!("Sending layout exit message failed ({}).", e);
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
            id: self.id.clone(),
            top_level_browsing_context_id: self.top_level_browsing_context_id.clone(),
            script_chan: self.event_loop.sender(),
            layout_chan: self.layout_chan.clone(),
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
                return warn!(
                    "Pipeline remove child already removed ({:?}).",
                    browsing_context_id
                );
            },
            Some(index) => self.children.remove(index),
        };
    }

    /// Notify the script thread that this pipeline is visible.
    pub fn notify_visibility(&self, is_visible: bool) {
        let script_msg = ConstellationControlMsg::ChangeFrameVisibilityStatus(self.id, is_visible);
        let compositor_msg = CompositorMsg::PipelineVisibilityChanged(self.id, is_visible);
        let err = self.event_loop.send(script_msg);
        if let Err(e) = err {
            warn!("Sending visibility change failed ({}).", e);
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
    script_to_constellation_chan: ScriptToConstellationChan,
    background_hang_monitor_to_constellation_chan: IpcSender<HangMonitorAlert>,
    sampling_profiler_port: Option<IpcReceiver<SamplerControlMsg>>,
    layout_to_constellation_chan: IpcSender<LayoutMsg>,
    scheduler_chan: IpcSender<TimerSchedulerMsg>,
    devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
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
    pipeline_port: IpcReceiver<LayoutControlMsg>,
    pipeline_namespace_id: PipelineNamespaceId,
    layout_content_process_shutdown_chan: IpcSender<()>,
    layout_content_process_shutdown_port: IpcReceiver<()>,
    script_content_process_shutdown_chan: IpcSender<()>,
    script_content_process_shutdown_port: IpcReceiver<()>,
    webrender_api_sender: webrender_api::RenderApiSender,
    webrender_document: webrender_api::DocumentId,
    webgl_chan: Option<WebGLPipeline>,
    webvr_chan: Option<IpcSender<WebVRMsg>>,
}

impl UnprivilegedPipelineContent {
    pub fn start_all<Message, LTF, STF>(
        self,
        wait_for_completion: bool,
        background_hang_monitor_register: Box<BackgroundHangMonitorRegister>,
    ) where
        LTF: LayoutThreadFactory<Message = Message>,
        STF: ScriptThreadFactory<Message = Message>,
    {
        let image_cache = Arc::new(ImageCacheImpl::new(self.webrender_api_sender.create_api()));
        let paint_time_metrics = PaintTimeMetrics::new(
            self.id,
            self.time_profiler_chan.clone(),
            self.layout_to_constellation_chan.clone(),
            self.script_chan.clone(),
            self.load_data.url.clone(),
        );
        let layout_thread_busy_flag = Arc::new(AtomicBool::new(false));
        let layout_pair = STF::create(
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
                devtools_chan: self.devtools_chan,
                window_size: self.window_size,
                pipeline_namespace_id: self.pipeline_namespace_id,
                content_process_shutdown_chan: self.script_content_process_shutdown_chan,
                webgl_chan: self.webgl_chan,
                webvr_chan: self.webvr_chan,
                webrender_document: self.webrender_document,
                webrender_api_sender: self.webrender_api_sender.clone(),
                layout_is_busy: layout_thread_busy_flag.clone(),
            },
            self.load_data.clone(),
        );

        LTF::create(
            self.id,
            self.top_level_browsing_context_id,
            self.load_data.url,
            self.parent_pipeline_id.is_some(),
            layout_pair,
            self.pipeline_port,
            background_hang_monitor_register,
            self.layout_to_constellation_chan,
            self.script_chan,
            image_cache,
            self.font_cache_thread,
            self.time_profiler_chan,
            self.mem_profiler_chan,
            Some(self.layout_content_process_shutdown_chan),
            self.webrender_api_sender,
            self.webrender_document,
            paint_time_metrics,
            layout_thread_busy_flag.clone(),
            self.opts.load_webfonts_synchronously,
            self.opts.initial_window_size,
            self.opts.device_pixels_per_px,
            self.opts.dump_display_list,
            self.opts.dump_display_list_json,
            self.opts.dump_style_tree,
            self.opts.dump_rule_tree,
            self.opts.relayout_event,
            self.opts.nonincremental_layout,
            self.opts.trace_layout,
            self.opts.dump_flow_tree,
        );

        if wait_for_completion {
            let _ = self.script_content_process_shutdown_port.recv();
            let _ = self.layout_content_process_shutdown_port.recv();
        }
    }

    #[cfg(any(
        target_os = "android",
        target_arch = "arm",
        all(target_arch = "aarch64", not(target_os = "windows"))
    ))]
    pub fn spawn_multiprocess(self) -> Result<(), Error> {
        use ipc_channel::ipc::IpcOneShotServer;
        // Note that this function can panic, due to process creation,
        // avoiding this panic would require a mechanism for dealing
        // with low-resource scenarios.
        let (server, token) = IpcOneShotServer::<IpcSender<UnprivilegedPipelineContent>>::new()
            .expect("Failed to create IPC one-shot server.");

        let path_to_self = env::current_exe().expect("Failed to get current executor.");
        let mut child_process = process::Command::new(path_to_self);
        self.setup_common(&mut child_process, token);
        let _ = child_process
            .spawn()
            .expect("Failed to start unsandboxed child process!");

        let (_receiver, sender) = server.accept().expect("Server failed to accept.");
        sender.send(self)?;

        Ok(())
    }

    #[cfg(all(
        not(target_os = "windows"),
        not(target_os = "ios"),
        not(target_os = "android"),
        not(target_arch = "arm"),
        not(target_arch = "aarch64")
    ))]
    pub fn spawn_multiprocess(self) -> Result<(), Error> {
        use crate::sandboxing::content_process_sandbox_profile;
        use gaol::sandbox::{self, Sandbox, SandboxMethods};
        use ipc_channel::ipc::IpcOneShotServer;

        impl CommandMethods for sandbox::Command {
            fn arg<T>(&mut self, arg: T)
            where
                T: AsRef<OsStr>,
            {
                self.arg(arg);
            }

            fn env<T, U>(&mut self, key: T, val: U)
            where
                T: AsRef<OsStr>,
                U: AsRef<OsStr>,
            {
                self.env(key, val);
            }
        }

        // Note that this function can panic, due to process creation,
        // avoiding this panic would require a mechanism for dealing
        // with low-resource scenarios.
        let (server, token) = IpcOneShotServer::<IpcSender<UnprivilegedPipelineContent>>::new()
            .expect("Failed to create IPC one-shot server.");

        // If there is a sandbox, use the `gaol` API to create the child process.
        if self.opts.sandbox {
            let mut command = sandbox::Command::me().expect("Failed to get current sandbox.");
            self.setup_common(&mut command, token);

            let profile = content_process_sandbox_profile();
            let _ = Sandbox::new(profile)
                .start(&mut command)
                .expect("Failed to start sandboxed child process!");
        } else {
            let path_to_self = env::current_exe().expect("Failed to get current executor.");
            let mut child_process = process::Command::new(path_to_self);
            self.setup_common(&mut child_process, token);
            let _ = child_process
                .spawn()
                .expect("Failed to start unsandboxed child process!");
        }

        let (_receiver, sender) = server.accept().expect("Server failed to accept.");
        sender.send(self)?;

        Ok(())
    }

    #[cfg(any(target_os = "windows", target_os = "ios"))]
    pub fn spawn_multiprocess(self) -> Result<(), Error> {
        error!("Multiprocess is not supported on Windows or iOS.");
        process::exit(1);
    }

    #[cfg(not(windows))]
    fn setup_common<C: CommandMethods>(&self, command: &mut C, token: String) {
        C::arg(command, "--content-process");
        C::arg(command, token);

        if let Ok(value) = env::var("RUST_BACKTRACE") {
            C::env(command, "RUST_BACKTRACE", value);
        }

        if let Ok(value) = env::var("RUST_LOG") {
            C::env(command, "RUST_LOG", value);
        }
    }

    pub fn register_with_background_hang_monitor(&mut self) -> Box<BackgroundHangMonitorRegister> {
        HangMonitorRegister::init(
            self.background_hang_monitor_to_constellation_chan.clone(),
            self.sampling_profiler_port
                .take()
                .expect("no sampling profiler?"),
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

    pub fn swmanager_senders(&self) -> SWManagerSenders {
        SWManagerSenders {
            swmanager_sender: self.swmanager_thread.clone(),
            resource_sender: self.resource_threads.sender(),
        }
    }
}

/// A trait to unify commands launched as multiprocess with or without a sandbox.
trait CommandMethods {
    /// A command line argument.
    fn arg<T>(&mut self, arg: T)
    where
        T: AsRef<OsStr>;

    /// An environment variable.
    fn env<T, U>(&mut self, key: T, val: U)
    where
        T: AsRef<OsStr>,
        U: AsRef<OsStr>;
}

impl CommandMethods for process::Command {
    fn arg<T>(&mut self, arg: T)
    where
        T: AsRef<OsStr>,
    {
        self.arg(arg);
    }

    fn env<T, U>(&mut self, key: T, val: U)
    where
        T: AsRef<OsStr>,
        U: AsRef<OsStr>,
    {
        self.env(key, val);
    }
}
