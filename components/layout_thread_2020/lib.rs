/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// Work around https://github.com/rust-lang/rust/issues/62132
#![recursion_limit = "128"]

//! The layout thread. Performs layout on the DOM, builds display lists and sends them to be
//! painted.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;
use std::{process, thread};

use app_units::Au;
use crossbeam_channel::{select, Receiver, Sender};
use embedder_traits::resources::{self, Resource};
use euclid::default::Size2D as UntypedSize2D;
use euclid::{Point2D, Rect, Scale, Size2D};
use fnv::FnvHashMap;
use fxhash::FxHashMap;
use gfx::font_cache_thread::FontCacheThread;
use gfx::font_context;
use gfx_traits::{node_id_from_scroll_id, Epoch};
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use layout::context::LayoutContext;
use layout::display_list::{DisplayList, WebRenderImageInfo};
use layout::dom::DOMLayoutData;
use layout::query::{
    process_content_box_request, process_content_boxes_request, process_element_inner_text_query,
    process_node_geometry_request, process_node_scroll_area_request,
    process_node_scroll_id_request, process_offset_parent_query, process_resolved_font_style_query,
    process_resolved_style_request, process_text_index_request, LayoutRPCImpl, LayoutThreadData,
};
use layout::traversal::RecalcStyle;
use layout::{layout_debug, BoxTree, FragmentTree};
use layout_traits::LayoutThreadFactory;
use lazy_static::lazy_static;
use log::{debug, error, warn};
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use metrics::{PaintTimeMetrics, ProfilerMetadataFactory, ProgressiveWebMetric};
use msg::constellation_msg::{
    BackgroundHangMonitor, BackgroundHangMonitorRegister, BrowsingContextId, HangAnnotation,
    LayoutHangAnnotation, MonitoredComponentId, MonitoredComponentType, PipelineId,
    TopLevelBrowsingContextId,
};
use net_traits::image_cache::{ImageCache, UsePlaceholder};
use parking_lot::RwLock;
use profile_traits::mem::{self as profile_mem, Report, ReportKind, ReportsChan};
use profile_traits::path;
use profile_traits::time::{
    self as profile_time, profile, TimerMetadata, TimerMetadataFrameType, TimerMetadataReflowType,
};
use script::layout_dom::{ServoLayoutDocument, ServoLayoutElement, ServoLayoutNode};
use script_layout_interface::message::{
    LayoutThreadInit, Msg, NodesFromPointQueryType, QueryMsg, ReflowComplete, ReflowGoal,
    ScriptReflow,
};
use script_layout_interface::rpc::{LayoutRPC, OffsetParentResponse, TextIndexResponse};
use script_traits::{
    ConstellationControlMsg, DrawAPaintImageResult, IFrameSizeMsg, LayoutControlMsg,
    LayoutMsg as ConstellationMsg, PaintWorkletError, Painter, ScrollState, UntrustedNodeAddress,
    WebrenderIpcSender, WindowSizeData, WindowSizeType,
};
use servo_arc::Arc as ServoArc;
use servo_atoms::Atom;
use servo_config::opts::{self, DebugOptions};
use servo_url::{ImmutableOrigin, ServoUrl};
use style::animation::DocumentAnimationSet;
use style::context::{
    QuirksMode, RegisteredSpeculativePainter, RegisteredSpeculativePainters, SharedStyleContext,
};
use style::dom::{TElement, TNode};
use style::driver;
use style::error_reporting::RustLogReporter;
use style::global_style_data::{GLOBAL_STYLE_DATA, STYLE_THREAD_POOL};
use style::media_queries::{Device, MediaList, MediaType};
use style::properties::PropertyId;
use style::selector_parser::SnapshotMap;
use style::shared_lock::{SharedRwLock, SharedRwLockReadGuard, StylesheetGuards};
use style::stylesheets::{
    DocumentStyleSheet, Origin, Stylesheet, StylesheetInDocument, UserAgentStylesheets,
};
use style::stylist::Stylist;
use style::thread_state::{self, ThreadState};
use style::traversal::DomTraversal;
use style::traversal_flags::TraversalFlags;
use style_traits::{CSSPixel, DevicePixel, SpeculativePainter};
use webrender_api::{units, HitTestFlags};

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

    /// Starts at zero, and increased by one every time a layout completes.
    /// This can be used to easily check for invalid stale data.
    generation: Cell<u32>,

    /// The number of Web fonts that have been requested but not yet loaded.
    outstanding_web_fonts: Arc<AtomicUsize>,

    /// The box tree.
    box_tree: RefCell<Option<Arc<BoxTree>>>,

    /// The fragment tree.
    fragment_tree: RefCell<Option<Arc<FragmentTree>>>,

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

    /// The sizes of all iframes encountered during the last layout operation.
    last_iframe_sizes: RefCell<FnvHashMap<BrowsingContextId, Size2D<f32, CSSPixel>>>,

    /// Flag that indicates if LayoutThread is busy handling a request.
    busy: Arc<AtomicBool>,

    /// Debug options, copied from configuration to this `LayoutThread` in order
    /// to avoid having to constantly access the thread-safe global options.
    debug: DebugOptions,
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
        window_size: WindowSizeData,
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
                        window_size,
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
        webrender_api_sender: WebrenderIpcSender,
        paint_time_metrics: PaintTimeMetrics,
        busy: Arc<AtomicBool>,
        window_size: WindowSizeData,
    ) -> LayoutThread {
        // Let webrender know about this pipeline by sending an empty display list.
        webrender_api_sender.send_initial_transaction(id.to_webrender());

        // The device pixel ratio is incorrect (it does not have the hidpi value),
        // but it will be set correctly when the initial reflow takes place.
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
            id,
            top_level_browsing_context_id: top_level_browsing_context_id,
            url,
            is_iframe,
            port,
            pipeline_port: pipeline_receiver,
            constellation_chan,
            script_chan: script_chan.clone(),
            background_hang_monitor,
            time_profiler_chan,
            mem_profiler_chan,
            registered_painters: RegisteredPaintersImpl(Default::default()),
            image_cache,
            font_cache_thread,
            first_reflow: Cell::new(true),
            font_cache_receiver,
            font_cache_sender: ipc_font_cache_sender,
            generation: Cell::new(0),
            outstanding_web_fonts: Arc::new(AtomicUsize::new(0)),
            box_tree: Default::default(),
            fragment_tree: Default::default(),
            // Epoch starts at 1 because of the initial display list for epoch 0 that we send to WR
            epoch: Cell::new(Epoch(1)),
            viewport_size: Size2D::new(Au(0), Au(0)),
            webrender_api: webrender_api_sender,
            stylist: Stylist::new(device, QuirksMode::NoQuirks),
            rw_data: Arc::new(Mutex::new(LayoutThreadData {
                display_list: None,
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
            webrender_image_cache: Default::default(),
            paint_time_metrics: paint_time_metrics,
            last_iframe_sizes: Default::default(),
            busy,
            debug: opts::get().debug.clone(),
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
        use_rayon: bool,
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
            use_rayon,
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
            Msg::RegisterPaint(_name, _properties, _painter) => {},
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
            info.script_chan.clone(),
            info.image_cache.clone(),
            self.font_cache_thread.clone(),
            self.time_profiler_chan.clone(),
            self.mem_profiler_chan.clone(),
            self.webrender_api.clone(),
            info.paint_time_metrics,
            info.layout_is_busy,
            info.window_size,
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

    fn exit_now(&mut self) {
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
                self.debug.load_webfonts_synchronously,
            );
        }
    }

    /// Sets quirks mode for the document, causing the quirks mode stylesheet to be used.
    fn handle_set_quirks_mode<'a, 'b>(&mut self, quirks_mode: QuirksMode) {
        self.stylist.set_quirks_mode(quirks_mode);
    }

    /// The high-level routine that performs layout threads.
    fn handle_reflow<'a, 'b>(
        &mut self,
        data: &mut ScriptReflowResult,
        possibly_locked_rw_data: &mut RwData<'a, 'b>,
    ) {
        let document = unsafe { ServoLayoutNode::<DOMLayoutData>::new(&data.document) };
        let document = document.as_document().unwrap();

        let mut rw_data = possibly_locked_rw_data.lock();

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
                        &QueryMsg::ResolvedFontStyleQuery(_, _, _) => {
                            rw_data.resolved_font_style_response = None;
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
                        &QueryMsg::InnerWindowDimensionsQuery(browsing_context_id) => {
                            rw_data.inner_window_dimensions_response = self
                                .last_iframe_sizes
                                .borrow()
                                .get(&browsing_context_id)
                                .cloned();
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

        let initial_viewport = data.window_size.initial_viewport;
        let device_pixel_ratio = data.window_size.device_pixel_ratio;
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
        if self.first_reflow.get() {
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
            self.stylist
                .force_stylesheet_origins_dirty(Origin::Author.into());
        }

        // Flush shadow roots stylesheets if dirty.
        document.flush_shadow_roots_stylesheets(&mut self.stylist, guards.author.clone());

        let restyles = std::mem::take(&mut data.pending_restyles);
        debug!("Draining restyles: {}", restyles.len());

        let mut map = SnapshotMap::new();
        let elements_with_snapshot: Vec<_> = restyles
            .iter()
            .filter(|r| r.1.snapshot.is_some())
            .map(|r| unsafe {
                ServoLayoutNode::<DOMLayoutData>::new(&r.0)
                    .as_element()
                    .unwrap()
            })
            .collect();

        for (el, restyle) in restyles {
            let el = unsafe {
                ServoLayoutNode::<DOMLayoutData>::new(&el)
                    .as_element()
                    .unwrap()
            };

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

        let rayon_pool = STYLE_THREAD_POOL.lock().unwrap();
        let rayon_pool = rayon_pool.pool();
        let rayon_pool = rayon_pool.as_ref();

        // Create a layout context for use throughout the following passes.
        let mut layout_context = self.build_layout_context(
            guards.clone(),
            &map,
            origin,
            data.animation_timeline_value,
            &data.animations,
            data.stylesheets_changed,
            rayon_pool.is_some(),
        );

        let dirty_root = unsafe {
            ServoLayoutNode::<DOMLayoutData>::new(&data.dirty_root.unwrap())
                .as_element()
                .unwrap()
        };

        let traversal = RecalcStyle::new(layout_context);
        let token = {
            let shared =
                DomTraversal::<ServoLayoutElement<DOMLayoutData>>::shared_context(&traversal);
            RecalcStyle::pre_traverse(dirty_root, shared)
        };

        if token.should_traverse() {
            let dirty_root: ServoLayoutNode<DOMLayoutData> =
                driver::traverse_dom(&traversal, token, rayon_pool).as_node();

            let root_node = root_element.as_node();
            let mut box_tree = self.box_tree.borrow_mut();
            let box_tree = &mut *box_tree;
            let mut build_box_tree = || {
                if !BoxTree::update(traversal.context(), dirty_root) {
                    *box_tree = Some(Arc::new(BoxTree::construct(traversal.context(), root_node)));
                }
            };
            if let Some(pool) = rayon_pool {
                pool.install(build_box_tree)
            } else {
                build_box_tree()
            };

            let viewport_size = Size2D::new(
                self.viewport_size.width.to_f32_px(),
                self.viewport_size.height.to_f32_px(),
            );
            let run_layout = || {
                box_tree
                    .as_ref()
                    .unwrap()
                    .layout(traversal.context(), viewport_size)
            };
            let fragment_tree = Arc::new(if let Some(pool) = rayon_pool {
                pool.install(run_layout)
            } else {
                run_layout()
            });
            *self.fragment_tree.borrow_mut() = Some(fragment_tree);
        }

        layout_context = traversal.destroy();

        for element in elements_with_snapshot {
            unsafe { element.unset_snapshot_flags() }
        }

        if self.debug.dump_style_tree {
            println!(
                "{:?}",
                style::dom::ShowSubtreeDataAndPrimaryValues(root_element.as_node())
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
        if let Some(root) = &*self.fragment_tree.borrow() {
            self.perform_post_style_recalc_layout_passes(
                root.clone(),
                &data.reflow_goal,
                Some(&document),
                &mut layout_context,
            );
        }

        self.first_reflow.set(false);
        self.respond_to_query_if_necessary(
            &data.reflow_goal,
            &mut *rw_data,
            &mut layout_context,
            data.result.borrow_mut().as_mut().unwrap(),
        );
    }

    fn respond_to_query_if_necessary(
        &self,
        reflow_goal: &ReflowGoal,
        rw_data: &mut LayoutThreadData,
        context: &mut LayoutContext,
        reflow_result: &mut ReflowComplete,
    ) {
        reflow_result.pending_images =
            std::mem::replace(&mut *context.pending_images.lock().unwrap(), vec![]);

        match *reflow_goal {
            ReflowGoal::LayoutQuery(ref querymsg, _) => match querymsg {
                &QueryMsg::ContentBoxQuery(node) => {
                    rw_data.content_box_response =
                        process_content_box_request(node, self.fragment_tree.borrow().clone());
                },
                &QueryMsg::ContentBoxesQuery(node) => {
                    rw_data.content_boxes_response = process_content_boxes_request(node);
                },
                &QueryMsg::TextIndexQuery(node, point_in_node) => {
                    let point_in_node = Point2D::new(
                        Au::from_f32_px(point_in_node.x),
                        Au::from_f32_px(point_in_node.y),
                    );
                    rw_data.text_index_response = process_text_index_request(node, point_in_node);
                },
                &QueryMsg::ClientRectQuery(node) => {
                    rw_data.client_rect_response =
                        process_node_geometry_request(node, self.fragment_tree.borrow().clone());
                },
                &QueryMsg::ScrollingAreaQuery(node) => {
                    rw_data.scrolling_area_response =
                        process_node_scroll_area_request(node, self.fragment_tree.borrow().clone());
                },
                &QueryMsg::NodeScrollIdQuery(node) => {
                    let node = unsafe { ServoLayoutNode::<DOMLayoutData>::new(&node) };
                    rw_data.scroll_id_response =
                        Some(process_node_scroll_id_request(self.id, node));
                },
                &QueryMsg::ResolvedStyleQuery(node, ref pseudo, ref property) => {
                    let node = unsafe { ServoLayoutNode::<DOMLayoutData>::new(&node) };
                    let fragment_tree = self.fragment_tree.borrow().clone();
                    rw_data.resolved_style_response = process_resolved_style_request(
                        context,
                        node,
                        pseudo,
                        property,
                        fragment_tree,
                    );
                },
                &QueryMsg::ResolvedFontStyleQuery(node, ref property, ref value) => {
                    let node = unsafe { ServoLayoutNode::<DOMLayoutData>::new(&node) };
                    rw_data.resolved_font_style_response =
                        process_resolved_font_style_query(node, property, value);
                },
                &QueryMsg::OffsetParentQuery(node) => {
                    rw_data.offset_parent_response =
                        process_offset_parent_query(node, self.fragment_tree.borrow().clone());
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

                    let client_point = units::WorldPoint::from_untyped(client_point);
                    let results = self.webrender_api.hit_test(
                        Some(self.id.to_webrender()),
                        client_point,
                        flags,
                    );

                    rw_data.nodes_from_point_response =
                        results.iter().map(|result| result.node).collect()
                },
                &QueryMsg::ElementInnerTextQuery(node) => {
                    let node = unsafe { ServoLayoutNode::<DOMLayoutData>::new(&node) };
                    rw_data.element_inner_text_response = process_element_inner_text_query(node);
                },
                &QueryMsg::InnerWindowDimensionsQuery(_browsing_context_id) => {
                    // TODO(jdm): port the iframe sizing code from layout2013's display
                    //            builder in order to support query iframe sizing.
                    rw_data.inner_window_dimensions_response = None;
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
        self.webrender_api
            .send_scroll_node(units::LayoutPoint::from_untyped(point), state.scroll_id);
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

    fn perform_post_style_recalc_layout_passes(
        &self,
        fragment_tree: Arc<FragmentTree>,
        reflow_goal: &ReflowGoal,
        document: Option<&ServoLayoutDocument<DOMLayoutData>>,
        context: &mut LayoutContext,
    ) {
        Self::cancel_animations_for_nodes_not_in_fragment_tree(
            &context.style_context.animations,
            &fragment_tree,
        );

        if self.debug.trace_layout {
            if let Some(box_tree) = &*self.box_tree.borrow() {
                layout_debug::begin_trace(box_tree.clone(), fragment_tree.clone());
            }
        }

        if !reflow_goal.needs_display_list() {
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

        let mut epoch = self.epoch.get();
        epoch.next();
        self.epoch.set(epoch);

        let viewport_size = units::LayoutSize::from_untyped(Size2D::new(
            self.viewport_size.width.to_f32_px(),
            self.viewport_size.height.to_f32_px(),
        ));
        let mut display_list = DisplayList::new(
            viewport_size,
            fragment_tree.scrollable_overflow(),
            self.id.to_webrender(),
            epoch.into(),
        );

        // `dump_serialized_display_list` doesn't actually print anything. It sets up
        // the display list for printing the serialized version when `finalize()` is called.
        // We need to call this before adding any display items so that they are printed
        // during `finalize()`.
        if self.debug.dump_display_list {
            display_list.wr.dump_serialized_display_list();
        }

        // Build the root stacking context. This turns the `FragmentTree` into a
        // tree of fragments in CSS painting order and also creates all
        // applicable spatial and clip nodes.
        let root_stacking_context =
            display_list.build_stacking_context_tree(&fragment_tree, &self.debug);

        // Build the rest of the display list which inclues all of the WebRender primitives.
        let (iframe_sizes, is_contentful) =
            display_list.build(context, &fragment_tree, &root_stacking_context);

        if self.debug.dump_flow_tree {
            fragment_tree.print();
        }
        if self.debug.dump_stacking_context_tree {
            root_stacking_context.debug_print();
        }
        debug!("Layout done!");

        // Observe notifications about rendered frames if needed right before
        // sending the display list to WebRender in order to set time related
        // Progressive Web Metrics.
        self.paint_time_metrics
            .maybe_observe_paint_time(self, epoch, is_contentful);

        if reflow_goal.needs_display() {
            self.webrender_api
                .send_display_list(display_list.compositor_info, display_list.wr.finalize().1);
        }

        self.update_iframe_sizes(iframe_sizes);

        if self.debug.trace_layout {
            layout_debug::end_trace(self.generation.get());
        }

        self.generation.set(self.generation.get() + 1);
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

    /// Cancel animations for any nodes which have been removed from fragment tree.
    /// TODO(mrobinson): We should look into a way of doing this during flow tree construction.
    /// This also doesn't yet handles nodes that have been reparented.
    fn cancel_animations_for_nodes_not_in_fragment_tree(
        animations: &DocumentAnimationSet,
        root: &FragmentTree,
    ) {
        // Assume all nodes have been removed until proven otherwise.
        let mut animations = animations.sets.write();
        let mut invalid_nodes = animations.keys().cloned().collect();
        root.remove_nodes_in_fragment_tree_from_set(&mut invalid_nodes);

        // Cancel animations for any nodes that are no longer in the fragment tree.
        for node in &invalid_nodes {
            if let Some(state) = animations.get_mut(node) {
                state.cancel_all_animations();
            }
        }
    }

    /// Update the recorded iframe sizes of the contents of this layout thread and
    /// when these sizes changes, send a message to the constellation informing it
    /// of the new sizes.
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
}

impl ProfilerMetadataFactory for LayoutThread {
    fn new_metadata(&self) -> Option<TimerMetadata> {
        self.profiler_metadata()
    }
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

struct RegisteredPaintersImpl(FnvHashMap<Atom, RegisteredPainterImpl>);

impl RegisteredSpeculativePainters for RegisteredPaintersImpl {
    fn get(&self, name: &Atom) -> Option<&dyn RegisteredSpeculativePainter> {
        self.0
            .get(&name)
            .map(|painter| painter as &dyn RegisteredSpeculativePainter)
    }
}
