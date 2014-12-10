/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use CompositorProxy;
use layout_traits::{LayoutTaskFactory, LayoutControlChan};
use main_thread::MainThreadProxy;
use script_traits::{InitialScriptState, ScriptControlChan, ScriptTaskFactory};
use script_traits::{AttachLayoutMsg, LoadMsg, NewLayoutInfo, ExitPipelineMsg};

use devtools_traits::DevtoolsControlChan;
use gfx::font_cache_task::FontCacheTask;
use gfx::paint_task::{PaintPermissionGranted, PaintPermissionRevoked};
use gfx::paint_task::{PaintChan, PaintTask};
use servo_msg::compositor_msg::ScriptToMainThreadProxy;
use servo_msg::constellation_msg::{ConstellationChan, Failure, PipelineId, SubpageId};
use servo_msg::constellation_msg::{LoadData, WindowSizeData};
use servo_net::image_cache_task::ImageCacheTask;
use servo_net::resource_task::ResourceTask;
use servo_net::storage_task::StorageTask;
use servo_util::time::TimeProfilerChan;
use std::rc::Rc;

/// A uniquely-identifiable pipeline of script task, layout task, and paint task.
pub struct Pipeline {
    pub id: PipelineId,
    pub subpage_id: Option<SubpageId>,
    pub script_chan: ScriptControlChan,
    pub layout_chan: LayoutControlChan,
    pub paint_chan: PaintChan,
    pub layout_shutdown_port: Receiver<()>,
    pub paint_shutdown_port: Receiver<()>,
    /// The most recently loaded page
    pub load_data: LoadData,
}

/// The subset of the pipeline that is needed for layer composition.
#[deriving(Clone)]
pub struct CompositionPipeline {
    pub id: PipelineId,
    pub script_chan: ScriptControlChan,
    pub paint_chan: PaintChan,
}

/// Initial setup data needed to construct a pipeline.
pub struct InitialPipelineState {
    /// The ID of the pipeline to create.
    pub id: PipelineId,
    /// The subpage ID of the pipeline to create. If `None`, this is the root.
    pub subpage_id: Option<SubpageId>,
    /// A channel to the main thread.
    pub main_thread_proxy: Box<MainThreadProxy + Send>,
    /// A channel to the associated constellation.
    pub constellation_chan: ConstellationChan,
    /// A channel to the compositor. If `None`, this is headless.
    pub compositor_proxy: Option<CompositorProxy>,
    /// A channel to the developer tools, if applicable.
    pub devtools_chan: Option<DevtoolsControlChan>,
    /// A channel to the image cache task.
    pub image_cache_task: ImageCacheTask,
    /// A channel to the font cache task.
    pub font_cache_task: FontCacheTask,
    /// A channel to the resource task.
    pub resource_task: ResourceTask,
    /// A channel to the storage task.
    pub storage_task: StorageTask,
    /// A channel to the time profiler thread.
    pub time_profiler_proxy: TimeProfilerChan,
    /// Information about the initial window size.
    pub window_size: WindowSizeData,
    /// The pipeline to use for script, if applicable. If this is `Some`, then `subpage_id` must
    /// also be `Some`.
    pub script_pipeline: Option<Rc<Pipeline>>,
    /// Information about the page to load.
    pub load_data: LoadData,
}

impl Pipeline {
    /// Starts a paint task, layout task, and possibly a script task. Returns the channels wrapped
    /// in a structure.
    pub fn create<LTF,STF>(state: InitialPipelineState)
                           -> Pipeline
                           where LTF:LayoutTaskFactory, STF:ScriptTaskFactory {
        let layout_pair = ScriptTaskFactory::create_layout_channel(None::<&mut STF>);
        let (paint_port, paint_chan) = PaintChan::new();
        let (paint_shutdown_chan, paint_shutdown_port) = channel();
        let (layout_shutdown_chan, layout_shutdown_port) = channel();
        let (pipeline_chan, pipeline_port) = channel();

        let failure = Failure {
            pipeline_id: state.id,
            subpage_id: state.subpage_id,
        };

        let script_chan = match state.script_pipeline {
            None => {
                let (script_chan, script_port) = channel();
                ScriptTaskFactory::create(None::<&mut STF>, InitialScriptState {
                    id: state.id,
                    main_thread_proxy: box state.main_thread_proxy as
                        Box<ScriptToMainThreadProxy + Send>,
                    compositor: state.compositor_proxy.clone(),
                    control_chan: ScriptControlChan(script_chan.clone()),
                    control_port: script_port,
                    constellation_proxy: state.constellation_chan.clone(),
                    failure_info: failure.clone(),
                    resource_task: state.resource_task.clone(),
                    storage_task: state.storage_task.clone(),
                    image_cache_task: state.image_cache_task.clone(),
                    devtools_chan: state.devtools_chan,
                    window_size: state.window_size,
                }, &layout_pair);
                ScriptControlChan(script_chan)
            }
            Some(spipe) => {
                let new_layout_info = NewLayoutInfo {
                    old_pipeline_id: spipe.id.clone(),
                    new_pipeline_id: state.id,
                    subpage_id: state.subpage_id
                                     .expect("script_pipeline != None but subpage_id == None"),
                    layout_chan: ScriptTaskFactory::clone_layout_channel(None::<&mut STF>,
                                                                         &layout_pair),
                };

                let ScriptControlChan(ref chan) = spipe.script_chan;
                chan.send(AttachLayoutMsg(new_layout_info));
                spipe.script_chan.clone()
            }
        };

        PaintTask::create(state.id,
                          paint_port,
                          state.compositor_proxy,
                          state.constellation_chan.clone(),
                          state.font_cache_task.clone(),
                          failure.clone(),
                          state.time_profiler_proxy.clone(),
                          paint_shutdown_chan);

        LayoutTaskFactory::create(None::<&mut LTF>,
                                  state.id,
                                  layout_pair,
                                  pipeline_port,
                                  state.constellation_chan,
                                  failure,
                                  script_chan.clone(),
                                  paint_chan.clone(),
                                  state.resource_task,
                                  state.image_cache_task,
                                  state.font_cache_task,
                                  state.time_profiler_proxy,
                                  layout_shutdown_chan);

        Pipeline::new(state.id,
                      state.subpage_id,
                      script_chan,
                      LayoutControlChan(pipeline_chan),
                      paint_chan,
                      layout_shutdown_port,
                      paint_shutdown_port,
                      state.load_data)
    }

    pub fn new(id: PipelineId,
               subpage_id: Option<SubpageId>,
               script_chan: ScriptControlChan,
               layout_chan: LayoutControlChan,
               paint_chan: PaintChan,
               layout_shutdown_port: Receiver<()>,
               paint_shutdown_port: Receiver<()>,
               load_data: LoadData)
               -> Pipeline {
        Pipeline {
            id: id,
            subpage_id: subpage_id,
            script_chan: script_chan,
            layout_chan: layout_chan,
            paint_chan: paint_chan,
            layout_shutdown_port: layout_shutdown_port,
            paint_shutdown_port: paint_shutdown_port,
            load_data: load_data,
        }
    }

    pub fn load(&self) {
        let ScriptControlChan(ref chan) = self.script_chan;
        chan.send(LoadMsg(self.id, self.load_data.clone()));
    }

    pub fn grant_paint_permission(&self) {
        let _ = self.paint_chan.send_opt(PaintPermissionGranted);
    }

    pub fn revoke_paint_permission(&self) {
        debug!("pipeline revoking paint channel paint permission");
        let _ = self.paint_chan.send_opt(PaintPermissionRevoked);
    }

    pub fn exit(&self) {
        debug!("pipeline {} exiting", self.id);

        // Script task handles shutting down layout, and layout handles shutting down the painter.
        // For now, if the script task has failed, we give up on clean shutdown.
        let ScriptControlChan(ref chan) = self.script_chan;
        if chan.send_opt(ExitPipelineMsg(self.id)).is_ok() {
            // Wait until all slave tasks have terminated and run destructors
            // NOTE: We don't wait for script task as we don't always own it
            let _ = self.paint_shutdown_port.recv_opt();
            let _ = self.layout_shutdown_port.recv_opt();
        }
    }

    pub fn to_sendable(&self) -> CompositionPipeline {
        CompositionPipeline {
            id: self.id.clone(),
            script_chan: self.script_chan.clone(),
            paint_chan: self.paint_chan.clone(),
        }
    }
}
