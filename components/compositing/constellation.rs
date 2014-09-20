/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositor_task::{CompositorChan, LoadComplete, ShutdownComplete, SetLayerOrigin, SetIds};
use devtools_traits::DevtoolsControlChan;
use std::collections::hashmap::{HashMap, HashSet};
use geom::rect::{Rect, TypedRect};
use geom::scale_factor::ScaleFactor;
use geom::size::TypedSize2D;
use gfx::render_task;
use libc;
use pipeline::{Pipeline, CompositionPipeline};
use layout_traits::{LayoutControlChan, LayoutTaskFactory, ExitNowMsg};
use script_traits::{ResizeMsg, ResizeInactiveMsg, ExitPipelineMsg};
use script_traits::{ScriptControlChan, ScriptTaskFactory};
use servo_msg::compositor_msg::LayerId;
use servo_msg::constellation_msg::{ConstellationChan, ExitMsg, FailureMsg, Failure, FrameRectMsg};
use servo_msg::constellation_msg::{IFrameSandboxState, IFrameUnsandboxed, InitLoadUrlMsg};
use servo_msg::constellation_msg::{LoadCompleteMsg, LoadIframeUrlMsg, LoadUrlMsg, Msg, NavigateMsg};
use servo_msg::constellation_msg::{NavigationType, PipelineId, RendererReadyMsg, ResizedWindowMsg};
use servo_msg::constellation_msg::{SubpageId, WindowSizeData};
use servo_msg::constellation_msg;
use servo_net::image_cache_task::{ImageCacheTask, ImageCacheTaskClient};
use gfx::font_cache_task::FontCacheTask;
use servo_net::resource_task::ResourceTask;
use servo_net::resource_task;
use servo_util::geometry::PagePx;
use servo_util::opts::Opts;
use servo_util::time::TimeProfilerChan;
use servo_util::task::spawn_named;
use std::cell::RefCell;
use std::mem::replace;
use std::io;
use std::rc::Rc;
use url::Url;

/// Maintains the pipelines and navigation context and grants permission to composite
pub struct Constellation<LTF, STF> {
    pub chan: ConstellationChan,
    pub request_port: Receiver<Msg>,
    pub compositor_chan: CompositorChan,
    pub resource_task: ResourceTask,
    pub image_cache_task: ImageCacheTask,
    devtools_chan: Option<DevtoolsControlChan>,
    pipelines: HashMap<PipelineId, Rc<Pipeline>>,
    font_cache_task: FontCacheTask,
    navigation_context: NavigationContext,
    next_pipeline_id: PipelineId,
    pending_frames: Vec<FrameChange>,
    pending_sizes: HashMap<(PipelineId, SubpageId), TypedRect<PagePx, f32>>,
    pub time_profiler_chan: TimeProfilerChan,
    pub window_size: WindowSizeData,
    pub opts: Opts,
}

/// Stores the Id of the outermost frame's pipeline, along with a vector of children frames
struct FrameTree {
    pub pipeline: Rc<Pipeline>,
    pub parent: RefCell<Option<Rc<Pipeline>>>,
    pub children: RefCell<Vec<ChildFrameTree>>,
}

#[deriving(Clone)]
struct ChildFrameTree {
    frame_tree: Rc<FrameTree>,
    /// Clipping rect representing the size and position, in page coordinates, of the visible
    /// region of the child frame relative to the parent.
    pub rect: Option<TypedRect<PagePx, f32>>,
}

pub struct SendableFrameTree {
    pub pipeline: CompositionPipeline,
    pub children: Vec<SendableChildFrameTree>,
}

pub struct SendableChildFrameTree {
    pub frame_tree: SendableFrameTree,
    pub rect: Option<TypedRect<PagePx, f32>>,
}

enum ReplaceResult {
    ReplacedNode(Rc<FrameTree>),
    OriginalNode(Rc<FrameTree>),
}

impl FrameTree {
    fn to_sendable(&self) -> SendableFrameTree {
        let sendable_frame_tree = SendableFrameTree {
            pipeline: self.pipeline.to_sendable(),
            children: self.children.borrow().iter().map(|frame_tree| frame_tree.to_sendable()).collect(),
        };
        sendable_frame_tree
    }
}

trait FrameTreeTraversal {
    fn contains(&self, id: PipelineId) -> bool;
    fn find(&self, id: PipelineId) -> Option<Self>;
    fn replace_child(&self, id: PipelineId, new_child: Self) -> ReplaceResult;
    fn iter(&self) -> FrameTreeIterator;
}

impl FrameTreeTraversal for Rc<FrameTree> {
    fn contains(&self, id: PipelineId) -> bool {
        self.iter().any(|frame_tree| id == frame_tree.pipeline.id)
    }

    /// Returns the frame tree whose key is id
    fn find(&self, id: PipelineId) -> Option<Rc<FrameTree>> {
        self.iter().find(|frame_tree| id == frame_tree.pipeline.id)
    }

    /// Replaces a node of the frame tree in place. Returns the node that was removed or the original node
    /// if the node to replace could not be found.
    fn replace_child(&self, id: PipelineId, new_child: Rc<FrameTree>) -> ReplaceResult {
        for frame_tree in self.iter() {
            let mut children = frame_tree.children.borrow_mut();
            let mut child = children.iter_mut()
                .find(|child| child.frame_tree.pipeline.id == id);
            for child in child.iter_mut() {
                *new_child.parent.borrow_mut() = child.frame_tree.parent.borrow().clone();
                return ReplacedNode(replace(&mut child.frame_tree, new_child));
            }
        }
        OriginalNode(new_child)
    }

    fn iter(&self) -> FrameTreeIterator {
        FrameTreeIterator {
            stack: vec!(self.clone()),
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
struct FrameTreeIterator {
    stack: Vec<Rc<FrameTree>>,
}

impl Iterator<Rc<FrameTree>> for FrameTreeIterator {
    fn next(&mut self) -> Option<Rc<FrameTree>> {
        if !self.stack.is_empty() {
            let next = self.stack.pop();
            for cft in next.as_ref().unwrap().children.borrow().iter() {
                self.stack.push(cft.frame_tree.clone());
            }
            Some(next.unwrap())
        } else {
            None
        }
    }
}

/// Represents the portion of a page that is changing in navigating.
struct FrameChange {
    pub before: Option<PipelineId>,
    pub after: Rc<FrameTree>,
    pub navigation_type: NavigationType,
}

/// Stores the Id's of the pipelines previous and next in the browser's history
struct NavigationContext {
    pub previous: Vec<Rc<FrameTree>>,
    pub next: Vec<Rc<FrameTree>>,
    pub current: Option<Rc<FrameTree>>,
}

impl NavigationContext {
    fn new() -> NavigationContext {
        NavigationContext {
            previous: vec!(),
            next: vec!(),
            current: None,
        }
    }

    /* Note that the following two methods can fail. They should only be called  *
     * when it is known that there exists either a previous page or a next page. */

    fn back(&mut self) -> Rc<FrameTree> {
        self.next.push(self.current.take().unwrap());
        let prev = self.previous.pop().unwrap();
        self.current = Some(prev.clone());
        prev
    }

    fn forward(&mut self) -> Rc<FrameTree> {
        self.previous.push(self.current.take().unwrap());
        let next = self.next.pop().unwrap();
        self.current = Some(next.clone());
        next
    }

    /// Loads a new set of page frames, returning all evicted frame trees
    fn load(&mut self, frame_tree: Rc<FrameTree>) -> Vec<Rc<FrameTree>> {
        debug!("navigating to {:?}", frame_tree.pipeline.id);
        let evicted = replace(&mut self.next, vec!());
        if self.current.is_some() {
            self.previous.push(self.current.take().unwrap());
        }
        self.current = Some(frame_tree.clone());
        evicted
    }

    /// Returns the frame trees whose keys are pipeline_id.
    fn find_all(&mut self, pipeline_id: PipelineId) -> Vec<Rc<FrameTree>> {
        let from_current = self.current.iter().filter_map(|frame_tree| {
            frame_tree.find(pipeline_id)
        });
        let from_next = self.next.iter().filter_map(|frame_tree| {
            frame_tree.find(pipeline_id)
        });
        let from_prev = self.previous.iter().filter_map(|frame_tree| {
            frame_tree.find(pipeline_id)
        });
        from_prev.chain(from_current).chain(from_next).collect()
    }

    fn contains(&mut self, pipeline_id: PipelineId) -> bool {
        let from_current = self.current.iter();
        let from_next = self.next.iter();
        let from_prev = self.previous.iter();

        let mut all_contained = from_prev.chain(from_current).chain(from_next);
        all_contained.any(|frame_tree| {
            frame_tree.contains(pipeline_id)
        })
    }
}

impl<LTF: LayoutTaskFactory, STF: ScriptTaskFactory> Constellation<LTF, STF> {
    pub fn start(compositor_chan: CompositorChan,
                 opts: &Opts,
                 resource_task: ResourceTask,
                 image_cache_task: ImageCacheTask,
                 font_cache_task: FontCacheTask,
                 time_profiler_chan: TimeProfilerChan,
                 devtools_chan: Option<DevtoolsControlChan>)
                 -> ConstellationChan {
        let (constellation_port, constellation_chan) = ConstellationChan::new();
        let constellation_chan_clone = constellation_chan.clone();
        let opts_clone = opts.clone();
        spawn_named("Constellation", proc() {
            let mut constellation : Constellation<LTF, STF> = Constellation {
                chan: constellation_chan_clone,
                request_port: constellation_port,
                compositor_chan: compositor_chan,
                devtools_chan: devtools_chan,
                resource_task: resource_task,
                image_cache_task: image_cache_task,
                font_cache_task: font_cache_task,
                pipelines: HashMap::new(),
                navigation_context: NavigationContext::new(),
                next_pipeline_id: PipelineId(0),
                pending_frames: vec!(),
                pending_sizes: HashMap::new(),
                time_profiler_chan: time_profiler_chan,
                window_size: WindowSizeData {
                    visible_viewport: TypedSize2D(800_f32, 600_f32),
                    initial_viewport: TypedSize2D(800_f32, 600_f32),
                    device_pixel_ratio: ScaleFactor(1.0),
                },
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

    /// Helper function for creating a pipeline
    fn new_pipeline(&self,
                    id: PipelineId,
                    subpage_id: Option<SubpageId>,
                    script_pipeline: Option<Rc<Pipeline>>,
                    url: Url)
                    -> Rc<Pipeline> {
            let pipe = Pipeline::create::<LTF, STF>(id,
                                                    subpage_id,
                                                    self.chan.clone(),
                                                    self.compositor_chan.clone(),
                                                    self.devtools_chan.clone(),
                                                    self.image_cache_task.clone(),
                                                    self.font_cache_task.clone(),
                                                    self.resource_task.clone(),
                                                    self.time_profiler_chan.clone(),
                                                    self.window_size,
                                                    self.opts.clone(),
                                                    script_pipeline,
                                                    url);
            pipe.load();
            Rc::new(pipe)
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
    fn find_all(&mut self, pipeline_id: PipelineId) -> Vec<Rc<FrameTree>> {
        let matching_navi_frames = self.navigation_context.find_all(pipeline_id);
        let matching_pending_frames = self.pending_frames.iter().filter_map(|frame_change| {
            frame_change.after.find(pipeline_id)
        });
        matching_navi_frames.into_iter().chain(matching_pending_frames).collect()
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
                self.handle_frame_rect_msg(pipeline_id, subpage_id, Rect::from_untyped(&rect));
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
            pipeline.exit();
        }
        self.image_cache_task.exit();
        self.resource_task.send(resource_task::Exit);
        self.font_cache_task.exit();
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
            None => {
                debug!("no existing pipeline found; bailing out of failure recovery.");
                return; // already failed?
            }
            Some(pipeline) => pipeline.clone()
        };

        fn force_pipeline_exit(old_pipeline: &Rc<Pipeline>) {
            let ScriptControlChan(ref old_script) = old_pipeline.script_chan;
            let _ = old_script.send_opt(ExitPipelineMsg(old_pipeline.id));
            let _ = old_pipeline.render_chan.send_opt(render_task::ExitMsg(None));
            let LayoutControlChan(ref old_layout) = old_pipeline.layout_chan;
            let _ = old_layout.send_opt(ExitNowMsg);
        }
        force_pipeline_exit(&old_pipeline);
        self.pipelines.remove(&pipeline_id);

        loop {
            let idx = self.pending_frames.iter().position(|pending| {
                pending.after.pipeline.id == pipeline_id
            });
            idx.map(|idx| {
                debug!("removing pending frame change for failed pipeline");
                force_pipeline_exit(&self.pending_frames[idx].after.pipeline);
                self.pending_frames.remove(idx)
            });
            if idx.is_none() {
                break;
            }
        }
        debug!("creating replacement pipeline for about:failure");

        let new_id = self.get_next_pipeline_id();
        let pipeline = self.new_pipeline(new_id, subpage_id, None,
                                         Url::parse("about:failure").unwrap());

        self.pending_frames.push(FrameChange{
            before: Some(pipeline_id),
            after: Rc::new(FrameTree {
                pipeline: pipeline.clone(),
                parent: RefCell::new(None),
                children: RefCell::new(vec!()),
            }),
            navigation_type: constellation_msg::Load,
        });

        self.pipelines.insert(new_id, pipeline);
    }

    fn handle_init_load(&mut self, url: Url) {
        let next_pipeline_id = self.get_next_pipeline_id();
        let pipeline = self.new_pipeline(next_pipeline_id, None, None, url);

        self.pending_frames.push(FrameChange {
            before: None,
            after: Rc::new(FrameTree {
                pipeline: pipeline.clone(),
                parent: RefCell::new(None),
                children: RefCell::new(vec!()),
            }),
            navigation_type: constellation_msg::Load,
        });
        self.pipelines.insert(pipeline.id, pipeline);
    }

    fn handle_frame_rect_msg(&mut self, pipeline_id: PipelineId, subpage_id: SubpageId,
                             rect: TypedRect<PagePx, f32>) {
        debug!("Received frame rect {:?} from {:?}, {:?}", rect, pipeline_id, subpage_id);
        let mut already_sent = HashSet::new();

        // Returns true if a child frame tree's subpage id matches the given subpage id
        let subpage_eq = |child_frame_tree: & &mut ChildFrameTree| {
            child_frame_tree.frame_tree.pipeline.
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
                child_frame_tree.rect = Some(rect);
                // NOTE: work around borrowchk issues
                let pipeline = &child_frame_tree.frame_tree.pipeline;
                if !already_sent.contains(&pipeline.id) {
                    if is_active {
                        let ScriptControlChan(ref script_chan) = pipeline.script_chan;
                        script_chan.send(ResizeMsg(pipeline.id, WindowSizeData {
                            visible_viewport: rect.size,
                            initial_viewport: rect.size * ScaleFactor(1.0),
                            device_pixel_ratio: self.window_size.device_pixel_ratio,
                        }));
                        self.compositor_chan.send(SetLayerOrigin(pipeline.id,
                                                                 LayerId::null(),
                                                                 rect.to_untyped().origin));
                    } else {
                        already_sent.insert(pipeline.id);
                    }
                };
            };

            // If the subframe is in the current frame tree, the compositor needs the new size
            for current_frame in self.current_frame().iter() {
                debug!("Constellation: Sending size for frame in current frame tree.");
                let source_frame = current_frame.find(pipeline_id);
                for source_frame in source_frame.iter() {
                    let mut children = source_frame.children.borrow_mut();
                    let found_child = children.iter_mut().find(|child| subpage_eq(child));
                    found_child.map(|child| update_child_rect(child, true));
                }
            }

            // Update all frames with matching pipeline- and subpage-ids
            for frame_tree in frames.iter() {
                let mut children = frame_tree.children.borrow_mut();
                let found_child = children.iter_mut().find(|child| subpage_eq(child));
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
        let frame_trees = self.find_all(source_pipeline_id);
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

        let source_url = source_pipeline.url.clone();

        let same_script = (source_url.host() == url.host() &&
                           source_url.port() == url.port()) && sandbox == IFrameUnsandboxed;
        // FIXME(tkuehn): Need to follow the standardized spec for checking same-origin
        // Reuse the script task if the URL is same-origin
        let new_pipeline = if same_script {
            debug!("Constellation: loading same-origin iframe at {:?}", url);
            Some(source_pipeline.clone())
        } else {
            debug!("Constellation: loading cross-origin iframe at {:?}", url);
            None
        };

        let pipeline = self.new_pipeline(
            next_pipeline_id,
            Some(subpage_id),
            new_pipeline,
            url
        );

        let rect = self.pending_sizes.pop(&(source_pipeline_id, subpage_id));
        for frame_tree in frame_trees.iter() {
            frame_tree.children.borrow_mut().push(ChildFrameTree {
                frame_tree: Rc::new(FrameTree {
                    pipeline: pipeline.clone(),
                    parent: RefCell::new(Some(source_pipeline.clone())),
                    children: RefCell::new(vec!()),
                }),
                rect: rect,
            });
        }
        self.pipelines.insert(pipeline.id, pipeline);
    }

    fn handle_load_url_msg(&mut self, source_id: PipelineId, url: Url) {
        debug!("Constellation: received message to load {:s}", url.to_string());
        // Make sure no pending page would be overridden.
        let source_frame = self.current_frame().as_ref().unwrap().find(source_id).expect(
            "Constellation: received a LoadUrlMsg from a pipeline_id associated
            with a pipeline not in the active frame tree. This should be
            impossible.");

        for frame_change in self.pending_frames.iter() {
            let old_id = frame_change.before.expect("Constellation: Received load msg
                from pipeline, but there is no currently active page. This should
                be impossible.");
            let changing_frame = self.current_frame().as_ref().unwrap().find(old_id).expect("Constellation:
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

        let pipeline = self.new_pipeline(next_pipeline_id, subpage_id, None, url);

        self.pending_frames.push(FrameChange{
            before: Some(source_id),
            after: Rc::new(FrameTree {
                pipeline: pipeline.clone(),
                parent: parent,
                children: RefCell::new(vec!()),
            }),
            navigation_type: constellation_msg::Load,
        });
        self.pipelines.insert(pipeline.id, pipeline);
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
                    let old = self.current_frame().as_ref().unwrap();
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
                    let old = self.current_frame().as_ref().unwrap();
                    for frame in old.iter() {
                        frame.pipeline.revoke_paint_permission();
                    }
                }
                self.navigation_context.back()
            }
        };

        for frame in destination_frame.iter() {
            frame.pipeline.load();
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
            // they may come from a page load in a subframe.
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
        let pending_index = self.pending_frames.iter().rposition(|frame_change| {
            frame_change.after.pipeline.id == pipeline_id
        });
        for &pending_index in pending_index.iter() {
            let frame_change = self.pending_frames.swap_remove(pending_index).unwrap();
            let to_add = frame_change.after.clone();

            // Create the next frame tree that will be given to the compositor
            let next_frame_tree = if to_add.parent.borrow().is_some() {
                // NOTE: work around borrowchk issues
                self.current_frame().as_ref().unwrap().clone()
            } else {
                to_add.clone()
            };

            // If there are frames to revoke permission from, do so now.
            match frame_change.before {
                Some(revoke_id) if self.current_frame().is_some() => {
                    debug!("Constellation: revoking permission from {:?}", revoke_id);
                    let current_frame = self.current_frame().as_ref().unwrap();

                    let to_revoke = current_frame.find(revoke_id).expect(
                        "Constellation: pending frame change refers to an old \
                        frame not contained in the current frame. This is a bug");

                    for frame in to_revoke.iter() {
                        frame.pipeline.revoke_paint_permission();
                    }

                    // If to_add is not the root frame, then replace revoked_frame with it.
                    // This conveniently keeps scissor rect size intact.
                    // NOTE: work around borrowchk issue
                    let mut flag = false;
                    {
                        if to_add.parent.borrow().is_some() {
                            debug!("Constellation: replacing {:?} with {:?} in {:?}",
                                   revoke_id, to_add.pipeline.id,
                                   next_frame_tree.pipeline.id);
                            flag = true;
                        }
                    }
                    if flag {
                        next_frame_tree.replace_child(revoke_id, to_add);
                    }
                }

                _ => {
                    // Add to_add to parent's children, if it is not the root
                    let parent = &to_add.parent;
                    for parent in parent.borrow().iter() {
                        let subpage_id = to_add.pipeline.subpage_id
                            .expect("Constellation:
                            Child frame's subpage id is None. This should be impossible.");
                        let rect = self.pending_sizes.pop(&(parent.id, subpage_id));
                        let parent = next_frame_tree.find(parent.id).expect(
                            "Constellation: pending frame has a parent frame that is not
                            active. This is a bug.");
                        parent.children.borrow_mut().push(ChildFrameTree {
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
    fn handle_resized_window_msg(&mut self, new_size: WindowSizeData) {
        let mut already_seen = HashSet::new();
        for frame_tree in self.current_frame().iter() {
            debug!("constellation sending resize message to active frame");
            let pipeline = &frame_tree.pipeline;
            let ScriptControlChan(ref chan) = pipeline.script_chan;
            let _ = chan.send_opt(ResizeMsg(pipeline.id, new_size));
            already_seen.insert(pipeline.id);
        }
        for frame_tree in self.navigation_context.previous.iter()
            .chain(self.navigation_context.next.iter()) {
            let pipeline = &frame_tree.pipeline;
            if !already_seen.contains(&pipeline.id) {
                debug!("constellation sending resize message to inactive frame");
                let ScriptControlChan(ref chan) = pipeline.script_chan;
                let _ = chan.send_opt(ResizeInactiveMsg(pipeline.id, new_size));
                already_seen.insert(pipeline.id);
            }
        }

        // If there are any pending outermost frames, then tell them to resize. (This is how the
        // initial window size gets sent to the first page loaded, giving it permission to reflow.)
        for change in self.pending_frames.iter() {
            let frame_tree = &change.after;
            if frame_tree.parent.borrow().is_none() {
                debug!("constellation sending resize message to pending outer frame ({:?})",
                       frame_tree.pipeline.id);
                let ScriptControlChan(ref chan) = frame_tree.pipeline.script_chan;
                let _ = chan.send_opt(ResizeMsg(frame_tree.pipeline.id, new_size));
            }
        }

        self.window_size = new_size;
    }

    // Close all pipelines at and beneath a given frame
    fn close_pipelines(&mut self, frame_tree: Rc<FrameTree>) {
        // TODO(tkuehn): should only exit once per unique script task,
        // and then that script task will handle sub-exits
        for frame_tree in frame_tree.iter() {
            frame_tree.pipeline.exit();
            self.pipelines.remove(&frame_tree.pipeline.id);
        }
    }

    fn handle_evicted_frames(&mut self, evicted: Vec<Rc<FrameTree>>) {
        for frame_tree in evicted.iter() {
            if !self.navigation_context.contains(frame_tree.pipeline.id) {
                self.close_pipelines(frame_tree.clone());
            } else {
                let frames = frame_tree.children.borrow().iter()
                    .map(|child| child.frame_tree.clone()).collect();
                self.handle_evicted_frames(frames);
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
                debug!("evicting old frames due to load");
                let evicted = self.navigation_context.load(frame_tree);
                self.handle_evicted_frames(evicted);
            }
            _ => {
                debug!("ignoring non-load navigation type");
            }
        }
    }

    fn set_ids(&self, frame_tree: &Rc<FrameTree>) {
        let (chan, port) = channel();
        debug!("Constellation sending SetIds");
        self.compositor_chan.send(SetIds(frame_tree.to_sendable(), chan, self.chan.clone()));
        match port.recv_opt() {
            Ok(()) => {
                let mut iter = frame_tree.iter();
                for frame in iter {
                    frame.pipeline.grant_paint_permission();
                }
            }
            Err(()) => {} // message has been discarded, probably shutting down
        }
    }
}

