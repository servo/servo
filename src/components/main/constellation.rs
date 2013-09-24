/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositing::{CompositorChan, SetIds, SetLayerClipRect};

use std::comm;
use std::comm::Port;
use std::task::spawn_with;
use geom::size::Size2D;
use geom::rect::Rect;
use gfx::opts::Opts;
use pipeline::Pipeline;
use servo_msg::constellation_msg::{ConstellationChan, ExitMsg, FailureMsg, FrameRectMsg};
use servo_msg::constellation_msg::{IFrameSandboxState, InitLoadUrlMsg, LoadIframeUrlMsg, LoadUrlMsg};
use servo_msg::constellation_msg::{Msg, NavigateMsg, NavigationType, IFrameUnsandboxed};
use servo_msg::constellation_msg::{PipelineId, RendererReadyMsg, ResizedWindowMsg, SubpageId};
use servo_msg::constellation_msg;
use script::script_task::{ResizeMsg, ResizeInactiveMsg, ExecuteMsg};
use servo_net::image_cache_task::{ImageCacheTask, ImageCacheTaskClient};
use servo_net::resource_task::ResourceTask;
use servo_net::resource_task;
use servo_util::time::ProfilerChan;
use servo_util::url::make_url;
use std::hashmap::{HashMap, HashSet};
use std::util::replace;
use extra::url::Url;
use extra::future::{Future, from_value};

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
    pending_sizes: HashMap<(PipelineId, SubpageId), Rect<f32>>,
    profiler_chan: ProfilerChan,
    opts: Opts,
}

/// Stores the Id of the outermost frame's pipeline, along with a vector of children frames
struct FrameTree {
    pipeline: @mut Pipeline,
    parent: Option<@mut Pipeline>,
    children: ~[ChildFrameTree],
}

// Need to clone the FrameTrees, but _not_ the Pipelines
impl Clone for FrameTree {
    fn clone(&self) -> FrameTree {
        let mut children = do self.children.iter().map |child_frame_tree| {
            child_frame_tree.clone()
        };
        FrameTree {
            pipeline: self.pipeline,
            parent: self.parent.clone(),
            children: children.collect(),
        }
    }
}

struct ChildFrameTree {
    frame_tree: @mut FrameTree,
    /// Clipping rect representing the size and position, in page coordinates, of the visible
    /// region of the child frame relative to the parent.
    rect: Option<Rect<f32>>,
}

impl Clone for ChildFrameTree {
    fn clone(&self) -> ChildFrameTree {
        ChildFrameTree {
            frame_tree: @mut (*self.frame_tree).clone(),
            rect: self.rect.clone(),
        }
    }
}

pub struct SendableFrameTree {
    pipeline: Pipeline,
    children: ~[SendableChildFrameTree],
}

pub struct SendableChildFrameTree {
    frame_tree: SendableFrameTree,
    rect: Option<Rect<f32>>,
}

impl SendableFrameTree {
    fn contains(&self, id: PipelineId) -> bool {
        self.pipeline.id == id ||
        do self.children.iter().any |&SendableChildFrameTree { frame_tree: ref frame_tree, _ }| {
            frame_tree.contains(id)
        }
    }
}

impl FrameTree {
    fn contains(@mut self, id: PipelineId) -> bool {
        do self.iter().any |frame_tree| {
            id == frame_tree.pipeline.id
        }
    }

    /// Returns the frame tree whose key is id
    fn find(@mut self, id: PipelineId) -> Option<@mut FrameTree> {
        do self.iter().find |frame_tree| {
            id == frame_tree.pipeline.id
        }
    }

    /// Replaces a node of the frame tree in place. Returns the node that was removed or the original node
    /// if the node to replace could not be found.
    fn replace_child(@mut self, id: PipelineId, new_child: @mut FrameTree) -> Either<@mut FrameTree, @mut FrameTree> {
        for frame_tree in self.iter() {
            let mut child = frame_tree.children.mut_iter()
                .find(|child| child.frame_tree.pipeline.id == id);
            for child in child.mut_iter() {
                new_child.parent = child.frame_tree.parent;
                return Left(replace(&mut child.frame_tree, new_child));
            }
        }
        Right(new_child)
    }

    fn to_sendable(&self) -> SendableFrameTree {
        let sendable_frame_tree = SendableFrameTree {
            pipeline: (*self.pipeline).clone(),
            children: self.children.iter().map(|frame_tree| frame_tree.to_sendable()).collect(),
        };
        sendable_frame_tree
    }

    pub fn iter(@mut self) -> FrameTreeIterator {
        FrameTreeIterator {
            stack: ~[self],
        }
    }
}

impl ChildFrameTree {
    fn to_sendable(&self) -> SendableChildFrameTree {
        SendableChildFrameTree {
            frame_tree: self.frame_tree.to_sendable(),
            rect: self.rect,
        }
    }
}

/// An iterator over a frame tree, returning nodes in depth-first order.
/// Note that this iterator should _not_ be used to mutate nodes _during_
/// iteration. Mutating nodes once the iterator is out of scope is OK.
pub struct FrameTreeIterator {
    priv stack: ~[@mut FrameTree],
}

impl Iterator<@mut FrameTree> for FrameTreeIterator {
    fn next(&mut self) -> Option<@mut FrameTree> {
        if !self.stack.is_empty() {
            let next = self.stack.pop();
            for &ChildFrameTree { frame_tree, _ } in next.children.rev_iter() {
                self.stack.push(frame_tree);
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
    navigation_type: NavigationType,
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
        self.next.push(self.current.take_unwrap());
        self.current = Some(self.previous.pop());
        self.current.unwrap()
    }

    pub fn forward(&mut self) -> @mut FrameTree {
        self.previous.push(self.current.take_unwrap());
        self.current = Some(self.next.pop());
        self.current.unwrap()
    }

    /// Loads a new set of page frames, returning all evicted frame trees
    pub fn load(&mut self, frame_tree: @mut FrameTree) -> ~[@mut FrameTree] {
        debug!("navigating to %?", frame_tree.pipeline.id);
        let evicted = replace(&mut self.next, ~[]);
        if self.current.is_some() {
            self.previous.push(self.current.take_unwrap());
        }
        self.current = Some(frame_tree);
        evicted
    }

    /// Returns the frame trees whose keys are pipeline_id.
    pub fn find_all(&mut self, pipeline_id: PipelineId) -> ~[@mut FrameTree] {
        let from_current = do self.current.iter().filter_map |frame_tree| {
            frame_tree.find(pipeline_id)
        };
        let from_next =  do self.next.iter().filter_map |frame_tree| {
            frame_tree.find(pipeline_id)
        };
        let from_prev = do self.previous.iter().filter_map |frame_tree| {
            frame_tree.find(pipeline_id)
        };
        from_prev.chain(from_current).chain(from_next).collect()
    }

    pub fn contains(&mut self, pipeline_id: PipelineId) -> bool {
        let from_current = self.current.iter();
        let from_next = self.next.iter();
        let from_prev = self.previous.iter();

        let mut all_contained = from_prev.chain(from_current).chain(from_next);
        do all_contained.any |frame_tree| {
            frame_tree.contains(pipeline_id)
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
            
        let (constellation_port, constellation_chan) = special_stream!(ConstellationChan);
        do spawn_with((constellation_port, constellation_chan.clone(),
                       compositor_chan, resource_task, image_cache_task,
                       profiler_chan, opts.clone()))
            |(constellation_port, constellation_chan, compositor_chan, resource_task,
              image_cache_task, profiler_chan, opts)| {
            let mut constellation = Constellation {
                chan: constellation_chan,
                request_port: constellation_port,
                compositor_chan: compositor_chan,
                resource_task: resource_task,
                image_cache_task: image_cache_task,
                pipelines: HashMap::new(),
                navigation_context: NavigationContext::new(),
                next_pipeline_id: PipelineId(0),
                pending_frames: ~[],
                pending_sizes: HashMap::new(),
                profiler_chan: profiler_chan,
                opts: opts
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
        *self.next_pipeline_id += 1;
        id
    }
    
    /// Convenience function for getting the currently active frame tree.
    /// The currently active frame tree should always be the current painter
    fn current_frame<'a>(&'a self) -> &'a Option<@mut FrameTree> {
        &self.navigation_context.current
    }

    /// Returns both the navigation context and pending frame trees whose keys are pipeline_id.
    pub fn find_all(&mut self, pipeline_id: PipelineId) -> ~[@mut FrameTree] {
        let matching_navi_frames = self.navigation_context.find_all(pipeline_id);
        let matching_pending_frames = do self.pending_frames.iter().filter_map |frame_change| {
            frame_change.after.find(pipeline_id)
        };
        matching_navi_frames.move_iter().chain(matching_pending_frames).collect()
    }

    /// Handles loading pages, navigation, and granting access to the compositor
    fn handle_request(&mut self, request: Msg) -> bool {
        match request {
            ExitMsg(sender) => {
                self.handle_exit(sender);
                return false;
            }
            FailureMsg(pipeline_id, subpage_id) => {
                self.handle_failure_msg(pipeline_id, subpage_id);
            }
            // This should only be called once per constellation, and only by the browser
            InitLoadUrlMsg(url) => {
                self.handle_init_load(url);
            }
            // A layout assigned a size and position to a subframe. This needs to be reflected by all
            // frame trees in the navigation context containing the subframe.
            FrameRectMsg(pipeline_id, subpage_id, rect) => {
                self.handle_frame_rect_msg(pipeline_id, subpage_id, rect);
            }
            LoadIframeUrlMsg(url, source_pipeline_id, subpage_id, size_future, sandbox) => {
                self.handle_load_iframe_url_msg(url, source_pipeline_id, subpage_id, size_future, sandbox);
            }
            // Load a new page, usually -- but not always -- from a mouse click or typed url
            // If there is already a pending page (self.pending_frames), it will not be overridden;
            // However, if the id is not encompassed by another change, it will be.
            LoadUrlMsg(source_id, url, size_future) => {
                self.handle_load_url_msg(source_id, url, size_future);
            }
            // Handle a forward or back request
            NavigateMsg(direction) => {
                self.handle_navigate_msg(direction);
            }
            // Notification that rendering has finished and is requesting permission to paint.
            RendererReadyMsg(pipeline_id) => {
                self.handle_renderer_ready_msg(pipeline_id);
            }

            ResizedWindowMsg(new_size) => {
                self.handle_resized_window_msg(new_size);
            }
        }
        true
    }

    fn handle_exit(&self, sender: Chan<()>) {
        for (_id, ref pipeline) in self.pipelines.iter() {
            pipeline.exit();
        }
        self.image_cache_task.exit();
        self.resource_task.send(resource_task::Exit);

        sender.send(());
    }

    fn handle_failure_msg(&mut self, pipeline_id: PipelineId, subpage_id: Option<SubpageId>) {
        let new_id = self.get_next_pipeline_id();
        let pipeline = @mut Pipeline::create(new_id,
                                             subpage_id,
                                             self.chan.clone(),
                                             self.compositor_chan.clone(),
                                             self.image_cache_task.clone(),
                                             self.resource_task.clone(),
                                             self.profiler_chan.clone(),
                                             self.opts.clone(),
                                             {
                                                let size = self.compositor_chan.get_size();
                                                from_value(Size2D(size.width as uint, size.height as uint))
                                             });
        let failure = ~"about:failure";
        let url = make_url(failure, None);
        pipeline.load(url);

        let frames = self.find_all(pipeline_id);
        for frame_tree in frames.iter() {
            frame_tree.pipeline = pipeline;
        };

        self.pipelines.insert(pipeline_id, pipeline);
    }

    fn handle_init_load(&mut self, url: Url) {
        let pipeline = @mut Pipeline::create(self.get_next_pipeline_id(),
                                             None,
                                             self.chan.clone(),
                                             self.compositor_chan.clone(),
                                             self.image_cache_task.clone(),
                                             self.resource_task.clone(),
                                             self.profiler_chan.clone(),
                                             self.opts.clone(),
                                             {
                                                 let size = self.compositor_chan.get_size();
                                                 from_value(Size2D(size.width as uint, size.height as uint))
                                             });
        if url.path.ends_with(".js") {
            pipeline.script_chan.send(ExecuteMsg(pipeline.id, url));
        } else {
            pipeline.load(url);

            self.pending_frames.push(FrameChange{
                before: None,
                after: @mut FrameTree {
                    pipeline: pipeline, 
                    parent: None,
                    children: ~[],
                },
                navigation_type: constellation_msg::Load,
            });
        }
        self.pipelines.insert(pipeline.id, pipeline);
    }
    
    fn handle_frame_rect_msg(&mut self, pipeline_id: PipelineId, subpage_id: SubpageId, rect: Rect<f32>) {
        debug!("Received frame rect %? from %?, %?", rect, pipeline_id, subpage_id);
        let mut already_sent = HashSet::new();

        // Returns true if a child frame tree's subpage id matches the given subpage id
        let subpage_eq = |child_frame_tree: & &mut ChildFrameTree| {
            child_frame_tree.frame_tree.pipeline.subpage_id.expect("Constellation:
                child frame does not have a subpage id. This should not be possible.")
                == subpage_id
        };

        // Update a child's frame rect and inform its script task of the change,
        // if it hasn't been already. Optionally inform the compositor if 
        // resize happens immediately.
        let update_child_rect = |child_frame_tree: &mut ChildFrameTree, is_active: bool| {
            child_frame_tree.rect = Some(rect.clone());
            let pipeline = &child_frame_tree.frame_tree.pipeline;
            if !already_sent.contains(&pipeline.id) {
                let Size2D { width, height } = rect.size;
                if is_active {
                    pipeline.script_chan.send(ResizeMsg(pipeline.id, Size2D {
                        width:  width  as uint,
                        height: height as uint
                    }));
                    self.compositor_chan.send(SetLayerClipRect(pipeline.id, rect));
                } else {
                    pipeline.script_chan.send(ResizeInactiveMsg(pipeline.id,
                                                                Size2D(width as uint, height as uint)));
                }
                already_sent.insert(pipeline.id);
            }
        };

        // If the subframe is in the current frame tree, the compositor needs the new size
        for current_frame in self.current_frame().iter() {
            debug!("Constellation: Sending size for frame in current frame tree.");
            let source_frame = current_frame.find(pipeline_id);
            for source_frame in source_frame.iter() {
                let found_child = source_frame.children.mut_iter()
                    .find(|child| subpage_eq(child));
                found_child.map_move(|child| update_child_rect(child, true));
            }
        }

        // Update all frames with matching pipeline- and subpage-ids
        let frames = self.find_all(pipeline_id);
        for frame_tree in frames.iter() {
            let found_child = frame_tree.children.mut_iter()
                .find(|child| subpage_eq(child));
            found_child.map_move(|child| update_child_rect(child, false));
        }

        // At this point, if no pipelines were sent a resize msg, then this subpage id
        // should be added to pending sizes
        if already_sent.len() == 0 {
            self.pending_sizes.insert((pipeline_id, subpage_id), rect);
        }
    }

    fn handle_load_iframe_url_msg(&mut self,
                                  url: Url,
                                  source_pipeline_id: PipelineId,
                                  subpage_id: SubpageId,
                                  size_future: Future<Size2D<uint>>,
                                  sandbox: IFrameSandboxState) {
        // A message from the script associated with pipeline_id that it has
        // parsed an iframe during html parsing. This iframe will result in a
        // new pipeline being spawned and a frame tree being added to pipeline_id's
        // frame tree's children. This message is never the result of a link clicked
        // or a new url entered.
        //     Start by finding the frame trees matching the pipeline id,
        // and add the new pipeline to their sub frames.
        let frame_trees: ~[@mut FrameTree] = {
            let matching_navi_frames = self.navigation_context.find_all(source_pipeline_id);
            let matching_pending_frames = do self.pending_frames.iter().filter_map |frame_change| {
                frame_change.after.find(source_pipeline_id)
            };
            matching_navi_frames.move_iter().chain(matching_pending_frames).collect()
        };

        if frame_trees.is_empty() {
            fail!("Constellation: source pipeline id of LoadIframeUrlMsg is not in
                   navigation context, nor is it in a pending frame. This should be
                   impossible.");
        }

        let next_pipeline_id = self.get_next_pipeline_id();

        // Compare the pipeline's url to the new url. If the origin is the same,
        // then reuse the script task in creating the new pipeline
        let source_pipeline = *self.pipelines.find(&source_pipeline_id).expect("Constellation:
            source Id of LoadIframeUrlMsg does have an associated pipeline in
            constellation. This should be impossible.");

        let source_url = source_pipeline.url.clone().expect("Constellation: LoadUrlIframeMsg's
        source's Url is None. There should never be a LoadUrlIframeMsg from a pipeline
        that was never given a url to load.");

        let same_script = (source_url.host == url.host &&
                           source_url.port == url.port) && sandbox == IFrameUnsandboxed;
        // FIXME(tkuehn): Need to follow the standardized spec for checking same-origin
        let pipeline = @mut if same_script {
            debug!("Constellation: loading same-origin iframe at %?", url);
            // Reuse the script task if same-origin url's
            Pipeline::with_script(next_pipeline_id,
                                  Some(subpage_id),
                                  self.chan.clone(),
                                  self.compositor_chan.clone(),
                                  self.image_cache_task.clone(),
                                  self.profiler_chan.clone(),
                                  self.opts.clone(),
                                  source_pipeline,
                                  size_future)
        } else {
            debug!("Constellation: loading cross-origin iframe at %?", url);
            // Create a new script task if not same-origin url's
            Pipeline::create(next_pipeline_id,
                             Some(subpage_id),
                             self.chan.clone(),
                             self.compositor_chan.clone(),
                             self.image_cache_task.clone(),
                             self.resource_task.clone(),
                             self.profiler_chan.clone(),
                             self.opts.clone(),
                             size_future)
        };

        if url.path.ends_with(".js") {
            pipeline.execute(url);
        } else {
            debug!("Constellation: sending load msg to pipeline %?", pipeline.id);
            pipeline.load(url);
        }
        let rect = self.pending_sizes.pop(&(source_pipeline_id, subpage_id));
        for frame_tree in frame_trees.iter() {
            frame_tree.children.push(ChildFrameTree {
                frame_tree: @mut FrameTree {
                    pipeline: pipeline,
                    parent: Some(source_pipeline),
                    children: ~[],
                },
                rect: rect,
            });
        }
        self.pipelines.insert(pipeline.id, pipeline);
    }

    fn handle_load_url_msg(&mut self, source_id: PipelineId, url: Url, size_future: Future<Size2D<uint>>) {
        debug!("Constellation: received message to load %s", url.to_str());
        // Make sure no pending page would be overridden.
        let source_frame = self.current_frame().get_ref().find(source_id).expect(
            "Constellation: received a LoadUrlMsg from a pipeline_id associated
            with a pipeline not in the active frame tree. This should be
            impossible.");

        for frame_change in self.pending_frames.iter() {
            let old_id = frame_change.before.expect("Constellation: Received load msg
                from pipeline, but there is no currently active page. This should
                be impossible.");
            let changing_frame = self.current_frame().get_ref().find(old_id).expect("Constellation:
                Pending change has non-active source pipeline. This should be
                impossible.");
            if changing_frame.contains(source_id) || source_frame.contains(old_id) {
                // id that sent load msg is being changed already; abort
                return;
            }
        }
        // Being here means either there are no pending frames, or none of the pending
        // changes would be overriden by changing the subframe associated with source_id.

        let parent = source_frame.parent.clone();
        let subpage_id = source_frame.pipeline.subpage_id;
        let next_pipeline_id = self.get_next_pipeline_id();

        let pipeline = @mut Pipeline::create(next_pipeline_id,
                                             subpage_id,
                                             self.chan.clone(),
                                             self.compositor_chan.clone(),
                                             self.image_cache_task.clone(),
                                             self.resource_task.clone(),
                                             self.profiler_chan.clone(),
                                             self.opts.clone(),
                                             size_future);

        if url.path.ends_with(".js") {
            pipeline.script_chan.send(ExecuteMsg(pipeline.id, url));
        } else {
            pipeline.load(url);

            self.pending_frames.push(FrameChange{
                before: Some(source_id),
                after: @mut FrameTree {
                    pipeline: pipeline, 
                    parent: parent,
                    children: ~[],
                },
                navigation_type: constellation_msg::Load,
            });
        }
        self.pipelines.insert(pipeline.id, pipeline);
    }
    
    fn handle_navigate_msg(&mut self, direction: constellation_msg::NavigationDirection) {
        debug!("received message to navigate %?", direction);

        // TODO(tkuehn): what is the "critical point" beyond which pending frames
        // should not be cleared? Currently, the behavior is that forward/back
        // navigation always has navigation priority, and after that new page loading is
        // first come, first served.
        let destination_frame = match direction {
            constellation_msg::Forward => {
                if self.navigation_context.next.is_empty() {
                    debug!("no next page to navigate to");
                    return;
                } else {
                    let old = self.current_frame().get_ref();
                    for frame in old.iter() {
                        frame.pipeline.revoke_paint_permission();
                    }
                }
                self.navigation_context.forward()
            }
            constellation_msg::Back => {
                if self.navigation_context.previous.is_empty() {
                    debug!("no previous page to navigate to");
                    return;
                } else {
                    let old = self.current_frame().get_ref();
                    for frame in old.iter() {
                        frame.pipeline.revoke_paint_permission();
                    }
                }
                self.navigation_context.back()
            }
        };

        for frame in destination_frame.iter() {
            let pipeline = &frame.pipeline;
            pipeline.reload();
        }
        self.grant_paint_permission(destination_frame, constellation_msg::Navigate);

    }
    
    fn handle_renderer_ready_msg(&mut self, pipeline_id: PipelineId) {
        debug!("Renderer %? ready to send paint msg", pipeline_id);
        // This message could originate from a pipeline in the navigation context or
        // from a pending frame. The only time that we will grant paint permission is
        // when the message originates from a pending frame or the current frame.

        for &current_frame in self.current_frame().iter() {
            // Messages originating in the current frame are not navigations;
            // TODO(tkuehn): In fact, this kind of message might be provably
            // impossible to occur.
            if current_frame.contains(pipeline_id) {
                for frame in current_frame.iter() {
                    frame.pipeline.grant_paint_permission();
                }
                return;
            }
        }

        // Find the pending frame change whose new pipeline id is pipeline_id.
        // If it is not found, it simply means that this pipeline will not receive
        // permission to paint.
        let pending_index = do self.pending_frames.iter().rposition |frame_change| {
            frame_change.after.pipeline.id == pipeline_id
        };
        for &pending_index in pending_index.iter() {
            let frame_change = self.pending_frames.swap_remove(pending_index);
            let to_add = frame_change.after;

            // Create the next frame tree that will be given to the compositor
            let next_frame_tree = match to_add.parent {
                None => to_add, // to_add is the root
                Some(_parent) => @mut (*self.current_frame().unwrap()).clone(),
            };

            // If there are frames to revoke permission from, do so now.
            match frame_change.before {
                Some(revoke_id) => {
                    debug!("Constellation: revoking permission from %?", revoke_id);
                    let current_frame = self.current_frame().unwrap();

                    let to_revoke = current_frame.find(revoke_id).expect(
                        "Constellation: pending frame change refers to an old
                        frame not contained in the current frame. This is a bug");

                    for frame in to_revoke.iter() {
                        frame.pipeline.revoke_paint_permission();
                    }

                    // If to_add is not the root frame, then replace revoked_frame with it.
                    // This conveniently keeps scissor rect size intact.
                    if to_add.parent.is_some() {
                        debug!("Constellation: replacing %? with %? in %?",
                            revoke_id, to_add.pipeline.id, next_frame_tree.pipeline.id);
                        next_frame_tree.replace_child(revoke_id, to_add);
                    }
                }

                None => {
                    // Add to_add to parent's children, if it is not the root
                    let parent = &to_add.parent;
                    for parent in parent.iter() {
                        let subpage_id = to_add.pipeline.subpage_id.expect("Constellation:
                            Child frame's subpage id is None. This should be impossible.");
                        let rect = self.pending_sizes.pop(&(parent.id, subpage_id));
                        let parent = next_frame_tree.find(parent.id).expect(
                            "Constellation: pending frame has a parent frame that is not
                            active. This is a bug.");
                        parent.children.push(ChildFrameTree {
                            frame_tree: to_add,
                            rect: rect,
                        });
                    }
                }
            }
        self.grant_paint_permission(next_frame_tree, frame_change.navigation_type);
        }
    }

    fn handle_resized_window_msg(&mut self, new_size: Size2D<uint>) {
        let mut already_seen = HashSet::new();
        for &@FrameTree { pipeline: pipeline, _ } in self.current_frame().iter() {
            pipeline.script_chan.send(ResizeMsg(pipeline.id, new_size));
            already_seen.insert(pipeline.id);
        }
        for frame_tree in self.navigation_context.previous.iter()
            .chain(self.navigation_context.next.iter()) {
            let pipeline = &frame_tree.pipeline;
            if !already_seen.contains(&pipeline.id) {
                pipeline.script_chan.send(ResizeInactiveMsg(pipeline.id, new_size));
                already_seen.insert(pipeline.id);
            }
        }
    }

    // Close all pipelines at and beneath a given frame
    fn close_pipelines(&mut self, frame_tree: @mut FrameTree) {
        // TODO(tkuehn): should only exit once per unique script task,
        // and then that script task will handle sub-exits
        for @FrameTree { pipeline, _ } in frame_tree.iter() {
            pipeline.exit();
            self.pipelines.remove(&pipeline.id);
        }
    }

    fn handle_evicted_frames(&mut self, evicted: ~[@mut FrameTree]) {
        for &frame_tree in evicted.iter() {
            if !self.navigation_context.contains(frame_tree.pipeline.id) {
                self.close_pipelines(frame_tree);
            } else {
                self.handle_evicted_frames(frame_tree.children.iter()
                    .map(|child| child.frame_tree)
                    .collect());
            }
        }
    }

    // Grants a frame tree permission to paint; optionally updates navigation to reflect a new page
    fn grant_paint_permission(&mut self, frame_tree: @mut FrameTree, navigation_type: NavigationType) {
        // Give permission to paint to the new frame and all child frames
        self.set_ids(frame_tree);

        // Don't call navigation_context.load() on a Navigate type (or None, as in the case of
        // parsed iframes that finish loading)
        match navigation_type {
            constellation_msg::Load => {
                let evicted = self.navigation_context.load(frame_tree);
                self.handle_evicted_frames(evicted);
            }
            _ => {}
        }
    }

    fn set_ids(&self, frame_tree: @mut FrameTree) {
        let (port, chan) = comm::stream();
        self.compositor_chan.send(SetIds(frame_tree.to_sendable(), chan, self.chan.clone()));
        port.recv();
        for frame in frame_tree.iter() {
            frame.pipeline.grant_paint_permission();
        }
    }
}

