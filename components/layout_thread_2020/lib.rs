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
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate profile_traits;

mod dom_wrapper;

use crate::dom_wrapper::drop_style_and_layout_data;
use crate::dom_wrapper::{ServoLayoutDocument, ServoLayoutElement, ServoLayoutNode};
use app_units::Au;
use crossbeam_channel::{unbounded, Receiver, Sender};
use embedder_traits::resources::{self, Resource};
use euclid::{default::Size2D as UntypedSize2D, Point2D, Rect, Scale, Size2D};
use fnv::FnvHashMap;
use fxhash::FxHashMap;
use gfx::font_cache_thread::FontCacheThread;
use gfx::font_context;
use gfx_traits::{node_id_from_scroll_id, Epoch};
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use layout::context::LayoutContext;
use layout::display_list::{DisplayListBuilder, WebRenderImageInfo};
use layout::query::{
    process_content_box_request, process_content_boxes_request, LayoutRPCImpl, LayoutThreadData,
};
use layout::query::{process_element_inner_text_query, process_node_geometry_request};
use layout::query::{process_node_scroll_area_request, process_node_scroll_id_request};
use layout::query::{
    process_offset_parent_query, process_resolved_style_request, process_style_query,
    process_text_index_request,
};
use layout::traversal::RecalcStyle;
use layout::{BoxTreeRoot, FragmentTreeRoot};
use layout_traits::LayoutThreadFactory;
use libc::c_void;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use metrics::{PaintTimeMetrics, ProfilerMetadataFactory, ProgressiveWebMetric};
use msg::constellation_msg::{
    BackgroundHangMonitor, BackgroundHangMonitorRegister, HangAnnotation,
};
use msg::constellation_msg::{LayoutHangAnnotation, MonitoredComponentType, PipelineId};
use msg::constellation_msg::{MonitoredComponentId, TopLevelBrowsingContextId};
use net_traits::image_cache::{ImageCache, UsePlaceholder};
use parking_lot::RwLock;
use profile_traits::mem::{self as profile_mem, Report, ReportKind, ReportsChan};
use profile_traits::time::{self as profile_time, profile, TimerMetadata};
use profile_traits::time::{TimerMetadataFrameType, TimerMetadataReflowType};
use script_layout_interface::message::{LayoutThreadInit, Msg, NodesFromPointQueryType};
use script_layout_interface::message::{QueryMsg, ReflowComplete, ReflowGoal, ScriptReflow};
use script_layout_interface::rpc::TextIndexResponse;
use script_layout_interface::rpc::{LayoutRPC, OffsetParentResponse, StyleResponse};
use script_traits::{ConstellationControlMsg, LayoutControlMsg, LayoutMsg as ConstellationMsg};
use script_traits::{DrawAPaintImageResult, PaintWorkletError};
use script_traits::{Painter, WebrenderIpcSender};
use script_traits::{ScrollState, UntrustedNodeAddress, WindowSizeData};
use selectors::Element;
use servo_arc::Arc as ServoArc;
use servo_atoms::Atom;
use servo_config::opts;
use servo_config::pref;
use servo_url::ServoUrl;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::process;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::Duration;
use style::animation::Animation;
use style::context::{QuirksMode, RegisteredSpeculativePainter, RegisteredSpeculativePainters};
use style::context::{SharedStyleContext, ThreadLocalStyleContextCreationInfo};
use style::dom::{TDocument, TElement, TNode};
use style::driver;
use style::error_reporting::RustLogReporter;
use style::global_style_data::{GLOBAL_STYLE_DATA, STYLE_THREAD_POOL};
use style::invalidation::element::restyle_hints::RestyleHint;
use style::media_queries::{Device, MediaList, MediaType};
use style::properties::PropertyId;
use style::selector_parser::SnapshotMap;
use style::shared_lock::{SharedRwLock, SharedRwLockReadGuard, StylesheetGuards};
use style::stylesheets::{
    DocumentStyleSheet, Origin, Stylesheet, StylesheetInDocument, UserAgentStylesheets,
};
use style::stylist::Stylist;
use style::thread_state::{self, ThreadState};
use style::timer::Timer;
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

    /// A channel on which new animations that have been triggered by style recalculation can be
    /// sent.
    new_animations_sender: Sender<Animation>,

    /// Receives newly-discovered animations.
    _new_animations_receiver: Receiver<Animation>,

    /// The number of Web fonts that have been requested but not yet loaded.
    outstanding_web_fonts: Arc<AtomicUsize>,

    /// The root of the box tree.
    box_tree_root: RefCell<Option<BoxTreeRoot>>,

    /// The root of the fragment tree.
    fragment_tree_root: RefCell<Option<FragmentTreeRoot>>,

    /// The document-specific shared lock used for author-origin stylesheets
    document_shared_lock: Option<SharedRwLock>,

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

    /// Webrender document.
    webrender_document: webrender_api::DocumentId,

    /// The timer object to control the timing of the animations. This should
    /// only be a test-mode timer during testing for animations.
    timer: Timer,

    /// Paint time metrics.
    paint_time_metrics: PaintTimeMetrics,

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

    /// Dumps the flow tree after a layout.
    dump_flow_tree: bool,

    /// Emits notifications when there is a relayout.
    relayout_event: bool,
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
        webrender_document: webrender_api::DocumentId,
        paint_time_metrics: PaintTimeMetrics,
        busy: Arc<AtomicBool>,
        load_webfonts_synchronously: bool,
        window_size: WindowSizeData,
        dump_display_list: bool,
        dump_display_list_json: bool,
        dump_style_tree: bool,
        dump_rule_tree: bool,
        relayout_event: bool,
        _nonincremental_layout: bool,
        _trace_layout: bool,
        dump_flow_tree: bool,
    ) {
        thread::Builder::new()
            .name(format!("LayoutThread {:?}", id))
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
                        webrender_document,
                        paint_time_metrics,
                        busy,
                        load_webfonts_synchronously,
                        window_size,
                        relayout_event,
                        dump_display_list,
                        dump_display_list_json,
                        dump_style_tree,
                        dump_rule_tree,
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
        webrender_api_sender: WebrenderIpcSender,
        webrender_document: webrender_api::DocumentId,
        paint_time_metrics: PaintTimeMetrics,
        busy: Arc<AtomicBool>,
        load_webfonts_synchronously: bool,
        window_size: WindowSizeData,
        relayout_event: bool,
        dump_display_list: bool,
        dump_display_list_json: bool,
        dump_style_tree: bool,
        dump_rule_tree: bool,
        dump_flow_tree: bool,
    ) -> LayoutThread {
        // Let webrender know about this pipeline by sending an empty display list.
        webrender_api_sender.send_initial_transaction(webrender_document, id.to_webrender());

        // The device pixel ratio is incorrect (it does not have the hidpi value),
        // but it will be set correctly when the initial reflow takes place.
        let device = Device::new(
            MediaType::screen(),
            window_size.initial_viewport,
            window_size.device_pixel_ratio,
        );

        // Create the channel on which new animations can be sent.
        let (new_animations_sender, new_animations_receiver) = unbounded();

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
            script_chan: script_chan.clone(),
            background_hang_monitor,
            time_profiler_chan: time_profiler_chan,
            mem_profiler_chan: mem_profiler_chan,
            registered_painters: RegisteredPaintersImpl(Default::default()),
            image_cache,
            font_cache_thread: font_cache_thread,
            first_reflow: Cell::new(true),
            font_cache_receiver: font_cache_receiver,
            font_cache_sender: ipc_font_cache_sender,
            generation: Cell::new(0),
            new_animations_sender: new_animations_sender,
            _new_animations_receiver: new_animations_receiver,
            outstanding_web_fonts: Arc::new(AtomicUsize::new(0)),
            box_tree_root: Default::default(),
            fragment_tree_root: Default::default(),
            document_shared_lock: None,
            // Epoch starts at 1 because of the initial display list for epoch 0 that we send to WR
            epoch: Cell::new(Epoch(1)),
            viewport_size: Size2D::new(Au(0), Au(0)),
            webrender_api: webrender_api_sender,
            webrender_document,
            stylist: Stylist::new(device, QuirksMode::NoQuirks),
            rw_data: Arc::new(Mutex::new(LayoutThreadData {
                constellation_chan: constellation_chan,
                display_list: None,
                content_box_response: None,
                content_boxes_response: Vec::new(),
                client_rect_response: Rect::zero(),
                scroll_id_response: None,
                scroll_area_response: Rect::zero(),
                resolved_style_response: String::new(),
                offset_parent_response: OffsetParentResponse::empty(),
                style_response: StyleResponse(None),
                scroll_offsets: HashMap::new(),
                text_index_response: TextIndexResponse(None),
                nodes_from_point_response: vec![],
                element_inner_text_response: String::new(),
                inner_window_dimensions_response: None,
            })),
            webrender_image_cache: Default::default(),
            timer: if pref!(layout.animations.test.enabled) {
                Timer::test_mode()
            } else {
                Timer::new()
            },
            paint_time_metrics: paint_time_metrics,
            busy,
            load_webfonts_synchronously,
            relayout_event,
            dump_display_list,
            dump_display_list_json,
            dump_style_tree,
            dump_rule_tree,
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
        script_initiated_layout: bool,
        snapshot_map: &'a SnapshotMap,
    ) -> LayoutContext<'a> {
        let thread_local_style_context_creation_data =
            ThreadLocalStyleContextCreationInfo::new(self.new_animations_sender.clone());

        LayoutContext {
            id: self.id,
            origin: self.url.origin(),
            style_context: SharedStyleContext {
                stylist: &self.stylist,
                options: GLOBAL_STYLE_DATA.options.clone(),
                guards,
                visited_styles_enabled: false,
                running_animations: Default::default(),
                expired_animations: Default::default(),
                registered_speculative_painters: &self.registered_painters,
                local_context_creation_data: Mutex::new(thread_local_style_context_creation_data),
                timer: self.timer.clone(),
                traversal_flags: TraversalFlags::empty(),
                snapshot_map: snapshot_map,
            },
            image_cache: self.image_cache.clone(),
            font_cache_thread: Mutex::new(self.font_cache_thread.clone()),
            webrender_image_cache: self.webrender_image_cache.clone(),
            pending_images: if script_initiated_layout {
                Some(Mutex::new(Vec::new()))
            } else {
                None
            },
            use_rayon: STYLE_THREAD_POOL.pool().is_some(),
        }
    }

    fn notify_activity_to_hang_monitor(&self, request: &Msg) {
        let hang_annotation = match request {
            Msg::AddStylesheet(..) => LayoutHangAnnotation::AddStylesheet,
            Msg::RemoveStylesheet(..) => LayoutHangAnnotation::RemoveStylesheet,
            Msg::SetQuirksMode(..) => LayoutHangAnnotation::SetQuirksMode,
            Msg::Reflow(..) => LayoutHangAnnotation::Reflow,
            Msg::GetRPC(..) => LayoutHangAnnotation::GetRPC,
            Msg::TickAnimations => LayoutHangAnnotation::TickAnimations,
            Msg::AdvanceClockMs(..) => LayoutHangAnnotation::AdvanceClockMs,
            Msg::ReapStyleAndLayoutData(..) => LayoutHangAnnotation::ReapStyleAndLayoutData,
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
            Msg::GetRunningAnimations(..) => LayoutHangAnnotation::GetRunningAnimations,
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
            Request::FromPipeline(LayoutControlMsg::TickAnimations) => {
                self.handle_request_helper(Msg::TickAnimations, possibly_locked_rw_data)
            },
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
            Msg::TickAnimations => self.tick_all_animations(),
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
                    self.webrender_document,
                    webrender_api::units::LayoutPoint::from_untyped(point),
                    state.scroll_id,
                    webrender_api::ScrollClamping::ToContentBounds,
                );
            },
            Msg::ReapStyleAndLayoutData(dead_data) => unsafe {
                drop_style_and_layout_data(dead_data)
            },
            Msg::CollectReports(reports_chan) => {
                self.collect_reports(reports_chan, possibly_locked_rw_data);
            },
            Msg::GetCurrentEpoch(sender) => {
                let _rw_data = possibly_locked_rw_data.lock();
                sender.send(self.epoch.get()).unwrap();
            },
            Msg::AdvanceClockMs(how_many, do_tick) => {
                self.handle_advance_clock_ms(how_many, do_tick);
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
            Msg::GetRunningAnimations(sender) => {
                let _ = sender.send(0);
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
            self.webrender_document,
            info.paint_time_metrics,
            info.layout_is_busy,
            self.load_webfonts_synchronously,
            info.window_size,
            self.dump_display_list,
            self.dump_display_list_json,
            self.dump_style_tree,
            self.dump_rule_tree,
            self.relayout_event,
            true,  // nonincremental_layout
            false, // trace_layout
            self.dump_flow_tree,
        );
    }

    /// Enters a quiescent state in which no new messages will be processed until an `ExitNow` is
    /// received. A pong is immediately sent on the given response channel.
    fn prepare_to_exit(&mut self, response_chan: Sender<()>) {
        response_chan.send(()).unwrap();
        loop {
            match self.port.recv().unwrap() {
                Msg::ReapStyleAndLayoutData(dead_data) => unsafe {
                    drop_style_and_layout_data(dead_data)
                },
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
                self.load_webfonts_synchronously,
            );
        }
    }

    /// Advances the animation clock of the document.
    fn handle_advance_clock_ms<'a, 'b>(&mut self, how_many_ms: i32, tick_animations: bool) {
        self.timer.increment(how_many_ms as f64 / 1000.0);
        if tick_animations {
            self.tick_all_animations();
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
        let document = unsafe { ServoLayoutNode::new(&data.document) };
        let document = document.as_document().unwrap();

        let mut rw_data = possibly_locked_rw_data.lock();

        let element = match document.root_element() {
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
                        &QueryMsg::StyleQuery(_) => {
                            rw_data.style_response = StyleResponse(None);
                        },
                        &QueryMsg::TextIndexQuery(..) => {
                            rw_data.text_index_response = TextIndexResponse(None);
                        },
                        &QueryMsg::ElementInnerTextQuery(_) => {
                            rw_data.element_inner_text_response = String::new();
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

        let initial_viewport = data.window_size.initial_viewport;
        let device_pixel_ratio = data.window_size.device_pixel_ratio;
        let old_viewport_size = self.viewport_size;
        let current_screen_size = Size2D::new(
            Au::from_f32_px(initial_viewport.width),
            Au::from_f32_px(initial_viewport.height),
        );

        // Calculate the actual viewport as per DEVICE-ADAPT § 6
        // If the entire flow tree is invalid, then it will be reflowed anyhow.
        let document_shared_lock = document.style_shared_lock();
        self.document_shared_lock = Some(document_shared_lock.clone());
        let author_guard = document_shared_lock.read();

        let ua_stylesheets = &*UA_STYLESHEETS;
        let ua_or_user_guard = ua_stylesheets.shared_lock.read();
        let guards = StylesheetGuards {
            author: &author_guard,
            ua_or_user: &ua_or_user_guard,
        };

        let had_used_viewport_units = self.stylist.device().used_viewport_units();
        let device = Device::new(MediaType::screen(), initial_viewport, device_pixel_ratio);
        let sheet_origins_affected_by_device_change = self.stylist.set_device(device, &guards);

        self.stylist
            .force_stylesheet_origins_dirty(sheet_origins_affected_by_device_change);
        self.viewport_size =
            self.stylist
                .viewport_constraints()
                .map_or(current_screen_size, |constraints| {
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
                if let Some(mut data) = element.mutate_data() {
                    data.hint.insert(RestyleHint::recascade_subtree());
                }
            }
        }

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
        document.flush_shadow_roots_stylesheets(
            &self.stylist.device(),
            document.quirks_mode(),
            guards.author.clone(),
        );

        let restyles = document.drain_pending_restyles();

        let mut map = SnapshotMap::new();
        let elements_with_snapshot: Vec<_> = restyles
            .iter()
            .filter(|r| r.1.snapshot.is_some())
            .map(|r| r.0)
            .collect();

        for (el, restyle) in restyles {
            // Propagate the descendant bit up the ancestors. Do this before
            // the restyle calculation so that we can also do it for new
            // unstyled nodes, which the descendants bit helps us find.
            if let Some(parent) = el.parent_element() {
                unsafe { parent.note_dirty_descendant() };
            }

            // If we haven't styled this node yet, we don't need to track a
            // restyle.
            let style_data = match el.get_data() {
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

            let mut style_data = style_data.borrow_mut();

            // Stash the data on the element for processing by the style system.
            style_data.hint.insert(restyle.hint.into());
            style_data.damage = restyle.damage;
            debug!("Noting restyle for {:?}: {:?}", el, style_data);
        }

        self.stylist.flush(&guards, Some(element), Some(&map));

        // Create a layout context for use throughout the following passes.
        let mut layout_context = self.build_layout_context(guards.clone(), true, &map);

        let traversal = RecalcStyle::new(layout_context);
        let token = {
            let shared = DomTraversal::<ServoLayoutElement>::shared_context(&traversal);
            RecalcStyle::pre_traverse(element, shared)
        };

        let rayon_pool = STYLE_THREAD_POOL.pool();
        let rayon_pool = rayon_pool.as_ref();

        let box_tree = if token.should_traverse() {
            driver::traverse_dom(&traversal, token, rayon_pool);

            let root_node = document.root_element().unwrap().as_node();
            let build_box_tree = || BoxTreeRoot::construct(traversal.context(), root_node);
            let box_tree = if let Some(pool) = rayon_pool {
                pool.install(build_box_tree)
            } else {
                build_box_tree()
            };
            Some(box_tree)
        } else {
            None
        };

        layout_context = traversal.destroy();

        if let Some(box_tree) = box_tree {
            let viewport_size = Size2D::new(
                self.viewport_size.width.to_f32_px(),
                self.viewport_size.height.to_f32_px(),
            );
            let run_layout = || box_tree.layout(&layout_context, viewport_size);
            let fragment_tree = if let Some(pool) = rayon_pool {
                pool.install(run_layout)
            } else {
                run_layout()
            };
            *self.box_tree_root.borrow_mut() = Some(box_tree);
            *self.fragment_tree_root.borrow_mut() = Some(fragment_tree);
        }

        for element in elements_with_snapshot {
            unsafe { element.unset_snapshot_flags() }
        }

        if self.dump_style_tree {
            println!(
                "{:?}",
                style::dom::ShowSubtreeDataAndPrimaryValues(element.as_node())
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
        unsafe {
            layout_context.style_context.stylist.rule_tree().maybe_gc();
        }

        // Perform post-style recalculation layout passes.
        if let Some(root) = &*self.fragment_tree_root.borrow() {
            self.perform_post_style_recalc_layout_passes(
                root,
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
        let pending_images = match &context.pending_images {
            Some(pending) => std::mem::take(&mut *pending.lock().unwrap()),
            None => Vec::new(),
        };
        reflow_result.pending_images = pending_images;
        match *reflow_goal {
            ReflowGoal::LayoutQuery(ref querymsg, _) => match querymsg {
                &QueryMsg::ContentBoxQuery(node) => {
                    rw_data.content_box_response = process_content_box_request(
                        node,
                        (&*self.fragment_tree_root.borrow()).as_ref(),
                    );
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
                    rw_data.client_rect_response = process_node_geometry_request(
                        node,
                        (&*self.fragment_tree_root.borrow()).as_ref(),
                    );
                },
                &QueryMsg::NodeScrollGeometryQuery(node) => {
                    rw_data.scroll_area_response = process_node_scroll_area_request(node);
                },
                &QueryMsg::NodeScrollIdQuery(node) => {
                    let node = unsafe { ServoLayoutNode::new(&node) };
                    rw_data.scroll_id_response =
                        Some(process_node_scroll_id_request(self.id, node));
                },
                &QueryMsg::ResolvedStyleQuery(node, ref pseudo, ref property) => {
                    let node = unsafe { ServoLayoutNode::new(&node) };
                    rw_data.resolved_style_response =
                        process_resolved_style_request(context, node, pseudo, property);
                },
                &QueryMsg::OffsetParentQuery(node) => {
                    rw_data.offset_parent_response = process_offset_parent_query(node);
                },
                &QueryMsg::StyleQuery(node) => {
                    let node = unsafe { ServoLayoutNode::new(&node) };
                    rw_data.style_response = process_style_query(node);
                },
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
                        self.webrender_document,
                        Some(self.id.to_webrender()),
                        client_point,
                        flags,
                    );

                    rw_data.nodes_from_point_response = results
                        .items
                        .iter()
                        .map(|item| UntrustedNodeAddress(item.tag.0 as *const c_void))
                        .collect()
                },
                &QueryMsg::ElementInnerTextQuery(node) => {
                    let node = unsafe { ServoLayoutNode::new(&node) };
                    rw_data.element_inner_text_response = process_element_inner_text_query(node);
                },
                &QueryMsg::InnerWindowDimensionsQuery(_browsing_context_id) => {
                    // TODO(jdm): port the iframe sizing code from layout2013's display
                    //            builder in order to support query iframe sizing.
                    rw_data.inner_window_dimensions_response = None;
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

    fn tick_all_animations<'a, 'b>(&mut self) {
        self.tick_animations();
    }

    fn tick_animations(&mut self) {
        if self.relayout_event {
            println!(
                "**** pipeline={}\tForDisplay\tSpecial\tAnimationTick",
                self.id
            );
        }

        if let Some(root) = &*self.fragment_tree_root.borrow() {
            // Unwrap here should not panic since self.fragment_tree_root is only ever set to Some(_)
            // in handle_reflow() where self.document_shared_lock is as well.
            let author_shared_lock = self.document_shared_lock.clone().unwrap();
            let author_guard = author_shared_lock.read();
            let ua_or_user_guard = UA_STYLESHEETS.shared_lock.read();
            let guards = StylesheetGuards {
                author: &author_guard,
                ua_or_user: &ua_or_user_guard,
            };
            let snapshots = SnapshotMap::new();
            let mut layout_context = self.build_layout_context(guards, false, &snapshots);

            self.perform_post_style_recalc_layout_passes(
                root,
                &ReflowGoal::TickAnimations,
                None,
                &mut layout_context,
            );
            assert!(layout_context.pending_images.is_none());
        }
    }

    fn perform_post_style_recalc_layout_passes(
        &self,
        fragment_tree: &FragmentTreeRoot,
        reflow_goal: &ReflowGoal,
        document: Option<&ServoLayoutDocument>,
        context: &mut LayoutContext,
    ) {
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

        let mut display_list = DisplayListBuilder::new(
            self.id.to_webrender(),
            context,
            fragment_tree.scrollable_overflow(),
        );

        fragment_tree.build_display_list(&mut display_list);

        if self.dump_flow_tree {
            fragment_tree.print();
        }
        if self.dump_display_list {
            display_list.wr.print_display_list();
        }

        debug!("Layout done!");

        let mut epoch = self.epoch.get();
        epoch.next();
        self.epoch.set(epoch);

        // Observe notifications about rendered frames if needed right before
        // sending the display list to WebRender in order to set time related
        // Progressive Web Metrics.
        self.paint_time_metrics
            .maybe_observe_paint_time(self, epoch, display_list.is_contentful);

        let viewport_size = webrender_api::units::LayoutSize::from_untyped(Size2D::new(
            self.viewport_size.width.to_f32_px(),
            self.viewport_size.height.to_f32_px(),
        ));
        self.webrender_api.send_display_list(
            self.webrender_document,
            epoch,
            viewport_size,
            display_list.wr.finalize(),
        );

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
