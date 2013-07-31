/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use extra::net::url::Url;
use compositing::CompositorChan;
use gfx::render_task::{RenderChan, RenderTask};
use gfx::render_task::{PaintPermissionGranted, PaintPermissionRevoked};
use gfx::render_task;
use gfx::opts::Opts;
use layout::layout_task::LayoutTask;
use script::layout_interface::LayoutChan;
use script::script_task::{ExecuteMsg, LoadMsg};
use servo_msg::constellation_msg::{ConstellationChan, NavigationType, PipelineId, SubpageId};
use script::script_task::{AttachLayoutMsg, NewLayoutInfo, ScriptTask, ScriptChan};
use script::script_task;
use servo_net::image_cache_task::ImageCacheTask;
use servo_net::resource_task::ResourceTask;
use servo_util::time::ProfilerChan;
use geom::size::Size2D;
use extra::future::Future;
use std::comm;

/// A uniquely-identifiable pipeline of stript task, layout task, and render task. 
#[deriving(Clone)]
pub struct Pipeline {
    id: PipelineId,
    subpage_id: Option<SubpageId>,
    script_chan: ScriptChan,
    layout_chan: LayoutChan,
    render_chan: RenderChan,
    /// The most recently loaded url
    url: Option<Url>,
    navigation_type: Option<NavigationType>,
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
                           copy opts,
                           profiler_chan.clone());

        LayoutTask::create(id,
                           layout_port,
                           constellation_chan,
                           script_pipeline.script_chan.clone(),
                           render_chan.clone(),
                           image_cache_task.clone(),
                           copy opts,
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
                           copy opts,
                           profiler_chan.clone());

        LayoutTask::create(id,
                           layout_port,
                           constellation_chan,
                           script_chan.clone(),
                           render_chan.clone(),
                           image_cache_task,
                           copy opts,
                           profiler_chan);
        Pipeline::new(id,
                      subpage_id,
                      script_chan,
                      layout_chan,
                      render_chan)
    }

    pub fn new(id: PipelineId,
               subpage_id: Option<SubpageId>,
               script_chan: ScriptChan,
               layout_chan: LayoutChan,
               render_chan: RenderChan)
               -> Pipeline {
        Pipeline {
            id: id,
            subpage_id: subpage_id,
            script_chan: script_chan,
            layout_chan: layout_chan,
            render_chan: render_chan,
            url: None,
            navigation_type: None,
        }
    }

    pub fn load(&mut self, url: Url, navigation_type: Option<NavigationType>) {
        self.url = Some(url.clone());
        self.navigation_type = navigation_type;
        self.script_chan.send(LoadMsg(self.id, url));
    }

    pub fn execute(&mut self, url: Url) {
        self.url = Some(url.clone());
        self.script_chan.send(ExecuteMsg(self.id, url));
    }

    pub fn grant_paint_permission(&self) {
        self.render_chan.send(PaintPermissionGranted);
    }

    pub fn revoke_paint_permission(&self) {
        self.render_chan.send(PaintPermissionRevoked);
    }

    pub fn reload(&mut self, navigation_type: Option<NavigationType>) {
        if self.url.is_some() {
            let url = self.url.get_ref().clone();
            self.load(url, navigation_type);
        }
    }

    pub fn exit(&self) {
        // Script task handles shutting down layout, as well
        self.script_chan.send(script_task::ExitMsg);

        let (response_port, response_chan) = comm::stream();
        self.render_chan.send(render_task::ExitMsg(response_chan));
        response_port.recv();
    }
}

