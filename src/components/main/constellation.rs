/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositing::{CompositorChan, SetLayoutChan, SetRenderChan};

use std::cell::Cell;
use std::comm;
use std::comm::Port;
use std::task;
use gfx::opts::Opts;
use gfx::render_task::{TokenBestowMsg, TokenProcureMsg};
use pipeline::Pipeline;
use servo_msg::compositor::{CompositorToken};
use servo_msg::constellation::{ConstellationChan, ExitMsg, LoadUrlMsg, Msg, RendererReadyMsg};
use servo_msg::constellation::TokenSurrenderMsg;
use script::script_task::{ExecuteMsg, LoadMsg};
use servo_net::image_cache_task::{ImageCacheTask, ImageCacheTaskClient};
use servo_net::resource_task::ResourceTask;
use servo_net::resource_task;
use servo_util::time::ProfilerChan;
use std::hashmap::HashMap;

pub struct Constellation {
    chan: ConstellationChan,
    request_port: Port<Msg>,
    compositor_chan: CompositorChan,
    resource_task: ResourceTask,
    image_cache_task: ImageCacheTask,
    pipelines: HashMap<uint, Pipeline>,
    navigation_context: NavigationContext,
    next_id: uint,
    current_token_holder: Option<uint>,
    loading: Option<uint>,
    profiler_chan: ProfilerChan,
    opts: Opts,
}

pub struct NavigationContext {
    previous: ~[uint],
    next: ~[uint],
    current: Option<uint>,
}

impl NavigationContext {
    pub fn new() -> NavigationContext {
        NavigationContext {
            previous: ~[],
            next: ~[],
            current: None,
        }
    }

    pub fn back(&mut self) -> uint {
        do self.current.mutate |id| {
            self.next.push(id);
            self.previous.pop()
        }
        self.current.get()
    }

    pub fn forward(&mut self) -> uint {
        do self.current.mutate |id| {
            self.previous.push(id);
            self.next.pop()
        }
        self.current.get()
    }

    pub fn navigate(&mut self, id: uint) {
        do self.current.mutate_default(id) |cur_id| {
            self.previous.push(cur_id);
            id
        }
    }
}

impl Constellation {
    pub fn start(compositor_chan: CompositorChan,
                 opts: &Opts,
                 resource_task: ResourceTask,
                 image_cache_task: ImageCacheTask,
                 profiler_chan: ProfilerChan)
                 -> ConstellationChan {
            
        let opts = Cell::new(copy *opts);

        let (constellation_port, constellation_chan) = comm::stream();
        let (constellation_port, constellation_chan) = (Cell::new(constellation_port),
                                                        ConstellationChan::new(constellation_chan));

        let compositor_chan = Cell::new(compositor_chan);
        let constellation_chan_clone = Cell::new(constellation_chan.clone());
        {
            do task::spawn {
                let mut constellation = Constellation {
                    chan: constellation_chan_clone.take(),
                    request_port: constellation_port.take(),
                    compositor_chan: compositor_chan.take(),
                    resource_task: resource_task.clone(),
                    image_cache_task: image_cache_task.clone(),
                    pipelines: HashMap::new(),
                    navigation_context: NavigationContext::new(),
                    next_id: 0,
                    current_token_holder: None,
                    loading: None,
                    profiler_chan: profiler_chan.clone(),
                    opts: opts.take(),
                };
                constellation.run();
            }
        }
        constellation_chan
    }

    fn run(&mut self) {
        loop {
            let request = self.request_port.recv();
            if !self.handle_request(request) {
                break;
            }
        }
    }

    fn get_next_id(&mut self) -> uint {
        let id = self.next_id;
        self.next_id = id + 1;
        id
    }

    fn handle_request(&mut self, request: Msg) -> bool {
        match request {
            LoadUrlMsg(url) => {
                let pipeline_id = self.get_next_id();
                let pipeline = Pipeline::create(pipeline_id,
                                                self.chan.clone(),
                                                self.compositor_chan.clone(),
                                                self.image_cache_task.clone(),
                                                self.resource_task.clone(),
                                                self.profiler_chan.clone(),
                                                copy self.opts);
                if url.path.ends_with(".js") {
                    pipeline.script_chan.send(ExecuteMsg(url));
                } else {
                    pipeline.script_chan.send(LoadMsg(url));
                    self.loading = Some(pipeline_id);
                }
                self.pipelines.insert(pipeline_id, pipeline);
            }

            RendererReadyMsg(pipeline_id) => {
                let loading = self.loading.clone();
                do loading.map() |&id| {
                    if pipeline_id == id {
                        match self.current_token_holder {
                            Some(ref id) => {
                                let current_holder = self.pipelines.get(id);
                                current_holder.render_chan.send(TokenProcureMsg);
                            }
                            None => self.bestow_compositor_token(id, ~CompositorToken::new())
                        }
                    }
                };
            }

            TokenSurrenderMsg(token) => {
                self.remove_active_pipeline();
                let token = Cell::new(token);
                let loading = self.loading;
                do loading.map |&id| {
                    self.bestow_compositor_token(id, token.take());
                };
            }

            ExitMsg(sender) => {
                for self.pipelines.each |_, pipeline| {
                    pipeline.exit();
                }
                self.image_cache_task.exit();
                self.resource_task.send(resource_task::Exit);

                sender.send(());
                return false
            }
        }
        true
    }
    
    fn remove_active_pipeline(&mut self) {
// FIXME(tkuehn): currently, pipelines are not removed at all
//        do self.current_token_holder.map |id| {
//            self.pipelines.pop(id).unwrap().exit();
//        };

        self.current_token_holder = None;
    }

    fn bestow_compositor_token(&mut self, id: uint, compositor_token: ~CompositorToken) {
        let pipeline = self.pipelines.get(&id);
        pipeline.render_chan.send(TokenBestowMsg(compositor_token));
        self.compositor_chan.send(SetLayoutChan(pipeline.layout_chan.clone()));
        self.compositor_chan.send(SetRenderChan(pipeline.render_chan.clone()));
        self.current_token_holder = Some(id);
        self.loading = None;
        self.navigation_context.navigate(id);
    }
}

