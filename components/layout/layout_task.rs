/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The layout task. Performs layout on the DOM, builds display lists and sends them to be
//! painted.

#![allow(unsafe_code)]

use animation;
use construct::ConstructionResult;
use context::{SharedLayoutContext, heap_size_of_local_context};
use cssparser::ToCss;
use data::LayoutDataWrapper;
use display_list_builder::ToGfxColor;
use flow::{self, Flow, ImmutableFlowUtils, MutableFlowUtils, MutableOwnedFlowUtils};
use flow_ref::FlowRef;
use fragment::{Fragment, FragmentBorderBoxIterator, SpecificFragmentInfo};
use incremental::{LayoutDamageComputation, REFLOW, REFLOW_ENTIRE_DOCUMENT, REPAINT};
use layout_debug;
use opaque_node::OpaqueNodeMethods;
use parallel::{self, WorkQueueData};
use query::{LayoutRPCImpl, process_content_box_request, process_content_boxes_request, MarginPadding, Side};
use query::{MarginRetrievingFragmentBorderBoxIterator, PositionProperty, PositionRetrievingFragmentBorderBoxIterator};
use sequential;
use wrapper::LayoutNode;

use azure::azure::AzColor;
use canvas_traits::CanvasMsg;
use encoding::EncodingRef;
use encoding::all::UTF_8;
use fnv::FnvHasher;
use euclid::Matrix4;
use euclid::point::Point2D;
use euclid::rect::Rect;
use euclid::scale_factor::ScaleFactor;
use euclid::size::Size2D;
use gfx_traits::color;
use gfx::display_list::{ClippingRegion, DisplayList, OpaqueNode};
use gfx::display_list::StackingContext;
use gfx::font_cache_task::FontCacheTask;
use gfx::paint_task::{LayoutToPaintMsg, PaintLayer};
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use layout_traits::LayoutTaskFactory;
use log;
use msg::compositor_msg::{Epoch, ScrollPolicy, LayerId};
use msg::constellation_msg::Msg as ConstellationMsg;
use msg::constellation_msg::{ConstellationChan, Failure, PipelineExitType, PipelineId};
use profile_traits::mem::{self, Report, ReportKind, ReportsChan};
use profile_traits::time::{self, ProfilerMetadata, profile};
use profile_traits::time::{TimerMetadataFrameType, TimerMetadataReflowType};
use net_traits::{load_bytes_iter, PendingAsyncLoad};
use net_traits::image_cache_task::{ImageCacheTask, ImageCacheResult, ImageCacheChan};
use script::dom::bindings::js::LayoutJS;
use script::dom::node::{LayoutData, Node};
use script::layout_interface::Animation;
use script::layout_interface::{LayoutChan, LayoutRPC, OffsetParentResponse};
use script::layout_interface::{NewLayoutTaskInfo, Msg, Reflow, ReflowGoal, ReflowQueryType};
use script::layout_interface::{ScriptLayoutChan, ScriptReflow, TrustedNodeAddress};
use script_traits::{ConstellationControlMsg, LayoutControlMsg, OpaqueScriptLayoutChannel};
use script_traits::StylesheetLoadResponder;
use selectors::parser::PseudoElement;
use serde_json;
use std::borrow::ToOwned;
use std::cell::Cell;
use std::collections::HashMap;
use std::collections::hash_state::DefaultState;
use std::mem::transmute;
use std::ops::{Deref, DerefMut};
use std::sync::mpsc::{channel, Sender, Receiver, Select};
use std::sync::{Arc, Mutex, MutexGuard};
use string_cache::Atom;
use style::computed_values::{self, filter, mix_blend_mode};
use style::media_queries::{MediaType, MediaQueryList, Device};
use style::properties::style_structs;
use style::properties::longhands::{display, position};
use style::selector_matching::Stylist;
use style::stylesheets::{Origin, Stylesheet, CSSRuleIteratorExt};
use url::Url;
use util::geometry::{Au, MAX_RECT, ZERO_POINT};
use util::ipc::OptionalIpcSender;
use util::logical_geometry::LogicalPoint;
use util::mem::HeapSizeOf;
use util::opts;
use util::task::spawn_named_with_send_on_failure;
use util::task_state;
use util::workqueue::WorkQueue;
use wrapper::ThreadSafeLayoutNode;

/// The number of screens of data we're allowed to generate display lists for in each direction.
pub const DISPLAY_PORT_SIZE_FACTOR: i32 = 8;

/// The number of screens we have to traverse before we decide to generate new display lists.
const DISPLAY_PORT_THRESHOLD_SIZE_FACTOR: i32 = 4;

/// Mutable data belonging to the LayoutTask.
///
/// This needs to be protected by a mutex so we can do fast RPCs.
pub struct LayoutTaskData {
    /// The root of the flow tree.
    pub root_flow: Option<FlowRef>,

    /// The image cache.
    pub image_cache_task: ImageCacheTask,

    /// The channel on which messages can be sent to the constellation.
    pub constellation_chan: ConstellationChan,

    /// The size of the viewport.
    pub screen_size: Size2D<Au>,

    /// The root stacking context.
    pub stacking_context: Option<Arc<StackingContext>>,

    /// Performs CSS selector matching and style resolution.
    pub stylist: Box<Stylist>,

    /// The workers that we use for parallel operation.
    pub parallel_traversal: Option<WorkQueue<SharedLayoutContext, WorkQueueData>>,

    /// Starts at zero, and increased by one every time a layout completes.
    /// This can be used to easily check for invalid stale data.
    pub generation: u32,

    /// A queued response for the union of the content boxes of a node.
    pub content_box_response: Rect<Au>,

    /// A queued response for the content boxes of a node.
    pub content_boxes_response: Vec<Rect<Au>>,

    /// A queued response for the client {top, left, width, height} of a node in pixels.
    pub client_rect_response: Rect<i32>,

    /// A queued response for the resolved style property of an element.
    pub resolved_style_response: Option<String>,

    /// A queued response for the offset parent/rect of a node.
    pub offset_parent_response: OffsetParentResponse,

    /// The list of currently-running animations.
    pub running_animations: Arc<HashMap<OpaqueNode,Vec<Animation>>>,

    /// Receives newly-discovered animations.
    pub new_animations_receiver: Receiver<Animation>,

    /// A channel on which new animations that have been triggered by style recalculation can be
    /// sent.
    pub new_animations_sender: Sender<Animation>,

    /// A counter for epoch messages
    epoch: Epoch,

    /// The position and size of the visible rect for each layer. We do not build display lists
    /// for any areas more than `DISPLAY_PORT_SIZE_FACTOR` screens away from this area.
    pub visible_rects: Arc<HashMap<LayerId, Rect<Au>, DefaultState<FnvHasher>>>,
}

/// Information needed by the layout task.
pub struct LayoutTask {
    /// The ID of the pipeline that we belong to.
    pub id: PipelineId,

    /// The URL of the pipeline that we belong to.
    pub url: Url,

    /// Is the current reflow of an iframe, as opposed to a root window?
    pub is_iframe: bool,

    /// The port on which we receive messages from the script task.
    pub port: Receiver<Msg>,

    /// The port on which we receive messages from the constellation.
    pub pipeline_port: Receiver<LayoutControlMsg>,

    /// The port on which we receive messages from the image cache
    image_cache_receiver: Receiver<ImageCacheResult>,

    /// The channel on which the image cache can send messages to ourself.
    image_cache_sender: ImageCacheChan,

    /// The channel on which we or others can send messages to ourselves.
    pub chan: LayoutChan,

    /// The channel on which messages can be sent to the constellation.
    pub constellation_chan: ConstellationChan,

    /// The channel on which messages can be sent to the script task.
    pub script_chan: Sender<ConstellationControlMsg>,

    /// The channel on which messages can be sent to the painting task.
    pub paint_chan: OptionalIpcSender<LayoutToPaintMsg>,

    /// The channel on which messages can be sent to the time profiler.
    pub time_profiler_chan: time::ProfilerChan,

    /// The channel on which messages can be sent to the memory profiler.
    pub mem_profiler_chan: mem::ProfilerChan,

    /// The channel on which messages can be sent to the image cache.
    pub image_cache_task: ImageCacheTask,

    /// Public interface to the font cache task.
    pub font_cache_task: FontCacheTask,

    /// Is this the first reflow in this LayoutTask?
    pub first_reflow: Cell<bool>,

    /// To receive a canvas renderer associated to a layer, this message is propagated
    /// to the paint chan
    pub canvas_layers_receiver: Receiver<(LayerId, IpcSender<CanvasMsg>)>,
    pub canvas_layers_sender: Sender<(LayerId, IpcSender<CanvasMsg>)>,

    /// A mutex to allow for fast, read-only RPC of layout's internal data
    /// structures, while still letting the LayoutTask modify them.
    ///
    /// All the other elements of this struct are read-only.
    pub rw_data: Arc<Mutex<LayoutTaskData>>,
}

impl LayoutTaskFactory for LayoutTask {
    /// Spawns a new layout task.
    fn create(_phantom: Option<&mut LayoutTask>,
              id: PipelineId,
              url: Url,
              is_iframe: bool,
              chan: OpaqueScriptLayoutChannel,
              pipeline_port: IpcReceiver<LayoutControlMsg>,
              constellation_chan: ConstellationChan,
              failure_msg: Failure,
              script_chan: Sender<ConstellationControlMsg>,
              paint_chan: OptionalIpcSender<LayoutToPaintMsg>,
              image_cache_task: ImageCacheTask,
              font_cache_task: FontCacheTask,
              time_profiler_chan: time::ProfilerChan,
              mem_profiler_chan: mem::ProfilerChan,
              shutdown_chan: Sender<()>) {
        let ConstellationChan(con_chan) = constellation_chan.clone();
        spawn_named_with_send_on_failure(format!("LayoutTask {:?}", id), task_state::LAYOUT, move || {
            { // Ensures layout task is destroyed before we send shutdown message
                let sender = chan.sender();
                let layout_chan = LayoutChan(sender);
                let layout = LayoutTask::new(id,
                                             url,
                                             is_iframe,
                                             chan.receiver(),
                                             layout_chan.clone(),
                                             pipeline_port,
                                             constellation_chan,
                                             script_chan,
                                             paint_chan,
                                             image_cache_task,
                                             font_cache_task,
                                             time_profiler_chan,
                                             mem_profiler_chan.clone());

                let reporter_name = format!("layout-reporter-{}", id.0);
                mem_profiler_chan.run_with_memory_reporting(|| {
                    layout.start();
                }, reporter_name, layout_chan.0, Msg::CollectReports);
            }
            shutdown_chan.send(()).unwrap();
        }, ConstellationMsg::Failure(failure_msg), con_chan);
    }
}

/// The `LayoutTask` `rw_data` lock must remain locked until the first reflow,
/// as RPC calls don't make sense until then. Use this in combination with
/// `LayoutTask::lock_rw_data` and `LayoutTask::return_rw_data`.
pub enum RWGuard<'a> {
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

fn add_font_face_rules(stylesheet: &Stylesheet, device: &Device, font_cache_task: &FontCacheTask) {
    for font_face in stylesheet.effective_rules(&device).font_face() {
        for source in &font_face.sources {
            font_cache_task.add_web_font(font_face.family.clone(), source.clone());
        }
    }
}

impl LayoutTask {
    /// Creates a new `LayoutTask` structure.
    fn new(id: PipelineId,
           url: Url,
           is_iframe: bool,
           port: Receiver<Msg>,
           chan: LayoutChan,
           pipeline_port: IpcReceiver<LayoutControlMsg>,
           constellation_chan: ConstellationChan,
           script_chan: Sender<ConstellationControlMsg>,
           paint_chan: OptionalIpcSender<LayoutToPaintMsg>,
           image_cache_task: ImageCacheTask,
           font_cache_task: FontCacheTask,
           time_profiler_chan: time::ProfilerChan,
           mem_profiler_chan: mem::ProfilerChan)
           -> LayoutTask {
        let screen_size = Size2D::new(Au(0), Au(0));
        let device = Device::new(
            MediaType::Screen,
            opts::get().initial_window_size.as_f32() * ScaleFactor::new(1.0));
        let parallel_traversal = if opts::get().layout_threads != 1 {
            Some(WorkQueue::new("LayoutWorker", task_state::LAYOUT,
                                opts::get().layout_threads))
        } else {
            None
        };

        // Create the channel on which new animations can be sent.
        let (new_animations_sender, new_animations_receiver) = channel();
        let (canvas_layers_sender, canvas_layers_receiver) = channel();

        // Proxy IPC messages from the pipeline to the layout thread.
        let pipeline_receiver = ROUTER.route_ipc_receiver_to_new_mpsc_receiver(pipeline_port);

        // Ask the router to proxy IPC messages from the image cache task to the layout thread.
        let (ipc_image_cache_sender, ipc_image_cache_receiver) = ipc::channel().unwrap();
        let image_cache_receiver =
            ROUTER.route_ipc_receiver_to_new_mpsc_receiver(ipc_image_cache_receiver);

        let stylist = box Stylist::new(device);
        for user_or_user_agent_stylesheet in stylist.stylesheets() {
            add_font_face_rules(user_or_user_agent_stylesheet, &stylist.device, &font_cache_task);
        }

        LayoutTask {
            id: id,
            url: url,
            is_iframe: is_iframe,
            port: port,
            pipeline_port: pipeline_receiver,
            chan: chan,
            script_chan: script_chan,
            constellation_chan: constellation_chan.clone(),
            paint_chan: paint_chan,
            time_profiler_chan: time_profiler_chan,
            mem_profiler_chan: mem_profiler_chan,
            image_cache_task: image_cache_task.clone(),
            font_cache_task: font_cache_task,
            first_reflow: Cell::new(true),
            image_cache_receiver: image_cache_receiver,
            image_cache_sender: ImageCacheChan(ipc_image_cache_sender),
            canvas_layers_receiver: canvas_layers_receiver,
            canvas_layers_sender: canvas_layers_sender,
            rw_data: Arc::new(Mutex::new(
                LayoutTaskData {
                    root_flow: None,
                    image_cache_task: image_cache_task,
                    constellation_chan: constellation_chan,
                    screen_size: screen_size,
                    stacking_context: None,
                    stylist: stylist,
                    parallel_traversal: parallel_traversal,
                    generation: 0,
                    content_box_response: Rect::zero(),
                    content_boxes_response: Vec::new(),
                    client_rect_response: Rect::zero(),
                    resolved_style_response: None,
                    running_animations: Arc::new(HashMap::new()),
                    offset_parent_response: OffsetParentResponse::empty(),
                    visible_rects: Arc::new(HashMap::with_hash_state(Default::default())),
                    new_animations_receiver: new_animations_receiver,
                    new_animations_sender: new_animations_sender,
                    epoch: Epoch(0),
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
                                   screen_size_changed: bool,
                                   reflow_root: Option<&LayoutNode>,
                                   url: &Url,
                                   goal: ReflowGoal)
                                   -> SharedLayoutContext {
        SharedLayoutContext {
            image_cache_task: rw_data.image_cache_task.clone(),
            image_cache_sender: self.image_cache_sender.clone(),
            screen_size: rw_data.screen_size.clone(),
            screen_size_changed: screen_size_changed,
            constellation_chan: rw_data.constellation_chan.clone(),
            layout_chan: self.chan.clone(),
            font_cache_task: self.font_cache_task.clone(),
            canvas_layers_sender: self.canvas_layers_sender.clone(),
            stylist: &*rw_data.stylist,
            url: (*url).clone(),
            reflow_root: reflow_root.map(|node| node.opaque()),
            visible_rects: rw_data.visible_rects.clone(),
            generation: rw_data.generation,
            new_animations_sender: rw_data.new_animations_sender.clone(),
            goal: goal,
            running_animations: rw_data.running_animations.clone(),
        }
    }

    /// Receives and dispatches messages from the script and constellation tasks
    fn handle_request<'a>(&'a self,
                          possibly_locked_rw_data: &mut Option<MutexGuard<'a, LayoutTaskData>>)
                          -> bool {
        enum PortToRead {
            Pipeline,
            Script,
            ImageCache,
        }

        let port_to_read = {
            let sel = Select::new();
            let mut port1 = sel.handle(&self.port);
            let mut port2 = sel.handle(&self.pipeline_port);
            let mut port3 = sel.handle(&self.image_cache_receiver);
            unsafe {
                port1.add();
                port2.add();
                port3.add();
            }
            let ret = sel.wait();
            if ret == port1.id() {
                PortToRead::Script
            } else if ret == port2.id() {
                PortToRead::Pipeline
            } else if ret == port3.id() {
                PortToRead::ImageCache
            } else {
                panic!("invalid select result");
            }
        };

        match port_to_read {
            PortToRead::Pipeline => {
                match self.pipeline_port.recv().unwrap() {
                    LayoutControlMsg::SetVisibleRects(new_visible_rects) => {
                        self.handle_request_helper(Msg::SetVisibleRects(new_visible_rects),
                                                   possibly_locked_rw_data)
                    }
                    LayoutControlMsg::TickAnimations => {
                        self.handle_request_helper(Msg::TickAnimations, possibly_locked_rw_data)
                    }
                    LayoutControlMsg::GetCurrentEpoch(sender) => {
                        self.handle_request_helper(Msg::GetCurrentEpoch(sender),
                                                   possibly_locked_rw_data)
                    }
                    LayoutControlMsg::ExitNow(exit_type) => {
                        self.handle_request_helper(Msg::ExitNow(exit_type),
                                                   possibly_locked_rw_data)
                    }
                }
            }
            PortToRead::Script => {
                let msg = self.port.recv().unwrap();
                self.handle_request_helper(msg, possibly_locked_rw_data)
            }
            PortToRead::ImageCache => {
                let _ = self.image_cache_receiver.recv().unwrap();
                self.repaint(possibly_locked_rw_data)
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

    /// Repaint the scene, without performing style matching. This is typically
    /// used when an image arrives asynchronously and triggers a relayout and
    /// repaint.
    /// TODO: In the future we could detect if the image size hasn't changed
    /// since last time and avoid performing a complete layout pass.
    fn repaint<'a>(&'a self,
                   possibly_locked_rw_data: &mut Option<MutexGuard<'a, LayoutTaskData>>) -> bool {
        let mut rw_data = self.lock_rw_data(possibly_locked_rw_data);

        let reflow_info = Reflow {
            goal: ReflowGoal::ForDisplay,
            page_clip_rect: MAX_RECT,
        };

        let mut layout_context = self.build_shared_layout_context(&*rw_data,
                                                                  false,
                                                                  None,
                                                                  &self.url,
                                                                  reflow_info.goal);

        self.perform_post_style_recalc_layout_passes(&reflow_info,
                                                     &mut *rw_data,
                                                     &mut layout_context);


        true
    }

    /// Receives and dispatches messages from other tasks.
    fn handle_request_helper<'a>(&'a self,
                                 request: Msg,
                                 possibly_locked_rw_data: &mut Option<MutexGuard<'a,
                                                                                 LayoutTaskData>>)
                                 -> bool {
        match request {
            Msg::AddStylesheet(sheet, mq) => {
                self.handle_add_stylesheet(sheet, mq, possibly_locked_rw_data)
            }
            Msg::LoadStylesheet(url, mq, pending, link_element) => {
                self.handle_load_stylesheet(url, mq, pending, link_element, possibly_locked_rw_data)
            }
            Msg::SetQuirksMode => self.handle_set_quirks_mode(possibly_locked_rw_data),
            Msg::GetRPC(response_chan) => {
                response_chan.send(box LayoutRPCImpl(self.rw_data.clone()) as
                                   Box<LayoutRPC + Send>).unwrap();
            },
            Msg::Reflow(data) => {
                profile(time::ProfilerCategory::LayoutPerform,
                        self.profiler_metadata(),
                        self.time_profiler_chan.clone(),
                        || self.handle_reflow(&*data, possibly_locked_rw_data));
            },
            Msg::TickAnimations => self.tick_all_animations(possibly_locked_rw_data),
            Msg::SetVisibleRects(new_visible_rects) => {
                self.set_visible_rects(new_visible_rects, possibly_locked_rw_data);
            }
            Msg::ReapLayoutData(dead_layout_data) => {
                unsafe {
                    self.handle_reap_layout_data(dead_layout_data)
                }
            },
            Msg::CollectReports(reports_chan) => {
                self.collect_reports(reports_chan, possibly_locked_rw_data);
            },
            Msg::GetCurrentEpoch(sender) => {
                let rw_data = self.lock_rw_data(possibly_locked_rw_data);
                sender.send(rw_data.epoch).unwrap();
            },
            Msg::CreateLayoutTask(info) => {
                self.create_layout_task(info)
            }
            Msg::PrepareToExit(response_chan) => {
                self.prepare_to_exit(response_chan, possibly_locked_rw_data);
                return false
            },
            Msg::ExitNow(exit_type) => {
                debug!("layout: ExitNow received");
                self.exit_now(possibly_locked_rw_data, exit_type);
                return false
            }
        }

        true
    }

    fn collect_reports<'a>(&'a self,
                           reports_chan: ReportsChan,
                           possibly_locked_rw_data: &mut Option<MutexGuard<'a, LayoutTaskData>>) {
        let mut reports = vec![];

        // FIXME(njn): Just measuring the display tree for now.
        let rw_data = self.lock_rw_data(possibly_locked_rw_data);
        let stacking_context = rw_data.stacking_context.as_ref();
        reports.push(Report {
            path: path![format!("url({})", self.url), "layout-task", "display-list"],
            kind: ReportKind::ExplicitJemallocHeapSize,
            size: stacking_context.map_or(0, |sc| sc.heap_size_of_children()),
        });

        // The LayoutTask has a context in TLS...
        reports.push(Report {
            path: path![format!("url({})", self.url), "layout-task", "local-context"],
            kind: ReportKind::ExplicitJemallocHeapSize,
            size: heap_size_of_local_context(),
        });

        // ... as do each of the LayoutWorkers, if present.
        if let Some(ref traversal) = rw_data.parallel_traversal {
            let sizes = traversal.heap_size_of_tls(heap_size_of_local_context);
            for (i, size) in sizes.iter().enumerate() {
                reports.push(Report {
                    path: path![format!("url({})", self.url),
                                format!("layout-worker-{}-local-context", i)],
                    kind: ReportKind::ExplicitJemallocHeapSize,
                    size: *size,
                });
            }
        }

        reports_chan.send(reports);
    }

    fn create_layout_task(&self, info: NewLayoutTaskInfo) {
        LayoutTaskFactory::create(None::<&mut LayoutTask>,
                                  info.id,
                                  info.url.clone(),
                                  info.is_parent,
                                  info.layout_pair,
                                  info.pipeline_port,
                                  info.constellation_chan,
                                  info.failure,
                                  info.script_chan.clone(),
                                  *info.paint_chan
                                       .downcast::<OptionalIpcSender<LayoutToPaintMsg>>()
                                       .unwrap(),
                                  self.image_cache_task.clone(),
                                  self.font_cache_task.clone(),
                                  self.time_profiler_chan.clone(),
                                  self.mem_profiler_chan.clone(),
                                  info.layout_shutdown_chan);
    }

    /// Enters a quiescent state in which no new messages except for
    /// `layout_interface::Msg::ReapLayoutData` will be processed until an `ExitNow` is
    /// received. A pong is immediately sent on the given response channel.
    fn prepare_to_exit<'a>(&'a self,
                           response_chan: Sender<()>,
                           possibly_locked_rw_data: &mut Option<MutexGuard<'a, LayoutTaskData>>) {
        response_chan.send(()).unwrap();
        loop {
            match self.port.recv().unwrap() {
                Msg::ReapLayoutData(dead_layout_data) => {
                    unsafe {
                        self.handle_reap_layout_data(dead_layout_data)
                    }
                }
                Msg::ExitNow(exit_type) => {
                    debug!("layout task is exiting...");
                    self.exit_now(possibly_locked_rw_data, exit_type);
                    break
                }
                Msg::CollectReports(_) => {
                    // Just ignore these messages at this point.
                }
                _ => {
                    panic!("layout: unexpected message received after `PrepareToExitMsg`")
                }
            }
        }
    }

    /// Shuts down the layout task now. If there are any DOM nodes left, layout will now (safely)
    /// crash.
    fn exit_now<'a>(&'a self,
                    possibly_locked_rw_data: &mut Option<MutexGuard<'a, LayoutTaskData>>,
                    exit_type: PipelineExitType) {
        let (response_chan, response_port) = ipc::channel().unwrap();

        {
            let mut rw_data = self.lock_rw_data(possibly_locked_rw_data);
            if let Some(ref mut traversal) = (&mut *rw_data).parallel_traversal {
                traversal.shutdown()
            }
            LayoutTask::return_rw_data(possibly_locked_rw_data, rw_data);
        }

        self.paint_chan.send(LayoutToPaintMsg::Exit(Some(response_chan), exit_type)).unwrap();
        response_port.recv().unwrap()
    }

    fn handle_load_stylesheet<'a>(&'a self,
                                  url: Url,
                                  mq: MediaQueryList,
                                  pending: PendingAsyncLoad,
                                  responder: Box<StylesheetLoadResponder+Send>,
                                  possibly_locked_rw_data:
                                    &mut Option<MutexGuard<'a, LayoutTaskData>>) {
        // TODO: Get the actual value. http://dev.w3.org/csswg/css-syntax/#environment-encoding
        let environment_encoding = UTF_8 as EncodingRef;

        // TODO we don't really even need to load this if mq does not match
        let (metadata, iter) = load_bytes_iter(pending);
        let protocol_encoding_label = metadata.charset.as_ref().map(|s| &**s);
        let final_url = metadata.final_url;

        let sheet = Stylesheet::from_bytes_iter(iter,
                                                final_url,
                                                protocol_encoding_label,
                                                Some(environment_encoding),
                                                Origin::Author);

        //TODO: mark critical subresources as blocking load as well (#5974)
        self.script_chan.send(ConstellationControlMsg::StylesheetLoadComplete(self.id, url, responder)).unwrap();

        self.handle_add_stylesheet(sheet, mq, possibly_locked_rw_data);
    }

    fn handle_add_stylesheet<'a>(&'a self,
                                 sheet: Stylesheet,
                                 mq: MediaQueryList,
                                 possibly_locked_rw_data:
                                    &mut Option<MutexGuard<'a, LayoutTaskData>>) {
        // Find all font-face rules and notify the font cache of them.
        // GWTODO: Need to handle unloading web fonts (when we handle unloading stylesheets!)

        let mut rw_data = self.lock_rw_data(possibly_locked_rw_data);

        if mq.evaluate(&rw_data.stylist.device) {
            add_font_face_rules(&sheet, &rw_data.stylist.device, &self.font_cache_task);
            rw_data.stylist.add_stylesheet(sheet);
        }

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
                                  traversal: &mut WorkQueue<SharedLayoutContext, WorkQueueData>,
                                  layout_root: &mut FlowRef,
                                  shared_layout_context: &SharedLayoutContext) {
        let _scope = layout_debug_scope!("solve_constraints_parallel");

        // NOTE: this currently computes borders, so any pruning should separate that
        // operation out.
        parallel::traverse_flow_tree_preorder(layout_root,
                                              self.profiler_metadata(),
                                              self.time_profiler_chan.clone(),
                                              shared_layout_context,
                                              traversal);
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

    fn process_node_geometry_request<'a>(&'a self,
                                      requested_node: TrustedNodeAddress,
                                      layout_root: &mut FlowRef,
                                      rw_data: &mut RWGuard<'a>) {
        let requested_node: OpaqueNode = OpaqueNodeMethods::from_script_node(requested_node);
        let mut iterator = FragmentLocatingFragmentIterator::new(requested_node);
        sequential::iterate_through_flow_tree_fragment_border_boxes(layout_root, &mut iterator);
        rw_data.client_rect_response = iterator.client_rect;
    }

    // Compute the resolved value of property for a given (pseudo)element.
    // Stores the result in rw_data.resolved_style_response.
    // https://drafts.csswg.org/cssom/#resolved-value
    fn process_resolved_style_request<'a>(&'a self,
                                          requested_node: TrustedNodeAddress,
                                          pseudo: &Option<PseudoElement>,
                                          property: &Atom,
                                          layout_root: &mut FlowRef,
                                          rw_data: &mut RWGuard<'a>) {
        // FIXME: Isolate this transmutation into a "bridge" module.
        // FIXME(rust#16366): The following line had to be moved because of a
        // rustc bug. It should be in the next unsafe block.
        let node: LayoutJS<Node> = unsafe {
            LayoutJS::from_trusted_node_address(requested_node)
        };
        let node: &LayoutNode = unsafe {
            transmute(&node)
        };

        let layout_node = ThreadSafeLayoutNode::new(node);
        let layout_node = match pseudo {
            &Some(PseudoElement::Before) => layout_node.get_before_pseudo(),
            &Some(PseudoElement::After) => layout_node.get_after_pseudo(),
            _ => Some(layout_node)
        };

        let layout_node = match layout_node {
            None => {
                // The pseudo doesn't exist, return nothing.  Chrome seems to query
                // the element itself in this case, Firefox uses the resolved value.
                // https://www.w3.org/Bugs/Public/show_bug.cgi?id=29006
                rw_data.resolved_style_response = None;
                return;
            }
            Some(layout_node) => layout_node
        };

        let style = &*layout_node.style();

        let positioned = match style.get_box().position {
            position::computed_value::T::relative |
            /*position::computed_value::T::sticky |*/
            position::computed_value::T::fixed |
            position::computed_value::T::absolute => true,
            _ => false
        };

        //TODO: determine whether requested property applies to the element.
        //      eg. width does not apply to non-replaced inline elements.
        // Existing browsers disagree about when left/top/right/bottom apply
        // (Chrome seems to think they never apply and always returns resolved values).
        // There are probably other quirks.
        let applies = true;

        // TODO: we will return neither the computed nor used value for margin and padding.
        // Firefox returns blank strings for the computed value of shorthands,
        // so this should be web-compatible.
        match property.clone() {
            atom!("margin-bottom") | atom!("margin-top") |
            atom!("margin-left") | atom!("margin-right") |
            atom!("padding-bottom") | atom!("padding-top") |
            atom!("padding-left") | atom!("padding-right")
            if applies && style.get_box().display != display::computed_value::T::none => {
                let (margin_padding, side) = match *property {
                    atom!("margin-bottom") => (MarginPadding::Margin, Side::Bottom),
                    atom!("margin-top") => (MarginPadding::Margin, Side::Top),
                    atom!("margin-left") => (MarginPadding::Margin, Side::Left),
                    atom!("margin-right") => (MarginPadding::Margin, Side::Right),
                    atom!("padding-bottom") => (MarginPadding::Padding, Side::Bottom),
                    atom!("padding-top") => (MarginPadding::Padding, Side::Top),
                    atom!("padding-left") => (MarginPadding::Padding, Side::Left),
                    atom!("padding-right") => (MarginPadding::Padding, Side::Right),
                    _ => unreachable!()
                };
                let requested_node: OpaqueNode = OpaqueNodeMethods::from_script_node(requested_node);
                let mut iterator =
                    MarginRetrievingFragmentBorderBoxIterator::new(requested_node,
                                                                   side,
                                                                   margin_padding,
                                                                   style.writing_mode);
                sequential::iterate_through_flow_tree_fragment_border_boxes(layout_root, &mut iterator);
                rw_data.resolved_style_response = iterator.result.map(|r| r.to_css_string());
            },

            atom!("bottom") | atom!("top") | atom!("right") |
            atom!("left") | atom!("width") | atom!("height")
            if applies && positioned && style.get_box().display != display::computed_value::T::none => {
                let layout_data = layout_node.borrow_layout_data();
                let position = layout_data.as_ref().map(|layout_data| {
                    match layout_data.data.flow_construction_result {
                        ConstructionResult::Flow(ref flow_ref, _) =>
                            flow::base(flow_ref.deref()).stacking_relative_position,
                        // TODO search parents until we find node with a flow ref.
                        _ => ZERO_POINT
                    }
                }).unwrap_or(ZERO_POINT);
                let property = match *property {
                    atom!("bottom") => PositionProperty::Bottom,
                    atom!("top") => PositionProperty::Top,
                    atom!("left") => PositionProperty::Left,
                    atom!("right") => PositionProperty::Right,
                    atom!("width") => PositionProperty::Width,
                    atom!("height") => PositionProperty::Height,
                    _ => unreachable!()
                };
                let requested_node: OpaqueNode = OpaqueNodeMethods::from_script_node(requested_node);
                let mut iterator =
                    PositionRetrievingFragmentBorderBoxIterator::new(requested_node,
                                                                     property,
                                                                     position);
                sequential::iterate_through_flow_tree_fragment_border_boxes(layout_root, &mut iterator);
                rw_data.resolved_style_response = iterator.result.map(|r| r.to_css_string());
            },
            // FIXME: implement used value computation for line-height
            property => {
                rw_data.resolved_style_response = style.computed_value_to_string(property.as_slice());
            }
        };
    }

    fn process_offset_parent_query<'a>(&'a self,
                                       requested_node: TrustedNodeAddress,
                                       layout_root: &mut FlowRef,
                                       rw_data: &mut RWGuard<'a>) {
        let requested_node: OpaqueNode = OpaqueNodeMethods::from_script_node(requested_node);
        let mut iterator = ParentOffsetBorderBoxIterator::new(requested_node);
        sequential::iterate_through_flow_tree_fragment_border_boxes(layout_root, &mut iterator);
        let parent_info_index = iterator.parent_nodes.iter().rposition(|info| info.is_some());
        match parent_info_index {
            Some(parent_info_index) => {
                let parent = iterator.parent_nodes[parent_info_index].as_ref().unwrap();
                let origin = iterator.node_border_box.origin - parent.border_box.origin;
                let size = iterator.node_border_box.size;
                rw_data.offset_parent_response = OffsetParentResponse {
                    node_address: Some(parent.node_address.to_untrusted_node_address()),
                    rect: Rect::new(origin, size),
                };
            }
            None => {
                rw_data.offset_parent_response = OffsetParentResponse::empty();
            }
        }
    }

    fn compute_abs_pos_and_build_display_list<'a>(&'a self,
                                                  data: &Reflow,
                                                  layout_root: &mut FlowRef,
                                                  shared_layout_context: &mut SharedLayoutContext,
                                                  rw_data: &mut LayoutTaskData) {
        let writing_mode = flow::base(&**layout_root).writing_mode;
        profile(time::ProfilerCategory::LayoutDispListBuild,
                self.profiler_metadata(),
                self.time_profiler_chan.clone(),
                || {
            flow::mut_base(&mut **layout_root).stacking_relative_position =
                LogicalPoint::zero(writing_mode).to_physical(writing_mode,
                                                             rw_data.screen_size);

            flow::mut_base(&mut **layout_root).clip =
                ClippingRegion::from_rect(&data.page_clip_rect);

            match (&mut rw_data.parallel_traversal, opts::get().parallel_display_list_building) {
                (&mut Some(ref mut traversal), true) => {
                    parallel::build_display_list_for_subtree(layout_root,
                                                             self.profiler_metadata(),
                                                             self.time_profiler_chan.clone(),
                                                             shared_layout_context,
                                                             traversal);
                }
                _ => {
                    sequential::build_display_list_for_subtree(layout_root,
                                                               shared_layout_context);
                }
            }

            if data.goal == ReflowGoal::ForDisplay {
                debug!("Done building display list.");

                let root_background_color = get_root_flow_background_color(&mut **layout_root);
                let root_size = {
                    let root_flow = flow::base(&**layout_root);
                    root_flow.position.size.to_physical(root_flow.writing_mode)
                };
                let mut display_list = box DisplayList::new();
                flow::mut_base(&mut **layout_root).display_list_building_result
                                                  .add_to(&mut *display_list);
                let paint_layer = PaintLayer::new(layout_root.layer_id(0),
                                                  root_background_color,
                                                  ScrollPolicy::Scrollable);
                let origin = Rect::new(Point2D::new(Au(0), Au(0)), root_size);

                let stacking_context = Arc::new(StackingContext::new(display_list,
                                                                     &origin,
                                                                     &origin,
                                                                     0,
                                                                     filter::T::new(Vec::new()),
                                                                     mix_blend_mode::T::normal,
                                                                     Some(paint_layer),
                                                                     Matrix4::identity(),
                                                                     Matrix4::identity(),
                                                                     true,
                                                                     false));

                if opts::get().dump_display_list {
                    println!("#### start printing display list.");
                    stacking_context.print("#".to_owned());
                }
                if opts::get().dump_display_list_json {
                    println!("{}", serde_json::to_string_pretty(&stacking_context).unwrap());
                }

                rw_data.stacking_context = Some(stacking_context.clone());

                debug!("Layout done!");

                rw_data.epoch.next();
                self.paint_chan
                    .send(LayoutToPaintMsg::PaintInit(rw_data.epoch, stacking_context))
                    .unwrap();
            }
        });
    }

    /// The high-level routine that performs layout tasks.
    fn handle_reflow<'a>(&'a self,
                         data: &ScriptReflow,
                         possibly_locked_rw_data: &mut Option<MutexGuard<'a, LayoutTaskData>>) {
        // FIXME: Isolate this transmutation into a "bridge" module.
        // FIXME(rust#16366): The following line had to be moved because of a
        // rustc bug. It should be in the next unsafe block.
        let mut node: LayoutJS<Node> = unsafe {
            LayoutJS::from_trusted_node_address(data.document_root)
        };
        let node: &mut LayoutNode = unsafe {
            transmute(&mut node)
        };

        debug!("layout: received layout request for: {}", self.url.serialize());
        if log_enabled!(log::LogLevel::Debug) {
            node.dump();
        }

        let mut rw_data = self.lock_rw_data(possibly_locked_rw_data);

        let initial_viewport = data.window_size.initial_viewport;
        let old_screen_size = rw_data.screen_size;
        let current_screen_size = Size2D::new(Au::from_f32_px(initial_viewport.width.get()),
                                              Au::from_f32_px(initial_viewport.height.get()));
        rw_data.screen_size = current_screen_size;

        // Handle conditions where the entire flow tree is invalid.
        let screen_size_changed = current_screen_size != old_screen_size;
        if screen_size_changed {
            // Calculate the actual viewport as per DEVICE-ADAPT § 6
            let device = Device::new(MediaType::Screen, initial_viewport);
            rw_data.stylist.set_device(device);

            if let Some(constraints) = rw_data.stylist.constrain_viewport() {
                debug!("Viewport constraints: {:?}", constraints);

                // other rules are evaluated against the actual viewport
                rw_data.screen_size = Size2D::new(Au::from_f32_px(constraints.size.width.get()),
                                                  Au::from_f32_px(constraints.size.height.get()));
                let device = Device::new(MediaType::Screen, constraints.size);
                rw_data.stylist.set_device(device);

                // let the constellation know about the viewport constraints
                let ConstellationChan(ref constellation_chan) = rw_data.constellation_chan;
                constellation_chan.send(ConstellationMsg::ViewportConstrained(
                        self.id, constraints)).unwrap();
            }
        }

        // If the entire flow tree is invalid, then it will be reflowed anyhow.
        let needs_dirtying = rw_data.stylist.update();
        let needs_reflow = screen_size_changed && !needs_dirtying;
        unsafe {
            if needs_dirtying {
                LayoutTask::dirty_all_nodes(node);
            }
        }
        if needs_reflow {
            if let Some(mut flow) = self.try_get_layout_root(*node) {
                LayoutTask::reflow_all_nodes(&mut *flow);
            }
        }

        // Create a layout context for use throughout the following passes.
        let mut shared_layout_context = self.build_shared_layout_context(&*rw_data,
                                                                         screen_size_changed,
                                                                         Some(&node),
                                                                         &self.url,
                                                                         data.reflow_info.goal);

        if node.is_dirty() || node.has_dirty_descendants() || rw_data.stylist.is_dirty() {
            // Recalculate CSS styles and rebuild flows and fragments.
            profile(time::ProfilerCategory::LayoutStyleRecalc,
                    self.profiler_metadata(),
                    self.time_profiler_chan.clone(),
                    || {
                // Perform CSS selector matching and flow construction.
                let rw_data = &mut *rw_data;
                match rw_data.parallel_traversal {
                    None => {
                        sequential::traverse_dom_preorder(*node, &shared_layout_context);
                    }
                    Some(ref mut traversal) => {
                        parallel::traverse_dom_preorder(*node, &shared_layout_context, traversal);
                    }
                }
            });

            // Retrieve the (possibly rebuilt) root flow.
            rw_data.root_flow = Some(self.get_layout_root((*node).clone()));

            // Kick off animations if any were triggered.
            animation::process_new_animations(&mut *rw_data, self.id);
        }

        // Send new canvas renderers to the paint task
        while let Ok((layer_id, renderer)) = self.canvas_layers_receiver.try_recv() {
            // Just send if there's an actual renderer
            self.paint_chan.send(LayoutToPaintMsg::CanvasLayer(layer_id, renderer)).unwrap();
        }

        // Perform post-style recalculation layout passes.
        self.perform_post_style_recalc_layout_passes(&data.reflow_info,
                                                     &mut rw_data,
                                                     &mut shared_layout_context);

        let mut root_flow = (*rw_data.root_flow.as_ref().unwrap()).clone();
        match data.query_type {
            ReflowQueryType::ContentBoxQuery(node) =>
                process_content_box_request(node, &mut root_flow, &mut rw_data),
            ReflowQueryType::ContentBoxesQuery(node) =>
                process_content_boxes_request(node, &mut root_flow, &mut rw_data),
            ReflowQueryType::NodeGeometryQuery(node) =>
                self.process_node_geometry_request(node, &mut root_flow, &mut rw_data),
            ReflowQueryType::ResolvedStyleQuery(node, ref pseudo, ref property) =>
                self.process_resolved_style_request(node, pseudo, property, &mut root_flow, &mut rw_data),
            ReflowQueryType::OffsetParentQuery(node) =>
                self.process_offset_parent_query(node, &mut root_flow, &mut rw_data),
            ReflowQueryType::NoQuery => {}
        }


        // Tell script that we're done.
        //
        // FIXME(pcwalton): This should probably be *one* channel, but we can't fix this without
        // either select or a filtered recv() that only looks for messages of a given type.
        data.script_join_chan.send(()).unwrap();
        data.script_chan.send(ConstellationControlMsg::ReflowComplete(self.id, data.id)).unwrap();
    }

    fn set_visible_rects<'a>(&'a self,
                             new_visible_rects: Vec<(LayerId, Rect<Au>)>,
                             possibly_locked_rw_data: &mut Option<MutexGuard<'a, LayoutTaskData>>)
                             -> bool {
        let mut rw_data = self.lock_rw_data(possibly_locked_rw_data);

        // First, determine if we need to regenerate the display lists. This will happen if the
        // layers have moved more than `DISPLAY_PORT_THRESHOLD_SIZE_FACTOR` away from their last
        // positions.
        let mut must_regenerate_display_lists = false;
        let mut old_visible_rects = HashMap::with_hash_state(Default::default());
        let inflation_amount =
            Size2D::new(rw_data.screen_size.width * DISPLAY_PORT_THRESHOLD_SIZE_FACTOR,
                        rw_data.screen_size.height * DISPLAY_PORT_THRESHOLD_SIZE_FACTOR);
        for &(ref layer_id, ref new_visible_rect) in &new_visible_rects {
            match rw_data.visible_rects.get(layer_id) {
                None => {
                    old_visible_rects.insert(*layer_id, *new_visible_rect);
                }
                Some(old_visible_rect) => {
                    old_visible_rects.insert(*layer_id, *old_visible_rect);

                    if !old_visible_rect.inflate(inflation_amount.width, inflation_amount.height)
                                        .intersects(new_visible_rect) {
                        must_regenerate_display_lists = true;
                    }
                }
            }
        }

        if !must_regenerate_display_lists {
            // Update `visible_rects` in case there are new layers that were discovered.
            rw_data.visible_rects = Arc::new(old_visible_rects);
            return true
        }

        debug!("regenerating display lists!");
        for &(ref layer_id, ref new_visible_rect) in &new_visible_rects {
            old_visible_rects.insert(*layer_id, *new_visible_rect);
        }
        rw_data.visible_rects = Arc::new(old_visible_rects);

        // Regenerate the display lists.
        let reflow_info = Reflow {
            goal: ReflowGoal::ForDisplay,
            page_clip_rect: MAX_RECT,
        };

        let mut layout_context = self.build_shared_layout_context(&*rw_data,
                                                                  false,
                                                                  None,
                                                                  &self.url,
                                                                  reflow_info.goal);

        self.perform_post_main_layout_passes(&reflow_info, &mut *rw_data, &mut layout_context);
        true
    }

    fn tick_all_animations<'a>(&'a self,
                               possibly_locked_rw_data: &mut Option<MutexGuard<'a,
                                                                               LayoutTaskData>>) {
        let mut rw_data = self.lock_rw_data(possibly_locked_rw_data);
        animation::tick_all_animations(self, &mut rw_data)
    }

    pub fn tick_animations<'a>(&'a self, rw_data: &mut LayoutTaskData) {
        let reflow_info = Reflow {
            goal: ReflowGoal::ForDisplay,
            page_clip_rect: MAX_RECT,
        };

        let mut layout_context = self.build_shared_layout_context(&*rw_data,
                                                                  false,
                                                                  None,
                                                                  &self.url,
                                                                  reflow_info.goal);

        {
            // Perform an abbreviated style recalc that operates without access to the DOM.
            let mut root_flow = (*rw_data.root_flow.as_ref().unwrap()).clone();
            let animations = &*rw_data.running_animations;
            profile(time::ProfilerCategory::LayoutStyleRecalc,
                    self.profiler_metadata(),
                    self.time_profiler_chan.clone(),
                    || animation::recalc_style_for_animations(root_flow.deref_mut(), animations));
        }

        self.perform_post_style_recalc_layout_passes(&reflow_info,
                                                     &mut *rw_data,
                                                     &mut layout_context);
    }

    fn perform_post_style_recalc_layout_passes<'a>(&'a self,
                                                   data: &Reflow,
                                                   rw_data: &mut LayoutTaskData,
                                                   layout_context: &mut SharedLayoutContext) {
        let mut root_flow = (*rw_data.root_flow.as_ref().unwrap()).clone();
        profile(time::ProfilerCategory::LayoutRestyleDamagePropagation,
                self.profiler_metadata(),
                self.time_profiler_chan.clone(),
                || {
            if opts::get().nonincremental_layout || root_flow.deref_mut()
                                                             .compute_layout_damage()
                                                             .contains(REFLOW_ENTIRE_DOCUMENT) {
                root_flow.deref_mut().reflow_entire_document()
            }
        });

        // Verification of the flow tree, which ensures that all nodes were either marked as leaves
        // or as non-leaves. This becomes a no-op in release builds. (It is inconsequential to
        // memory safety but is a useful debugging tool.)
        self.verify_flow_tree(&mut root_flow);

        if opts::get().trace_layout {
            layout_debug::begin_trace(root_flow.clone());
        }

        // Resolve generated content.
        profile(time::ProfilerCategory::LayoutGeneratedContent,
                self.profiler_metadata(),
                self.time_profiler_chan.clone(),
                || sequential::resolve_generated_content(&mut root_flow, &layout_context));

        // Perform the primary layout passes over the flow tree to compute the locations of all
        // the boxes.
        profile(time::ProfilerCategory::LayoutMain,
                self.profiler_metadata(),
                self.time_profiler_chan.clone(),
                || {
            match rw_data.parallel_traversal {
                None => {
                    // Sequential mode.
                    self.solve_constraints(&mut root_flow, &layout_context)
                }
                Some(ref mut parallel) => {
                    // Parallel mode.
                    self.solve_constraints_parallel(parallel,
                                                    &mut root_flow,
                                                    &mut *layout_context);
                }
            }
        });

        self.perform_post_main_layout_passes(data, rw_data, layout_context);
    }

    fn perform_post_main_layout_passes<'a>(&'a self,
                                           data: &Reflow,
                                           rw_data: &mut LayoutTaskData,
                                           layout_context: &mut SharedLayoutContext) {
        // Build the display list if necessary, and send it to the painter.
        let mut root_flow = (*rw_data.root_flow.as_ref().unwrap()).clone();
        self.compute_abs_pos_and_build_display_list(data,
                                                    &mut root_flow,
                                                    &mut *layout_context,
                                                    rw_data);
        self.first_reflow.set(false);

        if opts::get().trace_layout {
            layout_debug::end_trace();
        }

        if opts::get().dump_flow_tree {
            root_flow.dump();
        }

        rw_data.generation += 1;
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
        debug!("reflowing all nodes!");
        flow::mut_base(flow).restyle_damage.insert(REFLOW | REPAINT);

        for child in flow::child_iter(flow) {
            LayoutTask::reflow_all_nodes(child);
        }
    }

    /// Handles a message to destroy layout data. Layout data must be destroyed on *this* task
    /// because the struct type is transmuted to a different type on the script side.
    unsafe fn handle_reap_layout_data(&self, layout_data: LayoutData) {
        let layout_data_wrapper: LayoutDataWrapper = transmute(layout_data);
        layout_data_wrapper.remove_compositor_layers(self.constellation_chan.clone());
    }

    /// Returns profiling information which is passed to the time profiler.
    fn profiler_metadata(&self) -> ProfilerMetadata {
        Some((&self.url,
              if self.is_iframe {
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

struct FragmentLocatingFragmentIterator {
    node_address: OpaqueNode,
    client_rect: Rect<i32>,
}

impl FragmentLocatingFragmentIterator {
    fn new(node_address: OpaqueNode) -> FragmentLocatingFragmentIterator {
        FragmentLocatingFragmentIterator {
            node_address: node_address,
            client_rect: Rect::zero()
        }
    }
}

struct ParentBorderBoxInfo {
    node_address: OpaqueNode,
    border_box: Rect<Au>,
}

struct ParentOffsetBorderBoxIterator {
    node_address: OpaqueNode,
    last_level: i32,
    has_found_node: bool,
    node_border_box: Rect<Au>,
    parent_nodes: Vec<Option<ParentBorderBoxInfo>>,
}

impl ParentOffsetBorderBoxIterator {
    fn new(node_address: OpaqueNode) -> ParentOffsetBorderBoxIterator {
        ParentOffsetBorderBoxIterator {
            node_address: node_address,
            last_level: -1,
            has_found_node: false,
            node_border_box: Rect::zero(),
            parent_nodes: Vec::new(),
        }
    }
}

impl FragmentBorderBoxIterator for FragmentLocatingFragmentIterator {
    fn process(&mut self, fragment: &Fragment, _: i32, border_box: &Rect<Au>) {
        let style_structs::Border {
            border_top_width: top_width,
            border_right_width: right_width,
            border_bottom_width: bottom_width,
            border_left_width: left_width,
            ..
        } = *fragment.style.get_border();
        self.client_rect.origin.y = top_width.to_px();
        self.client_rect.origin.x = left_width.to_px();
        self.client_rect.size.width = (border_box.size.width - left_width - right_width).to_px();
        self.client_rect.size.height = (border_box.size.height - top_width - bottom_width).to_px();
    }

    fn should_process(&mut self, fragment: &Fragment) -> bool {
        fragment.node == self.node_address
    }
}

// https://drafts.csswg.org/cssom-view/#extensions-to-the-htmlelement-interface
impl FragmentBorderBoxIterator for ParentOffsetBorderBoxIterator {
    fn process(&mut self, fragment: &Fragment, level: i32, border_box: &Rect<Au>) {
        if fragment.node == self.node_address {
            // Found the fragment in the flow tree that matches the
            // DOM node being looked for.
            self.has_found_node = true;
            self.node_border_box = *border_box;

            // offsetParent returns null if the node is fixed.
            if fragment.style.get_box().position == computed_values::position::T::fixed {
                self.parent_nodes.clear();
            }
        } else if level > self.last_level {
            // TODO(gw): Is there a less fragile way of checking whether this
            // fragment is the body element, rather than just checking that
            // the parent nodes stack contains the root node only?
            let is_body_element = self.parent_nodes.len() == 1;

            let is_valid_parent = match (is_body_element,
                                         fragment.style.get_box().position,
                                         &fragment.specific) {
                // Spec says it's valid if any of these are true:
                //  1) Is the body element
                //  2) Is static position *and* is a table or table cell
                //  3) Is not static position
                (true, _, _) |
                (false, computed_values::position::T::static_, &SpecificFragmentInfo::Table) |
                (false, computed_values::position::T::static_, &SpecificFragmentInfo::TableCell) |
                (false, computed_values::position::T::absolute, _) |
                (false, computed_values::position::T::relative, _) |
                (false, computed_values::position::T::fixed, _) => true,

                // Otherwise, it's not a valid parent
                (false, computed_values::position::T::static_, _) => false,
            };

            let parent_info = if is_valid_parent {
                Some(ParentBorderBoxInfo {
                    border_box: *border_box,
                    node_address: fragment.node,
                })
            } else {
                None
            };

            self.parent_nodes.push(parent_info);
        } else if level < self.last_level {
            self.parent_nodes.pop();
        }
    }

    fn should_process(&mut self, _: &Fragment) -> bool {
        !self.has_found_node
    }
}

// The default computed value for background-color is transparent (see
// http://dev.w3.org/csswg/css-backgrounds/#background-color). However, we
// need to propagate the background color from the root HTML/Body
// element (http://dev.w3.org/csswg/css-backgrounds/#special-backgrounds) if
// it is non-transparent. The phrase in the spec "If the canvas background
// is not opaque, what shows through is UA-dependent." is handled by rust-layers
// clearing the frame buffer to white. This ensures that setting a background
// color on an iframe element, while the iframe content itself has a default
// transparent background color is handled correctly.
fn get_root_flow_background_color(flow: &mut Flow) -> AzColor {
    if !flow.is_block_like() {
        return color::transparent()
    }

    let block_flow = flow.as_block();
    let kid = match block_flow.base.children.iter_mut().next() {
        None => return color::transparent(),
        Some(kid) => kid,
    };
    if !kid.is_block_like() {
        return color::transparent()
    }

    let kid_block_flow = kid.as_block();
    kid_block_flow.fragment
                  .style
                  .resolve_color(kid_block_flow.fragment.style.get_background().background_color)
                  .to_gfx_color()
}
