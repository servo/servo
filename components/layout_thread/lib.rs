/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The layout thread. Performs layout on the DOM, builds display lists and sends them to be
//! painted.

#![feature(mpsc_select)]

extern crate app_units;
extern crate atomic_refcell;
extern crate euclid;
extern crate fnv;
extern crate gfx;
extern crate gfx_traits;
#[macro_use]
extern crate html5ever;
extern crate ipc_channel;
#[macro_use]
extern crate layout;
extern crate layout_traits;
#[macro_use]
extern crate lazy_static;
extern crate libc;
#[macro_use]
extern crate log;
extern crate malloc_size_of;
extern crate metrics;
extern crate msg;
extern crate net_traits;
extern crate nonzero;
extern crate parking_lot;
#[macro_use]
extern crate profile_traits;
extern crate range;
extern crate rayon;
extern crate script;
extern crate script_layout_interface;
extern crate script_traits;
extern crate selectors;
extern crate serde_json;
extern crate servo_allocator;
extern crate servo_arc;
extern crate servo_atoms;
extern crate servo_config;
extern crate servo_geometry;
extern crate servo_url;
extern crate style;
extern crate style_traits;
extern crate webrender_api;

mod dom_wrapper;

use app_units::Au;
use dom_wrapper::{ServoLayoutElement, ServoLayoutDocument, ServoLayoutNode};
use dom_wrapper::drop_style_and_layout_data;
use euclid::{Point2D, Rect, Size2D, TypedScale, TypedSize2D};
use fnv::FnvHashMap;
use gfx::display_list::{OpaqueNode, WebRenderImageInfo};
use gfx::font;
use gfx::font_cache_thread::FontCacheThread;
use gfx::font_context;
use gfx_traits::{Epoch, node_id_from_clip_id};
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use layout::animation;
use layout::construct::ConstructionResult;
use layout::context::LayoutContext;
use layout::context::RegisteredPainter;
use layout::context::RegisteredPainters;
use layout::context::malloc_size_of_persistent_local_context;
use layout::display_list::WebRenderDisplayListConverter;
use layout::flow::{Flow, GetBaseFlow, ImmutableFlowUtils, MutableOwnedFlowUtils};
use layout::flow_ref::FlowRef;
use layout::incremental::{LayoutDamageComputation, RelayoutMode, SpecialRestyleDamage};
use layout::layout_debug;
use layout::parallel;
use layout::query::{LayoutRPCImpl, LayoutThreadData, process_content_box_request, process_content_boxes_request};
use layout::query::{process_margin_style_query, process_node_overflow_request, process_resolved_style_request};
use layout::query::{process_node_geometry_request, process_node_scroll_area_request};
use layout::query::{process_node_scroll_root_id_request, process_offset_parent_query};
use layout::sequential;
use layout::traversal::{ComputeStackingRelativePositions, PreorderFlowTraversal, RecalcStyleAndConstructFlows};
use layout::wrapper::LayoutNodeLayoutData;
use layout_traits::LayoutThreadFactory;
use libc::c_void;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use metrics::{PaintTimeMetrics, ProfilerMetadataFactory, ProgressiveWebMetric};
use msg::constellation_msg::PipelineId;
use msg::constellation_msg::TopLevelBrowsingContextId;
use net_traits::image_cache::{ImageCache, UsePlaceholder};
use parking_lot::RwLock;
use profile_traits::mem::{self, Report, ReportKind, ReportsChan};
use profile_traits::time::{self, TimerMetadata, profile};
use profile_traits::time::{TimerMetadataFrameType, TimerMetadataReflowType};
use script_layout_interface::message::{Msg, NewLayoutThreadInfo, NodesFromPointQueryType, Reflow};
use script_layout_interface::message::{ReflowComplete, ReflowGoal, ScriptReflow};
use script_layout_interface::rpc::{LayoutRPC, MarginStyleResponse, NodeOverflowResponse, OffsetParentResponse};
use script_layout_interface::rpc::TextIndexResponse;
use script_layout_interface::wrapper_traits::LayoutNode;
use script_traits::{ConstellationControlMsg, LayoutControlMsg, LayoutMsg as ConstellationMsg};
use script_traits::{DrawAPaintImageResult, PaintWorkletError};
use script_traits::{ScrollState, UntrustedNodeAddress};
use script_traits::Painter;
use selectors::Element;
use servo_arc::Arc as ServoArc;
use servo_atoms::Atom;
use servo_config::opts;
use servo_config::prefs::PREFS;
use servo_config::resource_files::read_resource_file;
use servo_geometry::max_rect;
use servo_url::ServoUrl;
use std::borrow::ToOwned;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::mem as std_mem;
use std::ops::{Deref, DerefMut};
use std::process;
use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use style::animation::Animation;
use style::context::{QuirksMode, RegisteredSpeculativePainter, RegisteredSpeculativePainters};
use style::context::{SharedStyleContext, StyleSystemOptions, ThreadLocalStyleContextCreationInfo};
use style::dom::{ShowSubtree, ShowSubtreeDataAndPrimaryValues, TElement, TNode};
use style::driver;
use style::error_reporting::{NullReporter, RustLogReporter};
use style::invalidation::element::restyle_hints::RestyleHint;
use style::logical_geometry::LogicalPoint;
use style::media_queries::{Device, MediaList, MediaType};
use style::properties::PropertyId;
use style::selector_parser::SnapshotMap;
use style::servo::restyle_damage::ServoRestyleDamage;
use style::shared_lock::{SharedRwLock, SharedRwLockReadGuard, StylesheetGuards};
use style::stylesheets::{Origin, Stylesheet, DocumentStyleSheet, StylesheetInDocument, UserAgentStylesheets};
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

    /// The channel on which messages can be sent to the constellation.
    constellation_chan: IpcSender<ConstellationMsg>,

    /// The channel on which messages can be sent to the script thread.
    script_chan: IpcSender<ConstellationControlMsg>,

    /// The channel on which messages can be sent to the time profiler.
    time_profiler_chan: time::ProfilerChan,

    /// The channel on which messages can be sent to the memory profiler.
    mem_profiler_chan: mem::ProfilerChan,

    /// Reference to the script thread image cache.
    image_cache: Arc<ImageCache>,

    /// Public interface to the font cache thread.
    font_cache_thread: FontCacheThread,

    /// Is this the first reflow in this LayoutThread?
    first_reflow: Cell<bool>,

    /// The workers that we use for parallel operation.
    parallel_traversal: Option<rayon::ThreadPool>,

    /// Flag to indicate whether to use parallel operations
    parallel_flag: bool,

    /// Starts at zero, and increased by one every time a layout completes.
    /// This can be used to easily check for invalid stale data.
    generation: Cell<u32>,

    /// A channel on which new animations that have been triggered by style recalculation can be
    /// sent.
    new_animations_sender: Sender<Animation>,

    /// Receives newly-discovered animations.
    new_animations_receiver: Receiver<Animation>,

    /// The number of Web fonts that have been requested but not yet loaded.
    outstanding_web_fonts: Arc<AtomicUsize>,

    /// The root of the flow tree.
    root_flow: RefCell<Option<FlowRef>>,

    /// The document-specific shared lock used for author-origin stylesheets
    document_shared_lock: Option<SharedRwLock>,

    /// The list of currently-running animations.
    running_animations: ServoArc<RwLock<FnvHashMap<OpaqueNode, Vec<Animation>>>>,

    /// The list of animations that have expired since the last style recalculation.
    expired_animations: ServoArc<RwLock<FnvHashMap<OpaqueNode, Vec<Animation>>>>,

    /// A counter for epoch messages
    epoch: Cell<Epoch>,

    /// The size of the viewport. This may be different from the size of the screen due to viewport
    /// constraints.
    viewport_size: Size2D<Au>,

    /// A mutex to allow for fast, read-only RPC of layout's internal data
    /// structures, while still letting the LayoutThread modify them.
    ///
    /// All the other elements of this struct are read-only.
    rw_data: Arc<Mutex<LayoutThreadData>>,

    webrender_image_cache: Arc<RwLock<FnvHashMap<(ServoUrl, UsePlaceholder),
                                                 WebRenderImageInfo>>>,

    /// The executors for paint worklets.
    registered_painters: RegisteredPaintersImpl,

    /// Webrender interface.
    webrender_api: webrender_api::RenderApi,

    /// Webrender document.
    webrender_document: webrender_api::DocumentId,

    /// The timer object to control the timing of the animations. This should
    /// only be a test-mode timer during testing for animations.
    timer: Timer,

    // Number of layout threads. This is copied from `servo_config::opts`, but we'd
    // rather limit the dependency on that module here.
    layout_threads: usize,

    /// Paint time metrics.
    paint_time_metrics: PaintTimeMetrics,
}

impl LayoutThreadFactory for LayoutThread {
    type Message = Msg;

    /// Spawns a new layout thread.
    fn create(id: PipelineId,
              top_level_browsing_context_id: TopLevelBrowsingContextId,
              url: ServoUrl,
              is_iframe: bool,
              chan: (Sender<Msg>, Receiver<Msg>),
              pipeline_port: IpcReceiver<LayoutControlMsg>,
              constellation_chan: IpcSender<ConstellationMsg>,
              script_chan: IpcSender<ConstellationControlMsg>,
              image_cache: Arc<ImageCache>,
              font_cache_thread: FontCacheThread,
              time_profiler_chan: time::ProfilerChan,
              mem_profiler_chan: mem::ProfilerChan,
              content_process_shutdown_chan: Option<IpcSender<()>>,
              webrender_api_sender: webrender_api::RenderApiSender,
              webrender_document: webrender_api::DocumentId,
              layout_threads: usize,
              paint_time_metrics: PaintTimeMetrics) {
        thread::Builder::new().name(format!("LayoutThread {:?}", id)).spawn(move || {
            thread_state::initialize(ThreadState::LAYOUT);

            // In order to get accurate crash reports, we install the top-level bc id.
            TopLevelBrowsingContextId::install(top_level_browsing_context_id);

            { // Ensures layout thread is destroyed before we send shutdown message
                let sender = chan.0;
                let layout = LayoutThread::new(id,
                                               top_level_browsing_context_id,
                                               url,
                                               is_iframe,
                                               chan.1,
                                               pipeline_port,
                                               constellation_chan,
                                               script_chan,
                                               image_cache.clone(),
                                               font_cache_thread,
                                               time_profiler_chan,
                                               mem_profiler_chan.clone(),
                                               webrender_api_sender,
                                               webrender_document,
                                               layout_threads,
                                               paint_time_metrics);

                let reporter_name = format!("layout-reporter-{}", id);
                mem_profiler_chan.run_with_memory_reporting(|| {
                    layout.start();
                }, reporter_name, sender, Msg::CollectReports);
            }
            if let Some(content_process_shutdown_chan) = content_process_shutdown_chan {
                let _ = content_process_shutdown_chan.send(());
            }
        }).expect("Thread spawning failed");
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
        self.script_reflow.script_join_chan.send(
            self.result
                .borrow_mut()
                .take()
                .unwrap()).unwrap();
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

fn add_font_face_rules(stylesheet: &Stylesheet,
                       guard: &SharedRwLockReadGuard,
                       device: &Device,
                       font_cache_thread: &FontCacheThread,
                       font_cache_sender: &IpcSender<()>,
                       outstanding_web_fonts_counter: &Arc<AtomicUsize>) {
    if opts::get().load_webfonts_synchronously {
        let (sender, receiver) = ipc::channel().unwrap();
        stylesheet.effective_font_face_rules(&device, guard, |rule| {
            if let Some(font_face) = rule.font_face() {
                let effective_sources = font_face.effective_sources();
                font_cache_thread.add_web_font(font_face.family().clone(),
                                               effective_sources,
                                               sender.clone());
                receiver.recv().unwrap();
            }
        })
    } else {
        stylesheet.effective_font_face_rules(&device, guard, |rule| {
            if let Some(font_face) = rule.font_face() {
                let effective_sources = font_face.effective_sources();
                outstanding_web_fonts_counter.fetch_add(1, Ordering::SeqCst);
                font_cache_thread.add_web_font(font_face.family().clone(),
                                              effective_sources,
                                              (*font_cache_sender).clone());
            }
        })
    }
}

impl LayoutThread {
    /// Creates a new `LayoutThread` structure.
    fn new(id: PipelineId,
           top_level_browsing_context_id: TopLevelBrowsingContextId,
           url: ServoUrl,
           is_iframe: bool,
           port: Receiver<Msg>,
           pipeline_port: IpcReceiver<LayoutControlMsg>,
           constellation_chan: IpcSender<ConstellationMsg>,
           script_chan: IpcSender<ConstellationControlMsg>,
           image_cache: Arc<ImageCache>,
           font_cache_thread: FontCacheThread,
           time_profiler_chan: time::ProfilerChan,
           mem_profiler_chan: mem::ProfilerChan,
           webrender_api_sender: webrender_api::RenderApiSender,
           webrender_document: webrender_api::DocumentId,
           layout_threads: usize,
           paint_time_metrics: PaintTimeMetrics)
           -> LayoutThread {
        // The device pixel ratio is incorrect (it does not have the hidpi value),
        // but it will be set correctly when the initial reflow takes place.
        let device = Device::new(
            MediaType::screen(),
            opts::get().initial_window_size.to_f32() * TypedScale::new(1.0),
            TypedScale::new(opts::get().device_pixels_per_px.unwrap_or(1.0)));

        let configuration =
            rayon::Configuration::new().num_threads(layout_threads)
                                       .start_handler(|_| thread_state::initialize_layout_worker_thread());
        let parallel_traversal = if layout_threads > 1 {
            Some(rayon::ThreadPool::new(configuration).expect("ThreadPool creation failed"))
        } else {
            None
        };
        debug!("Possible layout Threads: {}", layout_threads);

        // Create the channel on which new animations can be sent.
        let (new_animations_sender, new_animations_receiver) = channel();

        // Proxy IPC messages from the pipeline to the layout thread.
        let pipeline_receiver = ROUTER.route_ipc_receiver_to_new_mpsc_receiver(pipeline_port);

        // Ask the router to proxy IPC messages from the font cache thread to the layout thread.
        let (ipc_font_cache_sender, ipc_font_cache_receiver) = ipc::channel().unwrap();
        let font_cache_receiver =
            ROUTER.route_ipc_receiver_to_new_mpsc_receiver(ipc_font_cache_receiver);


        LayoutThread {
            id: id,
            top_level_browsing_context_id: top_level_browsing_context_id,
            url: url,
            is_iframe: is_iframe,
            port: port,
            pipeline_port: pipeline_receiver,
            script_chan: script_chan.clone(),
            constellation_chan: constellation_chan.clone(),
            time_profiler_chan: time_profiler_chan,
            mem_profiler_chan: mem_profiler_chan,
            registered_painters: RegisteredPaintersImpl(FnvHashMap::default()),
            image_cache: image_cache.clone(),
            font_cache_thread: font_cache_thread,
            first_reflow: Cell::new(true),
            font_cache_receiver: font_cache_receiver,
            font_cache_sender: ipc_font_cache_sender,
            parallel_traversal: parallel_traversal,
            parallel_flag: true,
            generation: Cell::new(0),
            new_animations_sender: new_animations_sender,
            new_animations_receiver: new_animations_receiver,
            outstanding_web_fonts: Arc::new(AtomicUsize::new(0)),
            root_flow: RefCell::new(None),
            document_shared_lock: None,
            running_animations: ServoArc::new(RwLock::new(FnvHashMap::default())),
            expired_animations: ServoArc::new(RwLock::new(FnvHashMap::default())),
            epoch: Cell::new(Epoch(0)),
            viewport_size: Size2D::new(Au(0), Au(0)),
            webrender_api: webrender_api_sender.create_api(),
            webrender_document,
            stylist: Stylist::new(device, QuirksMode::NoQuirks),
            rw_data: Arc::new(Mutex::new(
                LayoutThreadData {
                    constellation_chan: constellation_chan,
                    display_list: None,
                    content_box_response: None,
                    content_boxes_response: Vec::new(),
                    client_rect_response: Rect::zero(),
                    scroll_root_id_response: None,
                    scroll_area_response: Rect::zero(),
                    overflow_response: NodeOverflowResponse(None),
                    resolved_style_response: String::new(),
                    offset_parent_response: OffsetParentResponse::empty(),
                    margin_style_response: MarginStyleResponse::empty(),
                    scroll_offsets: HashMap::new(),
                    text_index_response: TextIndexResponse(None),
                    nodes_from_point_response: vec![],
                })),
            webrender_image_cache:
                Arc::new(RwLock::new(FnvHashMap::default())),
            timer:
                if PREFS.get("layout.animations.test.enabled")
                           .as_boolean().unwrap_or(false) {
                   Timer::test_mode()
                } else {
                    Timer::new()
                },
            layout_threads: layout_threads,
            paint_time_metrics: paint_time_metrics,
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
    fn build_layout_context<'a>(&'a self,
                                guards: StylesheetGuards<'a>,
                                script_initiated_layout: bool,
                                snapshot_map: &'a SnapshotMap)
                                -> LayoutContext<'a> {
        let thread_local_style_context_creation_data =
            ThreadLocalStyleContextCreationInfo::new(self.new_animations_sender.clone());

        LayoutContext {
            id: self.id,
            style_context: SharedStyleContext {
                stylist: &self.stylist,
                options: StyleSystemOptions::default(),
                guards: guards,
                visited_styles_enabled: false,
                running_animations: self.running_animations.clone(),
                expired_animations: self.expired_animations.clone(),
                registered_speculative_painters: &self.registered_painters,
                local_context_creation_data: Mutex::new(thread_local_style_context_creation_data),
                timer: self.timer.clone(),
                traversal_flags: TraversalFlags::empty(),
                snapshot_map: snapshot_map,
            },
            image_cache: self.image_cache.clone(),
            font_cache_thread: Mutex::new(self.font_cache_thread.clone()),
            webrender_image_cache: self.webrender_image_cache.clone(),
            pending_images: if script_initiated_layout { Some(Mutex::new(vec![])) } else { None },
            newly_transitioning_nodes: if script_initiated_layout { Some(Mutex::new(vec![])) } else { None },
            registered_painters: &self.registered_painters,
        }
    }

    /// Receives and dispatches messages from the script and constellation threads
    fn handle_request<'a, 'b>(&mut self, possibly_locked_rw_data: &mut RwData<'a, 'b>) -> bool {
        enum Request {
            FromPipeline(LayoutControlMsg),
            FromScript(Msg),
            FromFontCache,
        }

        let request = {
            let port_from_script = &self.port;
            let port_from_pipeline = &self.pipeline_port;
            let port_from_font_cache = &self.font_cache_receiver;
            select! {
                msg = port_from_pipeline.recv() => {
                    Request::FromPipeline(msg.unwrap())
                },
                msg = port_from_script.recv() => {
                    Request::FromScript(msg.unwrap())
                },
                msg = port_from_font_cache.recv() => {
                    msg.unwrap();
                    Request::FromFontCache
                }
            }
        };

        match request {
            Request::FromPipeline(LayoutControlMsg::SetScrollStates(new_scroll_states)) => {
                self.handle_request_helper(Msg::SetScrollStates(new_scroll_states),
                                           possibly_locked_rw_data)
            },
            Request::FromPipeline(LayoutControlMsg::TickAnimations) => {
                self.handle_request_helper(Msg::TickAnimations, possibly_locked_rw_data)
            },
            Request::FromPipeline(LayoutControlMsg::GetCurrentEpoch(sender)) => {
                self.handle_request_helper(Msg::GetCurrentEpoch(sender), possibly_locked_rw_data)
            },
            Request::FromPipeline(LayoutControlMsg::GetWebFontLoadState(sender)) => {
                self.handle_request_helper(Msg::GetWebFontLoadState(sender),
                                           possibly_locked_rw_data)
            },
            Request::FromPipeline(LayoutControlMsg::ExitNow) => {
                self.handle_request_helper(Msg::ExitNow, possibly_locked_rw_data)
            },
            Request::FromPipeline(LayoutControlMsg::PaintMetric(epoch, paint_time)) => {
                self.paint_time_metrics.maybe_set_metric(epoch, paint_time);
                true
            },
            Request::FromScript(msg) => {
                self.handle_request_helper(msg, possibly_locked_rw_data)
            },
            Request::FromFontCache => {
                let _rw_data = possibly_locked_rw_data.lock();
                self.outstanding_web_fonts.fetch_sub(1, Ordering::SeqCst);
                font_context::invalidate_font_caches();
                self.script_chan.send(ConstellationControlMsg::WebFontLoaded(self.id)).unwrap();
                true
            },
        }
    }

    /// Receives and dispatches messages from other threads.
    fn handle_request_helper<'a, 'b>(
        &mut self,
        request: Msg,
        possibly_locked_rw_data: &mut RwData<'a, 'b>,
    ) -> bool {
        match request {
            Msg::AddStylesheet(stylesheet, before_stylesheet) => {
                let guard = stylesheet.shared_lock.read();
                self.handle_add_stylesheet(&stylesheet, &guard);

                match before_stylesheet {
                    Some(insertion_point) => {
                        self.stylist.insert_stylesheet_before(
                            DocumentStyleSheet(stylesheet.clone()),
                            DocumentStyleSheet(insertion_point),
                            &guard,
                        )
                    }
                    None => {
                        self.stylist.append_stylesheet(
                            DocumentStyleSheet(stylesheet.clone()),
                            &guard,
                        )
                    }
                }
            }
            Msg::RemoveStylesheet(stylesheet) => {
                let guard = stylesheet.shared_lock.read();
                self.stylist.remove_stylesheet(
                    DocumentStyleSheet(stylesheet.clone()),
                    &guard,
                );
            }
            Msg::SetQuirksMode(mode) => self.handle_set_quirks_mode(mode),
            Msg::GetRPC(response_chan) => {
                response_chan.send(
                    Box::new(LayoutRPCImpl(self.rw_data.clone())) as Box<LayoutRPC + Send>
                ).unwrap();
            },
            Msg::Reflow(data) => {
                let mut data = ScriptReflowResult::new(data);
                profile(time::ProfilerCategory::LayoutPerform,
                        self.profiler_metadata(),
                        self.time_profiler_chan.clone(),
                        || self.handle_reflow(&mut data, possibly_locked_rw_data));
            },
            Msg::TickAnimations => self.tick_all_animations(possibly_locked_rw_data),
            Msg::SetScrollStates(new_scroll_states) => {
                self.set_scroll_states(new_scroll_states, possibly_locked_rw_data);
            }
            Msg::UpdateScrollStateFromScript(state) => {
                let mut rw_data = possibly_locked_rw_data.lock();
                rw_data.scroll_offsets.insert(state.scroll_root_id, state.scroll_offset);

                let point = Point2D::new(-state.scroll_offset.x, -state.scroll_offset.y);
                self.webrender_api.scroll_node_with_id(
                    self.webrender_document,
                    webrender_api::LayoutPoint::from_untyped(&point),
                    state.scroll_root_id,
                    webrender_api::ScrollClamping::ToContentBounds
                );
            }
            Msg::ReapStyleAndLayoutData(dead_data) => {
                unsafe {
                    drop_style_and_layout_data(dead_data)
                }
            }
            Msg::CollectReports(reports_chan) => {
                self.collect_reports(reports_chan, possibly_locked_rw_data);
            },
            Msg::GetCurrentEpoch(sender) => {
                let _rw_data = possibly_locked_rw_data.lock();
                sender.send(self.epoch.get()).unwrap();
            },
            Msg::AdvanceClockMs(how_many, do_tick) => {
                self.handle_advance_clock_ms(how_many, possibly_locked_rw_data, do_tick);
            }
            Msg::GetWebFontLoadState(sender) => {
                let _rw_data = possibly_locked_rw_data.lock();
                let outstanding_web_fonts = self.outstanding_web_fonts.load(Ordering::SeqCst);
                sender.send(outstanding_web_fonts != 0).unwrap();
            },
            Msg::CreateLayoutThread(info) => {
                self.create_layout_thread(info)
            }
            Msg::SetFinalUrl(final_url) => {
                self.url = final_url;
            },
            Msg::RegisterPaint(name, mut properties, painter) => {
                debug!("Registering the painter");
                let properties = properties.drain(..)
                    .filter_map(|name| PropertyId::parse(&*name)
                        .ok().map(|id| (name.clone(), id)))
                    .filter(|&(_, ref id)| id.as_shorthand().is_err())
                    .collect();
                let registered_painter = RegisteredPainterImpl {
                    name: name.clone(),
                    properties: properties,
                    painter: painter,
                };
                self.registered_painters.0.insert(name, registered_painter);
            },
            Msg::PrepareToExit(response_chan) => {
                self.prepare_to_exit(response_chan);
                return false
            },
            Msg::ExitNow => {
                debug!("layout: ExitNow received");
                self.exit_now();
                return false
            },
            Msg::SetNavigationStart(time) => {
                self.paint_time_metrics.set_navigation_start(time);
            },
        }

        true
    }

    fn collect_reports<'a, 'b>(&self,
                               reports_chan: ReportsChan,
                               possibly_locked_rw_data: &mut RwData<'a, 'b>) {
        let mut reports = vec![];
        // Servo uses vanilla jemalloc, which doesn't have a
        // malloc_enclosing_size_of function.
        let mut ops = MallocSizeOfOps::new(::servo_allocator::usable_size, None, None);

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

    fn create_layout_thread(&self, info: NewLayoutThreadInfo) {
        LayoutThread::create(info.id,
                             self.top_level_browsing_context_id,
                             info.url.clone(),
                             info.is_parent,
                             info.layout_pair,
                             info.pipeline_port,
                             info.constellation_chan,
                             info.script_chan.clone(),
                             info.image_cache.clone(),
                             self.font_cache_thread.clone(),
                             self.time_profiler_chan.clone(),
                             self.mem_profiler_chan.clone(),
                             info.content_process_shutdown_chan,
                             self.webrender_api.clone_sender(),
                             self.webrender_document,
                             info.layout_threads,
                             info.paint_time_metrics);
    }

    /// Enters a quiescent state in which no new messages will be processed until an `ExitNow` is
    /// received. A pong is immediately sent on the given response channel.
    fn prepare_to_exit(&mut self, response_chan: Sender<()>) {
        response_chan.send(()).unwrap();
        loop {
            match self.port.recv().unwrap() {
                Msg::ReapStyleAndLayoutData(dead_data) => {
                    unsafe {
                        drop_style_and_layout_data(dead_data)
                    }
                }
                Msg::ExitNow => {
                    debug!("layout thread is exiting...");
                    self.exit_now();
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

    /// Shuts down the layout thread now. If there are any DOM nodes left, layout will now (safely)
    /// crash.
    fn exit_now(&mut self) {
        // Drop the root flow explicitly to avoid holding style data, such as
        // rule nodes.  The `Stylist` checks when it is dropped that all rule
        // nodes have been GCed, so we want drop anyone who holds them first.
        self.root_flow.borrow_mut().take();
        // Drop the rayon threadpool if present.
        let _ = self.parallel_traversal.take();
    }

    fn handle_add_stylesheet(
        &self,
        stylesheet: &Stylesheet,
        guard: &SharedRwLockReadGuard,
    ) {
        // Find all font-face rules and notify the font cache of them.
        // GWTODO: Need to handle unloading web fonts.
        if stylesheet.is_effective_for_device(self.stylist.device(), &guard) {
            add_font_face_rules(&*stylesheet,
                                &guard,
                                self.stylist.device(),
                                &self.font_cache_thread,
                                &self.font_cache_sender,
                                &self.outstanding_web_fonts);
        }
    }

    /// Advances the animation clock of the document.
    fn handle_advance_clock_ms<'a, 'b>(&mut self,
                                       how_many_ms: i32,
                                       possibly_locked_rw_data: &mut RwData<'a, 'b>,
                                       tick_animations: bool) {
        self.timer.increment(how_many_ms as f64 / 1000.0);
        if tick_animations {
            self.tick_all_animations(possibly_locked_rw_data);
        }
    }

    /// Sets quirks mode for the document, causing the quirks mode stylesheet to be used.
    fn handle_set_quirks_mode<'a, 'b>(&mut self, quirks_mode: QuirksMode) {
        self.stylist.set_quirks_mode(quirks_mode);
    }

    fn try_get_layout_root<N: LayoutNode>(&self, node: N) -> Option<FlowRef> {
        let result = node.mutate_layout_data()?.flow_construction_result.get();

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

        FlowRef::deref_mut(&mut flow).mark_as_root();

        Some(flow)
    }

    /// Performs layout constraint solving.
    ///
    /// This corresponds to `Reflow()` in Gecko and `layout()` in WebKit/Blink and should be
    /// benchmarked against those two. It is marked `#[inline(never)]` to aid profiling.
    #[inline(never)]
    fn solve_constraints(layout_root: &mut Flow,
                         layout_context: &LayoutContext) {
        let _scope = layout_debug_scope!("solve_constraints");
        sequential::reflow(layout_root, layout_context, RelayoutMode::Incremental);
    }

    /// Performs layout constraint solving in parallel.
    ///
    /// This corresponds to `Reflow()` in Gecko and `layout()` in WebKit/Blink and should be
    /// benchmarked against those two. It is marked `#[inline(never)]` to aid profiling.
    #[inline(never)]
    fn solve_constraints_parallel(traversal: &rayon::ThreadPool,
                                  layout_root: &mut Flow,
                                  profiler_metadata: Option<TimerMetadata>,
                                  time_profiler_chan: time::ProfilerChan,
                                  layout_context: &LayoutContext) {
        let _scope = layout_debug_scope!("solve_constraints_parallel");

        // NOTE: this currently computes borders, so any pruning should separate that
        // operation out.
        parallel::reflow(layout_root,
                         profiler_metadata,
                         time_profiler_chan,
                         layout_context,
                         traversal);
    }

    /// Computes the stacking-relative positions of all flows and, if the painting is dirty and the
    /// reflow type need it, builds the display list.
    fn compute_abs_pos_and_build_display_list(&self,
                                              data: &Reflow,
                                              reflow_goal: &ReflowGoal,
                                              document: Option<&ServoLayoutDocument>,
                                              layout_root: &mut Flow,
                                              layout_context: &mut LayoutContext,
                                              rw_data: &mut LayoutThreadData) {
        let writing_mode = layout_root.base().writing_mode;
        let (metadata, sender) = (self.profiler_metadata(), self.time_profiler_chan.clone());
        profile(time::ProfilerCategory::LayoutDispListBuild,
                metadata.clone(),
                sender.clone(),
                || {
            layout_root.mut_base().stacking_relative_position =
                LogicalPoint::zero(writing_mode).to_physical(writing_mode,
                                                             self.viewport_size).to_vector();

            layout_root.mut_base().clip = data.page_clip_rect;

            let traversal = ComputeStackingRelativePositions { layout_context: layout_context };
            traversal.traverse(layout_root);

            if layout_root.base().restyle_damage.contains(ServoRestyleDamage::REPAINT) ||
                    rw_data.display_list.is_none() {
                if reflow_goal.needs_display_list() {
                    let mut build_state =
                        sequential::build_display_list_for_subtree(layout_root, layout_context);

                    debug!("Done building display list.");

                    let root_size = {
                        let root_flow = layout_root.base();
                        if self.stylist.viewport_constraints().is_some() {
                            root_flow.position.size.to_physical(root_flow.writing_mode)
                        } else {
                            root_flow.overflow.scroll.size
                        }
                    };

                    let origin = Rect::new(Point2D::new(Au(0), Au(0)), root_size);
                    build_state.root_stacking_context.bounds = origin;
                    build_state.root_stacking_context.overflow = origin;

                    if !build_state.iframe_sizes.is_empty() {
                        // build_state.iframe_sizes is only used here, so its okay to replace
                        // it with an empty vector
                        let iframe_sizes = std::mem::replace(&mut build_state.iframe_sizes, vec![]);
                        let msg = ConstellationMsg::IFrameSizes(iframe_sizes);
                        if let Err(e) = self.constellation_chan.send(msg) {
                            warn!("Layout resize to constellation failed ({}).", e);
                        }
                    }

                    rw_data.display_list = Some(Arc::new(build_state.to_display_list()));
                }
            }

            if !reflow_goal.needs_display() {
                // Defer the paint step until the next ForDisplay.
                //
                // We need to tell the document about this so it doesn't
                // incorrectly suppress reflows. See #13131.
                document.expect("No document in a non-display reflow?").needs_paint_from_layout();
                return;
            }
            if let Some(document) = document {
                document.will_paint();
            }
            let display_list = (*rw_data.display_list.as_ref().unwrap()).clone();

            if opts::get().dump_display_list {
                display_list.print();
            }
            if opts::get().dump_display_list_json {
                println!("{}", serde_json::to_string_pretty(&display_list).unwrap());
            }

            debug!("Layout done!");

            // TODO: Avoid the temporary conversion and build webrender sc/dl directly!
            let builder = rw_data.display_list.as_ref().unwrap().convert_to_webrender(self.id);

            let viewport_size = Size2D::new(self.viewport_size.width.to_f32_px(),
                                            self.viewport_size.height.to_f32_px());

            let mut epoch = self.epoch.get();
            epoch.next();
            self.epoch.set(epoch);

            let viewport_size = webrender_api::LayoutSize::from_untyped(&viewport_size);

            // Observe notifications about rendered frames if needed right before
            // sending the display list to WebRender in order to set time related
            // Progressive Web Metrics.
            self.paint_time_metrics.maybe_observe_paint_time(self, epoch, &display_list);

            self.webrender_api.set_display_list(
                self.webrender_document,
                webrender_api::Epoch(epoch.0),
                Some(get_root_flow_background_color(layout_root)),
                viewport_size,
                builder.finalize(),
                true,
                webrender_api::ResourceUpdates::new());
            self.webrender_api.generate_frame(self.webrender_document, None);
        });
    }

    /// The high-level routine that performs layout threads.
    fn handle_reflow<'a, 'b>(&mut self,
                             data: &mut ScriptReflowResult,
                             possibly_locked_rw_data: &mut RwData<'a, 'b>) {
        let document = unsafe { ServoLayoutNode::new(&data.document) };
        let document = document.as_document().unwrap();

        // Parallelize if there's more than 750 objects based on rzambre's suggestion
        // https://github.com/servo/servo/issues/10110
        self.parallel_flag = self.layout_threads > 1 && data.dom_count > 750;
        debug!("layout: received layout request for: {}", self.url);
        debug!("Number of objects in DOM: {}", data.dom_count);
        debug!("layout: parallel? {}", self.parallel_flag);

        let mut rw_data = possibly_locked_rw_data.lock();

        let element = match document.root_element() {
            None => {
                // Since we cannot compute anything, give spec-required placeholders.
                debug!("layout: No root node: bailing");
                match data.reflow_goal {
                    ReflowGoal::ContentBoxQuery(_) => {
                        rw_data.content_box_response = None;
                    },
                    ReflowGoal::ContentBoxesQuery(_) => {
                        rw_data.content_boxes_response = Vec::new();
                    },
                    ReflowGoal::NodesFromPointQuery(..) => {
                        rw_data.nodes_from_point_response = Vec::new();
                    },
                    ReflowGoal::NodeGeometryQuery(_) => {
                        rw_data.client_rect_response = Rect::zero();
                    },
                    ReflowGoal::NodeScrollGeometryQuery(_) => {
                        rw_data.scroll_area_response = Rect::zero();
                    },
                    ReflowGoal::NodeOverflowQuery(_) => {
                        rw_data.overflow_response = NodeOverflowResponse(None);
                    },
                    ReflowGoal::NodeScrollRootIdQuery(_) => {
                        rw_data.scroll_root_id_response = None;
                    },
                    ReflowGoal::ResolvedStyleQuery(_, _, _) => {
                        rw_data.resolved_style_response = String::new();
                    },
                    ReflowGoal::OffsetParentQuery(_) => {
                        rw_data.offset_parent_response = OffsetParentResponse::empty();
                    },
                    ReflowGoal::MarginStyleQuery(_) => {
                        rw_data.margin_style_response = MarginStyleResponse::empty();
                    },
                    ReflowGoal::TextIndexQuery(..) => {
                        rw_data.text_index_response = TextIndexResponse(None);
                    }
                    ReflowGoal::Full | ReflowGoal:: TickAnimations => {}
                }
                return;
            },
            Some(x) => x,
        };

        debug!("layout: processing reflow request for: {:?} ({}) (query={:?})",
               element, self.url, data.reflow_goal);
        trace!("{:?}", ShowSubtree(element.as_node()));

        let initial_viewport = data.window_size.initial_viewport;
        let device_pixel_ratio = data.window_size.device_pixel_ratio;
        let old_viewport_size = self.viewport_size;
        let current_screen_size = Size2D::new(Au::from_f32_px(initial_viewport.width),
                                              Au::from_f32_px(initial_viewport.height));

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
        let sheet_origins_affected_by_device_change =
            self.stylist.set_device(device, &guards);

        self.stylist.force_stylesheet_origins_dirty(sheet_origins_affected_by_device_change);
        self.viewport_size =
            self.stylist.viewport_constraints().map_or(current_screen_size, |constraints| {
                debug!("Viewport constraints: {:?}", constraints);

                // other rules are evaluated against the actual viewport
                Size2D::new(Au::from_f32_px(constraints.size.width),
                            Au::from_f32_px(constraints.size.height))
            });

        let viewport_size_changed = self.viewport_size != old_viewport_size;
        if viewport_size_changed {
            if let Some(constraints) = self.stylist.viewport_constraints() {
                // let the constellation know about the viewport constraints
                rw_data.constellation_chan
                       .send(ConstellationMsg::ViewportConstrained(self.id, constraints.clone()))
                       .unwrap();
            }
            if had_used_viewport_units {
                if let Some(mut data) = element.mutate_data() {
                    data.hint.insert(RestyleHint::recascade_subtree());
                }
            }
        }

        {
            if self.first_reflow.get() {
                debug!("First reflow, rebuilding user and UA rules");
                for stylesheet in &ua_stylesheets.user_or_user_agent_stylesheets {
                    self.stylist.append_stylesheet(stylesheet.clone(), &ua_or_user_guard);
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
                self.stylist.force_stylesheet_origins_dirty(Origin::Author.into());
            }

            self.stylist.flush(&guards, Some(element));
        }

        if viewport_size_changed {
            if let Some(mut flow) = self.try_get_layout_root(element.as_node()) {
                LayoutThread::reflow_all_nodes(FlowRef::deref_mut(&mut flow));
            }
        }

        let restyles = document.drain_pending_restyles();
        debug!("Draining restyles: {}", restyles.len());

        let mut map = SnapshotMap::new();
        let elements_with_snapshot: Vec<_> =
            restyles
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
                }
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

        // Create a layout context for use throughout the following passes.
        let mut layout_context =
            self.build_layout_context(guards.clone(), true, &map);

        let thread_pool = if self.parallel_flag {
            self.parallel_traversal.as_ref()
        } else {
            None
        };

        let traversal = RecalcStyleAndConstructFlows::new(layout_context);
        let token = {
            let shared =
                <RecalcStyleAndConstructFlows as DomTraversal<ServoLayoutElement>>::shared_context(&traversal);
            RecalcStyleAndConstructFlows::pre_traverse(element, shared)
        };

        if token.should_traverse() {
            // Recalculate CSS styles and rebuild flows and fragments.
            profile(time::ProfilerCategory::LayoutStyleRecalc,
                    self.profiler_metadata(),
                    self.time_profiler_chan.clone(),
                    || {
                // Perform CSS selector matching and flow construction.
                driver::traverse_dom::<ServoLayoutElement, RecalcStyleAndConstructFlows>(
                    &traversal,
                    token,
                    thread_pool,
                );
            });
            // TODO(pcwalton): Measure energy usage of text shaping, perhaps?
            let text_shaping_time =
                (font::get_and_reset_text_shaping_performance_counter() as u64) /
                (self.layout_threads as u64);
            time::send_profile_data(time::ProfilerCategory::LayoutTextShaping,
                                    self.profiler_metadata(),
                                    &self.time_profiler_chan,
                                    0,
                                    text_shaping_time,
                                    0,
                                    0);

            // Retrieve the (possibly rebuilt) root flow.
            *self.root_flow.borrow_mut() = self.try_get_layout_root(element.as_node());
        }

        for element in elements_with_snapshot {
            unsafe { element.unset_snapshot_flags() }
        }

        layout_context = traversal.destroy();

        if opts::get().dump_style_tree {
            println!("{:?}", ShowSubtreeDataAndPrimaryValues(element.as_node()));
        }

        if opts::get().dump_rule_tree {
            layout_context.style_context.stylist.rule_tree().dump_stdout(&guards);
        }

        // GC the rule tree if some heuristics are met.
        unsafe { layout_context.style_context.stylist.rule_tree().maybe_gc(); }

        // Perform post-style recalculation layout passes.
        if let Some(mut root_flow) = self.root_flow.borrow().clone() {
            self.perform_post_style_recalc_layout_passes(&mut root_flow,
                                                         &data.reflow_info,
                                                         &data.reflow_goal,
                                                         Some(&document),
                                                         &mut rw_data,
                                                         &mut layout_context);
        }

        self.first_reflow.set(false);
        self.respond_to_query_if_necessary(&data.reflow_goal,
                                           &mut *rw_data,
                                           &mut layout_context,
                                           data.result.borrow_mut().as_mut().unwrap());
    }

    fn respond_to_query_if_necessary(&self,
                                     reflow_goal: &ReflowGoal,
                                     rw_data: &mut LayoutThreadData,
                                     context: &mut LayoutContext,
                                     reflow_result: &mut ReflowComplete) {
        let pending_images = match context.pending_images {
            Some(ref pending) => std_mem::replace(&mut *pending.lock().unwrap(), vec![]),
            None => vec![],
        };
        reflow_result.pending_images = pending_images;

        let newly_transitioning_nodes = match context.newly_transitioning_nodes {
            Some(ref nodes) => std_mem::replace(&mut *nodes.lock().unwrap(), vec![]),
            None => vec![],
        };
        reflow_result.newly_transitioning_nodes = newly_transitioning_nodes;

        let mut root_flow = match self.root_flow.borrow().clone() {
            Some(root_flow) => root_flow,
            None => return,
        };
        let root_flow = FlowRef::deref_mut(&mut root_flow);
        match *reflow_goal {
            ReflowGoal::ContentBoxQuery(node) => {
                let node = unsafe { ServoLayoutNode::new(&node) };
                rw_data.content_box_response = process_content_box_request(node, root_flow);
            },
            ReflowGoal::ContentBoxesQuery(node) => {
                let node = unsafe { ServoLayoutNode::new(&node) };
                rw_data.content_boxes_response = process_content_boxes_request(node, root_flow);
            },
            ReflowGoal::TextIndexQuery(node, point_in_node) => {
                let node = unsafe { ServoLayoutNode::new(&node) };
                let opaque_node = node.opaque();
                let point_in_node = Point2D::new(
                    Au::from_f32_px(point_in_node.x),
                    Au::from_f32_px(point_in_node.y)
                );
                rw_data.text_index_response = TextIndexResponse(
                    rw_data.display_list
                    .as_ref()
                    .expect("Tried to hit test with no display list")
                    .text_index(opaque_node, &point_in_node)
                );
            },
            ReflowGoal::NodeGeometryQuery(node) => {
                let node = unsafe { ServoLayoutNode::new(&node) };
                rw_data.client_rect_response = process_node_geometry_request(node, root_flow);
            },
            ReflowGoal::NodeScrollGeometryQuery(node) => {
                let node = unsafe { ServoLayoutNode::new(&node) };
                rw_data.scroll_area_response = process_node_scroll_area_request(node, root_flow);
            },
            ReflowGoal::NodeOverflowQuery(node) => {
                let node = unsafe { ServoLayoutNode::new(&node) };
                rw_data.overflow_response = process_node_overflow_request(node);
            },
            ReflowGoal::NodeScrollRootIdQuery(node) => {
                let node = unsafe { ServoLayoutNode::new(&node) };
                rw_data.scroll_root_id_response = Some(process_node_scroll_root_id_request(self.id,
                                                                                           node));
            },
            ReflowGoal::ResolvedStyleQuery(node, ref pseudo, ref property) => {
                let node = unsafe { ServoLayoutNode::new(&node) };
                rw_data.resolved_style_response =
                    process_resolved_style_request(context,
                                                   node,
                                                   pseudo,
                                                   property,
                                                   root_flow);
            },
            ReflowGoal::OffsetParentQuery(node) => {
                let node = unsafe { ServoLayoutNode::new(&node) };
                rw_data.offset_parent_response = process_offset_parent_query(node, root_flow);
            },
            ReflowGoal::MarginStyleQuery(node) => {
                let node = unsafe { ServoLayoutNode::new(&node) };
                rw_data.margin_style_response = process_margin_style_query(node);
            },
            ReflowGoal::NodesFromPointQuery(client_point, ref reflow_goal) => {
                let mut flags = match reflow_goal {
                    &NodesFromPointQueryType::Topmost => webrender_api::HitTestFlags::empty(),
                    &NodesFromPointQueryType::All => webrender_api::HitTestFlags::FIND_ALL,
                };

                // The point we get is not relative to the entire WebRender scene, but to this
                // particular pipeline, so we need to tell WebRender about that.
                flags.insert(webrender_api::HitTestFlags::POINT_RELATIVE_TO_PIPELINE_VIEWPORT);

                let client_point = webrender_api::WorldPoint::from_untyped(&client_point);
                let results = self.webrender_api.hit_test(
                    self.webrender_document,
                    Some(self.id.to_webrender()),
                    client_point,
                    flags
                );

                rw_data.nodes_from_point_response = results.items.iter()
                   .map(|item| UntrustedNodeAddress(item.tag.0 as *const c_void))
                   .collect()
            },

            ReflowGoal::Full | ReflowGoal::TickAnimations => {}
        }
    }

    fn set_scroll_states<'a, 'b>(&mut self,
                                 new_scroll_states: Vec<ScrollState>,
                                 possibly_locked_rw_data: &mut RwData<'a, 'b>) {
        let mut rw_data = possibly_locked_rw_data.lock();
        let mut script_scroll_states = vec![];
        let mut layout_scroll_states = HashMap::new();
        for new_scroll_state in &new_scroll_states {
            let offset = new_scroll_state.scroll_offset;
            layout_scroll_states.insert(new_scroll_state.scroll_root_id, offset);

            if new_scroll_state.scroll_root_id.is_root_scroll_node() {
                script_scroll_states.push((UntrustedNodeAddress::from_id(0), offset))
            } else if let Some(id) = new_scroll_state.scroll_root_id.external_id() {
                if let Some(node_id) = node_id_from_clip_id(id as usize) {
                    script_scroll_states.push((UntrustedNodeAddress::from_id(node_id), offset))
                }
            }
        }
        let _ = self.script_chan
                    .send(ConstellationControlMsg::SetScrollState(self.id, script_scroll_states));
        rw_data.scroll_offsets = layout_scroll_states
    }

    fn tick_all_animations<'a, 'b>(&mut self, possibly_locked_rw_data: &mut RwData<'a, 'b>) {
        let mut rw_data = possibly_locked_rw_data.lock();
        self.tick_animations(&mut rw_data);
    }

    fn tick_animations(&mut self, rw_data: &mut LayoutThreadData) {
        if opts::get().relayout_event {
            println!("**** pipeline={}\tForDisplay\tSpecial\tAnimationTick", self.id);
        }

        if let Some(mut root_flow) = self.root_flow.borrow().clone() {
            let reflow_info = Reflow {
                page_clip_rect: max_rect(),
            };

            // Unwrap here should not panic since self.root_flow is only ever set to Some(_)
            // in handle_reflow() where self.document_shared_lock is as well.
            let author_shared_lock = self.document_shared_lock.clone().unwrap();
            let author_guard = author_shared_lock.read();
            let ua_or_user_guard = UA_STYLESHEETS.shared_lock.read();
            let guards = StylesheetGuards {
                author: &author_guard,
                ua_or_user: &ua_or_user_guard,
            };
            let snapshots = SnapshotMap::new();
            let mut layout_context = self.build_layout_context(guards,
                                                               false,
                                                               &snapshots);

            {
                // Perform an abbreviated style recalc that operates without access to the DOM.
                let animations = self.running_animations.read();
                profile(time::ProfilerCategory::LayoutStyleRecalc,
                        self.profiler_metadata(),
                        self.time_profiler_chan.clone(),
                        || {
                            animation::recalc_style_for_animations(
                                &layout_context, FlowRef::deref_mut(&mut root_flow), &animations)
                        });
            }
            self.perform_post_style_recalc_layout_passes(&mut root_flow,
                                                         &reflow_info,
                                                         &ReflowGoal::TickAnimations,
                                                         None,
                                                         &mut *rw_data,
                                                         &mut layout_context);
            assert!(layout_context.pending_images.is_none());
            assert!(layout_context.newly_transitioning_nodes.is_none());
        }
    }

    fn perform_post_style_recalc_layout_passes(&self,
                                               root_flow: &mut FlowRef,
                                               data: &Reflow,
                                               reflow_goal: &ReflowGoal,
                                               document: Option<&ServoLayoutDocument>,
                                               rw_data: &mut LayoutThreadData,
                                               context: &mut LayoutContext) {
        {
            let mut newly_transitioning_nodes = context
                .newly_transitioning_nodes
                .as_ref()
                .map(|nodes| nodes.lock().unwrap());
            let newly_transitioning_nodes = newly_transitioning_nodes
                .as_mut()
                .map(|nodes| &mut **nodes);
            // Kick off animations if any were triggered, expire completed ones.
            animation::update_animation_state(&self.constellation_chan,
                                              &self.script_chan,
                                              &mut *self.running_animations.write(),
                                              &mut *self.expired_animations.write(),
                                              newly_transitioning_nodes,
                                              &self.new_animations_receiver,
                                              self.id,
                                              &self.timer);
        }

        profile(time::ProfilerCategory::LayoutRestyleDamagePropagation,
                self.profiler_metadata(),
                self.time_profiler_chan.clone(),
                || {
            // Call `compute_layout_damage` even in non-incremental mode, because it sets flags
            // that are needed in both incremental and non-incremental traversals.
            let damage = FlowRef::deref_mut(root_flow).compute_layout_damage();

            if opts::get().nonincremental_layout || damage.contains(SpecialRestyleDamage::REFLOW_ENTIRE_DOCUMENT) {
                FlowRef::deref_mut(root_flow).reflow_entire_document()
            }
        });

        if opts::get().trace_layout {
            layout_debug::begin_trace(root_flow.clone());
        }

        // Resolve generated content.
        profile(time::ProfilerCategory::LayoutGeneratedContent,
                self.profiler_metadata(),
                self.time_profiler_chan.clone(),
                || sequential::resolve_generated_content(FlowRef::deref_mut(root_flow), &context));

        // Guess float placement.
        profile(time::ProfilerCategory::LayoutFloatPlacementSpeculation,
                self.profiler_metadata(),
                self.time_profiler_chan.clone(),
                || sequential::guess_float_placement(FlowRef::deref_mut(root_flow)));

        // Perform the primary layout passes over the flow tree to compute the locations of all
        // the boxes.
        if root_flow.base().restyle_damage.intersects(ServoRestyleDamage::REFLOW |
                                                              ServoRestyleDamage::REFLOW_OUT_OF_FLOW) {
            profile(time::ProfilerCategory::LayoutMain,
                    self.profiler_metadata(),
                    self.time_profiler_chan.clone(),
                    || {
                let profiler_metadata = self.profiler_metadata();

                if let (true, Some(traversal)) = (self.parallel_flag, self.parallel_traversal.as_ref()) {
                    // Parallel mode.
                    LayoutThread::solve_constraints_parallel(traversal,
                                                             FlowRef::deref_mut(root_flow),
                                                             profiler_metadata,
                                                             self.time_profiler_chan.clone(),
                                                             &*context);
                } else {
                    //Sequential mode
                    LayoutThread::solve_constraints(FlowRef::deref_mut(root_flow), &context)
                }
            });
        }

        profile(time::ProfilerCategory::LayoutStoreOverflow,
                self.profiler_metadata(),
                self.time_profiler_chan.clone(),
                || {
            sequential::store_overflow(context,
                                       FlowRef::deref_mut(root_flow) as &mut Flow);
        });

        self.perform_post_main_layout_passes(data,
                                             root_flow,
                                             reflow_goal,
                                             document,
                                             rw_data,
                                             context);
    }

    fn perform_post_main_layout_passes(&self,
                                       data: &Reflow,
                                       mut root_flow: &mut FlowRef,
                                       reflow_goal: &ReflowGoal,
                                       document: Option<&ServoLayoutDocument>,
                                       rw_data: &mut LayoutThreadData,
                                       layout_context: &mut LayoutContext) {
        // Build the display list if necessary, and send it to the painter.
        self.compute_abs_pos_and_build_display_list(data,
                                                    reflow_goal,
                                                    document,
                                                    FlowRef::deref_mut(&mut root_flow),
                                                    &mut *layout_context,
                                                    rw_data);

        if opts::get().trace_layout {
            layout_debug::end_trace(self.generation.get());
        }

        if opts::get().dump_flow_tree {
            root_flow.print("Post layout flow tree".to_owned());
        }

        self.generation.set(self.generation.get() + 1);
    }

    fn reflow_all_nodes(flow: &mut Flow) {
        debug!("reflowing all nodes!");
        flow.mut_base()
            .restyle_damage
            .insert(ServoRestyleDamage::REPAINT | ServoRestyleDamage::STORE_OVERFLOW |
                    ServoRestyleDamage::REFLOW | ServoRestyleDamage::REPOSITION);

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
fn get_root_flow_background_color(flow: &mut Flow) -> webrender_api::ColorF {
    let transparent = webrender_api::ColorF { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
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
    let color = kid_block_flow.fragment
                  .style
                  .resolve_color(kid_block_flow.fragment.style.get_background().background_color);
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
        filename: &'static str,
    ) -> Result<DocumentStyleSheet, &'static str> {
        let res = read_resource_file(filename).map_err(|_| filename)?;
        Ok(DocumentStyleSheet(ServoArc::new(Stylesheet::from_bytes(
            &res,
            ServoUrl::parse(&format!("chrome://resources/{:?}", filename)).unwrap(),
            None,
            None,
            Origin::UserAgent,
            MediaList::empty(),
            shared_lock.clone(),
            None,
            &NullReporter,
            QuirksMode::NoQuirks,
        ))))
    }

    let shared_lock = SharedRwLock::new();
    let mut user_or_user_agent_stylesheets = vec!();
    // FIXME: presentational-hints.css should be at author origin with zero specificity.
    //        (Does it make a difference?)
    for &filename in &["user-agent.css", "servo.css", "presentational-hints.css"] {
        user_or_user_agent_stylesheets.push(parse_ua_stylesheet(&shared_lock, filename)?);
    }
    for &(ref contents, ref url) in &opts::get().user_stylesheets {
        user_or_user_agent_stylesheets.push(
            DocumentStyleSheet(ServoArc::new(Stylesheet::from_bytes(
                &contents,
                url.clone(),
                None,
                None,
                Origin::User,
                MediaList::empty(),
                shared_lock.clone(),
                None,
                &RustLogReporter,
                QuirksMode::NoQuirks,
            )))
        );
    }

    let quirks_mode_stylesheet = parse_ua_stylesheet(&shared_lock, "quirks-mode.css")?;

    Ok(UserAgentStylesheets {
        shared_lock: shared_lock,
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
            }
        }
    };
}

struct RegisteredPainterImpl {
    painter: Box<Painter>,
    name: Atom,
    properties: FnvHashMap<Atom, PropertyId>,
}

impl SpeculativePainter for RegisteredPainterImpl {
    fn speculatively_draw_a_paint_image(&self, properties: Vec<(Atom, String)>, arguments: Vec<String>) {
        self.painter.speculatively_draw_a_paint_image(properties, arguments);
    }
}

impl RegisteredSpeculativePainter for RegisteredPainterImpl {
    fn properties(&self) -> &FnvHashMap<Atom, PropertyId> {
        &self.properties
    }
    fn name(&self) -> Atom {
        self.name.clone()
    }
}

impl Painter for RegisteredPainterImpl {
    fn draw_a_paint_image(&self,
                          size: TypedSize2D<f32, CSSPixel>,
                          device_pixel_ratio: TypedScale<f32, CSSPixel, DevicePixel>,
                          properties: Vec<(Atom, String)>,
                          arguments: Vec<String>)
                          -> Result<DrawAPaintImageResult, PaintWorkletError>
    {
        self.painter.draw_a_paint_image(size, device_pixel_ratio, properties, arguments)
    }
}

impl RegisteredPainter for RegisteredPainterImpl {}

struct RegisteredPaintersImpl(FnvHashMap<Atom, RegisteredPainterImpl>);

impl RegisteredSpeculativePainters for RegisteredPaintersImpl  {
    fn get(&self, name: &Atom) -> Option<&RegisteredSpeculativePainter> {
        self.0.get(&name).map(|painter| painter as &RegisteredSpeculativePainter)
    }
}

impl RegisteredPainters for RegisteredPaintersImpl {
    fn get(&self, name: &Atom) -> Option<&RegisteredPainter> {
        self.0.get(&name).map(|painter| painter as &RegisteredPainter)
    }
}
