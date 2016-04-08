/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use CompositorProxy;
use compositor_thread;
use compositor_thread::Msg as CompositorMsg;
use devtools_traits::{DevtoolsControlMsg, ScriptToDevtoolsControlMsg};
use euclid::scale_factor::ScaleFactor;
use euclid::size::TypedSize2D;
use gfx::font_cache_thread::FontCacheThread;
use gfx::paint_thread::{ChromeToPaintMsg, LayoutToPaintMsg, PaintThread};
use gfx_traits::PaintMsg;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use layers::geometry::DevicePixel;
use layout_traits::{LayoutControlChan, LayoutThreadFactory};
use msg::constellation_msg::{ConstellationChan, Failure, FrameId, PipelineId, SubpageId};
use msg::constellation_msg::{LoadData, WindowSizeData};
use msg::constellation_msg::{PipelineNamespaceId};
use net_traits::ResourceThread;
use net_traits::image_cache_thread::ImageCacheThread;
use net_traits::storage_thread::StorageThread;
use profile_traits::mem as profile_mem;
use profile_traits::time;
use script_traits::{ConstellationControlMsg, InitialScriptState, MozBrowserEvent};
use script_traits::{LayoutControlMsg, LayoutMsg, NewLayoutInfo, ScriptMsg};
use script_traits::{ScriptToCompositorMsg, ScriptThreadFactory, TimerEventRequest};
use std::mem;
use std::sync::mpsc::{Receiver, Sender, channel};
use url::Url;
use util;
use util::geometry::{PagePx, ViewportPx};
use util::ipc::OptionalIpcSender;
use util::opts::{self, Opts};
use util::prefs;
use util::thread;
use webrender_traits;

/// A uniquely-identifiable pipeline of script thread, layout thread, and paint thread.
pub struct Pipeline {
    pub id: PipelineId,
    pub parent_info: Option<(PipelineId, SubpageId)>,
    pub script_chan: IpcSender<ConstellationControlMsg>,
    /// A channel to layout, for performing reflows and shutdown.
    pub layout_chan: LayoutControlChan,
    /// A channel to the compositor.
    pub compositor_proxy: Box<CompositorProxy + 'static + Send>,
    pub chrome_to_paint_chan: Sender<ChromeToPaintMsg>,
    pub layout_shutdown_port: IpcReceiver<()>,
    pub paint_shutdown_port: IpcReceiver<()>,
    /// URL corresponding to the most recently-loaded page.
    pub url: Url,
    /// The title of the most recently-loaded page.
    pub title: Option<String>,
    pub size: Option<TypedSize2D<PagePx, f32>>,
    /// Whether this pipeline is currently running animations. Pipelines that are running
    /// animations cause composites to be continually scheduled.
    pub running_animations: bool,
    pub children: Vec<FrameId>,
    pub is_private: bool,
}

/// The subset of the pipeline that is needed for layer composition.
#[derive(Clone)]
pub struct CompositionPipeline {
    pub id: PipelineId,
    pub script_chan: IpcSender<ConstellationControlMsg>,
    pub layout_chan: LayoutControlChan,
    pub chrome_to_paint_chan: Sender<ChromeToPaintMsg>,
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
    pub parent_info: Option<(PipelineId, SubpageId)>,
    /// A channel to the associated constellation.
    pub constellation_chan: ConstellationChan<ScriptMsg>,
    /// A channel for the layout thread to send messages to the constellation.
    pub layout_to_constellation_chan: ConstellationChan<LayoutMsg>,
    /// A channel to the associated paint thread.
    pub painter_chan: ConstellationChan<PaintMsg>,
    /// A channel to schedule timer events.
    pub scheduler_chan: IpcSender<TimerEventRequest>,
    /// A channel to the compositor.
    pub compositor_proxy: Box<CompositorProxy + 'static + Send>,
    /// A channel to the developer tools, if applicable.
    pub devtools_chan: Option<Sender<DevtoolsControlMsg>>,
    /// A channel to the image cache thread.
    pub image_cache_thread: ImageCacheThread,
    /// A channel to the font cache thread.
    pub font_cache_thread: FontCacheThread,
    /// A channel to the resource thread.
    pub resource_thread: ResourceThread,
    /// A channel to the storage thread.
    pub storage_thread: StorageThread,
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
    /// Optional webrender api (if enabled).
    pub webrender_api_sender: Option<webrender_traits::RenderApiSender>,
}

impl Pipeline {
    /// Starts a paint thread, layout thread, and possibly a script thread.
    /// Returns the channels wrapped in a struct.
    pub fn create<LTF, STF>(state: InitialPipelineState)
                            -> (Pipeline, UnprivilegedPipelineContent, PrivilegedPipelineContent)
        where LTF: LayoutThreadFactory, STF: ScriptThreadFactory {
        // Note: we allow channel creation to panic, since recovering from this
        // probably requires a general low-memory strategy.
        let (layout_to_paint_chan, layout_to_paint_port) = util::ipc::optional_ipc_channel();
        let (chrome_to_paint_chan, chrome_to_paint_port) = channel();
        let (paint_shutdown_chan, paint_shutdown_port) = ipc::channel()
            .expect("Pipeline paint shutdown chan");
        let (layout_shutdown_chan, layout_shutdown_port) = ipc::channel()
            .expect("Pipeline layout shutdown chan");
        let (pipeline_chan, pipeline_port) = ipc::channel()
            .expect("Pipeline main chan");;
        let (script_to_compositor_chan, script_to_compositor_port) = ipc::channel()
            .expect("Pipeline script to compositor chan");
        let mut pipeline_port = Some(pipeline_port);

        let failure = Failure {
            pipeline_id: state.id,
            parent_info: state.parent_info,
        };

        let window_size = state.window_size.map(|size| {
            WindowSizeData {
                visible_viewport: size,
                initial_viewport: size * ScaleFactor::new(1.0),
                device_pixel_ratio: state.device_pixel_ratio,
            }
        });

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

        let (layout_content_process_shutdown_chan, layout_content_process_shutdown_port) =
            ipc::channel().expect("Pipeline layout content shutdown chan");

        let (script_chan, script_port) = match state.script_chan {
            Some(script_chan) => {
                let (containing_pipeline_id, subpage_id) =
                    state.parent_info.expect("script_pipeline != None but subpage_id == None");
                let new_layout_info = NewLayoutInfo {
                    containing_pipeline_id: containing_pipeline_id,
                    new_pipeline_id: state.id,
                    subpage_id: subpage_id,
                    load_data: state.load_data.clone(),
                    paint_chan: layout_to_paint_chan.clone().to_opaque(),
                    failure: failure,
                    pipeline_port: mem::replace(&mut pipeline_port, None)
                        .expect("script_pipeline != None but pipeline_port == None"),
                    layout_shutdown_chan: layout_shutdown_chan.clone(),
                    content_process_shutdown_chan: layout_content_process_shutdown_chan.clone(),
                };

                if let Err(e) = script_chan.send(ConstellationControlMsg::AttachLayout(new_layout_info)) {
                    warn!("Sending to script during pipeline creation failed ({})", e);
                }
                (script_chan, None)
            }
            None => {
                let (script_chan, script_port) = ipc::channel().expect("Pipeline script chan");
                (script_chan, Some(script_port))
            }
        };

        let (script_content_process_shutdown_chan, script_content_process_shutdown_port) =
            ipc::channel().expect("Pipeline script content process shutdown chan");

        let pipeline = Pipeline::new(state.id,
                                     state.parent_info,
                                     script_chan.clone(),
                                     LayoutControlChan(pipeline_chan),
                                     state.compositor_proxy.clone_compositor_proxy(),
                                     chrome_to_paint_chan.clone(),
                                     layout_shutdown_port,
                                     paint_shutdown_port,
                                     state.load_data.url.clone(),
                                     state.window_size);

        let unprivileged_pipeline_content = UnprivilegedPipelineContent {
            id: state.id,
            parent_info: state.parent_info,
            constellation_chan: state.constellation_chan,
            scheduler_chan: state.scheduler_chan,
            devtools_chan: script_to_devtools_chan,
            image_cache_thread: state.image_cache_thread,
            font_cache_thread: state.font_cache_thread.clone(),
            resource_thread: state.resource_thread,
            storage_thread: state.storage_thread,
            time_profiler_chan: state.time_profiler_chan.clone(),
            mem_profiler_chan: state.mem_profiler_chan.clone(),
            window_size: window_size,
            layout_to_constellation_chan: state.layout_to_constellation_chan,
            script_chan: script_chan,
            load_data: state.load_data.clone(),
            failure: failure,
            script_port: script_port,
            opts: (*opts::get()).clone(),
            layout_to_paint_chan: layout_to_paint_chan,
            pipeline_port: pipeline_port,
            layout_shutdown_chan: layout_shutdown_chan,
            paint_shutdown_chan: paint_shutdown_chan.clone(),
            script_to_compositor_chan: script_to_compositor_chan,
            pipeline_namespace_id: state.pipeline_namespace_id,
            layout_content_process_shutdown_chan: layout_content_process_shutdown_chan,
            layout_content_process_shutdown_port: layout_content_process_shutdown_port,
            script_content_process_shutdown_chan: script_content_process_shutdown_chan,
            script_content_process_shutdown_port: script_content_process_shutdown_port,
            webrender_api_sender: state.webrender_api_sender,
        };

        let privileged_pipeline_content = PrivilegedPipelineContent {
            id: state.id,
            painter_chan: state.painter_chan,
            compositor_proxy: state.compositor_proxy,
            font_cache_thread: state.font_cache_thread,
            time_profiler_chan: state.time_profiler_chan,
            mem_profiler_chan: state.mem_profiler_chan,
            load_data: state.load_data,
            failure: failure,
            layout_to_paint_port: layout_to_paint_port,
            chrome_to_paint_chan: chrome_to_paint_chan,
            chrome_to_paint_port: chrome_to_paint_port,
            paint_shutdown_chan: paint_shutdown_chan,
            script_to_compositor_port: script_to_compositor_port,
        };

        (pipeline, unprivileged_pipeline_content, privileged_pipeline_content)
    }

    pub fn new(id: PipelineId,
               parent_info: Option<(PipelineId, SubpageId)>,
               script_chan: IpcSender<ConstellationControlMsg>,
               layout_chan: LayoutControlChan,
               compositor_proxy: Box<CompositorProxy + 'static + Send>,
               chrome_to_paint_chan: Sender<ChromeToPaintMsg>,
               layout_shutdown_port: IpcReceiver<()>,
               paint_shutdown_port: IpcReceiver<()>,
               url: Url,
               size: Option<TypedSize2D<PagePx, f32>>)
               -> Pipeline {
        Pipeline {
            id: id,
            parent_info: parent_info,
            script_chan: script_chan,
            layout_chan: layout_chan,
            compositor_proxy: compositor_proxy,
            chrome_to_paint_chan: chrome_to_paint_chan,
            layout_shutdown_port: layout_shutdown_port,
            paint_shutdown_port: paint_shutdown_port,
            url: url,
            title: None,
            children: vec!(),
            size: size,
            running_animations: false,
            is_private: false,
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
        if let Ok((sender, receiver)) = ipc::channel() {
            self.compositor_proxy.send(CompositorMsg::PipelineExited(self.id, sender));
            if let Err(e) = receiver.recv() {
                warn!("Sending exit message failed ({}).", e);
            }
        }

        // Script thread handles shutting down layout, and layout handles shutting down the painter.
        // For now, if the script thread has failed, we give up on clean shutdown.
        if self.script_chan
               .send(ConstellationControlMsg::ExitPipeline(self.id))
               .is_ok() {
            // Wait until all slave threads have terminated and run destructors
            // NOTE: We don't wait for script thread as we don't always own it
            let _ = self.paint_shutdown_port.recv();
            let _ = self.layout_shutdown_port.recv();
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
        let LayoutControlChan(ref layout_channel) = self.layout_chan;
        if let Err(e) = layout_channel.send(LayoutControlMsg::ExitNow) {
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
                                     subpage_id: SubpageId,
                                     event: MozBrowserEvent) {
        assert!(prefs::get_pref("dom.mozbrowser.enabled").as_boolean().unwrap_or(false));

        let event = ConstellationControlMsg::MozBrowserEvent(self.id,
                                                             subpage_id,
                                                             event);
        if let Err(e) = self.script_chan.send(event) {
            warn!("Sending mozbrowser event to script failed ({}).", e);
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct UnprivilegedPipelineContent {
    id: PipelineId,
    parent_info: Option<(PipelineId, SubpageId)>,
    constellation_chan: ConstellationChan<ScriptMsg>,
    layout_to_constellation_chan: ConstellationChan<LayoutMsg>,
    scheduler_chan: IpcSender<TimerEventRequest>,
    devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
    script_to_compositor_chan: IpcSender<ScriptToCompositorMsg>,
    image_cache_thread: ImageCacheThread,
    font_cache_thread: FontCacheThread,
    resource_thread: ResourceThread,
    storage_thread: StorageThread,
    time_profiler_chan: time::ProfilerChan,
    mem_profiler_chan: profile_mem::ProfilerChan,
    window_size: Option<WindowSizeData>,
    script_chan: IpcSender<ConstellationControlMsg>,
    load_data: LoadData,
    failure: Failure,
    script_port: Option<IpcReceiver<ConstellationControlMsg>>,
    layout_to_paint_chan: OptionalIpcSender<LayoutToPaintMsg>,
    opts: Opts,
    paint_shutdown_chan: IpcSender<()>,
    pipeline_port: Option<IpcReceiver<LayoutControlMsg>>,
    pipeline_namespace_id: PipelineNamespaceId,
    layout_shutdown_chan: IpcSender<()>,
    layout_content_process_shutdown_chan: IpcSender<()>,
    layout_content_process_shutdown_port: IpcReceiver<()>,
    script_content_process_shutdown_chan: IpcSender<()>,
    script_content_process_shutdown_port: IpcReceiver<()>,
    webrender_api_sender: Option<webrender_traits::RenderApiSender>,
}

impl UnprivilegedPipelineContent {
    pub fn start_all<LTF, STF>(mut self, wait_for_completion: bool)
                               where LTF: LayoutThreadFactory, STF: ScriptThreadFactory {
        let layout_pair = ScriptThreadFactory::create_layout_channel(None::<&mut STF>);

        ScriptThreadFactory::create(None::<&mut STF>, InitialScriptState {
            id: self.id,
            parent_info: self.parent_info,
            compositor: self.script_to_compositor_chan,
            control_chan: self.script_chan.clone(),
            control_port: mem::replace(&mut self.script_port, None).expect("No script port."),
            constellation_chan: self.constellation_chan.clone(),
            layout_to_constellation_chan: self.layout_to_constellation_chan.clone(),
            scheduler_chan: self.scheduler_chan.clone(),
            failure_info: self.failure.clone(),
            resource_thread: self.resource_thread,
            storage_thread: self.storage_thread.clone(),
            image_cache_thread: self.image_cache_thread.clone(),
            time_profiler_chan: self.time_profiler_chan.clone(),
            mem_profiler_chan: self.mem_profiler_chan.clone(),
            devtools_chan: self.devtools_chan,
            window_size: self.window_size,
            pipeline_namespace_id: self.pipeline_namespace_id,
            content_process_shutdown_chan: self.script_content_process_shutdown_chan.clone(),
        }, &layout_pair, self.load_data.clone());

        LayoutThreadFactory::create(None::<&mut LTF>,
                                  self.id,
                                  self.load_data.url.clone(),
                                  self.parent_info.is_some(),
                                  layout_pair,
                                  self.pipeline_port.expect("No pipeline port."),
                                  self.layout_to_constellation_chan,
                                  self.failure,
                                  self.script_chan.clone(),
                                  self.layout_to_paint_chan.clone(),
                                  self.image_cache_thread,
                                  self.font_cache_thread,
                                  self.time_profiler_chan,
                                  self.mem_profiler_chan,
                                  self.layout_shutdown_chan,
                                  self.layout_content_process_shutdown_chan.clone(),
                                  self.webrender_api_sender);

        if wait_for_completion {
            let _ = self.script_content_process_shutdown_port.recv();
            let _ = self.layout_content_process_shutdown_port.recv();
        }
    }

    pub fn opts(&self) -> Opts {
        self.opts.clone()
    }
}

pub struct PrivilegedPipelineContent {
    id: PipelineId,
    painter_chan: ConstellationChan<PaintMsg>,
    compositor_proxy: Box<CompositorProxy + Send + 'static>,
    script_to_compositor_port: IpcReceiver<ScriptToCompositorMsg>,
    font_cache_thread: FontCacheThread,
    time_profiler_chan: time::ProfilerChan,
    mem_profiler_chan: profile_mem::ProfilerChan,
    load_data: LoadData,
    failure: Failure,
    layout_to_paint_port: Receiver<LayoutToPaintMsg>,
    chrome_to_paint_chan: Sender<ChromeToPaintMsg>,
    chrome_to_paint_port: Receiver<ChromeToPaintMsg>,
    paint_shutdown_chan: IpcSender<()>,
}

impl PrivilegedPipelineContent {
    pub fn start_all(self) {
        PaintThread::create(self.id,
                          self.load_data.url,
                          self.chrome_to_paint_chan,
                          self.layout_to_paint_port,
                          self.chrome_to_paint_port,
                          self.compositor_proxy.clone_compositor_proxy(),
                          self.painter_chan,
                          self.font_cache_thread,
                          self.failure,
                          self.time_profiler_chan,
                          self.mem_profiler_chan,
                          self.paint_shutdown_chan);

        let compositor_proxy_for_script_listener_thread =
            self.compositor_proxy.clone_compositor_proxy();
        let script_to_compositor_port = self.script_to_compositor_port;
        thread::spawn_named("CompositorScriptListener".to_owned(), move || {
            compositor_thread::run_script_listener_thread(
                compositor_proxy_for_script_listener_thread,
                script_to_compositor_port)
        });
    }

    pub fn start_paint_thread(self) {
        PaintThread::create(self.id,
                          self.load_data.url,
                          self.chrome_to_paint_chan,
                          self.layout_to_paint_port,
                          self.chrome_to_paint_port,
                          self.compositor_proxy,
                          self.painter_chan,
                          self.font_cache_thread,
                          self.failure,
                          self.time_profiler_chan,
                          self.mem_profiler_chan,
                          self.paint_shutdown_chan);

    }
}
