/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The layout task. Performs layout on the DOM, builds display lists and sends them to be
//! rendered.

use css::node_style::StyledNode;
use construct::FlowConstructionResult;
use context::SharedLayoutContext;
use flow::{mod, Flow, ImmutableFlowUtils, MutableFlowUtils, MutableOwnedFlowUtils};
use flow_ref::FlowRef;
use fragment::{Fragment, FragmentBoundsIterator};
use incremental::{LayoutDamageComputation, REFLOW, REFLOW_ENTIRE_DOCUMENT, REPAINT};
use layout_debug;
use parallel::UnsafeFlow;
use parallel;
use sequential;
use util::{LayoutDataAccess, LayoutDataWrapper, OpaqueNodeMethods, ToGfxColor};
use wrapper::{LayoutNode, TLayoutNode, ThreadSafeLayoutNode};

use encoding::EncodingRef;
use encoding::all::UTF_8;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use geom::scale_factor::ScaleFactor;
use gfx::display_list::{DisplayList, OpaqueNode, StackingContext};
use gfx::render_task::{RenderInitMsg, RenderChan, RenderLayer};
use gfx::{render_task, color};
use layout_traits;
use layout_traits::{LayoutControlMsg, LayoutTaskFactory};
use log;
use script::dom::bindings::js::JS;
use script::dom::node::{ElementNodeTypeId, LayoutDataRef, Node};
use script::dom::element::{HTMLBodyElementTypeId, HTMLHtmlElementTypeId};
use script::layout_interface::{
    AddStylesheetMsg, ContentBoxResponse, ContentBoxesResponse, ContentBoxesQuery,
    ContentBoxQuery, ExitNowMsg, GetRPCMsg, HitTestResponse, LayoutChan, LayoutRPC,
    LoadStylesheetMsg, MouseOverResponse, Msg, NoQuery, PrepareToExitMsg, ReapLayoutDataMsg,
    Reflow, ReflowForDisplay, ReflowMsg, ScriptLayoutChan, TrustedNodeAddress,
};
use script_traits::{SendEventMsg, ReflowEvent, ReflowCompleteMsg, OpaqueScriptLayoutChannel};
use script_traits::{ScriptControlChan, UntrustedNodeAddress};
use servo_msg::compositor_msg::Scrollable;
use servo_msg::constellation_msg::{ConstellationChan, PipelineId, Failure, FailureMsg};
use servo_net::image_cache_task::{ImageCacheTask, ImageResponseMsg};
use gfx::font_cache_task::{FontCacheTask};
use servo_net::local_image_cache::{ImageResponder, LocalImageCache};
use servo_net::resource_task::{ResourceTask, load_bytes_iter};
use servo_util::geometry::Au;
use servo_util::logical_geometry::LogicalPoint;
use servo_util::opts;
use servo_util::smallvec::{SmallVec, SmallVec1, VecLike};
use servo_util::task::spawn_named_with_send_on_failure;
use servo_util::task_state;
use servo_util::time::{TimeProfilerChan, profile};
use servo_util::time;
use servo_util::workqueue::WorkQueue;
use std::cell::Cell;
use std::comm::{channel, Sender, Receiver, Select};
use std::mem;
use std::ptr;
use style::{AuthorOrigin, Stylesheet, Stylist, TNode, iter_font_face_rules};
use style::{Device, Screen};
use sync::{Arc, Mutex, MutexGuard};
use url::Url;

/// Mutable data belonging to the LayoutTask.
///
/// This needs to be protected by a mutex so we can do fast RPCs.
pub struct LayoutTaskData {
    /// The local image cache.
    pub local_image_cache: Arc<Mutex<LocalImageCache<UntrustedNodeAddress>>>,

    /// The size of the viewport.
    pub screen_size: Size2D<Au>,

    /// The root stacking context.
    pub stacking_context: Option<Arc<StackingContext>>,

    pub stylist: Box<Stylist>,

    /// The workers that we use for parallel operation.
    pub parallel_traversal: Option<WorkQueue<*const SharedLayoutContext, UnsafeFlow>>,

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

    /// The channel on which messages can be sent to the constellation.
    pub constellation_chan: ConstellationChan,

    /// The channel on which messages can be sent to the script task.
    pub script_chan: ScriptControlChan,

    /// The channel on which messages can be sent to the painting task.
    pub render_chan: RenderChan,

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
    fn respond(&self) -> proc(ImageResponseMsg, UntrustedNodeAddress):Send {
        let id = self.id.clone();
        let script_chan = self.script_chan.clone();
        let f: proc(ImageResponseMsg, UntrustedNodeAddress):Send =
            proc(_, node_address) {
                let ScriptControlChan(chan) = script_chan;
                debug!("Dirtying {:x}", node_address as uint);
                let mut nodes = SmallVec1::new();
                nodes.vec_push(node_address);
                drop(chan.send_opt(SendEventMsg(id.clone(), ReflowEvent(nodes))))
            };
        f
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
                  render_chan: RenderChan,
                  resource_task: ResourceTask,
                  img_cache_task: ImageCacheTask,
                  font_cache_task: FontCacheTask,
                  time_profiler_chan: TimeProfilerChan,
                  shutdown_chan: Sender<()>) {
        let ConstellationChan(con_chan) = constellation_chan.clone();
        spawn_named_with_send_on_failure("LayoutTask", task_state::LAYOUT, proc() {
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
                        render_chan,
                        resource_task,
                        img_cache_task,
                        font_cache_task,
                        time_profiler_chan);
                layout.start();
            }
            shutdown_chan.send(());
        }, FailureMsg(failure_msg), con_chan, false);
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

impl<'a> Deref<LayoutTaskData> for RWGuard<'a> {
    fn deref(&self) -> &LayoutTaskData {
        match *self {
            Held(ref x) => x.deref(),
            Used(ref x) => x.deref(),
        }
    }
}

impl<'a> DerefMut<LayoutTaskData> for RWGuard<'a> {
    fn deref_mut(&mut self) -> &mut LayoutTaskData {
        match *self {
            Held(ref mut x) => x.deref_mut(),
            Used(ref mut x) => x.deref_mut(),
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
           render_chan: RenderChan,
           resource_task: ResourceTask,
           image_cache_task: ImageCacheTask,
           font_cache_task: FontCacheTask,
           time_profiler_chan: TimeProfilerChan)
           -> LayoutTask {
        let local_image_cache =
            Arc::new(Mutex::new(LocalImageCache::new(image_cache_task.clone())));
        let screen_size = Size2D(Au(0), Au(0));
        let device = Device::new(Screen, opts::get().initial_window_size.as_f32() * ScaleFactor(1.0));
        let parallel_traversal = if opts::get().layout_threads != 1 {
            Some(WorkQueue::new("LayoutWorker", task_state::LAYOUT,
                                opts::get().layout_threads, ptr::null()))
        } else {
            None
        };

        LayoutTask {
            id: id,
            port: port,
            pipeline_port: pipeline_port,
            chan: chan,
            constellation_chan: constellation_chan,
            script_chan: script_chan,
            render_chan: render_chan,
            time_profiler_chan: time_profiler_chan,
            resource_task: resource_task,
            image_cache_task: image_cache_task.clone(),
            font_cache_task: font_cache_task,
            first_reflow: Cell::new(true),
            rw_data: Arc::new(Mutex::new(
                LayoutTaskData {
                    local_image_cache: local_image_cache,
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
        let mut possibly_locked_rw_data = Some(self.rw_data.lock());
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
            constellation_chan: self.constellation_chan.clone(),
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
                Script
            } else if ret == port2.id() {
                Pipeline
            } else {
                panic!("invalid select result");
            }
        };

        match port_to_read {
            Pipeline => {
                match self.pipeline_port.recv() {
                    layout_traits::ExitNowMsg => {
                        self.handle_script_request(ExitNowMsg, possibly_locked_rw_data)
                    }
                }
            },
            Script => {
                let msg = self.port.recv();
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
            None    => Used(self.rw_data.lock()),
            Some(x) => Held(x),
        }
    }

    /// If no reflow has ever been triggered, this will keep the lock, locked
    /// (and saved in `possibly_locked_rw_data`). If it has been, the lock will
    /// be unlocked.
    fn return_rw_data<'a>(possibly_locked_rw_data: &mut Option<MutexGuard<'a, LayoutTaskData>>,
                          rw_data: RWGuard<'a>) {
        match rw_data {
            Used(x) => drop(x),
            Held(x) => *possibly_locked_rw_data = Some(x),
        }
    }

    /// Receives and dispatches messages from the script task.
    fn handle_script_request<'a>(&'a self,
                                 request: Msg,
                                 possibly_locked_rw_data: &mut Option<MutexGuard<'a,
                                                                                 LayoutTaskData>>)
                                 -> bool {
        match request {
            AddStylesheetMsg(sheet) => self.handle_add_stylesheet(sheet, possibly_locked_rw_data),
            LoadStylesheetMsg(url) => self.handle_load_stylesheet(url, possibly_locked_rw_data),
            GetRPCMsg(response_chan) => {
                response_chan.send(box LayoutRPCImpl(self.rw_data.clone()) as
                                   Box<LayoutRPC + Send>);
            },
            ReflowMsg(data) => {
                profile(time::LayoutPerformCategory,
                        Some((&data.url, data.iframe, self.first_reflow.get())),
                        self.time_profiler_chan.clone(),
                        || self.handle_reflow(&*data, possibly_locked_rw_data));
            },
            ReapLayoutDataMsg(dead_layout_data) => {
                unsafe {
                    LayoutTask::handle_reap_layout_data(dead_layout_data)
                }
            },
            PrepareToExitMsg(response_chan) => {
                debug!("layout: PrepareToExitMsg received");
                self.prepare_to_exit(response_chan, possibly_locked_rw_data);
                return false
            },
            ExitNowMsg => {
                debug!("layout: ExitNowMsg received");
                self.exit_now(possibly_locked_rw_data);
                return false
            }
        }

        true
    }

    /// Enters a quiescent state in which no new messages except for `ReapLayoutDataMsg` will be
    /// processed until an `ExitNowMsg` is received. A pong is immediately sent on the given
    /// response channel.
    fn prepare_to_exit<'a>(&'a self,
                           response_chan: Sender<()>,
                           possibly_locked_rw_data: &mut Option<MutexGuard<'a, LayoutTaskData>>) {
        response_chan.send(());
        loop {
            match self.port.recv() {
                ReapLayoutDataMsg(dead_layout_data) => {
                    unsafe {
                        LayoutTask::handle_reap_layout_data(dead_layout_data)
                    }
                }
                ExitNowMsg => {
                    debug!("layout task is exiting...");
                    self.exit_now(possibly_locked_rw_data);
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
    fn exit_now<'a>(&'a self, possibly_locked_rw_data: &mut Option<MutexGuard<'a, LayoutTaskData>>) {
        let (response_chan, response_port) = channel();

        {
            let mut rw_data = self.lock_rw_data(possibly_locked_rw_data);
            match rw_data.deref_mut().parallel_traversal {
                None => {}
                Some(ref mut traversal) => traversal.shutdown(),
            }
            LayoutTask::return_rw_data(possibly_locked_rw_data, rw_data);
        }

        self.render_chan.send(render_task::ExitMsg(Some(response_chan)));
        response_port.recv()
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
                                                AuthorOrigin);
        self.handle_add_stylesheet(sheet, possibly_locked_rw_data);
    }

    fn handle_add_stylesheet<'a>(&'a self,
                                 sheet: Stylesheet,
                                 possibly_locked_rw_data:
                                    &mut Option<MutexGuard<'a, LayoutTaskData>>) {
        // Find all font-face rules and notify the font cache of them.
        // GWTODO: Need to handle unloading web fonts (when we handle unloading stylesheets!)
        let mut rw_data = self.lock_rw_data(possibly_locked_rw_data);
        iter_font_face_rules(&sheet, &rw_data.stylist.device, |family, src| {
            self.font_cache_task.add_web_font(family.to_string(), (*src).clone());
        });
        rw_data.stylist.add_stylesheet(sheet);
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
            FlowConstructionResult(mut flow, abs_descendants) => {
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
                                                      &data.url,
                                                      data.iframe,
                                                      self.first_reflow.get(),
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
        let requested_node: OpaqueNode = OpaqueNodeMethods::from_script_node(requested_node);
        let mut iterator = UnioningFragmentBoundsIterator::new(requested_node);
        sequential::iterate_through_flow_tree_fragment_bounds(layout_root, &mut iterator);
        rw_data.content_box_response = iterator.rect;
    }

    fn process_content_boxes_request<'a>(&'a self,
                                         requested_node: TrustedNodeAddress,
                                         layout_root: &mut FlowRef,
                                         rw_data: &mut RWGuard<'a>) {
        let requested_node: OpaqueNode = OpaqueNodeMethods::from_script_node(requested_node);
        let mut iterator = CollectingFragmentBoundsIterator::new(requested_node);
        sequential::iterate_through_flow_tree_fragment_bounds(layout_root, &mut iterator);
        rw_data.content_boxes_response = iterator.rects;
    }

    fn build_display_list_for_reflow<'a>(&'a self,
                                         data: &Reflow,
                                         node: &mut LayoutNode,
                                         layout_root: &mut FlowRef,
                                         shared_layout_ctx: &mut SharedLayoutContext,
                                         rw_data: &mut RWGuard<'a>) {
        let writing_mode = flow::base(layout_root.deref()).writing_mode;
        profile(time::LayoutDispListBuildCategory,
                Some((&data.url, data.iframe, self.first_reflow.get())),
                     self.time_profiler_chan.clone(),
                     || {
            shared_layout_ctx.dirty =
                flow::base(layout_root.deref()).position.to_physical(writing_mode,
                                                                     rw_data.screen_size);
            flow::mut_base(layout_root.deref_mut()).stacking_relative_position =
                LogicalPoint::zero(writing_mode).to_physical(writing_mode,
                                                             rw_data.screen_size);

            flow::mut_base(layout_root.deref_mut()).clip_rect = data.page_clip_rect;

            let rw_data = rw_data.deref_mut();
            match rw_data.parallel_traversal {
                None => {
                    sequential::build_display_list_for_subtree(layout_root, shared_layout_ctx);
                }
                Some(ref mut traversal) => {
                    parallel::build_display_list_for_subtree(layout_root,
                                                             &data.url,
                                                             data.iframe,
                                                             self.first_reflow.get(),
                                                             self.time_profiler_chan.clone(),
                                                             shared_layout_ctx,
                                                             traversal);
                }
            }

            debug!("Done building display list.");

            // FIXME(pcwalton): This is really ugly and can't handle overflow: scroll. Refactor
            // it with extreme prejudice.
            let mut color = color::rgba(1.0, 1.0, 1.0, 1.0);
            for child in node.traverse_preorder() {
                if child.type_id() == Some(ElementNodeTypeId(HTMLHtmlElementTypeId)) ||
                        child.type_id() == Some(ElementNodeTypeId(HTMLBodyElementTypeId)) {
                    let element_bg_color = {
                        let thread_safe_child = ThreadSafeLayoutNode::new(&child);
                        thread_safe_child.style()
                                         .resolve_color(thread_safe_child.style()
                                                                         .get_background()
                                                                         .background_color)
                                         .to_gfx_color()
                    };
                    // FIXME: Add equality operators for azure color type.
                    if element_bg_color.r != 0.0 || element_bg_color.g != 0.0 ||
                       element_bg_color.b != 0.0 || element_bg_color.a != 0.0 {
                        color = element_bg_color;
                        break;
                    }
                }
            }

            let root_size = {
                let root_flow = flow::base(layout_root.deref());
                root_flow.position.size.to_physical(root_flow.writing_mode)
            };
            let mut display_list = box DisplayList::new();
            flow::mut_base(layout_root.deref_mut()).display_list_building_result
                                                   .add_to(&mut *display_list);
            let render_layer = Arc::new(RenderLayer::new(layout_root.layer_id(0),
                                                         color,
                                                         Scrollable));
            let origin = Rect(Point2D(Au(0), Au(0)), root_size);
            let stacking_context = Arc::new(StackingContext::new(display_list,
                                                                 origin,
                                                                 0,
                                                                 Some(render_layer)));

            rw_data.stacking_context = Some(stacking_context.clone());

            debug!("Layout done!");

            self.render_chan.send(RenderInitMsg(stacking_context));
        });
    }

    /// The high-level routine that performs layout tasks.
    fn handle_reflow<'a>(&'a self,
                         data: &Reflow,
                         possibly_locked_rw_data: &mut Option<MutexGuard<'a, LayoutTaskData>>) {
        // FIXME: Isolate this transmutation into a "bridge" module.
        // FIXME(rust#16366): The following line had to be moved because of a
        // rustc bug. It should be in the next unsafe block.
        let mut node: JS<Node> = unsafe {
            JS::from_trusted_node_address(data.document_root)
        };
        let node: &mut LayoutNode = unsafe {
            mem::transmute(&mut node)
        };

        debug!("layout: received layout request for: {:s}", data.url.serialize());
        debug!("layout: parsed Node tree");
        if log_enabled!(log::DEBUG) {
            node.dump();
        }

        let mut rw_data = self.lock_rw_data(possibly_locked_rw_data);

        {
            // Reset the image cache.
            let mut local_image_cache = rw_data.local_image_cache.lock();
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
        let mut shared_layout_ctx = self.build_shared_layout_context(rw_data.deref(),
                                                                     node,
                                                                     &data.url);

        // Handle conditions where the entire flow tree is invalid.
        let screen_size_changed = current_screen_size != old_screen_size;

        if screen_size_changed {
            let device = Device::new(Screen, data.window_size.initial_viewport);
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
                |mut flow| LayoutTask::reflow_all_nodes(flow.deref_mut()));
        }

        let mut layout_root = profile(time::LayoutStyleRecalcCategory,
                                      Some((&data.url,
                                      data.iframe,
                                      self.first_reflow.get())),
                                      self.time_profiler_chan.clone(),
                                      || {
            // Perform CSS selector matching and flow construction.
            let rw_data = rw_data.deref_mut();
            match rw_data.parallel_traversal {
                None => {
                    sequential::traverse_dom_preorder(*node, &shared_layout_ctx);
                }
                Some(ref mut traversal) => {
                    parallel::traverse_dom_preorder(*node, &shared_layout_ctx, traversal)
                }
            }

            self.get_layout_root((*node).clone())
        });

        profile(time::LayoutRestyleDamagePropagation,
                Some((&data.url, data.iframe, self.first_reflow.get())),
                self.time_profiler_chan.clone(),
                || {
            if opts::get().nonincremental_layout ||
                    layout_root.deref_mut().compute_layout_damage().contains(REFLOW_ENTIRE_DOCUMENT) {
                layout_root.deref_mut().reflow_entire_document()
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
        profile(time::LayoutMainCategory,
                Some((&data.url, data.iframe, self.first_reflow.get())),
                self.time_profiler_chan.clone(),
                || {
            let rw_data = rw_data.deref_mut();
            match rw_data.parallel_traversal {
                None => {
                    // Sequential mode.
                    self.solve_constraints(&mut layout_root, &shared_layout_ctx)
                }
                Some(_) => {
                    // Parallel mode.
                    self.solve_constraints_parallel(data,
                                                    rw_data,
                                                    &mut layout_root,
                                                    &mut shared_layout_ctx);
                }
            }
        });

        // Build the display list if necessary, and send it to the renderer.
        if data.goal == ReflowForDisplay {
            self.build_display_list_for_reflow(data,
                                               node,
                                               &mut layout_root,
                                               &mut shared_layout_ctx,
                                               &mut rw_data);
        }

        match data.query_type {
            ContentBoxQuery(node) =>
                self.process_content_box_request(node, &mut layout_root, &mut rw_data),
            ContentBoxesQuery(node) =>
                self.process_content_boxes_request(node, &mut layout_root, &mut rw_data),
            NoQuery => {},
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
        data.script_join_chan.send(());
        let ScriptControlChan(ref chan) = data.script_chan;
        chan.send(ReflowCompleteMsg(self.id, data.id));
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

    // When images can't be loaded in time to display they trigger
    // this callback in some task somewhere. This will send a message
    // to the script task, and ultimately cause the image to be
    // re-requested. We probably don't need to go all the way back to
    // the script task for this.
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
}

struct LayoutRPCImpl(Arc<Mutex<LayoutTaskData>>);

impl LayoutRPC for LayoutRPCImpl {
    // The neat thing here is that in order to answer the following two queries we only
    // need to compare nodes for equality. Thus we can safely work only with `OpaqueNode`.
    fn content_box(&self) -> ContentBoxResponse {
        let &LayoutRPCImpl(ref rw_data) = self;
        let rw_data = rw_data.lock();
        ContentBoxResponse(rw_data.content_box_response)
    }

    /// Requests the dimensions of all the content boxes, as in the `getClientRects()` call.
    fn content_boxes(&self) -> ContentBoxesResponse {
        let &LayoutRPCImpl(ref rw_data) = self;
        let rw_data = rw_data.lock();
        ContentBoxesResponse(rw_data.content_boxes_response.clone())
    }

    /// Requests the node containing the point of interest.
    fn hit_test(&self, _: TrustedNodeAddress, point: Point2D<f32>) -> Result<HitTestResponse, ()> {
        let point = Point2D(Au::from_frac_px(point.x as f64), Au::from_frac_px(point.y as f64));
        let resp = {
            let &LayoutRPCImpl(ref rw_data) = self;
            let rw_data = rw_data.lock();
            match rw_data.stacking_context {
                None => panic!("no root stacking context!"),
                Some(ref stacking_context) => {
                    let mut result = Vec::new();
                    stacking_context.hit_test(point, &mut result, true);
                    if !result.is_empty() {
                        Some(HitTestResponse(result[0]))
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
        let mut mouse_over_list: Vec<UntrustedNodeAddress> = vec!();
        let point = Point2D(Au::from_frac_px(point.x as f64), Au::from_frac_px(point.y as f64));
        {
            let &LayoutRPCImpl(ref rw_data) = self;
            let rw_data = rw_data.lock();
            match rw_data.stacking_context {
                None => panic!("no root stacking context!"),
                Some(ref stacking_context) => {
                    stacking_context.hit_test(point, &mut mouse_over_list, false);
                }
            }
        }

        if mouse_over_list.is_empty() {
            Err(())
        } else {
            Ok(MouseOverResponse(mouse_over_list))
        }
    }
}

struct UnioningFragmentBoundsIterator {
    node_address: OpaqueNode,
    rect: Rect<Au>,
}

impl UnioningFragmentBoundsIterator {
    fn new(node_address: OpaqueNode) -> UnioningFragmentBoundsIterator {
        UnioningFragmentBoundsIterator {
            node_address: node_address,
            rect: Rect::zero(),
        }
    }
}

impl FragmentBoundsIterator for UnioningFragmentBoundsIterator {
    fn process(&mut self, _: &Fragment, bounds: Rect<Au>) {
        if self.rect.is_empty() {
            self.rect = bounds;
        } else {
            self.rect = self.rect.union(&bounds);
        }
    }

    fn should_process(&mut self, fragment: &Fragment) -> bool {
        self.node_address == fragment.node
    }
}

struct CollectingFragmentBoundsIterator {
    node_address: OpaqueNode,
    rects: Vec<Rect<Au>>,
}

impl CollectingFragmentBoundsIterator {
    fn new(node_address: OpaqueNode) -> CollectingFragmentBoundsIterator {
        CollectingFragmentBoundsIterator {
            node_address: node_address,
            rects: Vec::new(),
        }
    }
}

impl FragmentBoundsIterator for CollectingFragmentBoundsIterator {
    fn process(&mut self, _: &Fragment, bounds: Rect<Au>) {
        self.rects.push(bounds);
    }

    fn should_process(&mut self, fragment: &Fragment) -> bool {
        self.node_address == fragment.node
    }
}
