/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use CompositorProxy;
use content_process::{mod, AuxiliaryContentProcessData, ContentProcess, ContentProcessIpc, Zone};
use layout_traits::{LayoutControlMsg, LayoutTaskFactory, LayoutControlChan};
use script_traits::{ScriptControlChan, ScriptTaskFactory};
use script_traits::{ConstellationControlMsg};

use devtools_traits::DevtoolsControlChan;
use gfx::font_cache_task::FontCacheTask;
use gfx::paint_task::Msg as PaintMsg;
use gfx::paint_task::{PaintChan, PaintTask};
use servo_msg::compositor_msg::ScriptToCompositorMsg;
use servo_msg::constellation_msg::{ConstellationChan, Failure, PipelineId, SubpageId};
use servo_msg::constellation_msg::{LoadData, WindowSizeData, PipelineExitType};
use servo_net::image_cache_task::ImageCacheTask;
use servo_net::resource_task::ResourceTask;
use servo_net::server::SharedServerProxy;
use servo_net::storage_task::StorageTask;
use servo_util::ipc;
use servo_util::opts;
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
    /// Load data corresponding to the most recently-loaded page.
    pub load_data: LoadData,
    /// The title of the most recently-loaded page.
    pub title: Option<String>,
}

/// The subset of the pipeline that is needed for layer composition.
#[deriving(Clone)]
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
                           subpage_id: Option<SubpageId>,
                           constellation_chan: ConstellationChan,
                           compositor_proxy: Box<CompositorProxy+'static+Send>,
                           script_to_compositor_client: SharedServerProxy<ScriptToCompositorMsg,
                                                                          ()>,
                           _: Option<DevtoolsControlChan>,
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
        let (paint_port, paint_chan) = PaintChan::new();
        let (_, layout_shutdown_port) = channel();
        let (pipeline_port, pipeline_chan) = ipc::channel();

        let failure = Failure {
            pipeline_id: id,
            subpage_id: subpage_id,
        };

        let (script_port, script_chan) = ipc::channel();
        let content_process_ipc = ContentProcessIpc {
            script_to_compositor_client: script_to_compositor_client,
            script_port: script_port,
            constellation_chan: constellation_chan.clone(),
            storage_task: storage_task,
            pipeline_to_layout_port: pipeline_port,
            layout_to_paint_chan: paint_chan.create_layout_channel(),
            font_cache_task: font_cache_task.clone(),
        };

        match script_pipeline {
            None => {
                let data = AuxiliaryContentProcessData {
                    pipeline_id: id,
                    failure: failure,
                    window_size: window_size,
                    zone: Zone::from_load_data(&load_data),
                };

                if !opts::get().multiprocess {
                    let content_process = ContentProcess {
                        ipc: content_process_ipc,
                        resource_task: resource_task.clone(),
                        image_cache_task: image_cache_task.clone(),
                        time_profiler_chan: time_profiler_chan.clone(),
                    };
                    content_process.create_script_and_layout_threads(data)
                } else {
                    content_process::spawn(content_process_ipc, data)
                }
            }
            Some(_spipe) => {
                panic!("layout connection to existing script thread not yet ported to e10s")
            }
        }

        PaintTask::create(id,
                          paint_port,
                          compositor_proxy,
                          constellation_chan.clone(),
                          font_cache_task,
                          failure.clone(),
                          time_profiler_chan.clone());

        Pipeline::new(id,
                      subpage_id,
                      ScriptControlChan(script_chan),
                      LayoutControlChan(pipeline_chan),
                      paint_chan,
                      layout_shutdown_port,
                      load_data)
    }

    pub fn new(id: PipelineId,
               subpage_id: Option<SubpageId>,
               script_chan: ScriptControlChan,
               layout_chan: LayoutControlChan,
               paint_chan: PaintChan,
               layout_shutdown_port: Receiver<()>,
               load_data: LoadData)
               -> Pipeline {
        Pipeline {
            id: id,
            subpage_id: subpage_id,
            script_chan: script_chan,
            layout_chan: layout_chan,
            paint_chan: paint_chan,
            layout_shutdown_port: layout_shutdown_port,
            load_data: load_data,
            title: None,
        }
    }

    pub fn load(&self) {
        let ScriptControlChan(ref chan) = self.script_chan;
        chan.send(ConstellationControlMsg::Load(self.id, self.load_data.clone()));
    }

    pub fn grant_paint_permission(&self) {
        let _ = self.paint_chan.send_opt(PaintMsg::PaintPermissionGranted);
    }

    pub fn revoke_paint_permission(&self) {
        debug!("pipeline revoking paint channel paint permission");
        let _ = self.paint_chan.send_opt(PaintMsg::PaintPermissionRevoked);
    }

    pub fn exit(&self, exit_type: PipelineExitType) {
        debug!("pipeline {} exiting", self.id);

        // Script task handles shutting down layout, and layout handles shutting down the painter.
        // For now, if the script task has failed, we give up on clean shutdown.
        let ScriptControlChan(ref chan) = self.script_chan;
        if chan.send_opt(ConstellationControlMsg::ExitPipeline(self.id, exit_type)).is_ok() {
            // Wait until all slave tasks have terminated and run destructors
            // NOTE: We don't wait for script task as we don't always own it
            let _ = self.layout_shutdown_port.recv_opt();
        }

    }

    pub fn force_exit(&self) {
        let ScriptControlChan(ref script_channel) = self.script_chan;
        let _ = script_channel.send_opt(
            ConstellationControlMsg::ExitPipeline(self.id,
                                                  PipelineExitType::PipelineOnly));
        let _ = self.paint_chan.send_opt(PaintMsg::Exit(PipelineExitType::PipelineOnly));
        let LayoutControlChan(ref layout_channel) = self.layout_chan;
        let _ = layout_channel.send_opt(LayoutControlMsg::ExitNowMsg(PipelineExitType::PipelineOnly));
    }

    pub fn to_sendable(&self) -> CompositionPipeline {
        CompositionPipeline {
            id: self.id.clone(),
            script_chan: self.script_chan.clone(),
            paint_chan: self.paint_chan.clone(),
        }
    }
}
