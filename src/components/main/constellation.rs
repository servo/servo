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
use servo_msg::constellation::{ConstellationChan, ExitMsg, LoadUrlMsg, Msg, NavigateMsg};
use servo_msg::constellation::{Forward, Back, RendererReadyMsg, TokenSurrenderMsg};
use script::script_task::ExecuteMsg;
use servo_net::image_cache_task::{ImageCacheTask, ImageCacheTaskClient};
use servo_net::resource_task::ResourceTask;
use servo_net::resource_task;
use servo_util::time::ProfilerChan;
use std::hashmap::HashMap;
use std::util::replace;

pub struct Constellation {
    chan: ConstellationChan,
    request_port: Port<Msg>,
    compositor_chan: CompositorChan,
    resource_task: ResourceTask,
    image_cache_task: ImageCacheTask,
    pipelines: HashMap<uint, Pipeline>,
    navigation_context: NavigationContext,
    next_id: uint,
    current_token_bearer: Option<uint>,
    next_token_bearer: Option<(uint, NavigationType)>,
    compositor_token: Option<~CompositorToken>,
    profiler_chan: ProfilerChan,
    opts: Opts,
}

/// Represents the two different ways to which a page can be navigated
enum NavigationType {
    Load,               // entered or clicked on a url
    Navigate,           // browser forward/back buttons
}

/// Stores the ID's of the pipelines previous and next in the browser's history
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
        self.next.push(self.current.get());
        self.current = Some(self.previous.pop());
        debug!("previous: %? next: %? current: %?", self.previous, self.next, self.current);
        self.current.get()
    }

    pub fn forward(&mut self) -> uint {
        self.previous.push(self.current.get());
        self.current = Some(self.next.pop());
        debug!("previous: %? next: %? current: %?", self.previous, self.next, self.current);
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
                    current_token_bearer: None,
                    next_token_bearer: None,
                    compositor_token: Some(~CompositorToken::new()),
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

    /// Helper function for getting a unique pipeline ID
    fn get_next_id(&mut self) -> uint {
        let id = self.next_id;
        self.next_id = id + 1;
        id
    }

    /// Handles loading pages, navigation, and granting access to the compositor
    fn handle_request(&mut self, request: Msg) -> bool {
        match request {
            LoadUrlMsg(url) => {
                let pipeline_id = self.get_next_id();
                let mut pipeline = Pipeline::create(pipeline_id,
                                                self.chan.clone(),
                                                self.compositor_chan.clone(),
                                                self.image_cache_task.clone(),
                                                self.resource_task.clone(),
                                                self.profiler_chan.clone(),
                                                copy self.opts);
                if url.path.ends_with(".js") {
                    pipeline.script_chan.send(ExecuteMsg(url));
                } else {
                    pipeline.load(url);
                    self.next_token_bearer = Some((pipeline_id, Load));
                }
                self.pipelines.insert(pipeline_id, pipeline);
            }

            NavigateMsg(direction) => {
                debug!("received message to navigate %?", direction);
                let destination_id = match direction {
                    Forward => {
                        if self.navigation_context.next.is_empty() {
                            debug!("no next page to navigate to");
                            return true
                        }
                        self.navigation_context.forward()
                    }
                    Back => {
                        if self.navigation_context.previous.is_empty() {
                            debug!("no previous page to navigate to");
                            return true
                        }
                        self.navigation_context.back()
                    }
                };
                debug!("navigating to pipeline %u", destination_id);
                self.pipelines.get(&destination_id).reload();
                self.next_token_bearer = Some((destination_id, Navigate));
                self.procure_or_bestow();
            }

            RendererReadyMsg(pipeline_id) => {
                let next_token_bearer = self.next_token_bearer;
                for next_token_bearer.iter().advance |&(id, _)| {
                    if pipeline_id == id {
                        self.procure_or_bestow();
                    }
                };
            }

            TokenSurrenderMsg(token) => {
                self.remove_active_pipeline();
                let token = Cell::new(token);
                let next_token_bearer = self.next_token_bearer;
                for next_token_bearer.iter().advance |&(id, nav_type)| {
                    self.bestow_compositor_token(id, token.take(), nav_type);
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
    
    /// Either procures the token, sends the token to next bearer, or does nothing if waiting for token surrender.
    fn procure_or_bestow(&mut self) {
        let current_token_bearer = replace(&mut self.current_token_bearer, None);
        match current_token_bearer {
            Some(ref id) => {
                let pipeline = self.pipelines.get(id);
                pipeline.render_chan.send(TokenProcureMsg);
            }
            None => {
                let compositor_token = replace(&mut self.compositor_token, None);
                for compositor_token.iter().advance |&token| {
                    let (id, nav_type) = self.next_token_bearer.get();
                    self.bestow_compositor_token(id, token, nav_type);
                }
            }
        };
    }

    fn remove_active_pipeline(&mut self) {
// FIXME(tkuehn): currently, pipelines are not removed at all
//        do self.current_token_bearer.map |id| {
//            self.pipelines.pop(id).unwrap().exit();
//        };

        self.current_token_bearer = None;
    }

    fn bestow_compositor_token(&mut self, id: uint, compositor_token: ~CompositorToken, navigation_type: NavigationType) {
        let pipeline = self.pipelines.get(&id);
        pipeline.render_chan.send(TokenBestowMsg(compositor_token));
        self.compositor_chan.send(SetLayoutChan(pipeline.layout_chan.clone()));
        self.compositor_chan.send(SetRenderChan(pipeline.render_chan.clone()));
        self.current_token_bearer = Some(id);
        self.next_token_bearer = None;
        // Don't navigate on Navigate type, because that is handled by forward/back
        match navigation_type {
            Load => self.navigation_context.navigate(id),
            _ => {}
        }
    }
}

