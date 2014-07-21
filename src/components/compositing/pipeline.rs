/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use CompositorChan;
use layout_traits::LayoutTaskFactory;

use gfx::render_task::{PaintPermissionGranted, PaintPermissionRevoked};
use gfx::render_task::{RenderChan, RenderTask};
use script::layout_interface::LayoutChan;
use script::script_task::LoadMsg;
use script::script_task::{AttachLayoutMsg, NewLayoutInfo, ScriptTask, ScriptChan};
use script::script_task;
use servo_msg::constellation_msg::{ConstellationChan, Failure, PipelineId, SubpageId};
use servo_msg::constellation_msg::WindowSizeData;
use servo_net::image_cache_task::ImageCacheTask;
use gfx::font_cache_task::FontCacheTask;
use servo_net::resource_task::ResourceTask;
use servo_util::opts::Opts;
use servo_util::time::TimeProfilerChan;
use std::rc::Rc;
use url::Url;

/// A uniquely-identifiable pipeline of script task, layout task, and render task.
pub struct Pipeline {
    pub id: PipelineId,
    pub subpage_id: Option<SubpageId>,
    pub script_chan: ScriptChan,
    pub layout_chan: LayoutChan,
    pub render_chan: RenderChan,
    pub layout_shutdown_port: Receiver<()>,
    pub render_shutdown_port: Receiver<()>,
    /// The most recently loaded url
    pub url: Url,
}

/// The subset of the pipeline that is needed for layer composition.
#[deriving(Clone)]
pub struct CompositionPipeline {
    pub id: PipelineId,
    pub script_chan: ScriptChan,
    pub render_chan: RenderChan,
}

impl Pipeline {
    /// Starts a render task, layout task, and possibly a script task.
    /// Returns the channels wrapped in a struct.
    /// If script_pipeline is not None, then subpage_id must also be not None.
    pub fn create<LTF:LayoutTaskFactory>(
                      id: PipelineId,
                      subpage_id: Option<SubpageId>,
                      constellation_chan: ConstellationChan,
                      compositor_chan: CompositorChan,
                      image_cache_task: ImageCacheTask,
                      font_cache_task: FontCacheTask,
                      resource_task: ResourceTask,
                      time_profiler_chan: TimeProfilerChan,
                      window_size: WindowSizeData,
                      opts: Opts,
                      script_pipeline: Option<Rc<Pipeline>>,
                      url: Url)
                      -> Pipeline {
        let (layout_port, layout_chan) = LayoutChan::new();
        let (render_port, render_chan) = RenderChan::new();
        let (render_shutdown_chan, render_shutdown_port) = channel();
        let (layout_shutdown_chan, layout_shutdown_port) = channel();

        let failure = Failure {
            pipeline_id: id,
            subpage_id: subpage_id,
        };

        let script_chan = match script_pipeline {
            None => {
                let (script_port, script_chan) = ScriptChan::new();
                ScriptTask::create(id,
                               box compositor_chan.clone(),
                               layout_chan.clone(),
                               script_port,
                               script_chan.clone(),
                               constellation_chan.clone(),
                               failure.clone(),
                               resource_task,
                               image_cache_task.clone(),
                               window_size);
                script_chan
            }
            Some(spipe) => {
                let new_layout_info = NewLayoutInfo {
                    old_pipeline_id: spipe.id.clone(),
                    new_pipeline_id: id,
                    subpage_id: subpage_id.expect("script_pipeline != None but subpage_id == None"),
                    layout_chan: layout_chan.clone(),
                };

                let ScriptChan(ref chan) = spipe.script_chan;
                chan.send(AttachLayoutMsg(new_layout_info));
                spipe.script_chan.clone()
            }
        };

        RenderTask::create(id,
                           render_port,
                           compositor_chan.clone(),
                           constellation_chan.clone(),
                           font_cache_task.clone(),
                           failure.clone(),
                           opts.clone(),
                           time_profiler_chan.clone(),
                           render_shutdown_chan);

        LayoutTaskFactory::create(None::<&mut LTF>,
                                  id,
                                  layout_port,
                                  layout_chan.clone(),
                                  constellation_chan,
                                  failure,
                                  script_chan.clone(),
                                  render_chan.clone(),
                                  image_cache_task,
                                  font_cache_task,
                                  opts.clone(),
                                  time_profiler_chan,
                                  layout_shutdown_chan);

        Pipeline::new(id,
                      subpage_id,
                      script_chan,
                      layout_chan,
                      render_chan,
                      layout_shutdown_port,
                      render_shutdown_port,
                      url)
    }

    pub fn new(id: PipelineId,
               subpage_id: Option<SubpageId>,
               script_chan: ScriptChan,
               layout_chan: LayoutChan,
               render_chan: RenderChan,
               layout_shutdown_port: Receiver<()>,
               render_shutdown_port: Receiver<()>,
               url: Url)
               -> Pipeline {
        Pipeline {
            id: id,
            subpage_id: subpage_id,
            script_chan: script_chan,
            layout_chan: layout_chan,
            render_chan: render_chan,
            layout_shutdown_port: layout_shutdown_port,
            render_shutdown_port: render_shutdown_port,
            url: url,
        }
    }

    pub fn load(&self) {
        let ScriptChan(ref chan) = self.script_chan;
        chan.send(LoadMsg(self.id, self.url.clone()));
    }

    pub fn grant_paint_permission(&self) {
        let _ = self.render_chan.send_opt(PaintPermissionGranted);
    }

    pub fn revoke_paint_permission(&self) {
        debug!("pipeline revoking render channel paint permission");
        let _ = self.render_chan.send_opt(PaintPermissionRevoked);
    }

    pub fn exit(&self) {
        debug!("pipeline {:?} exiting", self.id);

        // Script task handles shutting down layout, and layout handles shutting down the renderer.
        // For now, if the script task has failed, we give up on clean shutdown.
        let ScriptChan(ref chan) = self.script_chan;
        if chan.send_opt(script_task::ExitPipelineMsg(self.id)).is_ok() {
            // Wait until all slave tasks have terminated and run destructors
            // NOTE: We don't wait for script task as we don't always own it
            let _ = self.render_shutdown_port.recv_opt();
            let _ = self.layout_shutdown_port.recv_opt();
        }
    }

    pub fn to_sendable(&self) -> CompositionPipeline {
        CompositionPipeline {
            id: self.id.clone(),
            script_chan: self.script_chan.clone(),
            render_chan: self.render_chan.clone(),
        }
    }
}

