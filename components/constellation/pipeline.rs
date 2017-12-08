/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bluetooth_traits::BluetoothRequest;
use canvas_traits::webgl::WebGLPipeline;
use compositing::CompositionPipeline;
use compositing::CompositorProxy;
use compositing::compositor_thread::Msg as CompositorMsg;
use devtools_traits::{DevtoolsControlMsg, ScriptToDevtoolsControlMsg};
use euclid::{TypedSize2D, TypedScale};
use event_loop::EventLoop;
use gfx::font_cache_thread::FontCacheThread;
use ipc_channel::Error;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use layout_traits::LayoutThreadFactory;
use metrics::PaintTimeMetrics;
use msg::constellation_msg::{BrowsingContextId, TopLevelBrowsingContextId, FrameType, PipelineId, PipelineNamespaceId};
use net::image_cache::ImageCacheImpl;
use net_traits::{IpcSend, ResourceThreads};
use net_traits::image_cache::ImageCache;
use profile_traits::mem as profile_mem;
use profile_traits::time;
use script_traits::{ConstellationControlMsg, DiscardBrowsingContext, ScriptToConstellationChan};
use script_traits::{DocumentActivity, InitialScriptState};
use script_traits::{LayoutControlMsg, LayoutMsg, LoadData, MozBrowserEvent};
use script_traits::{NewLayoutInfo, SWManagerMsg, SWManagerSenders};
use script_traits::{ScriptThreadFactory, TimerSchedulerMsg, WindowSizeData};
use servo_config::opts::{self, Opts};
use servo_config::prefs::{PREFS, Pref};
use servo_url::ServoUrl;
use std::collections::HashMap;
#[cfg(not(windows))]
use std::env;
use std::ffi::OsStr;
use std::process;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use style_traits::CSSPixel;
use style_traits::DevicePixel;
use webrender_api;
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

    /// The parent pipeline of this one. `None` if this is a root pipeline.
    /// Note that because of mozbrowser iframes, even top-level pipelines
    /// may have a parent (in which case the frame type will be
    /// `MozbrowserIFrame`).
    /// TODO: move this field to `BrowsingContext`.
    pub parent_info: Option<(PipelineId, FrameType)>,

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

    /// Whether this pipeline is in private browsing mode.
    /// TODO: move this field to `BrowsingContext`.
    pub is_private: bool,

    /// Whether this pipeline should be treated as visible for the purposes of scheduling and
    /// resource management.
    pub visible: bool,
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
    pub parent_info: Option<(PipelineId, FrameType)>,

    /// A channel to the associated constellation.
    pub script_to_constellation_chan: ScriptToConstellationChan,

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
    pub window_size: Option<TypedSize2D<f32, CSSPixel>>,

    /// Information about the device pixel ratio.
    pub device_pixel_ratio: TypedScale<f32, CSSPixel, DevicePixel>,

    /// The event loop to run in, if applicable.
    pub event_loop: Option<Rc<EventLoop>>,

    /// Information about the page to load.
    pub load_data: LoadData,


    /// The ID of the pipeline namespace for this script thread.
    pub pipeline_namespace_id: PipelineNamespaceId,

    /// Pipeline visibility to be inherited
    pub prev_visibility: Option<bool>,

    /// Webrender api.
    pub webrender_api_sender: webrender_api::RenderApiSender,

    /// The ID of the document processed by this script thread.
    pub webrender_document: webrender_api::DocumentId,

    /// Whether this pipeline is considered private.
    pub is_private: bool,

    /// A channel to the webgl thread.
    pub webgl_chan: WebGLPipeline,

    /// A channel to the webvr thread.
    pub webvr_chan: Option<IpcSender<WebVRMsg>>,
}

impl Pipeline {
    /// Starts a layout thread, and possibly a script thread, in
    /// a new process if requested.
    pub fn spawn<Message, LTF, STF>(state: InitialPipelineState) -> Result<Pipeline, Error>
        where LTF: LayoutThreadFactory<Message=Message>,
              STF: ScriptThreadFactory<Message=Message>
    {
        // Note: we allow channel creation to panic, since recovering from this
        // probably requires a general low-memory strategy.
        let (pipeline_chan, pipeline_port) = ipc::channel()
            .expect("Pipeline main chan");

        let (layout_content_process_shutdown_chan, layout_content_process_shutdown_port) =
            ipc::channel().expect("Pipeline layout content shutdown chan");

        let device_pixel_ratio = state.device_pixel_ratio;
        let window_size = state.window_size.map(|size| {
            WindowSizeData {
                initial_viewport: size,
                device_pixel_ratio: device_pixel_ratio,
            }
        });

        let url = state.load_data.url.clone();

        let script_chan = match state.event_loop {
            Some(script_chan) => {
                let new_layout_info = NewLayoutInfo {
                    parent_info: state.parent_info,
                    new_pipeline_id: state.id,
                    browsing_context_id: state.browsing_context_id,
                    top_level_browsing_context_id: state.top_level_browsing_context_id,
                    load_data: state.load_data,
                    window_size: window_size,
                    pipeline_port: pipeline_port,
                    content_process_shutdown_chan: Some(layout_content_process_shutdown_chan.clone()),
                    layout_threads: PREFS.get("layout.threads").as_u64().expect("count") as usize,
                };

                if let Err(e) = script_chan.send(ConstellationControlMsg::AttachLayout(new_layout_info)) {
                    warn!("Sending to script during pipeline creation failed ({})", e);
                }
                script_chan
            }
            None => {
                let (script_chan, script_port) = ipc::channel().expect("Pipeline script chan");

                // Route messages coming from content to devtools as appropriate.
                let script_to_devtools_chan = state.devtools_chan.as_ref().map(|devtools_chan| {
                    let (script_to_devtools_chan, script_to_devtools_port) = ipc::channel()
                        .expect("Pipeline script to devtools chan");
                    let devtools_chan = (*devtools_chan).clone();
                    ROUTER.add_route(script_to_devtools_port.to_opaque(), Box::new(move |message| {
                        match message.to::<ScriptToDevtoolsControlMsg>() {
                            Err(e) => error!("Cast to ScriptToDevtoolsControlMsg failed ({}).", e),
                            Ok(message) => if let Err(e) = devtools_chan.send(DevtoolsControlMsg::FromScript(message)) {
                                warn!("Sending to devtools failed ({})", e)
                            },
                        }
                    }));
                    script_to_devtools_chan
                });

                let (script_content_process_shutdown_chan, script_content_process_shutdown_port) =
                    ipc::channel().expect("Pipeline script content process shutdown chan");

                let unprivileged_pipeline_content = UnprivilegedPipelineContent {
                    id: state.id,
                    browsing_context_id: state.browsing_context_id,
                    top_level_browsing_context_id: state.top_level_browsing_context_id,
                    parent_info: state.parent_info,
                    script_to_constellation_chan: state.script_to_constellation_chan.clone(),
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
                    load_data: state.load_data,
                    script_port: script_port,
                    opts: (*opts::get()).clone(),
                    prefs: PREFS.cloned(),
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
                if opts::multiprocess() {
                    let _ = unprivileged_pipeline_content.spawn_multiprocess()?;
                } else {
                    unprivileged_pipeline_content.start_all::<Message, LTF, STF>(false);
                }

                EventLoop::new(script_chan)
            }
        };

        Ok(Pipeline::new(state.id,
                         state.browsing_context_id,
                         state.top_level_browsing_context_id,
                         state.parent_info,
                         script_chan,
                         pipeline_chan,
                         state.compositor_proxy,
                         state.is_private,
                         url,
                         state.prev_visibility.unwrap_or(true)))
    }

    /// Creates a new `Pipeline`, after the script and layout threads have been
    /// spawned.
    pub fn new(id: PipelineId,
               browsing_context_id: BrowsingContextId,
               top_level_browsing_context_id: TopLevelBrowsingContextId,
               parent_info: Option<(PipelineId, FrameType)>,
               event_loop: Rc<EventLoop>,
               layout_chan: IpcSender<LayoutControlMsg>,
               compositor_proxy: CompositorProxy,
               is_private: bool,
               url: ServoUrl,
               visible: bool)
               -> Pipeline {
        let pipeline = Pipeline {
            id: id,
            browsing_context_id: browsing_context_id,
            top_level_browsing_context_id: top_level_browsing_context_id,
            parent_info: parent_info,
            event_loop: event_loop,
            layout_chan: layout_chan,
            compositor_proxy: compositor_proxy,
            url: url,
            children: vec!(),
            running_animations: false,
            visible: visible,
            is_private: is_private,
        };

        pipeline.notify_visibility();

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
            self.compositor_proxy.send(CompositorMsg::PipelineExited(self.id, sender));
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
        match self.children.iter().position(|id| *id == browsing_context_id) {
            None => return warn!("Pipeline remove child already removed ({:?}).", browsing_context_id),
            Some(index) => self.children.remove(index),
        };
    }

    /// Send a mozbrowser event to the script thread for this pipeline.
    /// This will cause an event to be fired on an iframe in the document,
    /// or on the `Window` if no frame is given.
    pub fn trigger_mozbrowser_event(&self,
                                     child_id: Option<TopLevelBrowsingContextId>,
                                     event: MozBrowserEvent) {
        assert!(PREFS.is_mozbrowser_enabled());

        let event = ConstellationControlMsg::MozBrowserEvent(self.id,
                                                             child_id,
                                                             event);
        if let Err(e) = self.event_loop.send(event) {
            warn!("Sending mozbrowser event to script failed ({}).", e);
        }
    }

    /// Notify the script thread that this pipeline is visible.
    fn notify_visibility(&self) {
        let script_msg = ConstellationControlMsg::ChangeFrameVisibilityStatus(self.id, self.visible);
        let compositor_msg = CompositorMsg::PipelineVisibilityChanged(self.id, self.visible);
        let err = self.event_loop.send(script_msg);
        if let Err(e) = err {
            warn!("Sending visibility change failed ({}).", e);
        }
        self.compositor_proxy.send(compositor_msg);
    }

    /// Change the visibility of this pipeline.
    pub fn change_visibility(&mut self, visible: bool) {
        if visible == self.visible {
            return;
        }
        self.visible = visible;
        self.notify_visibility();
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
    parent_info: Option<(PipelineId, FrameType)>,
    script_to_constellation_chan: ScriptToConstellationChan,
    layout_to_constellation_chan: IpcSender<LayoutMsg>,
    scheduler_chan: IpcSender<TimerSchedulerMsg>,
    devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
    bluetooth_thread: IpcSender<BluetoothRequest>,
    swmanager_thread: IpcSender<SWManagerMsg>,
    font_cache_thread: FontCacheThread,
    resource_threads: ResourceThreads,
    time_profiler_chan: time::ProfilerChan,
    mem_profiler_chan: profile_mem::ProfilerChan,
    window_size: Option<WindowSizeData>,
    script_chan: IpcSender<ConstellationControlMsg>,
    load_data: LoadData,
    script_port: IpcReceiver<ConstellationControlMsg>,
    opts: Opts,
    prefs: HashMap<String, Pref>,
    pipeline_port: IpcReceiver<LayoutControlMsg>,
    pipeline_namespace_id: PipelineNamespaceId,
    layout_content_process_shutdown_chan: IpcSender<()>,
    layout_content_process_shutdown_port: IpcReceiver<()>,
    script_content_process_shutdown_chan: IpcSender<()>,
    script_content_process_shutdown_port: IpcReceiver<()>,
    webrender_api_sender: webrender_api::RenderApiSender,
    webrender_document: webrender_api::DocumentId,
    webgl_chan: WebGLPipeline,
    webvr_chan: Option<IpcSender<WebVRMsg>>,
}

impl UnprivilegedPipelineContent {
    pub fn start_all<Message, LTF, STF>(self, wait_for_completion: bool)
        where LTF: LayoutThreadFactory<Message=Message>,
              STF: ScriptThreadFactory<Message=Message>
    {
        let image_cache = Arc::new(ImageCacheImpl::new(self.webrender_api_sender.create_api()));
        let paint_time_metrics = PaintTimeMetrics::new(self.id,
                                                       self.time_profiler_chan.clone(),
                                                       self.layout_to_constellation_chan.clone(),
                                                       self.script_chan.clone(),
                                                       self.load_data.url.clone());
        let layout_pair = STF::create(InitialScriptState {
            id: self.id,
            browsing_context_id: self.browsing_context_id,
            top_level_browsing_context_id: self.top_level_browsing_context_id,
            parent_info: self.parent_info,
            control_chan: self.script_chan.clone(),
            control_port: self.script_port,
            script_to_constellation_chan: self.script_to_constellation_chan.clone(),
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
        }, self.load_data.clone());

        LTF::create(self.id,
                    self.top_level_browsing_context_id,
                    self.load_data.url,
                    self.parent_info.is_some(),
                    layout_pair,
                    self.pipeline_port,
                    self.layout_to_constellation_chan,
                    self.script_chan,
                    image_cache.clone(),
                    self.font_cache_thread,
                    self.time_profiler_chan,
                    self.mem_profiler_chan,
                    Some(self.layout_content_process_shutdown_chan),
                    self.webrender_api_sender,
                    self.webrender_document,
                    self.prefs.get("layout.threads").expect("exists").value()
                        .as_u64().expect("count") as usize,
                    paint_time_metrics);

        if wait_for_completion {
            let _ = self.script_content_process_shutdown_port.recv();
            let _ = self.layout_content_process_shutdown_port.recv();
        }
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "ios")))]
    pub fn spawn_multiprocess(self) -> Result<(), Error> {
        use gaol::sandbox::{self, Sandbox, SandboxMethods};
        use ipc_channel::ipc::IpcOneShotServer;
        use sandboxing::content_process_sandbox_profile;

        impl CommandMethods for sandbox::Command {
            fn arg<T>(&mut self, arg: T)
                where T: AsRef<OsStr> {
                self.arg(arg);
            }

            fn env<T, U>(&mut self, key: T, val: U)
                where T: AsRef<OsStr>, U: AsRef<OsStr> {
                self.env(key, val);
            }
        }

        // Note that this function can panic, due to process creation,
        // avoiding this panic would require a mechanism for dealing
        // with low-resource scenarios.
        let (server, token) =
            IpcOneShotServer::<IpcSender<UnprivilegedPipelineContent>>::new()
            .expect("Failed to create IPC one-shot server.");

        // If there is a sandbox, use the `gaol` API to create the child process.
        if opts::get().sandbox {
            let mut command = sandbox::Command::me().expect("Failed to get current sandbox.");
            self.setup_common(&mut command, token);

            let profile = content_process_sandbox_profile();
            let _ = Sandbox::new(profile)
                .start(&mut command)
                .expect("Failed to start sandboxed child process!");
        } else {
            let path_to_self = env::current_exe()
                .expect("Failed to get current executor.");
            let mut child_process = process::Command::new(path_to_self);
            self.setup_common(&mut child_process, token);
            let _ = child_process.spawn().expect("Failed to start unsandboxed child process!");
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

    pub fn script_to_constellation_chan(&self) -> &ScriptToConstellationChan {
        &self.script_to_constellation_chan
    }

    pub fn opts(&self) -> Opts {
        self.opts.clone()
    }

    pub fn prefs(&self) -> HashMap<String, Pref> {
        self.prefs.clone()
    }

    pub fn swmanager_senders(&self) -> SWManagerSenders {
        SWManagerSenders {
            swmanager_sender: self.swmanager_thread.clone(),
            resource_sender: self.resource_threads.sender()
        }
    }
}

/// A trait to unify commands launched as multiprocess with or without a sandbox.
trait CommandMethods {
    /// A command line argument.
    fn arg<T>(&mut self, arg: T)
        where T: AsRef<OsStr>;

    /// An environment variable.
    fn env<T, U>(&mut self, key: T, val: U)
        where T: AsRef<OsStr>, U: AsRef<OsStr>;
}

impl CommandMethods for process::Command {
    fn arg<T>(&mut self, arg: T)
        where T: AsRef<OsStr> {
        self.arg(arg);
    }

    fn env<T, U>(&mut self, key: T, val: U)
        where T: AsRef<OsStr>, U: AsRef<OsStr> {
        self.env(key, val);
    }
}
