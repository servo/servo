/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositing::{CompositorChan, LoadComplete, SetIds, SetLayerClipRect, ShutdownComplete};

use collections::hashmap::{HashMap, HashSet};
use extra::url::Url;
use geom::rect::Rect;
use geom::size::Size2D;
use gfx::render_task;
use pipeline::{Pipeline, CompositionPipeline};
use script::script_task::{ResizeMsg, ResizeInactiveMsg, ExitPipelineMsg};
use script::layout_interface;
use script::layout_interface::LayoutChan;
use script::script_task::ScriptChan;
use servo_msg::compositor_msg::LayerId;
use servo_msg::constellation_msg::{ConstellationChan, ExitMsg, FailureMsg, Failure, FrameRectMsg};
use servo_msg::constellation_msg::{IFrameSandboxState, IFrameUnsandboxed, InitLoadUrlMsg};
use servo_msg::constellation_msg::{LoadCompleteMsg, LoadIframeUrlMsg, LoadUrlMsg, Msg, NavigateMsg};
use servo_msg::constellation_msg::{NavigationType, PipelineId, RendererReadyMsg, ResizedWindowMsg};
use servo_msg::constellation_msg::SubpageId;
use servo_msg::constellation_msg;
use servo_net::image_cache_task::{ImageCacheTask, ImageCacheTaskClient};
use servo_net::resource_task::ResourceTask;
use servo_net::resource_task;
use servo_util::opts::Opts;
use servo_util::time::ProfilerChan;
use servo_util::url::parse_url;
use servo_util::task::spawn_named;
use std::cell::RefCell;
use std::mem::replace;
use std::io;
use std::libc;
use std::rc::Rc;

/// Maintains the pipelines and navigation context and grants permission to composite
pub struct Constellation {
    chan: ConstellationChan,
    request_port: Port<Msg>,
    compositor_chan: CompositorChan,
    resource_task: ResourceTask,
    image_cache_task: ImageCacheTask,
    pipelines: HashMap<PipelineId, Rc<Pipeline>>,
    navigation_context: NavigationContext,
    priv next_pipeline_id: PipelineId,
    pending_frames: ~[FrameChange],
    pending_sizes: HashMap<(PipelineId, SubpageId), Rect<f32>>,
    profiler_chan: ProfilerChan,
    window_size: Size2D<uint>,
    opts: Opts,
}

/// Stores the Id of the outermost frame's pipeline, along with a vector of children frames
struct FrameTree {
    pipeline: RefCell<Rc<Pipeline>>,
    parent: RefCell<Option<Rc<Pipeline>>>,
    children: RefCell<~[ChildFrameTree]>,
}

// Need to clone the FrameTrees, but _not_ the Pipelines
impl Clone for FrameTree {
    fn clone(&self) -> FrameTree {
        let children = self.children.borrow();
        let mut children = children.get().iter().map(|child_frame_tree| {
            child_frame_tree.clone()
        });
        FrameTree {
            pipeline: self.pipeline.clone(),
            parent: self.parent.clone(),
            children: RefCell::new(children.collect()),
        }
    }
}

struct ChildFrameTree {
    frame_tree: Rc<FrameTree>,
    /// Clipping rect representing the size and position, in page coordinates, of the visible
    /// region of the child frame relative to the parent.
    rect: Option<Rect<f32>>,
}

impl Clone for ChildFrameTree {
    fn clone(&self) -> ChildFrameTree {
        ChildFrameTree {
            frame_tree: self.frame_tree.clone(),
            rect: self.rect.clone(),
        }
    }
}

pub struct SendableFrameTree {
    pipeline: CompositionPipeline,
    children: ~[SendableChildFrameTree],
}

pub struct SendableChildFrameTree {
    frame_tree: SendableFrameTree,
    rect: Option<Rect<f32>>,
}

enum ReplaceResult {
    ReplacedNode(Rc<FrameTree>),
    OriginalNode(Rc<FrameTree>),
}

impl FrameTree {
    fn contains(&self, id: PipelineId) -> bool {
        self.iter().any(|frame_tree| {
            // NOTE: work around borrowchk issue
            let tmp = frame_tree.borrow().pipeline.borrow();
            id == tmp.get().borrow().id
        })
    }

    /// Returns the frame tree whose key is id
    fn find(&self, id: PipelineId) -> Option<Rc<FrameTree>> {
        self.iter().find(|frame_tree| {
            // NOTE: work around borrowchk issue
            let tmp = frame_tree.borrow().pipeline.borrow();
            id == tmp.get().borrow().id
        })
    }

    /// Replaces a node of the frame tree in place. Returns the node that was removed or the original node
    /// if the node to replace could not be found.
    fn replace_child(&self, id: PipelineId, new_child: Rc<FrameTree>) -> ReplaceResult {
        for frame_tree in self.iter() {
            // NOTE: work around mutability issue
            let mut children = frame_tree.borrow().children.borrow_mut();
            let mut child = children.get().mut_iter()
                .find(|child| {
                    // NOTE: work around borrowchk issue
                    let tmp = child.frame_tree.borrow().pipeline.borrow();
                    tmp.get().borrow().id == id
                });
            for child in child.mut_iter() {
                // NOTE: work around lifetime issues
                {
                    let tmp = child.frame_tree.borrow().parent.borrow();
                    new_child.borrow().parent.set(tmp.get().clone());
                }
                return ReplacedNode(replace(&mut child.frame_tree, new_child));
            }
        }
        OriginalNode(new_child)
    }

    fn to_sendable(&self) -> SendableFrameTree {
        let sendable_frame_tree = SendableFrameTree {
            pipeline: {
                // NOTE: work around borrowchk issues
                let tmp = self.pipeline.borrow();
                tmp.get().borrow().to_sendable()
            },
            children: {
                // NOTE: work around borrowchk issues
                let tmp = self.children.borrow();
                tmp.get().iter().map(|frame_tree| frame_tree.to_sendable()).collect()
            },
        };
        sendable_frame_tree
    }

    pub fn iter(&self) -> FrameTreeIterator {
        FrameTreeIterator {
            stack: ~[Rc::new(self.clone())],
        }
    }
}

impl ChildFrameTree {
    fn to_sendable(&self) -> SendableChildFrameTree {
        SendableChildFrameTree {
            frame_tree: self.frame_tree.borrow().to_sendable(),
            rect: self.rect,
        }
    }
}

/// An iterator over a frame tree, returning nodes in depth-first order.
/// Note that this iterator should _not_ be used to mutate nodes _during_
/// iteration. Mutating nodes once the iterator is out of scope is OK.
pub struct FrameTreeIterator {
    priv stack: ~[Rc<FrameTree>],
}

impl Iterator<Rc<FrameTree>> for FrameTreeIterator {
    fn next(&mut self) -> Option<Rc<FrameTree>> {
        if !self.stack.is_empty() {
            let next = self.stack.pop();
            {
                // NOTE: work around borrowchk issues
                let tmp = next.get_ref().borrow().children.borrow();
                for cft in tmp.get().rev_iter() {
                    self.stack.push(cft.frame_tree.clone());
                }
            }
            Some(next.unwrap())
        } else {
            None
        }
    }
}

/// Represents the portion of a page that is changing in navigating.
struct FrameChange {
    before: Option<PipelineId>,
    after: Rc<FrameTree>,
    navigation_type: NavigationType,
}

/// Stores the Id's of the pipelines previous and next in the browser's history
struct NavigationContext {
    previous: ~[Rc<FrameTree>],
    next: ~[Rc<FrameTree>],
    current: Option<Rc<FrameTree>>,
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

    pub fn back(&mut self) -> Rc<FrameTree> {
        self.next.push(self.current.take_unwrap());
        let prev = self.previous.pop().unwrap();
        self.current = Some(prev.clone());
        prev
    }

    pub fn forward(&mut self) -> Rc<FrameTree> {
        self.previous.push(self.current.take_unwrap());
        let next = self.next.pop().unwrap();
        self.current = Some(next.clone());
        next
    }

    /// Loads a new set of page frames, returning all evicted frame trees
    pub fn load(&mut self, frame_tree: Rc<FrameTree>) -> ~[Rc<FrameTree>] {
        debug!("navigating to {:?}", {
            // NOTE: work around borrowchk issues
            let tmp = frame_tree.borrow().pipeline.borrow();
            tmp.get().borrow().id
        });
        let evicted = replace(&mut self.next, ~[]);
        if self.current.is_some() {
            self.previous.push(self.current.take_unwrap());
        }
        self.current = Some(frame_tree.clone());
        evicted
    }

    /// Returns the frame trees whose keys are pipeline_id.
    pub fn find_all(&mut self, pipeline_id: PipelineId) -> ~[Rc<FrameTree>] {
        let from_current = self.current.iter().filter_map(|frame_tree| {
            frame_tree.borrow().find(pipeline_id)
        });
        let from_next = self.next.iter().filter_map(|frame_tree| {
            frame_tree.borrow().find(pipeline_id)
        });
        let from_prev = self.previous.iter().filter_map(|frame_tree| {
            frame_tree.borrow().find(pipeline_id)
        });
        from_prev.chain(from_current).chain(from_next).collect()
    }

    pub fn contains(&mut self, pipeline_id: PipelineId) -> bool {
        let from_current = self.current.iter();
        let from_next = self.next.iter();
        let from_prev = self.previous.iter();

        let mut all_contained = from_prev.chain(from_current).chain(from_next);
        all_contained.any(|frame_tree| {
            frame_tree.borrow().contains(pipeline_id)
        })
    }
}

impl Constellation {
    pub fn start(compositor_chan: CompositorChan,
                 opts: &Opts,
                 resource_task: ResourceTask,
                 image_cache_task: ImageCacheTask,
                 profiler_chan: ProfilerChan)
                 -> ConstellationChan {
        let (constellation_port, constellation_chan) = ConstellationChan::new();
        let constellation_chan_clone = constellation_chan.clone();
        let opts_clone = opts.clone();
        spawn_named("Constellation", proc() {
            let mut constellation = Constellation {
                chan: constellation_chan_clone,
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
                window_size: Size2D(800u, 600u),
                opts: opts_clone,
            };
            constellation.run();
        });
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
        let PipelineId(ref mut i) = self.next_pipeline_id;
        *i += 1;
        id
    }

    /// Convenience function for getting the currently active frame tree.
    /// The currently active frame tree should always be the current painter
    fn current_frame<'a>(&'a self) -> &'a Option<Rc<FrameTree>> {
        &self.navigation_context.current
    }

    /// Returns both the navigation context and pending frame trees whose keys are pipeline_id.
    pub fn find_all(&mut self, pipeline_id: PipelineId) -> ~[Rc<FrameTree>] {
        let matching_navi_frames = self.navigation_context.find_all(pipeline_id);
        let matching_pending_frames = self.pending_frames.iter().filter_map(|frame_change| {
            frame_change.after.borrow().find(pipeline_id)
        });
        matching_navi_frames.move_iter().chain(matching_pending_frames).collect()
    }

    /// Handles loading pages, navigation, and granting access to the compositor
    fn handle_request(&mut self, request: Msg) -> bool {
        match request {
            ExitMsg => {
                debug!("constellation exiting");
                self.handle_exit();
                return false;
            }
            FailureMsg(Failure { pipeline_id, subpage_id }) => {
                self.handle_failure_msg(pipeline_id, subpage_id);
            }
            // This should only be called once per constellation, and only by the browser
            InitLoadUrlMsg(url) => {
                debug!("constellation got init load URL message");
                self.handle_init_load(url);
            }
            // A layout assigned a size and position to a subframe. This needs to be reflected by
            // all frame trees in the navigation context containing the subframe.
            FrameRectMsg(pipeline_id, subpage_id, rect) => {
                debug!("constellation got frame rect message");
                self.handle_frame_rect_msg(pipeline_id, subpage_id, rect);
            }
            LoadIframeUrlMsg(url, source_pipeline_id, subpage_id, sandbox) => {
                debug!("constellation got iframe URL load message");
                self.handle_load_iframe_url_msg(url, source_pipeline_id, subpage_id, sandbox);
            }
            // Load a new page, usually -- but not always -- from a mouse click or typed url
            // If there is already a pending page (self.pending_frames), it will not be overridden;
            // However, if the id is not encompassed by another change, it will be.
            LoadUrlMsg(source_id, url) => {
                debug!("constellation got URL load message");
                self.handle_load_url_msg(source_id, url);
            }
            // A page loaded through one of several methods above has completed all parsing,
            // script, and reflow messages have been sent. 
            LoadCompleteMsg(pipeline_id, url) => {
                debug!("constellation got load complete message");
                self.compositor_chan.send(LoadComplete(pipeline_id, url));
            }
            // Handle a forward or back request
            NavigateMsg(direction) => {
                debug!("constellation got navigation message");
                self.handle_navigate_msg(direction);
            }
            // Notification that rendering has finished and is requesting permission to paint.
            RendererReadyMsg(pipeline_id) => {
                debug!("constellation got renderer ready message");
                self.handle_renderer_ready_msg(pipeline_id);
            }
            ResizedWindowMsg(new_size) => {
                debug!("constellation got window resize message");
                self.handle_resized_window_msg(new_size);
            }
        }
        true
    }

    fn handle_exit(&self) {
        for (_id, ref pipeline) in self.pipelines.iter() {
            pipeline.borrow().exit();
        }
        self.image_cache_task.exit();
        self.resource_task.send(resource_task::Exit);
        self.compositor_chan.send(ShutdownComplete);
    }

    fn handle_failure_msg(&mut self, pipeline_id: PipelineId, subpage_id: Option<SubpageId>) {
        debug!("handling failure message from pipeline {:?}, {:?}", pipeline_id, subpage_id);

        if self.opts.hard_fail {
            // It's quite difficult to make Servo exit cleanly if some tasks have failed.
            // Hard fail exists for test runners so we crash and that's good enough.
            let mut stderr = io::stderr();
            stderr.write_str("Pipeline failed in hard-fail mode.  Crashing!\n").unwrap();
            stderr.flush().unwrap();
            unsafe { libc::exit(1); }
        }

        let old_pipeline = match self.pipelines.find(&pipeline_id) {
            None => return, // already failed?
            Some(id) => id.clone()
        };

        let ScriptChan(ref old_script) = old_pipeline.borrow().script_chan;
        old_script.try_send(ExitPipelineMsg(pipeline_id));
        old_pipeline.borrow().render_chan.chan.try_send(render_task::ExitMsg(None));
        let LayoutChan(ref old_layout) = old_pipeline.borrow().layout_chan;
        old_layout.try_send(layout_interface::ExitNowMsg);
        self.pipelines.remove(&pipeline_id);

        let new_id = self.get_next_pipeline_id();
        let pipeline = Pipeline::create(new_id,
                                        subpage_id,
                                        self.chan.clone(),
                                        self.compositor_chan.clone(),
                                        self.image_cache_task.clone(),
                                        self.resource_task.clone(),
                                        self.profiler_chan.clone(),
                                        self.window_size,
                                        self.opts.clone());
        let url = parse_url("about:failure", None);
        pipeline.load(url);

        let pipeline_wrapped = Rc::new(pipeline);
        self.pending_frames.push(FrameChange{
            before: Some(pipeline_id),
            after: Rc::new(FrameTree {
                pipeline: RefCell::new(pipeline_wrapped.clone()),
                parent: RefCell::new(None),
                children: RefCell::new(~[]),
            }),
            navigation_type: constellation_msg::Navigate,
        });

        self.pipelines.insert(pipeline_id, pipeline_wrapped);
    }

    fn handle_init_load(&mut self, url: Url) {
        let pipeline = Pipeline::create(self.get_next_pipeline_id(),
                                        None,
                                        self.chan.clone(),
                                        self.compositor_chan.clone(),
                                        self.image_cache_task.clone(),
                                        self.resource_task.clone(),
                                        self.profiler_chan.clone(),
                                        self.window_size,
                                        self.opts.clone());
        pipeline.load(url);
        let pipeline_wrapped = Rc::new(pipeline);

        self.pending_frames.push(FrameChange {
            before: None,
            after: Rc::new(FrameTree {
                pipeline: RefCell::new(pipeline_wrapped.clone()),
                parent: RefCell::new(None),
                children: RefCell::new(~[]),
            }),
            navigation_type: constellation_msg::Load,
        });
        self.pipelines.insert(pipeline_wrapped.borrow().id, pipeline_wrapped);
    }

    fn handle_frame_rect_msg(&mut self, pipeline_id: PipelineId, subpage_id: SubpageId, rect: Rect<f32>) {
        debug!("Received frame rect {} from {:?}, {:?}", rect, pipeline_id, subpage_id);
        let mut already_sent = HashSet::new();

        // Returns true if a child frame tree's subpage id matches the given subpage id
        let subpage_eq = |child_frame_tree: & &mut ChildFrameTree| {
            // NOTE: work around borrowchk issues
            let tmp = child_frame_tree.frame_tree.borrow().pipeline.borrow();
            tmp.get().borrow().
                subpage_id.expect("Constellation:
                child frame does not have a subpage id. This should not be possible.")
                == subpage_id
        };

        let frames = self.find_all(pipeline_id);

        {
            // Update a child's frame rect and inform its script task of the change,
            // if it hasn't been already. Optionally inform the compositor if 
            // resize happens immediately.
            let update_child_rect = |child_frame_tree: &mut ChildFrameTree, is_active: bool| {
                child_frame_tree.rect = Some(rect.clone());
                // NOTE: work around borrowchk issues
                let pipeline = &child_frame_tree.frame_tree.borrow().pipeline.borrow();
                if !already_sent.contains(&pipeline.get().borrow().id) {
                    let Size2D { width, height } = rect.size;
                    if is_active {
                        let pipeline = pipeline.get().borrow();
                        let ScriptChan(ref script_chan) = pipeline.script_chan;
                        script_chan.send(ResizeMsg(pipeline.id, Size2D {
                            width:  width  as uint,
                            height: height as uint
                        }));
                        self.compositor_chan.send(SetLayerClipRect(pipeline.id,
                                                                   LayerId::null(),
                                                                   rect));
                    } else {
                        let pipeline = pipeline.get().borrow();
                        already_sent.insert(pipeline.id);
                    }
                };
            };

            // If the subframe is in the current frame tree, the compositor needs the new size
            for current_frame in self.current_frame().iter() {
                debug!("Constellation: Sending size for frame in current frame tree.");
                let source_frame = current_frame.borrow().find(pipeline_id);
                for source_frame in source_frame.iter() {
                    // NOTE: work around borrowchk issues
                    let mut tmp = source_frame.borrow().children.borrow_mut();
                    let found_child = tmp.get().mut_iter().find(|child| subpage_eq(child));
                    found_child.map(|child| update_child_rect(child, true));
                }
            }

            // Update all frames with matching pipeline- and subpage-ids
            for frame_tree in frames.iter() {
                // NOTE: work around borrowchk issues
                let mut tmp = frame_tree.borrow().children.borrow_mut();
                let found_child = tmp.get().mut_iter().find(|child| subpage_eq(child));
                found_child.map(|child| update_child_rect(child, false));
            }
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
                                  sandbox: IFrameSandboxState) {
        // A message from the script associated with pipeline_id that it has
        // parsed an iframe during html parsing. This iframe will result in a
        // new pipeline being spawned and a frame tree being added to pipeline_id's
        // frame tree's children. This message is never the result of a link clicked
        // or a new url entered.
        //     Start by finding the frame trees matching the pipeline id,
        // and add the new pipeline to their sub frames.
        let frame_trees: ~[Rc<FrameTree>] = {
            let matching_navi_frames = self.navigation_context.find_all(source_pipeline_id);
            let matching_pending_frames = self.pending_frames.iter().filter_map(|frame_change| {
                frame_change.after.borrow().find(source_pipeline_id)
            });
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
        let source_pipeline = self.pipelines.find(&source_pipeline_id).expect("Constellation:
            source Id of LoadIframeUrlMsg does have an associated pipeline in
            constellation. This should be impossible.").clone();

        let source_url = source_pipeline.borrow().url.get().clone().expect("Constellation: LoadUrlIframeMsg's
        source's Url is None. There should never be a LoadUrlIframeMsg from a pipeline
        that was never given a url to load.");

        let same_script = (source_url.host == url.host &&
                           source_url.port == url.port) && sandbox == IFrameUnsandboxed;
        // FIXME(tkuehn): Need to follow the standardized spec for checking same-origin
        let pipeline = if same_script {
            debug!("Constellation: loading same-origin iframe at {:?}", url);
            // Reuse the script task if same-origin url's
            Pipeline::with_script(next_pipeline_id,
                                  Some(subpage_id),
                                  self.chan.clone(),
                                  self.compositor_chan.clone(),
                                  self.image_cache_task.clone(),
                                  self.profiler_chan.clone(),
                                  self.opts.clone(),
                                  source_pipeline.clone())
        } else {
            debug!("Constellation: loading cross-origin iframe at {:?}", url);
            // Create a new script task if not same-origin url's
            Pipeline::create(next_pipeline_id,
                             Some(subpage_id),
                             self.chan.clone(),
                             self.compositor_chan.clone(),
                             self.image_cache_task.clone(),
                             self.resource_task.clone(),
                             self.profiler_chan.clone(),
                             self.window_size,
                             self.opts.clone())
        };

        debug!("Constellation: sending load msg to pipeline {:?}", pipeline.id);
        pipeline.load(url);
        let pipeline_wrapped = Rc::new(pipeline);
        let rect = self.pending_sizes.pop(&(source_pipeline_id, subpage_id));
        for frame_tree in frame_trees.iter() {
            // NOTE: work around borrowchk issues
            let mut tmp = frame_tree.borrow().children.borrow_mut();
            tmp.get().push(ChildFrameTree {
                frame_tree: Rc::new(FrameTree {
                    pipeline: RefCell::new(pipeline_wrapped.clone()),
                    parent: RefCell::new(Some(source_pipeline.clone())),
                    children: RefCell::new(~[]),
                }),
                rect: rect,
            });
        }
        self.pipelines.insert(pipeline_wrapped.borrow().id, pipeline_wrapped);
    }

    fn handle_load_url_msg(&mut self, source_id: PipelineId, url: Url) {
        debug!("Constellation: received message to load {:s}", url.to_str());
        // Make sure no pending page would be overridden.
        let source_frame = self.current_frame().get_ref().borrow().find(source_id).expect(
            "Constellation: received a LoadUrlMsg from a pipeline_id associated
            with a pipeline not in the active frame tree. This should be
            impossible.");

        for frame_change in self.pending_frames.iter() {
            let old_id = frame_change.before.expect("Constellation: Received load msg
                from pipeline, but there is no currently active page. This should
                be impossible.");
            let changing_frame = self.current_frame().get_ref().borrow().find(old_id).expect("Constellation:
                Pending change has non-active source pipeline. This should be
                impossible.");
            if changing_frame.borrow().contains(source_id) || source_frame.borrow().contains(old_id) {
                // id that sent load msg is being changed already; abort
                return;
            }
        }
        // Being here means either there are no pending frames, or none of the pending
        // changes would be overriden by changing the subframe associated with source_id.

        let parent = source_frame.borrow().parent.clone();
        // NOTE: work around borrowchk issues
        let tmp = source_frame.borrow().pipeline.borrow();
        let subpage_id = tmp.get().borrow().subpage_id;
        let next_pipeline_id = self.get_next_pipeline_id();

        let pipeline = Pipeline::create(next_pipeline_id,
                                        subpage_id,
                                        self.chan.clone(),
                                        self.compositor_chan.clone(),
                                        self.image_cache_task.clone(),
                                        self.resource_task.clone(),
                                        self.profiler_chan.clone(),
                                        self.window_size,
                                        self.opts.clone());

        pipeline.load(url);
        let pipeline_wrapped = Rc::new(pipeline);

        self.pending_frames.push(FrameChange{
            before: Some(source_id),
            after: Rc::new(FrameTree {
                pipeline: RefCell::new(pipeline_wrapped.clone()),
                parent: parent,
                children: RefCell::new(~[]),
            }),
            navigation_type: constellation_msg::Load,
        });
        self.pipelines.insert(pipeline_wrapped.borrow().id, pipeline_wrapped);
    }

    fn handle_navigate_msg(&mut self, direction: constellation_msg::NavigationDirection) {
        debug!("received message to navigate {:?}", direction);

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
                    for frame in old.borrow().iter() {
                        // NOTE: work around borrowchk issues
                        let tmp = frame.borrow().pipeline.borrow();
                        tmp.get().borrow().revoke_paint_permission();
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
                    for frame in old.borrow().iter() {
                        // NOTE: work around borrowchk issues
                        let tmp = frame.borrow().pipeline.borrow();
                        tmp.get().borrow().revoke_paint_permission();
                    }
                }
                self.navigation_context.back()
            }
        };

        for frame in destination_frame.borrow().iter() {
            // NOTE: work around borrowchk issues
            let pipeline = &frame.borrow().pipeline.borrow();
            pipeline.get().borrow().reload();
        }
        self.grant_paint_permission(destination_frame, constellation_msg::Navigate);

    }

    fn handle_renderer_ready_msg(&mut self, pipeline_id: PipelineId) {
        debug!("Renderer {:?} ready to send paint msg", pipeline_id);
        // This message could originate from a pipeline in the navigation context or
        // from a pending frame. The only time that we will grant paint permission is
        // when the message originates from a pending frame or the current frame.

        for current_frame in self.current_frame().iter() {
            // Messages originating in the current frame are not navigations;
            // TODO(tkuehn): In fact, this kind of message might be provably
            // impossible to occur.
            if current_frame.borrow().contains(pipeline_id) {
                for frame in current_frame.borrow().iter() {
                    // NOTE: work around borrowchk issues
                    let tmp = frame.borrow().pipeline.borrow();
                    tmp.get().borrow().grant_paint_permission();
                }
                return;
            }
        }

        // Find the pending frame change whose new pipeline id is pipeline_id.
        // If it is not found, it simply means that this pipeline will not receive
        // permission to paint.
        let pending_index = self.pending_frames.iter().rposition(|frame_change| {
            // NOTE: work around borrowchk issues
            let tmp = frame_change.after.borrow().pipeline.borrow();
            tmp.get().borrow().id == pipeline_id
        });
        for &pending_index in pending_index.iter() {
            let frame_change = self.pending_frames.swap_remove(pending_index);
            let to_add = frame_change.after.clone();

            // Create the next frame tree that will be given to the compositor
            // NOTE: work around borrowchk issues
            let tmp = to_add.borrow().parent.clone();
            let tmp = tmp.borrow();
            let next_frame_tree = if tmp.get().is_some() {
                // NOTE: work around borrowchk issues
                let tmp = self.current_frame().get_ref();
                tmp.clone()
            } else {
                to_add.clone()
            };

            // If there are frames to revoke permission from, do so now.
            match frame_change.before {
                Some(revoke_id) => {
                    debug!("Constellation: revoking permission from {:?}", revoke_id);
                    let current_frame = self.current_frame().get_ref();

                    let to_revoke = current_frame.borrow().find(revoke_id).expect(
                        "Constellation: pending frame change refers to an old
                        frame not contained in the current frame. This is a bug");

                    for frame in to_revoke.borrow().iter() {
                        // NOTE: work around borrowchk issues
                        let tmp = frame.borrow().pipeline.borrow();
                        tmp.get().borrow().revoke_paint_permission();
                    }

                    // If to_add is not the root frame, then replace revoked_frame with it.
                    // This conveniently keeps scissor rect size intact.
                    // NOTE: work around borrowchk issue
                    let mut flag = false;
                    {
                        // NOTE: work around borrowchk issue
                        let tmp = to_add.borrow().parent.borrow();
                        if tmp.get().is_some() {
                            debug!("Constellation: replacing {:?} with {:?} in {:?}",
                                   revoke_id, {
                                    // NOTE: work around borrowchk issues
                                    let tmp = to_add.borrow().pipeline.borrow();
                                    tmp.get().borrow().id
                                }, {
                                    // NOTE: work around borrowchk issues
                                    let tmp = next_frame_tree.borrow().pipeline.borrow();
                                    tmp.get().borrow().id
                                });
                            flag = true;
                        }
                    }
                    if flag {
                        next_frame_tree.borrow().replace_child(revoke_id, to_add);
                    }
                }

                None => {
                    // Add to_add to parent's children, if it is not the root
                    let parent = &to_add.borrow().parent;
                    // NOTE: work around borrowchk issue
                    let tmp = parent.borrow();
                    for parent in tmp.get().iter() {
                        // NOTE: work around borrowchk issues
                        let tmp = to_add.borrow().pipeline.borrow();
                        let subpage_id = tmp.get().borrow().subpage_id
                            .expect("Constellation:
                            Child frame's subpage id is None. This should be impossible.");
                        let rect = self.pending_sizes.pop(&(parent.borrow().id, subpage_id));
                        let parent = next_frame_tree.borrow().find(parent.borrow().id).expect(
                            "Constellation: pending frame has a parent frame that is not
                            active. This is a bug.");
                        // NOTE: work around borrowchk issue
                        let mut tmp = parent.borrow().children.borrow_mut();
                        tmp.get().push(ChildFrameTree {
                            frame_tree: to_add.clone(),
                            rect: rect,
                        });
                    }
                }
            }

            self.grant_paint_permission(next_frame_tree, frame_change.navigation_type);
        }
    }

    /// Called when the window is resized.
    fn handle_resized_window_msg(&mut self, new_size: Size2D<uint>) {
        let mut already_seen = HashSet::new();
        for frame_tree in self.current_frame().iter() {
            debug!("constellation sending resize message to active frame");
            // NOTE: work around borrowchk issues
            let tmp = frame_tree.borrow().pipeline.borrow();
            let pipeline = tmp.get().borrow();
            let ScriptChan(ref chan) = pipeline.script_chan;
            chan.try_send(ResizeMsg(pipeline.id, new_size));
            already_seen.insert(pipeline.id);
        }
        for frame_tree in self.navigation_context.previous.iter()
            .chain(self.navigation_context.next.iter()) {
            // NOTE: work around borrowchk issues
            let tmp = frame_tree.borrow().pipeline.borrow();
            let pipeline = &tmp.get().borrow();
            if !already_seen.contains(&pipeline.id) {
                debug!("constellation sending resize message to inactive frame");
                let ScriptChan(ref chan) = pipeline.script_chan;
                chan.try_send(ResizeInactiveMsg(pipeline.id, new_size));
                already_seen.insert(pipeline.id);
            }
        }

        // If there are any pending outermost frames, then tell them to resize. (This is how the
        // initial window size gets sent to the first page loaded, giving it permission to reflow.)
        for change in self.pending_frames.iter() {
            let frame_tree = change.after.borrow();
            // NOTE: work around borrowchk issue
            let tmp = frame_tree.parent.borrow();
            if tmp.get().is_none() {
                debug!("constellation sending resize message to pending outer frame");
                // NOTE: work around borrowchk issues
                let tmp = frame_tree.pipeline.borrow();
                let pipeline = tmp.get().borrow();
                let ScriptChan(ref chan) = pipeline.script_chan;
                chan.send(ResizeMsg(pipeline.id, new_size))
            }
        }

        self.window_size = new_size;
    }

    // Close all pipelines at and beneath a given frame
    fn close_pipelines(&mut self, frame_tree: Rc<FrameTree>) {
        // TODO(tkuehn): should only exit once per unique script task,
        // and then that script task will handle sub-exits
        for frame_tree in frame_tree.borrow().iter() {
            // NOTE: work around borrowchk issues
            let tmp = frame_tree.borrow().pipeline.borrow();
            let pipeline = tmp.get().borrow();
            pipeline.exit();
            self.pipelines.remove(&pipeline.id);
        }
    }

    fn handle_evicted_frames(&mut self, evicted: ~[Rc<FrameTree>]) {
        for frame_tree in evicted.iter() {
            // NOTE: work around borrowchk issues
            let tmp = frame_tree.borrow().pipeline.borrow();
            if !self.navigation_context.contains(tmp.get().borrow().id) {
                self.close_pipelines(frame_tree.clone());
            } else {
                // NOTE: work around borrowchk issue
                let tmp = frame_tree.borrow().children.borrow();
                let mut frames = tmp.get().iter()
                    .map(|child| child.frame_tree.clone());
                self.handle_evicted_frames(frames.collect());
            }
        }
    }

    // Grants a frame tree permission to paint; optionally updates navigation to reflect a new page
    fn grant_paint_permission(&mut self, frame_tree: Rc<FrameTree>, navigation_type: NavigationType) {
        // Give permission to paint to the new frame and all child frames
        self.set_ids(&frame_tree);

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

    fn set_ids(&self, frame_tree: &Rc<FrameTree>) {
        let (port, chan) = Chan::new();
        debug!("Constellation sending SetIds");
        self.compositor_chan.send(SetIds(frame_tree.borrow().to_sendable(), chan, self.chan.clone()));
        match port.recv_opt() {
            Some(()) => {
                let mut iter = frame_tree.borrow().iter();
                for frame in iter {
                    // NOTE: work around borrowchk issues
                    let tmp = frame.borrow().pipeline.borrow();
                    tmp.get().borrow().grant_paint_permission();
                }
            }
            None => {} // message has been discarded, probably shutting down
        }
    }
}

