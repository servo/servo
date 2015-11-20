/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The layout task. Performs layout on the DOM, builds display lists and sends them to be
//! painted.

#![allow(unsafe_code)]

use animation;
use app_units::Au;
use azure::azure::AzColor;
use canvas_traits::CanvasMsg;
use construct::ConstructionResult;
use context::{SharedLayoutContext, StylistWrapper, heap_size_of_local_context};
use data::LayoutDataWrapper;
use display_list_builder::ToGfxColor;
use euclid::Matrix4;
use euclid::point::Point2D;
use euclid::rect::Rect;
use euclid::scale_factor::ScaleFactor;
use euclid::size::Size2D;
use flow::{self, Flow, ImmutableFlowUtils, MutableFlowUtils, MutableOwnedFlowUtils};
use flow_ref::{self, FlowRef};
use fnv::FnvHasher;
use gfx::display_list::{ClippingRegion, DisplayList, LayerInfo, OpaqueNode, StackingContext};
use gfx::font_cache_task::FontCacheTask;
use gfx::font_context;
use gfx::paint_task::{LayoutToPaintMsg, PaintLayer};
use gfx_traits::color;
use incremental::{LayoutDamageComputation, REFLOW, REFLOW_ENTIRE_DOCUMENT, REPAINT};
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use layout_debug;
use layout_traits::LayoutTaskFactory;
use log;
use msg::compositor_msg::{Epoch, LayerId, ScrollPolicy};
use msg::constellation_msg::ScriptMsg as ConstellationMsg;
use msg::constellation_msg::{ConstellationChan, Failure, PipelineId};
use net_traits::image_cache_task::{ImageCacheChan, ImageCacheResult, ImageCacheTask};
use parallel::{self, WorkQueueData};
use profile_traits::mem::{self, Report, ReportKind, ReportsChan};
use profile_traits::time::{TimerMetadataFrameType, TimerMetadataReflowType};
use profile_traits::time::{self, TimerMetadata, profile};
use query::{LayoutRPCImpl, process_content_box_request, process_content_boxes_request};
use query::{process_node_geometry_request, process_offset_parent_query, process_resolved_style_request};
use script::dom::node::LayoutData;
use script::layout_interface::Animation;
use script::layout_interface::{LayoutRPC, OffsetParentResponse};
use script::layout_interface::{Msg, NewLayoutTaskInfo, Reflow, ReflowGoal, ReflowQueryType};
use script::layout_interface::{ScriptLayoutChan, ScriptReflow};
use script_traits::{ConstellationControlMsg, LayoutControlMsg, OpaqueScriptLayoutChannel};
use sequential;
use serde_json;
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::collections::hash_state::DefaultState;
use std::mem::transmute;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::{Arc, Mutex, MutexGuard};
use style::computed_values::{filter, mix_blend_mode};
use style::media_queries::{Device, MediaType};
use style::selector_matching::{Stylist, USER_OR_USER_AGENT_STYLESHEETS};
use style::stylesheets::{CSSRuleIteratorExt, Stylesheet};
use url::Url;
use util::geometry::MAX_RECT;
use util::ipc::OptionalIpcSender;
use util::logical_geometry::LogicalPoint;
use util::mem::HeapSizeOf;
use util::opts;
use util::task;
use util::task_state;
use util::workqueue::WorkQueue;
use wrapper::{LayoutDocument, LayoutElement, LayoutNode, ServoLayoutNode};

/// The number of screens of data we're allowed to generate display lists for in each direction.
pub const DISPLAY_PORT_SIZE_FACTOR: i32 = 8;

/// The number of screens we have to traverse before we decide to generate new display lists.
const DISPLAY_PORT_THRESHOLD_SIZE_FACTOR: i32 = 4;

/// Mutable data belonging to the LayoutTask.
///
/// This needs to be protected by a mutex so we can do fast RPCs.
pub struct LayoutTaskData {
    /// The channel on which messages can be sent to the constellation.
    pub constellation_chan: ConstellationChan<ConstellationMsg>,

    /// The root stacking context.
    pub stacking_context: Option<Arc<StackingContext>>,

    /// Performs CSS selector matching and style resolution.
    pub stylist: Box<Stylist>,

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
}

/// Information needed by the layout task.
pub struct LayoutTask {
    /// The ID of the pipeline that we belong to.
    id: PipelineId,

    /// The URL of the pipeline that we belong to.
    url: Url,

    /// Is the current reflow of an iframe, as opposed to a root window?
    is_iframe: bool,

    /// The port on which we receive messages from the script task.
    port: Receiver<Msg>,

    /// The port on which we receive messages from the constellation.
    pipeline_port: Receiver<LayoutControlMsg>,

    /// The port on which we receive messages from the image cache
    image_cache_receiver: Receiver<ImageCacheResult>,

    /// The channel on which the image cache can send messages to ourself.
    image_cache_sender: ImageCacheChan,

    /// The port on which we receive messages from the font cache task.
    font_cache_receiver: Receiver<()>,

    /// The channel on which the font cache can send messages to us.
    font_cache_sender: IpcSender<()>,

    /// The channel on which messages can be sent to the constellation.
    constellation_chan: ConstellationChan<ConstellationMsg>,

    /// The channel on which messages can be sent to the script task.
    script_chan: IpcSender<ConstellationControlMsg>,

    /// The channel on which messages can be sent to the painting task.
    paint_chan: OptionalIpcSender<LayoutToPaintMsg>,

    /// The channel on which messages can be sent to the time profiler.
    time_profiler_chan: time::ProfilerChan,

    /// The channel on which messages can be sent to the memory profiler.
    mem_profiler_chan: mem::ProfilerChan,

    /// The channel on which messages can be sent to the image cache.
    image_cache_task: ImageCacheTask,

    /// Public interface to the font cache task.
    font_cache_task: FontCacheTask,

    /// Is this the first reflow in this LayoutTask?
    first_reflow: bool,

    /// To receive a canvas renderer associated to a layer, this message is propagated
    /// to the paint chan
    canvas_layers_receiver: Receiver<(LayerId, IpcSender<CanvasMsg>)>,
    canvas_layers_sender: Sender<(LayerId, IpcSender<CanvasMsg>)>,

    /// The workers that we use for parallel operation.
    parallel_traversal: Option<WorkQueue<SharedLayoutContext, WorkQueueData>>,

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

    /// The position and size of the visible rect for each layer. We do not build display lists
    /// for any areas more than `DISPLAY_PORT_SIZE_FACTOR` screens away from this area.
    visible_rects: Arc<HashMap<LayerId, Rect<Au>, DefaultState<FnvHasher>>>,

    /// The list of currently-running animations.
    running_animations: Arc<HashMap<OpaqueNode, Vec<Animation>>>,

    /// A counter for epoch messages
    epoch: Epoch,

    /// The size of the viewport. This may be different from the size of the screen due to viewport
    /// constraints.
    viewport_size: Size2D<Au>,

    /// A mutex to allow for fast, read-only RPC of layout's internal data
    /// structures, while still letting the LayoutTask modify them.
    ///
    /// All the other elements of this struct are read-only.
    rw_data: Arc<Mutex<LayoutTaskData>>,
}

impl LayoutTaskFactory for LayoutTask {
    /// Spawns a new layout task.
    fn create(_phantom: Option<&mut LayoutTask>,
              id: PipelineId,
              url: Url,
              is_iframe: bool,
              chan: OpaqueScriptLayoutChannel,
              pipeline_port: IpcReceiver<LayoutControlMsg>,
              constellation_chan: ConstellationChan<ConstellationMsg>,
              failure_msg: Failure,
              script_chan: IpcSender<ConstellationControlMsg>,
              paint_chan: OptionalIpcSender<LayoutToPaintMsg>,
              image_cache_task: ImageCacheTask,
              font_cache_task: FontCacheTask,
              time_profiler_chan: time::ProfilerChan,
              mem_profiler_chan: mem::ProfilerChan,
              shutdown_chan: IpcSender<()>,
              content_process_shutdown_chan: IpcSender<()>) {
        let ConstellationChan(con_chan) = constellation_chan.clone();
        task::spawn_named_with_send_on_failure(format!("LayoutTask {:?}", id),
                                               task_state::LAYOUT,
                                               move || {
            { // Ensures layout task is destroyed before we send shutdown message
                let sender = chan.sender();
                let layout = LayoutTask::new(id,
                                             url,
                                             is_iframe,
                                             chan.receiver(),
                                             pipeline_port,
                                             constellation_chan,
                                             script_chan,
                                             paint_chan,
                                             image_cache_task,
                                             font_cache_task,
                                             time_profiler_chan,
                                             mem_profiler_chan.clone());

                let reporter_name = format!("layout-reporter-{}", id);
                mem_profiler_chan.run_with_memory_reporting(|| {
                    layout.start();
                }, reporter_name, sender, Msg::CollectReports);
            }
            let _ = shutdown_chan.send(());
            let _ = content_process_shutdown_chan.send(());
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

struct RwData<'a, 'b: 'a> {
    rw_data: &'b Arc<Mutex<LayoutTaskData>>,
    possibly_locked_rw_data: &'a mut Option<MutexGuard<'b, LayoutTaskData>>,
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
                       font_cache_task: &FontCacheTask,
                       font_cache_sender: &IpcSender<()>,
                       outstanding_web_fonts_counter: &Arc<AtomicUsize>) {
    for font_face in stylesheet.effective_rules(&device).font_face() {
        for source in &font_face.sources {
            if opts::get().load_webfonts_synchronously {
                let (sender, receiver) = ipc::channel().unwrap();
                font_cache_task.add_web_font(font_face.family.clone(),
                                             (*source).clone(),
                                             sender);
                receiver.recv().unwrap();
            } else {
                outstanding_web_fonts_counter.fetch_add(1, Ordering::SeqCst);
                font_cache_task.add_web_font(font_face.family.clone(),
                                             (*source).clone(),
                                             (*font_cache_sender).clone());
            }
        }
    }
}

impl LayoutTask {
    /// Creates a new `LayoutTask` structure.
    fn new(id: PipelineId,
           url: Url,
           is_iframe: bool,
           port: Receiver<Msg>,
           pipeline_port: IpcReceiver<LayoutControlMsg>,
           constellation_chan: ConstellationChan<ConstellationMsg>,
           script_chan: IpcSender<ConstellationControlMsg>,
           paint_chan: OptionalIpcSender<LayoutToPaintMsg>,
           image_cache_task: ImageCacheTask,
           font_cache_task: FontCacheTask,
           time_profiler_chan: time::ProfilerChan,
           mem_profiler_chan: mem::ProfilerChan)
           -> LayoutTask {
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

        // Ask the router to proxy IPC messages from the font cache task to the layout thread.
        let (ipc_font_cache_sender, ipc_font_cache_receiver) = ipc::channel().unwrap();
        let font_cache_receiver =
            ROUTER.route_ipc_receiver_to_new_mpsc_receiver(ipc_font_cache_receiver);

        let stylist = box Stylist::new(device);
        let outstanding_web_fonts_counter = Arc::new(AtomicUsize::new(0));
        for stylesheet in &*USER_OR_USER_AGENT_STYLESHEETS {
            add_font_face_rules(stylesheet,
                                &stylist.device,
                                &font_cache_task,
                                &ipc_font_cache_sender,
                                &outstanding_web_fonts_counter);
        }

        LayoutTask {
            id: id,
            url: url,
            is_iframe: is_iframe,
            port: port,
            pipeline_port: pipeline_receiver,
            script_chan: script_chan,
            constellation_chan: constellation_chan.clone(),
            paint_chan: paint_chan,
            time_profiler_chan: time_profiler_chan,
            mem_profiler_chan: mem_profiler_chan,
            image_cache_task: image_cache_task,
            font_cache_task: font_cache_task,
            first_reflow: true,
            image_cache_receiver: image_cache_receiver,
            image_cache_sender: ImageCacheChan(ipc_image_cache_sender),
            font_cache_receiver: font_cache_receiver,
            font_cache_sender: ipc_font_cache_sender,
            canvas_layers_receiver: canvas_layers_receiver,
            canvas_layers_sender: canvas_layers_sender,
            parallel_traversal: parallel_traversal,
            generation: 0,
            new_animations_sender: new_animations_sender,
            new_animations_receiver: new_animations_receiver,
            outstanding_web_fonts: outstanding_web_fonts_counter,
            root_flow: None,
            visible_rects: Arc::new(HashMap::with_hash_state(Default::default())),
            running_animations: Arc::new(HashMap::new()),
            epoch: Epoch(0),
            viewport_size: Size2D::new(Au(0), Au(0)),
            rw_data: Arc::new(Mutex::new(
                LayoutTaskData {
                    constellation_chan: constellation_chan,
                    stacking_context: None,
                    stylist: stylist,
                    content_box_response: Rect::zero(),
                    content_boxes_response: Vec::new(),
                    client_rect_response: Rect::zero(),
                    resolved_style_response: None,
                    offset_parent_response: OffsetParentResponse::empty(),
              })),
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
                                   rw_data: &LayoutTaskData,
                                   screen_size_changed: bool,
                                   url: &Url,
                                   goal: ReflowGoal)
                                   -> SharedLayoutContext {
        SharedLayoutContext {
            image_cache_task: self.image_cache_task.clone(),
            image_cache_sender: Mutex::new(self.image_cache_sender.clone()),
            viewport_size: self.viewport_size.clone(),
            screen_size_changed: screen_size_changed,
            font_cache_task: Mutex::new(self.font_cache_task.clone()),
            canvas_layers_sender: Mutex::new(self.canvas_layers_sender.clone()),
            stylist: StylistWrapper(&*rw_data.stylist),
            url: (*url).clone(),
            visible_rects: self.visible_rects.clone(),
            generation: self.generation,
            new_animations_sender: Mutex::new(self.new_animations_sender.clone()),
            goal: goal,
            running_animations: self.running_animations.clone(),
        }
    }

    /// Receives and dispatches messages from the script and constellation tasks
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
            Request::FromPipeline(LayoutControlMsg::SetVisibleRects(new_visible_rects)) => {
                self.handle_request_helper(Msg::SetVisibleRects(new_visible_rects),
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

        let reflow_info = Reflow {
            goal: ReflowGoal::ForDisplay,
            page_clip_rect: MAX_RECT,
        };

        let mut layout_context = self.build_shared_layout_context(&*rw_data,
                                                                  false,
                                                                  &self.url,
                                                                  reflow_info.goal);

        self.perform_post_style_recalc_layout_passes(&reflow_info,
                                                     &mut *rw_data,
                                                     &mut layout_context);


        true
    }

    /// Receives and dispatches messages from other tasks.
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
            Msg::SetVisibleRects(new_visible_rects) => {
                self.set_visible_rects(new_visible_rects, possibly_locked_rw_data);
            }
            Msg::ReapLayoutData(dead_layout_data) => {
                unsafe {
                    self.handle_reap_layout_data(dead_layout_data)
                }
            }
            Msg::CollectReports(reports_chan) => {
                self.collect_reports(reports_chan, possibly_locked_rw_data);
            },
            Msg::GetCurrentEpoch(sender) => {
                let _rw_data = possibly_locked_rw_data.lock();
                sender.send(self.epoch).unwrap();
            },
            Msg::GetWebFontLoadState(sender) => {
                let _rw_data = possibly_locked_rw_data.lock();
                let outstanding_web_fonts = self.outstanding_web_fonts.load(Ordering::SeqCst);
                sender.send(outstanding_web_fonts != 0).unwrap();
            },
            Msg::CreateLayoutTask(info) => {
                self.create_layout_task(info)
            }
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
        if let Some(ref traversal) = self.parallel_traversal {
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
                                  info.paint_chan.to::<LayoutToPaintMsg>(),
                                  self.image_cache_task.clone(),
                                  self.font_cache_task.clone(),
                                  self.time_profiler_chan.clone(),
                                  self.mem_profiler_chan.clone(),
                                  info.layout_shutdown_chan,
                                  info.content_process_shutdown_chan);
    }

    /// Enters a quiescent state in which no new messages will be processed until an `ExitNow` is
    /// received. A pong is immediately sent on the given response channel.
    fn prepare_to_exit(&mut self, response_chan: Sender<()>) {
        response_chan.send(()).unwrap();
        loop {
            match self.port.recv().unwrap() {
                Msg::ReapLayoutData(dead_layout_data) => {
                    unsafe {
                        self.handle_reap_layout_data(dead_layout_data)
                    }
                }
                Msg::ExitNow => {
                    debug!("layout task is exiting...");
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

    /// Shuts down the layout task now. If there are any DOM nodes left, layout will now (safely)
    /// crash.
    fn exit_now<'a, 'b>(&mut self) {
        if let Some(ref mut traversal) = self.parallel_traversal {
            traversal.shutdown()
        }

        let (response_chan, response_port) = ipc::channel().unwrap();
        self.paint_chan.send(LayoutToPaintMsg::Exit(response_chan)).unwrap();
        response_port.recv().unwrap()
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
                                &self.font_cache_task,
                                &self.font_cache_sender,
                                &self.outstanding_web_fonts);
        }

        possibly_locked_rw_data.block(rw_data);
    }

    /// Sets quirks mode for the document, causing the quirks mode stylesheet to be used.
    fn handle_set_quirks_mode<'a, 'b>(&self, possibly_locked_rw_data: &mut RwData<'a, 'b>) {
        let mut rw_data = possibly_locked_rw_data.lock();
        rw_data.stylist.set_quirks_mode(true);
        possibly_locked_rw_data.block(rw_data);
    }

    fn try_get_layout_root(&self, node: ServoLayoutNode) -> Option<FlowRef> {
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

        flow_ref::deref_mut(&mut flow).mark_as_root();

        Some(flow)
    }

    /// Performs layout constraint solving.
    ///
    /// This corresponds to `Reflow()` in Gecko and `layout()` in WebKit/Blink and should be
    /// benchmarked against those two. It is marked `#[inline(never)]` to aid profiling.
    #[inline(never)]
    fn solve_constraints(layout_root: &mut FlowRef,
                         shared_layout_context: &SharedLayoutContext) {
        let _scope = layout_debug_scope!("solve_constraints");
        sequential::traverse_flow_tree_preorder(layout_root, shared_layout_context);
    }

    /// Performs layout constraint solving in parallel.
    ///
    /// This corresponds to `Reflow()` in Gecko and `layout()` in WebKit/Blink and should be
    /// benchmarked against those two. It is marked `#[inline(never)]` to aid profiling.
    #[inline(never)]
    fn solve_constraints_parallel(traversal: &mut WorkQueue<SharedLayoutContext, WorkQueueData>,
                                  layout_root: &mut FlowRef,
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

    fn compute_abs_pos_and_build_display_list(&mut self,
                                              data: &Reflow,
                                              layout_root: &mut FlowRef,
                                              shared_layout_context: &mut SharedLayoutContext,
                                              rw_data: &mut LayoutTaskData) {
        let writing_mode = flow::base(&**layout_root).writing_mode;
        let (metadata, sender) = (self.profiler_metadata(), self.time_profiler_chan.clone());
        profile(time::ProfilerCategory::LayoutDispListBuild,
                metadata.clone(),
                sender.clone(),
                || {
            flow::mut_base(flow_ref::deref_mut(layout_root)).stacking_relative_position =
                LogicalPoint::zero(writing_mode).to_physical(writing_mode,
                                                             self.viewport_size);

            flow::mut_base(flow_ref::deref_mut(layout_root)).clip =
                ClippingRegion::from_rect(&data.page_clip_rect);

            match (&mut self.parallel_traversal, opts::get().parallel_display_list_building) {
                (&mut Some(ref mut traversal), true) => {
                    parallel::build_display_list_for_subtree(layout_root,
                                                             metadata,
                                                             sender,
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

                let root_background_color = get_root_flow_background_color(
                    flow_ref::deref_mut(layout_root));
                let root_size = {
                    let root_flow = flow::base(&**layout_root);
                    if rw_data.stylist.viewport_constraints().is_some() {
                        root_flow.position.size.to_physical(root_flow.writing_mode)
                    } else {
                        root_flow.overflow.size
                    }
                };
                let mut display_list = box DisplayList::new();
                display_list.append_from(&mut flow::mut_base(flow_ref::deref_mut(layout_root))
                                         .display_list_building_result);

                let origin = Rect::new(Point2D::new(Au(0), Au(0)), root_size);
                let stacking_context = Arc::new(StackingContext::new(display_list,
                                                                     &origin,
                                                                     &origin,
                                                                     0,
                                                                     filter::T::new(Vec::new()),
                                                                     mix_blend_mode::T::normal,
                                                                     Matrix4::identity(),
                                                                     Matrix4::identity(),
                                                                     true,
                                                                     false,
                                                                     None));
                if opts::get().dump_display_list {
                    stacking_context.print("DisplayList".to_owned());
                }
                if opts::get().dump_display_list_json {
                    println!("{}", serde_json::to_string_pretty(&stacking_context).unwrap());
                }

                rw_data.stacking_context = Some(stacking_context.clone());

                let layer_info = LayerInfo::new(layout_root.layer_id(),
                                                ScrollPolicy::Scrollable,
                                                None);
                let paint_layer = PaintLayer::new_with_stacking_context(layer_info,
                                                                        stacking_context,
                                                                        root_background_color);

                debug!("Layout done!");

                self.epoch.next();
                self.paint_chan
                    .send(LayoutToPaintMsg::PaintInit(self.epoch, paint_layer))
                    .unwrap();
            }
        });
    }

    /// The high-level routine that performs layout tasks.
    fn handle_reflow<'a, 'b>(&mut self,
                             data: &ScriptReflow,
                             possibly_locked_rw_data: &mut RwData<'a, 'b>) {
        let document = unsafe { ServoLayoutNode::new(&data.document) };
        let document = document.as_document().unwrap();

        debug!("layout: received layout request for: {}", self.url.serialize());

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
                    ReflowQueryType::NodeGeometryQuery(_) => {
                        rw_data.client_rect_response = Rect::zero();
                    },
                    ReflowQueryType::ResolvedStyleQuery(_, _, _) => {
                        rw_data.resolved_style_response = None;
                    },
                    ReflowQueryType::OffsetParentQuery(_) => {
                        rw_data.offset_parent_response = OffsetParentResponse::empty();
                    },
                    ReflowQueryType::NoQuery => {}
                }
                return;
            },
            Some(x) => x,
        };

        if log_enabled!(log::LogLevel::Debug) {
            node.dump();
        }

        let stylesheets: Vec<&Stylesheet> = data.document_stylesheets.iter().map(|entry| &**entry)
                                                                            .collect();
        let stylesheets_changed = data.stylesheets_changed;
        let initial_viewport = data.window_size.initial_viewport;
        let old_viewport_size = self.viewport_size;
        let current_screen_size = Size2D::new(Au::from_f32_px(initial_viewport.width.get()),
                                              Au::from_f32_px(initial_viewport.height.get()));

        // Calculate the actual viewport as per DEVICE-ADAPT § 6
        let device = Device::new(MediaType::Screen, initial_viewport);
        rw_data.stylist.set_device(device, &stylesheets);

        let constraints = rw_data.stylist.viewport_constraints().clone();
        self.viewport_size = match constraints {
            Some(ref constraints) => {
                debug!("Viewport constraints: {:?}", constraints);

                // other rules are evaluated against the actual viewport
                Size2D::new(Au::from_f32_px(constraints.size.width.get()),
                            Au::from_f32_px(constraints.size.height.get()))
            }
            None => current_screen_size,
        };

        // Handle conditions where the entire flow tree is invalid.
        let viewport_size_changed = self.viewport_size != old_viewport_size;
        if viewport_size_changed {
            if let Some(constraints) = constraints {
                // let the constellation know about the viewport constraints
                let ConstellationChan(ref constellation_chan) = rw_data.constellation_chan;
                constellation_chan.send(ConstellationMsg::ViewportConstrained(
                        self.id, constraints)).unwrap();
            }
        }

        // If the entire flow tree is invalid, then it will be reflowed anyhow.
        let needs_dirtying = rw_data.stylist.update(&stylesheets, stylesheets_changed);
        let needs_reflow = viewport_size_changed && !needs_dirtying;
        unsafe {
            if needs_dirtying {
                LayoutTask::dirty_all_nodes(node);
            }
        }
        if needs_reflow {
            if let Some(mut flow) = self.try_get_layout_root(node) {
                LayoutTask::reflow_all_nodes(flow_ref::deref_mut(&mut flow));
            }
        }

        let modified_elements = document.drain_modified_elements();
        if !needs_dirtying {
            for (el, snapshot) in modified_elements {
                let hint = rw_data.stylist.compute_restyle_hint(&el, &snapshot, el.get_state());
                el.note_restyle_hint(hint);
            }
        }

        // Create a layout context for use throughout the following passes.
        let mut shared_layout_context = self.build_shared_layout_context(&*rw_data,
                                                                         viewport_size_changed,
                                                                         &self.url,
                                                                         data.reflow_info.goal);

        if node.is_dirty() || node.has_dirty_descendants() {
            // Recalculate CSS styles and rebuild flows and fragments.
            profile(time::ProfilerCategory::LayoutStyleRecalc,
                    self.profiler_metadata(),
                    self.time_profiler_chan.clone(),
                    || {
                // Perform CSS selector matching and flow construction.
                match self.parallel_traversal {
                    None => {
                        sequential::traverse_dom_preorder(node, &shared_layout_context);
                    }
                    Some(ref mut traversal) => {
                        parallel::traverse_dom_preorder(node, &shared_layout_context, traversal);
                    }
                }
            });

            // Retrieve the (possibly rebuilt) root flow.
            self.root_flow = self.try_get_layout_root(node);
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

        if let Some(mut root_flow) = self.root_flow.clone() {
            match data.query_type {
                ReflowQueryType::ContentBoxQuery(node) => {
                    let node = unsafe { ServoLayoutNode::new(&node) };
                    rw_data.content_box_response = process_content_box_request(node, &mut root_flow);
                },
                ReflowQueryType::ContentBoxesQuery(node) => {
                    let node = unsafe { ServoLayoutNode::new(&node) };
                    rw_data.content_boxes_response = process_content_boxes_request(node, &mut root_flow);
                },
                ReflowQueryType::NodeGeometryQuery(node) => {
                    let node = unsafe { ServoLayoutNode::new(&node) };
                    rw_data.client_rect_response = process_node_geometry_request(node, &mut root_flow);
                },
                ReflowQueryType::ResolvedStyleQuery(node, ref pseudo, ref property) => {
                    let node = unsafe { ServoLayoutNode::new(&node) };
                    rw_data.resolved_style_response =
                        process_resolved_style_request(node, pseudo, property, &mut root_flow);
                },
                ReflowQueryType::OffsetParentQuery(node) => {
                    let node = unsafe { ServoLayoutNode::new(&node) };
                    rw_data.offset_parent_response = process_offset_parent_query(node, &mut root_flow);
                },
                ReflowQueryType::NoQuery => {}
            }
        }
    }

    fn set_visible_rects<'a, 'b>(&mut self,
                                 new_visible_rects: Vec<(LayerId, Rect<Au>)>,
                                 possibly_locked_rw_data: &mut RwData<'a, 'b>)
                                 -> bool {
        let mut rw_data = possibly_locked_rw_data.lock();

        // First, determine if we need to regenerate the display lists. This will happen if the
        // layers have moved more than `DISPLAY_PORT_THRESHOLD_SIZE_FACTOR` away from their last
        // positions.
        let mut must_regenerate_display_lists = false;
        let mut old_visible_rects = HashMap::with_hash_state(Default::default());
        let inflation_amount =
            Size2D::new(self.viewport_size.width * DISPLAY_PORT_THRESHOLD_SIZE_FACTOR,
                        self.viewport_size.height * DISPLAY_PORT_THRESHOLD_SIZE_FACTOR);
        for &(ref layer_id, ref new_visible_rect) in &new_visible_rects {
            match self.visible_rects.get(layer_id) {
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
            self.visible_rects = Arc::new(old_visible_rects);
            return true
        }

        debug!("regenerating display lists!");
        for &(ref layer_id, ref new_visible_rect) in &new_visible_rects {
            old_visible_rects.insert(*layer_id, *new_visible_rect);
        }
        self.visible_rects = Arc::new(old_visible_rects);

        // Regenerate the display lists.
        let reflow_info = Reflow {
            goal: ReflowGoal::ForDisplay,
            page_clip_rect: MAX_RECT,
        };

        let mut layout_context = self.build_shared_layout_context(&*rw_data,
                                                                  false,
                                                                  &self.url,
                                                                  reflow_info.goal);

        self.perform_post_main_layout_passes(&reflow_info, &mut *rw_data, &mut layout_context);
        true
    }

    fn tick_all_animations<'a, 'b>(&mut self, possibly_locked_rw_data: &mut RwData<'a, 'b>) {
        let mut rw_data = possibly_locked_rw_data.lock();
        self.tick_animations(&mut rw_data);

        self.script_chan
          .send(ConstellationControlMsg::TickAllAnimations(self.id))
          .unwrap();
    }

    pub fn tick_animations(&mut self, rw_data: &mut LayoutTaskData) {
        let reflow_info = Reflow {
            goal: ReflowGoal::ForDisplay,
            page_clip_rect: MAX_RECT,
        };

        let mut layout_context = self.build_shared_layout_context(&*rw_data,
                                                                  false,
                                                                  &self.url,
                                                                  reflow_info.goal);

        if let Some(mut root_flow) = self.root_flow.clone() {
            // Perform an abbreviated style recalc that operates without access to the DOM.
            let animations = &*self.running_animations;
            profile(time::ProfilerCategory::LayoutStyleRecalc,
                    self.profiler_metadata(),
                    self.time_profiler_chan.clone(),
                    || {
                        animation::recalc_style_for_animations(flow_ref::deref_mut(&mut root_flow),
                                                               animations)
                    });
        }

        self.perform_post_style_recalc_layout_passes(&reflow_info,
                                                     &mut *rw_data,
                                                     &mut layout_context);
    }

    fn reflow_with_newly_loaded_web_font<'a, 'b>(&mut self, possibly_locked_rw_data: &mut RwData<'a, 'b>) {
        let mut rw_data = possibly_locked_rw_data.lock();
        font_context::invalidate_font_caches();

        let reflow_info = Reflow {
            goal: ReflowGoal::ForDisplay,
            page_clip_rect: MAX_RECT,
        };

        let mut layout_context = self.build_shared_layout_context(&*rw_data,
                                                                  false,
                                                                  &self.url,
                                                                  reflow_info.goal);

        // No need to do a style recalc here.
        if self.root_flow.is_none() {
            return
        }
        self.perform_post_style_recalc_layout_passes(&reflow_info,
                                                     &mut *rw_data,
                                                     &mut layout_context);
    }

    fn perform_post_style_recalc_layout_passes(&mut self,
                                               data: &Reflow,
                                               rw_data: &mut LayoutTaskData,
                                               layout_context: &mut SharedLayoutContext) {
        if let Some(mut root_flow) = self.root_flow.clone() {
            // Kick off animations if any were triggered, expire completed ones.
            animation::update_animation_state(&self.constellation_chan,
                                              &mut self.running_animations,
                                              &self.new_animations_receiver,
                                              self.id);

            profile(time::ProfilerCategory::LayoutRestyleDamagePropagation,
                    self.profiler_metadata(),
                    self.time_profiler_chan.clone(),
                    || {
                if opts::get().nonincremental_layout ||
                        flow_ref::deref_mut(&mut root_flow).compute_layout_damage()
                                                           .contains(REFLOW_ENTIRE_DOCUMENT) {
                    flow_ref::deref_mut(&mut root_flow).reflow_entire_document()
                }
            });

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
                let profiler_metadata = self.profiler_metadata();
                match self.parallel_traversal {
                    None => {
                        // Sequential mode.
                        LayoutTask::solve_constraints(&mut root_flow, &layout_context)
                    }
                    Some(ref mut parallel) => {
                        // Parallel mode.
                        LayoutTask::solve_constraints_parallel(parallel,
                                                               &mut root_flow,
                                                               profiler_metadata,
                                                               self.time_profiler_chan.clone(),
                                                               &*layout_context);
                    }
                }
            });

            self.perform_post_main_layout_passes(data, rw_data, layout_context);
        }
    }

    fn perform_post_main_layout_passes(&mut self,
                                       data: &Reflow,
                                       rw_data: &mut LayoutTaskData,
                                       layout_context: &mut SharedLayoutContext) {
        // Build the display list if necessary, and send it to the painter.
        if let Some(mut root_flow) = self.root_flow.clone() {
            self.compute_abs_pos_and_build_display_list(data,
                                                        &mut root_flow,
                                                        &mut *layout_context,
                                                        rw_data);
            self.first_reflow = false;

            if opts::get().trace_layout {
                layout_debug::end_trace();
            }

            if opts::get().dump_flow_tree {
                root_flow.dump();
            }

            self.generation += 1;
        }
    }

    unsafe fn dirty_all_nodes(node: ServoLayoutNode) {
        for node in node.traverse_preorder() {
            // TODO(cgaebel): mark nodes which are sensitive to media queries as
            // "changed":
            // > node.set_changed(true);
            node.set_dirty(true);
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
        let _: LayoutDataWrapper = transmute(layout_data);
    }

    /// Returns profiling information which is passed to the time profiler.
    fn profiler_metadata(&self) -> Option<TimerMetadata> {
        Some(TimerMetadata {
            url: self.url.serialize(),
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
fn get_root_flow_background_color(flow: &mut Flow) -> AzColor {
    if !flow.is_block_like() {
        return color::transparent()
    }

    let block_flow = flow.as_mut_block();
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
