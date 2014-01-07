/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositing::CompositorChan;
use layout::layout_task::LayoutTask;

use extra::url::Url;
use geom::size::Size2D;
use gfx::opts::Opts;
use gfx::render_task::{PaintPermissionGranted, PaintPermissionRevoked};
use gfx::render_task::{RenderChan, RenderTask};
use layout::util::OpaqueNode;
use script::layout_interface::LayoutChan;
use script::script_task::LoadMsg;
use script::script_task::{AttachLayoutMsg, NewLayoutInfo, ScriptTask, ScriptChan};
use script::script_task;
use servo_msg::constellation_msg::{ConstellationChan, FailureMsg, PipelineId, SubpageId};
use servo_net::image_cache_task::ImageCacheTask;
use servo_net::resource_task::ResourceTask;
use servo_util::time::ProfilerChan;
use std::task;
use std::comm;

/// A uniquely-identifiable pipeline of script task, layout task, and render task. 
pub struct Pipeline {
    id: PipelineId,
    subpage_id: Option<SubpageId>,
    script_chan: ScriptChan,
    layout_chan: LayoutChan,
    render_chan: RenderChan<OpaqueNode>,
    layout_shutdown_port: Port<()>,
    render_shutdown_port: Port<()>,
    /// The most recently loaded url
    url: Option<Url>,
}

/// The subset of the pipeline that is needed for layer composition.
#[deriving(Clone)]
pub struct CompositionPipeline {
    id: PipelineId,
    script_chan: ScriptChan,
    render_chan: RenderChan<OpaqueNode>,
}

impl Pipeline {
    /// Starts a render task, layout task, and script task. Returns the channels wrapped in a
    /// struct.
    pub fn with_script(id: PipelineId,
                       subpage_id: Option<SubpageId>,
                       constellation_chan: ConstellationChan,
                       compositor_chan: CompositorChan,
                       image_cache_task: ImageCacheTask,
                       profiler_chan: ProfilerChan,
                       opts: Opts,
                       script_pipeline: &Pipeline)
                       -> Pipeline {
        let (layout_port, layout_chan) = special_stream!(LayoutChan);
        let (render_port, render_chan) = special_stream!(RenderChan);
        let (render_shutdown_port, render_shutdown_chan) = comm::stream();
        let (layout_shutdown_port, layout_shutdown_chan) = comm::stream();

        RenderTask::create(id,
                           render_port,
                           compositor_chan.clone(),
                           constellation_chan.clone(),
                           opts.clone(),
                           profiler_chan.clone(),
                           render_shutdown_chan);

        LayoutTask::create(id,
                           layout_port,
                           constellation_chan,
                           script_pipeline.script_chan.clone(),
                           render_chan.clone(),
                           image_cache_task.clone(),
                           opts.clone(),
                           profiler_chan,
                           layout_shutdown_chan);

        let new_layout_info = NewLayoutInfo {
            old_id: script_pipeline.id.clone(),
            new_id: id,
            layout_chan: layout_chan.clone(),
        };

        script_pipeline.script_chan.send(AttachLayoutMsg(new_layout_info));

        Pipeline::new(id,
                      subpage_id,
                      script_pipeline.script_chan.clone(),
                      layout_chan,
                      render_chan,
                      layout_shutdown_port,
                      render_shutdown_port)
    }

    pub fn create(id: PipelineId,
                  subpage_id: Option<SubpageId>,
                  constellation_chan: ConstellationChan,
                  compositor_chan: CompositorChan,
                  image_cache_task: ImageCacheTask,
                  resource_task: ResourceTask,
                  profiler_chan: ProfilerChan,
                  window_size: Size2D<uint>,
                  opts: Opts)
                  -> Pipeline {
        let (script_port, script_chan) = special_stream!(ScriptChan);
        let (layout_port, layout_chan) = special_stream!(LayoutChan);
        let (render_port, render_chan) = special_stream!(RenderChan);
        let (render_shutdown_port, render_shutdown_chan) = comm::stream();
        let (layout_shutdown_port, layout_shutdown_chan) = comm::stream();
        let pipeline = Pipeline::new(id,
                                     subpage_id,
                                     script_chan.clone(),
                                     layout_chan.clone(),
                                     render_chan.clone(),
                                     layout_shutdown_port,
                                     render_shutdown_port);

        // Wrap task creation within a supervised task so that failure will
        // only tear down those tasks instead of ours.
        let hard_fail = opts.hard_fail;
        let failure_chan = constellation_chan.clone();
        let mut supervised_task = task::task();
        let task_port = supervised_task.future_result();
        supervised_task.supervised();

        spawn_with!(supervised_task, [
                    script_port,
                    resource_task,
                    render_port,
                    layout_port,
                    constellation_chan,
                    image_cache_task,
                    profiler_chan,
                    layout_shutdown_chan,
                    render_shutdown_chan
                ], {
            ScriptTask::create(id,
                               compositor_chan.clone(),
                               layout_chan.clone(),
                               script_port,
                               script_chan.clone(),
                               constellation_chan.clone(),
                               resource_task,
                               image_cache_task.clone(),
                               window_size);

            RenderTask::create(id,
                               render_port,
                               compositor_chan.clone(),
                               constellation_chan.clone(),
                               opts.clone(),
                               profiler_chan.clone(),
                               render_shutdown_chan);

            LayoutTask::create(id,
                               layout_port,
                               constellation_chan,
                               script_chan.clone(),
                               render_chan.clone(),
                               image_cache_task,
                               opts.clone(),
                               profiler_chan,
                               layout_shutdown_chan);
        });

        spawn_with!(task::task(), [failure_chan], {
            match task_port.recv() {
                Ok(*) => (),
                Err(*) => {
                    if hard_fail {
                        fail!("Pipeline failed in hard-fail mode");
                    }
                    failure_chan.send(FailureMsg(id, subpage_id));
                }
            }
        });

        pipeline
    }

    pub fn new(id: PipelineId,
               subpage_id: Option<SubpageId>,
               script_chan: ScriptChan,
               layout_chan: LayoutChan,
               render_chan: RenderChan<OpaqueNode>,
               layout_shutdown_port: Port<()>,
               render_shutdown_port: Port<()>)
               -> Pipeline {
        Pipeline {
            id: id,
            subpage_id: subpage_id,
            script_chan: script_chan,
            layout_chan: layout_chan,
            render_chan: render_chan,
            layout_shutdown_port: layout_shutdown_port,
            render_shutdown_port: render_shutdown_port,
            url: None,
        }
    }

    pub fn load(&mut self, url: Url) {
        self.url = Some(url.clone());
        self.script_chan.send(LoadMsg(self.id, url));
    }

    pub fn grant_paint_permission(&self) {
        self.render_chan.try_send(PaintPermissionGranted);
    }

    pub fn revoke_paint_permission(&self) {
        self.render_chan.send(PaintPermissionRevoked);
    }

    pub fn reload(&mut self) {
        do self.url.clone().map() |url| {
            self.load(url);
        };
    }

    pub fn exit(&self) {
        // Script task handles shutting down layout, and layout handles shutting down the renderer.
        self.script_chan.try_send(script_task::ExitPipelineMsg(self.id));

        // Wait until all slave tasks have terminated and run destructors
        // NOTE: We don't wait for script task as we don't always own it
        self.render_shutdown_port.try_recv();
        self.layout_shutdown_port.try_recv();
    }

    pub fn to_sendable(&self) -> CompositionPipeline {
        CompositionPipeline {
            id: self.id.clone(),
            script_chan: self.script_chan.clone(),
            render_chan: self.render_chan.clone(),
        }
    }
}

