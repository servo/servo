/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The layout thread. Performs layout on the DOM, builds display lists and sends them to be
//! painted.

#![feature(box_syntax)]
#![feature(mpsc_select)]
#![feature(plugin)]
#![feature(proc_macro)]

#![plugin(plugins)]

extern crate app_units;
extern crate core;
extern crate euclid;
extern crate fnv;
extern crate gfx;
extern crate gfx_traits;
extern crate heapsize;
#[macro_use] extern crate heapsize_derive;
extern crate ipc_channel;
#[macro_use]
extern crate layout;
extern crate layout_traits;
#[allow(unused_extern_crates)]
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate msg;
extern crate net_traits;
extern crate parking_lot;
#[macro_use]
extern crate profile_traits;
extern crate rayon;
extern crate script;
extern crate script_layout_interface;
extern crate script_traits;
extern crate selectors;
extern crate serde_json;
extern crate servo_url;
extern crate style;
extern crate style_traits;
extern crate util;
extern crate webrender_traits;

use app_units::Au;
use euclid::point::Point2D;
use euclid::rect::Rect;
use euclid::scale_factor::ScaleFactor;
use euclid::size::Size2D;
use fnv::FnvHasher;
use gfx::display_list::{ClippingRegion, OpaqueNode};
use gfx::display_list::WebRenderImageInfo;
use gfx::font;
use gfx::font_cache_thread::FontCacheThread;
use gfx::font_context;
use gfx_traits::{Epoch, FragmentType, ScrollRootId};
use heapsize::HeapSizeOf;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use layout::animation;
use layout::construct::ConstructionResult;
use layout::context::{LayoutContext, SharedLayoutContext, heap_size_of_local_context};
use layout::display_list_builder::ToGfxColor;
use layout::flow::{self, Flow, ImmutableFlowUtils, MutableFlowUtils, MutableOwnedFlowUtils};
use layout::flow_ref::FlowRef;
use layout::incremental::{LayoutDamageComputation, REFLOW_ENTIRE_DOCUMENT};
use layout::layout_debug;
use layout::parallel;
use layout::query::{LayoutRPCImpl, LayoutThreadData, process_content_box_request, process_content_boxes_request};
use layout::query::{process_margin_style_query, process_node_overflow_request, process_resolved_style_request};
use layout::query::{process_node_geometry_request, process_node_scroll_area_request};
use layout::query::process_offset_parent_query;
use layout::sequential;
use layout::traversal::{ComputeAbsolutePositions, RecalcStyleAndConstructFlows};
use layout::webrender_helpers::{WebRenderDisplayListConverter, WebRenderFrameBuilder};
use layout::wrapper::LayoutNodeLayoutData;
use layout::wrapper::drop_style_and_layout_data;
use layout_traits::LayoutThreadFactory;
use msg::constellation_msg::PipelineId;
use net_traits::image_cache_thread::{ImageCacheChan, ImageCacheResult, ImageCacheThread};
use net_traits::image_cache_thread::UsePlaceholder;
use parking_lot::RwLock;
use profile_traits::mem::{self, Report, ReportKind, ReportsChan};
use profile_traits::time::{self, TimerMetadata, profile};
use profile_traits::time::{TimerMetadataFrameType, TimerMetadataReflowType};
use script::layout_wrapper::{ServoLayoutDocument, ServoLayoutNode};
use script_layout_interface::message::{Msg, NewLayoutThreadInfo, Reflow, ReflowQueryType, ScriptReflow};
use script_layout_interface::reporter::CSSErrorReporter;
use script_layout_interface::rpc::{LayoutRPC, MarginStyleResponse, NodeOverflowResponse, OffsetParentResponse};
use script_layout_interface::wrapper_traits::LayoutNode;
use script_traits::{ConstellationControlMsg, LayoutControlMsg, LayoutMsg as ConstellationMsg};
use script_traits::{StackingContextScrollState, UntrustedNodeAddress};
use selectors::Element;
use servo_url::ServoUrl;
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::ops::{Deref, DerefMut};
use std::process;
use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use style::animation::Animation;
use style::context::{LocalStyleContextCreationInfo, ReflowGoal, SharedStyleContext};
use style::dom::{TElement, TNode};
use style::error_reporting::{ParseErrorReporter, StdoutErrorReporter};
use style::logical_geometry::LogicalPoint;
use style::media_queries::{Device, MediaType};
use style::parser::ParserContextExtraData;
use style::selector_matching::Stylist;
use style::servo::restyle_damage::{REFLOW, REFLOW_OUT_OF_FLOW, REPAINT, REPOSITION, STORE_OVERFLOW};
use style::stylesheets::{Origin, Stylesheet, UserAgentStylesheets};
use style::thread_state;
use style::timer::Timer;
use util::geometry::max_rect;
use util::opts;
use util::prefs::PREFS;
use util::resource_files::read_resource_file;
use util::thread;

/// Information needed by the layout thread.
pub struct LayoutThread {
    /// The ID of the pipeline that we belong to.
    id: PipelineId,

    /// The URL of the pipeline that we belong to.
    url: ServoUrl,

    /// Is the current reflow of an iframe, as opposed to a root window?
    is_iframe: bool,

    /// The port on which we receive messages from the script thread.
    port: Receiver<Msg>,

    /// The port on which we receive messages from the constellation.
    pipeline_port: Receiver<LayoutControlMsg>,

    /// The port on which we receive messages from the image cache
    image_cache_receiver: Receiver<ImageCacheResult>,

    /// The channel on which the image cache can send messages to ourself.
    image_cache_sender: ImageCacheChan,

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

    /// The channel on which messages can be sent to the image cache.
    image_cache_thread: ImageCacheThread,

    /// Public interface to the font cache thread.
    font_cache_thread: FontCacheThread,

    /// Is this the first reflow in this LayoutThread?
    first_reflow: bool,

    /// The workers that we use for parallel operation.
    parallel_traversal: Option<rayon::ThreadPool>,

    /// Starts at zero, and increased by one every time a layout completes.
    /// This can be used to easily check for invalid stale data.
    generation: u32,

    /// A channel on which new animations that have been triggered by style recalculation can be
    /// sent.
    new_animations_sender: Sender<Animation>,

    /// Receives newly-discovered animations.
    new_animations_receiver: Receiver<Animation>,

    /// The number of Web fonts that have been requested but not yet loaded.
    outstanding_web_fonts: Arc<AtomicUsize>,

    /// The root of the flow tree.
    root_flow: Option<FlowRef>,

    /// The list of currently-running animations.
    running_animations: Arc<RwLock<HashMap<OpaqueNode, Vec<Animation>>>>,

    /// The list of animations that have expired since the last style recalculation.
    expired_animations: Arc<RwLock<HashMap<OpaqueNode, Vec<Animation>>>>,

    /// A counter for epoch messages
    epoch: Epoch,

    /// The size of the viewport. This may be different from the size of the screen due to viewport
    /// constraints.
    viewport_size: Size2D<Au>,

    /// A mutex to allow for fast, read-only RPC of layout's internal data
    /// structures, while still letting the LayoutThread modify them.
    ///
    /// All the other elements of this struct are read-only.
    rw_data: Arc<Mutex<LayoutThreadData>>,

    /// The CSS error reporter for all CSS loaded in this layout thread
    error_reporter: CSSErrorReporter,

    webrender_image_cache: Arc<RwLock<HashMap<(ServoUrl, UsePlaceholder),
                                              WebRenderImageInfo,
                                              BuildHasherDefault<FnvHasher>>>>,

    // Webrender interface.
    webrender_api: webrender_traits::RenderApi,

    /// The timer object to control the timing of the animations. This should
    /// only be a test-mode timer during testing for animations.
    timer: Timer,

    // Number of layout threads. This is copied from `util::opts`, but we'd
    // rather limit the dependency on that module here.
    layout_threads: usize,
}

impl LayoutThreadFactory for LayoutThread {
    type Message = Msg;

    /// Spawns a new layout thread.
    fn create(id: PipelineId,
              url: ServoUrl,
              is_iframe: bool,
              chan: (Sender<Msg>, Receiver<Msg>),
              pipeline_port: IpcReceiver<LayoutControlMsg>,
              constellation_chan: IpcSender<ConstellationMsg>,
              script_chan: IpcSender<ConstellationControlMsg>,
              image_cache_thread: ImageCacheThread,
              font_cache_thread: FontCacheThread,
              time_profiler_chan: time::ProfilerChan,
              mem_profiler_chan: mem::ProfilerChan,
              content_process_shutdown_chan: IpcSender<()>,
              webrender_api_sender: webrender_traits::RenderApiSender,
              layout_threads: usize) {
        thread::spawn_named(format!("LayoutThread {:?}", id),
                      move || {
            thread_state::initialize(thread_state::LAYOUT);
            PipelineId::install(id);
            { // Ensures layout thread is destroyed before we send shutdown message
                let sender = chan.0;
                let layout = LayoutThread::new(id,
                                               url,
                                               is_iframe,
                                               chan.1,
                                               pipeline_port,
                                               constellation_chan,
                                               script_chan,
                                               image_cache_thread,
                                               font_cache_thread,
                                               time_profiler_chan,
                                               mem_profiler_chan.clone(),
                                               webrender_api_sender,
                                               layout_threads);

                let reporter_name = format!("layout-reporter-{}", id);
                mem_profiler_chan.run_with_memory_reporting(|| {
                    layout.start();
                }, reporter_name, sender, Msg::CollectReports);
            }
            let _ = content_process_shutdown_chan.send(());
        });
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
            None    => RWGuard::Used(self.rw_data.lock().unwrap()),
            Some(x) => RWGuard::Held(x),
        }
    }

    /// If no reflow has ever been triggered, this will keep the lock, locked
    /// (and saved in `possibly_locked_rw_data`). If it has been, the lock will
    /// be unlocked.
    fn block(&mut self, rw_data: RWGuard<'b>) {
        match rw_data {
            RWGuard::Used(x) => drop(x),
            RWGuard::Held(x) => *self.possibly_locked_rw_data = Some(x),
        }
    }
}

fn add_font_face_rules(stylesheet: &Stylesheet,
                       device: &Device,
                       font_cache_thread: &FontCacheThread,
                       font_cache_sender: &IpcSender<()>,
                       outstanding_web_fonts_counter: &Arc<AtomicUsize>) {
    if opts::get().load_webfonts_synchronously {
        let (sender, receiver) = ipc::channel().unwrap();
        stylesheet.effective_font_face_rules(&device, |font_face| {
            let effective_sources = font_face.effective_sources();
            font_cache_thread.add_web_font(font_face.family.clone(),
                                           effective_sources,
                                           sender.clone());
            receiver.recv().unwrap();
        })
    } else {
        stylesheet.effective_font_face_rules(&device, |font_face| {
            let effective_sources = font_face.effective_sources();
            outstanding_web_fonts_counter.fetch_add(1, Ordering::SeqCst);
            font_cache_thread.add_web_font(font_face.family.clone(),
                                          effective_sources,
                                          (*font_cache_sender).clone());
        })
    }
}

impl LayoutThread {
    /// Creates a new `LayoutThread` structure.
    fn new(id: PipelineId,
           url: ServoUrl,
           is_iframe: bool,
           port: Receiver<Msg>,
           pipeline_port: IpcReceiver<LayoutControlMsg>,
           constellation_chan: IpcSender<ConstellationMsg>,
           script_chan: IpcSender<ConstellationControlMsg>,
           image_cache_thread: ImageCacheThread,
           font_cache_thread: FontCacheThread,
           time_profiler_chan: time::ProfilerChan,
           mem_profiler_chan: mem::ProfilerChan,
           webrender_api_sender: webrender_traits::RenderApiSender,
           layout_threads: usize)
           -> LayoutThread {
        let device = Device::new(
            MediaType::Screen,
            opts::get().initial_window_size.to_f32() * ScaleFactor::new(1.0));
        let parallel_traversal = if layout_threads != 1 {
            let configuration =
                rayon::Configuration::new().set_num_threads(layout_threads);
            rayon::ThreadPool::new(configuration).ok()
        } else {
            None
        };

        // Create the channel on which new animations can be sent.
        let (new_animations_sender, new_animations_receiver) = channel();

        // Proxy IPC messages from the pipeline to the layout thread.
        let pipeline_receiver = ROUTER.route_ipc_receiver_to_new_mpsc_receiver(pipeline_port);

        // Ask the router to proxy IPC messages from the image cache thread to the layout thread.
        let (ipc_image_cache_sender, ipc_image_cache_receiver) = ipc::channel().unwrap();
        let image_cache_receiver =
            ROUTER.route_ipc_receiver_to_new_mpsc_receiver(ipc_image_cache_receiver);

        // Ask the router to proxy IPC messages from the font cache thread to the layout thread.
        let (ipc_font_cache_sender, ipc_font_cache_receiver) = ipc::channel().unwrap();
        let font_cache_receiver =
            ROUTER.route_ipc_receiver_to_new_mpsc_receiver(ipc_font_cache_receiver);

        let stylist = Arc::new(Stylist::new(device));
        let outstanding_web_fonts_counter = Arc::new(AtomicUsize::new(0));
        for stylesheet in &*UA_STYLESHEETS.user_or_user_agent_stylesheets {
            add_font_face_rules(stylesheet,
                                &stylist.device,
                                &font_cache_thread,
                                &ipc_font_cache_sender,
                                &outstanding_web_fonts_counter);
        }

        LayoutThread {
            id: id,
            url: url,
            is_iframe: is_iframe,
            port: port,
            pipeline_port: pipeline_receiver,
            script_chan: script_chan.clone(),
            constellation_chan: constellation_chan.clone(),
            time_profiler_chan: time_profiler_chan,
            mem_profiler_chan: mem_profiler_chan,
            image_cache_thread: image_cache_thread,
            font_cache_thread: font_cache_thread,
            first_reflow: true,
            image_cache_receiver: image_cache_receiver,
            image_cache_sender: ImageCacheChan(ipc_image_cache_sender),
            font_cache_receiver: font_cache_receiver,
            font_cache_sender: ipc_font_cache_sender,
            parallel_traversal: parallel_traversal,
            generation: 0,
            new_animations_sender: new_animations_sender,
            new_animations_receiver: new_animations_receiver,
            outstanding_web_fonts: outstanding_web_fonts_counter,
            root_flow: None,
            running_animations: Arc::new(RwLock::new(HashMap::new())),
            expired_animations: Arc::new(RwLock::new(HashMap::new())),
            epoch: Epoch(0),
            viewport_size: Size2D::new(Au(0), Au(0)),
            webrender_api: webrender_api_sender.create_api(),
            rw_data: Arc::new(Mutex::new(
                LayoutThreadData {
                    constellation_chan: constellation_chan,
                    display_list: None,
                    stylist: stylist,
                    content_box_response: Rect::zero(),
                    content_boxes_response: Vec::new(),
                    client_rect_response: Rect::zero(),
                    hit_test_response: (None, false),
                    scroll_area_response: Rect::zero(),
                    overflow_response: NodeOverflowResponse(None),
                    resolved_style_response: None,
                    offset_parent_response: OffsetParentResponse::empty(),
                    margin_style_response: MarginStyleResponse::empty(),
                    stacking_context_scroll_offsets: HashMap::new(),
                })),
            error_reporter: CSSErrorReporter {
                pipelineid: id,
                script_chan: Arc::new(Mutex::new(script_chan)),
            },
            webrender_image_cache:
                Arc::new(RwLock::new(HashMap::with_hasher(Default::default()))),
            timer:
                if PREFS.get("layout.animations.test.enabled")
                           .as_boolean().unwrap_or(false) {
                   Timer::test_mode()
                } else {
                    Timer::new()
                },
            layout_threads: layout_threads,
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
    fn build_shared_layout_context(&self,
                                   rw_data: &LayoutThreadData,
                                   screen_size_changed: bool,
                                   goal: ReflowGoal)
                                   -> SharedLayoutContext {
        let local_style_context_creation_data = LocalStyleContextCreationInfo::new(self.new_animations_sender.clone());

        SharedLayoutContext {
            style_context: SharedStyleContext {
                viewport_size: self.viewport_size.clone(),
                screen_size_changed: screen_size_changed,
                stylist: rw_data.stylist.clone(),
                generation: self.generation,
                goal: goal,
                running_animations: self.running_animations.clone(),
                expired_animations: self.expired_animations.clone(),
                error_reporter: self.error_reporter.clone(),
                local_context_creation_data: Mutex::new(local_style_context_creation_data),
                timer: self.timer.clone(),
            },
            image_cache_thread: Mutex::new(self.image_cache_thread.clone()),
            image_cache_sender: Mutex::new(self.image_cache_sender.clone()),
            font_cache_thread: Mutex::new(self.font_cache_thread.clone()),
            webrender_image_cache: self.webrender_image_cache.clone(),
        }
    }

    /// Receives and dispatches messages from the script and constellation threads
    fn handle_request<'a, 'b>(&mut self, possibly_locked_rw_data: &mut RwData<'a, 'b>) -> bool {
        enum Request {
            FromPipeline(LayoutControlMsg),
            FromScript(Msg),
            FromImageCache,
            FromFontCache,
        }

        let request = {
            let port_from_script = &self.port;
            let port_from_pipeline = &self.pipeline_port;
            let port_from_image_cache = &self.image_cache_receiver;
            let port_from_font_cache = &self.font_cache_receiver;
            select! {
                msg = port_from_pipeline.recv() => {
                    Request::FromPipeline(msg.unwrap())
                },
                msg = port_from_script.recv() => {
                    Request::FromScript(msg.unwrap())
                },
                msg = port_from_image_cache.recv() => {
                    msg.unwrap();
                    Request::FromImageCache
                },
                msg = port_from_font_cache.recv() => {
                    msg.unwrap();
                    Request::FromFontCache
                }
            }
        };

        match request {
            Request::FromPipeline(LayoutControlMsg::SetStackingContextScrollStates(
                    new_scroll_states)) => {
                self.handle_request_helper(Msg::SetStackingContextScrollStates(new_scroll_states),
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
            Request::FromScript(msg) => {
                self.handle_request_helper(msg, possibly_locked_rw_data)
            },
            Request::FromImageCache => {
                self.repaint(possibly_locked_rw_data)
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

    /// Repaint the scene, without performing style matching. This is typically
    /// used when an image arrives asynchronously and triggers a relayout and
    /// repaint.
    /// TODO: In the future we could detect if the image size hasn't changed
    /// since last time and avoid performing a complete layout pass.
    fn repaint<'a, 'b>(&mut self, possibly_locked_rw_data: &mut RwData<'a, 'b>) -> bool {
        let mut rw_data = possibly_locked_rw_data.lock();

        if let Some(mut root_flow) = self.root_flow.clone() {
            let flow = flow::mut_base(FlowRef::deref_mut(&mut root_flow));
            flow.restyle_damage.insert(REPAINT);
        }

        let reflow_info = Reflow {
            goal: ReflowGoal::ForDisplay,
            page_clip_rect: max_rect(),
        };
        let mut layout_context = self.build_shared_layout_context(&*rw_data,
                                                                  false,
                                                                  reflow_info.goal);

        self.perform_post_style_recalc_layout_passes(&reflow_info,
                                                     None,
                                                     None,
                                                     &mut *rw_data,
                                                     &mut layout_context);


        true
    }

    /// Receives and dispatches messages from other threads.
    fn handle_request_helper<'a, 'b>(&mut self,
                                     request: Msg,
                                     possibly_locked_rw_data: &mut RwData<'a, 'b>)
                                     -> bool {
        match request {
            Msg::AddStylesheet(style_info) => {
                self.handle_add_stylesheet(style_info, possibly_locked_rw_data)
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
                        || self.handle_reflow(&data, possibly_locked_rw_data));
            },
            Msg::TickAnimations => self.tick_all_animations(possibly_locked_rw_data),
            Msg::ReflowWithNewlyLoadedWebFont => {
                self.reflow_with_newly_loaded_web_font(possibly_locked_rw_data)
            }
            Msg::SetStackingContextScrollStates(new_scroll_states) => {
                self.set_stacking_context_scroll_states(new_scroll_states,
                                                        possibly_locked_rw_data);
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
                sender.send(self.epoch).unwrap();
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
            Msg::PrepareToExit(response_chan) => {
                self.prepare_to_exit(response_chan);
                return false
            },
            Msg::ExitNow => {
                debug!("layout: ExitNow received");
                self.exit_now();
                return false
            }
        }

        true
    }

    fn collect_reports<'a, 'b>(&self,
                               reports_chan: ReportsChan,
                               possibly_locked_rw_data: &mut RwData<'a, 'b>) {
        let mut reports = vec![];

        // FIXME(njn): Just measuring the display tree for now.
        let rw_data = possibly_locked_rw_data.lock();
        let display_list = rw_data.display_list.as_ref();
        let formatted_url = &format!("url({})", self.url);
        reports.push(Report {
            path: path![formatted_url, "layout-thread", "display-list"],
            kind: ReportKind::ExplicitJemallocHeapSize,
            size: display_list.map_or(0, |sc| sc.heap_size_of_children()),
        });

        let stylist = rw_data.stylist.as_ref();
        reports.push(Report {
            path: path![formatted_url, "layout-thread", "stylist"],
            kind: ReportKind::ExplicitJemallocHeapSize,
            size: stylist.heap_size_of_children(),
        });

        // The LayoutThread has a context in TLS...
        reports.push(Report {
            path: path![formatted_url, "layout-thread", "local-context"],
            kind: ReportKind::ExplicitJemallocHeapSize,
            size: heap_size_of_local_context(),
        });

        reports_chan.send(reports);
    }

    fn create_layout_thread(&self, info: NewLayoutThreadInfo) {
        LayoutThread::create(info.id,
                             info.url.clone(),
                             info.is_parent,
                             info.layout_pair,
                             info.pipeline_port,
                             info.constellation_chan,
                             info.script_chan.clone(),
                             self.image_cache_thread.clone(),
                             self.font_cache_thread.clone(),
                             self.time_profiler_chan.clone(),
                             self.mem_profiler_chan.clone(),
                             info.content_process_shutdown_chan,
                             self.webrender_api.clone_sender(),
                             info.layout_threads);
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
        // Drop the rayon threadpool if present.
        let _ = self.parallel_traversal.take();
    }

    fn handle_add_stylesheet<'a, 'b>(&self,
                                     stylesheet: Arc<Stylesheet>,
                                     possibly_locked_rw_data: &mut RwData<'a, 'b>) {
        // Find all font-face rules and notify the font cache of them.
        // GWTODO: Need to handle unloading web fonts.

        let rw_data = possibly_locked_rw_data.lock();
        if stylesheet.is_effective_for_device(&rw_data.stylist.device) {
            add_font_face_rules(&*stylesheet,
                                &rw_data.stylist.device,
                                &self.font_cache_thread,
                                &self.font_cache_sender,
                                &self.outstanding_web_fonts);
        }

        possibly_locked_rw_data.block(rw_data);
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
    fn handle_set_quirks_mode<'a, 'b>(&self, possibly_locked_rw_data: &mut RwData<'a, 'b>) {
        let mut rw_data = possibly_locked_rw_data.lock();
        Arc::get_mut(&mut rw_data.stylist).unwrap().set_quirks_mode(true);
        possibly_locked_rw_data.block(rw_data);
    }

    fn try_get_layout_root<N: LayoutNode>(&self, node: N) -> Option<FlowRef> {
        let mut data = match node.mutate_layout_data() {
            Some(x) => x,
            None => return None,
        };
        let result = data.flow_construction_result.swap_out();

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
                         shared_layout_context: &SharedLayoutContext) {
        let _scope = layout_debug_scope!("solve_constraints");
        sequential::traverse_flow_tree_preorder(layout_root, shared_layout_context);
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
                                  shared_layout_context: &SharedLayoutContext) {
        let _scope = layout_debug_scope!("solve_constraints_parallel");

        // NOTE: this currently computes borders, so any pruning should separate that
        // operation out.
        parallel::traverse_flow_tree_preorder(layout_root,
                                              profiler_metadata,
                                              time_profiler_chan,
                                              shared_layout_context,
                                              traversal);
    }

    /// Computes the stacking-relative positions of all flows and, if the painting is dirty and the
    /// reflow goal and query type need it, builds the display list.
    fn compute_abs_pos_and_build_display_list(&mut self,
                                              data: &Reflow,
                                              query_type: Option<&ReflowQueryType>,
                                              document: Option<&ServoLayoutDocument>,
                                              layout_root: &mut Flow,
                                              shared_layout_context: &mut SharedLayoutContext,
                                              rw_data: &mut LayoutThreadData) {
        let writing_mode = flow::base(layout_root).writing_mode;
        let (metadata, sender) = (self.profiler_metadata(), self.time_profiler_chan.clone());
        profile(time::ProfilerCategory::LayoutDispListBuild,
                metadata.clone(),
                sender.clone(),
                || {
            flow::mut_base(layout_root).stacking_relative_position =
                LogicalPoint::zero(writing_mode).to_physical(writing_mode,
                                                             self.viewport_size);

            flow::mut_base(layout_root).clip =
                ClippingRegion::from_rect(&data.page_clip_rect);

            if flow::base(layout_root).restyle_damage.contains(REPOSITION) {
                layout_root.traverse_preorder(&ComputeAbsolutePositions {
                    layout_context: shared_layout_context
                });
            }

            if flow::base(layout_root).restyle_damage.contains(REPAINT) ||
                    rw_data.display_list.is_none() {
                let display_list_needed = query_type.map(reflow_query_type_needs_display_list)
                                                    .unwrap_or(false);
                match (data.goal, display_list_needed) {
                    (ReflowGoal::ForDisplay, _) | (ReflowGoal::ForScriptQuery, true) => {
                        let mut build_state =
                            sequential::build_display_list_for_subtree(layout_root,
                                                                       shared_layout_context);

                        debug!("Done building display list.");

                        let root_size = {
                            let root_flow = flow::base(layout_root);
                            if rw_data.stylist.viewport_constraints().is_some() {
                                root_flow.position.size.to_physical(root_flow.writing_mode)
                            } else {
                                root_flow.overflow.scroll.size
                            }
                        };

                        let origin = Rect::new(Point2D::new(Au(0), Au(0)), root_size);
                        build_state.root_stacking_context.bounds = origin;
                        build_state.root_stacking_context.overflow = origin;
                        rw_data.display_list = Some(Arc::new(build_state.to_display_list()));
                    }
                    (ReflowGoal::ForScriptQuery, false) => {}
                }
            }

            if data.goal != ReflowGoal::ForDisplay {
                // Defer the paint step until the next ForDisplay.
                //
                // We need to tell the document about this so it doesn't
                // incorrectly suppress reflows. See #13131.
                document.expect("No document in a non-display reflow?")
                        .needs_paint_from_layout();
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
            let pipeline_id = self.id.to_webrender();
            let mut frame_builder = WebRenderFrameBuilder::new(pipeline_id);
            let built_display_list = rw_data.display_list.as_ref().unwrap().convert_to_webrender(
                &mut frame_builder);

            let viewport_size = Size2D::new(self.viewport_size.width.to_f32_px(),
                                            self.viewport_size.height.to_f32_px());

            self.epoch.next();
            let Epoch(epoch_number) = self.epoch;

            self.webrender_api.set_root_display_list(
                get_root_flow_background_color(layout_root),
                webrender_traits::Epoch(epoch_number),
                pipeline_id,
                viewport_size,
                built_display_list,
                frame_builder.auxiliary_lists_builder.finalize());
        });
    }

    /// The high-level routine that performs layout threads.
    fn handle_reflow<'a, 'b>(&mut self,
                             data: &ScriptReflow,
                             possibly_locked_rw_data: &mut RwData<'a, 'b>) {
        let document = unsafe { ServoLayoutNode::new(&data.document) };
        let document = document.as_document().unwrap();

        // FIXME(pcwalton): Combine `ReflowGoal` and `ReflowQueryType`. Then remove this assert.
        debug_assert!((data.reflow_info.goal == ReflowGoal::ForDisplay &&
                       data.query_type == ReflowQueryType::NoQuery) ||
                      (data.reflow_info.goal == ReflowGoal::ForScriptQuery &&
                       data.query_type != ReflowQueryType::NoQuery));

        debug!("layout: received layout request for: {}", self.url);

        let mut rw_data = possibly_locked_rw_data.lock();

        let node: ServoLayoutNode = match document.root_node() {
            None => {
                // Since we cannot compute anything, give spec-required placeholders.
                debug!("layout: No root node: bailing");
                match data.query_type {
                    ReflowQueryType::ContentBoxQuery(_) => {
                        rw_data.content_box_response = Rect::zero();
                    },
                    ReflowQueryType::ContentBoxesQuery(_) => {
                        rw_data.content_boxes_response = Vec::new();
                    },
                    ReflowQueryType::HitTestQuery(..) => {
                        rw_data.hit_test_response = (None, false);
                    },
                    ReflowQueryType::NodeGeometryQuery(_) => {
                        rw_data.client_rect_response = Rect::zero();
                    },
                    ReflowQueryType::NodeScrollGeometryQuery(_) => {
                        rw_data.scroll_area_response = Rect::zero();
                    },
                    ReflowQueryType::NodeOverflowQuery(_) => {
                        rw_data.overflow_response = NodeOverflowResponse(None);
                    },
                    ReflowQueryType::ResolvedStyleQuery(_, _, _) => {
                        rw_data.resolved_style_response = None;
                    },
                    ReflowQueryType::OffsetParentQuery(_) => {
                        rw_data.offset_parent_response = OffsetParentResponse::empty();
                    },
                    ReflowQueryType::MarginStyleQuery(_) => {
                        rw_data.margin_style_response = MarginStyleResponse::empty();
                    },
                    ReflowQueryType::NoQuery => {}
                }
                return;
            },
            Some(x) => x,
        };

        debug!("layout: received layout request for: {}", self.url);
        if log_enabled!(log::LogLevel::Debug) {
            node.dump();
        }

        let initial_viewport = data.window_size.initial_viewport;
        let old_viewport_size = self.viewport_size;
        let current_screen_size = Size2D::new(Au::from_f32_px(initial_viewport.width),
                                              Au::from_f32_px(initial_viewport.height));

        // Calculate the actual viewport as per DEVICE-ADAPT § 6
        let device = Device::new(MediaType::Screen, initial_viewport);
        Arc::get_mut(&mut rw_data.stylist).unwrap().set_device(device, &data.document_stylesheets);

        let constraints = rw_data.stylist.viewport_constraints().clone();
        self.viewport_size = match constraints {
            Some(ref constraints) => {
                debug!("Viewport constraints: {:?}", constraints);

                // other rules are evaluated against the actual viewport
                Size2D::new(Au::from_f32_px(constraints.size.width),
                            Au::from_f32_px(constraints.size.height))
            }
            None => current_screen_size,
        };

        // Handle conditions where the entire flow tree is invalid.
        let mut needs_dirtying = false;

        let viewport_size_changed = self.viewport_size != old_viewport_size;
        if viewport_size_changed {
            if let Some(constraints) = constraints {
                // let the constellation know about the viewport constraints
                rw_data.constellation_chan
                       .send(ConstellationMsg::ViewportConstrained(self.id, constraints))
                       .unwrap();
            }
            if data.document_stylesheets.iter().any(|sheet| sheet.dirty_on_viewport_size_change) {
                let mut iter = node.traverse_preorder();

                let mut next = iter.next();
                while let Some(node) = next {
                    if node.needs_dirty_on_viewport_size_changed() {
                        // NB: The dirty bit is propagated down the tree.
                        unsafe { node.set_dirty(); }

                        if let Some(p) = node.parent_node().and_then(|n| n.as_element()) {
                            unsafe { p.note_dirty_descendant() };
                        }

                        next = iter.next_skipping_children();
                    } else {
                        next = iter.next();
                    }
                }
            }
        }

        // If the entire flow tree is invalid, then it will be reflowed anyhow.
        needs_dirtying |= Arc::get_mut(&mut rw_data.stylist).unwrap().update(&data.document_stylesheets,
                                                                             Some(&*UA_STYLESHEETS),
                                                                             data.stylesheets_changed);
        let needs_reflow = viewport_size_changed && !needs_dirtying;
        unsafe {
            if needs_dirtying {
                // NB: The dirty flag is propagated down during the restyle
                // process.
                node.set_dirty();
            }
        }
        if needs_reflow {
            if let Some(mut flow) = self.try_get_layout_root(node) {
                LayoutThread::reflow_all_nodes(FlowRef::deref_mut(&mut flow));
            }
        }

        let restyles = document.drain_pending_restyles();
        if !needs_dirtying {
            for (el, restyle) in restyles {
                // Propagate the descendant bit up the ancestors. Do this before
                // the restyle calculation so that we can also do it for new
                // unstyled nodes, which the descendants bit helps us find.
                if let Some(parent) = el.parent_element() {
                    unsafe { parent.note_dirty_descendant() };
                }

                if el.get_data().is_none() {
                    // If we haven't styled this node yet, we don't need to track
                    // a restyle.
                    continue;
                }

                // Start with the explicit hint, if any.
                let mut hint = restyle.hint;

                // Expand any snapshots.
                if let Some(s) = restyle.snapshot {
                    hint |= rw_data.stylist.compute_restyle_hint(&el, &s, el.get_state());
                }

                // Apply the cumulative hint.
                if !hint.is_empty() {
                    el.note_restyle_hint::<RecalcStyleAndConstructFlows>(hint);
                }

                // Apply explicit damage, if any.
                if !restyle.damage.is_empty() {
                    let mut d = el.mutate_layout_data().unwrap();
                    d.base.restyle_damage |= restyle.damage;
                }
            }
        }

        // Create a layout context for use throughout the following passes.
        let mut shared_layout_context = self.build_shared_layout_context(&*rw_data,
                                                                         viewport_size_changed,
                                                                         data.reflow_info.goal);

        let el = node.as_element();
        if el.is_some() && (el.unwrap().deprecated_dirty_bit_is_set() || el.unwrap().has_dirty_descendants()) {
            // Recalculate CSS styles and rebuild flows and fragments.
            profile(time::ProfilerCategory::LayoutStyleRecalc,
                    self.profiler_metadata(),
                    self.time_profiler_chan.clone(),
                    || {
                // Perform CSS selector matching and flow construction.
                match self.parallel_traversal {
                    None => {
                        sequential::traverse_dom::<ServoLayoutNode, RecalcStyleAndConstructFlows>(
                            node, &shared_layout_context);
                    }
                    Some(ref mut traversal) => {
                        parallel::traverse_dom::<ServoLayoutNode, RecalcStyleAndConstructFlows>(
                            node, &shared_layout_context, traversal);
                    }
                }
            });

            // TODO(pcwalton): Measure energy usage of text shaping, perhaps?
            let text_shaping_time =
                (font::get_and_reset_text_shaping_performance_counter() as u64) /
                (self.layout_threads as u64);
            time::send_profile_data(time::ProfilerCategory::LayoutTextShaping,
                                    self.profiler_metadata(),
                                    self.time_profiler_chan.clone(),
                                    0,
                                    text_shaping_time,
                                    0,
                                    0);

            // Retrieve the (possibly rebuilt) root flow.
            self.root_flow = self.try_get_layout_root(node);
        }

        if opts::get().dump_style_tree {
            node.dump_style();
        }

        if opts::get().dump_rule_tree {
            shared_layout_context.style_context.stylist.rule_tree.dump_stdout();
        }

        // GC the rule tree if some heuristics are met.
        unsafe { shared_layout_context.style_context.stylist.rule_tree.maybe_gc(); }

        // Perform post-style recalculation layout passes.
        self.perform_post_style_recalc_layout_passes(&data.reflow_info,
                                                     Some(&data.query_type),
                                                     Some(&document),
                                                     &mut rw_data,
                                                     &mut shared_layout_context);

        self.respond_to_query_if_necessary(&data.query_type,
                                           &mut *rw_data,
                                           &mut shared_layout_context);
    }

    fn respond_to_query_if_necessary(&mut self,
                                     query_type: &ReflowQueryType,
                                     rw_data: &mut LayoutThreadData,
                                     shared_layout_context: &mut SharedLayoutContext) {
        let mut root_flow = match self.root_flow.clone() {
            Some(root_flow) => root_flow,
            None => return,
        };
        let root_flow = FlowRef::deref_mut(&mut root_flow);
        match *query_type {
            ReflowQueryType::ContentBoxQuery(node) => {
                let node = unsafe { ServoLayoutNode::new(&node) };
                rw_data.content_box_response = process_content_box_request(node, root_flow);
            },
            ReflowQueryType::ContentBoxesQuery(node) => {
                let node = unsafe { ServoLayoutNode::new(&node) };
                rw_data.content_boxes_response = process_content_boxes_request(node, root_flow);
            },
            ReflowQueryType::HitTestQuery(translated_point, client_point, update_cursor) => {
                let translated_point = Point2D::new(Au::from_f32_px(translated_point.x),
                                                    Au::from_f32_px(translated_point.y));

                let client_point = Point2D::new(Au::from_f32_px(client_point.x),
                                                Au::from_f32_px(client_point.y));

                let result = rw_data.display_list
                                    .as_ref()
                                    .expect("Tried to hit test with no display list")
                                    .hit_test(&translated_point,
                                              &client_point,
                                              &rw_data.stacking_context_scroll_offsets);
                rw_data.hit_test_response = (result.last().cloned(), update_cursor);
            },
            ReflowQueryType::NodeGeometryQuery(node) => {
                let node = unsafe { ServoLayoutNode::new(&node) };
                rw_data.client_rect_response = process_node_geometry_request(node, root_flow);
            },
            ReflowQueryType::NodeScrollGeometryQuery(node) => {
                let node = unsafe { ServoLayoutNode::new(&node) };
                rw_data.scroll_area_response = process_node_scroll_area_request(node, root_flow);
            },
            ReflowQueryType::NodeOverflowQuery(node) => {
                let node = unsafe { ServoLayoutNode::new(&node) };
                rw_data.overflow_response = process_node_overflow_request(node);
            },
            ReflowQueryType::ResolvedStyleQuery(node, ref pseudo, ref property) => {
                let node = unsafe { ServoLayoutNode::new(&node) };
                let layout_context = LayoutContext::new(&shared_layout_context);
                rw_data.resolved_style_response =
                    process_resolved_style_request(node,
                                                   &layout_context,
                                                   pseudo,
                                                   property,
                                                   root_flow);
            },
            ReflowQueryType::OffsetParentQuery(node) => {
                let node = unsafe { ServoLayoutNode::new(&node) };
                rw_data.offset_parent_response = process_offset_parent_query(node, root_flow);
            },
            ReflowQueryType::MarginStyleQuery(node) => {
                let node = unsafe { ServoLayoutNode::new(&node) };
                rw_data.margin_style_response = process_margin_style_query(node);
            },
            ReflowQueryType::NoQuery => {}
        }
    }

    fn set_stacking_context_scroll_states<'a, 'b>(
            &mut self,
            new_scroll_states: Vec<StackingContextScrollState>,
            possibly_locked_rw_data: &mut RwData<'a, 'b>) {
        let mut rw_data = possibly_locked_rw_data.lock();
        let mut script_scroll_states = vec![];
        let mut layout_scroll_states = HashMap::new();
        for new_scroll_state in &new_scroll_states {
            let offset = new_scroll_state.scroll_offset;
            layout_scroll_states.insert(new_scroll_state.scroll_root_id, offset);

            if new_scroll_state.scroll_root_id == ScrollRootId::root() {
                script_scroll_states.push((UntrustedNodeAddress::from_id(0), offset))
            } else if !new_scroll_state.scroll_root_id.is_special() &&
                    new_scroll_state.scroll_root_id.fragment_type() == FragmentType::FragmentBody {
                let id = new_scroll_state.scroll_root_id.id();
                script_scroll_states.push((UntrustedNodeAddress::from_id(id), offset))
            }
        }
        let _ = self.script_chan
                    .send(ConstellationControlMsg::SetScrollState(self.id, script_scroll_states));
        rw_data.stacking_context_scroll_offsets = layout_scroll_states
    }

    fn tick_all_animations<'a, 'b>(&mut self, possibly_locked_rw_data: &mut RwData<'a, 'b>) {
        let mut rw_data = possibly_locked_rw_data.lock();
        self.tick_animations(&mut rw_data);
    }

    fn tick_animations(&mut self, rw_data: &mut LayoutThreadData) {
        let reflow_info = Reflow {
            goal: ReflowGoal::ForDisplay,
            page_clip_rect: max_rect(),
        };

        let mut layout_context = self.build_shared_layout_context(&*rw_data,
                                                                  false,
                                                                  reflow_info.goal);

        if let Some(mut root_flow) = self.root_flow.clone() {
            // Perform an abbreviated style recalc that operates without access to the DOM.
            let animations = self.running_animations.read();
            profile(time::ProfilerCategory::LayoutStyleRecalc,
                    self.profiler_metadata(),
                    self.time_profiler_chan.clone(),
                    || {
                        animation::recalc_style_for_animations(&layout_context,
                                                               FlowRef::deref_mut(&mut root_flow),
                                                               &animations)
                    });
        }

        self.perform_post_style_recalc_layout_passes(&reflow_info,
                                                     None,
                                                     None,
                                                     &mut *rw_data,
                                                     &mut layout_context);
    }

    fn reflow_with_newly_loaded_web_font<'a, 'b>(&mut self, possibly_locked_rw_data: &mut RwData<'a, 'b>) {
        let mut rw_data = possibly_locked_rw_data.lock();
        font_context::invalidate_font_caches();

        let reflow_info = Reflow {
            goal: ReflowGoal::ForDisplay,
            page_clip_rect: max_rect(),
        };

        let mut layout_context = self.build_shared_layout_context(&*rw_data,
                                                                  false,
                                                                  reflow_info.goal);

        // No need to do a style recalc here.
        if self.root_flow.is_none() {
            return
        }
        self.perform_post_style_recalc_layout_passes(&reflow_info,
                                                     None,
                                                     None,
                                                     &mut *rw_data,
                                                     &mut layout_context);
    }

    fn perform_post_style_recalc_layout_passes(&mut self,
                                               data: &Reflow,
                                               query_type: Option<&ReflowQueryType>,
                                               document: Option<&ServoLayoutDocument>,
                                               rw_data: &mut LayoutThreadData,
                                               layout_context: &mut SharedLayoutContext) {
        if let Some(mut root_flow) = self.root_flow.clone() {
            // Kick off animations if any were triggered, expire completed ones.
            animation::update_animation_state(&self.constellation_chan,
                                              &self.script_chan,
                                              &mut *self.running_animations.write(),
                                              &mut *self.expired_animations.write(),
                                              &self.new_animations_receiver,
                                              self.id,
                                              &self.timer);

            profile(time::ProfilerCategory::LayoutRestyleDamagePropagation,
                    self.profiler_metadata(),
                    self.time_profiler_chan.clone(),
                    || {
                // Call `compute_layout_damage` even in non-incremental mode, because it sets flags
                // that are needed in both incremental and non-incremental traversals.
                let damage = FlowRef::deref_mut(&mut root_flow).compute_layout_damage();

                if opts::get().nonincremental_layout || damage.contains(REFLOW_ENTIRE_DOCUMENT) {
                    FlowRef::deref_mut(&mut root_flow).reflow_entire_document()
                }
            });

            if opts::get().trace_layout {
                layout_debug::begin_trace(root_flow.clone());
            }

            // Resolve generated content.
            profile(time::ProfilerCategory::LayoutGeneratedContent,
                    self.profiler_metadata(),
                    self.time_profiler_chan.clone(),
                    || sequential::resolve_generated_content(FlowRef::deref_mut(&mut root_flow), &layout_context));

            // Guess float placement.
            profile(time::ProfilerCategory::LayoutFloatPlacementSpeculation,
                    self.profiler_metadata(),
                    self.time_profiler_chan.clone(),
                    || sequential::guess_float_placement(FlowRef::deref_mut(&mut root_flow)));

            // Perform the primary layout passes over the flow tree to compute the locations of all
            // the boxes.
            if flow::base(&*root_flow).restyle_damage.intersects(REFLOW | REFLOW_OUT_OF_FLOW) {
                profile(time::ProfilerCategory::LayoutMain,
                        self.profiler_metadata(),
                        self.time_profiler_chan.clone(),
                        || {
                    let profiler_metadata = self.profiler_metadata();
                    match self.parallel_traversal {
                        None => {
                            // Sequential mode.
                            LayoutThread::solve_constraints(FlowRef::deref_mut(&mut root_flow), &layout_context)
                        }
                        Some(ref mut parallel) => {
                            // Parallel mode.
                            LayoutThread::solve_constraints_parallel(parallel,
                                                                     FlowRef::deref_mut(&mut root_flow),
                                                                     profiler_metadata,
                                                                     self.time_profiler_chan.clone(),
                                                                     &*layout_context);
                        }
                    }
                });
            }

            profile(time::ProfilerCategory::LayoutStoreOverflow,
                    self.profiler_metadata(),
                    self.time_profiler_chan.clone(),
                    || {
                let layout_context = LayoutContext::new(&*layout_context);
                sequential::store_overflow(&layout_context,
                                           FlowRef::deref_mut(&mut root_flow) as &mut Flow);
            });

            self.perform_post_main_layout_passes(data,
                                                 query_type,
                                                 document,
                                                 rw_data,
                                                 layout_context);
        }
    }

    fn perform_post_main_layout_passes(&mut self,
                                       data: &Reflow,
                                       query_type: Option<&ReflowQueryType>,
                                       document: Option<&ServoLayoutDocument>,
                                       rw_data: &mut LayoutThreadData,
                                       layout_context: &mut SharedLayoutContext) {
        // Build the display list if necessary, and send it to the painter.
        if let Some(mut root_flow) = self.root_flow.clone() {
            self.compute_abs_pos_and_build_display_list(data,
                                                        query_type,
                                                        document,
                                                        FlowRef::deref_mut(&mut root_flow),
                                                        &mut *layout_context,
                                                        rw_data);
            self.first_reflow = false;

            if opts::get().trace_layout {
                layout_debug::end_trace(self.generation);
            }

            if opts::get().dump_flow_tree {
                root_flow.print("Post layout flow tree".to_owned());
            }

            self.generation += 1;
        }
    }

    fn reflow_all_nodes(flow: &mut Flow) {
        debug!("reflowing all nodes!");
        flow::mut_base(flow).restyle_damage.insert(REPAINT | STORE_OVERFLOW | REFLOW);

        for child in flow::child_iter_mut(flow) {
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
            incremental: if self.first_reflow {
                TimerMetadataReflowType::FirstReflow
            } else {
                TimerMetadataReflowType::Incremental
            },
        })
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
fn get_root_flow_background_color(flow: &mut Flow) -> webrender_traits::ColorF {
    let transparent = webrender_traits::ColorF { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
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
    kid_block_flow.fragment
                  .style
                  .resolve_color(kid_block_flow.fragment.style.get_background().background_color)
                  .to_gfx_color()
}

fn get_ua_stylesheets() -> Result<UserAgentStylesheets, &'static str> {
    fn parse_ua_stylesheet(filename: &'static str) -> Result<Stylesheet, &'static str> {
        let res = try!(read_resource_file(filename).map_err(|_| filename));
        Ok(Stylesheet::from_bytes(
            &res,
            ServoUrl::parse(&format!("chrome://resources/{:?}", filename)).unwrap(),
            None,
            None,
            Origin::UserAgent,
            Box::new(StdoutErrorReporter),
            ParserContextExtraData::default()))
    }

    let mut user_or_user_agent_stylesheets = vec!();
    // FIXME: presentational-hints.css should be at author origin with zero specificity.
    //        (Does it make a difference?)
    for &filename in &["user-agent.css", "servo.css", "presentational-hints.css"] {
        user_or_user_agent_stylesheets.push(try!(parse_ua_stylesheet(filename)));
    }
    for &(ref contents, ref url) in &opts::get().user_stylesheets {
        user_or_user_agent_stylesheets.push(Stylesheet::from_bytes(
            &contents, url.clone(), None, None, Origin::User, Box::new(StdoutErrorReporter),
            ParserContextExtraData::default()));
    }

    let quirks_mode_stylesheet = try!(parse_ua_stylesheet("quirks-mode.css"));

    Ok(UserAgentStylesheets {
        user_or_user_agent_stylesheets: user_or_user_agent_stylesheets,
        quirks_mode_stylesheet: quirks_mode_stylesheet,
    })
}

/// Returns true if the given reflow query type needs a full, up-to-date display list to be present
/// or false if it only needs stacking-relative positions.
fn reflow_query_type_needs_display_list(query_type: &ReflowQueryType) -> bool {
    match *query_type {
        ReflowQueryType::HitTestQuery(..) => true,
        ReflowQueryType::ContentBoxQuery(_) | ReflowQueryType::ContentBoxesQuery(_) |
        ReflowQueryType::NodeGeometryQuery(_) | ReflowQueryType::NodeScrollGeometryQuery(_) |
        ReflowQueryType::NodeOverflowQuery(_) | ReflowQueryType::ResolvedStyleQuery(..) |
        ReflowQueryType::OffsetParentQuery(_) | ReflowQueryType::MarginStyleQuery(_) |
        ReflowQueryType::NoQuery => false,
    }
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
