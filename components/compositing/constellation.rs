/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use pipeline::{Pipeline, CompositionPipeline};

use compositor_task::CompositorProxy;
use compositor_task::Msg as CompositorMsg;
use devtools_traits::{DevtoolsControlChan, DevtoolsControlMsg};
use geom::rect::{Rect, TypedRect};
use geom::scale_factor::ScaleFactor;
use gfx::font_cache_task::FontCacheTask;
use layers::geometry::DevicePixel;
use layout_traits::LayoutTaskFactory;
use libc;
use script_traits::{CompositorEvent, ConstellationControlMsg};
use script_traits::{ScriptControlChan, ScriptTaskFactory};
use msg::compositor_msg::LayerId;
use msg::constellation_msg::{self, ConstellationChan, Failure};
use msg::constellation_msg::{IFrameSandboxState, NavigationDirection};
use msg::constellation_msg::{Key, KeyState, KeyModifiers};
use msg::constellation_msg::{LoadData, NavigationType};
use msg::constellation_msg::{PipelineExitType, PipelineId};
use msg::constellation_msg::{SubpageId, WindowSizeData};
use msg::constellation_msg::Msg as ConstellationMsg;
use net::image_cache_task::{ImageCacheTask, ImageCacheTaskClient};
use net::resource_task::ResourceTask;
use net::resource_task;
use net::storage_task::{StorageTask, StorageTaskMsg};
use util::cursor::Cursor;
use util::geometry::{PagePx, ViewportPx};
use util::opts;
use util::task::spawn_named;
use util::time::TimeProfilerChan;
use std::borrow::ToOwned;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::old_io as io;
use std::mem::replace;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, channel};
use url::Url;

/// Maintains the pipelines and navigation context and grants permission to composite.
pub struct Constellation<LTF, STF> {
    /// A channel through which messages can be sent to this object.
    pub chan: ConstellationChan,

    /// Receives messages.
    pub request_port: Receiver<ConstellationMsg>,

    /// A channel (the implementation of which is port-specific) through which messages can be sent
    /// to the compositor.
    pub compositor_proxy: Box<CompositorProxy>,

    /// A channel through which messages can be sent to the resource task.
    pub resource_task: ResourceTask,

    /// A channel through which messages can be sent to the image cache task.
    pub image_cache_task: ImageCacheTask,

    /// A channel through which messages can be sent to the developer tools.
    devtools_chan: Option<DevtoolsControlChan>,

    /// A channel through which messages can be sent to the storage task.
    storage_task: StorageTask,

    /// A list of all the pipelines. (See the `pipeline` module for more details.)
    pipelines: HashMap<PipelineId, Rc<Pipeline>>,

    /// A channel through which messages can be sent to the font cache.
    font_cache_task: FontCacheTask,

    navigation_context: NavigationContext,

    /// The next free ID to assign to a pipeline.
    next_pipeline_id: PipelineId,

    /// The next free ID to assign to a frame.
    next_frame_id: FrameId,

    /// Navigation operations that are in progress.
    pending_frames: Vec<FrameChange>,

    pending_sizes: HashMap<(PipelineId, SubpageId), TypedRect<PagePx, f32>>,

    /// A channel through which messages can be sent to the time profiler.
    pub time_profiler_chan: TimeProfilerChan,

    pub window_size: WindowSizeData,
}

/// A unique ID used to identify a frame.
#[derive(Copy)]
pub struct FrameId(u32);

/// One frame in the hierarchy.
struct FrameTree {
    /// The ID of this frame.
    pub id: FrameId,
    /// The pipeline for this frame.
    pub pipeline: RefCell<Rc<Pipeline>>,
    /// The parent frame's pipeline.
    pub parent: RefCell<Option<Rc<Pipeline>>>,
    /// A vector of child frames.
    pub children: RefCell<Vec<ChildFrameTree>>,
    /// Whether this frame has a compositor layer.
    pub has_compositor_layer: Cell<bool>,
}

impl FrameTree {
    fn new(id: FrameId, pipeline: Rc<Pipeline>, parent_pipeline: Option<Rc<Pipeline>>)
           -> FrameTree {
        FrameTree {
            id: id,
            pipeline: RefCell::new(pipeline.clone()),
            parent: RefCell::new(parent_pipeline),
            children: RefCell::new(vec!()),
            has_compositor_layer: Cell::new(false),
        }
    }

    fn add_child(&self, new_child: ChildFrameTree) {
        self.children.borrow_mut().push(new_child);
    }
}

#[derive(Clone)]
struct ChildFrameTree {
    frame_tree: Rc<FrameTree>,
    /// Clipping rect representing the size and position, in page coordinates, of the visible
    /// region of the child frame relative to the parent.
    pub rect: Option<TypedRect<PagePx, f32>>,
}

impl ChildFrameTree {
    fn new(frame_tree: Rc<FrameTree>, rect: Option<TypedRect<PagePx, f32>>) -> ChildFrameTree {
        ChildFrameTree {
            frame_tree: frame_tree,
            rect: rect,
        }
    }
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
        SendableFrameTree {
            pipeline: self.pipeline.borrow().to_sendable(),
            children: self.children
                          .borrow()
                          .iter()
                          .map(|frame_tree| frame_tree.to_sendable())
                          .collect(),
        }
    }
}

trait FrameTreeTraversal {
    fn contains(&self, id: PipelineId) -> bool;
    fn find(&self, id: PipelineId) -> Option<Self>;
    fn find_with_subpage_id(&self, id: Option<SubpageId>) -> Option<Rc<FrameTree>>;
    fn replace_child(&self, id: PipelineId, new_child: Self) -> ReplaceResult;
    fn iter(&self) -> FrameTreeIterator;
}

impl FrameTreeTraversal for Rc<FrameTree> {
    fn contains(&self, id: PipelineId) -> bool {
        self.iter().any(|frame_tree| id == frame_tree.pipeline.borrow().id)
    }

    /// Returns the frame tree whose key is id
    fn find(&self, id: PipelineId) -> Option<Rc<FrameTree>> {
        self.iter().find(|frame_tree| id == frame_tree.pipeline.borrow().id)
    }

    /// Returns the frame tree whose subpage is id
    fn find_with_subpage_id(&self, id: Option<SubpageId>) -> Option<Rc<FrameTree>> {
        self.iter().find(|frame_tree| id == frame_tree.pipeline.borrow().subpage_id())
    }

    /// Replaces a node of the frame tree in place. Returns the node that was removed or the
    /// original node if the node to replace could not be found.
    fn replace_child(&self, id: PipelineId, new_child: Rc<FrameTree>) -> ReplaceResult {
        for frame_tree in self.iter() {
            let mut children = frame_tree.children.borrow_mut();
            let child = children.iter_mut()
                .find(|child| child.frame_tree.pipeline.borrow().id == id);
            match child {
                Some(child) => {
                    *new_child.parent.borrow_mut() = child.frame_tree.parent.borrow().clone();
                    return ReplaceResult::ReplacedNode(replace(&mut child.frame_tree, new_child));
                }
                None => (),
            }
        }
        ReplaceResult::OriginalNode(new_child)
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

impl Iterator for FrameTreeIterator {
    type Item = Rc<FrameTree>;
    fn next(&mut self) -> Option<Rc<FrameTree>> {
        match self.stack.pop() {
            Some(next) => {
                for cft in next.children.borrow().iter() {
                    self.stack.push(cft.frame_tree.clone());
                }
                Some(next)
            }
            None => None,
        }
    }
}

/// Represents the portion of a page that is changing in navigating.
struct FrameChange {
    /// The old pipeline ID.
    pub before: Option<PipelineId>,
    /// The resulting frame tree after navigation.
    pub after: Rc<FrameTree>,
    /// The kind of navigation that is occurring.
    pub navigation_type: NavigationType,
}

/// Stores the Id's of the pipelines previous and next in the browser's history
struct NavigationContext {
    previous: Vec<Rc<FrameTree>>,
    next: Vec<Rc<FrameTree>>,
    current: Option<Rc<FrameTree>>,
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

    fn back(&mut self, compositor_proxy: &mut CompositorProxy) -> Rc<FrameTree> {
        self.next.push(self.current.take().unwrap());
        let prev = self.previous.pop().unwrap();
        self.set_current(prev.clone(), compositor_proxy);
        prev
    }

    fn forward(&mut self, compositor_proxy: &mut CompositorProxy) -> Rc<FrameTree> {
        self.previous.push(self.current.take().unwrap());
        let next = self.next.pop().unwrap();
        self.set_current(next.clone(), compositor_proxy);
        next
    }

    /// Loads a new set of page frames, returning all evicted frame trees
    fn load(&mut self, frame_tree: Rc<FrameTree>, compositor_proxy: &mut CompositorProxy)
            -> Vec<Rc<FrameTree>> {
        debug!("navigating to {:?}", frame_tree.pipeline.borrow().id);
        let evicted = replace(&mut self.next, vec!());
        match self.current.take() {
            Some(current) => self.previous.push(current),
            None => (),
        }
        self.set_current(frame_tree, compositor_proxy);
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

    /// Always use this method to set the currently-displayed frame. It correctly informs the
    /// compositor of the new URLs.
    fn set_current(&mut self, new_frame: Rc<FrameTree>, compositor_proxy: &mut CompositorProxy) {
        self.current = Some(new_frame.clone());
        compositor_proxy.send(CompositorMsg::ChangePageLoadData(
            new_frame.id,
            new_frame.pipeline.borrow().load_data.clone()));
    }
}

impl<LTF: LayoutTaskFactory, STF: ScriptTaskFactory> Constellation<LTF, STF> {
    pub fn start(compositor_proxy: Box<CompositorProxy+Send>,
                 resource_task: ResourceTask,
                 image_cache_task: ImageCacheTask,
                 font_cache_task: FontCacheTask,
                 time_profiler_chan: TimeProfilerChan,
                 devtools_chan: Option<DevtoolsControlChan>,
                 storage_task: StorageTask)
                 -> ConstellationChan {
        let (constellation_port, constellation_chan) = ConstellationChan::new();
        let constellation_chan_clone = constellation_chan.clone();
        spawn_named("Constellation".to_owned(), move || {
            let mut constellation: Constellation<LTF, STF> = Constellation {
                chan: constellation_chan_clone,
                request_port: constellation_port,
                compositor_proxy: compositor_proxy,
                devtools_chan: devtools_chan,
                resource_task: resource_task,
                image_cache_task: image_cache_task,
                font_cache_task: font_cache_task,
                storage_task: storage_task,
                pipelines: HashMap::new(),
                navigation_context: NavigationContext::new(),
                next_pipeline_id: PipelineId(0),
                next_frame_id: FrameId(0),
                pending_frames: vec!(),
                pending_sizes: HashMap::new(),
                time_profiler_chan: time_profiler_chan,
                window_size: WindowSizeData {
                    visible_viewport: opts::get().initial_window_size.as_f32() * ScaleFactor(1.0),
                    initial_viewport: opts::get().initial_window_size.as_f32() * ScaleFactor(1.0),
                    device_pixel_ratio: ScaleFactor(1.0),
                },
            };
            constellation.run();
        });
        constellation_chan
    }

    fn run(&mut self) {
        loop {
            let request = self.request_port.recv().unwrap();
            if !self.handle_request(request) {
                break;
            }
        }
    }

    /// Helper function for creating a pipeline
    fn new_pipeline(&mut self,
                    id: PipelineId,
                    parent: Option<(PipelineId, SubpageId)>,
                    script_pipeline: Option<Rc<Pipeline>>,
                    load_data: LoadData)
                    -> Rc<Pipeline> {
        let pipe = Pipeline::create::<LTF, STF>(id,
                                                parent,
                                                self.chan.clone(),
                                                self.compositor_proxy.clone_compositor_proxy(),
                                                self.devtools_chan.clone(),
                                                self.image_cache_task.clone(),
                                                self.font_cache_task.clone(),
                                                self.resource_task.clone(),
                                                self.storage_task.clone(),
                                                self.time_profiler_chan.clone(),
                                                self.window_size,
                                                script_pipeline,
                                                load_data.clone());
        pipe.load();
        Rc::new(pipe)
    }

    /// Helper function for getting a unique pipeline ID.
    fn get_next_pipeline_id(&mut self) -> PipelineId {
        let id = self.next_pipeline_id;
        let PipelineId(ref mut i) = self.next_pipeline_id;
        *i += 1;
        id
    }

    /// Helper function for getting a unique frame ID.
    fn get_next_frame_id(&mut self) -> FrameId {
        let id = self.next_frame_id;
        let FrameId(ref mut i) = self.next_frame_id;
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
        let mut matching_navi_frames = self.navigation_context.find_all(pipeline_id);
        matching_navi_frames.extend(self.pending_frames.iter().filter_map(|frame_change| {
            frame_change.after.find(pipeline_id)
        }));
        matching_navi_frames
    }

    /// Handles loading pages, navigation, and granting access to the compositor
    fn handle_request(&mut self, request: ConstellationMsg) -> bool {
        match request {
            ConstellationMsg::Exit => {
                debug!("constellation exiting");
                self.handle_exit();
                return false;
            }
            ConstellationMsg::Failure(Failure { pipeline_id, parent }) => {
                self.handle_failure_msg(pipeline_id, parent);
            }
            // This should only be called once per constellation, and only by the browser
            ConstellationMsg::InitLoadUrl(url) => {
                debug!("constellation got init load URL message");
                self.handle_init_load(url);
            }
            // A layout assigned a size and position to a subframe. This needs to be reflected by
            // all frame trees in the navigation context containing the subframe.
            ConstellationMsg::FrameRect(pipeline_id, subpage_id, rect) => {
                debug!("constellation got frame rect message");
                self.handle_frame_rect_msg(pipeline_id, subpage_id, Rect::from_untyped(&rect));
            }
            ConstellationMsg::ScriptLoadedURLInIFrame(url, source_pipeline_id, new_subpage_id, old_subpage_id, sandbox) => {
                debug!("constellation got iframe URL load message");
                self.handle_script_loaded_url_in_iframe_msg(url,
                                                            source_pipeline_id,
                                                            new_subpage_id,
                                                            old_subpage_id,
                                                            sandbox);
            }
            ConstellationMsg::SetCursor(cursor) => self.handle_set_cursor_msg(cursor),
            // Load a new page, usually -- but not always -- from a mouse click or typed url
            // If there is already a pending page (self.pending_frames), it will not be overridden;
            // However, if the id is not encompassed by another change, it will be.
            ConstellationMsg::LoadUrl(source_id, load_data) => {
                debug!("constellation got URL load message");
                self.handle_load_url_msg(source_id, load_data);
            }
            // A page loaded through one of several methods above has completed all parsing,
            // script, and reflow messages have been sent.
            ConstellationMsg::LoadComplete => {
                debug!("constellation got load complete message");
                self.compositor_proxy.send(CompositorMsg::LoadComplete);
            }
            // Handle a forward or back request
            ConstellationMsg::Navigate(direction) => {
                debug!("constellation got navigation message");
                self.handle_navigate_msg(direction);
            }
            // Notification that painting has finished and is requesting permission to paint.
            ConstellationMsg::PainterReady(pipeline_id) => {
                debug!("constellation got painter ready message");
                self.handle_painter_ready_msg(pipeline_id);
            }
            ConstellationMsg::ResizedWindow(new_size) => {
                debug!("constellation got window resize message");
                self.handle_resized_window_msg(new_size);
            }
            ConstellationMsg::KeyEvent(key, state, modifiers) => {
                debug!("constellation got key event message");
                self.handle_key_msg(key, state, modifiers);
            }
            ConstellationMsg::GetPipelineTitle(pipeline_id) => {
                debug!("constellation got get-pipeline-title message");
                self.handle_get_pipeline_title_msg(pipeline_id);
            }
        }
        true
    }

    fn handle_exit(&mut self) {
        for (_id, ref pipeline) in self.pipelines.iter() {
            pipeline.exit(PipelineExitType::Complete);
        }
        self.image_cache_task.exit();
        self.resource_task.send(resource_task::ControlMsg::Exit).unwrap();
        self.devtools_chan.as_ref().map(|chan| {
            chan.send(DevtoolsControlMsg::ServerExitMsg).unwrap();
        });
        self.storage_task.send(StorageTaskMsg::Exit).unwrap();
        self.font_cache_task.exit();
        self.compositor_proxy.send(CompositorMsg::ShutdownComplete);
    }

    fn handle_failure_msg(&mut self, pipeline_id: PipelineId, parent: Option<(PipelineId, SubpageId)>) {
        debug!("handling failure message from pipeline {:?}, {:?}", pipeline_id, parent);

        if opts::get().hard_fail {
            // It's quite difficult to make Servo exit cleanly if some tasks have failed.
            // Hard fail exists for test runners so we crash and that's good enough.
            let mut stderr = io::stderr();
            stderr.write_str("Pipeline failed in hard-fail mode.  Crashing!\n").unwrap();
            stderr.flush().unwrap();
            unsafe { libc::exit(1); }
        }

        let old_pipeline = match self.pipelines.get(&pipeline_id) {
            None => {
                debug!("no existing pipeline found; bailing out of failure recovery.");
                return; // already failed?
            }
            Some(pipeline) => pipeline.clone()
        };

        old_pipeline.force_exit();
        self.compositor_proxy.send(CompositorMsg::PaintTaskExited(old_pipeline.id));
        self.pipelines.remove(&pipeline_id);

        loop {
            let idx = self.pending_frames.iter().position(|pending| {
                pending.after.pipeline.borrow().id == pipeline_id
            });
            match idx {
                Some(idx) => {
                    debug!("removing pending frame change for failed pipeline");
                    self.pending_frames[idx].after.pipeline.borrow().force_exit();
                    self.compositor_proxy.send(CompositorMsg::PaintTaskExited(old_pipeline.id));
                    self.pending_frames.remove(idx);
                },
                None => break,
            }
        }
        debug!("creating replacement pipeline for about:failure");

        let new_id = self.get_next_pipeline_id();
        let new_frame_id = self.get_next_frame_id();
        let pipeline = self.new_pipeline(new_id, parent, None,
                                         LoadData::new(Url::parse("about:failure").unwrap()));

        self.browse(Some(pipeline_id),
                    Rc::new(FrameTree::new(new_frame_id, pipeline.clone(), None)),
                    NavigationType::Load);

        self.pipelines.insert(new_id, pipeline);
    }

    /// Performs navigation. This pushes a `FrameChange` object onto the list of pending frames.
    ///
    /// TODO(pcwalton): Send a `BeforeBrowse` message to the embedder and allow cancellation.
    fn browse(&mut self,
              before: Option<PipelineId>,
              after: Rc<FrameTree>,
              navigation_type: NavigationType) {
        self.pending_frames.push(FrameChange {
            before: before,
            after: after,
            navigation_type: navigation_type,
        });
    }

    fn handle_init_load(&mut self, url: Url) {
        let next_pipeline_id = self.get_next_pipeline_id();
        let next_frame_id = self.get_next_frame_id();
        let pipeline = self.new_pipeline(next_pipeline_id, None, None, LoadData::new(url));
        self.browse(None,
                    Rc::new(FrameTree::new(next_frame_id, pipeline.clone(), None)),
                    NavigationType::Load);
        self.pipelines.insert(pipeline.id, pipeline);
    }

    fn handle_frame_rect_msg(&mut self, pipeline_id: PipelineId, subpage_id: SubpageId,
                             rect: TypedRect<PagePx, f32>) {
        debug!("Received frame rect {:?} from {:?}, {:?}", rect, pipeline_id, subpage_id);
        let mut already_sent = HashSet::new();

        // Returns true if a child frame tree's subpage id matches the given subpage id
        let subpage_eq = |&:child_frame_tree: & &mut ChildFrameTree| {
            child_frame_tree.frame_tree.pipeline.borrow().
                subpage_id().expect("Constellation:
                child frame does not have a subpage id. This should not be possible.")
                == subpage_id
        };

        let frames = self.find_all(pipeline_id);

        {
            // If the subframe is in the current frame tree, the compositor needs the new size
            for current_frame in self.navigation_context.current.iter() {
                debug!("Constellation: Sending size for frame in current frame tree.");
                let source_frame = current_frame.find(pipeline_id);
                for source_frame in source_frame.iter() {
                    let mut children = source_frame.children.borrow_mut();
                    match children.iter_mut().find(|child| subpage_eq(child)) {
                        None => {}
                        Some(child) => {
                            let has_compositor_layer = child.frame_tree.has_compositor_layer.get();
                            update_child_rect(child,
                                              rect,
                                              has_compositor_layer,
                                              &mut already_sent,
                                              &mut self.compositor_proxy,
                                              self.window_size.device_pixel_ratio)
                        }
                    }
                }
            }

            // Update all frames with matching pipeline- and subpage-ids
            for frame_tree in frames.iter() {
                let mut children = frame_tree.children.borrow_mut();
                let found_child = children.iter_mut().find(|child| subpage_eq(child));
                found_child.map(|child| {
                    update_child_rect(child,
                                      rect,
                                      false,
                                      &mut already_sent,
                                      &mut self.compositor_proxy,
                                      self.window_size.device_pixel_ratio)
                });
            }
        }

        // At this point, if no pipelines were sent a resize msg, then this subpage id
        // should be added to pending sizes
        if already_sent.len() == 0 {
            self.pending_sizes.insert((pipeline_id, subpage_id), rect);
        }

        // Update a child's frame rect and inform its script task of the change,
        // if it hasn't been already. Optionally inform the compositor if
        // resize happens immediately.
        fn update_child_rect(child_frame_tree: &mut ChildFrameTree,
                             rect: TypedRect<PagePx,f32>,
                             is_active: bool,
                             already_sent: &mut HashSet<PipelineId>,
                             compositor_proxy: &mut Box<CompositorProxy>,
                             device_pixel_ratio: ScaleFactor<ViewportPx,DevicePixel,f32>) {
            child_frame_tree.rect = Some(rect);
            // NOTE: work around borrowchk issues
            let pipeline = &*child_frame_tree.frame_tree.pipeline.borrow();
            if !already_sent.contains(&pipeline.id) {
                if is_active {
                    let ScriptControlChan(ref script_chan) = pipeline.script_chan;
                    script_chan.send(ConstellationControlMsg::Resize(pipeline.id, WindowSizeData {
                        visible_viewport: rect.size,
                        initial_viewport: rect.size * ScaleFactor(1.0),
                        device_pixel_ratio: device_pixel_ratio,
                    })).unwrap();
                    compositor_proxy.send(CompositorMsg::SetLayerOrigin(
                        pipeline.id,
                        LayerId::null(),
                        rect.to_untyped().origin));
                } else {
                    already_sent.insert(pipeline.id);
                }
            };
        }
    }

    fn update_child_pipeline(&mut self,
                             frame_tree: Rc<FrameTree>,
                             new_pipeline: Rc<Pipeline>,
                             old_subpage_id: SubpageId) {
        let existing_tree = match frame_tree.find_with_subpage_id(Some(old_subpage_id)) {
            Some(existing_tree) => existing_tree.clone(),
            None => panic!("Tried to update non-existing frame tree with pipeline={:?} subpage={:?}",
                           new_pipeline.id,
                           old_subpage_id),
        };

        let old_pipeline = existing_tree.pipeline.borrow().clone();
        *existing_tree.pipeline.borrow_mut() = new_pipeline.clone();

        // If we have not yet sent this frame to the compositor for layer creation, we don't
        // need to inform the compositor of updates to the pipeline.
        if !existing_tree.has_compositor_layer.get() {
            return;
        }

        let (chan, port) = channel();
        self.compositor_proxy.send(CompositorMsg::ChangeLayerPipelineAndRemoveChildren(
            old_pipeline.to_sendable(),
            new_pipeline.to_sendable(),
            chan));
        let _ = port.recv();
    }

    fn create_or_update_child_pipeline(&mut self,
                                       frame_tree: Rc<FrameTree>,
                                       new_pipeline: Rc<Pipeline>,
                                       new_rect: Option<TypedRect<PagePx, f32>>,
                                       old_subpage_id: Option<SubpageId>) {
        match old_subpage_id {
            Some(old_subpage_id) =>
                self.update_child_pipeline(frame_tree.clone(), new_pipeline, old_subpage_id),
            None => {
                let child_tree = Rc::new(
                    FrameTree::new(self.get_next_frame_id(),
                                   new_pipeline,
                                   Some(frame_tree.pipeline.borrow().clone())));
                frame_tree.add_child(ChildFrameTree::new(child_tree, new_rect));
            }
        }
    }

    // The script task associated with pipeline_id has loaded a URL in an iframe via script. This
    // will result in a new pipeline being spawned and a frame tree being added to
    // containing_page_pipeline_id's frame tree's children. This message is never the result of a
    // page navigation.
    fn handle_script_loaded_url_in_iframe_msg(&mut self,
                                              url: Url,
                                              containing_page_pipeline_id: PipelineId,
                                              new_subpage_id: SubpageId,
                                              old_subpage_id: Option<SubpageId>,
                                              sandbox: IFrameSandboxState) {
        // Start by finding the frame trees matching the pipeline id,
        // and add the new pipeline to their sub frames.
        let frame_trees = self.find_all(containing_page_pipeline_id);
        if frame_trees.is_empty() {
            panic!("Constellation: source pipeline id of ScriptLoadedURLInIFrame is not in
                    navigation context, nor is it in a pending frame. This should be
                    impossible.");
        }

        // Compare the pipeline's url to the new url. If the origin is the same,
        // then reuse the script task in creating the new pipeline
        let source_pipeline = self.pipelines.get(&containing_page_pipeline_id).expect("Constellation:
            source Id of ScriptLoadedURLInIFrameMsg does have an associated pipeline in
            constellation. This should be impossible.").clone();

        let source_url = source_pipeline.load_data.url.clone();

        let same_script = (source_url.host() == url.host() &&
                           source_url.port() == url.port()) &&
                           sandbox == IFrameSandboxState::IFrameUnsandboxed;
        // FIXME(tkuehn): Need to follow the standardized spec for checking same-origin
        // Reuse the script task if the URL is same-origin
        let script_pipeline = if same_script {
            debug!("Constellation: loading same-origin iframe at {:?}", url);
            Some(source_pipeline.clone())
        } else {
            debug!("Constellation: loading cross-origin iframe at {:?}", url);
            None
        };

        let new_frame_pipeline_id = self.get_next_pipeline_id();
        let pipeline = self.new_pipeline(
            new_frame_pipeline_id,
            Some((containing_page_pipeline_id, new_subpage_id)),
            script_pipeline,
            LoadData::new(url)
        );

        let rect = self.pending_sizes.remove(&(containing_page_pipeline_id, new_subpage_id));
        for frame_tree in frame_trees.iter() {
            self.create_or_update_child_pipeline(frame_tree.clone(),
                                                 pipeline.clone(),
                                                 rect,
                                                 old_subpage_id);
        }
        self.pipelines.insert(pipeline.id, pipeline);
    }

    fn handle_set_cursor_msg(&mut self, cursor: Cursor) {
        self.compositor_proxy.send(CompositorMsg::SetCursor(cursor))
    }

    fn handle_load_url_msg(&mut self, source_id: PipelineId, load_data: LoadData) {
        let url = load_data.url.to_string();
        debug!("Constellation: received message to load {:?}", url);
        // Make sure no pending page would be overridden.
        let source_frame = self.current_frame().as_ref().unwrap().find(source_id).expect(
            "Constellation: received a LoadUrl message from a pipeline_id associated
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
        // changes would be overridden by changing the subframe associated with source_id.

        let parent = source_frame.parent.clone();
        let parent_id = source_frame.pipeline.borrow().parent;
        let next_pipeline_id = self.get_next_pipeline_id();
        let next_frame_id = self.get_next_frame_id();
        let pipeline = self.new_pipeline(next_pipeline_id, parent_id, None, load_data);
        self.browse(Some(source_id),
                    Rc::new(FrameTree::new(next_frame_id,
                                           pipeline.clone(),
                                           parent.borrow().clone())),
                    NavigationType::Load);
        self.pipelines.insert(pipeline.id, pipeline);
    }

    fn handle_navigate_msg(&mut self, direction: constellation_msg::NavigationDirection) {
        debug!("received message to navigate {:?}", direction);

        // TODO(tkuehn): what is the "critical point" beyond which pending frames
        // should not be cleared? Currently, the behavior is that forward/back
        // navigation always has navigation priority, and after that new page loading is
        // first come, first served.
        let destination_frame = match direction {
            NavigationDirection::Forward => {
                if self.navigation_context.next.is_empty() {
                    debug!("no next page to navigate to");
                    return;
                } else {
                    let old = self.current_frame().as_ref().unwrap();
                    for frame in old.iter() {
                        frame.pipeline.borrow().revoke_paint_permission();
                    }
                }
                self.navigation_context.forward(&mut *self.compositor_proxy)
            }
            NavigationDirection::Back => {
                if self.navigation_context.previous.is_empty() {
                    debug!("no previous page to navigate to");
                    return;
                } else {
                    let old = self.current_frame().as_ref().unwrap();
                    for frame in old.iter() {
                        frame.pipeline.borrow().revoke_paint_permission();
                    }
                }
                self.navigation_context.back(&mut *self.compositor_proxy)
            }
        };

        for frame in destination_frame.iter() {
            frame.pipeline.borrow().load();
        }
        self.send_frame_tree_and_grant_paint_permission(destination_frame);

    }

    fn pipeline_is_in_current_frame(&self, pipeline_id: PipelineId) -> bool {
        self.current_frame().iter()
            .any(|current_frame| current_frame.contains(pipeline_id))
    }

    fn handle_key_msg(&self, key: Key, state: KeyState, mods: KeyModifiers) {
        match *self.current_frame() {
            Some(ref frame) => {
                let ScriptControlChan(ref chan) = frame.pipeline.borrow().script_chan;
                chan.send(ConstellationControlMsg::SendEvent(
                    frame.pipeline.borrow().id,
                    CompositorEvent::KeyEvent(key, state, mods))).unwrap();
            },
            None => self.compositor_proxy.clone_compositor_proxy()
                        .send(CompositorMsg::KeyEvent(key, state, mods))
        }

    }

    fn handle_get_pipeline_title_msg(&mut self, pipeline_id: PipelineId) {
        match self.pipelines.get(&pipeline_id) {
            None => self.compositor_proxy.send(CompositorMsg::ChangePageTitle(pipeline_id, None)),
            Some(pipeline) => {
                let ScriptControlChan(ref script_channel) = pipeline.script_chan;
                script_channel.send(ConstellationControlMsg::GetTitle(pipeline_id)).unwrap();
            }
        }
    }

    fn handle_painter_ready_msg(&mut self, pipeline_id: PipelineId) {
        debug!("Painter {:?} ready to send paint msg", pipeline_id);
        // This message could originate from a pipeline in the navigation context or
        // from a pending frame. The only time that we will grant paint permission is
        // when the message originates from a pending frame or the current frame.

        // Messages originating in the current frame are not navigations;
        // they may come from a page load in a subframe.
        if self.pipeline_is_in_current_frame(pipeline_id) {
            self.create_compositor_layer_for_iframe_if_necessary(pipeline_id);
            return;
        }

        // Find the pending frame change whose new pipeline id is pipeline_id.
        // If it is not found, it simply means that this pipeline will not receive
        // permission to paint.
        let pending_index = self.pending_frames.iter().rposition(|frame_change| {
            frame_change.after.pipeline.borrow().id == pipeline_id
        });
        match pending_index {
            Some(pending_index) => {
                let frame_change = self.pending_frames.swap_remove(pending_index);
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
                            frame.pipeline.borrow().revoke_paint_permission();
                        }

                        // If to_add is not the root frame, then replace revoked_frame with it.
                        // This conveniently keeps scissor rect size intact.
                        // NOTE: work around borrowchk issue
                        let mut flag = false;
                        {
                            if to_add.parent.borrow().is_some() {
                                debug!("Constellation: replacing {:?} with {:?} in {:?}",
                                       revoke_id, to_add.pipeline.borrow().id,
                                       next_frame_tree.pipeline.borrow().id);
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
                            let subpage_id = to_add.pipeline.borrow().subpage_id()
                                .expect("Constellation:
                                Child frame's subpage id is None. This should be impossible.");
                            let rect = self.pending_sizes.remove(&(parent.id, subpage_id));
                            let parent = next_frame_tree.find(parent.id).expect(
                                "Constellation: pending frame has a parent frame that is not
                                active. This is a bug.");
                            parent.add_child(ChildFrameTree::new(to_add.clone(), rect));
                        }
                    }
                }

                self.send_frame_tree_and_grant_paint_permission(next_frame_tree.clone());
                self.handle_evicted_frames_for_load_navigation(next_frame_tree,
                                                               frame_change.navigation_type);
            },
            None => (),
        }
    }

    /// Called when the window is resized.
    fn handle_resized_window_msg(&mut self, new_size: WindowSizeData) {
        let mut already_seen = HashSet::new();
        for frame_tree in self.current_frame().iter() {
            debug!("constellation sending resize message to active frame");
            let pipeline = &*frame_tree.pipeline.borrow();;
            let ScriptControlChan(ref chan) = pipeline.script_chan;
            let _ = chan.send(ConstellationControlMsg::Resize(pipeline.id, new_size));
            already_seen.insert(pipeline.id);
        }
        for frame_tree in self.navigation_context.previous.iter()
            .chain(self.navigation_context.next.iter()) {
            let pipeline = &*frame_tree.pipeline.borrow();
            if !already_seen.contains(&pipeline.id) {
                debug!("constellation sending resize message to inactive frame");
                let ScriptControlChan(ref chan) = pipeline.script_chan;
                let _ = chan.send(ConstellationControlMsg::ResizeInactive(pipeline.id, new_size));
                already_seen.insert(pipeline.id);
            }
        }

        // If there are any pending outermost frames, then tell them to resize. (This is how the
        // initial window size gets sent to the first page loaded, giving it permission to reflow.)
        for change in self.pending_frames.iter() {
            let frame_tree = &change.after;
            if frame_tree.parent.borrow().is_none() {
                debug!("constellation sending resize message to pending outer frame ({:?})",
                       frame_tree.pipeline.borrow().id);
                let ScriptControlChan(ref chan) = frame_tree.pipeline.borrow().script_chan;
                let _ = chan.send(ConstellationControlMsg::Resize(
                    frame_tree.pipeline.borrow().id, new_size));
            }
        }

        self.window_size = new_size;
    }

    // Close all pipelines at and beneath a given frame
    fn close_pipelines(&mut self, frame_tree: Rc<FrameTree>) {
        // TODO(tkuehn): should only exit once per unique script task,
        // and then that script task will handle sub-exits
        for frame_tree in frame_tree.iter() {
            frame_tree.pipeline.borrow().exit(PipelineExitType::PipelineOnly);
            self.compositor_proxy.send(CompositorMsg::PaintTaskExited(frame_tree.pipeline.borrow().id));
            self.pipelines.remove(&frame_tree.pipeline.borrow().id);
        }
    }

    fn handle_evicted_frames(&mut self, evicted_frames: Vec<Rc<FrameTree>>) {
        for frame_tree in evicted_frames.into_iter() {
            if !self.navigation_context.contains(frame_tree.pipeline.borrow().id) {
                self.close_pipelines(frame_tree);
            } else {
                let frames = frame_tree.children.borrow().iter()
                    .map(|child| child.frame_tree.clone()).collect();
                self.handle_evicted_frames(frames);
            }
        }
    }


    fn handle_evicted_frames_for_load_navigation(&mut self,
                                                 frame_tree: Rc<FrameTree>,
                                                 navigation_type: NavigationType) {
        // Don't call navigation_context.load() on a Navigate type (or None, as in the case of
        // parsed iframes that finish loading).
        match navigation_type {
            NavigationType::Load => {
                debug!("Evicting frames for NavigationType::Load");
                let evicted_frames = self.navigation_context.load(frame_tree,
                                                                  &mut *self.compositor_proxy);
                self.handle_evicted_frames(evicted_frames);
            }
            _ => {}
        }
    }

    // Grants a frame tree permission to paint; optionally updates navigation to reflect a new page
    fn send_frame_tree_and_grant_paint_permission(&mut self, frame_tree: Rc<FrameTree>) {
        debug!("Constellation sending SetFrameTree");
        let (chan, port) = channel();
        self.compositor_proxy.send(CompositorMsg::SetFrameTree(frame_tree.to_sendable(),
                                                               chan,
                                                               self.chan.clone()));
        if port.recv().is_err() {
            debug!("Compositor has discarded SetFrameTree");
            return; // Our message has been discarded, probably shutting down.
        }

        let iter = frame_tree.iter();
        for frame in iter {
            frame.has_compositor_layer.set(true);
            frame.pipeline.borrow().grant_paint_permission();
        }
    }

    fn find_child_parent_pair_in_frame_tree(&self,
                                            frame_tree: Rc<FrameTree>,
                                            child_pipeline_id: PipelineId)
                                            -> Option<(ChildFrameTree, Rc<FrameTree>)> {
        for child in frame_tree.children.borrow().iter() {
            let child_frame_tree = child.frame_tree.clone();
            if child.frame_tree.pipeline.borrow().id == child_pipeline_id {
                return Some((ChildFrameTree::new(child_frame_tree, child.rect),
                            frame_tree.clone()));
            }
            let result = self.find_child_parent_pair_in_frame_tree(child_frame_tree,
                                                                   child_pipeline_id);
            if result.is_some() {
                return result;
            }
        }
        None
    }

    fn create_compositor_layer_for_iframe_if_necessary(&mut self, pipeline_id: PipelineId) {
        let current_frame_tree = match self.current_frame() {
            &Some(ref tree) => tree.clone(),
            &None => return,
        };

        let pair = self.find_child_parent_pair_in_frame_tree(current_frame_tree,
                                                             pipeline_id);
        let (child, parent) = match pair {
            Some(pair) => pair,
            None => return,
        };

        if child.frame_tree.has_compositor_layer.get() {
            child.frame_tree.pipeline.borrow().grant_paint_permission();
            return;
        }

        let (chan, port) = channel();
        self.compositor_proxy.send(CompositorMsg::CreateRootLayerForPipeline(
            parent.pipeline.borrow().to_sendable(),
            child.frame_tree.pipeline.borrow().to_sendable(),
            child.rect,
            chan));
        match port.recv() {
            Ok(()) => {
                child.frame_tree.has_compositor_layer.set(true);
                child.frame_tree.pipeline.borrow().grant_paint_permission();
            }
            Err(_) => {} // The message has been discarded, we are probably shutting down.
        }
    }
}
