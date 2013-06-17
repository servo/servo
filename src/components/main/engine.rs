/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositing::{CompositorChan, SetLayoutChan};
use layout::layout_task;

use core::cell::Cell;
use core::comm::Port;
use gfx::opts::Opts;
use gfx::render_task::RenderChan;
use gfx::render_task;
use script::compositor_interface::{ScriptListener, ReadyState};
use script::engine_interface::{EngineChan, ExitMsg, LoadUrlMsg, Msg};
use script::layout_interface::LayoutChan;
use script::layout_interface;
use script::script_task::{ExecuteMsg, LoadMsg, ScriptMsg, ScriptContext, ScriptChan};
use script::script_task;
use servo_net::image_cache_task::{ImageCacheTask, ImageCacheTaskClient};
use servo_net::resource_task::ResourceTask;
use servo_net::resource_task;
use servo_util::time::{ProfilerChan};

pub struct Engine {
    request_port: Port<Msg>,
    compositor_chan: CompositorChan,
    render_chan: RenderChan<CompositorChan>,
    resource_task: ResourceTask,
    image_cache_task: ImageCacheTask,
    layout_chan: LayoutChan,
    script_chan: ScriptChan,
    profiler_chan: ProfilerChan,
}

impl Engine {
    pub fn start(compositor_chan: CompositorChan,
                 opts: &Opts,
                 resource_task: ResourceTask,
                 image_cache_task: ImageCacheTask,
                 profiler_chan: ProfilerChan)
                 -> EngineChan {
        macro_rules! closure_stream(
            ($Msg:ty, $Chan:ident) => (
                {
                    let (port, chan) = comm::stream::<$Msg>();
                    (Cell(port), $Chan::new(chan))
                }
            );
        )
            
        // Create the script port and channel.
        let (script_port, script_chan) = closure_stream!(ScriptMsg, ScriptChan);

        // Create the engine port and channel.
        let (engine_port, engine_chan) = closure_stream!(Msg, EngineChan);
        
        // Create the layout port and channel.
        let (layout_port, layout_chan) = closure_stream!(layout_interface::Msg, LayoutChan);

        let (render_port, render_chan) = comm::stream::<render_task::Msg<CompositorChan>>();
        let (render_port, render_chan) = (Cell(render_port), RenderChan::new(render_chan));


        compositor_chan.send(SetLayoutChan(layout_chan.clone()));
        let compositor_chan = Cell(compositor_chan);

        let opts = Cell(copy *opts);

        {
            let engine_chan = engine_chan.clone();
            do task::spawn {
                let compositor_chan = compositor_chan.take();
                render_task::create_render_task(render_port.take(),
                                                compositor_chan.clone(),
                                                opts.with_ref(|o| copy *o),
                                                profiler_chan.clone());

                let opts = opts.take();

                layout_task::create_layout_task(layout_port.take(),
                                                script_chan.clone(),
                                                render_chan.clone(),
                                                image_cache_task.clone(),
                                                opts,
                                                profiler_chan.clone());

                let compositor_chan_clone = compositor_chan.clone();
                ScriptContext::create_script_context(layout_chan.clone(),
                                                     script_port.take(),
                                                     script_chan.clone(),
                                                     engine_chan.clone(),
                                                     |msg: ReadyState| {
                                                         compositor_chan_clone.set_ready_state(msg)
                                                     },
                                                     resource_task.clone(),
                                                     image_cache_task.clone());

                Engine {
                    request_port: engine_port.take(),
                    compositor_chan: compositor_chan.clone(),
                    render_chan: render_chan.clone(),
                    resource_task: resource_task.clone(),
                    image_cache_task: image_cache_task.clone(),
                    layout_chan: layout_chan.clone(),
                    script_chan: script_chan.clone(),
                    profiler_chan: profiler_chan.clone(),
                }.run();
            }
        }
        engine_chan
    }

    fn run(&self) {
        while self.handle_request(self.request_port.recv()) {
            // Go on...
        }
    }

    fn handle_request(&self, request: Msg) -> bool {
        match request {
            LoadUrlMsg(url) => {
                if url.path.ends_with(".js") {
                    self.script_chan.send(ExecuteMsg(url))
                } else {
                    self.script_chan.send(LoadMsg(url))
                }
                return true
            }

            ExitMsg(sender) => {
                self.script_chan.send(script_task::ExitMsg);
                self.layout_chan.send(layout_interface::ExitMsg);

                let (response_port, response_chan) = comm::stream();

                self.render_chan.send(render_task::ExitMsg(response_chan));
                response_port.recv();

                self.image_cache_task.exit();
                self.resource_task.send(resource_task::Exit);

                sender.send(());
                return false
            }
        }
    }
}

