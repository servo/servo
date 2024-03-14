/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// Work around https://github.com/rust-lang/rust/issues/62132
#![recursion_limit = "128"]

//! Layout. Performs layout on the DOM, builds display lists and sends them to be
//! painted.

use std::borrow::ToOwned;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::process;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use app_units::Au;
use embedder_traits::resources::{self, Resource};
use euclid::default::Size2D as UntypedSize2D;
use euclid::{Point2D, Rect, Scale, Size2D};
use fnv::FnvHashMap;
use fxhash::{FxHashMap, FxHashSet};
use gfx::font_cache_thread::FontCacheThread;
use gfx::{font, font_context};
use gfx_traits::{node_id_from_scroll_id, Epoch};
use histogram::Histogram;
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use layout::construct::ConstructionResult;
use layout::context::{
    malloc_size_of_persistent_local_context, LayoutContext, RegisteredPainter, RegisteredPainters,
};
use layout::display_list::items::WebRenderImageInfo;
use layout::display_list::{IndexableText, ToLayout};
use layout::flow::{Flow, FlowFlags, GetBaseFlow, ImmutableFlowUtils, MutableOwnedFlowUtils};
use layout::flow_ref::FlowRef;
use layout::incremental::{RelayoutMode, SpecialRestyleDamage};
use layout::query::{
    process_client_rect_query, process_content_box_request, process_content_boxes_request,
    process_element_inner_text_query, process_node_scroll_id_request, process_offset_parent_query,
    process_resolved_font_style_request, process_resolved_style_request,
    process_scrolling_area_request, LayoutRPCImpl, LayoutThreadData,
};
use layout::traversal::{
    construct_flows_at_ancestors, ComputeStackingRelativePositions, PreorderFlowTraversal,
    RecalcStyleAndConstructFlows,
};
use layout::wrapper::LayoutNodeLayoutData;
use layout::{layout_debug, layout_debug_scope, parallel, sequential, LayoutData};
use lazy_static::lazy_static;
use log::{debug, error, trace, warn};
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use metrics::{PaintTimeMetrics, ProfilerMetadataFactory};
use msg::constellation_msg::{BrowsingContextId, PipelineId};
use net_traits::image_cache::{ImageCache, UsePlaceholder};
use parking_lot::RwLock;
use profile_traits::mem::{Report, ReportKind, ReportsChan};
use profile_traits::path;
use profile_traits::time::{
    self as profile_time, profile, TimerMetadata, TimerMetadataFrameType, TimerMetadataReflowType,
};
use script::layout_dom::{ServoLayoutDocument, ServoLayoutElement, ServoLayoutNode};
use script_layout_interface::message::{
    Msg, NodesFromPointQueryType, QueryMsg, Reflow, ReflowComplete, ReflowGoal, ScriptReflow,
};
use script_layout_interface::rpc::{LayoutRPC, OffsetParentResponse, TextIndexResponse};
use script_layout_interface::wrapper_traits::LayoutNode;
use script_layout_interface::{Layout, LayoutConfig, LayoutFactory};
use script_traits::{
    ConstellationControlMsg, DrawAPaintImageResult, IFrameSizeMsg, LayoutControlMsg,
    LayoutMsg as ConstellationMsg, PaintWorkletError, Painter, ScrollState, UntrustedNodeAddress,
    WebrenderIpcSender, WindowSizeData, WindowSizeType,
};
use servo_arc::Arc as ServoArc;
use servo_atoms::Atom;
use servo_config::opts::{self, DebugOptions};
use servo_url::{ImmutableOrigin, ServoUrl};
use style::animation::{AnimationSetKey, DocumentAnimationSet, ElementAnimationSet};
use style::context::{
    QuirksMode, RegisteredSpeculativePainter, RegisteredSpeculativePainters, SharedStyleContext,
};
use style::dom::{ShowSubtree, ShowSubtreeDataAndPrimaryValues, TElement, TNode};
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
    DocumentStyleSheet, Origin, Stylesheet, StylesheetInDocument, UrlExtraData,
    UserAgentStylesheets,
};
use style::stylist::Stylist;
use style::traversal::DomTraversal;
use style::traversal_flags::TraversalFlags;
use style_traits::{CSSPixel, DevicePixel, SpeculativePainter};
use url::Url;
use webrender_api::{units, ColorF, HitTestFlags};

/// Information needed by layout.
pub struct LayoutThread {
    /// The ID of the pipeline that we belong to.
    id: PipelineId,

    /// The URL of the pipeline that we belong to.
    url: ServoUrl,

    /// Performs CSS selector matching and style resolution.
    stylist: Stylist,

    /// Is the current reflow of an iframe, as opposed to a root window?
    is_iframe: bool,

    /// The channel on which the font cache can send messages to us.
    font_cache_sender: IpcSender<()>,

    /// The channel on which messages can be sent to the constellation.
    constellation_chan: IpcSender<ConstellationMsg>,

    /// The channel on which messages can be sent to the script thread.
    script_chan: IpcSender<ConstellationControlMsg>,

    /// The channel on which messages can be sent to the time profiler.
    time_profiler_chan: profile_time::ProfilerChan,

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

    /// The time a layout query has waited before serviced by layout.
    layout_query_waiting_time: Histogram,

    /// The sizes of all iframes encountered during the last layout operation.
    last_iframe_sizes: RefCell<FnvHashMap<BrowsingContextId, Size2D<f32, CSSPixel>>>,

    /// Debug options, copied from configuration to this `LayoutThread` in order
    /// to avoid having to constantly access the thread-safe global options.
    debug: DebugOptions,

    /// True to turn off incremental layout.
    nonincremental_layout: bool,
}

pub struct LayoutFactoryImpl();

impl LayoutFactory for LayoutFactoryImpl {
    fn create(&self, config: LayoutConfig) -> Box<dyn Layout> {
        Box::new(LayoutThread::new(
            config.id,
            config.url,
            config.is_iframe,
            config.constellation_chan,
            config.script_chan,
            config.image_cache,
            config.font_cache_thread,
            config.time_profiler_chan,
            config.webrender_api_sender,
            config.paint_time_metrics,
            config.window_size,
        ))
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

impl Layout for LayoutThread {
    fn process(&mut self, msg: script_layout_interface::message::Msg) {
        self.handle_request(Request::FromScript(msg));
    }

    fn handle_constellation_msg(&mut self, msg: script_traits::LayoutControlMsg) {
        self.handle_request(Request::FromPipeline(msg));
    }

    fn handle_font_cache_msg(&mut self) {
        self.handle_request(Request::FromFontCache);
    }

    fn rpc(&self) -> Box<dyn script_layout_interface::rpc::LayoutRPC> {
        Box::new(LayoutRPCImpl(self.rw_data.clone())) as Box<dyn LayoutRPC>
    }

    fn waiting_for_web_fonts_to_load(&self) -> bool {
        self.outstanding_web_fonts.load(Ordering::SeqCst) != 0
    }

    fn current_epoch(&self) -> Epoch {
        self.epoch.get()
    }
}
enum Request {
    FromPipeline(LayoutControlMsg),
    FromScript(Msg),
    FromFontCache,
}

impl LayoutThread {
    fn new(
        id: PipelineId,
        url: ServoUrl,
        is_iframe: bool,
        constellation_chan: IpcSender<ConstellationMsg>,
        script_chan: IpcSender<ConstellationControlMsg>,
        image_cache: Arc<dyn ImageCache>,
        font_cache_thread: FontCacheThread,
        time_profiler_chan: profile_time::ProfilerChan,
        webrender_api: WebrenderIpcSender,
        paint_time_metrics: PaintTimeMetrics,
        window_size: WindowSizeData,
    ) -> LayoutThread {
        // Let webrender know about this pipeline by sending an empty display list.
        webrender_api.send_initial_transaction(id.to_webrender());

        let device = Device::new(
            MediaType::screen(),
            QuirksMode::NoQuirks,
            window_size.initial_viewport,
            window_size.device_pixel_ratio,
        );

        // Ask the router to proxy IPC messages from the font cache thread to layout.
        let (ipc_font_cache_sender, ipc_font_cache_receiver) = ipc::channel().unwrap();
        let cloned_script_chan = script_chan.clone();
        ROUTER.add_route(
            ipc_font_cache_receiver.to_opaque(),
            Box::new(move |_message| {
                let _ =
                    cloned_script_chan.send(ConstellationControlMsg::ForLayoutFromFontCache(id));
            }),
        );

        LayoutThread {
            id,
            url,
            is_iframe,
            script_chan,
            constellation_chan: constellation_chan.clone(),
            time_profiler_chan,
            registered_painters: RegisteredPaintersImpl(Default::default()),
            image_cache,
            font_cache_thread,
            first_reflow: Cell::new(true),
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
                constellation_chan,
                display_list: None,
                indexable_text: IndexableText::default(),
                content_box_response: None,
                content_boxes_response: Vec::new(),
                client_rect_response: Rect::zero(),
                scroll_id_response: None,
                scrolling_area_response: Rect::zero(),
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
            paint_time_metrics,
            layout_query_waiting_time: Histogram::new(),
            last_iframe_sizes: Default::default(),
            debug: opts::get().debug.clone(),
            nonincremental_layout: opts::get().nonincremental_layout,
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

    /// Receives and dispatches messages from the script and constellation threads
    fn handle_request<'a, 'b>(&mut self, request: Request) {
        let rw_data = self.rw_data.clone();
        let mut possibly_locked_rw_data = Some(rw_data.lock().unwrap());
        let mut rw_data = RwData {
            rw_data: &rw_data,
            possibly_locked_rw_data: &mut possibly_locked_rw_data,
        };

        match request {
            Request::FromPipeline(LayoutControlMsg::SetScrollStates(new_scroll_states)) => {
                self.handle_request_helper(Msg::SetScrollStates(new_scroll_states), &mut rw_data)
            },
            Request::FromPipeline(LayoutControlMsg::ExitNow) => {
                self.handle_request_helper(Msg::ExitNow, &mut rw_data);
            },
            Request::FromPipeline(LayoutControlMsg::PaintMetric(epoch, paint_time)) => {
                self.paint_time_metrics.maybe_set_metric(epoch, paint_time);
            },
            Request::FromScript(msg) => self.handle_request_helper(msg, &mut rw_data),
            Request::FromFontCache => {
                let _rw_data = rw_data.lock();
                self.outstanding_web_fonts.fetch_sub(1, Ordering::SeqCst);
                font_context::invalidate_font_caches();
                self.script_chan
                    .send(ConstellationControlMsg::WebFontLoaded(self.id))
                    .unwrap();
            },
        };
    }

    /// Receives and dispatches messages from other threads.
    fn handle_request_helper<'a, 'b>(
        &mut self,
        request: Msg,
        possibly_locked_rw_data: &mut RwData<'a, 'b>,
    ) {
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
            Msg::CollectReports(reports_chan) => {
                self.collect_reports(reports_chan, possibly_locked_rw_data);
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
            // Receiving the Exit message at this stage only happens when layout is undergoing a "force exit".
            Msg::ExitNow => {
                debug!("layout: ExitNow received");
                self.exit_now();
            },
        }
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

    /// Shuts down layout now.
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
    }

    fn handle_add_stylesheet(&self, stylesheet: &Stylesheet, guard: &SharedRwLockReadGuard) {
        // Find all font-face rules and notify the font cache of them.
        // GWTODO: Need to handle unloading web fonts.
        if stylesheet.is_effective_for_device(self.stylist.device(), &guard) {
            let newly_loading_font_count =
                self.font_cache_thread.add_all_web_fonts_from_stylesheet(
                    &*stylesheet,
                    &guard,
                    self.stylist.device(),
                    &self.font_cache_sender,
                    self.debug.load_webfonts_synchronously,
                );
            self.outstanding_web_fonts
                .fetch_add(newly_loading_font_count, Ordering::SeqCst);
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

    /// Update the recorded iframe sizes of the contents of layout and when these sizes changes,
    /// send a message to the constellation informing it of the new sizes.
    fn update_iframe_sizes(
        &self,
        new_iframe_sizes: FnvHashMap<BrowsingContextId, Size2D<f32, CSSPixel>>,
    ) {
        let old_iframe_sizes =
            std::mem::replace(&mut *self.last_iframe_sizes.borrow_mut(), new_iframe_sizes);

        if self.last_iframe_sizes.borrow().is_empty() {
            return;
        }

        let size_messages: Vec<_> = self
            .last_iframe_sizes
            .borrow()
            .iter()
            .filter_map(|(browsing_context_id, size)| {
                match old_iframe_sizes.get(&browsing_context_id) {
                    Some(old_size) if old_size != size => Some(IFrameSizeMsg {
                        browsing_context_id: *browsing_context_id,
                        size: *size,
                        type_: WindowSizeType::Resize,
                    }),
                    None => Some(IFrameSizeMsg {
                        browsing_context_id: *browsing_context_id,
                        size: *size,
                        type_: WindowSizeType::Initial,
                    }),
                    _ => None,
                }
            })
            .collect();

        if !size_messages.is_empty() {
            let msg = ConstellationMsg::IFrameSizes(size_messages);
            if let Err(e) = self.constellation_chan.send(msg) {
                warn!("Layout resize to constellation failed ({}).", e);
            }
        }
    }

    /// Computes the stacking-relative positions of all flows and, if the painting is dirty and the
    /// reflow type need it, builds the display list.
    fn compute_abs_pos_and_build_display_list(
        &self,
        data: &Reflow,
        reflow_goal: &ReflowGoal,
        document: Option<&ServoLayoutDocument<LayoutData>>,
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
                            root_flow.overflow.scroll.size
                        };

                        let origin = Rect::new(Point2D::new(Au(0), Au(0)), root_size).to_layout();
                        build_state.root_stacking_context.bounds = origin;
                        build_state.root_stacking_context.overflow = origin;

                        // We will not use build_state.iframe_sizes again, so it's safe to move it.
                        let iframe_sizes =
                            std::mem::replace(&mut build_state.iframe_sizes, FnvHashMap::default());
                        self.update_iframe_sizes(iframe_sizes);

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

                if self.debug.dump_display_list {
                    display_list.print();
                }
                if self.debug.dump_display_list_json {
                    println!("{}", serde_json::to_string_pretty(&display_list).unwrap());
                }

                debug!("Layout done!");

                let viewport_size = units::LayoutSize::new(
                    self.viewport_size.width.to_f32_px(),
                    self.viewport_size.height.to_f32_px(),
                );

                let mut epoch = self.epoch.get();
                epoch.next();
                self.epoch.set(epoch);

                // TODO: Avoid the temporary conversion and build webrender sc/dl directly!
                let (mut builder, compositor_info, is_contentful) =
                    display_list.convert_to_webrender(self.id, viewport_size, epoch.into());

                // Observe notifications about rendered frames if needed right before
                // sending the display list to WebRender in order to set time related
                // Progressive Web Metrics.
                self.paint_time_metrics
                    .maybe_observe_paint_time(self, epoch, is_contentful.0);

                self.webrender_api
                    .send_display_list(compositor_info, builder.end().1);
            },
        );
    }

    /// The high-level routine that performs layout.
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
                        &QueryMsg::ScrollingAreaQuery(_) => {
                            rw_data.scrolling_area_response = Rect::zero();
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
                    ReflowGoal::Full |
                    ReflowGoal::TickAnimations |
                    ReflowGoal::UpdateScrollNode(_) => {},
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
        self.viewport_size = current_screen_size;

        let viewport_size_changed = self.viewport_size != old_viewport_size;
        if viewport_size_changed && had_used_viewport_units {
            if let Some(mut data) = root_element.mutate_data() {
                data.hint.insert(RestyleHint::recascade_subtree());
            }
        }

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
        document.flush_shadow_roots_stylesheets(&mut self.stylist, guards.author);

        let restyles = std::mem::take(&mut data.pending_restyles);
        debug!("Draining restyles: {}", restyles.len());

        let mut map = SnapshotMap::new();
        let elements_with_snapshot: Vec<_> = restyles
            .iter()
            .filter(|r| r.1.snapshot.is_some())
            .map(|r| unsafe {
                ServoLayoutNode::<LayoutData>::new(&r.0)
                    .as_element()
                    .unwrap()
            })
            .collect();

        for (el, restyle) in restyles {
            let el: ServoLayoutElement<LayoutData> =
                unsafe { ServoLayoutNode::new(&el).as_element().unwrap() };

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

        let pool = STYLE_THREAD_POOL.lock().unwrap();
        let thread_pool = pool.pool();
        let (thread_pool, num_threads) = if self.parallel_flag {
            (thread_pool.as_ref(), pool.num_threads.unwrap_or(1))
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
            let shared = <RecalcStyleAndConstructFlows as DomTraversal<
                ServoLayoutElement<LayoutData>,
            >>::shared_context(&traversal);
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
                        ServoLayoutElement<LayoutData>,
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

        if self.debug.dump_style_tree {
            println!(
                "{:?}",
                ShowSubtreeDataAndPrimaryValues(root_element.as_node())
            );
        }

        if self.debug.dump_rule_tree {
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
                thread_pool,
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
                &QueryMsg::ScrollingAreaQuery(node) => {
                    rw_data.scrolling_area_response =
                        process_scrolling_area_request(node, root_flow);
                },
                &QueryMsg::NodeScrollIdQuery(node) => {
                    let node: ServoLayoutNode<LayoutData> = unsafe { ServoLayoutNode::new(&node) };
                    rw_data.scroll_id_response =
                        Some(process_node_scroll_id_request(self.id, node));
                },
                &QueryMsg::ResolvedStyleQuery(node, ref pseudo, ref property) => {
                    let node: ServoLayoutNode<LayoutData> = unsafe { ServoLayoutNode::new(&node) };
                    rw_data.resolved_style_response =
                        process_resolved_style_request(context, node, pseudo, property, root_flow);
                },
                &QueryMsg::ResolvedFontStyleQuery(node, ref property, ref value) => {
                    let node: ServoLayoutNode<LayoutData> = unsafe { ServoLayoutNode::new(&node) };
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
                        &NodesFromPointQueryType::Topmost => HitTestFlags::empty(),
                        &NodesFromPointQueryType::All => HitTestFlags::FIND_ALL,
                    };

                    // The point we get is not relative to the entire WebRender scene, but to this
                    // particular pipeline, so we need to tell WebRender about that.
                    flags.insert(HitTestFlags::POINT_RELATIVE_TO_PIPELINE_VIEWPORT);

                    let client_point = units::DevicePoint::from_untyped(client_point);
                    let results = self.webrender_api.hit_test(
                        Some(self.id.to_webrender()),
                        client_point,
                        flags,
                    );

                    rw_data.nodes_from_point_response =
                        results.iter().map(|result| result.node).collect()
                },
                &QueryMsg::ElementInnerTextQuery(node) => {
                    let node: ServoLayoutNode<LayoutData> = unsafe { ServoLayoutNode::new(&node) };
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
            ReflowGoal::UpdateScrollNode(scroll_state) => {
                self.update_scroll_node_state(&scroll_state, rw_data);
            },
            ReflowGoal::Full | ReflowGoal::TickAnimations => {},
        }
    }

    fn update_scroll_node_state(&self, state: &ScrollState, rw_data: &mut LayoutThreadData) {
        rw_data
            .scroll_offsets
            .insert(state.scroll_id, state.scroll_offset);

        let point = Point2D::new(-state.scroll_offset.x, -state.scroll_offset.y);
        self.webrender_api.send_scroll_node(
            self.id.to_webrender(),
            units::LayoutPoint::from_untyped(point),
            state.scroll_id,
        );
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
        document: Option<&ServoLayoutDocument<LayoutData>>,
        rw_data: &mut LayoutThreadData,
        context: &mut LayoutContext,
        thread_pool: Option<&rayon::ThreadPool>,
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

        if self.debug.trace_layout {
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
        document: Option<&ServoLayoutDocument<LayoutData>>,
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

        if self.debug.trace_layout {
            layout_debug::end_trace(self.generation.get());
        }

        if self.debug.dump_flow_tree {
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
fn get_root_flow_background_color(flow: &mut dyn Flow) -> ColorF {
    let transparent = ColorF {
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
            .background_color
            .clone(),
    );
    color.to_layout()
}

fn get_ua_stylesheets() -> Result<UserAgentStylesheets, &'static str> {
    fn parse_ua_stylesheet(
        shared_lock: &SharedRwLock,
        filename: &str,
        content: &[u8],
    ) -> Result<DocumentStyleSheet, &'static str> {
        let url = Url::parse(&format!("chrome://resources/{:?}", filename))
            .ok()
            .unwrap();
        Ok(DocumentStyleSheet(ServoArc::new(Stylesheet::from_bytes(
            content,
            url.into(),
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
                UrlExtraData(url.get_arc()),
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
