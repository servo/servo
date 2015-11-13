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
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use layers::geometry::DevicePixel;
use layout_traits::{LayoutControlChan, LayoutThreadFactory};
use msg::constellation_msg::{ConstellationChan, Failure, FrameId, PipelineId, SubpageId};
use msg::constellation_msg::{LoadData, MozBrowserEvent, WindowSizeData};
use msg::constellation_msg::{PipelineNamespaceId};
use net_traits::ResourceThread;
use net_traits::image_cache_thread::ImageCacheThread;
use net_traits::storage_thread::StorageThread;
use profile_traits::mem as profile_mem;
use profile_traits::time;
use script_traits::{ConstellationControlMsg, InitialScriptState};
use script_traits::{LayoutControlMsg, NewLayoutInfo, ScriptThreadFactory};
use script_traits::{TimerEventRequest};
use std::any::Any;
use std::mem;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use url::Url;
use util;
use util::geometry::{PagePx, ViewportPx};
use util::ipc::OptionalIpcSender;
use util::prefs;

/// A uniquely-identifiable pipeline of script thread, layout thread, and paint thread.
pub struct Pipeline {
    pub id: PipelineId,
    pub parent_info: Option<(PipelineId, SubpageId)>,
    pub script_chan: Sender<ConstellationControlMsg>,
    /// A channel to layout, for performing reflows and shutdown.
    pub layout_chan: LayoutControlChan,
    /// A channel to the compositor.
    pub compositor_proxy: Box<CompositorProxy + 'static + Send>,
    pub chrome_to_paint_chan: Sender<ChromeToPaintMsg>,
    pub layout_shutdown_port: Receiver<()>,
    pub paint_shutdown_port: Receiver<()>,
    /// URL corresponding to the most recently-loaded page.
    pub url: Url,
    /// The title of the most recently-loaded page.
    pub title: Option<String>,
    pub size: Option<TypedSize2D<PagePx, f32>>,
    /// Whether this pipeline is currently running animations. Pipelines that are running
    /// animations cause composites to be continually scheduled.
    pub running_animations: bool,
    pub children: Vec<FrameId>,
}

/// The subset of the pipeline that is needed for layer composition.
#[derive(Clone)]
pub struct CompositionPipeline {
    pub id: PipelineId,
    pub script_chan: Sender<ConstellationControlMsg>,
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
    pub constellation_chan: ConstellationChan,
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
    pub script_chan: Option<Sender<ConstellationControlMsg>>,
    /// Information about the page to load.
    pub load_data: LoadData,
    /// The ID of the pipeline namespace for this script thread.
    pub pipeline_namespace_id: PipelineNamespaceId,
}

impl Pipeline {
    /// Starts a paint thread, layout thread, and possibly a script thread.
    /// Returns the channels wrapped in a struct.
    pub fn create<LTF, STF>(state: InitialPipelineState)
                            -> (Pipeline, PipelineContent)
                            where LTF: LayoutThreadFactory, STF: ScriptThreadFactory {
        let (layout_to_paint_chan, layout_to_paint_port) = util::ipc::optional_ipc_channel();
        let (chrome_to_paint_chan, chrome_to_paint_port) = channel();
        let (paint_shutdown_chan, paint_shutdown_port) = channel();
        let (layout_shutdown_chan, layout_shutdown_port) = channel();
        let (pipeline_chan, pipeline_port) = ipc::channel().unwrap();
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
            let (script_to_devtools_chan, script_to_devtools_port) = ipc::channel().unwrap();
            let devtools_chan = (*devtools_chan).clone();
            ROUTER.add_route(script_to_devtools_port.to_opaque(), box move |message| {
                let message: ScriptToDevtoolsControlMsg = message.to().unwrap();
                devtools_chan.send(DevtoolsControlMsg::FromScript(message)).unwrap()
            });
            script_to_devtools_chan
        });

        let (script_chan, script_port) = match state.script_chan {
            Some(script_chan) => {
                let (containing_pipeline_id, subpage_id) =
                    state.parent_info.expect("script_pipeline != None but subpage_id == None");
                let new_layout_info = NewLayoutInfo {
                    containing_pipeline_id: containing_pipeline_id,
                    new_pipeline_id: state.id,
                    subpage_id: subpage_id,
                    load_data: state.load_data.clone(),
                    paint_chan: box layout_to_paint_chan.clone() as Box<Any + Send>,
                    failure: failure,
                    pipeline_port: mem::replace(&mut pipeline_port, None).unwrap(),
                    layout_shutdown_chan: layout_shutdown_chan.clone(),
                };

                script_chan.send(ConstellationControlMsg::AttachLayout(new_layout_info))
                           .unwrap();
                (script_chan, None)
            }
            None => {
                let (script_chan, script_port) = channel();
                (script_chan, Some(script_port))
            }
        };

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

        let pipeline_content = PipelineContent {
            id: state.id,
            parent_info: state.parent_info,
            constellation_chan: state.constellation_chan,
            scheduler_chan: state.scheduler_chan,
            compositor_proxy: state.compositor_proxy,
            devtools_chan: script_to_devtools_chan,
            image_cache_thread: state.image_cache_thread,
            font_cache_thread: state.font_cache_thread,
            resource_thread: state.resource_thread,
            storage_thread: state.storage_thread,
            time_profiler_chan: state.time_profiler_chan,
            mem_profiler_chan: state.mem_profiler_chan,
            window_size: window_size,
            script_chan: script_chan,
            load_data: state.load_data,
            failure: failure,
            script_port: script_port,
            layout_to_paint_chan: layout_to_paint_chan,
            chrome_to_paint_chan: chrome_to_paint_chan,
            layout_to_paint_port: Some(layout_to_paint_port),
            chrome_to_paint_port: Some(chrome_to_paint_port),
            pipeline_port: pipeline_port,
            paint_shutdown_chan: paint_shutdown_chan,
            layout_shutdown_chan: layout_shutdown_chan,
            pipeline_namespace_id: state.pipeline_namespace_id,
        };

        (pipeline, pipeline_content)
    }

    pub fn new(id: PipelineId,
               parent_info: Option<(PipelineId, SubpageId)>,
               script_chan: Sender<ConstellationControlMsg>,
               layout_chan: LayoutControlChan,
               compositor_proxy: Box<CompositorProxy + 'static + Send>,
               chrome_to_paint_chan: Sender<ChromeToPaintMsg>,
               layout_shutdown_port: Receiver<()>,
               paint_shutdown_port: Receiver<()>,
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

        // The compositor wants to know when pipelines shut down too.
        self.compositor_proxy.send(CompositorMsg::PipelineExited(self.id))
    }

    pub fn freeze(&self) {
        let _ = self.script_chan.send(ConstellationControlMsg::Freeze(self.id)).unwrap();
    }

    pub fn thaw(&self) {
        let _ = self.script_chan.send(ConstellationControlMsg::Thaw(self.id)).unwrap();
    }

    pub fn force_exit(&self) {
        let _ = self.script_chan.send(ConstellationControlMsg::ExitPipeline(self.id)).unwrap();
        let _ = self.chrome_to_paint_chan.send(ChromeToPaintMsg::Exit);
        let LayoutControlChan(ref layout_channel) = self.layout_chan;
        let _ = layout_channel.send(LayoutControlMsg::ExitNow).unwrap();
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
        let index = self.children.iter().position(|id| *id == frame_id).unwrap();
        self.children.remove(index);
    }

    pub fn trigger_mozbrowser_event(&self,
                                     subpage_id: SubpageId,
                                     event: MozBrowserEvent) {
        assert!(prefs::get_pref("dom.mozbrowser.enabled").as_boolean().unwrap_or(false));

        let event = ConstellationControlMsg::MozBrowserEvent(self.id,
                                                             subpage_id,
                                                             event);
        self.script_chan.send(event).unwrap();
    }
}

pub struct PipelineContent {
    id: PipelineId,
    parent_info: Option<(PipelineId, SubpageId)>,
    constellation_chan: ConstellationChan,
    scheduler_chan: IpcSender<TimerEventRequest>,
    compositor_proxy: Box<CompositorProxy + Send + 'static>,
    devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
    image_cache_thread: ImageCacheThread,
    font_cache_thread: FontCacheThread,
    resource_thread: ResourceThread,
    storage_thread: StorageThread,
    time_profiler_chan: time::ProfilerChan,
    mem_profiler_chan: profile_mem::ProfilerChan,
    window_size: Option<WindowSizeData>,
    script_chan: Sender<ConstellationControlMsg>,
    load_data: LoadData,
    failure: Failure,
    script_port: Option<Receiver<ConstellationControlMsg>>,
    layout_to_paint_chan: OptionalIpcSender<LayoutToPaintMsg>,
    chrome_to_paint_chan: Sender<ChromeToPaintMsg>,
    layout_to_paint_port: Option<Receiver<LayoutToPaintMsg>>,
    chrome_to_paint_port: Option<Receiver<ChromeToPaintMsg>>,
    paint_shutdown_chan: Sender<()>,
    pipeline_port: Option<IpcReceiver<LayoutControlMsg>>,
    layout_shutdown_chan: Sender<()>,
    pipeline_namespace_id: PipelineNamespaceId,
}

impl PipelineContent {
    pub fn start_all<LTF, STF>(mut self) where LTF: LayoutThreadFactory, STF: ScriptThreadFactory {
        let layout_pair = ScriptThreadFactory::create_layout_channel(None::<&mut STF>);
        let (script_to_compositor_chan, script_to_compositor_port) = ipc::channel().unwrap();

        self.start_paint_thread();

        let compositor_proxy_for_script_listener_thread =
            self.compositor_proxy.clone_compositor_proxy();
        thread::spawn(move || {
            compositor_thread::run_script_listener_thread(
                compositor_proxy_for_script_listener_thread,
                script_to_compositor_port)
        });

        ScriptThreadFactory::create(None::<&mut STF>, InitialScriptState {
            id: self.id,
            parent_info: self.parent_info,
            compositor: script_to_compositor_chan,
            control_chan: self.script_chan.clone(),
            control_port: mem::replace(&mut self.script_port, None).unwrap(),
            constellation_chan: self.constellation_chan.clone(),
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
        }, &layout_pair, self.load_data.clone());

        LayoutThreadFactory::create(None::<&mut LTF>,
                                  self.id,
                                  self.load_data.url.clone(),
                                  self.parent_info.is_some(),
                                  layout_pair,
                                  self.pipeline_port.unwrap(),
                                  self.constellation_chan,
                                  self.failure,
                                  self.script_chan.clone(),
                                  self.layout_to_paint_chan.clone(),
                                  self.image_cache_thread,
                                  self.font_cache_thread,
                                  self.time_profiler_chan,
                                  self.mem_profiler_chan,
                                  self.layout_shutdown_chan);
    }

    pub fn start_paint_thread(&mut self) {
        PaintThread::create(self.id,
                          self.load_data.url.clone(),
                          self.chrome_to_paint_chan.clone(),
                          mem::replace(&mut self.layout_to_paint_port, None).unwrap(),
                          mem::replace(&mut self.chrome_to_paint_port, None).unwrap(),
                          self.compositor_proxy.clone_compositor_proxy(),
                          self.constellation_chan.clone(),
                          self.font_cache_thread.clone(),
                          self.failure.clone(),
                          self.time_profiler_chan.clone(),
                          self.mem_profiler_chan.clone(),
                          self.paint_shutdown_chan.clone());

    }
}
