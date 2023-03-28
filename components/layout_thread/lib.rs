/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// Work around https://github.com/rust-lang/rust/issues/62132
#![recursion_limit = "128"]

//! The layout thread. Performs layout on the DOM, builds display lists and sends them to be
//! painted.

#[macro_use]
extern crate crossbeam_channel;
#[macro_use]
extern crate html5ever;
#[macro_use]
extern crate layout;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate profile_traits;

mod dom_wrapper;

use crate::dom_wrapper::{ServoLayoutDocument, ServoLayoutElement, ServoLayoutNode};
use app_units::Au;
use crossbeam_channel::{Receiver, Sender};
use embedder_traits::resources::{self, Resource};
use euclid::{default::Size2D as UntypedSize2D, Point2D, Rect, Scale, Size2D};
use fnv::FnvHashMap;
use fxhash::{FxHashMap, FxHashSet};
use gfx::font;
use gfx::font_cache_thread::FontCacheThread;
use gfx::font_context;
use gfx_traits::{node_id_from_scroll_id, Epoch};
use histogram::Histogram;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use layout::construct::ConstructionResult;
use layout::context::malloc_size_of_persistent_local_context;
use layout::context::LayoutContext;
use layout::context::RegisteredPainter;
use layout::context::RegisteredPainters;
use layout::display_list::items::WebRenderImageInfo;
use layout::display_list::{IndexableText, ToLayout};
use layout::flow::{Flow, FlowFlags, GetBaseFlow, ImmutableFlowUtils, MutableOwnedFlowUtils};
use layout::flow_ref::FlowRef;
use layout::incremental::{RelayoutMode, SpecialRestyleDamage};
use layout::layout_debug;
use layout::parallel;
use layout::query::{
    process_client_rect_query, process_content_box_request, process_content_boxes_request,
    process_element_inner_text_query, process_node_scroll_area_request,
    process_node_scroll_id_request, process_offset_parent_query,
    process_resolved_font_style_request, process_resolved_style_request, LayoutRPCImpl,
    LayoutThreadData,
};
use layout::sequential;
use layout::traversal::{
    construct_flows_at_ancestors, ComputeStackingRelativePositions, PreorderFlowTraversal,
    RecalcStyleAndConstructFlows,
};
use layout::wrapper::LayoutNodeLayoutData;
use layout_traits::LayoutThreadFactory;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use metrics::{PaintTimeMetrics, ProfilerMetadataFactory, ProgressiveWebMetric};
use msg::constellation_msg::{
    BackgroundHangMonitor, BackgroundHangMonitorRegister, HangAnnotation,
};
use msg::constellation_msg::{BrowsingContextId, MonitoredComponentId, TopLevelBrowsingContextId};
use msg::constellation_msg::{LayoutHangAnnotation, MonitoredComponentType, PipelineId};
use net_traits::image_cache::{ImageCache, UsePlaceholder};
use parking_lot::RwLock;
use profile_traits::mem::{self as profile_mem, Report, ReportKind, ReportsChan};
use profile_traits::time::{self as profile_time, profile, TimerMetadata};
use profile_traits::time::{TimerMetadataFrameType, TimerMetadataReflowType};
use script_layout_interface::message::{LayoutThreadInit, Msg, NodesFromPointQueryType, Reflow};
use script_layout_interface::message::{QueryMsg, ReflowComplete, ReflowGoal, ScriptReflow};
use script_layout_interface::rpc::TextIndexResponse;
use script_layout_interface::rpc::{LayoutRPC, OffsetParentResponse};
use script_layout_interface::wrapper_traits::LayoutNode;
use script_traits::{ConstellationControlMsg, LayoutControlMsg, LayoutMsg as ConstellationMsg};
use script_traits::{DrawAPaintImageResult, IFrameSizeMsg, PaintWorkletError, WindowSizeType};
use script_traits::{Painter, WebrenderIpcSender};
use script_traits::{ScrollState, UntrustedNodeAddress, WindowSizeData};
use servo_arc::Arc as ServoArc;
use servo_atoms::Atom;
use servo_config::opts;
use servo_url::{ImmutableOrigin, ServoUrl};
use std::borrow::ToOwned;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::process;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::Duration;
use style::animation::{AnimationSetKey, DocumentAnimationSet, ElementAnimationSet};
use style::context::SharedStyleContext;
use style::context::{QuirksMode, RegisteredSpeculativePainter, RegisteredSpeculativePainters};
use style::dom::{ShowSubtree, ShowSubtreeDataAndPrimaryValues, TDocument, TElement, TNode};
use style::driver;
use style::error_reporting::RustLogReporter;
use style::global_style_data::{GLOBAL_STYLE_DATA, STYLE_THREAD_POOL};
use style::invalidation::element::restyle_hints::RestyleHint;
use style::logical_geometry::LogicalPoint;
use style::media_queries::{Device, MediaList, MediaType};
use style::properties::PropertyId;
use style::selector_parser::{PseudoElement, SnapshotMap};
use style::servo::restyle_damage::ServoRestyleDamage;
use style::shared_lock::{SharedRwLock, SharedRwLockReadGuard, StylesheetGuards};
use style::stylesheets::{
    DocumentStyleSheet, Origin, Stylesheet, StylesheetInDocument, UserAgentStylesheets,
};
use style::stylist::Stylist;
use style::thread_state::{self, ThreadState};
use style::traversal::DomTraversal;
use style::traversal_flags::TraversalFlags;
use style_traits::CSSPixel;
use style_traits::DevicePixel;
use style_traits::SpeculativePainter;

/// Information needed by the layout thread.
pub struct LayoutThread {
    /// The ID of the pipeline that we belong to.
    id: PipelineId,

    /// The ID of the top-level browsing context that we belong to.
    top_level_browsing_context_id: TopLevelBrowsingContextId,

    /// The URL of the pipeline that we belong to.
    url: ServoUrl,

    /// Performs CSS selector matching and style resolution.
    stylist: Stylist,

    /// Is the current reflow of an iframe, as opposed to a root window?
    is_iframe: bool,

    /// The port on which we receive messages from the script thread.
    port: Receiver<Msg>,

    /// The port on which we receive messages from the constellation.
    pipeline_port: Receiver<LayoutControlMsg>,

    /// The port on which we receive messages from the font cache thread.
    font_cache_receiver: Receiver<()>,

    /// The channel on which the font cache can send messages to us.
    font_cache_sender: IpcSender<()>,

    /// A means of communication with the background hang monitor.
    background_hang_monitor: Box<dyn BackgroundHangMonitor>,

    /// The channel on which messages can be sent to the constellation.
    constellation_chan: IpcSender<ConstellationMsg>,

    /// The channel on which messages can be sent to the script thread.
    script_chan: IpcSender<ConstellationControlMsg>,

    /// The channel on which messages can be sent to the time profiler.
    time_profiler_chan: profile_time::ProfilerChan,

    /// The channel on which messages can be sent to the memory profiler.
    mem_profiler_chan: profile_mem::ProfilerChan,

    /// Reference to the script thread image cache.
    image_cache: Arc<dyn ImageCache>,

    /// Public interface to the font cache thread.
    font_cache_thread: FontCacheThread,

    /// Is this the first reflow in this LayoutThread?
    first_reflow: Cell<bool>,

    /// Flag to indicate whether to use parallel operations
    parallel_flag: bool,

    /// Starts at zero, and increased by one every time a layout completes.
    /// This can be used to easily check for invalid stale data.
    generation: Cell<u32>,

    /// The number of Web fonts that have been requested but not yet loaded.
    outstanding_web_fonts: Arc<AtomicUsize>,

    /// The root of the flow tree.
    root_flow: RefCell<Option<FlowRef>>,

    /// A counter for epoch messages
    epoch: Cell<Epoch>,

    /// The size of the viewport. This may be different from the size of the screen due to viewport
    /// constraints.
    viewport_size: UntypedSize2D<Au>,

    /// A mutex to allow for fast, read-only RPC of layout's internal data
    /// structures, while still letting the LayoutThread modify them.
    ///
    /// All the other elements of this struct are read-only.
    rw_data: Arc<Mutex<LayoutThreadData>>,

    webrender_image_cache: Arc<RwLock<FnvHashMap<(ServoUrl, UsePlaceholder), WebRenderImageInfo>>>,

    /// The executors for paint worklets.
    registered_painters: RegisteredPaintersImpl,

    /// Webrender interface.
    webrender_api: WebrenderIpcSender,

    /// Paint time metrics.
    paint_time_metrics: PaintTimeMetrics,

    /// The time a layout query has waited before serviced by layout thread.
    layout_query_waiting_time: Histogram,

    /// The sizes of all iframes encountered during the last layout operation.
    last_iframe_sizes: RefCell<HashMap<BrowsingContextId, Size2D<f32, CSSPixel>>>,

    /// Flag that indicates if LayoutThread is busy handling a request.
    busy: Arc<AtomicBool>,

    /// Load web fonts synchronously to avoid non-deterministic network-driven reflows.
    load_webfonts_synchronously: bool,

    /// Dumps the display list form after a layout.
    dump_display_list: bool,

    /// Dumps the display list in JSON form after a layout.
    dump_display_list_json: bool,

    /// Dumps the DOM after restyle.
    dump_style_tree: bool,

    /// Dumps the flow tree after a layout.
    dump_rule_tree: bool,

    /// Emits notifications when there is a relayout.
    relayout_event: bool,

    /// True to turn off incremental layout.
    nonincremental_layout: bool,

    /// True if each step of layout is traced to an external JSON file
    /// for debugging purposes. Setting this implies sequential layout
    /// and paint.
    trace_layout: bool,

    /// Dumps the flow tree after a layout.
    dump_flow_tree: bool,
}

impl LayoutThreadFactory for LayoutThread {
    type Message = Msg;

    /// Spawns a new layout thread.
    fn create(
        id: PipelineId,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
        url: ServoUrl,
        is_iframe: bool,
        chan: (Sender<Msg>, Receiver<Msg>),
        pipeline_port: IpcReceiver<LayoutControlMsg>,
        background_hang_monitor_register: Box<dyn BackgroundHangMonitorRegister>,
        constellation_chan: IpcSender<ConstellationMsg>,
        script_chan: IpcSender<ConstellationControlMsg>,
        image_cache: Arc<dyn ImageCache>,
        font_cache_thread: FontCacheThread,
        time_profiler_chan: profile_time::ProfilerChan,
        mem_profiler_chan: profile_mem::ProfilerChan,
        webrender_api_sender: WebrenderIpcSender,
        paint_time_metrics: PaintTimeMetrics,
        busy: Arc<AtomicBool>,
        load_webfonts_synchronously: bool,
        window_size: WindowSizeData,
        dump_display_list: bool,
        dump_display_list_json: bool,
        dump_style_tree: bool,
        dump_rule_tree: bool,
        relayout_event: bool,
        nonincremental_layout: bool,
        trace_layout: bool,
        dump_flow_tree: bool,
    ) {
        thread::Builder::new()
            .name(format!("Layout{}", id))
            .spawn(move || {
                thread_state::initialize(ThreadState::LAYOUT);

                // In order to get accurate crash reports, we install the top-level bc id.
                TopLevelBrowsingContextId::install(top_level_browsing_context_id);

                {
                    // Ensures layout thread is destroyed before we send shutdown message
                    let sender = chan.0;

                    let background_hang_monitor = background_hang_monitor_register
                        .register_component(
                            MonitoredComponentId(id, MonitoredComponentType::Layout),
                            Duration::from_millis(1000),
                            Duration::from_millis(5000),
                            None,
                        );

                    let layout = LayoutThread::new(
                        id,
                        top_level_browsing_context_id,
                        url,
                        is_iframe,
                        chan.1,
                        pipeline_port,
                        background_hang_monitor,
                        constellation_chan,
                        script_chan,
                        image_cache,
                        font_cache_thread,
                        time_profiler_chan,
                        mem_profiler_chan.clone(),
                        webrender_api_sender,
                        paint_time_metrics,
                        busy,
                        load_webfonts_synchronously,
                        window_size,
                        dump_display_list,
                        dump_display_list_json,
                        dump_style_tree,
                        dump_rule_tree,
                        relayout_event,
                        nonincremental_layout,
                        trace_layout,
                        dump_flow_tree,
                    );

                    let reporter_name = format!("layout-reporter-{}", id);
                    mem_profiler_chan.run_with_memory_reporting(
                        || {
                            layout.start();
                        },
                        reporter_name,
                        sender,
                        Msg::CollectReports,
                    );
                }
            })
            .expect("Thread spawning failed");
    }
}

struct ScriptReflowResult {
    script_reflow: ScriptReflow,
    result: RefCell<Option<ReflowComplete>>,
}

impl Deref for ScriptReflowResult {
    type Target = ScriptReflow;
    fn deref(&self) -> &ScriptReflow {
        &self.script_reflow
    }
}

impl DerefMut for ScriptReflowResult {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.script_reflow
    }
}

impl ScriptReflowResult {
    fn new(script_reflow: ScriptReflow) -> ScriptReflowResult {
        ScriptReflowResult {
            script_reflow: script_reflow,
            result: RefCell::new(Some(Default::default())),
        }
    }
}

impl Drop for ScriptReflowResult {
    fn drop(&mut self) {
        self.script_reflow
            .script_join_chan
            .send(self.result.borrow_mut().take().unwrap())
            .unwrap();
    }
}

/// The `LayoutThread` `rw_data` lock must remain locked until the first reflow,
/// as RPC calls don't make sense until then. Use this in combination with
/// `LayoutThread::lock_rw_data` and `LayoutThread::return_rw_data`.
pub enum RWGuard<'a> {
    /// If the lock was previously held, from when the thread started.
    Held(MutexGuard<'a, LayoutThreadData>),
    /// If the lock was just used, and has been returned since there has been
    /// a reflow already.
    Used(MutexGuard<'a, LayoutThreadData>),
}

impl<'a> Deref for RWGuard<'a> {
    type Target = LayoutThreadData;
    fn deref(&self) -> &LayoutThreadData {
        match *self {
            RWGuard::Held(ref x) => &**x,
            RWGuard::Used(ref x) => &**x,
        }
    }
}

impl<'a> DerefMut for RWGuard<'a> {
    fn deref_mut(&mut self) -> &mut LayoutThreadData {
        match *self {
            RWGuard::Held(ref mut x) => &mut **x,
            RWGuard::Used(ref mut x) => &mut **x,
        }
    }
}

struct RwData<'a, 'b: 'a> {
    rw_data: &'b Arc<Mutex<LayoutThreadData>>,
    possibly_locked_rw_data: &'a mut Option<MutexGuard<'b, LayoutThreadData>>,
}

impl<'a, 'b: 'a> RwData<'a, 'b> {
    /// If no reflow has happened yet, this will just return the lock in
    /// `possibly_locked_rw_data`. Otherwise, it will acquire the `rw_data` lock.
    ///
    /// If you do not wish RPCs to remain blocked, just drop the `RWGuard`
    /// returned from this function. If you _do_ wish for them to remain blocked,
    /// use `block`.
    fn lock(&mut self) -> RWGuard<'b> {
        match self.possibly_locked_rw_data.take() {
            None => RWGuard::Used(self.rw_data.lock().unwrap()),
            Some(x) => RWGuard::Held(x),
        }
    }
}

fn add_font_face_rules(
    stylesheet: &Stylesheet,
    guard: &SharedRwLockReadGuard,
    device: &Device,
    font_cache_thread: &FontCacheThread,
    font_cache_sender: &IpcSender<()>,
    outstanding_web_fonts_counter: &Arc<AtomicUsize>,
    load_webfonts_synchronously: bool,
) {
    if load_webfonts_synchronously {
        let (sender, receiver) = ipc::channel().unwrap();
        stylesheet.effective_font_face_rules(&device, guard, |rule| {
            if let Some(font_face) = rule.font_face() {
                let effective_sources = font_face.effective_sources();
                font_cache_thread.add_web_font(
                    font_face.family().clone(),
                    effective_sources,
                    sender.clone(),
                );
                receiver.recv().unwrap();
            }
        })
    } else {
        stylesheet.effective_font_face_rules(&device, guard, |rule| {
            if let Some(font_face) = rule.font_face() {
                let effective_sources = font_face.effective_sources();
                outstanding_web_fonts_counter.fetch_add(1, Ordering::SeqCst);
                font_cache_thread.add_web_font(
                    font_face.family().clone(),
                    effective_sources,
                    (*font_cache_sender).clone(),
                );
            }
        })
    }
}

impl LayoutThread {
    /// Creates a new `LayoutThread` structure.
    fn new(
        id: PipelineId,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
        url: ServoUrl,
        is_iframe: bool,
        port: Receiver<Msg>,
        pipeline_port: IpcReceiver<LayoutControlMsg>,
        background_hang_monitor: Box<dyn BackgroundHangMonitor>,
        constellation_chan: IpcSender<ConstellationMsg>,
        script_chan: IpcSender<ConstellationControlMsg>,
        image_cache: Arc<dyn ImageCache>,
        font_cache_thread: FontCacheThread,
        time_profiler_chan: profile_time::ProfilerChan,
        mem_profiler_chan: profile_mem::ProfilerChan,
        webrender_api: WebrenderIpcSender,
        paint_time_metrics: PaintTimeMetrics,
        busy: Arc<AtomicBool>,
        load_webfonts_synchronously: bool,
        window_size: WindowSizeData,
        dump_display_list: bool,
        dump_display_list_json: bool,
        dump_style_tree: bool,
        dump_rule_tree: bool,
        relayout_event: bool,
        nonincremental_layout: bool,
        trace_layout: bool,
        dump_flow_tree: bool,
    ) -> LayoutThread {
        // Let webrender know about this pipeline by sending an empty display list.
        webrender_api.send_initial_transaction(id.to_webrender());

        let device = Device::new(
            MediaType::screen(),
            QuirksMode::NoQuirks,
            window_size.initial_viewport,
            window_size.device_pixel_ratio,
        );

        // Proxy IPC messages from the pipeline to the layout thread.
        let pipeline_receiver = ROUTER.route_ipc_receiver_to_new_crossbeam_receiver(pipeline_port);

        // Ask the router to proxy IPC messages from the font cache thread to the layout thread.
        let (ipc_font_cache_sender, ipc_font_cache_receiver) = ipc::channel().unwrap();
        let font_cache_receiver =
            ROUTER.route_ipc_receiver_to_new_crossbeam_receiver(ipc_font_cache_receiver);

        LayoutThread {
            id: id,
            top_level_browsing_context_id: top_level_browsing_context_id,
            url: url,
            is_iframe: is_iframe,
            port: port,
            pipeline_port: pipeline_receiver,
            script_chan: script_chan,
            background_hang_monitor,
            constellation_chan: constellation_chan.clone(),
            time_profiler_chan: time_profiler_chan,
            mem_profiler_chan: mem_profiler_chan,
            registered_painters: RegisteredPaintersImpl(Default::default()),
            image_cache: image_cache,
            font_cache_thread: font_cache_thread,
            first_reflow: Cell::new(true),
            font_cache_receiver: font_cache_receiver,
            font_cache_sender: ipc_font_cache_sender,
            parallel_flag: true,
            generation: Cell::new(0),
            outstanding_web_fonts: Arc::new(AtomicUsize::new(0)),
            root_flow: RefCell::new(None),
            // Epoch starts at 1 because of the initial display list for epoch 0 that we send to WR
            epoch: Cell::new(Epoch(1)),
            viewport_size: Size2D::new(Au(0), Au(0)),
            webrender_api,
            stylist: Stylist::new(device, QuirksMode::NoQuirks),
            rw_data: Arc::new(Mutex::new(LayoutThreadData {
                constellation_chan: constellation_chan,
                display_list: None,
                indexable_text: IndexableText::default(),
                content_box_response: None,
                content_boxes_response: Vec::new(),
                client_rect_response: Rect::zero(),
                scroll_id_response: None,
                scroll_area_response: Rect::zero(),
                resolved_style_response: String::new(),
                resolved_font_style_response: None,
                offset_parent_response: OffsetParentResponse::empty(),
                scroll_offsets: HashMap::new(),
                text_index_response: TextIndexResponse(None),
                nodes_from_point_response: vec![],
                element_inner_text_response: String::new(),
                inner_window_dimensions_response: None,
            })),
            webrender_image_cache: Arc::new(RwLock::new(FnvHashMap::default())),
            paint_time_metrics: paint_time_metrics,
            layout_query_waiting_time: Histogram::new(),
            last_iframe_sizes: Default::default(),
            busy,
            load_webfonts_synchronously,
            dump_display_list,
            dump_display_list_json,
            dump_style_tree,
            dump_rule_tree,
            relayout_event,
            nonincremental_layout,
            trace_layout,
            dump_flow_tree,
        }
    }

    /// Starts listening on the port.
    fn start(mut self) {
        let rw_data = self.rw_data.clone();
        let mut possibly_locked_rw_data = Some(rw_data.lock().unwrap());
        let mut rw_data = RwData {
            rw_data: &rw_data,
            possibly_locked_rw_data: &mut possibly_locked_rw_data,
        };
        while self.handle_request(&mut rw_data) {
            // Loop indefinitely.
        }
    }

    // Create a layout context for use in building display lists, hit testing, &c.
    fn build_layout_context<'a>(
        &'a self,
        guards: StylesheetGuards<'a>,
        snapshot_map: &'a SnapshotMap,
        origin: ImmutableOrigin,
        animation_timeline_value: f64,
        animations: &DocumentAnimationSet,
        stylesheets_changed: bool,
    ) -> LayoutContext<'a> {
        let traversal_flags = match stylesheets_changed {
            true => TraversalFlags::ForCSSRuleChanges,
            false => TraversalFlags::empty(),
        };

        LayoutContext {
            id: self.id,
            origin,
            style_context: SharedStyleContext {
                stylist: &self.stylist,
                options: GLOBAL_STYLE_DATA.options.clone(),
                guards,
                visited_styles_enabled: false,
                animations: animations.clone(),
                registered_speculative_painters: &self.registered_painters,
                current_time_for_animations: animation_timeline_value,
                traversal_flags,
                snapshot_map: snapshot_map,
            },
            image_cache: self.image_cache.clone(),
            font_cache_thread: Mutex::new(self.font_cache_thread.clone()),
            webrender_image_cache: self.webrender_image_cache.clone(),
            pending_images: Mutex::new(vec![]),
            registered_painters: &self.registered_painters,
        }
    }

    fn notify_activity_to_hang_monitor(&self, request: &Msg) {
        let hang_annotation = match request {
            Msg::AddStylesheet(..) => LayoutHangAnnotation::AddStylesheet,
            Msg::RemoveStylesheet(..) => LayoutHangAnnotation::RemoveStylesheet,
            Msg::SetQuirksMode(..) => LayoutHangAnnotation::SetQuirksMode,
            Msg::Reflow(..) => LayoutHangAnnotation::Reflow,
            Msg::GetRPC(..) => LayoutHangAnnotation::GetRPC,
            Msg::CollectReports(..) => LayoutHangAnnotation::CollectReports,
            Msg::PrepareToExit(..) => LayoutHangAnnotation::PrepareToExit,
            Msg::ExitNow => LayoutHangAnnotation::ExitNow,
            Msg::GetCurrentEpoch(..) => LayoutHangAnnotation::GetCurrentEpoch,
            Msg::GetWebFontLoadState(..) => LayoutHangAnnotation::GetWebFontLoadState,
            Msg::CreateLayoutThread(..) => LayoutHangAnnotation::CreateLayoutThread,
            Msg::SetFinalUrl(..) => LayoutHangAnnotation::SetFinalUrl,
            Msg::SetScrollStates(..) => LayoutHangAnnotation::SetScrollStates,
            Msg::UpdateScrollStateFromScript(..) => {
                LayoutHangAnnotation::UpdateScrollStateFromScript
            },
            Msg::RegisterPaint(..) => LayoutHangAnnotation::RegisterPaint,
            Msg::SetNavigationStart(..) => LayoutHangAnnotation::SetNavigationStart,
        };
        self.background_hang_monitor
            .notify_activity(HangAnnotation::Layout(hang_annotation));
    }

    /// Receives and dispatches messages from the script and constellation threads
    fn handle_request<'a, 'b>(&mut self, possibly_locked_rw_data: &mut RwData<'a, 'b>) -> bool {
        enum Request {
            FromPipeline(LayoutControlMsg),
            FromScript(Msg),
            FromFontCache,
        }

        // Notify the background-hang-monitor we are waiting for an event.
        self.background_hang_monitor.notify_wait();

        let request = select! {
            recv(self.pipeline_port) -> msg => Request::FromPipeline(msg.unwrap()),
            recv(self.port) -> msg => Request::FromScript(msg.unwrap()),
            recv(self.font_cache_receiver) -> msg => { msg.unwrap(); Request::FromFontCache }
        };

        self.busy.store(true, Ordering::Relaxed);
        let result = match request {
            Request::FromPipeline(LayoutControlMsg::SetScrollStates(new_scroll_states)) => self
                .handle_request_helper(
                    Msg::SetScrollStates(new_scroll_states),
                    possibly_locked_rw_data,
                ),
            Request::FromPipeline(LayoutControlMsg::GetCurrentEpoch(sender)) => {
                self.handle_request_helper(Msg::GetCurrentEpoch(sender), possibly_locked_rw_data)
            },
            Request::FromPipeline(LayoutControlMsg::GetWebFontLoadState(sender)) => self
                .handle_request_helper(Msg::GetWebFontLoadState(sender), possibly_locked_rw_data),
            Request::FromPipeline(LayoutControlMsg::ExitNow) => {
                self.handle_request_helper(Msg::ExitNow, possibly_locked_rw_data)
            },
            Request::FromPipeline(LayoutControlMsg::PaintMetric(epoch, paint_time)) => {
                self.paint_time_metrics.maybe_set_metric(epoch, paint_time);
                true
            },
            Request::FromScript(msg) => self.handle_request_helper(msg, possibly_locked_rw_data),
            Request::FromFontCache => {
                let _rw_data = possibly_locked_rw_data.lock();
                self.outstanding_web_fonts.fetch_sub(1, Ordering::SeqCst);
                font_context::invalidate_font_caches();
                self.script_chan
                    .send(ConstellationControlMsg::WebFontLoaded(self.id))
                    .unwrap();
                true
            },
        };
        self.busy.store(false, Ordering::Relaxed);
        result
    }

    /// Receives and dispatches messages from other threads.
    fn handle_request_helper<'a, 'b>(
        &mut self,
        request: Msg,
        possibly_locked_rw_data: &mut RwData<'a, 'b>,
    ) -> bool {
        self.notify_activity_to_hang_monitor(&request);

        match request {
            Msg::AddStylesheet(stylesheet, before_stylesheet) => {
                let guard = stylesheet.shared_lock.read();
                self.handle_add_stylesheet(&stylesheet, &guard);

                match before_stylesheet {
                    Some(insertion_point) => self.stylist.insert_stylesheet_before(
                        DocumentStyleSheet(stylesheet.clone()),
                        DocumentStyleSheet(insertion_point),
                        &guard,
                    ),
                    None => self
                        .stylist
                        .append_stylesheet(DocumentStyleSheet(stylesheet.clone()), &guard),
                }
            },
            Msg::RemoveStylesheet(stylesheet) => {
                let guard = stylesheet.shared_lock.read();
                self.stylist
                    .remove_stylesheet(DocumentStyleSheet(stylesheet.clone()), &guard);
            },
            Msg::SetQuirksMode(mode) => self.handle_set_quirks_mode(mode),
            Msg::GetRPC(response_chan) => {
                response_chan
                    .send(Box::new(LayoutRPCImpl(self.rw_data.clone())) as Box<dyn LayoutRPC + Send>)
                    .unwrap();
            },
            Msg::Reflow(data) => {
                let mut data = ScriptReflowResult::new(data);
                profile(
                    profile_time::ProfilerCategory::LayoutPerform,
                    self.profiler_metadata(),
                    self.time_profiler_chan.clone(),
                    || self.handle_reflow(&mut data, possibly_locked_rw_data),
                );
            },
            Msg::SetScrollStates(new_scroll_states) => {
                self.set_scroll_states(new_scroll_states, possibly_locked_rw_data);
            },
            Msg::UpdateScrollStateFromScript(state) => {
                let mut rw_data = possibly_locked_rw_data.lock();
                rw_data
                    .scroll_offsets
                    .insert(state.scroll_id, state.scroll_offset);

                let point = Point2D::new(-state.scroll_offset.x, -state.scroll_offset.y);
                self.webrender_api.send_scroll_node(
                    webrender_api::units::LayoutPoint::from_untyped(point),
                    state.scroll_id,
                    webrender_api::ScrollClamping::ToContentBounds,
                );
            },
            Msg::CollectReports(reports_chan) => {
                self.collect_reports(reports_chan, possibly_locked_rw_data);
            },
            Msg::GetCurrentEpoch(sender) => {
                let _rw_data = possibly_locked_rw_data.lock();
                sender.send(self.epoch.get()).unwrap();
            },
            Msg::GetWebFontLoadState(sender) => {
                let _rw_data = possibly_locked_rw_data.lock();
                let outstanding_web_fonts = self.outstanding_web_fonts.load(Ordering::SeqCst);
                sender.send(outstanding_web_fonts != 0).unwrap();
            },
            Msg::CreateLayoutThread(info) => self.create_layout_thread(info),
            Msg::SetFinalUrl(final_url) => {
                self.url = final_url;
            },
            Msg::RegisterPaint(name, mut properties, painter) => {
                debug!("Registering the painter");
                let properties = properties
                    .drain(..)
                    .filter_map(|name| {
                        let id = PropertyId::parse_enabled_for_all_content(&*name).ok()?;
                        Some((name.clone(), id))
                    })
                    .filter(|&(_, ref id)| !id.is_shorthand())
                    .collect();
                let registered_painter = RegisteredPainterImpl {
                    name: name.clone(),
                    properties,
                    painter,
                };
                self.registered_painters.0.insert(name, registered_painter);
            },
            Msg::PrepareToExit(response_chan) => {
                self.prepare_to_exit(response_chan);
                return false;
            },
            // Receiving the Exit message at this stage only happens when layout is undergoing a "force exit".
            Msg::ExitNow => {
                debug!("layout: ExitNow received");
                self.exit_now();
                return false;
            },
            Msg::SetNavigationStart(time) => {
                self.paint_time_metrics.set_navigation_start(time);
            },
        }

        true
    }

    fn collect_reports<'a, 'b>(
        &self,
        reports_chan: ReportsChan,
        possibly_locked_rw_data: &mut RwData<'a, 'b>,
    ) {
        let mut reports = vec![];
        // Servo uses vanilla jemalloc, which doesn't have a
        // malloc_enclosing_size_of function.
        let mut ops = MallocSizeOfOps::new(servo_allocator::usable_size, None, None);

        // FIXME(njn): Just measuring the display tree for now.
        let rw_data = possibly_locked_rw_data.lock();
        let display_list = rw_data.display_list.as_ref();
        let formatted_url = &format!("url({})", self.url);
        reports.push(Report {
            path: path![formatted_url, "layout-thread", "display-list"],
            kind: ReportKind::ExplicitJemallocHeapSize,
            size: display_list.map_or(0, |sc| sc.size_of(&mut ops)),
        });

        reports.push(Report {
            path: path![formatted_url, "layout-thread", "stylist"],
            kind: ReportKind::ExplicitJemallocHeapSize,
            size: self.stylist.size_of(&mut ops),
        });

        // The LayoutThread has data in Persistent TLS...
        reports.push(Report {
            path: path![formatted_url, "layout-thread", "local-context"],
            kind: ReportKind::ExplicitJemallocHeapSize,
            size: malloc_size_of_persistent_local_context(&mut ops),
        });

        reports_chan.send(reports);
    }

    fn create_layout_thread(&self, info: LayoutThreadInit) {
        LayoutThread::create(
            info.id,
            self.top_level_browsing_context_id,
            info.url.clone(),
            info.is_parent,
            info.layout_pair,
            info.pipeline_port,
            info.background_hang_monitor_register,
            info.constellation_chan,
            info.script_chan,
            info.image_cache,
            self.font_cache_thread.clone(),
            self.time_profiler_chan.clone(),
            self.mem_profiler_chan.clone(),
            self.webrender_api.clone(),
            info.paint_time_metrics,
            info.layout_is_busy,
            self.load_webfonts_synchronously,
            info.window_size,
            self.dump_display_list,
            self.dump_display_list_json,
            self.dump_style_tree,
            self.dump_rule_tree,
            self.relayout_event,
            self.nonincremental_layout,
            self.trace_layout,
            self.dump_flow_tree,
        );
    }

    /// Enters a quiescent state in which no new messages will be processed until an `ExitNow` is
    /// received. A pong is immediately sent on the given response channel.
    fn prepare_to_exit(&mut self, response_chan: Sender<()>) {
        response_chan.send(()).unwrap();
        loop {
            match self.port.recv().unwrap() {
                Msg::ExitNow => {
                    debug!("layout thread is exiting...");
                    self.exit_now();
                    break;
                },
                Msg::CollectReports(_) => {
                    // Just ignore these messages at this point.
                },
                _ => panic!("layout: unexpected message received after `PrepareToExitMsg`"),
            }
        }
    }

    /// Shuts down the layout thread now. If there are any DOM nodes left, layout will now (safely)
    /// crash.
    fn exit_now(&mut self) {
        // Drop the root flow explicitly to avoid holding style data, such as
        // rule nodes.  The `Stylist` checks when it is dropped that all rule
        // nodes have been GCed, so we want drop anyone who holds them first.
        let waiting_time_min = self.layout_query_waiting_time.minimum().unwrap_or(0);
        let waiting_time_max = self.layout_query_waiting_time.maximum().unwrap_or(0);
        let waiting_time_mean = self.layout_query_waiting_time.mean().unwrap_or(0);
        let waiting_time_stddev = self.layout_query_waiting_time.stddev().unwrap_or(0);
        debug!(
            "layout: query waiting time: min: {}, max: {}, mean: {}, standard_deviation: {}",
            waiting_time_min, waiting_time_max, waiting_time_mean, waiting_time_stddev
        );

        self.root_flow.borrow_mut().take();
        self.background_hang_monitor.unregister();
    }

    fn handle_add_stylesheet(&self, stylesheet: &Stylesheet, guard: &SharedRwLockReadGuard) {
        // Find all font-face rules and notify the font cache of them.
        // GWTODO: Need to handle unloading web fonts.
        if stylesheet.is_effective_for_device(self.stylist.device(), &guard) {
            add_font_face_rules(
                &*stylesheet,
                &guard,
                self.stylist.device(),
                &self.font_cache_thread,
                &self.font_cache_sender,
                &self.outstanding_web_fonts,
                self.load_webfonts_synchronously,
            );
        }
    }

    /// Sets quirks mode for the document, causing the quirks mode stylesheet to be used.
    fn handle_set_quirks_mode<'a, 'b>(&mut self, quirks_mode: QuirksMode) {
        self.stylist.set_quirks_mode(quirks_mode);
    }

    fn try_get_layout_root<'dom>(&self, node: impl LayoutNode<'dom>) -> Option<FlowRef> {
        let result = node.mutate_layout_data()?.flow_construction_result.get();

        let mut flow = match result {
            ConstructionResult::Flow(mut flow, abs_descendants) => {
                // Note: Assuming that the root has display 'static' (as per
                // CSS Section 9.3.1). If it was absolutely positioned,
                // it would return a reference to itself in `abs_descendants`
                // and would lead to a circular reference. Otherwise, we
                // set Root as CB and push remaining absolute descendants.
                if flow
                    .base()
                    .flags
                    .contains(FlowFlags::IS_ABSOLUTELY_POSITIONED)
                {
                    flow.set_absolute_descendants(abs_descendants);
                } else {
                    flow.push_absolute_descendants(abs_descendants);
                }

                flow
            },
            _ => return None,
        };

        FlowRef::deref_mut(&mut flow).mark_as_root();

        Some(flow)
    }

    /// Performs layout constraint solving.
    ///
    /// This corresponds to `Reflow()` in Gecko and `layout()` in WebKit/Blink and should be
    /// benchmarked against those two. It is marked `#[inline(never)]` to aid profiling.
    #[inline(never)]
    fn solve_constraints(layout_root: &mut dyn Flow, layout_context: &LayoutContext) {
        let _scope = layout_debug_scope!("solve_constraints");
        sequential::reflow(layout_root, layout_context, RelayoutMode::Incremental);
    }

    /// Performs layout constraint solving in parallel.
    ///
    /// This corresponds to `Reflow()` in Gecko and `layout()` in WebKit/Blink and should be
    /// benchmarked against those two. It is marked `#[inline(never)]` to aid profiling.
    #[inline(never)]
    fn solve_constraints_parallel(
        traversal: &rayon::ThreadPool,
        layout_root: &mut dyn Flow,
        profiler_metadata: Option<TimerMetadata>,
        time_profiler_chan: profile_time::ProfilerChan,
        layout_context: &LayoutContext,
    ) {
        let _scope = layout_debug_scope!("solve_constraints_parallel");

        // NOTE: this currently computes borders, so any pruning should separate that
        // operation out.
        parallel::reflow(
            layout_root,
            profiler_metadata,
            time_profiler_chan,
            layout_context,
            traversal,
        );
    }

    /// Computes the stacking-relative positions of all flows and, if the painting is dirty and the
    /// reflow type need it, builds the display list.
    fn compute_abs_pos_and_build_display_list(
        &self,
        data: &Reflow,
        reflow_goal: &ReflowGoal,
        document: Option<&ServoLayoutDocument>,
        layout_root: &mut dyn Flow,
        layout_context: &mut LayoutContext,
        rw_data: &mut LayoutThreadData,
    ) {
        let writing_mode = layout_root.base().writing_mode;
        let (metadata, sender) = (self.profiler_metadata(), self.time_profiler_chan.clone());
        profile(
            profile_time::ProfilerCategory::LayoutDispListBuild,
            metadata.clone(),
            sender.clone(),
            || {
                layout_root.mut_base().stacking_relative_position =
                    LogicalPoint::zero(writing_mode)
                        .to_physical(writing_mode, self.viewport_size)
                        .to_vector();

                layout_root.mut_base().clip = data.page_clip_rect;

                let traversal = ComputeStackingRelativePositions {
                    layout_context: layout_context,
                };
                traversal.traverse(layout_root);

                if layout_root
                    .base()
                    .restyle_damage
                    .contains(ServoRestyleDamage::REPAINT) ||
                    rw_data.display_list.is_none()
                {
                    if reflow_goal.needs_display_list() {
                        let background_color = get_root_flow_background_color(layout_root);
                        let mut build_state = sequential::build_display_list_for_subtree(
                            layout_root,
                            layout_context,
                            background_color,
                            data.page_clip_rect.size,
                        );

                        debug!("Done building display list.");

                        let root_size = {
                            let root_flow = layout_root.base();
                            if self.stylist.viewport_constraints().is_some() {
                                root_flow.position.size.to_physical(root_flow.writing_mode)
                            } else {
                                root_flow.overflow.scroll.size
                            }
                        };

                        let origin = Rect::new(Point2D::new(Au(0), Au(0)), root_size).to_layout();
                        build_state.root_stacking_context.bounds = origin;
                        build_state.root_stacking_context.overflow = origin;

                        if !build_state.iframe_sizes.is_empty() {
                            // build_state.iframe_sizes is only used here, so its okay to replace
                            // it with an empty vector
                            let iframe_sizes =
                                std::mem::replace(&mut build_state.iframe_sizes, vec![]);
                            // Collect the last frame's iframe sizes to compute any differences.
                            // Every frame starts with a fresh collection so that any removed
                            // iframes do not linger.
                            let last_iframe_sizes = std::mem::replace(
                                &mut *self.last_iframe_sizes.borrow_mut(),
                                HashMap::default(),
                            );
                            let mut size_messages = vec![];
                            for new_size in iframe_sizes {
                                // Only notify the constellation about existing iframes
                                // that have a new size, or iframes that did not previously
                                // exist.
                                if let Some(old_size) = last_iframe_sizes.get(&new_size.id) {
                                    if *old_size != new_size.size {
                                        size_messages.push(IFrameSizeMsg {
                                            data: new_size,
                                            type_: WindowSizeType::Resize,
                                        });
                                    }
                                } else {
                                    size_messages.push(IFrameSizeMsg {
                                        data: new_size,
                                        type_: WindowSizeType::Initial,
                                    });
                                }
                                self.last_iframe_sizes
                                    .borrow_mut()
                                    .insert(new_size.id, new_size.size);
                            }

                            if !size_messages.is_empty() {
                                let msg = ConstellationMsg::IFrameSizes(size_messages);
                                if let Err(e) = self.constellation_chan.send(msg) {
                                    warn!("Layout resize to constellation failed ({}).", e);
                                }
                            }
                        }

                        rw_data.indexable_text = std::mem::replace(
                            &mut build_state.indexable_text,
                            IndexableText::default(),
                        );
                        rw_data.display_list = Some(build_state.to_display_list());
                    }
                }

                if !reflow_goal.needs_display() {
                    // Defer the paint step until the next ForDisplay.
                    //
                    // We need to tell the document about this so it doesn't
                    // incorrectly suppress reflows. See #13131.
                    document
                        .expect("No document in a non-display reflow?")
                        .needs_paint_from_layout();
                    return;
                }
                if let Some(document) = document {
                    document.will_paint();
                }

                let display_list = rw_data.display_list.as_mut().unwrap();

                if self.dump_display_list {
                    display_list.print();
                }
                if self.dump_display_list_json {
                    println!("{}", serde_json::to_string_pretty(&display_list).unwrap());
                }

                debug!("Layout done!");

                // TODO: Avoid the temporary conversion and build webrender sc/dl directly!
                let (builder, compositor_info, is_contentful) =
                    display_list.convert_to_webrender(self.id);

                let viewport_size = Size2D::new(
                    self.viewport_size.width.to_f32_px(),
                    self.viewport_size.height.to_f32_px(),
                );

                let mut epoch = self.epoch.get();
                epoch.next();
                self.epoch.set(epoch);

                let viewport_size = webrender_api::units::LayoutSize::from_untyped(viewport_size);

                // Observe notifications about rendered frames if needed right before
                // sending the display list to WebRender in order to set time related
                // Progressive Web Metrics.
                self.paint_time_metrics
                    .maybe_observe_paint_time(self, epoch, is_contentful.0);

                self.webrender_api.send_display_list(
                    epoch,
                    viewport_size,
                    compositor_info,
                    builder.finalize(),
                );
            },
        );
    }

    /// The high-level routine that performs layout threads.
    fn handle_reflow<'a, 'b>(
        &mut self,
        data: &mut ScriptReflowResult,
        possibly_locked_rw_data: &mut RwData<'a, 'b>,
    ) {
        let document = unsafe { ServoLayoutNode::new(&data.document) };
        let document = document.as_document().unwrap();

        // Parallelize if there's more than 750 objects based on rzambre's suggestion
        // https://github.com/servo/servo/issues/10110
        self.parallel_flag = data.dom_count > 750;
        debug!("layout: received layout request for: {}", self.url);
        debug!("Number of objects in DOM: {}", data.dom_count);
        debug!("layout: parallel? {}", self.parallel_flag);

        let mut rw_data = possibly_locked_rw_data.lock();

        // Record the time that layout query has been waited.
        let now = time::precise_time_ns();
        if let ReflowGoal::LayoutQuery(_, timestamp) = data.reflow_goal {
            self.layout_query_waiting_time
                .increment(now - timestamp)
                .expect("layout: wrong layout query timestamp");
        };

        let root_element = match document.root_element() {
            None => {
                // Since we cannot compute anything, give spec-required placeholders.
                debug!("layout: No root node: bailing");
                match data.reflow_goal {
                    ReflowGoal::LayoutQuery(ref query_msg, _) => match query_msg {
                        &QueryMsg::ContentBoxQuery(_) => {
                            rw_data.content_box_response = None;
                        },
                        &QueryMsg::ContentBoxesQuery(_) => {
                            rw_data.content_boxes_response = Vec::new();
                        },
                        &QueryMsg::NodesFromPointQuery(..) => {
                            rw_data.nodes_from_point_response = Vec::new();
                        },
                        &QueryMsg::ClientRectQuery(_) => {
                            rw_data.client_rect_response = Rect::zero();
                        },
                        &QueryMsg::NodeScrollGeometryQuery(_) => {
                            rw_data.scroll_area_response = Rect::zero();
                        },
                        &QueryMsg::NodeScrollIdQuery(_) => {
                            rw_data.scroll_id_response = None;
                        },
                        &QueryMsg::ResolvedStyleQuery(_, _, _) => {
                            rw_data.resolved_style_response = String::new();
                        },
                        &QueryMsg::OffsetParentQuery(_) => {
                            rw_data.offset_parent_response = OffsetParentResponse::empty();
                        },
                        &QueryMsg::StyleQuery => {},
                        &QueryMsg::TextIndexQuery(..) => {
                            rw_data.text_index_response = TextIndexResponse(None);
                        },
                        &QueryMsg::ElementInnerTextQuery(_) => {
                            rw_data.element_inner_text_response = String::new();
                        },
                        &QueryMsg::ResolvedFontStyleQuery(..) => {
                            rw_data.resolved_font_style_response = None;
                        },
                        &QueryMsg::InnerWindowDimensionsQuery(_) => {
                            rw_data.inner_window_dimensions_response = None;
                        },
                    },
                    ReflowGoal::Full | ReflowGoal::TickAnimations => {},
                }
                return;
            },
            Some(x) => x,
        };

        debug!(
            "layout: processing reflow request for: {:?} ({}) (query={:?})",
            root_element, self.url, data.reflow_goal
        );
        trace!("{:?}", ShowSubtree(root_element.as_node()));

        let initial_viewport = data.window_size.initial_viewport;
        let device_pixel_ratio = data.window_size.device_pixel_ratio;
        let old_viewport_size = self.viewport_size;
        let current_screen_size = Size2D::new(
            Au::from_f32_px(initial_viewport.width),
            Au::from_f32_px(initial_viewport.height),
        );

        let origin = data.origin.clone();

        // Calculate the actual viewport as per DEVICE-ADAPT § 6
        // If the entire flow tree is invalid, then it will be reflowed anyhow.
        let document_shared_lock = document.style_shared_lock();
        let author_guard = document_shared_lock.read();

        let ua_stylesheets = &*UA_STYLESHEETS;
        let ua_or_user_guard = ua_stylesheets.shared_lock.read();
        let guards = StylesheetGuards {
            author: &author_guard,
            ua_or_user: &ua_or_user_guard,
        };

        let had_used_viewport_units = self.stylist.device().used_viewport_units();
        let device = Device::new(
            MediaType::screen(),
            self.stylist.quirks_mode(),
            initial_viewport,
            device_pixel_ratio,
        );
        let sheet_origins_affected_by_device_change = self.stylist.set_device(device, &guards);

        self.stylist
            .force_stylesheet_origins_dirty(sheet_origins_affected_by_device_change);
        self.viewport_size =
            self.stylist
                .viewport_constraints()
                .map_or(current_screen_size, |constraints| {
                    debug!("Viewport constraints: {:?}", constraints);

                    // other rules are evaluated against the actual viewport
                    Size2D::new(
                        Au::from_f32_px(constraints.size.width),
                        Au::from_f32_px(constraints.size.height),
                    )
                });

        let viewport_size_changed = self.viewport_size != old_viewport_size;
        if viewport_size_changed {
            if let Some(constraints) = self.stylist.viewport_constraints() {
                // let the constellation know about the viewport constraints
                rw_data
                    .constellation_chan
                    .send(ConstellationMsg::ViewportConstrained(
                        self.id,
                        constraints.clone(),
                    ))
                    .unwrap();
            }
            if had_used_viewport_units {
                if let Some(mut data) = root_element.mutate_data() {
                    data.hint.insert(RestyleHint::recascade_subtree());
                }
            }
        }

        {
            if self.first_reflow.get() {
                debug!("First reflow, rebuilding user and UA rules");
                for stylesheet in &ua_stylesheets.user_or_user_agent_stylesheets {
                    self.stylist
                        .append_stylesheet(stylesheet.clone(), &ua_or_user_guard);
                    self.handle_add_stylesheet(&stylesheet.0, &ua_or_user_guard);
                }

                if self.stylist.quirks_mode() != QuirksMode::NoQuirks {
                    self.stylist.append_stylesheet(
                        ua_stylesheets.quirks_mode_stylesheet.clone(),
                        &ua_or_user_guard,
                    );
                    self.handle_add_stylesheet(
                        &ua_stylesheets.quirks_mode_stylesheet.0,
                        &ua_or_user_guard,
                    );
                }
            }

            if data.stylesheets_changed {
                debug!("Doc sheets changed, flushing author sheets too");
                self.stylist
                    .force_stylesheet_origins_dirty(Origin::Author.into());
            }
        }

        if viewport_size_changed {
            if let Some(mut flow) = self.try_get_layout_root(root_element.as_node()) {
                LayoutThread::reflow_all_nodes(FlowRef::deref_mut(&mut flow));
            }
        }

        debug!(
            "Shadow roots in document {:?}",
            document.shadow_roots().len()
        );

        // Flush shadow roots stylesheets if dirty.
        document.flush_shadow_roots_stylesheets(
            &self.stylist.device(),
            document.quirks_mode(),
            guards.author.clone(),
        );

        let restyles = std::mem::take(&mut data.pending_restyles);
        debug!("Draining restyles: {}", restyles.len());

        let mut map = SnapshotMap::new();
        let elements_with_snapshot: Vec<_> = restyles
            .iter()
            .filter(|r| r.1.snapshot.is_some())
            .map(|r| unsafe { ServoLayoutNode::new(&r.0).as_element().unwrap() })
            .collect();

        for (el, restyle) in restyles {
            let el = unsafe { ServoLayoutNode::new(&el).as_element().unwrap() };

            // If we haven't styled this node yet, we don't need to track a
            // restyle.
            let mut style_data = match el.mutate_data() {
                Some(d) => d,
                None => {
                    unsafe { el.unset_snapshot_flags() };
                    continue;
                },
            };

            if let Some(s) = restyle.snapshot {
                unsafe { el.set_has_snapshot() };
                map.insert(el.as_node().opaque(), s);
            }

            // Stash the data on the element for processing by the style system.
            style_data.hint.insert(restyle.hint.into());
            style_data.damage = restyle.damage;
            debug!("Noting restyle for {:?}: {:?}", el, style_data);
        }

        self.stylist.flush(&guards, Some(root_element), Some(&map));

        // Create a layout context for use throughout the following passes.
        let mut layout_context = self.build_layout_context(
            guards.clone(),
            &map,
            origin,
            data.animation_timeline_value,
            &data.animations,
            data.stylesheets_changed,
        );

        let pool;
        let (thread_pool, num_threads) = if self.parallel_flag {
            pool = STYLE_THREAD_POOL.pool();
            (pool.as_ref(), STYLE_THREAD_POOL.num_threads.unwrap_or(1))
        } else {
            (None, 1)
        };

        let dirty_root = unsafe {
            ServoLayoutNode::new(&data.dirty_root.unwrap())
                .as_element()
                .unwrap()
        };

        let traversal = RecalcStyleAndConstructFlows::new(layout_context);
        let token = {
            let shared =
                <RecalcStyleAndConstructFlows as DomTraversal<ServoLayoutElement>>::shared_context(
                    &traversal,
                );
            RecalcStyleAndConstructFlows::pre_traverse(dirty_root, shared)
        };

        if token.should_traverse() {
            // Recalculate CSS styles and rebuild flows and fragments.
            profile(
                profile_time::ProfilerCategory::LayoutStyleRecalc,
                self.profiler_metadata(),
                self.time_profiler_chan.clone(),
                || {
                    // Perform CSS selector matching and flow construction.
                    let root = driver::traverse_dom::<
                        ServoLayoutElement,
                        RecalcStyleAndConstructFlows,
                    >(&traversal, token, thread_pool);
                    unsafe {
                        construct_flows_at_ancestors(traversal.context(), root.as_node());
                    }
                },
            );
            // TODO(pcwalton): Measure energy usage of text shaping, perhaps?
            let text_shaping_time =
                font::get_and_reset_text_shaping_performance_counter() / num_threads;
            profile_time::send_profile_data(
                profile_time::ProfilerCategory::LayoutTextShaping,
                self.profiler_metadata(),
                &self.time_profiler_chan,
                0,
                text_shaping_time as u64,
            );

            // Retrieve the (possibly rebuilt) root flow.
            *self.root_flow.borrow_mut() = self.try_get_layout_root(root_element.as_node());
        }

        for element in elements_with_snapshot {
            unsafe { element.unset_snapshot_flags() }
        }

        layout_context = traversal.destroy();

        if self.dump_style_tree {
            println!(
                "{:?}",
                ShowSubtreeDataAndPrimaryValues(root_element.as_node())
            );
        }

        if self.dump_rule_tree {
            layout_context
                .style_context
                .stylist
                .rule_tree()
                .dump_stdout(&guards);
        }

        // GC the rule tree if some heuristics are met.
        layout_context.style_context.stylist.rule_tree().maybe_gc();

        // Perform post-style recalculation layout passes.
        if let Some(mut root_flow) = self.root_flow.borrow().clone() {
            self.perform_post_style_recalc_layout_passes(
                &mut root_flow,
                &data.reflow_info,
                &data.reflow_goal,
                Some(&document),
                &mut rw_data,
                &mut layout_context,
            );
        }

        self.first_reflow.set(false);
        self.respond_to_query_if_necessary(
            &data.reflow_goal,
            &mut *rw_data,
            &mut layout_context,
            data.result.borrow_mut().as_mut().unwrap(),
            document_shared_lock,
        );
    }

    fn respond_to_query_if_necessary(
        &self,
        reflow_goal: &ReflowGoal,
        rw_data: &mut LayoutThreadData,
        context: &mut LayoutContext,
        reflow_result: &mut ReflowComplete,
        shared_lock: &SharedRwLock,
    ) {
        reflow_result.pending_images =
            std::mem::replace(&mut *context.pending_images.lock().unwrap(), vec![]);

        let mut root_flow = match self.root_flow.borrow().clone() {
            Some(root_flow) => root_flow,
            None => return,
        };
        let root_flow = FlowRef::deref_mut(&mut root_flow);
        match *reflow_goal {
            ReflowGoal::LayoutQuery(ref querymsg, _) => match querymsg {
                &QueryMsg::ContentBoxQuery(node) => {
                    rw_data.content_box_response = process_content_box_request(node, root_flow);
                },
                &QueryMsg::ContentBoxesQuery(node) => {
                    rw_data.content_boxes_response = process_content_boxes_request(node, root_flow);
                },
                &QueryMsg::TextIndexQuery(node, point_in_node) => {
                    let point_in_node = Point2D::new(
                        Au::from_f32_px(point_in_node.x),
                        Au::from_f32_px(point_in_node.y),
                    );
                    rw_data.text_index_response =
                        TextIndexResponse(rw_data.indexable_text.text_index(node, point_in_node));
                },
                &QueryMsg::ClientRectQuery(node) => {
                    rw_data.client_rect_response = process_client_rect_query(node, root_flow);
                },
                &QueryMsg::NodeScrollGeometryQuery(node) => {
                    rw_data.scroll_area_response =
                        process_node_scroll_area_request(node, root_flow);
                },
                &QueryMsg::NodeScrollIdQuery(node) => {
                    let node = unsafe { ServoLayoutNode::new(&node) };
                    rw_data.scroll_id_response =
                        Some(process_node_scroll_id_request(self.id, node));
                },
                &QueryMsg::ResolvedStyleQuery(node, ref pseudo, ref property) => {
                    let node = unsafe { ServoLayoutNode::new(&node) };
                    rw_data.resolved_style_response =
                        process_resolved_style_request(context, node, pseudo, property, root_flow);
                },
                &QueryMsg::ResolvedFontStyleQuery(node, ref property, ref value) => {
                    let node = unsafe { ServoLayoutNode::new(&node) };
                    let url = self.url.clone();
                    rw_data.resolved_font_style_response = process_resolved_font_style_request(
                        context,
                        node,
                        value,
                        property,
                        url,
                        shared_lock,
                    );
                },
                &QueryMsg::OffsetParentQuery(node) => {
                    rw_data.offset_parent_response = process_offset_parent_query(node, root_flow);
                },
                &QueryMsg::StyleQuery => {},
                &QueryMsg::NodesFromPointQuery(client_point, ref reflow_goal) => {
                    let mut flags = match reflow_goal {
                        &NodesFromPointQueryType::Topmost => webrender_api::HitTestFlags::empty(),
                        &NodesFromPointQueryType::All => webrender_api::HitTestFlags::FIND_ALL,
                    };

                    // The point we get is not relative to the entire WebRender scene, but to this
                    // particular pipeline, so we need to tell WebRender about that.
                    flags.insert(webrender_api::HitTestFlags::POINT_RELATIVE_TO_PIPELINE_VIEWPORT);

                    let client_point = webrender_api::units::WorldPoint::from_untyped(client_point);
                    let results = self.webrender_api.hit_test(
                        Some(self.id.to_webrender()),
                        client_point,
                        flags,
                    );

                    rw_data.nodes_from_point_response =
                        results.iter().map(|result| result.node).collect()
                },
                &QueryMsg::ElementInnerTextQuery(node) => {
                    let node = unsafe { ServoLayoutNode::new(&node) };
                    rw_data.element_inner_text_response =
                        process_element_inner_text_query(node, &rw_data.indexable_text);
                },
                &QueryMsg::InnerWindowDimensionsQuery(browsing_context_id) => {
                    rw_data.inner_window_dimensions_response = self
                        .last_iframe_sizes
                        .borrow()
                        .get(&browsing_context_id)
                        .cloned();
                },
            },
            ReflowGoal::Full | ReflowGoal::TickAnimations => {},
        }
    }

    fn set_scroll_states<'a, 'b>(
        &mut self,
        new_scroll_states: Vec<ScrollState>,
        possibly_locked_rw_data: &mut RwData<'a, 'b>,
    ) {
        let mut rw_data = possibly_locked_rw_data.lock();
        let mut script_scroll_states = vec![];
        let mut layout_scroll_states = HashMap::new();
        for new_state in &new_scroll_states {
            let offset = new_state.scroll_offset;
            layout_scroll_states.insert(new_state.scroll_id, offset);

            if new_state.scroll_id.is_root() {
                script_scroll_states.push((UntrustedNodeAddress::from_id(0), offset))
            } else if let Some(node_id) = node_id_from_scroll_id(new_state.scroll_id.0 as usize) {
                script_scroll_states.push((UntrustedNodeAddress::from_id(node_id), offset))
            }
        }
        let _ = self
            .script_chan
            .send(ConstellationControlMsg::SetScrollState(
                self.id,
                script_scroll_states,
            ));
        rw_data.scroll_offsets = layout_scroll_states
    }

    /// Cancel animations for any nodes which have been removed from flow tree.
    /// TODO(mrobinson): We should look into a way of doing this during flow tree construction.
    /// This also doesn't yet handles nodes that have been reparented.
    fn cancel_animations_for_nodes_not_in_flow_tree(
        animations: &mut FxHashMap<AnimationSetKey, ElementAnimationSet>,
        root_flow: &mut dyn Flow,
    ) {
        // Assume all nodes have been removed until proven otherwise.
        let mut invalid_nodes = animations.keys().cloned().collect();

        fn traverse_flow(flow: &mut dyn Flow, invalid_nodes: &mut FxHashSet<AnimationSetKey>) {
            flow.mutate_fragments(&mut |fragment| {
                // Ideally we'd only not cancel ::before and ::after animations if they
                // were actually in the tree. At this point layout has lost information
                // about whether or not they exist, but have had their fragments accumulated
                // together.
                invalid_nodes.remove(&AnimationSetKey::new_for_non_pseudo(fragment.node));
                invalid_nodes.remove(&AnimationSetKey::new_for_pseudo(
                    fragment.node,
                    PseudoElement::Before,
                ));
                invalid_nodes.remove(&AnimationSetKey::new_for_pseudo(
                    fragment.node,
                    PseudoElement::After,
                ));
            });
            for kid in flow.mut_base().children.iter_mut() {
                traverse_flow(kid, invalid_nodes)
            }
        }

        traverse_flow(root_flow, &mut invalid_nodes);

        // Cancel animations for any nodes that are no longer in the flow tree.
        for node in &invalid_nodes {
            if let Some(state) = animations.get_mut(node) {
                state.cancel_all_animations();
            }
        }
    }

    fn perform_post_style_recalc_layout_passes(
        &self,
        root_flow: &mut FlowRef,
        data: &Reflow,
        reflow_goal: &ReflowGoal,
        document: Option<&ServoLayoutDocument>,
        rw_data: &mut LayoutThreadData,
        context: &mut LayoutContext,
    ) {
        Self::cancel_animations_for_nodes_not_in_flow_tree(
            &mut *(context.style_context.animations.sets.write()),
            FlowRef::deref_mut(root_flow),
        );

        profile(
            profile_time::ProfilerCategory::LayoutRestyleDamagePropagation,
            self.profiler_metadata(),
            self.time_profiler_chan.clone(),
            || {
                // Call `compute_layout_damage` even in non-incremental mode, because it sets flags
                // that are needed in both incremental and non-incremental traversals.
                let damage = FlowRef::deref_mut(root_flow).compute_layout_damage();

                if self.nonincremental_layout ||
                    damage.contains(SpecialRestyleDamage::REFLOW_ENTIRE_DOCUMENT)
                {
                    FlowRef::deref_mut(root_flow).reflow_entire_document()
                }
            },
        );

        if self.trace_layout {
            layout_debug::begin_trace(root_flow.clone());
        }

        // Resolve generated content.
        profile(
            profile_time::ProfilerCategory::LayoutGeneratedContent,
            self.profiler_metadata(),
            self.time_profiler_chan.clone(),
            || sequential::resolve_generated_content(FlowRef::deref_mut(root_flow), &context),
        );

        // Guess float placement.
        profile(
            profile_time::ProfilerCategory::LayoutFloatPlacementSpeculation,
            self.profiler_metadata(),
            self.time_profiler_chan.clone(),
            || sequential::guess_float_placement(FlowRef::deref_mut(root_flow)),
        );

        // Perform the primary layout passes over the flow tree to compute the locations of all
        // the boxes.
        if root_flow
            .base()
            .restyle_damage
            .intersects(ServoRestyleDamage::REFLOW | ServoRestyleDamage::REFLOW_OUT_OF_FLOW)
        {
            profile(
                profile_time::ProfilerCategory::LayoutMain,
                self.profiler_metadata(),
                self.time_profiler_chan.clone(),
                || {
                    let profiler_metadata = self.profiler_metadata();

                    let pool;
                    let thread_pool = if self.parallel_flag {
                        pool = STYLE_THREAD_POOL.pool();
                        pool.as_ref()
                    } else {
                        None
                    };

                    if let Some(pool) = thread_pool {
                        // Parallel mode.
                        LayoutThread::solve_constraints_parallel(
                            pool,
                            FlowRef::deref_mut(root_flow),
                            profiler_metadata,
                            self.time_profiler_chan.clone(),
                            &*context,
                        );
                    } else {
                        //Sequential mode
                        LayoutThread::solve_constraints(FlowRef::deref_mut(root_flow), &context)
                    }
                },
            );
        }

        profile(
            profile_time::ProfilerCategory::LayoutStoreOverflow,
            self.profiler_metadata(),
            self.time_profiler_chan.clone(),
            || {
                sequential::store_overflow(context, FlowRef::deref_mut(root_flow) as &mut dyn Flow);
            },
        );

        self.perform_post_main_layout_passes(
            data,
            root_flow,
            reflow_goal,
            document,
            rw_data,
            context,
        );
    }

    fn perform_post_main_layout_passes(
        &self,
        data: &Reflow,
        mut root_flow: &mut FlowRef,
        reflow_goal: &ReflowGoal,
        document: Option<&ServoLayoutDocument>,
        rw_data: &mut LayoutThreadData,
        layout_context: &mut LayoutContext,
    ) {
        // Build the display list if necessary, and send it to the painter.
        self.compute_abs_pos_and_build_display_list(
            data,
            reflow_goal,
            document,
            FlowRef::deref_mut(&mut root_flow),
            &mut *layout_context,
            rw_data,
        );

        if self.trace_layout {
            layout_debug::end_trace(self.generation.get());
        }

        if self.dump_flow_tree {
            root_flow.print("Post layout flow tree".to_owned());
        }

        self.generation.set(self.generation.get() + 1);
    }

    fn reflow_all_nodes(flow: &mut dyn Flow) {
        debug!("reflowing all nodes!");
        flow.mut_base().restyle_damage.insert(
            ServoRestyleDamage::REPAINT |
                ServoRestyleDamage::STORE_OVERFLOW |
                ServoRestyleDamage::REFLOW |
                ServoRestyleDamage::REPOSITION,
        );

        for child in flow.mut_base().child_iter_mut() {
            LayoutThread::reflow_all_nodes(child);
        }
    }

    /// Returns profiling information which is passed to the time profiler.
    fn profiler_metadata(&self) -> Option<TimerMetadata> {
        Some(TimerMetadata {
            url: self.url.to_string(),
            iframe: if self.is_iframe {
                TimerMetadataFrameType::IFrame
            } else {
                TimerMetadataFrameType::RootWindow
            },
            incremental: if self.first_reflow.get() {
                TimerMetadataReflowType::FirstReflow
            } else {
                TimerMetadataReflowType::Incremental
            },
        })
    }
}

impl ProfilerMetadataFactory for LayoutThread {
    fn new_metadata(&self) -> Option<TimerMetadata> {
        self.profiler_metadata()
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
fn get_root_flow_background_color(flow: &mut dyn Flow) -> webrender_api::ColorF {
    let transparent = webrender_api::ColorF {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    if !flow.is_block_like() {
        return transparent;
    }

    let block_flow = flow.as_mut_block();
    let kid = match block_flow.base.children.iter_mut().next() {
        None => return transparent,
        Some(kid) => kid,
    };
    if !kid.is_block_like() {
        return transparent;
    }

    let kid_block_flow = kid.as_block();
    let color = kid_block_flow.fragment.style.resolve_color(
        kid_block_flow
            .fragment
            .style
            .get_background()
            .background_color,
    );
    webrender_api::ColorF::new(
        color.red_f32(),
        color.green_f32(),
        color.blue_f32(),
        color.alpha_f32(),
    )
}

fn get_ua_stylesheets() -> Result<UserAgentStylesheets, &'static str> {
    fn parse_ua_stylesheet(
        shared_lock: &SharedRwLock,
        filename: &str,
        content: &[u8],
    ) -> Result<DocumentStyleSheet, &'static str> {
        Ok(DocumentStyleSheet(ServoArc::new(Stylesheet::from_bytes(
            content,
            ServoUrl::parse(&format!("chrome://resources/{:?}", filename)).unwrap(),
            None,
            None,
            Origin::UserAgent,
            MediaList::empty(),
            shared_lock.clone(),
            None,
            None,
            QuirksMode::NoQuirks,
        ))))
    }

    let shared_lock = &GLOBAL_STYLE_DATA.shared_lock;

    // FIXME: presentational-hints.css should be at author origin with zero specificity.
    //        (Does it make a difference?)
    let mut user_or_user_agent_stylesheets = vec![
        parse_ua_stylesheet(
            &shared_lock,
            "user-agent.css",
            &resources::read_bytes(Resource::UserAgentCSS),
        )?,
        parse_ua_stylesheet(
            &shared_lock,
            "servo.css",
            &resources::read_bytes(Resource::ServoCSS),
        )?,
        parse_ua_stylesheet(
            &shared_lock,
            "presentational-hints.css",
            &resources::read_bytes(Resource::PresentationalHintsCSS),
        )?,
    ];

    for &(ref contents, ref url) in &opts::get().user_stylesheets {
        user_or_user_agent_stylesheets.push(DocumentStyleSheet(ServoArc::new(
            Stylesheet::from_bytes(
                &contents,
                url.clone(),
                None,
                None,
                Origin::User,
                MediaList::empty(),
                shared_lock.clone(),
                None,
                Some(&RustLogReporter),
                QuirksMode::NoQuirks,
            ),
        )));
    }

    let quirks_mode_stylesheet = parse_ua_stylesheet(
        &shared_lock,
        "quirks-mode.css",
        &resources::read_bytes(Resource::QuirksModeCSS),
    )?;

    Ok(UserAgentStylesheets {
        shared_lock: shared_lock.clone(),
        user_or_user_agent_stylesheets: user_or_user_agent_stylesheets,
        quirks_mode_stylesheet: quirks_mode_stylesheet,
    })
}

lazy_static! {
    static ref UA_STYLESHEETS: UserAgentStylesheets = {
        match get_ua_stylesheets() {
            Ok(stylesheets) => stylesheets,
            Err(filename) => {
                error!("Failed to load UA stylesheet {}!", filename);
                process::exit(1);
            },
        }
    };
}

struct RegisteredPainterImpl {
    painter: Box<dyn Painter>,
    name: Atom,
    // FIXME: Should be a PrecomputedHashMap.
    properties: FxHashMap<Atom, PropertyId>,
}

impl SpeculativePainter for RegisteredPainterImpl {
    fn speculatively_draw_a_paint_image(
        &self,
        properties: Vec<(Atom, String)>,
        arguments: Vec<String>,
    ) {
        self.painter
            .speculatively_draw_a_paint_image(properties, arguments);
    }
}

impl RegisteredSpeculativePainter for RegisteredPainterImpl {
    fn properties(&self) -> &FxHashMap<Atom, PropertyId> {
        &self.properties
    }
    fn name(&self) -> Atom {
        self.name.clone()
    }
}

impl Painter for RegisteredPainterImpl {
    fn draw_a_paint_image(
        &self,
        size: Size2D<f32, CSSPixel>,
        device_pixel_ratio: Scale<f32, CSSPixel, DevicePixel>,
        properties: Vec<(Atom, String)>,
        arguments: Vec<String>,
    ) -> Result<DrawAPaintImageResult, PaintWorkletError> {
        self.painter
            .draw_a_paint_image(size, device_pixel_ratio, properties, arguments)
    }
}

impl RegisteredPainter for RegisteredPainterImpl {}

struct RegisteredPaintersImpl(FnvHashMap<Atom, RegisteredPainterImpl>);

impl RegisteredSpeculativePainters for RegisteredPaintersImpl {
    fn get(&self, name: &Atom) -> Option<&dyn RegisteredSpeculativePainter> {
        self.0
            .get(&name)
            .map(|painter| painter as &dyn RegisteredSpeculativePainter)
    }
}

impl RegisteredPainters for RegisteredPaintersImpl {
    fn get(&self, name: &Atom) -> Option<&dyn RegisteredPainter> {
        self.0
            .get(&name)
            .map(|painter| painter as &dyn RegisteredPainter)
    }
}
