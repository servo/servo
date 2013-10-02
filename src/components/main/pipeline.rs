/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use extra::url::Url;
use compositing::CompositorChan;
use gfx::render_task::{RenderChan, RenderTask};
use gfx::render_task::{PaintPermissionGranted, PaintPermissionRevoked};
use gfx::opts::Opts;
use layout::layout_task::LayoutTask;
use script::layout_interface::LayoutChan;
use script::script_task::{ExecuteMsg, LoadMsg};
use servo_msg::constellation_msg::{ConstellationChan, FailureMsg, PipelineId, SubpageId};
use script::dom::node::AbstractNode;
use script::script_task::{AttachLayoutMsg, NewLayoutInfo, ScriptTask, ScriptChan};
use script::script_task;
use servo_net::image_cache_task::ImageCacheTask;
use servo_net::resource_task::ResourceTask;
use servo_util::time::ProfilerChan;
use geom::size::Size2D;
use extra::future::Future;
use std::cell::Cell;
use std::task;

/// A uniquely-identifiable pipeline of script task, layout task, and render task. 
#[deriving(Clone)]
pub struct Pipeline {
    id: PipelineId,
    subpage_id: Option<SubpageId>,
    script_chan: ScriptChan,
    layout_chan: LayoutChan,
    render_chan: RenderChan<AbstractNode<()>>,
    /// The most recently loaded url
    url: Option<Url>,
}

impl Pipeline {
    /// Starts a render task, layout task, and script task. Returns the channels wrapped in a struct.
    pub fn with_script(id: PipelineId,
                       subpage_id: Option<SubpageId>,
                       constellation_chan: ConstellationChan,
                       compositor_chan: CompositorChan,
                       image_cache_task: ImageCacheTask,
                       profiler_chan: ProfilerChan,
                       opts: Opts,
                       script_pipeline: &Pipeline,
                       size_future: Future<Size2D<uint>>) -> Pipeline {
        
        let (layout_port, layout_chan) = special_stream!(LayoutChan);
        let (render_port, render_chan) = special_stream!(RenderChan);

        RenderTask::create(id,
                           render_port,
                           compositor_chan.clone(),
                           constellation_chan.clone(),
                           opts.clone(),
                           profiler_chan.clone());

        LayoutTask::create(id,
                           layout_port,
                           constellation_chan,
                           script_pipeline.script_chan.clone(),
                           render_chan.clone(),
                           image_cache_task.clone(),
                           opts.clone(),
                           profiler_chan);

        let new_layout_info = NewLayoutInfo {
            old_id: script_pipeline.id.clone(),
            new_id: id,
            layout_chan: layout_chan.clone(),
            size_future: size_future,
        };

        script_pipeline.script_chan.send(AttachLayoutMsg(new_layout_info));

        Pipeline::new(id,
                      subpage_id,
                      script_pipeline.script_chan.clone(),
                      layout_chan,
                      render_chan)

    }

    pub fn create(id: PipelineId,
                  subpage_id: Option<SubpageId>,
                  constellation_chan: ConstellationChan,
                  compositor_chan: CompositorChan,
                  image_cache_task: ImageCacheTask,
                  resource_task: ResourceTask,
                  profiler_chan: ProfilerChan,
                  opts: Opts,
                  size: Future<Size2D<uint>>) -> Pipeline {

        let (script_port, script_chan) = special_stream!(ScriptChan);
        let (layout_port, layout_chan) = special_stream!(LayoutChan);
        let (render_port, render_chan) = special_stream!(RenderChan);
        let pipeline = Pipeline::new(id,
                                     subpage_id,
                                     script_chan.clone(),
                                     layout_chan.clone(),
                                     render_chan.clone());
        let (port, chan) = stream::<task::TaskResult>();
        
        let script_port = Cell::new(script_port);
        let resource_task = Cell::new(resource_task);
        let size = Cell::new(size);
        let render_port = Cell::new(render_port);
        let layout_port = Cell::new(layout_port);
        let constellation_chan_handler = Cell::new(constellation_chan.clone());
        let constellation_chan = Cell::new(constellation_chan);
        let image_cache_task = Cell::new(image_cache_task);
        let profiler_chan = Cell::new(profiler_chan);

        do Pipeline::spawn(chan) {
            let script_port = script_port.take();
            let resource_task = resource_task.take();
            let size = size.take();
            let render_port = render_port.take();
            let layout_port = layout_port.take();
            let constellation_chan = constellation_chan.take();
            let image_cache_task = image_cache_task.take();
            let profiler_chan = profiler_chan.take();

            ScriptTask::create(id,
                               compositor_chan.clone(),
                               layout_chan.clone(),
                               script_port,
                               script_chan.clone(),
                               constellation_chan.clone(),
                               resource_task,
                               image_cache_task.clone(),
                               size);

            RenderTask::create(id,
                               render_port,
                               compositor_chan.clone(),
                               constellation_chan.clone(),
                               opts.clone(),
                               profiler_chan.clone());

            LayoutTask::create(id,
                               layout_port,
                               constellation_chan,
                               script_chan.clone(),
                               render_chan.clone(),
                               image_cache_task,
                               opts.clone(),
                               profiler_chan);
        };

        do spawn {
            match port.recv() {
                task::Success => (),
                task::Failure => {
                    let constellation_chan = constellation_chan_handler.take();
                    constellation_chan.send(FailureMsg(id, subpage_id));
                }
            }
        };

        pipeline
    }

    /// This function wraps the task creation within a supervised task
    /// so that failure will only tear down those tasks instead of ours.
    pub fn spawn(chan: Chan<task::TaskResult>, f: ~fn()) {
        let mut task = task::task();
        task.opts.notify_chan = Some(chan);
        task.supervised();
        do task.spawn {
            f();
        };
    }

    pub fn new(id: PipelineId,
               subpage_id: Option<SubpageId>,
               script_chan: ScriptChan,
               layout_chan: LayoutChan,
               render_chan: RenderChan<AbstractNode<()>>)
               -> Pipeline {
        Pipeline {
            id: id,
            subpage_id: subpage_id,
            script_chan: script_chan,
            layout_chan: layout_chan,
            render_chan: render_chan,
            url: None,
        }
    }

    pub fn load(&mut self, url: Url) {
        self.url = Some(url.clone());
        self.script_chan.send(LoadMsg(self.id, url));
    }

    pub fn execute(&mut self, url: Url) {
        self.url = Some(url.clone());
        self.script_chan.send(ExecuteMsg(self.id, url));
    }

    pub fn grant_paint_permission(&self) {
        self.render_chan.try_send(PaintPermissionGranted);
    }

    pub fn revoke_paint_permission(&self) {
        self.render_chan.send(PaintPermissionRevoked);
    }

    pub fn reload(&mut self) {
        do self.url.clone().map_move() |url| {
            self.load(url);
        };
    }

    pub fn exit(&self) {
        // Script task handles shutting down layout, 
        // and layout handles shutting down the renderer.
        self.script_chan.try_send(script_task::ExitPipelineMsg(self.id));
    }
}

