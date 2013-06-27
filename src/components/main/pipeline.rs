/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositing::CompositorChan;
use gfx::render_task::{RenderChan, RenderTask};
use gfx::render_task;
use gfx::opts::Opts;
use layout::layout_task::LayoutTask;
use script::layout_interface::LayoutChan;
use script::layout_interface;
use servo_msg::engine::{EngineChan};
use script::script_task::{ScriptTask, ScriptChan, ScriptMsg};
use script::script_task;
use servo_net::image_cache_task::ImageCacheTask;
use servo_net::resource_task::ResourceTask;
use servo_util::time::ProfilerChan;
use std::comm;

/// A uniquely-identifiable pipeline of stript task, layout task, and render task. 
pub struct Pipeline {
    id: uint,
    script_chan: ScriptChan,
    layout_chan: LayoutChan,
    render_chan: RenderChan,
}

impl Pipeline {
    /// Starts a render task, layout task, and script task. Returns the channels wrapped in a struct.
    pub fn create(id: uint,
                  engine_chan: EngineChan,
                  compositor_chan: CompositorChan,
                  image_cache_task: ImageCacheTask,
                  resource_task: ResourceTask,
                  profiler_chan: ProfilerChan,
                  opts: Opts) -> Pipeline {
        
        macro_rules! closure_stream(
            ($Msg:ty, $Chan:ident) => (
                {
                    let (port, chan) = comm::stream::<$Msg>();
                    (port, $Chan::new(chan))
                }
            );
        )
        // Create the script port and channel.
        let (script_port, script_chan) = closure_stream!(ScriptMsg, ScriptChan);

        // Create the layout port and channel.
        let (layout_port, layout_chan) = closure_stream!(layout_interface::Msg, LayoutChan);

        let (render_port, render_chan) = comm::stream::<render_task::Msg>();
        let render_chan = RenderChan::new(render_chan);

        RenderTask::create(render_port,
                           compositor_chan.clone(),
                           copy opts,
                           engine_chan.clone(),
                           profiler_chan.clone());

        LayoutTask::create(layout_port,
                           script_chan.clone(),
                           render_chan.clone(),
                           image_cache_task.clone(),
                           copy opts,
                           profiler_chan.clone());

        ScriptTask::create(id,
                           compositor_chan.clone(),
                           layout_chan.clone(),
                           script_port,
                           script_chan.clone(),
                           engine_chan,
                           resource_task.clone(),
                           image_cache_task.clone());

        Pipeline::new(id,
                      script_chan,
                      layout_chan,
                      render_chan)
    }

    pub fn new(id: uint,
               script_chan: ScriptChan,
               layout_chan: LayoutChan,
               render_chan: RenderChan)
               -> Pipeline {
        Pipeline {
            id: id,
            script_chan: script_chan,
            layout_chan: layout_chan,
            render_chan: render_chan,
        }
    }

    pub fn exit(&self) {
        self.script_chan.send(script_task::ExitMsg);
        self.layout_chan.send(layout_interface::ExitMsg);

        let (response_port, response_chan) = comm::stream();
        self.render_chan.send(render_task::ExitMsg(response_chan));
        response_port.recv();
    }
}

