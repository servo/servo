/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use CompositorProxy;
use layout_traits::{LayoutTaskFactory, LayoutControlChan};
use script_traits::{LayoutControlMsg, ScriptTaskFactory};
use script_traits::{NewLayoutInfo, ConstellationControlMsg};

use compositor_task;
use devtools_traits::{DevtoolsControlMsg, ScriptToDevtoolsControlMsg};
use euclid::rect::{TypedRect};
use euclid::scale_factor::ScaleFactor;
use gfx::paint_task::{ChromeToPaintMsg, LayoutToPaintMsg, PaintTask};
use gfx::font_cache_task::FontCacheTask;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use layers::geometry::DevicePixel;
use msg::constellation_msg::{ConstellationChan, Failure, FrameId, PipelineId, SubpageId};
use msg::constellation_msg::{LoadData, WindowSizeData, PipelineExitType, MozBrowserEvent};
use profile_traits::mem as profile_mem;
use profile_traits::time;
use net_traits::ResourceTask;
use net_traits::image_cache_task::ImageCacheTask;
use net_traits::storage_task::StorageTask;
use std::any::Any;
use std::mem;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use url::Url;
use util;
use util::geometry::{PagePx, ViewportPx};
use util::ipc::OptionalIpcSender;
use util::opts;

/// A uniquely-identifiable pipeline of script task, layout task, and paint task.
pub struct Pipeline {
    pub id: PipelineId,
    pub parent_info: Option<(PipelineId, SubpageId)>,
    pub script_chan: Sender<ConstellationControlMsg>,
    /// A channel to layout, for performing reflows and shutdown.
    pub layout_chan: LayoutControlChan,
    pub chrome_to_paint_chan: Sender<ChromeToPaintMsg>,
    pub layout_shutdown_port: Receiver<()>,
    pub paint_shutdown_port: Receiver<()>,
    /// URL corresponding to the most recently-loaded page.
    pub url: Url,
    /// The title of the most recently-loaded page.
    pub title: Option<String>,
    pub rect: Option<TypedRect<PagePx, f32>>,
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

impl Pipeline {
    /// Starts a paint task, layout task, and possibly a script task.
    /// Returns the channels wrapped in a struct.
    /// If script_pipeline is not None, then subpage_id must also be not None.
    pub fn create<LTF,STF>(id: PipelineId,
                           parent_info: Option<(PipelineId, SubpageId)>,
                           constellation_chan: ConstellationChan,
                           compositor_proxy: Box<CompositorProxy+'static+Send>,
                           devtools_chan: Option<Sender<DevtoolsControlMsg>>,
                           image_cache_task: ImageCacheTask,
                           font_cache_task: FontCacheTask,
                           resource_task: ResourceTask,
                           storage_task: StorageTask,
                           time_profiler_chan: time::ProfilerChan,
                           mem_profiler_chan: profile_mem::ProfilerChan,
                           window_rect: Option<TypedRect<PagePx, f32>>,
                           script_chan: Option<Sender<ConstellationControlMsg>>,
                           load_data: LoadData,
                           device_pixel_ratio: ScaleFactor<ViewportPx, DevicePixel, f32>)
                           -> (Pipeline, PipelineContent)
                           where LTF: LayoutTaskFactory, STF:ScriptTaskFactory {
        let (layout_to_paint_chan, layout_to_paint_port) = util::ipc::optional_ipc_channel();
        let (chrome_to_paint_chan, chrome_to_paint_port) = channel();
        let (paint_shutdown_chan, paint_shutdown_port) = channel();
        let (layout_shutdown_chan, layout_shutdown_port) = channel();
        let (pipeline_chan, pipeline_port) = ipc::channel().unwrap();
        let mut pipeline_port = Some(pipeline_port);

        let failure = Failure {
            pipeline_id: id,
            parent_info: parent_info,
        };

        let window_size = window_rect.map(|rect| {
            WindowSizeData {
                visible_viewport: rect.size,
                initial_viewport: rect.size * ScaleFactor::new(1.0),
                device_pixel_ratio: device_pixel_ratio,
            }
        });

        // Route messages coming from content to devtools as appropriate.
        let script_to_devtools_chan = devtools_chan.as_ref().map(|devtools_chan| {
            let (script_to_devtools_chan, script_to_devtools_port) = ipc::channel().unwrap();
            let devtools_chan = (*devtools_chan).clone();
            ROUTER.add_route(script_to_devtools_port.to_opaque(), box move |message| {
                let message: ScriptToDevtoolsControlMsg = message.to().unwrap();
                devtools_chan.send(DevtoolsControlMsg::FromScript(message)).unwrap()
            });
            script_to_devtools_chan
        });

        let (script_chan, script_port) = match script_chan {
            Some(script_chan) => {
                let (containing_pipeline_id, subpage_id) =
                    parent_info.expect("script_pipeline != None but subpage_id == None");
                let new_layout_info = NewLayoutInfo {
                    containing_pipeline_id: containing_pipeline_id,
                    new_pipeline_id: id,
                    subpage_id: subpage_id,
                    load_data: load_data.clone(),
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

        let pipeline = Pipeline::new(id,
                                     parent_info,
                                     script_chan.clone(),
                                     LayoutControlChan(pipeline_chan),
                                     chrome_to_paint_chan.clone(),
                                     layout_shutdown_port,
                                     paint_shutdown_port,
                                     load_data.url.clone(),
                                     window_rect);

        let pipeline_content = PipelineContent {
            id: id,
            parent_info: parent_info,
            constellation_chan: constellation_chan,
            compositor_proxy: compositor_proxy,
            devtools_chan: script_to_devtools_chan,
            image_cache_task: image_cache_task,
            font_cache_task: font_cache_task,
            resource_task: resource_task,
            storage_task: storage_task,
            time_profiler_chan: time_profiler_chan,
            mem_profiler_chan: mem_profiler_chan,
            window_size: window_size,
            script_chan: script_chan,
            load_data: load_data,
            failure: failure,
            script_port: script_port,
            layout_to_paint_chan: layout_to_paint_chan,
            chrome_to_paint_chan: chrome_to_paint_chan,
            layout_to_paint_port: Some(layout_to_paint_port),
            chrome_to_paint_port: Some(chrome_to_paint_port),
            pipeline_port: pipeline_port,
            paint_shutdown_chan: paint_shutdown_chan,
            layout_shutdown_chan: layout_shutdown_chan,
        };

        (pipeline, pipeline_content)
    }

    pub fn new(id: PipelineId,
               parent_info: Option<(PipelineId, SubpageId)>,
               script_chan: Sender<ConstellationControlMsg>,
               layout_chan: LayoutControlChan,
               chrome_to_paint_chan: Sender<ChromeToPaintMsg>,
               layout_shutdown_port: Receiver<()>,
               paint_shutdown_port: Receiver<()>,
               url: Url,
               rect: Option<TypedRect<PagePx, f32>>)
               -> Pipeline {
        Pipeline {
            id: id,
            parent_info: parent_info,
            script_chan: script_chan,
            layout_chan: layout_chan,
            chrome_to_paint_chan: chrome_to_paint_chan,
            layout_shutdown_port: layout_shutdown_port,
            paint_shutdown_port: paint_shutdown_port,
            url: url,
            title: None,
            children: vec!(),
            rect: rect,
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

    pub fn exit(&self, exit_type: PipelineExitType) {
        debug!("pipeline {:?} exiting", self.id);

        // Script task handles shutting down layout, and layout handles shutting down the painter.
        // For now, if the script task has failed, we give up on clean shutdown.
        if self.script_chan.send(ConstellationControlMsg::ExitPipeline(self.id, exit_type)).is_ok() {
            // Wait until all slave tasks have terminated and run destructors
            // NOTE: We don't wait for script task as we don't always own it
            let _ = self.paint_shutdown_port.recv();
            let _ = self.layout_shutdown_port.recv();
        }

    }

    pub fn freeze(&self) {
        let _ = self.script_chan.send(ConstellationControlMsg::Freeze(self.id)).unwrap();
    }

    pub fn thaw(&self) {
        let _ = self.script_chan.send(ConstellationControlMsg::Thaw(self.id)).unwrap();
    }

    pub fn force_exit(&self) {
        let _ = self.script_chan.send(
            ConstellationControlMsg::ExitPipeline(self.id,
                                                  PipelineExitType::PipelineOnly)).unwrap();
        let _ = self.chrome_to_paint_chan.send(ChromeToPaintMsg::Exit(
                    None,
                    PipelineExitType::PipelineOnly));
        let LayoutControlChan(ref layout_channel) = self.layout_chan;
        let _ = layout_channel.send(
            LayoutControlMsg::ExitNow(PipelineExitType::PipelineOnly)).unwrap();
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
        assert!(opts::experimental_enabled());

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
    compositor_proxy: Box<CompositorProxy + Send + 'static>,
    devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
    image_cache_task: ImageCacheTask,
    font_cache_task: FontCacheTask,
    resource_task: ResourceTask,
    storage_task: StorageTask,
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
}

impl PipelineContent {
    pub fn start_all<LTF,STF>(mut self) where LTF: LayoutTaskFactory, STF: ScriptTaskFactory {
        let layout_pair = ScriptTaskFactory::create_layout_channel(None::<&mut STF>);
        let (script_to_compositor_chan, script_to_compositor_port) = ipc::channel().unwrap();

        self.start_paint_task();

        let compositor_proxy_for_script_listener_thread =
            self.compositor_proxy.clone_compositor_proxy();
        thread::spawn(move || {
            compositor_task::run_script_listener_thread(
                compositor_proxy_for_script_listener_thread,
                script_to_compositor_port)
        });

        ScriptTaskFactory::create(None::<&mut STF>,
                                  self.id,
                                  self.parent_info,
                                  script_to_compositor_chan,
                                  &layout_pair,
                                  self.script_chan.clone(),
                                  mem::replace(&mut self.script_port, None).unwrap(),
                                  self.constellation_chan.clone(),
                                  self.failure.clone(),
                                  self.resource_task,
                                  self.storage_task.clone(),
                                  self.image_cache_task.clone(),
                                  self.mem_profiler_chan.clone(),
                                  self.devtools_chan,
                                  self.window_size,
                                  self.load_data.clone());

        LayoutTaskFactory::create(None::<&mut LTF>,
                                  self.id,
                                  self.load_data.url.clone(),
                                  self.parent_info.is_some(),
                                  layout_pair,
                                  self.pipeline_port.unwrap(),
                                  self.constellation_chan,
                                  self.failure,
                                  self.script_chan.clone(),
                                  self.layout_to_paint_chan.clone(),
                                  self.image_cache_task,
                                  self.font_cache_task,
                                  self.time_profiler_chan,
                                  self.mem_profiler_chan,
                                  self.layout_shutdown_chan);
    }

    pub fn start_paint_task(&mut self) {
        PaintTask::create(self.id,
                          self.load_data.url.clone(),
                          self.chrome_to_paint_chan.clone(),
                          mem::replace(&mut self.layout_to_paint_port, None).unwrap(),
                          mem::replace(&mut self.chrome_to_paint_port, None).unwrap(),
                          self.compositor_proxy.clone_compositor_proxy(),
                          self.constellation_chan.clone(),
                          self.font_cache_task.clone(),
                          self.failure.clone(),
                          self.time_profiler_chan.clone(),
                          self.mem_profiler_chan.clone(),
                          self.paint_shutdown_chan.clone());

    }
}

