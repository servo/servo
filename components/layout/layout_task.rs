/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The layout task. Performs layout on the DOM, builds display lists and sends them to be
//! painted.

#![allow(unsafe_blocks)]

use css::node_style::StyledNode;
use construct::ConstructionResult;
use context::{SharedLayoutContext, SharedLayoutContextWrapper};
use flow::{self, Flow, ImmutableFlowUtils, MutableFlowUtils, MutableOwnedFlowUtils};
use flow_ref::FlowRef;
use fragment::{Fragment, FragmentBorderBoxIterator};
use incremental::{LayoutDamageComputation, REFLOW, REFLOW_ENTIRE_DOCUMENT, REPAINT};
use layout_debug;
use parallel::{self, UnsafeFlow};
use sequential;
use util::{LayoutDataAccess, LayoutDataWrapper, OpaqueNodeMethods, ToGfxColor};
use wrapper::{LayoutNode, TLayoutNode, ThreadSafeLayoutNode};

use encoding::EncodingRef;
use encoding::all::UTF_8;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use geom::scale_factor::ScaleFactor;
use gfx::color;
use gfx::display_list::{ClippingRegion, DisplayItemMetadata, DisplayList, OpaqueNode};
use gfx::display_list::{StackingContext};
use gfx::font_cache_task::FontCacheTask;
use gfx::paint_task::{PaintChan, PaintLayer};
use gfx::paint_task::Msg as PaintMsg;
use layout_traits::{LayoutControlMsg, LayoutTaskFactory};
use log;
use script::dom::bindings::js::LayoutJS;
use script::dom::node::{LayoutDataRef, Node, NodeTypeId};
use script::dom::element::ElementTypeId;
use script::dom::htmlelement::HTMLElementTypeId;
use script::layout_interface::{ContentBoxResponse, ContentBoxesResponse};
use script::layout_interface::ReflowQueryType;
use script::layout_interface::{HitTestResponse, LayoutChan, LayoutRPC};
use script::layout_interface::{MouseOverResponse, Msg};
use script::layout_interface::{Reflow, ReflowGoal, ScriptLayoutChan, TrustedNodeAddress};
use script_traits::{ConstellationControlMsg, CompositorEvent, OpaqueScriptLayoutChannel};
use script_traits::{ScriptControlChan, UntrustedNodeAddress};
use msg::compositor_msg::ScrollPolicy;
use msg::constellation_msg::Msg as ConstellationMsg;
use msg::constellation_msg::{ConstellationChan, Failure, PipelineExitType, PipelineId};
use net::image_cache_task::{ImageCacheTask, ImageResponseMsg};
use net::local_image_cache::{ImageResponder, LocalImageCache};
use net::resource_task::{ResourceTask, load_bytes_iter};
use servo_util::cursor::Cursor;
use servo_util::geometry::Au;
use servo_util::logical_geometry::LogicalPoint;
use servo_util::opts;
use servo_util::smallvec::{SmallVec, SmallVec1, VecLike};
use servo_util::task::spawn_named_with_send_on_failure;
use servo_util::task_state;
use servo_util::time::{TimeProfilerCategory, ProfilerMetadata, TimeProfilerChan};
use servo_util::time::{TimerMetadataFrameType, TimerMetadataReflowType, profile};
use servo_util::workqueue::WorkQueue;
use std::borrow::ToOwned;
use std::cell::Cell;
use std::ops::{Deref, DerefMut};
use std::sync::mpsc::{channel, Sender, Receiver, Select};
use std::mem;
use std::ptr;
use style::selector_matching::Stylist;
use style::computed_values::{filter, mix_blend_mode};
use style::stylesheets::{Origin, Stylesheet, iter_font_face_rules};
use style::node::TNode;
use style::media_queries::{MediaType, Device};
use std::sync::{Arc, Mutex, MutexGuard};
use url::Url;

/// Mutable data belonging to the LayoutTask.
///
/// This needs to be protected by a mutex so we can do fast RPCs.
pub struct LayoutTaskData {
    /// The local image cache.
    pub local_image_cache: Arc<Mutex<LocalImageCache<UntrustedNodeAddress>>>,

    /// The channel on which messages can be sent to the constellation.
    pub constellation_chan: ConstellationChan,

    /// The size of the viewport.
    pub screen_size: Size2D<Au>,

    /// The root stacking context.
    pub stacking_context: Option<Arc<StackingContext>>,

    pub stylist: Box<Stylist>,

    /// The workers that we use for parallel operation.
    pub parallel_traversal: Option<WorkQueue<SharedLayoutContextWrapper, UnsafeFlow>>,

    /// The dirty rect. Used during display list construction.
    pub dirty: Rect<Au>,

    /// Starts at zero, and increased by one every time a layout completes.
    /// This can be used to easily check for invalid stale data.
    pub generation: uint,

    /// A queued response for the union of the content boxes of a node.
    pub content_box_response: Rect<Au>,

    /// A queued response for the content boxes of a node.
    pub content_boxes_response: Vec<Rect<Au>>,
}

/// Information needed by the layout task.
pub struct LayoutTask {
    /// The ID of the pipeline that we belong to.
    pub id: PipelineId,

    /// The port on which we receive messages from the script task.
    pub port: Receiver<Msg>,

    /// The port on which we receive messages from the constellation
    pub pipeline_port: Receiver<LayoutControlMsg>,

    //// The channel to send messages to ourself.
    pub chan: LayoutChan,

    /// The channel on which messages can be sent to the script task.
    pub script_chan: ScriptControlChan,

    /// The channel on which messages can be sent to the painting task.
    pub paint_chan: PaintChan,

    /// The channel on which messages can be sent to the time profiler.
    pub time_profiler_chan: TimeProfilerChan,

    /// The channel on which messages can be sent to the resource task.
    pub resource_task: ResourceTask,

    /// The channel on which messages can be sent to the image cache.
    pub image_cache_task: ImageCacheTask,

    /// Public interface to the font cache task.
    pub font_cache_task: FontCacheTask,

    /// Is this the first reflow in this LayoutTask?
    pub first_reflow: Cell<bool>,

    /// A mutex to allow for fast, read-only RPC of layout's internal data
    /// structures, while still letting the LayoutTask modify them.
    ///
    /// All the other elements of this struct are read-only.
    pub rw_data: Arc<Mutex<LayoutTaskData>>,
}

struct LayoutImageResponder {
    id: PipelineId,
    script_chan: ScriptControlChan,
}

impl ImageResponder<UntrustedNodeAddress> for LayoutImageResponder {
    fn respond(&self) -> Box<Fn(ImageResponseMsg, UntrustedNodeAddress)+Send> {
        let id = self.id.clone();
        let script_chan = self.script_chan.clone();
        box move |&:_, node_address| {
            let ScriptControlChan(ref chan) = script_chan;
            debug!("Dirtying {:x}", node_address.0 as uint);
            let mut nodes = SmallVec1::new();
            nodes.vec_push(node_address);
            drop(chan.send(ConstellationControlMsg::SendEvent(
                id, CompositorEvent::ReflowEvent(nodes))))
        }
    }
}

impl LayoutTaskFactory for LayoutTask {
    /// Spawns a new layout task.
    fn create(_phantom: Option<&mut LayoutTask>,
                  id: PipelineId,
                  chan: OpaqueScriptLayoutChannel,
                  pipeline_port: Receiver<LayoutControlMsg>,
                  constellation_chan: ConstellationChan,
                  failure_msg: Failure,
                  script_chan: ScriptControlChan,
                  paint_chan: PaintChan,
                  resource_task: ResourceTask,
                  img_cache_task: ImageCacheTask,
                  font_cache_task: FontCacheTask,
                  time_profiler_chan: TimeProfilerChan,
                  shutdown_chan: Sender<()>) {
        let ConstellationChan(con_chan) = constellation_chan.clone();
        spawn_named_with_send_on_failure("LayoutTask", task_state::LAYOUT, move || {
            { // Ensures layout task is destroyed before we send shutdown message
                let sender = chan.sender();
                let layout =
                    LayoutTask::new(
                        id,
                        chan.receiver(),
                        LayoutChan(sender),
                        pipeline_port,
                        constellation_chan,
                        script_chan,
                        paint_chan,
                        resource_task,
                        img_cache_task,
                        font_cache_task,
                        time_profiler_chan);
                layout.start();
            }
            shutdown_chan.send(()).unwrap();
        }, ConstellationMsg::Failure(failure_msg), con_chan);
    }
}

/// The `LayoutTask` `rw_data` lock must remain locked until the first reflow,
/// as RPC calls don't make sense until then. Use this in combination with
/// `LayoutTask::lock_rw_data` and `LayoutTask::return_rw_data`.
enum RWGuard<'a> {
    /// If the lock was previously held, from when the task started.
    Held(MutexGuard<'a, LayoutTaskData>),
    /// If the lock was just used, and has been returned since there has been
    /// a reflow already.
    Used(MutexGuard<'a, LayoutTaskData>),
}

impl<'a> Deref for RWGuard<'a> {
    type Target = LayoutTaskData;
    fn deref(&self) -> &LayoutTaskData {
        match *self {
            RWGuard::Held(ref x) => &**x,
            RWGuard::Used(ref x) => &**x,
        }
    }
}

impl<'a> DerefMut for RWGuard<'a> {
    fn deref_mut(&mut self) -> &mut LayoutTaskData {
        match *self {
            RWGuard::Held(ref mut x) => &mut **x,
            RWGuard::Used(ref mut x) => &mut **x,
        }
    }
}

impl LayoutTask {
    /// Creates a new `LayoutTask` structure.
    fn new(id: PipelineId,
           port: Receiver<Msg>,
           chan: LayoutChan,
           pipeline_port: Receiver<LayoutControlMsg>,
           constellation_chan: ConstellationChan,
           script_chan: ScriptControlChan,
           paint_chan: PaintChan,
           resource_task: ResourceTask,
           image_cache_task: ImageCacheTask,
           font_cache_task: FontCacheTask,
           time_profiler_chan: TimeProfilerChan)
           -> LayoutTask {
        let local_image_cache =
            Arc::new(Mutex::new(LocalImageCache::new(image_cache_task.clone())));
        let screen_size = Size2D(Au(0), Au(0));
        let device = Device::new(MediaType::Screen, opts::get().initial_window_size.as_f32() * ScaleFactor(1.0));
        let parallel_traversal = if opts::get().layout_threads != 1 {
            Some(WorkQueue::new("LayoutWorker", task_state::LAYOUT,
                                opts::get().layout_threads, SharedLayoutContextWrapper(ptr::null())))
        } else {
            None
        };

        LayoutTask {
            id: id,
            port: port,
            pipeline_port: pipeline_port,
            chan: chan,
            script_chan: script_chan,
            paint_chan: paint_chan,
            time_profiler_chan: time_profiler_chan,
            resource_task: resource_task,
            image_cache_task: image_cache_task.clone(),
            font_cache_task: font_cache_task,
            first_reflow: Cell::new(true),
            rw_data: Arc::new(Mutex::new(
                LayoutTaskData {
                    local_image_cache: local_image_cache,
                    constellation_chan: constellation_chan,
                    screen_size: screen_size,
                    stacking_context: None,
                    stylist: box Stylist::new(device),
                    parallel_traversal: parallel_traversal,
                    dirty: Rect::zero(),
                    generation: 0,
                    content_box_response: Rect::zero(),
                    content_boxes_response: Vec::new(),
              })),
        }
    }

    /// Starts listening on the port.
    fn start(self) {
        let mut possibly_locked_rw_data = Some((*self.rw_data).lock().unwrap());
        while self.handle_request(&mut possibly_locked_rw_data) {
            // Loop indefinitely.
        }
    }

    // Create a layout context for use in building display lists, hit testing, &c.
    fn build_shared_layout_context(&self,
                                   rw_data: &LayoutTaskData,
                                   reflow_root: &LayoutNode,
                                   url: &Url)
                                   -> SharedLayoutContext {
        SharedLayoutContext {
            image_cache: rw_data.local_image_cache.clone(),
            screen_size: rw_data.screen_size.clone(),
            constellation_chan: rw_data.constellation_chan.clone(),
            layout_chan: self.chan.clone(),
            font_cache_task: self.font_cache_task.clone(),
            stylist: &*rw_data.stylist,
            url: (*url).clone(),
            reflow_root: OpaqueNodeMethods::from_layout_node(reflow_root),
            dirty: Rect::zero(),
            generation: rw_data.generation,
        }
    }

    /// Receives and dispatches messages from the script and constellation tasks
    fn handle_request<'a>(&'a self,
                          possibly_locked_rw_data: &mut Option<MutexGuard<'a, LayoutTaskData>>)
                          -> bool {
        enum PortToRead {
            Pipeline,
            Script,
        }

        let port_to_read = {
            let sel = Select::new();
            let mut port1 = sel.handle(&self.port);
            let mut port2 = sel.handle(&self.pipeline_port);
            unsafe {
                port1.add();
                port2.add();
            }
            let ret = sel.wait();
            if ret == port1.id() {
                PortToRead::Script
            } else if ret == port2.id() {
                PortToRead::Pipeline
            } else {
                panic!("invalid select result");
            }
        };

        match port_to_read {
            PortToRead::Pipeline => {
                match self.pipeline_port.recv().unwrap() {
                    LayoutControlMsg::ExitNowMsg(exit_type) => {
                        self.handle_script_request(Msg::ExitNow(exit_type), possibly_locked_rw_data)
                    }
                }
            },
            PortToRead::Script => {
                let msg = self.port.recv().unwrap();
                self.handle_script_request(msg, possibly_locked_rw_data)
            }
        }
    }

    /// If no reflow has happened yet, this will just return the lock in
    /// `possibly_locked_rw_data`. Otherwise, it will acquire the `rw_data` lock.
    ///
    /// If you do not wish RPCs to remain blocked, just drop the `RWGuard`
    /// returned from this function. If you _do_ wish for them to remain blocked,
    /// use `return_rw_data`.
    fn lock_rw_data<'a>(&'a self,
                        possibly_locked_rw_data: &mut Option<MutexGuard<'a, LayoutTaskData>>)
                        -> RWGuard<'a> {
        match possibly_locked_rw_data.take() {
            None    => RWGuard::Used((*self.rw_data).lock().unwrap()),
            Some(x) => RWGuard::Held(x),
        }
    }

    /// If no reflow has ever been triggered, this will keep the lock, locked
    /// (and saved in `possibly_locked_rw_data`). If it has been, the lock will
    /// be unlocked.
    fn return_rw_data<'a>(possibly_locked_rw_data: &mut Option<MutexGuard<'a, LayoutTaskData>>,
                          rw_data: RWGuard<'a>) {
        match rw_data {
            RWGuard::Used(x) => drop(x),
            RWGuard::Held(x) => *possibly_locked_rw_data = Some(x),
        }
    }

    /// Receives and dispatches messages from the script task.
    fn handle_script_request<'a>(&'a self,
                                 request: Msg,
                                 possibly_locked_rw_data: &mut Option<MutexGuard<'a,
                                                                                 LayoutTaskData>>)
                                 -> bool {
        match request {
            Msg::AddStylesheet(sheet) => self.handle_add_stylesheet(sheet, possibly_locked_rw_data),
            Msg::LoadStylesheet(url) => self.handle_load_stylesheet(url, possibly_locked_rw_data),
            Msg::SetQuirksMode => self.handle_set_quirks_mode(possibly_locked_rw_data),
            Msg::GetRPC(response_chan) => {
                response_chan.send(box LayoutRPCImpl(self.rw_data.clone()) as
                                   Box<LayoutRPC + Send>).unwrap();
            },
            Msg::Reflow(data) => {
                profile(TimeProfilerCategory::LayoutPerform,
                        self.profiler_metadata(&*data),
                        self.time_profiler_chan.clone(),
                        || self.handle_reflow(&*data, possibly_locked_rw_data));
            },
            Msg::ReapLayoutData(dead_layout_data) => {
                unsafe {
                    LayoutTask::handle_reap_layout_data(dead_layout_data)
                }
            },
            Msg::PrepareToExit(response_chan) => {
                debug!("layout: PrepareToExitMsg received");
                self.prepare_to_exit(response_chan, possibly_locked_rw_data);
                return false
            },
            Msg::ExitNow(exit_type) => {
                debug!("layout: ExitNowMsg received");
                self.exit_now(possibly_locked_rw_data, exit_type);
                return false
            }
        }

        true
    }

    /// Enters a quiescent state in which no new messages except for `layout_interface::Msg::ReapLayoutData` will be
    /// processed until an `ExitNowMsg` is received. A pong is immediately sent on the given
    /// response channel.
    fn prepare_to_exit<'a>(&'a self,
                           response_chan: Sender<()>,
                           possibly_locked_rw_data: &mut Option<MutexGuard<'a, LayoutTaskData>>) {
        response_chan.send(()).unwrap();
        loop {
            match self.port.recv().unwrap() {
                Msg::ReapLayoutData(dead_layout_data) => {
                    unsafe {
                        LayoutTask::handle_reap_layout_data(dead_layout_data)
                    }
                }
                Msg::ExitNow(exit_type) => {
                    debug!("layout task is exiting...");
                    self.exit_now(possibly_locked_rw_data, exit_type);
                    break
                }
                _ => {
                    panic!("layout: message that wasn't `ExitNowMsg` received after \
                           `PrepareToExitMsg`")
                }
            }
        }
    }

    /// Shuts down the layout task now. If there are any DOM nodes left, layout will now (safely)
    /// crash.
    fn exit_now<'a>(&'a self,
                    possibly_locked_rw_data: &mut Option<MutexGuard<'a, LayoutTaskData>>,
                    exit_type: PipelineExitType) {
        let (response_chan, response_port) = channel();

        {
            let mut rw_data = self.lock_rw_data(possibly_locked_rw_data);
            match (&mut *rw_data).parallel_traversal {
                None => {}
                Some(ref mut traversal) => traversal.shutdown(),
            }
            LayoutTask::return_rw_data(possibly_locked_rw_data, rw_data);
        }

        self.paint_chan.send(PaintMsg::Exit(Some(response_chan), exit_type));
        response_port.recv().unwrap()
    }

    fn handle_load_stylesheet<'a>(&'a self,
                                  url: Url,
                                  possibly_locked_rw_data:
                                    &mut Option<MutexGuard<'a, LayoutTaskData>>) {
        // TODO: Get the actual value. http://dev.w3.org/csswg/css-syntax/#environment-encoding
        let environment_encoding = UTF_8 as EncodingRef;

        let (metadata, iter) = load_bytes_iter(&self.resource_task, url);
        let protocol_encoding_label = metadata.charset.as_ref().map(|s| s.as_slice());
        let final_url = metadata.final_url;

        let sheet = Stylesheet::from_bytes_iter(iter,
                                                final_url,
                                                protocol_encoding_label,
                                                Some(environment_encoding),
                                                Origin::Author);
        self.handle_add_stylesheet(sheet, possibly_locked_rw_data);
    }

    fn handle_add_stylesheet<'a>(&'a self,
                                 sheet: Stylesheet,
                                 possibly_locked_rw_data:
                                    &mut Option<MutexGuard<'a, LayoutTaskData>>) {
        // Find all font-face rules and notify the font cache of them.
        // GWTODO: Need to handle unloading web fonts (when we handle unloading stylesheets!)
        let mut rw_data = self.lock_rw_data(possibly_locked_rw_data);
        iter_font_face_rules(&sheet, &rw_data.stylist.device, &|&:family, src| {
            self.font_cache_task.add_web_font(family.to_owned(), (*src).clone());
        });
        rw_data.stylist.add_stylesheet(sheet);
        LayoutTask::return_rw_data(possibly_locked_rw_data, rw_data);
    }

    /// Sets quirks mode for the document, causing the quirks mode stylesheet to be loaded.
    fn handle_set_quirks_mode<'a>(&'a self,
                                  possibly_locked_rw_data:
                                    &mut Option<MutexGuard<'a, LayoutTaskData>>) {
        let mut rw_data = self.lock_rw_data(possibly_locked_rw_data);
        rw_data.stylist.add_quirks_mode_stylesheet();
        LayoutTask::return_rw_data(possibly_locked_rw_data, rw_data);
    }

    /// Retrieves the flow tree root from the root node.
    fn try_get_layout_root(&self, node: LayoutNode) -> Option<FlowRef> {
        let mut layout_data_ref = node.mutate_layout_data();
        let layout_data =
            match layout_data_ref.as_mut() {
                None              => return None,
                Some(layout_data) => layout_data,
            };

        let result = layout_data.data.flow_construction_result.swap_out();

        let mut flow = match result {
            ConstructionResult::Flow(mut flow, abs_descendants) => {
                // Note: Assuming that the root has display 'static' (as per
                // CSS Section 9.3.1). Otherwise, if it were absolutely
                // positioned, it would return a reference to itself in
                // `abs_descendants` and would lead to a circular reference.
                // Set Root as CB for any remaining absolute descendants.
                flow.set_absolute_descendants(abs_descendants);
                flow
            }
            _ => return None,
        };

        flow.mark_as_root();

        Some(flow)
    }

    fn get_layout_root(&self, node: LayoutNode) -> FlowRef {
        self.try_get_layout_root(node).expect("no layout root")
    }

    /// Performs layout constraint solving.
    ///
    /// This corresponds to `Reflow()` in Gecko and `layout()` in WebKit/Blink and should be
    /// benchmarked against those two. It is marked `#[inline(never)]` to aid profiling.
    #[inline(never)]
    fn solve_constraints<'a>(&self,
                         layout_root: &mut FlowRef,
                         shared_layout_context: &SharedLayoutContext) {
        let _scope = layout_debug_scope!("solve_constraints");
        sequential::traverse_flow_tree_preorder(layout_root, shared_layout_context);
    }

    /// Performs layout constraint solving in parallel.
    ///
    /// This corresponds to `Reflow()` in Gecko and `layout()` in WebKit/Blink and should be
    /// benchmarked against those two. It is marked `#[inline(never)]` to aid profiling.
    #[inline(never)]
    fn solve_constraints_parallel(&self,
                                  data: &Reflow,
                                  rw_data: &mut LayoutTaskData,
                                  layout_root: &mut FlowRef,
                                  shared_layout_context: &SharedLayoutContext) {
        let _scope = layout_debug_scope!("solve_constraints_parallel");

        match rw_data.parallel_traversal {
            None => panic!("solve_contraints_parallel() called with no parallel traversal ready"),
            Some(ref mut traversal) => {
                // NOTE: this currently computes borders, so any pruning should separate that
                // operation out.
                parallel::traverse_flow_tree_preorder(layout_root,
                                                      self.profiler_metadata(data),
                                                      self.time_profiler_chan.clone(),
                                                      shared_layout_context,
                                                      traversal);
            }
        }
    }

    /// Verifies that every node was either marked as a leaf or as a nonleaf in the flow tree.
    /// This is only on in debug builds.
    #[inline(never)]
    #[cfg(debug)]
    fn verify_flow_tree(&self, layout_root: &mut FlowRef) {
        let mut traversal = traversal::FlowTreeVerification;
        layout_root.traverse_preorder(&mut traversal);
    }

    #[cfg(not(debug))]
    fn verify_flow_tree(&self, _: &mut FlowRef) {
    }

    fn process_content_box_request<'a>(&'a self,
                                       requested_node: TrustedNodeAddress,
                                       layout_root: &mut FlowRef,
                                       rw_data: &mut RWGuard<'a>) {
        // FIXME(pcwalton): This has not been updated to handle the stacking context relative
        // stuff. So the position is wrong in most cases.
        let requested_node: OpaqueNode = OpaqueNodeMethods::from_script_node(requested_node);
        let mut iterator = UnioningFragmentBorderBoxIterator::new(requested_node);
        sequential::iterate_through_flow_tree_fragment_border_boxes(layout_root, &mut iterator);
        rw_data.content_box_response = iterator.rect;
    }

    fn process_content_boxes_request<'a>(&'a self,
                                         requested_node: TrustedNodeAddress,
                                         layout_root: &mut FlowRef,
                                         rw_data: &mut RWGuard<'a>) {
        // FIXME(pcwalton): This has not been updated to handle the stacking context relative
        // stuff. So the position is wrong in most cases.
        let requested_node: OpaqueNode = OpaqueNodeMethods::from_script_node(requested_node);
        let mut iterator = CollectingFragmentBorderBoxIterator::new(requested_node);
        sequential::iterate_through_flow_tree_fragment_border_boxes(layout_root, &mut iterator);
        rw_data.content_boxes_response = iterator.rects;
    }

    fn build_display_list_for_reflow<'a>(&'a self,
                                         data: &Reflow,
                                         node: &mut LayoutNode,
                                         layout_root: &mut FlowRef,
                                         shared_layout_context: &mut SharedLayoutContext,
                                         rw_data: &mut RWGuard<'a>) {
        let writing_mode = flow::base(&**layout_root).writing_mode;
        profile(TimeProfilerCategory::LayoutDispListBuild,
                self.profiler_metadata(data),
                self.time_profiler_chan.clone(),
                || {
            shared_layout_context.dirty =
                flow::base(&**layout_root).position.to_physical(writing_mode,
                                                                     rw_data.screen_size);
            flow::mut_base(&mut **layout_root).stacking_relative_position =
                LogicalPoint::zero(writing_mode).to_physical(writing_mode,
                                                             rw_data.screen_size);

            flow::mut_base(&mut **layout_root).clip =
                ClippingRegion::from_rect(&data.page_clip_rect);

            let rw_data = &mut **rw_data;
            match rw_data.parallel_traversal {
                None => {
                    sequential::build_display_list_for_subtree(layout_root, shared_layout_context);
                }
                Some(ref mut traversal) => {
                    parallel::build_display_list_for_subtree(layout_root,
                                                             self.profiler_metadata(data),
                                                             self.time_profiler_chan.clone(),
                                                             shared_layout_context,
                                                             traversal);
                }
            }

            debug!("Done building display list.");

            // FIXME(pcwalton): This is really ugly and can't handle overflow: scroll. Refactor
            // it with extreme prejudice.
            let mut color = color::white();
            for child in node.traverse_preorder() {
                if child.type_id() == Some(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLHtmlElement))) ||
                        child.type_id() == Some(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLBodyElement))) {
                    let element_bg_color = {
                        let thread_safe_child = ThreadSafeLayoutNode::new(&child);
                        thread_safe_child.style()
                                         .resolve_color(thread_safe_child.style()
                                                                         .get_background()
                                                                         .background_color)
                                         .to_gfx_color()
                    };

                    let black = color::transparent_black();
                    if element_bg_color != black {

                        color = element_bg_color;
                        break;
                    }
                }
            }

            let root_size = {
                let root_flow = flow::base(&**layout_root);
                root_flow.position.size.to_physical(root_flow.writing_mode)
            };
            let mut display_list = box DisplayList::new();
            flow::mut_base(&mut **layout_root).display_list_building_result
                                              .add_to(&mut *display_list);
            let paint_layer = Arc::new(PaintLayer::new(layout_root.layer_id(0),
                                                       color,
                                                       ScrollPolicy::Scrollable));
            let origin = Rect(Point2D(Au(0), Au(0)), root_size);
            let stacking_context = Arc::new(StackingContext::new(display_list,
                                                                 &origin,
                                                                 &origin,
                                                                 0,
                                                                 filter::T::new(Vec::new()),
                                                                 mix_blend_mode::T::normal,
                                                                 Some(paint_layer)));

            rw_data.stacking_context = Some(stacking_context.clone());

            debug!("Layout done!");

            self.paint_chan.send(PaintMsg::PaintInit(stacking_context));
        });
    }

    /// The high-level routine that performs layout tasks.
    fn handle_reflow<'a>(&'a self,
                         data: &Reflow,
                         possibly_locked_rw_data: &mut Option<MutexGuard<'a, LayoutTaskData>>) {
        // FIXME: Isolate this transmutation into a "bridge" module.
        // FIXME(rust#16366): The following line had to be moved because of a
        // rustc bug. It should be in the next unsafe block.
        let mut node: LayoutJS<Node> = unsafe {
            LayoutJS::from_trusted_node_address(data.document_root)
        };
        let node: &mut LayoutNode = unsafe {
            mem::transmute(&mut node)
        };

        debug!("layout: received layout request for: {}", data.url.serialize());
        debug!("layout: parsed Node tree");
        if log_enabled!(log::DEBUG) {
            node.dump();
        }

        let mut rw_data = self.lock_rw_data(possibly_locked_rw_data);

        {
            // Reset the image cache.
            let mut local_image_cache = rw_data.local_image_cache.lock().unwrap();
            local_image_cache.next_round(self.make_on_image_available_cb());
        }

        // TODO: Calculate the "actual viewport":
        // http://www.w3.org/TR/css-device-adapt/#actual-viewport
        let viewport_size = data.window_size.initial_viewport;

        let old_screen_size = rw_data.screen_size;
        let current_screen_size = Size2D(Au::from_frac32_px(viewport_size.width.get()),
                                         Au::from_frac32_px(viewport_size.height.get()));
        rw_data.screen_size = current_screen_size;

        // Create a layout context for use throughout the following passes.
        let mut shared_layout_context = self.build_shared_layout_context(&*rw_data,
                                                                         node,
                                                                         &data.url);

        // Handle conditions where the entire flow tree is invalid.
        let screen_size_changed = current_screen_size != old_screen_size;

        if screen_size_changed {
            let device = Device::new(MediaType::Screen, data.window_size.initial_viewport);
            rw_data.stylist.set_device(device);
        }

        let needs_dirtying = rw_data.stylist.update();

        // If the entire flow tree is invalid, then it will be reflowed anyhow.
        let needs_reflow = screen_size_changed && !needs_dirtying;

        unsafe {
            if needs_dirtying {
                LayoutTask::dirty_all_nodes(node);
            }
        }

        if needs_reflow {
            self.try_get_layout_root(*node).map(
                |mut flow| LayoutTask::reflow_all_nodes(&mut *flow));
        }

        let mut layout_root = profile(TimeProfilerCategory::LayoutStyleRecalc,
                                      self.profiler_metadata(data),
                                      self.time_profiler_chan.clone(),
                                      || {
            // Perform CSS selector matching and flow construction.
            let rw_data = &mut *rw_data;
            match rw_data.parallel_traversal {
                None => {
                    sequential::traverse_dom_preorder(*node, &shared_layout_context);
                }
                Some(ref mut traversal) => {
                    parallel::traverse_dom_preorder(*node, &shared_layout_context, traversal)
                }
            }

            self.get_layout_root((*node).clone())
        });

        profile(TimeProfilerCategory::LayoutRestyleDamagePropagation,
                self.profiler_metadata(data),
                self.time_profiler_chan.clone(),
                || {
            if opts::get().nonincremental_layout || layout_root.compute_layout_damage()
                                                               .contains(REFLOW_ENTIRE_DOCUMENT) {
                layout_root.reflow_entire_document()
            }
        });

        // Verification of the flow tree, which ensures that all nodes were either marked as leaves
        // or as non-leaves. This becomes a no-op in release builds. (It is inconsequential to
        // memory safety but is a useful debugging tool.)
        self.verify_flow_tree(&mut layout_root);

        if opts::get().trace_layout {
            layout_debug::begin_trace(layout_root.clone());
        }

        // Perform the primary layout passes over the flow tree to compute the locations of all
        // the boxes.
        profile(TimeProfilerCategory::LayoutMain,
                self.profiler_metadata(data),
                self.time_profiler_chan.clone(),
                || {
            let rw_data = &mut *rw_data;
            match rw_data.parallel_traversal {
                None => {
                    // Sequential mode.
                    self.solve_constraints(&mut layout_root, &shared_layout_context)
                }
                Some(_) => {
                    // Parallel mode.
                    self.solve_constraints_parallel(data,
                                                    rw_data,
                                                    &mut layout_root,
                                                    &mut shared_layout_context);
                }
            }
        });

        // Build the display list if necessary, and send it to the painter.
        match data.goal {
            ReflowGoal::ForDisplay => {
                self.build_display_list_for_reflow(data,
                                                   node,
                                                   &mut layout_root,
                                                   &mut shared_layout_context,
                                                   &mut rw_data);
            }
            ReflowGoal::ForScriptQuery => {}
        }

        match data.query_type {
            ReflowQueryType::ContentBoxQuery(node) => {
                self.process_content_box_request(node, &mut layout_root, &mut rw_data)
            }
            ReflowQueryType::ContentBoxesQuery(node) => {
                self.process_content_boxes_request(node, &mut layout_root, &mut rw_data)
            }
            ReflowQueryType::NoQuery => {}
        }

        self.first_reflow.set(false);

        if opts::get().trace_layout {
            layout_debug::end_trace();
        }

        if opts::get().dump_flow_tree {
            layout_root.dump();
        }

        rw_data.generation += 1;

        // Tell script that we're done.
        //
        // FIXME(pcwalton): This should probably be *one* channel, but we can't fix this without
        // either select or a filtered recv() that only looks for messages of a given type.
        data.script_join_chan.send(()).unwrap();
        let ScriptControlChan(ref chan) = data.script_chan;
        chan.send(ConstellationControlMsg::ReflowComplete(self.id, data.id)).unwrap();
    }

    unsafe fn dirty_all_nodes(node: &mut LayoutNode) {
        for node in node.traverse_preorder() {
            // TODO(cgaebel): mark nodes which are sensitive to media queries as
            // "changed":
            // > node.set_changed(true);
            node.set_dirty(true);
            node.set_dirty_siblings(true);
            node.set_dirty_descendants(true);
        }
    }

    fn reflow_all_nodes(flow: &mut Flow) {
        flow::mut_base(flow).restyle_damage.insert(REFLOW | REPAINT);

        for child in flow::child_iter(flow) {
            LayoutTask::reflow_all_nodes(child);
        }
    }

    /// When images can't be loaded in time to display they trigger
    /// this callback in some task somewhere. This will send a message
    /// to the script task, and ultimately cause the image to be
    /// re-requested. We probably don't need to go all the way back to
    /// the script task for this.
    ///
    /// FIXME(pcwalton): Rewrite all of this.
    fn make_on_image_available_cb(&self) -> Box<ImageResponder<UntrustedNodeAddress>+Send> {
        // This has a crazy signature because the image cache needs to
        // make multiple copies of the callback, and the dom event
        // channel is not a copyable type, so this is actually a
        // little factory to produce callbacks
        box LayoutImageResponder {
            id: self.id.clone(),
            script_chan: self.script_chan.clone(),
        } as Box<ImageResponder<UntrustedNodeAddress>+Send>
    }

    /// Handles a message to destroy layout data. Layout data must be destroyed on *this* task
    /// because it contains local managed pointers.
    unsafe fn handle_reap_layout_data(layout_data: LayoutDataRef) {
        let mut layout_data_ref = layout_data.borrow_mut();
        let _: Option<LayoutDataWrapper> = mem::transmute(
            mem::replace(&mut *layout_data_ref, None));
    }

    /// Returns profiling information which is passed to the time profiler.
    fn profiler_metadata<'a>(&self, data: &'a Reflow) -> ProfilerMetadata<'a> {
        Some((&data.url,
              if data.iframe {
                TimerMetadataFrameType::IFrame
              } else {
                TimerMetadataFrameType::RootWindow
              },
              if self.first_reflow.get() {
                TimerMetadataReflowType::FirstReflow
              } else {
                TimerMetadataReflowType::Incremental
              }))
    }
}

struct LayoutRPCImpl(Arc<Mutex<LayoutTaskData>>);

impl LayoutRPC for LayoutRPCImpl {
    // The neat thing here is that in order to answer the following two queries we only
    // need to compare nodes for equality. Thus we can safely work only with `OpaqueNode`.
    fn content_box(&self) -> ContentBoxResponse {
        let &LayoutRPCImpl(ref rw_data) = self;
        let rw_data = rw_data.lock().unwrap();
        ContentBoxResponse(rw_data.content_box_response)
    }

    /// Requests the dimensions of all the content boxes, as in the `getClientRects()` call.
    fn content_boxes(&self) -> ContentBoxesResponse {
        let &LayoutRPCImpl(ref rw_data) = self;
        let rw_data = rw_data.lock().unwrap();
        ContentBoxesResponse(rw_data.content_boxes_response.clone())
    }

    /// Requests the node containing the point of interest.
    fn hit_test(&self, _: TrustedNodeAddress, point: Point2D<f32>) -> Result<HitTestResponse, ()> {
        let point = Point2D(Au::from_frac_px(point.x as f64), Au::from_frac_px(point.y as f64));
        let resp = {
            let &LayoutRPCImpl(ref rw_data) = self;
            let rw_data = rw_data.lock().unwrap();
            match rw_data.stacking_context {
                None => panic!("no root stacking context!"),
                Some(ref stacking_context) => {
                    let mut result = Vec::new();
                    stacking_context.hit_test(point, &mut result, true);
                    if !result.is_empty() {
                        Some(HitTestResponse(result[0].node.to_untrusted_node_address()))
                    } else {
                        None
                    }
                }
            }
        };

        if resp.is_some() {
            return Ok(resp.unwrap());
        }
        Err(())
    }

    fn mouse_over(&self, _: TrustedNodeAddress, point: Point2D<f32>)
                  -> Result<MouseOverResponse, ()> {
        let mut mouse_over_list: Vec<DisplayItemMetadata> = vec!();
        let point = Point2D(Au::from_frac_px(point.x as f64), Au::from_frac_px(point.y as f64));
        {
            let &LayoutRPCImpl(ref rw_data) = self;
            let rw_data = rw_data.lock().unwrap();
            match rw_data.stacking_context {
                None => panic!("no root stacking context!"),
                Some(ref stacking_context) => {
                    stacking_context.hit_test(point, &mut mouse_over_list, false);
                }
            }

            // Compute the new cursor.
            let cursor = if !mouse_over_list.is_empty() {
                mouse_over_list[0].pointing.unwrap()
            } else {
                Cursor::DefaultCursor
            };
            let ConstellationChan(ref constellation_chan) = rw_data.constellation_chan;
            constellation_chan.send(ConstellationMsg::SetCursor(cursor)).unwrap();
        }

        if mouse_over_list.is_empty() {
            Err(())
        } else {
            let response_list =
                mouse_over_list.iter()
                               .map(|metadata| metadata.node.to_untrusted_node_address())
                               .collect();
            Ok(MouseOverResponse(response_list))
        }
    }
}

struct UnioningFragmentBorderBoxIterator {
    node_address: OpaqueNode,
    rect: Rect<Au>,
}

impl UnioningFragmentBorderBoxIterator {
    fn new(node_address: OpaqueNode) -> UnioningFragmentBorderBoxIterator {
        UnioningFragmentBorderBoxIterator {
            node_address: node_address,
            rect: Rect::zero(),
        }
    }
}

impl FragmentBorderBoxIterator for UnioningFragmentBorderBoxIterator {
    fn process(&mut self, _: &Fragment, border_box: &Rect<Au>) {
        self.rect = if self.rect.is_empty() {
            *border_box
        } else {
            self.rect.union(border_box)
        }
    }

    fn should_process(&mut self, fragment: &Fragment) -> bool {
        self.node_address == fragment.node
    }
}

struct CollectingFragmentBorderBoxIterator {
    node_address: OpaqueNode,
    rects: Vec<Rect<Au>>,
}

impl CollectingFragmentBorderBoxIterator {
    fn new(node_address: OpaqueNode) -> CollectingFragmentBorderBoxIterator {
        CollectingFragmentBorderBoxIterator {
            node_address: node_address,
            rects: Vec::new(),
        }
    }
}

impl FragmentBorderBoxIterator for CollectingFragmentBorderBoxIterator {
    fn process(&mut self, _: &Fragment, border_box: &Rect<Au>) {
        self.rects.push(*border_box);
    }

    fn should_process(&mut self, fragment: &Fragment) -> bool {
        self.node_address == fragment.node
    }
}
