/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositing::CompositionPipeline;
use compositing::CompositorProxy;
use compositing::compositor_thread::Msg as CompositorMsg;
use devtools_traits::{DevtoolsControlMsg, ScriptToDevtoolsControlMsg};
use euclid::scale_factor::ScaleFactor;
use euclid::size::TypedSize2D;
#[cfg(not(target_os = "windows"))]
use gaol;
use gfx::font_cache_thread::FontCacheThread;
use gfx::paint_thread::{LayoutToPaintMsg, PaintThread};
use gfx_traits::ChromeToPaintMsg;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use layers::geometry::DevicePixel;
use layout_traits::LayoutThreadFactory;
use msg::constellation_msg::{FrameId, FrameType, LoadData, PipelineId};
use msg::constellation_msg::{PipelineNamespaceId, SubpageId};
use net_traits::bluetooth_thread::BluetoothMethodMsg;
use net_traits::image_cache_thread::ImageCacheThread;
use net_traits::{ResourceThreads, IpcSend};
use profile_traits::mem as profile_mem;
use profile_traits::time;
use script_traits::{ConstellationControlMsg, InitialScriptState, MozBrowserEvent};
use script_traits::{LayoutControlMsg, LayoutMsg, NewLayoutInfo, ScriptMsg, SWManagerMsg, SWManagerSenders};
use script_traits::{ScriptThreadFactory, TimerEventRequest, WindowSizeData};
use std::collections::HashMap;
use std::io::Error as IOError;
use std::process;
use std::sync::mpsc::{Sender, channel};
use style_traits::{PagePx, ViewportPx};
use url::Url;
use util;
use util::ipc::OptionalIpcSender;
use util::opts::{self, Opts};
use util::prefs::{PREFS, Pref};
use webrender_traits;

pub enum ChildProcess {
    #[cfg(not(target_os = "windows"))]
    Sandboxed(gaol::platform::process::Process),
    #[cfg(not(target_os = "windows"))]
    Unsandboxed(process::Child),
}

/// A uniquely-identifiable pipeline of script thread, layout thread, and paint thread.
pub struct Pipeline {
    pub id: PipelineId,
    pub parent_info: Option<(PipelineId, SubpageId, FrameType)>,
    pub script_chan: IpcSender<ConstellationControlMsg>,
    /// A channel to layout, for performing reflows and shutdown.
    pub layout_chan: IpcSender<LayoutControlMsg>,
    /// A channel to the compositor.
    pub compositor_proxy: Box<CompositorProxy + 'static + Send>,
    pub chrome_to_paint_chan: Sender<ChromeToPaintMsg>,
    /// URL corresponding to the most recently-loaded page.
    pub url: Url,
    /// The title of the most recently-loaded page.
    pub title: Option<String>,
    pub size: Option<TypedSize2D<PagePx, f32>>,
    /// Whether this pipeline is currently running animations. Pipelines that are running
    /// animations cause composites to be continually scheduled.
    pub running_animations: bool,
    pub children: Vec<FrameId>,
    /// Whether this pipeline is considered distinct from public pipelines.
    pub is_private: bool,
    /// Whether this pipeline should be treated as visible for the purposes of scheduling and
    /// resource management.
    pub visible: bool,
    /// Frame that contains this Pipeline. Can be `None` if the pipeline is not apart of the
    /// frame tree.
    pub frame: Option<FrameId>,
}

/// Initial setup data needed to construct a pipeline.
///
/// *DO NOT* add any Senders to this unless you absolutely know what you're doing, or pcwalton will
/// have to rewrite your code. Use IPC senders instead.
pub struct InitialPipelineState {
    /// The ID of the pipeline to create.
    pub id: PipelineId,
    /// The subpage ID of this pipeline to create in its pipeline parent.
    /// If `None`, this is the root.
    pub parent_info: Option<(PipelineId, SubpageId, FrameType)>,
    /// A channel to the associated constellation.
    pub constellation_chan: IpcSender<ScriptMsg>,
    /// A channel for the layout thread to send messages to the constellation.
    pub layout_to_constellation_chan: IpcSender<LayoutMsg>,
    /// A channel to schedule timer events.
    pub scheduler_chan: IpcSender<TimerEventRequest>,
    /// A channel to the compositor.
    pub compositor_proxy: Box<CompositorProxy + 'static + Send>,
    /// A channel to the developer tools, if applicable.
    pub devtools_chan: Option<Sender<DevtoolsControlMsg>>,
    /// A channel to the bluetooth thread.
    pub bluetooth_thread: IpcSender<BluetoothMethodMsg>,
    /// A channel to the service worker manager thread
    pub swmanager_thread: IpcSender<SWManagerMsg>,
    /// A channel to the image cache thread.
    pub image_cache_thread: ImageCacheThread,
    /// A channel to the font cache thread.
    pub font_cache_thread: FontCacheThread,
    /// Channels to the resource-related threads.
    pub resource_threads: ResourceThreads,
    /// A channel to the time profiler thread.
    pub time_profiler_chan: time::ProfilerChan,
    /// A channel to the memory profiler thread.
    pub mem_profiler_chan: profile_mem::ProfilerChan,
    /// Information about the initial window size.
    pub window_size: Option<TypedSize2D<PagePx, f32>>,
    /// Information about the device pixel ratio.
    pub device_pixel_ratio: ScaleFactor<ViewportPx, DevicePixel, f32>,
    /// A channel to the script thread, if applicable. If this is `Some`,
    /// then `parent_info` must also be `Some`.
    pub script_chan: Option<IpcSender<ConstellationControlMsg>>,
    /// Information about the page to load.
    pub load_data: LoadData,
    /// The ID of the pipeline namespace for this script thread.
    pub pipeline_namespace_id: PipelineNamespaceId,
    /// Pipeline visibility is inherited from parent
    pub parent_visibility: Option<bool>,
    /// Optional webrender api (if enabled).
    pub webrender_api_sender: Option<webrender_traits::RenderApiSender>,
    /// Whether this pipeline is considered private.
    pub is_private: bool,
}

impl Pipeline {
    /// Starts a paint thread, layout thread, and possibly a script thread, in
    /// a new process if requested.
    pub fn spawn<Message, LTF, STF>(state: InitialPipelineState)
                                    -> Result<(Pipeline, Option<ChildProcess>), IOError>
        where LTF: LayoutThreadFactory<Message=Message>,
              STF: ScriptThreadFactory<Message=Message>
    {
        // Note: we allow channel creation to panic, since recovering from this
        // probably requires a general low-memory strategy.
        let (layout_to_paint_chan, layout_to_paint_port) = util::ipc::optional_ipc_channel();
        let (chrome_to_paint_chan, chrome_to_paint_port) = channel();
        let (pipeline_chan, pipeline_port) = ipc::channel()
            .expect("Pipeline main chan");;

        let (layout_content_process_shutdown_chan, layout_content_process_shutdown_port) =
            ipc::channel().expect("Pipeline layout content shutdown chan");

        let (script_chan, content_ports) = match state.script_chan {
            Some(script_chan) => {
                let (containing_pipeline_id, subpage_id, frame_type) =
                    state.parent_info.expect("script_pipeline != None but subpage_id == None");
                let new_layout_info = NewLayoutInfo {
                    containing_pipeline_id: containing_pipeline_id,
                    new_pipeline_id: state.id,
                    subpage_id: subpage_id,
                    frame_type: frame_type,
                    load_data: state.load_data.clone(),
                    paint_chan: layout_to_paint_chan.clone().to_opaque(),
                    pipeline_port: pipeline_port,
                    layout_to_constellation_chan: state.layout_to_constellation_chan.clone(),
                    content_process_shutdown_chan: layout_content_process_shutdown_chan.clone(),
                };

                if let Err(e) = script_chan.send(ConstellationControlMsg::AttachLayout(new_layout_info)) {
                    warn!("Sending to script during pipeline creation failed ({})", e);
                }
                (script_chan, None)
            }
            None => {
                let (script_chan, script_port) = ipc::channel().expect("Pipeline script chan");
                (script_chan, Some((script_port, pipeline_port)))
            }
        };

        PaintThread::create(state.id,
                            state.load_data.url.clone(),
                            chrome_to_paint_chan.clone(),
                            layout_to_paint_port,
                            chrome_to_paint_port,
                            state.compositor_proxy.clone_compositor_proxy(),
                            state.font_cache_thread.clone(),
                            state.time_profiler_chan.clone(),
                            state.mem_profiler_chan.clone());

        let mut child_process = None;
        if let Some((script_port, pipeline_port)) = content_ports {
            // Route messages coming from content to devtools as appropriate.
            let script_to_devtools_chan = state.devtools_chan.as_ref().map(|devtools_chan| {
                let (script_to_devtools_chan, script_to_devtools_port) = ipc::channel()
                    .expect("Pipeline script to devtools chan");
                let devtools_chan = (*devtools_chan).clone();
                ROUTER.add_route(script_to_devtools_port.to_opaque(), box move |message| {
                    match message.to::<ScriptToDevtoolsControlMsg>() {
                        Err(e) => error!("Cast to ScriptToDevtoolsControlMsg failed ({}).", e),
                        Ok(message) => if let Err(e) = devtools_chan.send(DevtoolsControlMsg::FromScript(message)) {
                            warn!("Sending to devtools failed ({})", e)
                        },
                    }
                });
                script_to_devtools_chan
            });

            let device_pixel_ratio = state.device_pixel_ratio;
            let window_size = state.window_size.map(|size| {
                WindowSizeData {
                    visible_viewport: size,
                    initial_viewport: size * ScaleFactor::new(1.0),
                    device_pixel_ratio: device_pixel_ratio,
                }
            });

            let (script_content_process_shutdown_chan, script_content_process_shutdown_port) =
                ipc::channel().expect("Pipeline script content process shutdown chan");

            let unprivileged_pipeline_content = UnprivilegedPipelineContent {
                id: state.id,
                parent_info: state.parent_info,
                constellation_chan: state.constellation_chan,
                scheduler_chan: state.scheduler_chan,
                devtools_chan: script_to_devtools_chan,
                bluetooth_thread: state.bluetooth_thread,
                swmanager_thread: state.swmanager_thread,
                image_cache_thread: state.image_cache_thread,
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
                prefs: PREFS.cloned(),
                layout_to_paint_chan: layout_to_paint_chan,
                pipeline_port: pipeline_port,
                pipeline_namespace_id: state.pipeline_namespace_id,
                layout_content_process_shutdown_chan: layout_content_process_shutdown_chan,
                layout_content_process_shutdown_port: layout_content_process_shutdown_port,
                script_content_process_shutdown_chan: script_content_process_shutdown_chan,
                script_content_process_shutdown_port: script_content_process_shutdown_port,
                webrender_api_sender: state.webrender_api_sender,
            };

            // Spawn the child process.
            //
            // Yes, that's all there is to it!
            if opts::multiprocess() {
                child_process = Some(try!(unprivileged_pipeline_content.spawn_multiprocess()));
            } else {
                unprivileged_pipeline_content.start_all::<Message, LTF, STF>(false);
            }
        }

        let pipeline = Pipeline::new(state.id,
                                     state.parent_info,
                                     script_chan,
                                     pipeline_chan,
                                     state.compositor_proxy,
                                     chrome_to_paint_chan,
                                     state.is_private,
                                     state.load_data.url,
                                     state.window_size,
                                     state.parent_visibility.unwrap_or(true));

        pipeline.notify_visibility();

        Ok((pipeline, child_process))
    }

    fn new(id: PipelineId,
           parent_info: Option<(PipelineId, SubpageId, FrameType)>,
           script_chan: IpcSender<ConstellationControlMsg>,
           layout_chan: IpcSender<LayoutControlMsg>,
           compositor_proxy: Box<CompositorProxy + 'static + Send>,
           chrome_to_paint_chan: Sender<ChromeToPaintMsg>,
           is_private: bool,
           url: Url,
           size: Option<TypedSize2D<PagePx, f32>>,
           visible: bool)
           -> Pipeline {
        Pipeline {
            id: id,
            parent_info: parent_info,
            script_chan: script_chan,
            layout_chan: layout_chan,
            compositor_proxy: compositor_proxy,
            chrome_to_paint_chan: chrome_to_paint_chan,
            url: url,
            title: None,
            children: vec!(),
            size: size,
            running_animations: false,
            visible: visible,
            is_private: is_private,
            frame: None,
        }
    }

    pub fn grant_paint_permission(&self) {
        let _ = self.chrome_to_paint_chan.send(ChromeToPaintMsg::PaintPermissionGranted);
    }

    pub fn revoke_paint_permission(&self) {
        debug!("pipeline revoking paint channel paint permission");
        let _ = self.chrome_to_paint_chan.send(ChromeToPaintMsg::PaintPermissionRevoked);
    }

    pub fn exit(&self) {
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
        if let Err(e) = self.script_chan.send(ConstellationControlMsg::ExitPipeline(self.id)) {
            warn!("Sending script exit message failed ({}).", e);
        }
    }

    pub fn freeze(&self) {
        if let Err(e) = self.script_chan.send(ConstellationControlMsg::Freeze(self.id)) {
            warn!("Sending freeze message failed ({}).", e);
        }
    }

    pub fn thaw(&self) {
        if let Err(e) = self.script_chan.send(ConstellationControlMsg::Thaw(self.id)) {
            warn!("Sending freeze message failed ({}).", e);
        }
    }

    pub fn force_exit(&self) {
        if let Err(e) = self.script_chan.send(ConstellationControlMsg::ExitPipeline(self.id)) {
            warn!("Sending script exit message failed ({}).", e);
        }
        if let Err(e) = self.chrome_to_paint_chan.send(ChromeToPaintMsg::Exit) {
            warn!("Sending paint exit message failed ({}).", e);
        }
        if let Err(e) = self.layout_chan.send(LayoutControlMsg::ExitNow) {
            warn!("Sending layout exit message failed ({}).", e);
        }
    }

    pub fn to_sendable(&self) -> CompositionPipeline {
        CompositionPipeline {
            id: self.id.clone(),
            script_chan: self.script_chan.clone(),
            layout_chan: self.layout_chan.clone(),
            chrome_to_paint_chan: self.chrome_to_paint_chan.clone(),
        }
    }

    pub fn add_child(&mut self, frame_id: FrameId) {
        self.children.push(frame_id);
    }

    pub fn remove_child(&mut self, frame_id: FrameId) {
        match self.children.iter().position(|id| *id == frame_id) {
            None => return warn!("Pipeline remove child already removed ({:?}).", frame_id),
            Some(index) => self.children.remove(index),
        };
    }

    pub fn trigger_mozbrowser_event(&self,
                                     subpage_id: Option<SubpageId>,
                                     event: MozBrowserEvent) {
        assert!(PREFS.is_mozbrowser_enabled());

        let event = ConstellationControlMsg::MozBrowserEvent(self.id,
                                                             subpage_id,
                                                             event);
        if let Err(e) = self.script_chan.send(event) {
            warn!("Sending mozbrowser event to script failed ({}).", e);
        }
    }

    fn notify_visibility(&self) {
        self.script_chan.send(ConstellationControlMsg::ChangeFrameVisibilityStatus(self.id, self.visible))
                        .expect("Pipeline script chan");

        self.compositor_proxy.send(CompositorMsg::PipelineVisibilityChanged(self.id, self.visible));
    }

    pub fn change_visibility(&mut self, visible: bool) {
        if visible == self.visible {
            return;
        }
        self.visible = visible;
        self.notify_visibility();
    }

}

#[derive(Deserialize, Serialize)]
pub struct UnprivilegedPipelineContent {
    id: PipelineId,
    parent_info: Option<(PipelineId, SubpageId, FrameType)>,
    constellation_chan: IpcSender<ScriptMsg>,
    layout_to_constellation_chan: IpcSender<LayoutMsg>,
    scheduler_chan: IpcSender<TimerEventRequest>,
    devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
    bluetooth_thread: IpcSender<BluetoothMethodMsg>,
    swmanager_thread: IpcSender<SWManagerMsg>,
    image_cache_thread: ImageCacheThread,
    font_cache_thread: FontCacheThread,
    resource_threads: ResourceThreads,
    time_profiler_chan: time::ProfilerChan,
    mem_profiler_chan: profile_mem::ProfilerChan,
    window_size: Option<WindowSizeData>,
    script_chan: IpcSender<ConstellationControlMsg>,
    load_data: LoadData,
    script_port: IpcReceiver<ConstellationControlMsg>,
    layout_to_paint_chan: OptionalIpcSender<LayoutToPaintMsg>,
    opts: Opts,
    prefs: HashMap<String, Pref>,
    pipeline_port: IpcReceiver<LayoutControlMsg>,
    pipeline_namespace_id: PipelineNamespaceId,
    layout_content_process_shutdown_chan: IpcSender<()>,
    layout_content_process_shutdown_port: IpcReceiver<()>,
    script_content_process_shutdown_chan: IpcSender<()>,
    script_content_process_shutdown_port: IpcReceiver<()>,
    webrender_api_sender: Option<webrender_traits::RenderApiSender>,
}

impl UnprivilegedPipelineContent {
    pub fn start_all<Message, LTF, STF>(self, wait_for_completion: bool)
        where LTF: LayoutThreadFactory<Message=Message>,
              STF: ScriptThreadFactory<Message=Message>
    {
        let layout_pair = STF::create(InitialScriptState {
            id: self.id,
            parent_info: self.parent_info,
            control_chan: self.script_chan.clone(),
            control_port: self.script_port,
            constellation_chan: self.constellation_chan,
            scheduler_chan: self.scheduler_chan,
            bluetooth_thread: self.bluetooth_thread,
            resource_threads: self.resource_threads,
            image_cache_thread: self.image_cache_thread.clone(),
            time_profiler_chan: self.time_profiler_chan.clone(),
            mem_profiler_chan: self.mem_profiler_chan.clone(),
            devtools_chan: self.devtools_chan,
            window_size: self.window_size,
            pipeline_namespace_id: self.pipeline_namespace_id,
            content_process_shutdown_chan: self.script_content_process_shutdown_chan,
        }, self.load_data.clone());

        LTF::create(self.id,
                    self.load_data.url,
                    self.parent_info.is_some(),
                    layout_pair,
                    self.pipeline_port,
                    self.layout_to_constellation_chan,
                    self.script_chan,
                    self.layout_to_paint_chan,
                    self.image_cache_thread,
                    self.font_cache_thread,
                    self.time_profiler_chan,
                    self.mem_profiler_chan,
                    self.layout_content_process_shutdown_chan,
                    self.webrender_api_sender);

        if wait_for_completion {
            let _ = self.script_content_process_shutdown_port.recv();
            let _ = self.layout_content_process_shutdown_port.recv();
        }
    }

    #[cfg(not(target_os = "windows"))]
    pub fn spawn_multiprocess(self) -> Result<ChildProcess, IOError> {
        use gaol::sandbox::{self, Sandbox, SandboxMethods};
        use ipc_channel::ipc::IpcOneShotServer;
        use sandboxing::content_process_sandbox_profile;
        use std::env;

        // Note that this function can panic, due to process creation,
        // avoiding this panic would require a mechanism for dealing
        // with low-resource scenarios.
        let (server, token) =
            IpcOneShotServer::<IpcSender<UnprivilegedPipelineContent>>::new()
            .expect("Failed to create IPC one-shot server.");

        // If there is a sandbox, use the `gaol` API to create the child process.
        let child_process = if opts::get().sandbox {
            let mut command = sandbox::Command::me().expect("Failed to get current sandbox.");
            command.arg("--content-process").arg(token);

            if let Ok(value) = env::var("RUST_BACKTRACE") {
                command.env("RUST_BACKTRACE", value);
            }

            let profile = content_process_sandbox_profile();
            ChildProcess::Sandboxed(Sandbox::new(profile).start(&mut command)
                                    .expect("Failed to start sandboxed child process!"))
        } else {
            let path_to_self = env::current_exe()
                .expect("Failed to get current executor.");
            let mut child_process = process::Command::new(path_to_self);
            child_process.arg("--content-process");
            child_process.arg(token);

            if let Ok(value) = env::var("RUST_BACKTRACE") {
                child_process.env("RUST_BACKTRACE", value);
            }

            ChildProcess::Unsandboxed(child_process.spawn()
                                      .expect("Failed to start unsandboxed child process!"))
        };

        let (_receiver, sender) = server.accept().expect("Server failed to accept.");
        try!(sender.send(self));

        Ok(child_process)
    }

    #[cfg(target_os = "windows")]
    pub fn spawn_multiprocess(self) -> Result<ChildProcess, IOError> {
        error!("Multiprocess is not supported on Windows.");
        process::exit(1);
    }

    pub fn constellation_chan(&self) -> IpcSender<ScriptMsg> {
        self.constellation_chan.clone()
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
