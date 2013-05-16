/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositing::CompositorImpl;
use dom::event::Event;
use layout::layout_task::LayoutTask;
use layout::layout_task;
use scripting::script_task::{ExecuteMsg, LoadMsg, ScriptTask};
use scripting::script_task;
use util::task::spawn_listener;

use core::cell::Cell;
use core::comm::{Chan, Port, SharedChan};
use gfx::opts::Opts;
use gfx::render_task::RenderTask;
use gfx::render_task;
use servo_net::image_cache_task::{ImageCacheTask, ImageCacheTaskClient};
use servo_net::resource_task::ResourceTask;
use servo_net::resource_task;
use std::net::url::Url;

pub type EngineTask = Chan<Msg>;

pub enum Msg {
    LoadUrlMsg(Url),
    ExitMsg(Chan<()>)
}

pub struct Engine {
    request_port: Port<Msg>,
    compositor: CompositorImpl,
    render_task: RenderTask,
    resource_task: ResourceTask,
    image_cache_task: ImageCacheTask,
    layout_task: LayoutTask,
    script_task: ScriptTask,
}

impl Engine {
    pub fn start(compositor: CompositorImpl,
                 opts: &Opts,
                 dom_event_port: Port<Event>,
                 dom_event_chan: SharedChan<Event>,
                 resource_task: ResourceTask,
                 image_cache_task: ImageCacheTask)
                 -> EngineTask {
        let dom_event_port = Cell(dom_event_port);
        let dom_event_chan = Cell(dom_event_chan);

        let opts = Cell(copy *opts);
        do spawn_listener::<Msg> |request| {
            let render_task = RenderTask(compositor.clone(), opts.with_ref(|o| copy *o));

            let opts = opts.take();
            let layout_task = LayoutTask(render_task.clone(), image_cache_task.clone(), opts);

            let script_task = ScriptTask::new(layout_task.clone(),
                                              dom_event_port.take(),
                                              dom_event_chan.take(),
                                              resource_task.clone(),
                                              image_cache_task.clone());

            Engine {
                request_port: request,
                compositor: compositor.clone(),
                render_task: render_task,
                resource_task: resource_task.clone(),
                image_cache_task: image_cache_task.clone(),
                layout_task: layout_task,
                script_task: script_task,
            }.run()
        }
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
                self.layout_task.send(layout_task::ExitMsg);

                let (response_port, response_chan) = comm::stream();

                self.render_task.send(render_task::ExitMsg(response_chan));
                response_port.recv();

                self.image_cache_task.exit();
                self.resource_task.send(resource_task::Exit);

                sender.send(());
                return false
            }
        }
    }
}

