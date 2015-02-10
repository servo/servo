/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use CompositorProxy;
use layout_traits::{LayoutControlMsg, LayoutTaskFactory, LayoutControlChan};
use script_traits::{ScriptControlChan, ScriptTaskFactory};
use script_traits::{NewLayoutInfo, ConstellationControlMsg};

use devtools_traits::DevtoolsControlChan;
use gfx::paint_task::Msg as PaintMsg;
use gfx::paint_task::{PaintChan, PaintTask};
use gfx::font_cache_task::FontCacheTask;
use msg::constellation_msg::{ConstellationChan, Failure, PipelineId, SubpageId};
use msg::constellation_msg::{LoadData, WindowSizeData, PipelineExitType};
use net::image_cache_task::ImageCacheTask;
use net::resource_task::ResourceTask;
use net::storage_task::StorageTask;
use util::time::TimeProfilerChan;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, channel};

/// A uniquely-identifiable pipeline of script task, layout task, and paint task.
pub struct Pipeline {
    pub id: PipelineId,
    pub parent: Option<(PipelineId, SubpageId)>,
    pub script_chan: ScriptControlChan,
    pub layout_chan: LayoutControlChan,
    pub paint_chan: PaintChan,
    pub layout_shutdown_port: Receiver<()>,
    pub paint_shutdown_port: Receiver<()>,
    /// Load data corresponding to the most recently-loaded page.
    pub load_data: LoadData,
    /// The title of the most recently-loaded page.
    pub title: Option<String>,
}

/// The subset of the pipeline that is needed for layer composition.
#[derive(Clone)]
pub struct CompositionPipeline {
    pub id: PipelineId,
    pub script_chan: ScriptControlChan,
    pub paint_chan: PaintChan,
}

impl Pipeline {
    /// Starts a paint task, layout task, and possibly a script task.
    /// Returns the channels wrapped in a struct.
    /// If script_pipeline is not None, then subpage_id must also be not None.
    pub fn create<LTF,STF>(id: PipelineId,
                           parent: Option<(PipelineId, SubpageId)>,
                           constellation_chan: ConstellationChan,
                           compositor_proxy: Box<CompositorProxy+'static+Send>,
                           devtools_chan: Option<DevtoolsControlChan>,
                           image_cache_task: ImageCacheTask,
                           font_cache_task: FontCacheTask,
                           resource_task: ResourceTask,
                           storage_task: StorageTask,
                           time_profiler_chan: TimeProfilerChan,
                           window_size: WindowSizeData,
                           script_pipeline: Option<Rc<Pipeline>>,
                           load_data: LoadData)
                           -> Pipeline
                           where LTF: LayoutTaskFactory, STF:ScriptTaskFactory {
        let layout_pair = ScriptTaskFactory::create_layout_channel(None::<&mut STF>);
        let (paint_port, paint_chan) = PaintChan::new();
        let (paint_shutdown_chan, paint_shutdown_port) = channel();
        let (layout_shutdown_chan, layout_shutdown_port) = channel();
        let (pipeline_chan, pipeline_port) = channel();

        let failure = Failure {
            pipeline_id: id,
            parent: parent,
        };

        let script_chan = match script_pipeline {
            None => {
                let (script_chan, script_port) = channel();
                ScriptTaskFactory::create(None::<&mut STF>,
                                          id,
                                          compositor_proxy.clone_compositor_proxy(),
                                          &layout_pair,
                                          ScriptControlChan(script_chan.clone()),
                                          script_port,
                                          constellation_chan.clone(),
                                          failure.clone(),
                                          resource_task.clone(),
                                          storage_task.clone(),
                                          image_cache_task.clone(),
                                          devtools_chan,
                                          window_size);
                ScriptControlChan(script_chan)
            }
            Some(spipe) => {
                let new_layout_info = NewLayoutInfo {
                    old_pipeline_id: spipe.id.clone(),
                    new_pipeline_id: id,
                    subpage_id: parent.expect("script_pipeline != None but subpage_id == None").1,
                    layout_chan: ScriptTaskFactory::clone_layout_channel(None::<&mut STF>, &layout_pair),
                };

                let ScriptControlChan(ref chan) = spipe.script_chan;
                chan.send(ConstellationControlMsg::AttachLayout(new_layout_info)).unwrap();
                spipe.script_chan.clone()
            }
        };

        PaintTask::create(id,
                          paint_port,
                          compositor_proxy,
                          constellation_chan.clone(),
                          font_cache_task.clone(),
                          failure.clone(),
                          time_profiler_chan.clone(),
                          paint_shutdown_chan);

        LayoutTaskFactory::create(None::<&mut LTF>,
                                  id,
                                  layout_pair,
                                  pipeline_port,
                                  constellation_chan,
                                  failure,
                                  script_chan.clone(),
                                  paint_chan.clone(),
                                  resource_task,
                                  image_cache_task,
                                  font_cache_task,
                                  time_profiler_chan,
                                  layout_shutdown_chan);

        Pipeline::new(id,
                      parent,
                      script_chan,
                      LayoutControlChan(pipeline_chan),
                      paint_chan,
                      layout_shutdown_port,
                      paint_shutdown_port,
                      load_data)
    }

    pub fn new(id: PipelineId,
               parent: Option<(PipelineId, SubpageId)>,
               script_chan: ScriptControlChan,
               layout_chan: LayoutControlChan,
               paint_chan: PaintChan,
               layout_shutdown_port: Receiver<()>,
               paint_shutdown_port: Receiver<()>,
               load_data: LoadData)
               -> Pipeline {
        Pipeline {
            id: id,
            parent: parent,
            script_chan: script_chan,
            layout_chan: layout_chan,
            paint_chan: paint_chan,
            layout_shutdown_port: layout_shutdown_port,
            paint_shutdown_port: paint_shutdown_port,
            load_data: load_data,
            title: None,
        }
    }

    pub fn load(&self) {
        let ScriptControlChan(ref chan) = self.script_chan;
        chan.send(ConstellationControlMsg::Load(self.id,
                                                self.parent,
                                                self.load_data.clone())).unwrap();
    }

    pub fn grant_paint_permission(&self) {
        let _ = self.paint_chan.send(PaintMsg::PaintPermissionGranted);
    }

    pub fn revoke_paint_permission(&self) {
        debug!("pipeline revoking paint channel paint permission");
        let _ = self.paint_chan.send(PaintMsg::PaintPermissionRevoked);
    }

    pub fn exit(&self, exit_type: PipelineExitType) {
        debug!("pipeline {:?} exiting", self.id);

        // Script task handles shutting down layout, and layout handles shutting down the painter.
        // For now, if the script task has failed, we give up on clean shutdown.
        let ScriptControlChan(ref chan) = self.script_chan;
        if chan.send(ConstellationControlMsg::ExitPipeline(self.id, exit_type)).is_ok() {
            // Wait until all slave tasks have terminated and run destructors
            // NOTE: We don't wait for script task as we don't always own it
            let _ = self.paint_shutdown_port.recv();
            let _ = self.layout_shutdown_port.recv();
        }

    }

    pub fn force_exit(&self) {
        let ScriptControlChan(ref script_channel) = self.script_chan;
        let _ = script_channel.send(
            ConstellationControlMsg::ExitPipeline(self.id,
                                                  PipelineExitType::PipelineOnly)).unwrap();
        let _ = self.paint_chan.send(PaintMsg::Exit(None, PipelineExitType::PipelineOnly));
        let LayoutControlChan(ref layout_channel) = self.layout_chan;
        let _ = layout_channel.send(
            LayoutControlMsg::ExitNowMsg(PipelineExitType::PipelineOnly)).unwrap();
    }

    pub fn to_sendable(&self) -> CompositionPipeline {
        CompositionPipeline {
            id: self.id.clone(),
            script_chan: self.script_chan.clone(),
            paint_chan: self.paint_chan.clone(),
        }
    }

    pub fn subpage_id(&self) -> Option<SubpageId> {
      self.parent.map(|parent| parent.1)
    }
}
