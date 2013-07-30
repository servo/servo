/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositing::{CompositorChan, SetIds};

use extra::net::url;

use std::cell::Cell;
use std::comm;
use std::comm::Port;
use std::task;
use geom::size::Size2D;
use gfx::opts::Opts;
use pipeline::Pipeline;
use servo_msg::constellation_msg::{ConstellationChan, ExitMsg};
use servo_msg::constellation_msg::{InitLoadUrlMsg, LoadIframeUrlMsg, LoadUrlMsg};
use servo_msg::constellation_msg::{Msg, NavigateMsg};
use servo_msg::constellation_msg::{PipelineId, RendererReadyMsg, ResizedWindowBroadcast};
use servo_msg::constellation_msg;
use script::script_task::{ResizeInactiveMsg, ExecuteMsg};
use servo_net::image_cache_task::{ImageCacheTask, ImageCacheTaskClient};
use servo_net::resource_task::ResourceTask;
use servo_net::resource_task;
use servo_util::time::ProfilerChan;
use std::hashmap::HashMap;
use std::util::replace;
use extra::future::from_value;

/// Maintains the pipelines and navigation context and grants permission to composite
pub struct Constellation {
    chan: ConstellationChan,
    request_port: Port<Msg>,
    compositor_chan: CompositorChan,
    resource_task: ResourceTask,
    image_cache_task: ImageCacheTask,
    pipelines: HashMap<PipelineId, @mut Pipeline>,
    navigation_context: NavigationContext,
    priv next_pipeline_id: PipelineId,
    pending_frames: ~[FrameChange],
    profiler_chan: ProfilerChan,
    opts: Opts,
}

/// Stores the Id of the outermost frame's pipeline, along with a vector of children frames
#[deriving(Clone)]
struct FrameTree {
    pipeline: @mut Pipeline,
    parent: Option<@mut Pipeline>,
    children: ~[@mut FrameTree],
}

pub struct SendableFrameTree {
    pipeline: Pipeline,
    children: ~[SendableFrameTree],
}

impl SendableFrameTree {
    fn contains(&self, id: PipelineId) -> bool {
        self.pipeline.id == id ||
        do self.children.iter().any |frame_tree| {
            frame_tree.contains(id)
        }
    }
}

impl FrameTree {
    fn contains(&self, id: PipelineId) -> bool {
        self.pipeline.id == id ||
        do self.children.iter().any |frame_tree| {
            frame_tree.contains(id)
        }
    }

    /// Returns the frame tree whose key is id
    fn find_mut(@mut self, id: PipelineId) -> Option<@mut FrameTree> {
        if self.pipeline.id == id { return Some(self); }
        for self.children.mut_iter().advance |frame_tree| {
            let found = frame_tree.find_mut(id);
            if found.is_some() { return found; }
        }
        None
    }

    /// Replaces a node of the frame tree in place. Returns the node that was removed or the original node
    /// if the node to replace could not be found.
    fn replace_child(&mut self, id: PipelineId, new_child: @mut FrameTree) -> Result<@mut FrameTree, @mut FrameTree> {
        let new_child_cell = Cell::new(new_child);
        for self.children.mut_iter().advance |child| {
            let new_child = new_child_cell.take();
            if child.pipeline.id == id {
                new_child.parent = child.parent;
                return Ok(replace(child, new_child));
            } 
            let replaced = child.replace_child(id, new_child);
            if replaced.is_ok() {
                return replaced;
            }
            new_child_cell.put_back(replaced.get_err());
        }
        Err(new_child_cell.take())
    }

    fn to_sendable(&self) -> SendableFrameTree {
        let mut sendable_frame_tree = SendableFrameTree {
            pipeline: (*self.pipeline).clone(),
            children: ~[],
        };

        for self.children.iter().advance |frame_tree| {
            sendable_frame_tree.children.push(frame_tree.to_sendable());
        }
        sendable_frame_tree
    }

    pub fn iter(@mut self) -> FrameTreeIterator {
        FrameTreeIterator {
            stack: ~[self],
        }
    }
}

pub struct FrameTreeIterator {
    priv stack: ~[@mut FrameTree],
}

impl Iterator<@mut FrameTree> for FrameTreeIterator {
    fn next(&mut self) -> Option<@mut FrameTree> {
        if !self.stack.is_empty() {
            let next = self.stack.pop();
            for next.children.iter().advance |&child| {
                self.stack.push(child);
            }
            Some(next)
        } else {
            None
        }
    }
}

/// Represents the portion of a page that is changing in navigating.
struct FrameChange {
    before: Option<PipelineId>,
    after: @mut FrameTree,
}

/// Stores the Id's of the pipelines previous and next in the browser's history
struct NavigationContext {
    previous: ~[@mut FrameTree],
    next: ~[@mut FrameTree],
    current: Option<@mut FrameTree>,
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
     * when it is known that there exists either a previous page or a next page. */

    pub fn back(&mut self) -> @mut FrameTree {
        self.next.push(self.current.swap_unwrap());
        self.current = Some(self.previous.pop());
        debug!("previous: %? next: %? current: %?", self.previous, self.next, *self.current.get_ref());
        self.current.get()
    }

    pub fn forward(&mut self) -> @mut FrameTree {
        self.previous.push(self.current.swap_unwrap());
        self.current = Some(self.next.pop());
        debug!("previous: %? next: %? current: %?", self.previous, self.next, *self.current.get_ref());
        self.current.get()
    }

    /// Loads a new set of page frames, returning all evicted frame trees
    pub fn load(&mut self, frame_tree: @mut FrameTree) -> ~[@mut FrameTree] {
        debug!("navigating to %?", frame_tree);
        let evicted = replace(&mut self.next, ~[]);
        if self.current.is_some() {
            self.previous.push(self.current.swap_unwrap());
        }
        self.current = Some(frame_tree);
        evicted
    }

    /// Returns the frame tree whose key is pipeline_id.
    pub fn find(&mut self, pipeline_id: PipelineId) -> Option<@mut FrameTree> {
        for self.current.mut_iter().advance |frame_tree| {
            let found = frame_tree.find_mut(pipeline_id);
            if found.is_some() { return found; }
        }
        let mut forward =  self.next.mut_rev_iter();
        let mut backward = self.previous.mut_rev_iter();
        loop {
            match (forward.next(), backward.next()) {
                (None, None) => {
                    return None;
                }
                (next_forward, next_backward) => {
                    let mut next_forward = next_forward;
                    let mut next_backward = next_backward;
                    for next_forward.mut_iter().advance |frame_tree| {
                        let found = frame_tree.find_mut(pipeline_id);
                        if found.is_some() { return found; }
                    }
                    for next_backward.mut_iter().advance |frame_tree| {
                        let found = frame_tree.find_mut(pipeline_id);
                        if found.is_some() { return found; }
                    }
                }
            }
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
                next_pipeline_id: PipelineId(0),
                pending_frames: ~[],
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

    /// Helper function for getting a unique pipeline Id
    fn get_next_pipeline_id(&mut self) -> PipelineId {
        let id = self.next_pipeline_id;
        self.next_pipeline_id = PipelineId(*id + 1);
        id
    }
    
    /// Convenience function for getting the currently active frame tree.
    /// The currently active frame tree should always be the current painter
    fn current_frame<'a>(&'a self) -> &'a Option<@mut FrameTree> {
        &self.navigation_context.current
    }

    /// Handles loading pages, navigation, and granting access to the compositor
    fn handle_request(&mut self, request: Msg) -> bool {
        match request {

            ExitMsg(sender) => {
                for self.pipelines.iter().advance |(_id, ref pipeline)| {
                    pipeline.exit();
                }
                self.image_cache_task.exit();
                self.resource_task.send(resource_task::Exit);

                sender.send(());
                return false
            }
            
            // This should only be called once per constellation, and only by the browser
            InitLoadUrlMsg(url) => {
                let pipeline = @mut Pipeline::create(self.get_next_pipeline_id(),
                                                    self.chan.clone(),
                                                    self.compositor_chan.clone(),
                                                    self.image_cache_task.clone(),
                                                    self.resource_task.clone(),
                                                    self.profiler_chan.clone(),
                                                    copy self.opts,
                                                    {
                                                        let size = self.compositor_chan.get_size();
                                                        from_value(Size2D(size.width as uint, size.height as uint))
                                                    });
                if url.path.ends_with(".js") {
                    pipeline.script_chan.send(ExecuteMsg(pipeline.id, url));
                } else {
                    pipeline.load(url, Some(constellation_msg::Load));

                    self.pending_frames.push(FrameChange{
                        before: None,
                        after: @mut FrameTree {
                            pipeline: pipeline, 
                            parent: None,
                            children: ~[],
                        },
                    });
                }
                self.pipelines.insert(pipeline.id, pipeline);
            }

            LoadIframeUrlMsg(url, source_pipeline_id, size_future) => {
                // A message from the script associated with pipeline_id that it has
                // parsed an iframe during html parsing. This iframe will result in a
                // new pipeline being spawned and a frame tree being added to pipeline_id's
                // frame tree's children. This message is never the result of a link clicked
                // or a new url entered.
                //     Start by finding the frame tree matching the pipeline id,
                // and add the new pipeline to its sub frames. The frame tree
                // could already be in the navigation context, or it could still
                // be loading, in which case it is a new id in a constellation.pending_frames
                // frame change, because pages aren't added to the navi context until they finish
                // loading (excluding iframes). Which is checked first is seemingly arbitrary
                let next_pipeline_id = self.get_next_pipeline_id();
                let frame_tree = {
                    let frame_tree = self.navigation_context.find(source_pipeline_id);
                    match frame_tree {
                        Some(frame_tree) => frame_tree,
                        None => {
                            let frame_change = do self.pending_frames.mut_iter().find_ |frame_change| {
                                frame_change.after.pipeline.id == source_pipeline_id
                            };
                            let frame_change = frame_change.expect("Constellation: source pipeline id
                                of LoadIframeUrlMsg is not in navigation context nor a pending painter.
                                This should be impossible.");
                            frame_change.after
                        }
                    }
                };

                // Compare the pipeline's url to the new url. If the origin is the same,
                // then reuse the script task in creating the new pipeline
                let source_pipeline = *self.pipelines.find(&source_pipeline_id).expect("Constellation:
                    source Id of LoadIframeUrlMsg does have an associated pipeline in
                    constellation. This should be impossible.");

                let source_url = source_pipeline.url.clone().expect("Constellation: LoadUrlIframeMsg's
                source's Url is None. There should never be a LoadUrlIframeMsg from a pipeline
                that was never given a url to load.");

                // FIXME(tkuehn): Need to follow the standardized spec for checking same-origin
                let pipeline = @mut if (source_url.host == url.host &&
                                       source_url.port == url.port) {
                    // Reuse the script task if same-origin url's
                    Pipeline::with_script(next_pipeline_id,
                                          self.chan.clone(),
                                          self.compositor_chan.clone(),
                                          self.image_cache_task.clone(),
                                          self.profiler_chan.clone(),
                                          copy self.opts,
                                          source_pipeline,
                                          size_future)
                } else {
                    // Create a new script task if not same-origin url's
                    Pipeline::create(next_pipeline_id,
                                     self.chan.clone(),
                                     self.compositor_chan.clone(),
                                     self.image_cache_task.clone(),
                                     self.resource_task.clone(),
                                     self.profiler_chan.clone(),
                                     copy self.opts,
                                     size_future)
                };

                if url.path.ends_with(".js") {
                    pipeline.execute(url);
                } else {
                    pipeline.load(url, None);
                }
                frame_tree.children.push(@mut FrameTree {
                    pipeline: pipeline,
                    parent: Some(source_pipeline),
                    children: ~[],
                });
                self.pipelines.insert(pipeline.id, pipeline);
            }

            // Load a new page, usually -- but not always -- from a mouse click or typed url
            // If there is already a pending page (self.pending_frames), it will not be overridden;
            // However, if the id is not encompassed by another change, it will be.
            LoadUrlMsg(source_id, url, size_future) => {
                debug!("received message to load %s", url::to_str(&url));
                // Make sure no pending page would be overridden.
                for self.pending_frames.iter().advance |frame_change| {
                    let old_id = frame_change.before.expect("Constellation: Received load msg
                        from pipeline, but there is no currently active page. This should
                        be impossible.");
                    let changing_frame = self.current_frame().get_ref().find_mut(old_id).expect("Constellation:
                        Pending change has non-active source pipeline. This should be
                        impossible.");
                    if changing_frame.contains(source_id) {
                        // id that sent load msg is being changed already; abort
                        return true;
                    }
                }
                // Being here means either there are no pending frames, or none of the pending
                // changes would be overriden by changing the subframe associated with source_id.

                let next_pipeline_id = self.get_next_pipeline_id();
                let pipeline = @mut Pipeline::create(next_pipeline_id,
                                                    self.chan.clone(),
                                                    self.compositor_chan.clone(),
                                                    self.image_cache_task.clone(),
                                                    self.resource_task.clone(),
                                                    self.profiler_chan.clone(),
                                                    copy self.opts,
                                                    size_future);

                if url.path.ends_with(".js") {
                    pipeline.script_chan.send(ExecuteMsg(pipeline.id, url));
                } else {
                    pipeline.load(url, Some(constellation_msg::Load));

                    let parent = if self.current_frame().get_ref().pipeline.id == source_id {
                        // source_id is the root of the current frame tree; replace whole tree
                        None
                    } else {
                        // id is not the root of the current frame tree, but is in the frame tree;
                        // replace only the subtree
                        let source_frame = self.current_frame().get_ref().find_mut(source_id).expect(
                            "Constellation: received a LoadUrlMsg from a pipeline_id associated
                            with a pipeline not in the active frame tree. This should be
                            impossible.");
                        source_frame.parent
                    };
                    self.pending_frames.push(FrameChange{
                        before: Some(source_id),
                        after: @mut FrameTree {
                            pipeline: pipeline, 
                            parent: parent,
                            children: ~[],
                        },
                    });
                }
                self.pipelines.insert(pipeline.id, pipeline);
            }

            // Handle a forward or back request
            NavigateMsg(direction) => {
                debug!("received message to navigate %?", direction);

                // TODO(tkuehn): what is the "critical point" beyond which pending frames
                // should not be cleared? Currently, the behavior is that forward/back
                // navigation always has navigation priority, and after that new page loading is
                // first come, first served.
                let destination_frame = match direction {
                    constellation_msg::Forward => {
                        if self.navigation_context.next.is_empty() {
                            debug!("no next page to navigate to");
                            return true
                        } else {
                            let old = self.current_frame().get_ref();
                            for old.iter().advance |frame| {
                                frame.pipeline.revoke_paint_permission();
                            }
                        }
                        self.navigation_context.forward()
                    }
                    constellation_msg::Back => {
                        if self.navigation_context.previous.is_empty() {
                            debug!("no previous page to navigate to");
                            return true
                        } else {
                            let old = self.current_frame().get_ref();
                            for old.iter().advance |frame| {
                                frame.pipeline.revoke_paint_permission();
                            }
                        }
                        self.navigation_context.back()
                    }
                };

                for destination_frame.iter().advance |frame| {
                    let pipeline = &frame.pipeline;
                    pipeline.reload(Some(constellation_msg::Navigate));
                }
                self.grant_paint_permission(destination_frame);

            }

            // Notification that rendering has finished and is requesting permission to paint.
            RendererReadyMsg(pipeline_id) => {
                // This message could originate from a pipeline in the navigation context or
                // from a pending frame. The only time that we will grant paint permission is
                // when the message originates from a pending frame or the current frame.

                for self.current_frame().iter().advance |&current_frame| {
                    // Messages originating in the current frame are not navigations;
                    // TODO(tkuehn): In fact, this kind of message might be provably
                    // impossible to occur.
                    if current_frame.contains(pipeline_id) {
                        self.set_ids(current_frame);
                        return true;
                    }
                }

                // Find the pending frame change whose new pipeline id is pipeline_id.
                // If it is not found, it simply means that this pipeline will not receive
                // permission to paint.
                let pending_index = do self.pending_frames.rposition |frame_change| {
                    frame_change.after.pipeline.id == pipeline_id
                };
                for pending_index.iter().advance |&pending_index| {
                    let frame_change = self.pending_frames.swap_remove(pending_index);
                    let to_add = frame_change.after;

                    // Create the next frame tree that will be given to the compositor
                    let next_frame_tree = match to_add.parent {
                        None => to_add, // to_add is the root
                        Some(_parent) => self.current_frame().get(),
                    };

                    // If there are frames to revoke permission from, do so now.
                    match frame_change.before {
                        Some(revoke_id) => {
                            let current_frame = self.current_frame().get();

                            let to_revoke = current_frame.find_mut(revoke_id).expect(
                                "Constellation: pending frame change refers to an old
                                frame not contained in the current frame. This is a bug");

                            for to_revoke.iter().advance |frame| {
                                frame.pipeline.revoke_paint_permission();
                            }

                            // If to_add is not the root frame, then replace revoked_frame with it
                            if to_add.parent.is_some() {
                                next_frame_tree.replace_child(revoke_id, to_add);
                            }
                        }

                        None => {
                            // Add to_add to parent's children, if it is not the root
                            let parent = &to_add.parent;
                            let to_add = Cell::new(to_add);
                            for parent.iter().advance |parent| {
                                let parent = next_frame_tree.find_mut(parent.id).expect(
                                    "Constellation: pending frame has a parent frame that is not
                                    active. This is a bug.");
                                parent.children.push(to_add.take());
                            }
                        }
                    }
                self.grant_paint_permission(next_frame_tree);
                }
            }

            ResizedWindowBroadcast(new_size) => match *self.current_frame() {
                Some(ref current_frame) => {
                    let current_frame_id = current_frame.pipeline.id.clone();
                    for self.navigation_context.previous.iter().advance |frame_tree| {
                        let pipeline = &frame_tree.pipeline;
                        if current_frame_id != pipeline.id {
                            pipeline.script_chan.send(ResizeInactiveMsg(new_size));
                        }
                    }
                    for self.navigation_context.next.iter().advance |frame_tree| {
                        let pipeline = &frame_tree.pipeline;
                        if current_frame_id != pipeline.id {
                            pipeline.script_chan.send(ResizeInactiveMsg(new_size));
                        }
                    }
                }
                None => {
                    for self.navigation_context.previous.iter().advance |frame_tree| {
                        frame_tree.pipeline.script_chan.send(ResizeInactiveMsg(new_size));
                    }
                    for self.navigation_context.next.iter().advance |frame_tree| {
                        frame_tree.pipeline.script_chan.send(ResizeInactiveMsg(new_size));
                    }
                }
            }

        }
        true
    }
    
    // Grants a frame tree permission to paint; optionally updates navigation to reflect a new page
    fn grant_paint_permission(&mut self, frame_tree: @mut FrameTree) {
        // Give permission to paint to the new frame and all child frames
        self.set_ids(frame_tree);

        // Don't call navigation_context.load() on a Navigate type (or None, as in the case of
        // parsed iframes that finish loading)
        match frame_tree.pipeline.navigation_type {
            Some(constellation_msg::Load) => {
                let evicted = self.navigation_context.load(frame_tree);
                for evicted.iter().advance |frame_tree| {
                    // exit any pipelines that don't exist outside the evicted frame trees
                    for frame_tree.iter().advance |frame| {
                        if self.navigation_context.find(frame.pipeline.id).is_none() {
                            frame_tree.pipeline.exit();
                            self.pipelines.remove(&frame_tree.pipeline.id);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn set_ids(&self, frame_tree: @mut FrameTree) {
        let (port, chan) = comm::stream();
        self.compositor_chan.send(SetIds(frame_tree.to_sendable(), chan));
        port.recv();
        for frame_tree.iter().advance |frame| {
            frame.pipeline.grant_paint_permission();
        }
    }
}

