/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositing::{CompositorChan, SetLayoutRenderChans};

use std::cell::Cell;
use std::comm;
use std::comm::Port;
use std::task;
use gfx::opts::Opts;
use gfx::render_task::{PaintPermissionGranted, PaintPermissionRevoked};
use pipeline::Pipeline;
use servo_msg::constellation_msg::{CompositorAck, ConstellationChan, ExitMsg, LoadUrlMsg};
use servo_msg::constellation_msg::{Msg, NavigateMsg, RendererReadyMsg, ResizedWindowBroadcast};
use servo_msg::constellation_msg;
use script::script_task::{ResizeInactiveMsg, ExecuteMsg};
use servo_net::image_cache_task::{ImageCacheTask, ImageCacheTaskClient};
use servo_net::resource_task::ResourceTask;
use servo_net::resource_task;
use servo_util::time::ProfilerChan;
use std::hashmap::HashMap;
use std::util::replace;

/// Maintains the pipelines and navigation context and grants permission to composite
pub struct Constellation {
    chan: ConstellationChan,
    request_port: Port<Msg>,
    compositor_chan: CompositorChan,
    resource_task: ResourceTask,
    image_cache_task: ImageCacheTask,
    pipelines: HashMap<uint, Pipeline>,
    navigation_context: NavigationContext,
    next_id: uint,
    current_painter: Option<uint>,
    next_painter: Option<uint>,
    profiler_chan: ProfilerChan,
    opts: Opts,
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

    /* Note that the following two methods can fail. They should only be called  *
     * when it is known that, e.g., there exists a previous page or a next page. */

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

    /// Navigates to a new id, returning all id's evicted from next
    pub fn navigate(&mut self, id: uint) -> ~[uint] {
        let evicted = replace(&mut self.next, ~[]);
        do self.current.mutate_default(id) |cur_id| {
            self.previous.push(cur_id);
            id
        }
        evicted
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

        let (constellation_port, constellation_chan) = special_stream!(ConstellationChan);
        let constellation_port = Cell::new(constellation_port);

        let compositor_chan = Cell::new(compositor_chan);
        let constellation_chan_clone = Cell::new(constellation_chan.clone());

        let resource_task = Cell::new(resource_task);
        let image_cache_task = Cell::new(image_cache_task);
        let profiler_chan = Cell::new(profiler_chan);

        do task::spawn {
            let mut constellation = Constellation {
                chan: constellation_chan_clone.take(),
                request_port: constellation_port.take(),
                compositor_chan: compositor_chan.take(),
                resource_task: resource_task.take(),
                image_cache_task: image_cache_task.take(),
                pipelines: HashMap::new(),
                navigation_context: NavigationContext::new(),
                next_id: 0,
                current_painter: None,
                next_painter: None,
                profiler_chan: profiler_chan.take(),
                opts: opts.take(),
            };
            constellation.run();
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
            // Load a new page, usually either from a mouse click or typed url
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
                    pipeline.navigation_type = Some(constellation_msg::Load);
                    self.next_painter = Some(pipeline_id);
                }
                self.pipelines.insert(pipeline_id, pipeline);
            }

            // Handle a forward or back request
            NavigateMsg(direction) => {
                debug!("received message to navigate %?", direction);
                let destination_id = match direction {
                    constellation_msg::Forward => {
                        if self.navigation_context.next.is_empty() {
                            debug!("no next page to navigate to");
                            return true
                        }
                        self.navigation_context.forward()
                    }
                    constellation_msg::Back => {
                        if self.navigation_context.previous.is_empty() {
                            debug!("no previous page to navigate to");
                            return true
                        }
                        self.navigation_context.back()
                    }
                };
                debug!("navigating to pipeline %u", destination_id);
                let mut pipeline = self.pipelines.pop(&destination_id).unwrap();
                pipeline.navigation_type = Some(constellation_msg::Navigate);
                pipeline.reload();
                self.pipelines.insert(destination_id, pipeline);
                self.next_painter = Some(destination_id);
                self.update_painter();
            }

            // Notification that rendering has finished and is requesting permission to paint.
            RendererReadyMsg(pipeline_id) => {
                let next_painter = self.next_painter;
                for next_painter.iter().advance |&id| {
                    if pipeline_id == id {
                        self.update_painter();
                    }
                }
            }

            ResizedWindowBroadcast(new_size) => match self.current_painter {
                Some(current_painter_id) => for self.pipelines.iter().advance |(&id, pipeline)| {
                    if current_painter_id != id {
                        pipeline.script_chan.send(ResizeInactiveMsg(new_size));
                    }
                },
                None => for self.pipelines.iter().advance |(_, pipeline)| {
                    pipeline.script_chan.send(ResizeInactiveMsg(new_size));
                },
            },

            // Acknowledgement from the compositor that it has updated its active pipeline id
            CompositorAck(id) => {
                self.grant_paint_permission(id);
            }

            ExitMsg(sender) => {
                for self.pipelines.iter().advance |(_, pipeline)| {
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
    
    fn update_painter(&mut self) {
        let current_painter = replace(&mut self.current_painter, None);
        for current_painter.iter().advance |id| {
            self.pipelines.get(id).render_chan.send(PaintPermissionRevoked);
        }
        let id = self.next_painter.get();
        let pipeline = self.pipelines.get(&id);
        self.compositor_chan.send(SetLayoutRenderChans(pipeline.layout_chan.clone(),
                                                       pipeline.render_chan.clone(),
                                                       id,
                                                       self.chan.clone()));
    }

    // Grants a renderer permission to paint; optionally updates navigation to reflect a new page
    fn grant_paint_permission(&mut self, id: uint) {
        let pipeline = self.pipelines.get(&id);
        pipeline.render_chan.send(PaintPermissionGranted);
        self.current_painter = Some(id);
        self.next_painter = None;
        // Don't navigate on Navigate type, because that is handled by forward/back
        match pipeline.navigation_type.get() {
            constellation_msg::Load => {
                let _evicted = self.navigation_context.navigate(id);
                /* FIXME(tkuehn): the following code causes a segfault
                for evicted.iter().advance |id| {
                    self.pipelines.get(id).exit();
                }
                */
            }
            _ => {}
        }
    }
}

