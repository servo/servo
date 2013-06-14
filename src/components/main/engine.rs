/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositing::CompositorTask;
use layout::layout_task;

use core::cell::Cell;
use core::comm::{Port, SharedChan};
use gfx::opts::Opts;
use gfx::render_task::RenderTask;
use gfx::render_task;
use script::compositor_interface::{CompositorInterface, ReadyState};
use script::engine_interface::{EngineTask, ExitMsg, LoadUrlMsg, Msg};
use script::layout_interface::LayoutTask;
use script::layout_interface;
use script::script_task::{ExecuteMsg, LoadMsg, ScriptMsg, ScriptTask};
use script::script_task;
use servo_net::image_cache_task::{ImageCacheTask, ImageCacheTaskClient};
use servo_net::resource_task::ResourceTask;
use servo_net::resource_task;
use servo_util::time::{ProfilerChan, ProfilerPort, ProfilerTask, ForcePrintMsg};

pub struct Engine {
    request_port: Port<Msg>,
    compositor: CompositorTask,
    render_task: RenderTask,
    resource_task: ResourceTask,
    image_cache_task: ImageCacheTask,
    layout_task: LayoutTask,
    script_task: ScriptTask,
    profiler_task: ProfilerTask,
}

impl Drop for Engine {
    fn finalize(&self) {
        self.profiler_task.chan.send(ForcePrintMsg);
    }
}

impl Engine {
    pub fn start(compositor: CompositorTask,
                 opts: &Opts,
                 script_port: Port<ScriptMsg>,
                 script_chan: SharedChan<ScriptMsg>,
                 resource_task: ResourceTask,
                 image_cache_task: ImageCacheTask,
                 profiler_port: ProfilerPort,
                 profiler_chan: ProfilerChan)
                 -> EngineTask {
        let (script_port, script_chan) = (Cell(script_port), Cell(script_chan));
        let (engine_port, engine_chan) = comm::stream();
        let (engine_port, engine_chan) = (Cell(engine_port), SharedChan::new(engine_chan));
        let engine_chan_clone = engine_chan.clone();
        let compositor = Cell(compositor);
        let profiler_port = Cell(profiler_port);
        let opts = Cell(copy *opts);

        do task::spawn {
            let compositor = compositor.take();
            let render_task = RenderTask::new(compositor.clone(),
                                              opts.with_ref(|o| copy *o),
                                              profiler_chan.clone());

            let opts = opts.take();

            let profiler_task = ProfilerTask::new(profiler_port.take(),
                                                  profiler_chan.clone(),
                                                  opts.profiler_period);

            let layout_task = layout_task::create_layout_task(render_task.clone(),
                                                              image_cache_task.clone(),
                                                              opts,
                                                              profiler_task.chan.clone());

            let compositor_clone = compositor.clone();
            let script_task = ScriptTask::new(script_port.take(),
                                              script_chan.take(),
                                              engine_chan_clone.clone(),
                                              |msg: ReadyState| {
                                                  compositor_clone.set_ready_state(msg)
                                              },
                                              layout_task.clone(),
                                              resource_task.clone(),
                                              image_cache_task.clone());


            Engine {
                request_port: engine_port.take(),
                compositor: compositor.clone(),
                render_task: render_task,
                resource_task: resource_task.clone(),
                image_cache_task: image_cache_task.clone(),
                layout_task: layout_task,
                script_task: script_task,
                profiler_task: profiler_task,
            }.run();
        }
        engine_chan.clone()
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
                    self.script_task.chan.send(ExecuteMsg(url))
                } else {
                    self.script_task.chan.send(LoadMsg(url))
                }
                return true
            }

            ExitMsg(sender) => {
                self.script_task.chan.send(script_task::ExitMsg);
                self.layout_task.chan.send(layout_interface::ExitMsg);

                let (response_port, response_chan) = comm::stream();

                self.render_task.channel.send(render_task::ExitMsg(response_chan));
                response_port.recv();

                self.image_cache_task.exit();
                self.resource_task.send(resource_task::Exit);

                sender.send(());
                return false
            }
        }
    }
}

