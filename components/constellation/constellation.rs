/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The `Constellation`, Servo's Grand Central Station
//!
//! The primary duty of a `Constellation` is to mediate between the
//! graphics compositor and the many `Pipeline`s in the browser's
//! navigation context, each `Pipeline` encompassing a `ScriptThread`,
//! `LayoutThread`, and `PaintThread`.

use backtrace::Backtrace;
use bluetooth_traits::BluetoothMethodMsg;
use canvas::canvas_paint_thread::CanvasPaintThread;
use canvas::webgl_paint_thread::WebGLPaintThread;
use canvas_traits::CanvasMsg;
use compositing::SendableFrameTree;
use compositing::compositor_thread::CompositorProxy;
use compositing::compositor_thread::Msg as ToCompositorMsg;
use devtools_traits::{ChromeToDevtoolsControlMsg, DevtoolsControlMsg};
use euclid::scale_factor::ScaleFactor;
use euclid::size::{Size2D, TypedSize2D};
use gfx::font_cache_thread::FontCacheThread;
use gfx_traits::Epoch;
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use layout_traits::LayoutThreadFactory;
use log::{Log, LogLevel, LogLevelFilter, LogMetadata, LogRecord};
use msg::constellation_msg::{FrameId, FrameType, PipelineId};
use msg::constellation_msg::{Key, KeyModifiers, KeyState};
use msg::constellation_msg::{PipelineNamespace, PipelineNamespaceId, TraversalDirection};
use net_traits::{self, IpcSend, ResourceThreads};
use net_traits::image_cache_thread::ImageCacheThread;
use net_traits::storage_thread::StorageThreadMsg;
use offscreen_gl_context::{GLContextAttributes, GLLimits};
use pipeline::{ChildProcess, InitialPipelineState, Pipeline};
use profile_traits::mem;
use profile_traits::time;
use rand::{Rng, SeedableRng, StdRng, random};
use script_traits::{AnimationState, AnimationTickType, CompositorEvent};
use script_traits::{ConstellationControlMsg, ConstellationMsg as FromCompositorMsg};
use script_traits::{DocumentState, LayoutControlMsg, LoadData};
use script_traits::{IFrameLoadInfo, IFrameSandboxState, TimerEventRequest};
use script_traits::{LayoutMsg as FromLayoutMsg, ScriptMsg as FromScriptMsg, ScriptThreadFactory};
use script_traits::{LogEntry, ServiceWorkerMsg, webdriver_msg};
use script_traits::{MozBrowserErrorType, MozBrowserEvent, WebDriverCommandMsg, WindowSizeData};
use script_traits::{SWManagerMsg, ScopeThings, WindowSizeType};
use std::borrow::ToOwned;
use std::collections::{HashMap, VecDeque};
use std::io::Error as IOError;
use std::iter::once;
use std::marker::PhantomData;
use std::mem::replace;
use std::process;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::time::Instant;
use style_traits::PagePx;
use style_traits::cursor::Cursor;
use style_traits::viewport::ViewportConstraints;
use timer_scheduler::TimerScheduler;
use url::Url;
use util::opts;
use util::prefs::PREFS;
use util::remutex::ReentrantMutex;
use util::thread::spawn_named;
use webrender_traits;

#[derive(Debug, PartialEq)]
enum ReadyToSave {
    NoRootFrame,
    PendingFrames,
    WebFontNotLoaded,
    DocumentLoading,
    EpochMismatch,
    PipelineUnknown,
    Ready,
}

/// Maintains the pipelines and navigation context and grants permission to composite.
///
/// It is parameterized over a `LayoutThreadFactory` and a
/// `ScriptThreadFactory` (which in practice are implemented by
/// `LayoutThread` in the `layout` crate, and `ScriptThread` in
/// the `script` crate).
pub struct Constellation<Message, LTF, STF> {
    /// A channel through which script messages can be sent to this object.
    script_sender: IpcSender<FromScriptMsg>,

    /// A channel through which layout thread messages can be sent to this object.
    layout_sender: IpcSender<FromLayoutMsg>,

    /// Receives messages from scripts.
    script_receiver: Receiver<FromScriptMsg>,

    /// Receives messages from the compositor
    compositor_receiver: Receiver<FromCompositorMsg>,

    /// Receives messages from the layout thread
    layout_receiver: Receiver<FromLayoutMsg>,

    /// A channel (the implementation of which is port-specific) through which messages can be sent
    /// to the compositor.
    compositor_proxy: Box<CompositorProxy>,

    /// Channels through which messages can be sent to the resource-related threads.
    public_resource_threads: ResourceThreads,

    /// Channels through which messages can be sent to the resource-related threads.
    private_resource_threads: ResourceThreads,

    /// A channel through which messages can be sent to the image cache thread.
    image_cache_thread: ImageCacheThread,

    /// A channel through which messages can be sent to the developer tools.
    devtools_chan: Option<Sender<DevtoolsControlMsg>>,

    /// A channel through which messages can be sent to the bluetooth thread.
    bluetooth_thread: IpcSender<BluetoothMethodMsg>,

    /// Sender to Service Worker Manager thread
    swmanager_chan: Option<IpcSender<ServiceWorkerMsg>>,

    /// to send messages to this object
    swmanager_sender: IpcSender<SWManagerMsg>,

    /// to receive sw manager message
    swmanager_receiver: Receiver<SWManagerMsg>,

    /// A list of all the pipelines. (See the `pipeline` module for more details.)
    pipelines: HashMap<PipelineId, Pipeline>,

    /// A list of all the frames
    frames: HashMap<FrameId, Frame>,

    /// A channel through which messages can be sent to the font cache.
    font_cache_thread: FontCacheThread,

    /// ID of the root frame.
    root_frame_id: FrameId,

    /// The next free ID to assign to a pipeline ID namespace.
    next_pipeline_namespace_id: PipelineNamespaceId,

    /// Pipeline ID that has currently focused element for key events.
    focus_pipeline_id: Option<PipelineId>,

    /// Navigation operations that are in progress.
    pending_frames: Vec<FrameChange>,

    /// A channel through which messages can be sent to the time profiler.
    time_profiler_chan: time::ProfilerChan,

    /// A channel through which messages can be sent to the memory profiler.
    mem_profiler_chan: mem::ProfilerChan,

    phantom: PhantomData<(Message, LTF, STF)>,

    window_size: WindowSizeData,

    /// Bits of state used to interact with the webdriver implementation
    webdriver: WebDriverData,

    scheduler_chan: IpcSender<TimerEventRequest>,

    /// A list of child content processes.
    #[cfg_attr(target_os = "windows", allow(dead_code))]
    child_processes: Vec<ChildProcess>,

    /// Document states for loaded pipelines (used only when writing screenshots).
    document_states: HashMap<PipelineId, DocumentState>,

    // Webrender interface.
    webrender_api_sender: webrender_traits::RenderApiSender,

    /// Are we shutting down?
    shutting_down: bool,

    /// Have we seen any warnings? Hopefully always empty!
    /// The buffer contains `(thread_name, reason)` entries.
    handled_warnings: VecDeque<(Option<String>, String)>,

    /// The random number generator and probability for closing pipelines.
    /// This is for testing the hardening of the constellation.
    random_pipeline_closure: Option<(StdRng, f32)>,
}

/// State needed to construct a constellation.
pub struct InitialConstellationState {
    /// A channel through which messages can be sent to the compositor.
    pub compositor_proxy: Box<CompositorProxy + Send>,
    /// A channel to the developer tools, if applicable.
    pub devtools_chan: Option<Sender<DevtoolsControlMsg>>,
    /// A channel to the bluetooth thread.
    pub bluetooth_thread: IpcSender<BluetoothMethodMsg>,
    /// A channel to the image cache thread.
    pub image_cache_thread: ImageCacheThread,
    /// A channel to the font cache thread.
    pub font_cache_thread: FontCacheThread,
    /// A channel to the resource thread.
    pub public_resource_threads: ResourceThreads,
    /// A channel to the resource thread.
    pub private_resource_threads: ResourceThreads,
    /// A channel to the time profiler thread.
    pub time_profiler_chan: time::ProfilerChan,
    /// A channel to the memory profiler thread.
    pub mem_profiler_chan: mem::ProfilerChan,
    /// Whether the constellation supports the clipboard.
    pub supports_clipboard: bool,
    /// Webrender API.
    pub webrender_api_sender: webrender_traits::RenderApiSender,
}

#[derive(Debug, Clone)]
struct FrameState {
    instant: Instant,
    pipeline_id: PipelineId,
    frame_id: FrameId,
}

impl FrameState {
    fn new(pipeline_id: PipelineId, frame_id: FrameId) -> FrameState {
        FrameState {
            instant: Instant::now(),
            pipeline_id: pipeline_id,
            frame_id: frame_id,
        }
    }
}

/// Stores the navigation context for a single frame in the frame tree.
struct Frame {
    id: FrameId,
    prev: Vec<FrameState>,
    current: FrameState,
    next: Vec<FrameState>,
}

impl Frame {
    fn new(id: FrameId, pipeline_id: PipelineId) -> Frame {
        Frame {
            id: id,
            prev: vec!(),
            current: FrameState::new(pipeline_id, id),
            next: vec!(),
        }
    }

    fn load(&mut self, pipeline_id: PipelineId) {
        self.prev.push(self.current.clone());
        self.current = FrameState::new(pipeline_id, self.id);
    }

    fn remove_forward_entries(&mut self) -> Vec<FrameState> {
        replace(&mut self.next, vec!())
    }

    fn replace_current(&mut self, pipeline_id: PipelineId) -> FrameState {
        replace(&mut self.current, FrameState::new(pipeline_id, self.id))
    }
}

/// Represents a pending change in the frame tree, that will be applied
/// once the new pipeline has loaded and completed initial layout / paint.
struct FrameChange {
    frame_id: FrameId,
    old_pipeline_id: Option<PipelineId>,
    new_pipeline_id: PipelineId,
    document_ready: bool,
    replace: bool,
}

/// An iterator over a frame tree, returning nodes in depth-first order.
/// Note that this iterator should _not_ be used to mutate nodes _during_
/// iteration. Mutating nodes once the iterator is out of scope is OK.
struct FrameTreeIterator<'a> {
    stack: Vec<FrameId>,
    frames: &'a HashMap<FrameId, Frame>,
    pipelines: &'a HashMap<PipelineId, Pipeline>,
}

impl<'a> Iterator for FrameTreeIterator<'a> {
    type Item = &'a Frame;
    fn next(&mut self) -> Option<&'a Frame> {
        loop {
            let frame_id = match self.stack.pop() {
                Some(frame_id) => frame_id,
                None => return None,
            };
            let frame = match self.frames.get(&frame_id) {
                Some(frame) => frame,
                None => {
                    warn!("Frame {:?} iterated after closure.", frame_id);
                    continue;
                },
            };
            let pipeline = match self.pipelines.get(&frame.current.pipeline_id) {
                Some(pipeline) => pipeline,
                None => {
                    warn!("Pipeline {:?} iterated after closure.", frame.current.pipeline_id);
                    continue;
                },
            };
            self.stack.extend(pipeline.children.iter().map(|&c| c));
            return Some(frame)
        }
    }
}

struct FullFrameTreeIterator<'a> {
    stack: Vec<FrameId>,
    frames: &'a HashMap<FrameId, Frame>,
    pipelines: &'a HashMap<PipelineId, Pipeline>,
}

impl<'a> Iterator for FullFrameTreeIterator<'a> {
    type Item = &'a Frame;
    fn next(&mut self) -> Option<&'a Frame> {
        loop {
            let frame_id = match self.stack.pop() {
                Some(frame_id) => frame_id,
                None => return None,
            };
            let frame = match self.frames.get(&frame_id) {
                Some(frame) => frame,
                None => {
                    warn!("Frame {:?} iterated after closure.", frame_id);
                    continue;
                },
            };
            for entry in frame.prev.iter().chain(frame.next.iter()).chain(once(&frame.current)) {
                if let Some(pipeline) = self.pipelines.get(&entry.pipeline_id) {
                    self.stack.extend(pipeline.children.iter().map(|&c| c));
                }
            }
            return Some(frame)
        }
    }
}

struct WebDriverData {
    load_channel: Option<(PipelineId, IpcSender<webdriver_msg::LoadStatus>)>,
    resize_channel: Option<IpcSender<WindowSizeData>>,
}

impl WebDriverData {
    fn new() -> WebDriverData {
        WebDriverData {
            load_channel: None,
            resize_channel: None,
        }
    }
}

#[derive(Clone, Copy)]
enum ExitPipelineMode {
    Normal,
    Force,
}

/// A logger directed at the constellation from content processes
#[derive(Clone)]
pub struct FromScriptLogger {
    /// A channel to the constellation
    pub constellation_chan: Arc<ReentrantMutex<IpcSender<FromScriptMsg>>>,
}

impl FromScriptLogger {
    /// Create a new constellation logger.
    pub fn new(constellation_chan: IpcSender<FromScriptMsg>) -> FromScriptLogger {
        FromScriptLogger {
            constellation_chan: Arc::new(ReentrantMutex::new(constellation_chan))
        }
    }

    /// The maximum log level the constellation logger is interested in.
    pub fn filter(&self) -> LogLevelFilter {
        LogLevelFilter::Warn
    }
}

impl Log for FromScriptLogger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= LogLevel::Warn
    }

    fn log(&self, record: &LogRecord) {
        if let Some(entry) = log_entry(record) {
            debug!("Sending log entry {:?}.", entry);
            let pipeline_id = PipelineId::installed();
            let thread_name = thread::current().name().map(ToOwned::to_owned);
            let msg = FromScriptMsg::LogEntry(pipeline_id, thread_name, entry);
            let chan = self.constellation_chan.lock().unwrap_or_else(|err| err.into_inner());
            let _ = chan.send(msg);
        }
    }
}

/// A logger directed at the constellation from the compositor
#[derive(Clone)]
pub struct FromCompositorLogger {
    /// A channel to the constellation
    pub constellation_chan: Arc<ReentrantMutex<Sender<FromCompositorMsg>>>,
}

impl FromCompositorLogger {
    /// Create a new constellation logger.
    pub fn new(constellation_chan: Sender<FromCompositorMsg>) -> FromCompositorLogger {
        FromCompositorLogger {
            constellation_chan: Arc::new(ReentrantMutex::new(constellation_chan))
        }
    }

    /// The maximum log level the constellation logger is interested in.
    pub fn filter(&self) -> LogLevelFilter {
        LogLevelFilter::Warn
    }
}

impl Log for FromCompositorLogger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= LogLevel::Warn
    }

    fn log(&self, record: &LogRecord) {
        if let Some(entry) = log_entry(record) {
            debug!("Sending log entry {:?}.", entry);
            let pipeline_id = PipelineId::installed();
            let thread_name = thread::current().name().map(ToOwned::to_owned);
            let msg = FromCompositorMsg::LogEntry(pipeline_id, thread_name, entry);
            let chan = self.constellation_chan.lock().unwrap_or_else(|err| err.into_inner());
            let _ = chan.send(msg);
        }
    }
}

fn log_entry(record: &LogRecord) -> Option<LogEntry> {
    match record.level() {
        LogLevel::Error if thread::panicking() => Some(LogEntry::Panic(
            format!("{}", record.args()),
            format!("{:?}", Backtrace::new())
        )),
        LogLevel::Error => Some(LogEntry::Error(
            format!("{}", record.args())
        )),
        LogLevel::Warn => Some(LogEntry::Warn(
            format!("{}", record.args())
        )),
        _ => None,
    }
}

const WARNINGS_BUFFER_SIZE: usize = 32;

impl<Message, LTF, STF> Constellation<Message, LTF, STF>
    where LTF: LayoutThreadFactory<Message=Message>,
          STF: ScriptThreadFactory<Message=Message>
{
    pub fn start(state: InitialConstellationState) -> (Sender<FromCompositorMsg>, IpcSender<SWManagerMsg>) {
        let (compositor_sender, compositor_receiver) = channel();

        // service worker manager to communicate with constellation
        let (swmanager_sender, swmanager_receiver) = ipc::channel().expect("ipc channel failure");
        let sw_mgr_clone = swmanager_sender.clone();

        spawn_named("Constellation".to_owned(), move || {
            let (ipc_script_sender, ipc_script_receiver) = ipc::channel().expect("ipc channel failure");
            let script_receiver = ROUTER.route_ipc_receiver_to_new_mpsc_receiver(ipc_script_receiver);

            let (ipc_layout_sender, ipc_layout_receiver) = ipc::channel().expect("ipc channel failure");
            let layout_receiver = ROUTER.route_ipc_receiver_to_new_mpsc_receiver(ipc_layout_receiver);

            let swmanager_receiver = ROUTER.route_ipc_receiver_to_new_mpsc_receiver(swmanager_receiver);

            PipelineNamespace::install(PipelineNamespaceId(0));

            let mut constellation: Constellation<Message, LTF, STF> = Constellation {
                script_sender: ipc_script_sender,
                layout_sender: ipc_layout_sender,
                script_receiver: script_receiver,
                compositor_receiver: compositor_receiver,
                layout_receiver: layout_receiver,
                compositor_proxy: state.compositor_proxy,
                devtools_chan: state.devtools_chan,
                bluetooth_thread: state.bluetooth_thread,
                public_resource_threads: state.public_resource_threads,
                private_resource_threads: state.private_resource_threads,
                image_cache_thread: state.image_cache_thread,
                font_cache_thread: state.font_cache_thread,
                swmanager_chan: None,
                swmanager_receiver: swmanager_receiver,
                swmanager_sender: sw_mgr_clone,
                pipelines: HashMap::new(),
                frames: HashMap::new(),
                pending_frames: vec!(),
                // We initialize the namespace at 1, since we reserved namespace 0 for the constellation
                next_pipeline_namespace_id: PipelineNamespaceId(1),
                root_frame_id: FrameId::new(),
                focus_pipeline_id: None,
                time_profiler_chan: state.time_profiler_chan,
                mem_profiler_chan: state.mem_profiler_chan,
                window_size: WindowSizeData {
                    visible_viewport: opts::get().initial_window_size.to_f32() *
                                          ScaleFactor::new(1.0),
                    initial_viewport: opts::get().initial_window_size.to_f32() *
                        ScaleFactor::new(1.0),
                    device_pixel_ratio:
                        ScaleFactor::new(opts::get().device_pixels_per_px.unwrap_or(1.0)),
                },
                phantom: PhantomData,
                webdriver: WebDriverData::new(),
                scheduler_chan: TimerScheduler::start(),
                child_processes: Vec::new(),
                document_states: HashMap::new(),
                webrender_api_sender: state.webrender_api_sender,
                shutting_down: false,
                handled_warnings: VecDeque::new(),
                random_pipeline_closure: opts::get().random_pipeline_closure_probability.map(|prob| {
                    let seed = opts::get().random_pipeline_closure_seed.unwrap_or_else(random);
                    let rng = StdRng::from_seed(&[seed]);
                    warn!("Randomly closing pipelines.");
                    info!("Using seed {} for random pipeline closure.", seed);
                    (rng, prob)
                }),
            };

            constellation.run();
        });
        (compositor_sender, swmanager_sender)
    }

    fn run(&mut self) {
        while !self.shutting_down || !self.pipelines.is_empty() {
            // Randomly close a pipeline if --random-pipeline-closure-probability is set
            // This is for testing the hardening of the constellation.
            self.maybe_close_random_pipeline();
            self.handle_request();
        }
        self.handle_shutdown();
    }

    fn next_pipeline_namespace_id(&mut self) -> PipelineNamespaceId {
        let namespace_id = self.next_pipeline_namespace_id;
        let PipelineNamespaceId(ref mut i) = self.next_pipeline_namespace_id;
        *i += 1;
        namespace_id
    }

    /// Helper function for creating a pipeline
    fn new_pipeline(&mut self,
                    pipeline_id: PipelineId,
                    frame_id: FrameId,
                    parent_info: Option<(PipelineId, FrameType)>,
                    old_pipeline_id: Option<PipelineId>,
                    initial_window_size: Option<TypedSize2D<f32, PagePx>>,
                    script_channel: Option<IpcSender<ConstellationControlMsg>>,
                    load_data: LoadData,
                    is_private: bool) {
        if self.shutting_down { return; }

        let resource_threads = if is_private {
            self.private_resource_threads.clone()
        } else {
            self.public_resource_threads.clone()
        };

        let prev_visibility = if let Some(id) = old_pipeline_id {
            self.pipelines.get(&id).map(|pipeline| pipeline.visible)
        } else if let Some((parent_pipeline_id, _)) = parent_info {
            self.pipelines.get(&parent_pipeline_id).map(|pipeline| pipeline.visible)
        } else {
            None
        };

        let result = Pipeline::spawn::<Message, LTF, STF>(InitialPipelineState {
            id: pipeline_id,
            frame_id: frame_id,
            parent_info: parent_info,
            constellation_chan: self.script_sender.clone(),
            layout_to_constellation_chan: self.layout_sender.clone(),
            scheduler_chan: self.scheduler_chan.clone(),
            compositor_proxy: self.compositor_proxy.clone_compositor_proxy(),
            devtools_chan: self.devtools_chan.clone(),
            bluetooth_thread: self.bluetooth_thread.clone(),
            swmanager_thread: self.swmanager_sender.clone(),
            image_cache_thread: self.image_cache_thread.clone(),
            font_cache_thread: self.font_cache_thread.clone(),
            resource_threads: resource_threads,
            time_profiler_chan: self.time_profiler_chan.clone(),
            mem_profiler_chan: self.mem_profiler_chan.clone(),
            window_size: initial_window_size,
            script_chan: script_channel,
            load_data: load_data,
            device_pixel_ratio: self.window_size.device_pixel_ratio,
            pipeline_namespace_id: self.next_pipeline_namespace_id(),
            prev_visibility: prev_visibility,
            webrender_api_sender: self.webrender_api_sender.clone(),
            is_private: is_private,
        });

        let (pipeline, child_process) = match result {
            Ok(result) => result,
            Err(e) => return self.handle_send_error(pipeline_id, e),
        };

        if let Some(child_process) = child_process {
            self.child_processes.push(child_process);
        }

        assert!(!self.pipelines.contains_key(&pipeline_id));
        self.pipelines.insert(pipeline_id, pipeline);
    }

    // Get an iterator for the current frame tree. Specify self.root_frame_id to
    // iterate the entire tree, or a specific frame id to iterate only that sub-tree.
    fn current_frame_tree_iter(&self, frame_id_root: FrameId) -> FrameTreeIterator {
        FrameTreeIterator {
            stack: vec!(frame_id_root),
            pipelines: &self.pipelines,
            frames: &self.frames,
        }
    }

    fn full_frame_tree_iter(&self, frame_id_root: FrameId) -> FullFrameTreeIterator {
        FullFrameTreeIterator {
            stack: vec!(frame_id_root),
            pipelines: &self.pipelines,
            frames: &self.frames,
        }
    }

    fn joint_session_future(&self, frame_id_root: FrameId) -> Vec<(Instant, FrameId, PipelineId)> {
        let mut future = vec!();
        for frame in self.full_frame_tree_iter(frame_id_root) {
            future.extend(frame.next.iter().map(|entry| (entry.instant, entry.frame_id, entry.pipeline_id)));
        }

        // reverse sorting
        future.sort_by(|a, b| b.cmp(a));
        future
    }

    fn joint_session_future_is_empty(&self, frame_id_root: FrameId) -> bool {
        self.full_frame_tree_iter(frame_id_root)
            .all(|frame| frame.next.is_empty())
    }

    fn joint_session_past(&self, frame_id_root: FrameId) -> Vec<(Instant, FrameId, PipelineId)> {
        let mut past = vec!();
        for frame in self.full_frame_tree_iter(frame_id_root) {
            let mut prev_instant = frame.current.instant;
            for entry in frame.prev.iter().rev() {
                past.push((prev_instant, entry.frame_id, entry.pipeline_id));
                prev_instant = entry.instant;
            }
        }

        past.sort();
        past
    }

    fn joint_session_past_is_empty(&self, frame_id_root: FrameId) -> bool {
        self.full_frame_tree_iter(frame_id_root)
            .all(|frame| frame.prev.is_empty())
    }

    // Create a new frame and update the internal bookkeeping.
    fn new_frame(&mut self, frame_id: FrameId, pipeline_id: PipelineId) {
        let frame = Frame::new(frame_id, pipeline_id);
        self.frames.insert(frame_id, frame);

        // If a child frame, add it to the parent pipeline.
        let parent_info = self.pipelines.get(&pipeline_id)
            .and_then(|pipeline| pipeline.parent_info);
        if let Some((parent_id, _)) = parent_info {
            if let Some(parent) = self.pipelines.get_mut(&parent_id) {
                parent.add_child(frame_id);
            }
        }
    }

    /// Handles loading pages, navigation, and granting access to the compositor
    #[allow(unsafe_code)]
    fn handle_request(&mut self) {
        enum Request {
            Script(FromScriptMsg),
            Compositor(FromCompositorMsg),
            Layout(FromLayoutMsg),
            FromSWManager(SWManagerMsg),
        }

        // Get one incoming request.
        // This is one of the few places where the compositor is
        // allowed to panic. If one of the receiver.recv() calls
        // fails, it is because the matching sender has been
        // reclaimed, but this can't happen in normal execution
        // because the constellation keeps a pointer to the sender,
        // so it should never be reclaimed. A possible scenario in
        // which receiver.recv() fails is if some unsafe code
        // produces undefined behaviour, resulting in the destructor
        // being called. If this happens, there's not much we can do
        // other than panic.
        let request = {
            let receiver_from_script = &self.script_receiver;
            let receiver_from_compositor = &self.compositor_receiver;
            let receiver_from_layout = &self.layout_receiver;
            let receiver_from_swmanager = &self.swmanager_receiver;
            select! {
                msg = receiver_from_script.recv() =>
                    Request::Script(msg.expect("Unexpected script channel panic in constellation")),
                msg = receiver_from_compositor.recv() =>
                    Request::Compositor(msg.expect("Unexpected compositor channel panic in constellation")),
                msg = receiver_from_layout.recv() =>
                    Request::Layout(msg.expect("Unexpected layout channel panic in constellation")),
                msg = receiver_from_swmanager.recv() =>
                    Request::FromSWManager(msg.expect("Unexpected panic channel panic in constellation"))
            }
        };

        match request {
            Request::Compositor(message) => {
                self.handle_request_from_compositor(message)
            },
            Request::Script(message) => {
                self.handle_request_from_script(message);
            },
            Request::Layout(message) => {
                self.handle_request_from_layout(message);
            },
            Request::FromSWManager(message) => {
                self.handle_request_from_swmanager(message);
            }
        }
    }

    fn handle_request_from_swmanager(&mut self, message: SWManagerMsg) {
        match message {
            SWManagerMsg::OwnSender(sw_sender) => {
                // store service worker manager for communicating with it.
                self.swmanager_chan = Some(sw_sender);
            }
        }
    }

    fn handle_request_from_compositor(&mut self, message: FromCompositorMsg) {
        match message {
            FromCompositorMsg::Exit => {
                debug!("constellation exiting");
                self.handle_exit();
            }
            // The compositor discovered the size of a subframe. This needs to be reflected by all
            // frame trees in the navigation context containing the subframe.
            FromCompositorMsg::FrameSize(pipeline_id, size) => {
                debug!("constellation got frame size message");
                self.handle_frame_size_msg(pipeline_id, &TypedSize2D::from_untyped(&size));
            }
            FromCompositorMsg::GetFrame(pipeline_id, resp_chan) => {
                debug!("constellation got get root pipeline message");
                self.handle_get_frame(pipeline_id, resp_chan);
            }
            FromCompositorMsg::GetPipeline(frame_id, resp_chan) => {
                debug!("constellation got get root pipeline message");
                self.handle_get_pipeline(frame_id, resp_chan);
            }
            FromCompositorMsg::GetPipelineTitle(pipeline_id) => {
                debug!("constellation got get-pipeline-title message");
                self.handle_get_pipeline_title_msg(pipeline_id);
            }
            FromCompositorMsg::KeyEvent(ch, key, state, modifiers) => {
                debug!("constellation got key event message");
                self.handle_key_msg(ch, key, state, modifiers);
            }
            // Load a new page from a typed url
            // If there is already a pending page (self.pending_frames), it will not be overridden;
            // However, if the id is not encompassed by another change, it will be.
            FromCompositorMsg::LoadUrl(source_id, load_data) => {
                debug!("constellation got URL load message from compositor");
                self.handle_load_url_msg(source_id, load_data, false);
            }
            FromCompositorMsg::IsReadyToSaveImage(pipeline_states) => {
                let is_ready = self.handle_is_ready_to_save_image(pipeline_states);
                debug!("Ready to save image {:?}.", is_ready);
                if opts::get().is_running_problem_test {
                    println!("got ready to save image query, result is {:?}", is_ready);
                }
                let is_ready = is_ready == ReadyToSave::Ready;
                self.compositor_proxy.send(ToCompositorMsg::IsReadyToSaveImageReply(is_ready));
                if opts::get().is_running_problem_test {
                    println!("sent response");
                }
            }
            // This should only be called once per constellation, and only by the browser
            FromCompositorMsg::InitLoadUrl(url) => {
                debug!("constellation got init load URL message");
                self.handle_init_load(url);
            }
            // Handle a forward or back request
            FromCompositorMsg::TraverseHistory(pipeline_id, direction) => {
                debug!("constellation got traverse history message from compositor");
                self.handle_traverse_history_msg(pipeline_id, direction);
            }
            FromCompositorMsg::WindowSize(new_size, size_type) => {
                debug!("constellation got window resize message");
                self.handle_window_size_msg(new_size, size_type);
            }
            FromCompositorMsg::TickAnimation(pipeline_id, tick_type) => {
                self.handle_tick_animation(pipeline_id, tick_type)
            }
            FromCompositorMsg::WebDriverCommand(command) => {
                debug!("constellation got webdriver command message");
                self.handle_webdriver_msg(command);
            }
            FromCompositorMsg::Reload => {
                debug!("constellation got reload message");
                self.handle_reload_msg();
            }
            FromCompositorMsg::LogEntry(pipeline_id, thread_name, entry) => {
                self.handle_log_entry(pipeline_id, thread_name, entry);
            }
        }
    }

    fn handle_request_from_script(&mut self, message: FromScriptMsg) {
        match message {
            FromScriptMsg::PipelineExited(pipeline_id) => {
                self.handle_pipeline_exited(pipeline_id);
            }
            FromScriptMsg::ScriptLoadedURLInIFrame(load_info) => {
                debug!("constellation got iframe URL load message {:?} {:?} {:?}",
                       load_info.parent_pipeline_id,
                       load_info.old_pipeline_id,
                       load_info.new_pipeline_id);
                self.handle_script_loaded_url_in_iframe_msg(load_info);
            }
            FromScriptMsg::ChangeRunningAnimationsState(pipeline_id, animation_state) => {
                self.handle_change_running_animations_state(pipeline_id, animation_state)
            }
            // Load a new page from a mouse click
            // If there is already a pending page (self.pending_frames), it will not be overridden;
            // However, if the id is not encompassed by another change, it will be.
            FromScriptMsg::LoadUrl(source_id, load_data, replace) => {
                debug!("constellation got URL load message from script");
                self.handle_load_url_msg(source_id, load_data, replace);
            }
            // A page loaded has completed all parsing, script, and reflow messages have been sent.
            FromScriptMsg::LoadComplete(pipeline_id) => {
                debug!("constellation got load complete message");
                self.handle_load_complete_msg(pipeline_id)
            }
            // Handle a forward or back request
            FromScriptMsg::TraverseHistory(pipeline_id, direction) => {
                debug!("constellation got traverse history message from script");
                self.handle_traverse_history_msg(pipeline_id, direction);
            }
            // Handle a joint session history length request.
            FromScriptMsg::JointSessionHistoryLength(pipeline_id, sender) => {
                debug!("constellation got joint session history length message from script");
                self.handle_joint_session_history_length(pipeline_id, sender);
            }
            // Notification that the new document is ready to become active
            FromScriptMsg::ActivateDocument(pipeline_id) => {
                debug!("constellation got activate document message");
                self.handle_activate_document_msg(pipeline_id);
            }
            // Update pipeline url after redirections
            FromScriptMsg::SetFinalUrl(pipeline_id, final_url) => {
                // The script may have finished loading after we already started shutting down.
                if let Some(ref mut pipeline) = self.pipelines.get_mut(&pipeline_id) {
                    debug!("constellation got set final url message");
                    pipeline.url = final_url;
                } else {
                    warn!("constellation got set final url message for dead pipeline");
                }
            }
            FromScriptMsg::MozBrowserEvent(parent_pipeline_id, pipeline_id, event) => {
                debug!("constellation got mozbrowser event message");
                self.handle_mozbrowser_event_msg(parent_pipeline_id,
                                                 pipeline_id,
                                                 event);
            }
            FromScriptMsg::Focus(pipeline_id) => {
                debug!("constellation got focus message");
                self.handle_focus_msg(pipeline_id);
            }
            FromScriptMsg::ForwardEvent(pipeline_id, event) => {
                let msg = ConstellationControlMsg::SendEvent(pipeline_id, event);
                let result = match self.pipelines.get(&pipeline_id) {
                    None => { debug!("Pipeline {:?} got event after closure.", pipeline_id); return; }
                    Some(pipeline) => pipeline.script_chan.send(msg),
                };
                if let Err(e) = result {
                    self.handle_send_error(pipeline_id, e);
                }
            }
            FromScriptMsg::GetClipboardContents(sender) => {
                if let Err(e) = sender.send("".to_owned()) {
                    warn!("Failed to send clipboard ({})", e);
                }
            }
            FromScriptMsg::SetClipboardContents(_) => {
            }
            FromScriptMsg::SetVisible(pipeline_id, visible) => {
                debug!("constellation got set visible messsage");
                self.handle_set_visible_msg(pipeline_id, visible);
            }
            FromScriptMsg::VisibilityChangeComplete(pipeline_id, visible) => {
                debug!("constellation got set visibility change complete message");
                self.handle_visibility_change_complete(pipeline_id, visible);
            }
            FromScriptMsg::RemoveIFrame(pipeline_id, sender) => {
                debug!("constellation got remove iframe message");
                self.handle_remove_iframe_msg(pipeline_id);
                if let Some(sender) = sender {
                    if let Err(e) = sender.send(()) {
                        warn!("Error replying to remove iframe ({})", e);
                    }
                }
            }
            FromScriptMsg::NewFavicon(url) => {
                debug!("constellation got new favicon message");
                self.compositor_proxy.send(ToCompositorMsg::NewFavicon(url));
            }
            FromScriptMsg::HeadParsed => {
                debug!("constellation got head parsed message");
                self.compositor_proxy.send(ToCompositorMsg::HeadParsed);
            }
            FromScriptMsg::CreateCanvasPaintThread(size, sender) => {
                debug!("constellation got create-canvas-paint-thread message");
                self.handle_create_canvas_paint_thread_msg(&size, sender)
            }
            FromScriptMsg::CreateWebGLPaintThread(size, attributes, sender) => {
                debug!("constellation got create-WebGL-paint-thread message");
                self.handle_create_webgl_paint_thread_msg(&size, attributes, sender)
            }
            FromScriptMsg::NodeStatus(message) => {
                debug!("constellation got NodeStatus message");
                self.compositor_proxy.send(ToCompositorMsg::Status(message));
            }
            FromScriptMsg::SetDocumentState(pipeline_id, state) => {
                debug!("constellation got SetDocumentState message");
                self.document_states.insert(pipeline_id, state);
            }
            FromScriptMsg::Alert(pipeline_id, message, sender) => {
                debug!("constellation got Alert message");
                self.handle_alert(pipeline_id, message, sender);
            }

            FromScriptMsg::ScrollFragmentPoint(pipeline_id, point, smooth) => {
                self.compositor_proxy.send(ToCompositorMsg::ScrollFragmentPoint(pipeline_id,
                                                                                point,
                                                                                smooth));
            }

            FromScriptMsg::GetClientWindow(send) => {
                self.compositor_proxy.send(ToCompositorMsg::GetClientWindow(send));
            }

            FromScriptMsg::MoveTo(point) => {
                self.compositor_proxy.send(ToCompositorMsg::MoveTo(point));
            }

            FromScriptMsg::ResizeTo(size) => {
                self.compositor_proxy.send(ToCompositorMsg::ResizeTo(size));
            }

            FromScriptMsg::Exit => {
                self.compositor_proxy.send(ToCompositorMsg::Exit);
            }
            FromScriptMsg::LogEntry(pipeline_id, thread_name, entry) => {
                self.handle_log_entry(pipeline_id, thread_name, entry);
            }

            FromScriptMsg::SetTitle(pipeline_id, title) => {
                self.compositor_proxy.send(ToCompositorMsg::ChangePageTitle(pipeline_id, title))
            }

            FromScriptMsg::SendKeyEvent(ch, key, key_state, key_modifiers) => {
                self.compositor_proxy.send(ToCompositorMsg::KeyEvent(ch, key, key_state, key_modifiers))
            }

            FromScriptMsg::TouchEventProcessed(result) => {
                self.compositor_proxy.send(ToCompositorMsg::TouchEventProcessed(result))
            }

            FromScriptMsg::RegisterServiceWorker(scope_things, scope) => {
                debug!("constellation got store registration scope message");
                self.handle_register_serviceworker(scope_things, scope);
            }
            FromScriptMsg::ForwardDOMMessage(msg_vec, scope_url) => {
                if let Some(ref mgr) = self.swmanager_chan {
                    let _ = mgr.send(ServiceWorkerMsg::ForwardDOMMessage(msg_vec, scope_url));
                } else {
                    warn!("Unable to forward DOMMessage for postMessage call");
                }
            }
        }
    }

    fn handle_request_from_layout(&mut self, message: FromLayoutMsg) {
        match message {
            FromLayoutMsg::ChangeRunningAnimationsState(pipeline_id, animation_state) => {
                self.handle_change_running_animations_state(pipeline_id, animation_state)
            }
            FromLayoutMsg::SetCursor(cursor) => {
                self.handle_set_cursor_msg(cursor)
            }
            FromLayoutMsg::ViewportConstrained(pipeline_id, constraints) => {
                debug!("constellation got viewport-constrained event message");
                self.handle_viewport_constrained_msg(pipeline_id, constraints);
            }
        }
    }

    fn handle_register_serviceworker(&self, scope_things: ScopeThings, scope: Url) {
        if let Some(ref mgr) = self.swmanager_chan {
            let _ = mgr.send(ServiceWorkerMsg::RegisterServiceWorker(scope_things, scope));
        } else {
            warn!("sending scope info to service worker manager failed");
        }
    }

    fn handle_exit(&mut self) {
        // TODO: add a timer, which forces shutdown if threads aren't responsive.
        if self.shutting_down { return; }
        self.shutting_down = true;

        self.mem_profiler_chan.send(mem::ProfilerMsg::Exit);

        // TODO: exit before the root frame is initialized?
        debug!("Removing root frame.");
        let root_frame_id = self.root_frame_id;
        self.close_frame(root_frame_id, ExitPipelineMode::Normal);

        // Close any pending frames and pipelines
        while let Some(pending) = self.pending_frames.pop() {
            debug!("Removing pending frame {}.", pending.frame_id);
            self.close_frame(pending.frame_id, ExitPipelineMode::Normal);
            debug!("Removing pending pipeline {}.", pending.new_pipeline_id);
            self.close_pipeline(pending.new_pipeline_id, ExitPipelineMode::Normal);
        }

        // In case there are frames which weren't attached to the frame tree, we close them.
        let frame_ids: Vec<FrameId> = self.frames.keys().cloned().collect();
        for frame_id in frame_ids {
            debug!("Removing detached frame {}.", frame_id);
            self.close_frame(frame_id, ExitPipelineMode::Normal);
        }

        // In case there are pipelines which weren't attached to the pipeline tree, we close them.
        let pipeline_ids: Vec<PipelineId> = self.pipelines.keys().cloned().collect();
        for pipeline_id in pipeline_ids {
            debug!("Removing detached pipeline {}.", pipeline_id);
            self.close_pipeline(pipeline_id, ExitPipelineMode::Normal);
        }
    }

    fn handle_shutdown(&mut self) {
        // At this point, there are no active pipelines,
        // so we can safely block on other threads, without worrying about deadlock.
        // Channels to receive signals when threads are done exiting.
        let (core_sender, core_receiver) = ipc::channel().expect("Failed to create IPC channel!");
        let (storage_sender, storage_receiver) = ipc::channel().expect("Failed to create IPC channel!");

        debug!("Exiting image cache.");
        self.image_cache_thread.exit();

        debug!("Exiting core resource threads.");
        if let Err(e) = self.public_resource_threads.send(net_traits::CoreResourceMsg::Exit(core_sender)) {
            warn!("Exit resource thread failed ({})", e);
        }

        if let Some(ref chan) = self.devtools_chan {
            debug!("Exiting devtools.");
            let msg = DevtoolsControlMsg::FromChrome(ChromeToDevtoolsControlMsg::ServerExitMsg);
            if let Err(e) = chan.send(msg) {
                warn!("Exit devtools failed ({})", e);
            }
        }

        debug!("Exiting storage resource threads.");
        if let Err(e) = self.public_resource_threads.send(StorageThreadMsg::Exit(storage_sender)) {
            warn!("Exit storage thread failed ({})", e);
        }

        debug!("Exiting bluetooth thread.");
        if let Err(e) = self.bluetooth_thread.send(BluetoothMethodMsg::Exit) {
            warn!("Exit bluetooth thread failed ({})", e);
        }

        debug!("Exiting service worker manager thread.");
        if let Some(mgr) = self.swmanager_chan.as_ref() {
            if let Err(e) = mgr.send(ServiceWorkerMsg::Exit) {
                warn!("Exit service worker manager failed ({})", e);
            }
        }

        debug!("Exiting font cache thread.");
        self.font_cache_thread.exit();

        // Receive exit signals from threads.
        if let Err(e) = core_receiver.recv() {
            warn!("Exit resource thread failed ({})", e);
        }
        if let Err(e) = storage_receiver.recv() {
            warn!("Exit storage thread failed ({})", e);
        }

        debug!("Asking compositor to complete shutdown.");
        self.compositor_proxy.send(ToCompositorMsg::ShutdownComplete);
    }

    fn handle_pipeline_exited(&mut self, pipeline_id: PipelineId) {
        debug!("Pipeline {:?} exited.", pipeline_id);
        self.pipelines.remove(&pipeline_id);
    }

    fn handle_send_error(&mut self, pipeline_id: PipelineId, err: IOError) {
        // Treat send error the same as receiving a panic message
        debug!("Pipeline {:?} send error ({}).", pipeline_id, err);
        self.handle_panic(Some(pipeline_id), format!("Send failed ({})", err), None);
    }

    fn handle_panic(&mut self, pipeline_id: Option<PipelineId>, reason: String, backtrace: Option<String>) {
        if opts::get().hard_fail {
            // It's quite difficult to make Servo exit cleanly if some threads have failed.
            // Hard fail exists for test runners so we crash and that's good enough.
            println!("Pipeline failed in hard-fail mode.  Crashing!");
            process::exit(1);
        }

        debug!("Panic handler for pipeline {:?}: {}.", pipeline_id, reason);

        // Notify the browser chrome that the pipeline has failed
        self.trigger_mozbrowsererror(pipeline_id, reason, backtrace);

        if let Some(pipeline_id) = pipeline_id {
            let pipeline_url = self.pipelines.get(&pipeline_id).map(|pipeline| pipeline.url.clone());
            let parent_info = self.pipelines.get(&pipeline_id).and_then(|pipeline| pipeline.parent_info);
            let window_size = self.pipelines.get(&pipeline_id).and_then(|pipeline| pipeline.size);
            let frame_id = self.pipelines.get(&pipeline_id).map(|pipeline| pipeline.frame_id);

            self.close_pipeline(pipeline_id, ExitPipelineMode::Force);
            self.pipelines.remove(&pipeline_id);

            while let Some(pending_pipeline_id) = self.pending_frames.iter().find(|pending| {
                pending.old_pipeline_id == Some(pipeline_id)
            }).map(|frame| frame.new_pipeline_id) {
                warn!("removing pending frame change for failed pipeline");
                self.close_pipeline(pending_pipeline_id, ExitPipelineMode::Force);
            }

            let failure_url = Url::parse("about:failure").expect("infallible");

            if let Some(pipeline_url) = pipeline_url {
                if pipeline_url == failure_url {
                    return error!("about:failure failed");
                }
            }

            warn!("creating replacement pipeline for about:failure");

            if let Some(frame_id) = frame_id {
                let new_pipeline_id = PipelineId::new();
                let load_data = LoadData::new(failure_url, None, None);
                self.new_pipeline(new_pipeline_id, frame_id, parent_info, Some(pipeline_id),
                                  window_size, None, load_data, false);

                self.pending_frames.push(FrameChange {
                    frame_id: frame_id,
                    old_pipeline_id: Some(pipeline_id),
                    new_pipeline_id: new_pipeline_id,
                    document_ready: false,
                    replace: false,
                });
            }
        }
    }

    fn handle_log_entry(&mut self, pipeline_id: Option<PipelineId>, thread_name: Option<String>, entry: LogEntry) {
        debug!("Received log entry {:?}.", entry);
        match entry {
            LogEntry::Panic(reason, backtrace) => self.handle_panic(pipeline_id, reason, Some(backtrace)),
            LogEntry::Error(reason) | LogEntry::Warn(reason) => {
                // VecDeque::truncate is unstable
                if WARNINGS_BUFFER_SIZE <= self.handled_warnings.len() {
                    self.handled_warnings.pop_front();
                }
                self.handled_warnings.push_back((thread_name, reason));
            },
        }
    }

    fn handle_init_load(&mut self, url: Url) {
        let window_size = self.window_size.visible_viewport;
        let root_pipeline_id = PipelineId::new();
        let root_frame_id = self.root_frame_id;
        self.new_pipeline(root_pipeline_id, root_frame_id, None, None, Some(window_size), None,
                          LoadData::new(url.clone(), None, None), false);
        self.handle_load_start_msg(root_pipeline_id);
        self.pending_frames.push(FrameChange {
            frame_id: self.root_frame_id,
            old_pipeline_id: None,
            new_pipeline_id: root_pipeline_id,
            document_ready: false,
            replace: false,
        });
        self.compositor_proxy.send(ToCompositorMsg::ChangePageUrl(root_pipeline_id, url));
    }

    fn handle_frame_size_msg(&mut self,
                             pipeline_id: PipelineId,
                             size: &TypedSize2D<f32, PagePx>) {
        let msg = ConstellationControlMsg::Resize(pipeline_id, WindowSizeData {
            visible_viewport: *size,
            initial_viewport: *size * ScaleFactor::new(1.0),
            device_pixel_ratio: self.window_size.device_pixel_ratio,
        }, WindowSizeType::Initial);

        // Store the new rect inside the pipeline
        let result = {
            // Find the pipeline that corresponds to this rectangle. It's possible that this
            // pipeline may have already exited before we process this message, so just
            // early exit if that occurs.
            match self.pipelines.get_mut(&pipeline_id) {
                Some(pipeline) => {
                    pipeline.size = Some(*size);
                    pipeline.script_chan.send(msg)
                }
                None => return,
            }
        };

        if let Err(e) = result {
            self.handle_send_error(pipeline_id, e);
        }
    }

    fn handle_subframe_loaded(&mut self, pipeline_id: PipelineId) {
        let (frame_id, parent_info) = match self.pipelines.get(&pipeline_id) {
            Some(pipeline) => (pipeline.frame_id, pipeline.parent_info),
            None => return warn!("Pipeline {:?} loaded after closure.", pipeline_id),
        };
        let subframe_parent_id = match parent_info {
            Some(ref parent) => parent.0,
            None => return warn!("Pipeline {:?} has no parent.", pipeline_id),
        };
        let msg = ConstellationControlMsg::DispatchFrameLoadEvent {
            target: frame_id,
            parent: subframe_parent_id,
            child: pipeline_id,
        };
        let result = match self.pipelines.get(&subframe_parent_id) {
            Some(pipeline) => pipeline.script_chan.send(msg),
            None => return warn!("Pipeline {:?} subframe loaded after closure.", subframe_parent_id),
        };
        if let Err(e) = result {
            self.handle_send_error(subframe_parent_id, e);
        }
    }

    // The script thread associated with pipeline_id has loaded a URL in an iframe via script. This
    // will result in a new pipeline being spawned and a frame tree being added to
    // parent_pipeline_id's frame tree's children. This message is never the result of a
    // page navigation.
    fn handle_script_loaded_url_in_iframe_msg(&mut self, load_info: IFrameLoadInfo) {
        let (load_data, script_chan, window_size, is_private) = {
            let old_pipeline = load_info.old_pipeline_id
                .and_then(|old_pipeline_id| self.pipelines.get(&old_pipeline_id));

            let source_pipeline =  match self.pipelines.get(&load_info.parent_pipeline_id) {
                Some(source_pipeline) => source_pipeline,
                None => return warn!("Script loaded url in closed iframe {}.", load_info.parent_pipeline_id),
            };

            // If no url is specified, reload.
            let load_data = load_info.load_data.unwrap_or_else(|| {
                let url = match old_pipeline {
                    Some(old_pipeline) => old_pipeline.url.clone(),
                    None => Url::parse("about:blank").expect("infallible"),
                };

                // TODO - loaddata here should have referrer info (not None, None)
                LoadData::new(url, None, None)
            });

            // Compare the pipeline's url to the new url. If the origin is the same,
            // then reuse the script thread in creating the new pipeline
            let source_url = &source_pipeline.url;

            let is_private = load_info.is_private || source_pipeline.is_private;

            // FIXME(#10968): this should probably match the origin check in
            //                HTMLIFrameElement::contentDocument.
            let same_script = source_url.host() == load_data.url.host() &&
                              source_url.port() == load_data.url.port() &&
                              load_info.sandbox == IFrameSandboxState::IFrameUnsandboxed &&
                              source_pipeline.is_private == is_private;

            // Reuse the script thread if the URL is same-origin
            let script_chan = if same_script {
                debug!("Constellation: loading same-origin iframe, \
                        parent url {:?}, iframe url {:?}", source_url, load_data.url);
                Some(source_pipeline.script_chan.clone())
            } else {
                debug!("Constellation: loading cross-origin iframe, \
                        parent url {:?}, iframe url {:?}", source_url, load_data.url);
                None
            };

            let window_size = old_pipeline.and_then(|old_pipeline| old_pipeline.size);

            if let Some(old_pipeline) = old_pipeline {
                old_pipeline.freeze();
            }

            (load_data, script_chan, window_size, is_private)
        };


        // Create the new pipeline, attached to the parent and push to pending frames
        self.new_pipeline(load_info.new_pipeline_id,
                          load_info.frame_id,
                          Some((load_info.parent_pipeline_id, load_info.frame_type)),
                          load_info.old_pipeline_id,
                          window_size,
                          script_chan,
                          load_data,
                          is_private);

        self.pending_frames.push(FrameChange {
            frame_id: load_info.frame_id,
            old_pipeline_id: load_info.old_pipeline_id,
            new_pipeline_id: load_info.new_pipeline_id,
            document_ready: false,
            replace: load_info.replace,
        });
    }

    fn handle_set_cursor_msg(&mut self, cursor: Cursor) {
        self.compositor_proxy.send(ToCompositorMsg::SetCursor(cursor))
    }

    fn handle_change_running_animations_state(&mut self,
                                              pipeline_id: PipelineId,
                                              animation_state: AnimationState) {
        self.compositor_proxy.send(ToCompositorMsg::ChangeRunningAnimationsState(pipeline_id,
                                                                               animation_state))
    }

    fn handle_tick_animation(&mut self, pipeline_id: PipelineId, tick_type: AnimationTickType) {
        let result = match tick_type {
            AnimationTickType::Script => {
                let msg = ConstellationControlMsg::TickAllAnimations(pipeline_id);
                match self.pipelines.get(&pipeline_id) {
                    Some(pipeline) => pipeline.script_chan.send(msg),
                    None => return warn!("Pipeline {:?} got script tick after closure.", pipeline_id),
                }
            }
            AnimationTickType::Layout => {
                let msg = LayoutControlMsg::TickAnimations;
                match self.pipelines.get(&pipeline_id) {
                    Some(pipeline) => pipeline.layout_chan.send(msg),
                    None => return warn!("Pipeline {:?} got layout tick after closure.", pipeline_id),
                }
            }
        };
        if let Err(e) = result {
            self.handle_send_error(pipeline_id, e);
        }
    }

    fn handle_alert(&mut self,
                    pipeline_id: PipelineId,
                    message: String,
                    sender: IpcSender<bool>) {
        let display_alert_dialog = if PREFS.is_mozbrowser_enabled() {
            let parent_pipeline_info = self.pipelines.get(&pipeline_id).and_then(|source| source.parent_info);
            if parent_pipeline_info.is_some() {
                let root_pipeline_id = self.frames.get(&self.root_frame_id)
                    .map(|root_frame| root_frame.current.pipeline_id);

                let ancestor_info = self.get_mozbrowser_ancestor_info(pipeline_id);
                if let Some((ancestor_id, mozbrowser_iframe_id)) = ancestor_info {
                    if root_pipeline_id == Some(ancestor_id) {
                        match root_pipeline_id.and_then(|pipeline_id| self.pipelines.get(&pipeline_id)) {
                            Some(root_pipeline) => {
                                // https://developer.mozilla.org/en-US/docs/Web/Events/mozbrowsershowmodalprompt
                                let event = MozBrowserEvent::ShowModalPrompt("alert".to_owned(), "Alert".to_owned(),
                                                                             String::from(message), "".to_owned());
                                root_pipeline.trigger_mozbrowser_event(Some(mozbrowser_iframe_id), event);
                            }
                            None => return warn!("Alert sent to Pipeline {:?} after closure.", root_pipeline_id),
                        }
                    } else {
                        warn!("A non-current frame is trying to show an alert.")
                    }
                }
                false
            } else {
                true
            }
        } else {
            true
        };

        let result = sender.send(display_alert_dialog);
        if let Err(e) = result {
            self.handle_send_error(pipeline_id, e);
        }
    }

    fn handle_load_url_msg(&mut self, source_id: PipelineId, load_data: LoadData, replace: bool) {
        self.load_url(source_id, load_data, replace);
    }

    fn load_url(&mut self, source_id: PipelineId, load_data: LoadData, replace: bool) -> Option<PipelineId> {
        debug!("Loading {} in pipeline {}.", load_data.url, source_id);
        // If this load targets an iframe, its framing element may exist
        // in a separate script thread than the framed document that initiated
        // the new load. The framing element must be notified about the
        // requested change so it can update its internal state.
        //
        // If replace is true, the current entry is replaced instead of a new entry being added.
        let (frame_id, parent_info) = match self.pipelines.get(&source_id) {
            Some(pipeline) => (pipeline.frame_id, pipeline.parent_info),
            None => {
                warn!("Pipeline {:?} loaded after closure.", source_id);
                return None;
            }
        };
        match parent_info {
            Some((parent_pipeline_id, _)) => {
                self.handle_load_start_msg(source_id);
                // Message the constellation to find the script thread for this iframe
                // and issue an iframe load through there.
                let msg = ConstellationControlMsg::Navigate(parent_pipeline_id, frame_id, load_data, replace);
                let result = match self.pipelines.get(&parent_pipeline_id) {
                    Some(parent_pipeline) => parent_pipeline.script_chan.send(msg),
                    None => {
                        warn!("Pipeline {:?} child loaded after closure", parent_pipeline_id);
                        return None;
                    },
                };
                if let Err(e) = result {
                    self.handle_send_error(parent_pipeline_id, e);
                }
                Some(source_id)
            }
            None => {
                // Make sure no pending page would be overridden.
                for frame_change in &self.pending_frames {
                    if frame_change.old_pipeline_id == Some(source_id) {
                        // id that sent load msg is being changed already; abort
                        return None;
                    }
                }

                if !self.pipeline_is_in_current_frame(source_id) {
                    // Disregard this load if the navigating pipeline is not actually
                    // active. This could be caused by a delayed navigation (eg. from
                    // a timer) or a race between multiple navigations (such as an
                    // onclick handler on an anchor element).
                    return None;
                }

                self.handle_load_start_msg(source_id);
                // Being here means either there are no pending frames, or none of the pending
                // changes would be overridden by changing the subframe associated with source_id.

                // Create the new pipeline
                let window_size = self.pipelines.get(&source_id).and_then(|source| source.size);
                let new_pipeline_id = PipelineId::new();
                let root_frame_id = self.root_frame_id;
                self.new_pipeline(new_pipeline_id, root_frame_id, None, None, window_size, None, load_data, false);
                self.pending_frames.push(FrameChange {
                    frame_id: root_frame_id,
                    old_pipeline_id: Some(source_id),
                    new_pipeline_id: new_pipeline_id,
                    document_ready: false,
                    replace: replace,
                });

                // Send message to ScriptThread that will suspend all timers
                match self.pipelines.get(&source_id) {
                    Some(source) => source.freeze(),
                    None => warn!("Pipeline {:?} loaded after closure", source_id),
                };
                Some(new_pipeline_id)
            }
        }
    }

    fn handle_load_start_msg(&mut self, pipeline_id: PipelineId) {
        let frame_id = self.get_top_level_frame_for_pipeline(Some(pipeline_id));
        let forward = !self.joint_session_future_is_empty(frame_id);
        let back = !self.joint_session_past_is_empty(frame_id);
        self.compositor_proxy.send(ToCompositorMsg::LoadStart(back, forward));
    }

    fn handle_load_complete_msg(&mut self, pipeline_id: PipelineId) {
        let mut webdriver_reset = false;
        if let Some((expected_pipeline_id, ref reply_chan)) = self.webdriver.load_channel {
            debug!("Sending load to WebDriver");
            if expected_pipeline_id == pipeline_id {
                let _ = reply_chan.send(webdriver_msg::LoadStatus::LoadComplete);
                webdriver_reset = true;
            }
        }
        if webdriver_reset {
            self.webdriver.load_channel = None;
        }
        let frame_id = self.get_top_level_frame_for_pipeline(Some(pipeline_id));
        let forward = !self.joint_session_future_is_empty(frame_id);
        let back = !self.joint_session_past_is_empty(frame_id);
        let root = self.root_frame_id == frame_id;
        self.compositor_proxy.send(ToCompositorMsg::LoadComplete(back, forward, root));
        self.handle_subframe_loaded(pipeline_id);
    }

    fn handle_traverse_history_msg(&mut self,
                                   pipeline_id: Option<PipelineId>,
                                   direction: TraversalDirection) {
        let frame_id = self.get_top_level_frame_for_pipeline(pipeline_id);

        let mut traversal_info = HashMap::new();

        match direction {
            TraversalDirection::Forward(delta) => {
                let mut future = self.joint_session_future(frame_id);
                for _ in 0..delta {
                    match future.pop() {
                        Some((_, frame_id, pipeline_id)) => {
                            traversal_info.insert(frame_id, pipeline_id);
                        },
                        None => return warn!("invalid traversal delta"),
                    }
                }
            },
            TraversalDirection::Back(delta) => {
                let mut past = self.joint_session_past(frame_id);
                for _ in 0..delta {
                    match past.pop() {
                        Some((_, frame_id, pipeline_id)) => {
                            traversal_info.insert(frame_id, pipeline_id);
                        },
                        None => return warn!("invalid traversal delta"),
                    }
                }
            },
        };
        for (frame_id, pipeline_id) in traversal_info {
            self.traverse_frame_to_pipeline(frame_id, pipeline_id);
        }
    }

    fn handle_joint_session_history_length(&self, pipeline_id: PipelineId, sender: IpcSender<u32>) {
        let frame_id = self.get_top_level_frame_for_pipeline(Some(pipeline_id));

        // Initialize length at 1 to count for the current active entry
        let mut length = 1;
        for frame in self.full_frame_tree_iter(frame_id) {
            length += frame.next.len();
            length += frame.prev.len();
        }
        let _ = sender.send(length as u32);
    }

    fn handle_key_msg(&mut self, ch: Option<char>, key: Key, state: KeyState, mods: KeyModifiers) {
        // Send to the explicitly focused pipeline (if it exists), or the root
        // frame's current pipeline. If neither exist, fall back to sending to
        // the compositor below.
        let root_pipeline_id = self.frames.get(&self.root_frame_id)
            .map(|root_frame| root_frame.current.pipeline_id);
        let pipeline_id = self.focus_pipeline_id.or(root_pipeline_id);

        match pipeline_id {
            Some(pipeline_id) => {
                let event = CompositorEvent::KeyEvent(ch, key, state, mods);
                let msg = ConstellationControlMsg::SendEvent(pipeline_id, event);
                let result = match self.pipelines.get(&pipeline_id) {
                    Some(pipeline) => pipeline.script_chan.send(msg),
                    None => return debug!("Pipeline {:?} got key event after closure.", pipeline_id),
                };
                if let Err(e) = result {
                    self.handle_send_error(pipeline_id, e);
                }
            },
            None => {
                let event = ToCompositorMsg::KeyEvent(ch, key, state, mods);
                self.compositor_proxy.clone_compositor_proxy().send(event);
            }
        }
    }

    fn handle_reload_msg(&mut self) {
        // Send Reload constellation msg to root script channel.
        let root_pipeline_id = self.frames.get(&self.root_frame_id)
            .map(|root_frame| root_frame.current.pipeline_id);

        if let Some(pipeline_id) = root_pipeline_id {
            let msg = ConstellationControlMsg::Reload(pipeline_id);
            let result = match self.pipelines.get(&pipeline_id) {
                Some(pipeline) => pipeline.script_chan.send(msg),
                None => return debug!("Pipeline {:?} got reload event after closure.", pipeline_id),
            };
            if let Err(e) = result {
                self.handle_send_error(pipeline_id, e);
            }
        }
    }

    fn handle_get_pipeline_title_msg(&mut self, pipeline_id: PipelineId) {
        let result = match self.pipelines.get(&pipeline_id) {
            None => return self.compositor_proxy.send(ToCompositorMsg::ChangePageTitle(pipeline_id, None)),
            Some(pipeline) => pipeline.script_chan.send(ConstellationControlMsg::GetTitle(pipeline_id)),
        };
        if let Err(e) = result {
            self.handle_send_error(pipeline_id, e);
        }
    }

    fn handle_mozbrowser_event_msg(&mut self,
                                   parent_pipeline_id: PipelineId,
                                   pipeline_id: PipelineId,
                                   event: MozBrowserEvent) {
        assert!(PREFS.is_mozbrowser_enabled());
        let frame_id = self.pipelines.get(&pipeline_id).map(|pipeline| pipeline.frame_id);

        // Find the script channel for the given parent pipeline,
        // and pass the event to that script thread.
        // If the pipeline lookup fails, it is because we have torn down the pipeline,
        // so it is reasonable to silently ignore the event.
        match self.pipelines.get(&parent_pipeline_id) {
            Some(pipeline) => pipeline.trigger_mozbrowser_event(frame_id, event),
            None => warn!("Pipeline {:?} handling mozbrowser event after closure.", parent_pipeline_id),
        }
    }

    fn handle_get_pipeline(&mut self, frame_id: Option<FrameId>,
                           resp_chan: IpcSender<Option<(PipelineId, bool)>>) {
        let frame_id = frame_id.unwrap_or(self.root_frame_id);
        let current_pipeline_id = self.frames.get(&frame_id)
            .map(|frame| frame.current.pipeline_id);
        let current_pipeline_id_loaded = current_pipeline_id
            .map(|id| (id, true));
        let pipeline_id_loaded = self.pending_frames.iter().rev()
            .find(|x| x.old_pipeline_id == current_pipeline_id)
            .map(|x| (x.new_pipeline_id, x.document_ready))
            .or(current_pipeline_id_loaded);
        if let Err(e) = resp_chan.send(pipeline_id_loaded) {
            warn!("Failed get_pipeline response ({}).", e);
        }
    }

    fn handle_get_frame(&mut self,
                        pipeline_id: PipelineId,
                        resp_chan: IpcSender<Option<FrameId>>) {
        let frame_id = self.pipelines.get(&pipeline_id).map(|pipeline| pipeline.frame_id);
        if let Err(e) = resp_chan.send(frame_id) {
            warn!("Failed get_frame response ({}).", e);
        }
    }

    fn focus_parent_pipeline(&mut self, pipeline_id: PipelineId) {
        let (frame_id, parent_info) = match self.pipelines.get(&pipeline_id) {
            Some(pipeline) => (pipeline.frame_id, pipeline.parent_info),
            None => return warn!("Pipeline {:?} focus parent after closure.", pipeline_id),
        };
        let (parent_pipeline_id, _) = match parent_info {
            Some(info) => info,
            None => return debug!("Pipeline {:?} focus has no parent.", pipeline_id),
        };

        // Send a message to the parent of the provided pipeline (if it exists)
        // telling it to mark the iframe element as focused.
        let msg = ConstellationControlMsg::FocusIFrame(parent_pipeline_id, frame_id);
        let result = match self.pipelines.get(&parent_pipeline_id) {
            Some(pipeline) => pipeline.script_chan.send(msg),
            None => return warn!("Pipeline {:?} focus after closure.", parent_pipeline_id),
        };
        if let Err(e) = result {
            self.handle_send_error(parent_pipeline_id, e);
        }
        self.focus_parent_pipeline(parent_pipeline_id);
    }

    fn handle_focus_msg(&mut self, pipeline_id: PipelineId) {
        self.focus_pipeline_id = Some(pipeline_id);

        // Focus parent iframes recursively
        self.focus_parent_pipeline(pipeline_id);
    }

    fn handle_remove_iframe_msg(&mut self, pipeline_id: PipelineId) {
        let frame_id = self.pipelines.get(&pipeline_id).map(|pipeline| pipeline.frame_id);
        match frame_id {
            Some(frame_id) => {
                // This iframe has already loaded and been added to the frame tree.
                self.close_frame(frame_id, ExitPipelineMode::Normal);
            }
            None => {
                // This iframe is currently loading / painting for the first time.
                // In this case, it doesn't exist in the frame tree, but the pipeline
                // still needs to be shut down.
                self.close_pipeline(pipeline_id, ExitPipelineMode::Normal);
            }
        }
    }

    fn handle_set_visible_msg(&mut self, pipeline_id: PipelineId, visible: bool) {
        let frame_id = match self.pipelines.get(&pipeline_id) {
            Some(pipeline) => pipeline.frame_id,
            None => return warn!("No frame associated with pipeline {:?}", pipeline_id),
        };

        let child_pipeline_ids: Vec<PipelineId> = self.full_frame_tree_iter(frame_id)
                                                      .flat_map(|frame| frame.next.iter()
                                                                .chain(frame.prev.iter())
                                                                .chain(once(&frame.current)))
                                                      .map(|state| state.pipeline_id)
                                                      .collect();
        for id in child_pipeline_ids {
            if let Some(pipeline) = self.pipelines.get_mut(&id) {
                pipeline.change_visibility(visible);
            }
        }
    }

    fn handle_visibility_change_complete(&mut self, pipeline_id: PipelineId, visibility: bool) {
        let (frame_id, parent_pipeline_info) = match self.pipelines.get(&pipeline_id) {
            None => return warn!("Visibity change for closed pipeline {:?}.", pipeline_id),
            Some(pipeline) => (pipeline.frame_id, pipeline.parent_info),
        };
        if let Some((parent_pipeline_id, _)) = parent_pipeline_info {
            let visibility_msg = ConstellationControlMsg::NotifyVisibilityChange(parent_pipeline_id,
                                                                                 frame_id,
                                                                                 visibility);
            let  result = match self.pipelines.get(&parent_pipeline_id) {
                None => return warn!("Parent pipeline {:?} closed", parent_pipeline_id),
                Some(parent_pipeline) => parent_pipeline.script_chan.send(visibility_msg),
            };

            if let Err(e) = result {
                self.handle_send_error(parent_pipeline_id, e);
            }
        }
    }

    fn handle_create_canvas_paint_thread_msg(
            &mut self,
            size: &Size2D<i32>,
            response_sender: IpcSender<IpcSender<CanvasMsg>>) {
        let webrender_api = self.webrender_api_sender.clone();
        let sender = CanvasPaintThread::start(*size, webrender_api,
                                              opts::get().enable_canvas_antialiasing);
        if let Err(e) = response_sender.send(sender) {
            warn!("Create canvas paint thread response failed ({})", e);
        }
    }

    fn handle_create_webgl_paint_thread_msg(
            &mut self,
            size: &Size2D<i32>,
            attributes: GLContextAttributes,
            response_sender: IpcSender<Result<(IpcSender<CanvasMsg>, GLLimits), String>>) {
        let webrender_api = self.webrender_api_sender.clone();
        let response = WebGLPaintThread::start(*size, attributes, webrender_api);

        if let Err(e) = response_sender.send(response) {
            warn!("Create WebGL paint thread response failed ({})", e);
        }
    }

    fn handle_webdriver_msg(&mut self, msg: WebDriverCommandMsg) {
        // Find the script channel for the given parent pipeline,
        // and pass the event to that script thread.
        match msg {
            WebDriverCommandMsg::GetWindowSize(_, reply) => {
               let _ = reply.send(self.window_size);
            },
            WebDriverCommandMsg::SetWindowSize(_, size, reply) => {
                self.webdriver.resize_channel = Some(reply);
                self.compositor_proxy.send(ToCompositorMsg::ResizeTo(size));
            },
            WebDriverCommandMsg::LoadUrl(pipeline_id, load_data, reply) => {
                self.load_url_for_webdriver(pipeline_id, load_data, reply, false);
            },
            WebDriverCommandMsg::Refresh(pipeline_id, reply) => {
                let load_data = match self.pipelines.get(&pipeline_id) {
                    Some(pipeline) => LoadData::new(pipeline.url.clone(), None, None),
                    None => return warn!("Pipeline {:?} Refresh after closure.", pipeline_id),
                };
                self.load_url_for_webdriver(pipeline_id, load_data, reply, true);
            }
            WebDriverCommandMsg::ScriptCommand(pipeline_id, cmd) => {
                let control_msg = ConstellationControlMsg::WebDriverScriptCommand(pipeline_id, cmd);
                let result = match self.pipelines.get(&pipeline_id) {
                    Some(pipeline) => pipeline.script_chan.send(control_msg),
                    None => return warn!("Pipeline {:?} ScriptCommand after closure.", pipeline_id),
                };
                if let Err(e) = result {
                    self.handle_send_error(pipeline_id, e);
                }
            },
            WebDriverCommandMsg::SendKeys(pipeline_id, cmd) => {
                let script_channel = match self.pipelines.get(&pipeline_id) {
                    Some(pipeline) => pipeline.script_chan.clone(),
                    None => return warn!("Pipeline {:?} SendKeys after closure.", pipeline_id),
                };
                for (key, mods, state) in cmd {
                    let event = CompositorEvent::KeyEvent(None, key, state, mods);
                    let control_msg = ConstellationControlMsg::SendEvent(pipeline_id, event);
                    if let Err(e) = script_channel.send(control_msg) {
                        return self.handle_send_error(pipeline_id, e);
                    }
                }
            },
            WebDriverCommandMsg::TakeScreenshot(pipeline_id, reply) => {
                let current_pipeline_id = self.frames.get(&self.root_frame_id)
                    .map(|root_frame| root_frame.current.pipeline_id);
                if Some(pipeline_id) == current_pipeline_id {
                    self.compositor_proxy.send(ToCompositorMsg::CreatePng(reply));
                } else {
                    if let Err(e) = reply.send(None) {
                        warn!("Screenshot reply failed ({})", e);
                    }
                }
            },
        }
    }

    fn traverse_frame_to_pipeline(&mut self, frame_id: FrameId, next_pipeline_id: PipelineId) {
        // Check if the currently focused pipeline is the pipeline being replaced
        // (or a child of it). This has to be done here, before the current
        // frame tree is modified below.
        let update_focus_pipeline = self.focused_pipeline_in_tree(frame_id);

        let prev_pipeline_id = match self.frames.get_mut(&frame_id) {
            Some(frame) => {
                let prev = frame.current.pipeline_id;

                // Check that this frame contains the pipeline passed in, so that this does not
                // change Frame's state before realizing `next_pipeline_id` is invalid.
                if frame.next.iter().find(|entry| next_pipeline_id == entry.pipeline_id).is_some() {
                    frame.prev.push(frame.current.clone());
                    while let Some(entry) = frame.next.pop() {
                        if entry.pipeline_id == next_pipeline_id {
                            frame.current = entry;
                            break;
                        } else {
                            frame.prev.push(entry);
                        }
                    }
                } else if frame.prev.iter().find(|entry| next_pipeline_id == entry.pipeline_id).is_some() {
                    frame.next.push(frame.current.clone());
                    while let Some(entry) = frame.prev.pop() {
                        if entry.pipeline_id == next_pipeline_id {
                            frame.current = entry;
                            break;
                        } else {
                            frame.next.push(entry);
                        }
                    }
                } else if prev != next_pipeline_id {
                    return warn!("Tried to traverse frame {:?} to pipeline {:?} it does not contain.",
                        frame_id, next_pipeline_id);
                }

                prev
            },
            None => return warn!("no frame to traverse"),
        };

        let pipeline_info = self.pipelines.get(&prev_pipeline_id).and_then(|p| p.parent_info);

        // If the currently focused pipeline is the one being changed (or a child
        // of the pipeline being changed) then update the focus pipeline to be
        // the replacement.
        if update_focus_pipeline {
            self.focus_pipeline_id = Some(next_pipeline_id);
        }

        // Suspend the old pipeline, and resume the new one.
        if let Some(prev_pipeline) = self.pipelines.get(&prev_pipeline_id) {
            prev_pipeline.freeze();
        }
        if let Some(next_pipeline) = self.pipelines.get(&next_pipeline_id) {
            next_pipeline.thaw();
        }

        // Set paint permissions correctly for the compositor layers.
        self.send_frame_tree();

        // Update the owning iframe to point to the new pipeline id.
        // This makes things like contentDocument work correctly.
        if let Some((parent_pipeline_id, _)) = pipeline_info {
            let msg = ConstellationControlMsg::UpdatePipelineId(parent_pipeline_id,
                                                                frame_id,
                                                                next_pipeline_id);
            let result = match self.pipelines.get(&parent_pipeline_id) {
                None => return warn!("Pipeline {:?} child traversed after closure.", parent_pipeline_id),
                Some(pipeline) => pipeline.script_chan.send(msg),
            };
            if let Err(e) = result {
                self.handle_send_error(parent_pipeline_id, e);
            }

            // If this is an iframe, send a mozbrowser location change event.
            // This is the result of a back/forward traversal.
            self.trigger_mozbrowserlocationchange(next_pipeline_id);
        }
    }

    fn get_top_level_frame_for_pipeline(&self, pipeline_id: Option<PipelineId>) -> FrameId {
        if PREFS.is_mozbrowser_enabled() {
            pipeline_id.and_then(|id| self.get_mozbrowser_ancestor_info(id))
                       .map(|(_, mozbrowser_iframe_id)| mozbrowser_iframe_id)
                       .unwrap_or(self.root_frame_id)
        } else {
            // If mozbrowser is not enabled, the root frame is the only top-level frame
            self.root_frame_id
        }
    }

    fn load_url_for_webdriver(&mut self,
                              pipeline_id: PipelineId,
                              load_data: LoadData,
                              reply: IpcSender<webdriver_msg::LoadStatus>,
                              replace: bool) {
        let new_pipeline_id = self.load_url(pipeline_id, load_data, replace);
        if let Some(id) = new_pipeline_id {
            self.webdriver.load_channel = Some((id, reply));
        }
    }

    fn add_or_replace_pipeline_in_frame_tree(&mut self, frame_change: FrameChange) {
        debug!("Setting frame {} to be pipeline {}.", frame_change.frame_id, frame_change.new_pipeline_id);

        // If the currently focused pipeline is the one being changed (or a child
        // of the pipeline being changed) then update the focus pipeline to be
        // the replacement.
        if let Some(old_pipeline_id) = frame_change.old_pipeline_id {
            if let Some(old_frame_id) = self.pipelines.get(&old_pipeline_id).map(|pipeline| pipeline.frame_id) {
                if self.focused_pipeline_in_tree(old_frame_id) {
                    self.focus_pipeline_id = Some(frame_change.new_pipeline_id);
                }
            }
        }

        if self.frames.contains_key(&frame_change.frame_id) {
            if frame_change.replace {
                let evicted = self.frames.get_mut(&frame_change.frame_id).map(|frame| {
                    frame.replace_current(frame_change.new_pipeline_id)
                });
                if let Some(evicted) = evicted {
                    self.close_pipeline(evicted.pipeline_id, ExitPipelineMode::Normal);
                }
            } else {
                if let Some(ref mut frame) = self.frames.get_mut(&frame_change.frame_id) {
                    frame.load(frame_change.new_pipeline_id);
                }
            }
        } else {
            // The new pipeline is in a new frame with no history
            self.new_frame(frame_change.frame_id, frame_change.new_pipeline_id);
        }

        if !frame_change.replace {
            // If this is an iframe, send a mozbrowser location change event.
            // This is the result of a link being clicked and a navigation completing.
            self.trigger_mozbrowserlocationchange(frame_change.new_pipeline_id);

            let top_level_frame_id = self.get_top_level_frame_for_pipeline(Some(frame_change.new_pipeline_id));
            self.clear_joint_session_future(top_level_frame_id);
        }

        // Build frame tree
        self.send_frame_tree();
    }

    fn handle_activate_document_msg(&mut self, pipeline_id: PipelineId) {
        debug!("Document ready to activate {:?}", pipeline_id);

        // Notify the parent (if there is one).
        if let Some(pipeline) = self.pipelines.get(&pipeline_id) {
            if let Some((parent_pipeline_id, _)) = pipeline.parent_info {
                if let Some(parent_pipeline) = self.pipelines.get(&parent_pipeline_id) {
                    let msg = ConstellationControlMsg::FramedContentChanged(parent_pipeline_id, pipeline.frame_id);
                    let _ = parent_pipeline.script_chan.send(msg);
                }
            }
        }

        // Find the pending frame change whose new pipeline id is pipeline_id.
        let pending_index = self.pending_frames.iter().rposition(|frame_change| {
            frame_change.new_pipeline_id == pipeline_id
        });

        // If it is found, remove it from the pending frames, and make it
        // the active document of its frame.
        if let Some(pending_index) = pending_index {
            let frame_change = self.pending_frames.swap_remove(pending_index);
            self.add_or_replace_pipeline_in_frame_tree(frame_change);
        }
    }

    /// Called when the window is resized.
    fn handle_window_size_msg(&mut self, new_size: WindowSizeData, size_type: WindowSizeType) {
        debug!("handle_window_size_msg: {:?} {:?}", new_size.initial_viewport.to_untyped(),
                                                       new_size.visible_viewport.to_untyped());

        if let Some(frame) = self.frames.get(&self.root_frame_id) {
            // Send Resize (or ResizeInactive) messages to each
            // pipeline in the frame tree.
            let pipeline_id = frame.current.pipeline_id;
            let pipeline = match self.pipelines.get(&pipeline_id) {
                None => return warn!("Pipeline {:?} resized after closing.", pipeline_id),
                Some(pipeline) => pipeline,
            };
            let _ = pipeline.script_chan.send(ConstellationControlMsg::Resize(
                pipeline.id,
                new_size,
                size_type
            ));
            for entry in frame.prev.iter().chain(&frame.next) {
                let pipeline = match self.pipelines.get(&entry.pipeline_id) {
                    None => {
                        warn!("Inactive pipeline {:?} resized after closing.", pipeline_id);
                        continue;
                    },
                    Some(pipeline) => pipeline,
                };
                let _ = pipeline.script_chan.send(ConstellationControlMsg::ResizeInactive(
                    pipeline.id,
                    new_size
                ));
            }
        }

        // Send resize message to any pending pipelines that aren't loaded yet.
        for pending_frame in &self.pending_frames {
            let pipeline_id = pending_frame.new_pipeline_id;
            let pipeline = match self.pipelines.get(&pipeline_id) {
                None => { warn!("Pending pipeline {:?} is closed", pipeline_id); continue; }
                Some(pipeline) => pipeline,
            };
            if pipeline.parent_info.is_none() {
                let _ = pipeline.script_chan.send(ConstellationControlMsg::Resize(
                    pipeline.id,
                    new_size,
                    size_type
                ));
            }
        }

        if let Some(resize_channel) = self.webdriver.resize_channel.take() {
            let _ = resize_channel.send(new_size);
        }

        self.window_size = new_size;
    }

    /// Handle updating actual viewport / zoom due to @viewport rules
    fn handle_viewport_constrained_msg(&mut self,
                                       pipeline_id: PipelineId,
                                       constraints: ViewportConstraints) {
        self.compositor_proxy.send(ToCompositorMsg::ViewportConstrained(pipeline_id, constraints));
    }

    /// Checks the state of all script and layout pipelines to see if they are idle
    /// and compares the current layout state to what the compositor has. This is used
    /// to check if the output image is "stable" and can be written as a screenshot
    /// for reftests.
    /// Since this function is only used in reftests, we do not harden it against panic.
    fn handle_is_ready_to_save_image(&mut self,
                                     pipeline_states: HashMap<PipelineId, Epoch>) -> ReadyToSave {
        // Note that this function can panic, due to ipc-channel creation failure.
        // avoiding this panic would require a mechanism for dealing
        // with low-resource scenarios.
        //
        // If there is no root frame yet, the initial page has
        // not loaded, so there is nothing to save yet.
        if !self.frames.contains_key(&self.root_frame_id) {
            return ReadyToSave::NoRootFrame;
        }

        // If there are pending loads, wait for those to complete.
        if !self.pending_frames.is_empty() {
            return ReadyToSave::PendingFrames;
        }

        let (state_sender, state_receiver) = ipc::channel().expect("Failed to create IPC channel!");
        let (epoch_sender, epoch_receiver) = ipc::channel().expect("Failed to create IPC channel!");

        // Step through the current frame tree, checking that the script
        // thread is idle, and that the current epoch of the layout thread
        // matches what the compositor has painted. If all these conditions
        // are met, then the output image should not change and a reftest
        // screenshot can safely be written.
        for frame in self.current_frame_tree_iter(self.root_frame_id) {
            let pipeline_id = frame.current.pipeline_id;
            debug!("Checking readiness of frame {}, pipeline {}.", frame.id, pipeline_id);

            let pipeline = match self.pipelines.get(&pipeline_id) {
                None => {
                    warn!("Pipeline {:?} screenshot while closing.", pipeline_id);
                    continue;
                },
                Some(pipeline) => pipeline,
            };

            // Check to see if there are any webfonts still loading.
            //
            // If GetWebFontLoadState returns false, either there are no
            // webfonts loading, or there's a WebFontLoaded message waiting in
            // script_chan's message queue. Therefore, we need to check this
            // before we check whether the document is ready; otherwise,
            // there's a race condition where a webfont has finished loading,
            // but hasn't yet notified the document.
            let msg = LayoutControlMsg::GetWebFontLoadState(state_sender.clone());
            if let Err(e) = pipeline.layout_chan.send(msg) {
                warn!("Get web font failed ({})", e);
            }
            if state_receiver.recv().unwrap_or(true) {
                return ReadyToSave::WebFontNotLoaded;
            }

            // See if this pipeline has reached idle script state yet.
            match self.document_states.get(&frame.current.pipeline_id) {
                Some(&DocumentState::Idle) => {}
                Some(&DocumentState::Pending) | None => {
                    return ReadyToSave::DocumentLoading;
                }
            }

            // Check the visible rectangle for this pipeline. If the constellation has received a
            // size for the pipeline, then its painting should be up to date. If the constellation
            // *hasn't* received a size, it could be that the layer was hidden by script before the
            // compositor discovered it, so we just don't check the layer.
            if let Some(size) = pipeline.size {
                // If the rectangle for this pipeline is zero sized, it will
                // never be painted. In this case, don't query the layout
                // thread as it won't contribute to the final output image.
                if size == TypedSize2D::zero() {
                    continue;
                }

                // Get the epoch that the compositor has drawn for this pipeline.
                let compositor_epoch = pipeline_states.get(&frame.current.pipeline_id);
                match compositor_epoch {
                    Some(compositor_epoch) => {
                        // Synchronously query the layout thread to see if the current
                        // epoch matches what the compositor has drawn. If they match
                        // (and script is idle) then this pipeline won't change again
                        // and can be considered stable.
                        let message = LayoutControlMsg::GetCurrentEpoch(epoch_sender.clone());
                        if let Err(e) = pipeline.layout_chan.send(message) {
                            warn!("Failed to send GetCurrentEpoch ({}).", e);
                        }
                        match epoch_receiver.recv() {
                            Err(e) => warn!("Failed to receive current epoch ({}).", e),
                            Ok(layout_thread_epoch) => if layout_thread_epoch != *compositor_epoch {
                                return ReadyToSave::EpochMismatch;
                            },
                        }
                    }
                    None => {
                        // The compositor doesn't know about this pipeline yet.
                        // Assume it hasn't rendered yet.
                        return ReadyToSave::PipelineUnknown;
                    }
                }
            }
        }

        // All script threads are idle and layout epochs match compositor, so output image!
        ReadyToSave::Ready
    }

    fn clear_joint_session_future(&mut self, frame_id: FrameId) {
        let mut evicted_pipelines = vec!();
        let mut frames_to_clear = vec!(frame_id);
        while let Some(frame_id) = frames_to_clear.pop() {
            let frame = match self.frames.get_mut(&frame_id) {
                Some(frame) => frame,
                None => {
                    warn!("Removed forward history after frame {:?} closure.", frame_id);
                    continue;
                }
            };
            evicted_pipelines.extend(frame.remove_forward_entries());
            for entry in frame.next.iter().chain(frame.prev.iter()).chain(once(&frame.current)) {
                let pipeline = match self.pipelines.get(&entry.pipeline_id) {
                    Some(pipeline) => pipeline,
                    None => {
                        warn!("Removed forward history after pipeline {:?} closure.", entry.pipeline_id);
                        continue;
                    }
                };
                frames_to_clear.extend_from_slice(&pipeline.children);
            }
        }
        for entry in evicted_pipelines {
            self.close_pipeline(entry.pipeline_id, ExitPipelineMode::Normal);
        }
    }

    // Close a frame (and all children)
    fn close_frame(&mut self, frame_id: FrameId, exit_mode: ExitPipelineMode) {
        // Store information about the pipelines to be closed. Then close the
        // pipelines, before removing ourself from the frames hash map. This
        // ordering is vital - so that if close_pipeline() ends up closing
        // any child frames, they can be removed from the parent frame correctly.
        let parent_info = self.frames.get(&frame_id)
            .and_then(|frame| self.pipelines.get(&frame.current.pipeline_id))
            .and_then(|pipeline| pipeline.parent_info);

        let pipelines_to_close = {
            let mut pipelines_to_close = vec!();

            if let Some(frame) = self.frames.get(&frame_id) {
                pipelines_to_close.extend_from_slice(&frame.next);
                pipelines_to_close.push(frame.current.clone());
                pipelines_to_close.extend_from_slice(&frame.prev);
            }

            pipelines_to_close
        };

        for entry in pipelines_to_close {
            self.close_pipeline(entry.pipeline_id, exit_mode);
        }

        if self.frames.remove(&frame_id).is_none() {
            warn!("Closing frame {:?} twice.", frame_id);
        }

        if let Some((parent_pipeline_id, _)) = parent_info {
            let parent_pipeline = match self.pipelines.get_mut(&parent_pipeline_id) {
                None => return warn!("Pipeline {:?} child closed after parent.", parent_pipeline_id),
                Some(parent_pipeline) => parent_pipeline,
            };
            parent_pipeline.remove_child(frame_id);
        }
    }

    // Close all pipelines at and beneath a given frame
    fn close_pipeline(&mut self, pipeline_id: PipelineId, exit_mode: ExitPipelineMode) {
        // Store information about the frames to be closed. Then close the
        // frames, before removing ourself from the pipelines hash map. This
        // ordering is vital - so that if close_frames() ends up closing
        // any child pipelines, they can be removed from the parent pipeline correctly.
        let frames_to_close = {
            let mut frames_to_close = vec!();

            if let Some(pipeline) = self.pipelines.get(&pipeline_id) {
                frames_to_close.extend_from_slice(&pipeline.children);
            }

            frames_to_close
        };

        // Remove any child frames
        for child_frame in &frames_to_close {
            self.close_frame(*child_frame, exit_mode);
        }

        // Note, we don't remove the pipeline now, we wait for the message to come back from
        // the pipeline.
        let pipeline = match self.pipelines.get(&pipeline_id) {
            Some(pipeline) => pipeline,
            None => return warn!("Closing pipeline {:?} twice.", pipeline_id),
        };

        // Remove this pipeline from pending frames if it hasn't loaded yet.
        let pending_index = self.pending_frames.iter().position(|frame_change| {
            frame_change.new_pipeline_id == pipeline_id
        });
        if let Some(pending_index) = pending_index {
            self.pending_frames.remove(pending_index);
        }

        // Inform script, compositor that this pipeline has exited.
        match exit_mode {
            ExitPipelineMode::Normal => pipeline.exit(),
            ExitPipelineMode::Force => pipeline.force_exit(),
        }
    }

    // Randomly close a pipeline -if --random-pipeline-closure-probability is set
    fn maybe_close_random_pipeline(&mut self) {
        match self.random_pipeline_closure {
            Some((ref mut rng, probability)) => if probability <= rng.gen::<f32>() { return },
            _ => return,
        };
        // In order to get repeatability, we sort the pipeline ids.
        let mut pipeline_ids: Vec<&PipelineId> = self.pipelines.keys().collect();
        pipeline_ids.sort();
        if let Some((ref mut rng, _)) = self.random_pipeline_closure {
            if let Some(pipeline_id) = rng.choose(&*pipeline_ids) {
                if let Some(pipeline) = self.pipelines.get(pipeline_id) {
                    // Don't kill the mozbrowser pipeline
                    if PREFS.is_mozbrowser_enabled() && pipeline.parent_info.is_none() {
                        info!("Not closing mozbrowser pipeline {}.", pipeline_id);
                    } else {
                        // Note that we deliberately do not do any of the tidying up
                        // associated with closing a pipeline. The constellation should cope!
                        warn!("Randomly closing pipeline {}.", pipeline_id);
                        pipeline.force_exit();
                    }
                }
            }
        }
    }

    // Convert a frame to a sendable form to pass to the compositor
    fn frame_to_sendable(&self, frame_id: FrameId) -> Option<SendableFrameTree> {
        self.frames.get(&frame_id).and_then(|frame: &Frame| {
            self.pipelines.get(&frame.current.pipeline_id).map(|pipeline: &Pipeline| {
                let mut frame_tree = SendableFrameTree {
                    pipeline: pipeline.to_sendable(),
                    size: pipeline.size,
                    children: vec!(),
                };

                for child_frame_id in &pipeline.children {
                    if let Some(frame) = self.frame_to_sendable(*child_frame_id) {
                        frame_tree.children.push(frame);
                    }
                }

                frame_tree
            })
        })
    }

    // Send the current frame tree to compositor
    fn send_frame_tree(&mut self) {
        // Note that this function can panic, due to ipc-channel creation failure.
        // avoiding this panic would require a mechanism for dealing
        // with low-resource scenarios.
        debug!("Sending frame tree for frame {}.", self.root_frame_id);
        if let Some(frame_tree) = self.frame_to_sendable(self.root_frame_id) {
            let (chan, port) = ipc::channel().expect("Failed to create IPC channel!");
            self.compositor_proxy.send(ToCompositorMsg::SetFrameTree(frame_tree,
                                                                     chan));
            if port.recv().is_err() {
                warn!("Compositor has discarded SetFrameTree");
                return; // Our message has been discarded, probably shutting down.
            }
        }
    }

    /// For a given pipeline, determine the mozbrowser iframe that transitively contains
    /// it. There could be arbitrary levels of nested iframes in between them.
    fn get_mozbrowser_ancestor_info(&self, original_pipeline_id: PipelineId) -> Option<(PipelineId, FrameId)> {
        let mut pipeline_id = original_pipeline_id;
        loop {
            match self.pipelines.get(&pipeline_id) {
                Some(pipeline) => match pipeline.parent_info {
                    Some((parent_id, FrameType::MozBrowserIFrame)) => return Some((parent_id, pipeline.frame_id)),
                    Some((parent_id, _)) => pipeline_id = parent_id,
                    None => return None,
                },
                None => {
                    warn!("Finding mozbrowser ancestor for pipeline {} after closure.", pipeline_id);
                    return None;
                },
            }
        }
    }

    // https://developer.mozilla.org/en-US/docs/Web/Events/mozbrowserlocationchange
    // Note that this is a no-op if the pipeline is not a mozbrowser iframe
    fn trigger_mozbrowserlocationchange(&self, pipeline_id: PipelineId) {
        if !PREFS.is_mozbrowser_enabled() { return; }

        let url = match self.pipelines.get(&pipeline_id) {
            Some(pipeline) => pipeline.url.to_string(),
            None => return warn!("triggered mozbrowser location change on closed pipeline {:?}", pipeline_id),
        };

        // If this is a mozbrowser iframe, then send the event with new url
        if let Some((ancestor_id, mozbrowser_frame_id)) = self.get_mozbrowser_ancestor_info(pipeline_id) {
            if let Some(ancestor) = self.pipelines.get(&ancestor_id) {
                let can_go_forward = !self.joint_session_future(mozbrowser_frame_id).is_empty();
                let can_go_back = !self.joint_session_past(mozbrowser_frame_id).is_empty();
                let event = MozBrowserEvent::LocationChange(url, can_go_back, can_go_forward);
                ancestor.trigger_mozbrowser_event(Some(mozbrowser_frame_id), event);
            }
        }
    }

    // https://developer.mozilla.org/en-US/docs/Web/Events/mozbrowsererror
    // Note that this does not require the pipeline to be an immediate child of the root
    fn trigger_mozbrowsererror(&mut self, pipeline_id: Option<PipelineId>, reason: String, backtrace: Option<String>) {
        if !PREFS.is_mozbrowser_enabled() { return; }

        let mut report = String::new();
        for (thread_name, warning) in self.handled_warnings.drain(..) {
            report.push_str("\nWARNING: ");
            if let Some(thread_name) = thread_name {
                report.push_str("<");
                report.push_str(&*thread_name);
                report.push_str(">: ");
            }
            report.push_str(&*warning);
        }
        report.push_str("\nERROR: ");
        report.push_str(&*reason);
        if let Some(backtrace) = backtrace {
            report.push_str("\n\n");
            report.push_str(&*backtrace);
        }

        let event = MozBrowserEvent::Error(MozBrowserErrorType::Fatal, reason, report);

        if let Some(pipeline_id) = pipeline_id {
            if let Some((ancestor_id, mozbrowser_iframe_id)) = self.get_mozbrowser_ancestor_info(pipeline_id) {
                if let Some(ancestor) = self.pipelines.get(&ancestor_id) {
                    return ancestor.trigger_mozbrowser_event(Some(mozbrowser_iframe_id), event);
                }
            }
        }

        if let Some(root_frame) = self.frames.get(&self.root_frame_id) {
            if let Some(root_pipeline) = self.pipelines.get(&root_frame.current.pipeline_id) {
                return root_pipeline.trigger_mozbrowser_event(None, event);
            }
        }

        warn!("Mozbrowser error after root pipeline closed.");
    }

    fn focused_pipeline_in_tree(&self, frame_id: FrameId) -> bool {
        self.focus_pipeline_id.map_or(false, |pipeline_id| {
            self.pipeline_exists_in_tree(pipeline_id, frame_id)
        })
    }

    fn pipeline_is_in_current_frame(&self, pipeline_id: PipelineId) -> bool {
        self.pipeline_exists_in_tree(pipeline_id, self.root_frame_id)
    }

    fn pipeline_exists_in_tree(&self,
                               pipeline_id: PipelineId,
                               root_frame_id: FrameId) -> bool {
        self.current_frame_tree_iter(root_frame_id)
            .any(|current_frame| current_frame.current.pipeline_id == pipeline_id)
    }

}
