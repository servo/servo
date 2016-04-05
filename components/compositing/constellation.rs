/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The `Constellation`, Servo's Grand Central Station
//!
//! The primary duty of a `Constellation` is to mediate between the
//! graphics compositor and the many `Pipeline`s in the browser's
//! navigation context, each `Pipeline` encompassing a `ScriptThread`,
//! `LayoutThread`, and `PaintThread`.

use AnimationTickType;
use CompositorMsg as FromCompositorMsg;
use canvas::canvas_paint_thread::CanvasPaintThread;
use canvas::webgl_paint_thread::WebGLPaintThread;
use canvas_traits::CanvasMsg;
use clipboard::ClipboardContext;
use compositor_thread::CompositorProxy;
use compositor_thread::Msg as ToCompositorMsg;
use devtools_traits::{ChromeToDevtoolsControlMsg, DevtoolsControlMsg};
use euclid::scale_factor::ScaleFactor;
use euclid::size::{Size2D, TypedSize2D};
#[cfg(not(target_os = "windows"))]
use gaol;
#[cfg(not(target_os = "windows"))]
use gaol::sandbox::{self, Sandbox, SandboxMethods};
use gfx::font_cache_thread::FontCacheThread;
use gfx_traits::{Epoch, PaintMsg as FromPaintMsg};
use ipc_channel::ipc::{self, IpcOneShotServer, IpcSender};
use ipc_channel::router::ROUTER;
use layout_traits::{LayoutControlChan, LayoutThreadFactory};
use msg::constellation_msg::WebDriverCommandMsg;
use msg::constellation_msg::{FrameId, PipelineId};
use msg::constellation_msg::{Key, KeyModifiers, KeyState, LoadData};
use msg::constellation_msg::{PipelineNamespace, PipelineNamespaceId, NavigationDirection};
use msg::constellation_msg::{SubpageId, WindowSizeData};
use msg::constellation_msg::{self, ConstellationChan, Failure};
use msg::webdriver_msg;
use net_traits::image_cache_thread::ImageCacheThread;
use net_traits::storage_thread::{StorageThread, StorageThreadMsg};
use net_traits::{self, ResourceThread};
use offscreen_gl_context::GLContextAttributes;
use pipeline::{CompositionPipeline, InitialPipelineState, Pipeline, UnprivilegedPipelineContent};
use profile_traits::mem;
use profile_traits::time;
use rand::{random, Rng, SeedableRng, StdRng};
#[cfg(not(target_os = "windows"))]
use sandboxing;
use script_traits::{AnimationState, CompositorEvent, ConstellationControlMsg};
use script_traits::{DocumentState, LayoutControlMsg};
use script_traits::{IFrameLoadInfo, IFrameSandboxState, MozBrowserEvent, TimerEventRequest};
use script_traits::{LayoutMsg as FromLayoutMsg, ScriptMsg as FromScriptMsg, ScriptThreadFactory};
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::env;
use std::io::{self, Write};
use std::marker::PhantomData;
use std::mem::replace;
use std::process;
use std::sync::mpsc::{Sender, channel, Receiver};
use style_traits::cursor::Cursor;
use style_traits::viewport::ViewportConstraints;
use timer_scheduler::TimerScheduler;
use url::Url;
use util::geometry::PagePx;
use util::thread::spawn_named;
use util::{opts, prefs};
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
pub struct Constellation<LTF, STF> {
    /// A channel through which script messages can be sent to this object.
    pub script_sender: ConstellationChan<FromScriptMsg>,

    /// A channel through which compositor messages can be sent to this object.
    pub compositor_sender: Sender<FromCompositorMsg>,

    /// A channel through which layout thread messages can be sent to this object.
    pub layout_sender: ConstellationChan<FromLayoutMsg>,

    /// A channel through which paint thread messages can be sent to this object.
    pub painter_sender: ConstellationChan<FromPaintMsg>,

    /// Receives messages from scripts.
    pub script_receiver: Receiver<FromScriptMsg>,

    /// Receives messages from the compositor
    pub compositor_receiver: Receiver<FromCompositorMsg>,

    /// Receives messages from the layout thread
    pub layout_receiver: Receiver<FromLayoutMsg>,

    /// Receives messages from paint thread.
    pub painter_receiver: Receiver<FromPaintMsg>,

    /// A channel (the implementation of which is port-specific) through which messages can be sent
    /// to the compositor.
    pub compositor_proxy: Box<CompositorProxy>,

    /// A channel through which messages can be sent to the resource thread.
    pub resource_thread: ResourceThread,

    /// A channel through which messages can be sent to the image cache thread.
    pub image_cache_thread: ImageCacheThread,

    /// A channel through which messages can be sent to the developer tools.
    devtools_chan: Option<Sender<DevtoolsControlMsg>>,

    /// A channel through which messages can be sent to the storage thread.
    storage_thread: StorageThread,

    /// A list of all the pipelines. (See the `pipeline` module for more details.)
    pipelines: HashMap<PipelineId, Pipeline>,

    /// A list of all the frames
    frames: HashMap<FrameId, Frame>,

    /// Maps from pipeline ID to the frame that contains it.
    pipeline_to_frame_map: HashMap<PipelineId, FrameId>,

    /// Maps from a (parent pipeline, subpage) to the actual child pipeline ID.
    subpage_map: HashMap<(PipelineId, SubpageId), PipelineId>,

    /// A channel through which messages can be sent to the font cache.
    font_cache_thread: FontCacheThread,

    /// ID of the root frame.
    root_frame_id: Option<FrameId>,

    /// The next free ID to assign to a pipeline ID namespace.
    next_pipeline_namespace_id: PipelineNamespaceId,

    /// The next free ID to assign to a frame.
    next_frame_id: FrameId,

    /// Pipeline ID that has currently focused element for key events.
    focus_pipeline_id: Option<PipelineId>,

    /// Navigation operations that are in progress.
    pending_frames: Vec<FrameChange>,

    /// A channel through which messages can be sent to the time profiler.
    pub time_profiler_chan: time::ProfilerChan,

    /// A channel through which messages can be sent to the memory profiler.
    pub mem_profiler_chan: mem::ProfilerChan,

    phantom: PhantomData<(LTF, STF)>,

    pub window_size: WindowSizeData,

    /// Means of accessing the clipboard
    clipboard_ctx: Option<ClipboardContext>,

    /// Bits of state used to interact with the webdriver implementation
    webdriver: WebDriverData,

    scheduler_chan: IpcSender<TimerEventRequest>,

    /// A list of child content processes.
    child_processes: Vec<ChildProcess>,

    /// Document states for loaded pipelines (used only when writing screenshots).
    document_states: HashMap<PipelineId, DocumentState>,

    // Webrender interface, if enabled.
    webrender_api_sender: Option<webrender_traits::RenderApiSender>,

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
    /// A channel to the image cache thread.
    pub image_cache_thread: ImageCacheThread,
    /// A channel to the font cache thread.
    pub font_cache_thread: FontCacheThread,
    /// A channel to the resource thread.
    pub resource_thread: ResourceThread,
    /// A channel to the storage thread.
    pub storage_thread: StorageThread,
    /// A channel to the time profiler thread.
    pub time_profiler_chan: time::ProfilerChan,
    /// A channel to the memory profiler thread.
    pub mem_profiler_chan: mem::ProfilerChan,
    /// Whether the constellation supports the clipboard.
    pub supports_clipboard: bool,
    /// Optional webrender API reference (if enabled).
    pub webrender_api_sender: Option<webrender_traits::RenderApiSender>,
}

/// Stores the navigation context for a single frame in the frame tree.
pub struct Frame {
    prev: Vec<PipelineId>,
    current: PipelineId,
    next: Vec<PipelineId>,
}

impl Frame {
    fn new(pipeline_id: PipelineId) -> Frame {
        Frame {
            prev: vec!(),
            current: pipeline_id,
            next: vec!(),
        }
    }

    fn load(&mut self, pipeline_id: PipelineId) -> Vec<PipelineId> {
        // TODO(gw): To also allow navigations within subframes
        // to affect the parent navigation history, this should bubble
        // up the navigation change to each parent.
        self.prev.push(self.current);
        self.current = pipeline_id;
        replace(&mut self.next, vec!())
    }
}

/// Represents a pending change in the frame tree, that will be applied
/// once the new pipeline has loaded and completed initial layout / paint.
struct FrameChange {
    old_pipeline_id: Option<PipelineId>,
    new_pipeline_id: PipelineId,
    document_ready: bool,
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
                    debug!("Frame {:?} iterated after closure.", frame_id);
                    continue;
                },
            };
            let pipeline = match self.pipelines.get(&frame.current) {
                Some(pipeline) => pipeline,
                None => {
                    debug!("Pipeline {:?} iterated after closure.", frame.current);
                    continue;
                },
            };
            self.stack.extend(pipeline.children.iter().map(|&c| c));
            return Some(frame)
        }
    }
}

pub struct SendableFrameTree {
    pub pipeline: CompositionPipeline,
    pub size: Option<TypedSize2D<PagePx, f32>>,
    pub children: Vec<SendableFrameTree>,
}

struct WebDriverData {
    load_channel: Option<(PipelineId, IpcSender<webdriver_msg::LoadStatus>)>
}

impl WebDriverData {
    pub fn new() -> WebDriverData {
        WebDriverData {
            load_channel: None
        }
    }
}

#[derive(Clone, Copy)]
enum ExitPipelineMode {
    Normal,
    Force,
}

enum ChildProcess {
#[cfg(not(target_os = "windows"))]
    Sandboxed(gaol::platform::process::Process),
    Unsandboxed(process::Child),
}

impl<LTF: LayoutThreadFactory, STF: ScriptThreadFactory> Constellation<LTF, STF> {
    pub fn start(state: InitialConstellationState) -> Sender<FromCompositorMsg> {
        let (ipc_script_receiver, ipc_script_sender) = ConstellationChan::<FromScriptMsg>::new();
        let script_receiver = ROUTER.route_ipc_receiver_to_new_mpsc_receiver(ipc_script_receiver);
        let (compositor_sender, compositor_receiver) = channel();
        let (ipc_layout_receiver, ipc_layout_sender) = ConstellationChan::<FromLayoutMsg>::new();
        let layout_receiver = ROUTER.route_ipc_receiver_to_new_mpsc_receiver(ipc_layout_receiver);
        let (ipc_painter_receiver, ipc_painter_sender) = ConstellationChan::<FromPaintMsg>::new();
        let painter_receiver = ROUTER.route_ipc_receiver_to_new_mpsc_receiver(ipc_painter_receiver);
        let compositor_sender_clone = compositor_sender.clone();
        spawn_named("Constellation".to_owned(), move || {
            let mut constellation: Constellation<LTF, STF> = Constellation {
                script_sender: ipc_script_sender,
                compositor_sender: compositor_sender_clone,
                layout_sender: ipc_layout_sender,
                painter_sender: ipc_painter_sender,
                script_receiver: script_receiver,
                compositor_receiver: compositor_receiver,
                layout_receiver: layout_receiver,
                painter_receiver: painter_receiver,
                compositor_proxy: state.compositor_proxy,
                devtools_chan: state.devtools_chan,
                resource_thread: state.resource_thread,
                image_cache_thread: state.image_cache_thread,
                font_cache_thread: state.font_cache_thread,
                storage_thread: state.storage_thread,
                pipelines: HashMap::new(),
                frames: HashMap::new(),
                pipeline_to_frame_map: HashMap::new(),
                subpage_map: HashMap::new(),
                pending_frames: vec!(),
                next_pipeline_namespace_id: PipelineNamespaceId(0),
                root_frame_id: None,
                next_frame_id: FrameId(0),
                focus_pipeline_id: None,
                time_profiler_chan: state.time_profiler_chan,
                mem_profiler_chan: state.mem_profiler_chan,
                window_size: WindowSizeData {
                    visible_viewport: opts::get().initial_window_size.as_f32() *
                                          ScaleFactor::new(1.0),
                    initial_viewport: opts::get().initial_window_size.as_f32() *
                        ScaleFactor::new(1.0),
                    device_pixel_ratio:
                        ScaleFactor::new(opts::get().device_pixels_per_px.unwrap_or(1.0)),
                },
                phantom: PhantomData,
                clipboard_ctx: if state.supports_clipboard {
                    ClipboardContext::new().ok()
                } else {
                    None
                },
                webdriver: WebDriverData::new(),
                scheduler_chan: TimerScheduler::start(),
                child_processes: Vec::new(),
                document_states: HashMap::new(),
                webrender_api_sender: state.webrender_api_sender,
                random_pipeline_closure: opts::get().random_pipeline_closure_probability.map(|prob| {
                    let seed = opts::get().random_pipeline_closure_seed.unwrap_or_else(random);
                    let rng = StdRng::from_seed(&[seed]);
                    warn!("Randomly closing pipelines.");
                    info!("Using seed {} for random pipeline closure.", seed);
                    (rng, prob)
                }),
            };
            let namespace_id = constellation.next_pipeline_namespace_id();
            PipelineNamespace::install(namespace_id);
            constellation.run();
        });
        compositor_sender
    }

    fn run(&mut self) {
        loop {
            // Randomly close a pipeline if --random-pipeline-closure-probability is set
            // This is for testing the hardening of the constellation.
            self.maybe_close_random_pipeline();
            if !self.handle_request() {
                break;
            }
        }
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
                    parent_info: Option<(PipelineId, SubpageId)>,
                    initial_window_size: Option<TypedSize2D<PagePx, f32>>,
                    script_channel: Option<IpcSender<ConstellationControlMsg>>,
                    load_data: LoadData) {
        let spawning_paint_only = script_channel.is_some();
        let (pipeline, unprivileged_pipeline_content, privileged_pipeline_content) =
            Pipeline::create::<LTF, STF>(InitialPipelineState {
                id: pipeline_id,
                parent_info: parent_info,
                constellation_chan: self.script_sender.clone(),
                layout_to_constellation_chan: self.layout_sender.clone(),
                painter_chan: self.painter_sender.clone(),
                scheduler_chan: self.scheduler_chan.clone(),
                compositor_proxy: self.compositor_proxy.clone_compositor_proxy(),
                devtools_chan: self.devtools_chan.clone(),
                image_cache_thread: self.image_cache_thread.clone(),
                font_cache_thread: self.font_cache_thread.clone(),
                resource_thread: self.resource_thread.clone(),
                storage_thread: self.storage_thread.clone(),
                time_profiler_chan: self.time_profiler_chan.clone(),
                mem_profiler_chan: self.mem_profiler_chan.clone(),
                window_size: initial_window_size,
                script_chan: script_channel,
                load_data: load_data,
                device_pixel_ratio: self.window_size.device_pixel_ratio,
                pipeline_namespace_id: self.next_pipeline_namespace_id(),
                webrender_api_sender: self.webrender_api_sender.clone(),
            });

        if spawning_paint_only {
            privileged_pipeline_content.start_paint_thread();
        } else {
            privileged_pipeline_content.start_all();

            // Spawn the child process.
            //
            // Yes, that's all there is to it!
            if opts::multiprocess() {
                self.spawn_multiprocess(pipeline_id, unprivileged_pipeline_content);
            } else {
                unprivileged_pipeline_content.start_all::<LTF, STF>(false);
            }
        }

        assert!(!self.pipelines.contains_key(&pipeline_id));
        self.pipelines.insert(pipeline_id, pipeline);
    }

    #[cfg(not(target_os = "windows"))]
    fn spawn_multiprocess(&mut self,
                          pipeline_id: PipelineId,
                          unprivileged_pipeline_content: UnprivilegedPipelineContent)
    {
        // Note that this function can panic, due to process creation,
        // avoiding this panic would require a mechanism for dealing
        // with low-resource scenarios.
        let (server, token) =
            IpcOneShotServer::<IpcSender<UnprivilegedPipelineContent>>::new().unwrap();

        // If there is a sandbox, use the `gaol` API to create the child process.
        let child_process = if opts::get().sandbox {
            let mut command = sandbox::Command::me().unwrap();
            command.arg("--content-process").arg(token);
            let profile = sandboxing::content_process_sandbox_profile();
            ChildProcess::Sandboxed(Sandbox::new(profile).start(&mut command).expect(
                "Failed to start sandboxed child process!"))
        } else {
            let path_to_self = env::current_exe().unwrap();
            let mut child_process = process::Command::new(path_to_self);
            child_process.arg("--content-process");
            child_process.arg(token);
            ChildProcess::Unsandboxed(child_process.spawn().unwrap())
        };

        self.child_processes.push(child_process);
        let (_receiver, sender) = server.accept().unwrap();
        sender.send(unprivileged_pipeline_content)
            .unwrap_or_else(|_| self.handle_send_error(pipeline_id));
    }

    #[cfg(target_os = "windows")]
    fn spawn_multiprocess(&mut self, _: PipelineId, _: UnprivilegedPipelineContent) {
        panic!("Multiprocess is not supported on Windows.");
    }

    // Push a new (loading) pipeline to the list of pending frame changes
    fn push_pending_frame(&mut self, new_pipeline_id: PipelineId,
                          old_pipeline_id: Option<PipelineId>) {
        self.pending_frames.push(FrameChange {
            old_pipeline_id: old_pipeline_id,
            new_pipeline_id: new_pipeline_id,
            document_ready: false,
        });
    }

    // Get an iterator for the current frame tree. Specify self.root_frame_id to
    // iterate the entire tree, or a specific frame id to iterate only that sub-tree.
    fn current_frame_tree_iter(&self, frame_id_root: Option<FrameId>) -> FrameTreeIterator {
        FrameTreeIterator {
            stack: frame_id_root.into_iter().collect(),
            pipelines: &self.pipelines,
            frames: &self.frames,
        }
    }

    // Create a new frame and update the internal bookkeeping.
    fn new_frame(&mut self, pipeline_id: PipelineId) -> FrameId {
        let id = self.next_frame_id;
        let FrameId(ref mut i) = self.next_frame_id;
        *i += 1;

        let frame = Frame::new(pipeline_id);

        assert!(!self.pipeline_to_frame_map.contains_key(&pipeline_id));
        assert!(!self.frames.contains_key(&id));

        self.pipeline_to_frame_map.insert(pipeline_id, id);
        self.frames.insert(id, frame);

        id
    }

    /// Handles loading pages, navigation, and granting access to the compositor
    #[allow(unsafe_code)]
    fn handle_request(&mut self) -> bool {
        enum Request {
            Script(FromScriptMsg),
            Compositor(FromCompositorMsg),
            Layout(FromLayoutMsg),
            Paint(FromPaintMsg)
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
            let receiver_from_paint = &self.painter_receiver;
            select! {
                msg = receiver_from_script.recv() =>
                    Request::Script(msg.expect("Unexpected script failure in constellation")),
                msg = receiver_from_compositor.recv() =>
                    Request::Compositor(msg.expect("Unexpected compositor failure in constellation")),
                msg = receiver_from_layout.recv() =>
                    Request::Layout(msg.expect("Unexpected layout failure in constellation")),
                msg = receiver_from_paint.recv() =>
                    Request::Paint(msg.expect("Unexpected paint failure in constellation"))
            }
        };

        // Process the request.
        match request {
            // Messages from compositor


            Request::Compositor(FromCompositorMsg::Exit) => {
                debug!("constellation exiting");
                self.handle_exit();
                return false;
            }
            // The compositor discovered the size of a subframe. This needs to be reflected by all
            // frame trees in the navigation context containing the subframe.
            Request::Compositor(FromCompositorMsg::FrameSize(pipeline_id, size)) => {
                debug!("constellation got frame size message");
                self.handle_frame_size_msg(pipeline_id, &Size2D::from_untyped(&size));
            }
            Request::Compositor(FromCompositorMsg::GetFrame(pipeline_id, resp_chan)) => {
                debug!("constellation got get root pipeline message");
                self.handle_get_frame(pipeline_id, resp_chan);
            }
            Request::Compositor(FromCompositorMsg::GetPipeline(frame_id, resp_chan)) => {
                debug!("constellation got get root pipeline message");
                self.handle_get_pipeline(frame_id, resp_chan);
            }
            Request::Compositor(FromCompositorMsg::GetPipelineTitle(pipeline_id)) => {
                debug!("constellation got get-pipeline-title message");
                self.handle_get_pipeline_title_msg(pipeline_id);
            }
            Request::Compositor(FromCompositorMsg::KeyEvent(key, state, modifiers)) => {
                debug!("constellation got key event message");
                self.handle_key_msg(key, state, modifiers);
            }
            // Load a new page from a typed url
            // If there is already a pending page (self.pending_frames), it will not be overridden;
            // However, if the id is not encompassed by another change, it will be.
            Request::Compositor(FromCompositorMsg::LoadUrl(source_id, load_data)) => {
                debug!("constellation got URL load message from compositor");
                self.handle_load_url_msg(source_id, load_data);
            }
            Request::Compositor(FromCompositorMsg::IsReadyToSaveImage(pipeline_states)) => {
                let is_ready = self.handle_is_ready_to_save_image(pipeline_states);
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
            Request::Compositor(FromCompositorMsg::InitLoadUrl(url)) => {
                debug!("constellation got init load URL message");
                self.handle_init_load(url);
            }
            // Handle a forward or back request
            Request::Compositor(FromCompositorMsg::Navigate(pipeline_info, direction)) => {
                debug!("constellation got navigation message from compositor");
                self.handle_navigate_msg(pipeline_info, direction);
            }
            Request::Compositor(FromCompositorMsg::ResizedWindow(new_size)) => {
                debug!("constellation got window resize message");
                self.handle_resized_window_msg(new_size);
            }
            Request::Compositor(FromCompositorMsg::TickAnimation(pipeline_id, tick_type)) => {
                self.handle_tick_animation(pipeline_id, tick_type)
            }
            Request::Compositor(FromCompositorMsg::WebDriverCommand(command)) => {
                debug!("constellation got webdriver command message");
                self.handle_webdriver_msg(command);
            }


            // Messages from script


            Request::Script(FromScriptMsg::Failure(Failure { pipeline_id, parent_info })) => {
                debug!("handling script failure message from pipeline {:?}, {:?}", pipeline_id, parent_info);
                self.handle_failure_msg(pipeline_id, parent_info);
            }
            Request::Script(FromScriptMsg::ScriptLoadedURLInIFrame(load_info)) => {
                debug!("constellation got iframe URL load message {:?} {:?} {:?}",
                       load_info.containing_pipeline_id,
                       load_info.old_subpage_id,
                       load_info.new_subpage_id);
                self.handle_script_loaded_url_in_iframe_msg(load_info);
            }
            Request::Script(FromScriptMsg::ChangeRunningAnimationsState(pipeline_id, animation_state)) => {
                self.handle_change_running_animations_state(pipeline_id, animation_state)
            }
            // Load a new page from a mouse click
            // If there is already a pending page (self.pending_frames), it will not be overridden;
            // However, if the id is not encompassed by another change, it will be.
            Request::Script(FromScriptMsg::LoadUrl(source_id, load_data)) => {
                debug!("constellation got URL load message from script");
                self.handle_load_url_msg(source_id, load_data);
            }
            // A page loaded has completed all parsing, script, and reflow messages have been sent.
            Request::Script(FromScriptMsg::LoadComplete(pipeline_id)) => {
                debug!("constellation got load complete message");
                self.handle_load_complete_msg(&pipeline_id)
            }
            // The DOM load event fired on a document
            Request::Script(FromScriptMsg::DOMLoad(pipeline_id)) => {
                debug!("constellation got dom load message");
                self.handle_dom_load(pipeline_id)
            }
            // Handle a forward or back request
            Request::Script(FromScriptMsg::Navigate(pipeline_info, direction)) => {
                debug!("constellation got navigation message from script");
                self.handle_navigate_msg(pipeline_info, direction);
            }
            // Notification that the new document is ready to become active
            Request::Script(FromScriptMsg::ActivateDocument(pipeline_id)) => {
                debug!("constellation got activate document message");
                self.handle_activate_document_msg(pipeline_id);
            }
            // Update pipeline url after redirections
            Request::Script(FromScriptMsg::SetFinalUrl(pipeline_id, final_url)) => {
                // The script may have finished loading after we already started shutting down.
                if let Some(ref mut pipeline) = self.pipelines.get_mut(&pipeline_id) {
                    debug!("constellation got set final url message");
                    pipeline.url = final_url;
                } else {
                    debug!("constellation got set final url message for dead pipeline");
                }
            }
            Request::Script(FromScriptMsg::MozBrowserEvent(pipeline_id,
                                              subpage_id,
                                              event)) => {
                debug!("constellation got mozbrowser event message");
                self.handle_mozbrowser_event_msg(pipeline_id,
                                                 subpage_id,
                                                 event);
            }
            Request::Script(FromScriptMsg::Focus(pipeline_id)) => {
                debug!("constellation got focus message");
                self.handle_focus_msg(pipeline_id);
            }
            Request::Script(FromScriptMsg::ForwardMouseButtonEvent(
                    pipeline_id, event_type, button, point)) => {
                let event = CompositorEvent::MouseButtonEvent(event_type, button, point);
                let msg = ConstellationControlMsg::SendEvent(pipeline_id, event);
                self.pipelines.get(&pipeline_id)
                    .and_then(|pipeline| pipeline.script_chan.send(msg).ok())
                    .unwrap_or_else(|| self.handle_send_error(pipeline_id));
            }
            Request::Script(FromScriptMsg::ForwardMouseMoveEvent(pipeline_id, point)) => {
                let event = CompositorEvent::MouseMoveEvent(Some(point));
                let msg = ConstellationControlMsg::SendEvent(pipeline_id, event);
                self.pipelines.get(&pipeline_id)
                    .and_then(|pipeline| pipeline.script_chan.send(msg).ok())
                    .unwrap_or_else(|| self.handle_send_error(pipeline_id));
            }
            Request::Script(FromScriptMsg::GetClipboardContents(sender)) => {
                let result = self.clipboard_ctx.as_ref().map_or(
                    "".to_owned(),
                    |ctx| ctx.get_contents().unwrap_or_else(|e| {
                        debug!("Error getting clipboard contents ({}), defaulting to empty string", e);
                        "".to_owned()
                    })
                );
                sender.send(result).unwrap_or_else(|e| debug!("Failed to send clipboard ({})", e))
            }
            Request::Script(FromScriptMsg::SetClipboardContents(s)) => {
                if let Some(ref mut ctx) = self.clipboard_ctx {
                    if let Err(e) = ctx.set_contents(s) {
                        debug!("Error setting clipboard contents ({})", e);
                    }
                }
            }
            Request::Script(FromScriptMsg::RemoveIFrame(pipeline_id, sender)) => {
                debug!("constellation got remove iframe message");
                self.handle_remove_iframe_msg(pipeline_id);
                if let Some(sender) = sender {
                    sender.send(()).unwrap_or_else(|e| debug!("Error replying to remove iframe ({})", e));
                }
            }
            Request::Script(FromScriptMsg::NewFavicon(url)) => {
                debug!("constellation got new favicon message");
                self.compositor_proxy.send(ToCompositorMsg::NewFavicon(url));
            }
            Request::Script(FromScriptMsg::HeadParsed) => {
                debug!("constellation got head parsed message");
                self.compositor_proxy.send(ToCompositorMsg::HeadParsed);
            }
            Request::Script(FromScriptMsg::CreateCanvasPaintThread(size, sender)) => {
                debug!("constellation got create-canvas-paint-thread message");
                self.handle_create_canvas_paint_thread_msg(&size, sender)
            }
            Request::Script(FromScriptMsg::CreateWebGLPaintThread(size, attributes, sender)) => {
                debug!("constellation got create-WebGL-paint-thread message");
                self.handle_create_webgl_paint_thread_msg(&size, attributes, sender)
            }
            Request::Script(FromScriptMsg::NodeStatus(message)) => {
                debug!("constellation got NodeStatus message");
                self.compositor_proxy.send(ToCompositorMsg::Status(message));
            }
            Request::Script(FromScriptMsg::SetDocumentState(pipeline_id, state)) => {
                debug!("constellation got SetDocumentState message");
                self.document_states.insert(pipeline_id, state);
            }


            // Messages from layout thread

            Request::Layout(FromLayoutMsg::ChangeRunningAnimationsState(pipeline_id, animation_state)) => {
                self.handle_change_running_animations_state(pipeline_id, animation_state)
            }
            Request::Layout(FromLayoutMsg::Failure(Failure { pipeline_id, parent_info })) => {
                debug!("handling paint failure message from pipeline {:?}, {:?}", pipeline_id, parent_info);
                self.handle_failure_msg(pipeline_id, parent_info);
            }
            Request::Layout(FromLayoutMsg::SetCursor(cursor)) => {
                self.handle_set_cursor_msg(cursor)
            }
            Request::Layout(FromLayoutMsg::ViewportConstrained(pipeline_id, constraints)) => {
                debug!("constellation got viewport-constrained event message");
                self.handle_viewport_constrained_msg(pipeline_id, constraints);
            }


            // Messages from paint thread


            // Notification that painting has finished and is requesting permission to paint.
            Request::Paint(FromPaintMsg::Failure(Failure { pipeline_id, parent_info })) => {
                debug!("handling paint failure message from pipeline {:?}, {:?}", pipeline_id, parent_info);
                self.handle_failure_msg(pipeline_id, parent_info);
            }

        }
        true
    }

    fn handle_exit(&mut self) {
        for (_id, ref pipeline) in &self.pipelines {
            pipeline.exit();
        }
        self.image_cache_thread.exit();
        self.resource_thread.send(net_traits::ControlMsg::Exit)
            .unwrap_or_else(|e| debug!("Exit resource thread failed ({})", e));
        self.devtools_chan.as_ref().map(|chan| {
            chan.send(DevtoolsControlMsg::FromChrome(ChromeToDevtoolsControlMsg::ServerExitMsg))
                .unwrap_or_else(|e| debug!("Exit devtools failed ({})", e));
        });
        self.storage_thread.send(StorageThreadMsg::Exit)
            .unwrap_or_else(|e| debug!("Exit storage thread failed ({})", e));
        self.font_cache_thread.exit();
        self.compositor_proxy.send(ToCompositorMsg::ShutdownComplete);
    }

    fn handle_send_error(&mut self, pipeline_id: PipelineId) {
        let parent_info = match self.pipelines.get(&pipeline_id) {
            None => return debug!("Pipeline {:?} send error after closure.", pipeline_id),
            Some(pipeline) => pipeline.parent_info,
        };
        // Treat send error the same as receiving a failure message
        self.handle_failure_msg(pipeline_id, parent_info);
    }

    fn handle_failure_msg(&mut self,
                          pipeline_id: PipelineId,
                          parent_info: Option<(PipelineId, SubpageId)>) {
        if opts::get().hard_fail {
            // It's quite difficult to make Servo exit cleanly if some threads have failed.
            // Hard fail exists for test runners so we crash and that's good enough.
            let mut stderr = io::stderr();
            stderr.write_all("Pipeline failed in hard-fail mode.  Crashing!\n".as_bytes()).unwrap();
            process::exit(1);
        }

        let window_size = self.pipelines.get(&pipeline_id).and_then(|pipeline| pipeline.size);

        self.close_pipeline(pipeline_id, ExitPipelineMode::Force);

        while let Some(pending_pipeline_id) = self.pending_frames.iter().find(|pending| {
            pending.old_pipeline_id == Some(pipeline_id)
        }).map(|frame| frame.new_pipeline_id) {
            debug!("removing pending frame change for failed pipeline");
            self.close_pipeline(pending_pipeline_id, ExitPipelineMode::Force);
        }

        debug!("creating replacement pipeline for about:failure");

        let new_pipeline_id = PipelineId::new();
        self.new_pipeline(new_pipeline_id,
                          parent_info,
                          window_size,
                          None,
                          LoadData::new(url!("about:failure")));

        self.push_pending_frame(new_pipeline_id, Some(pipeline_id));

    }

    fn handle_init_load(&mut self, url: Url) {
        let window_size = self.window_size.visible_viewport;
        let root_pipeline_id = PipelineId::new();
        debug_assert!(PipelineId::fake_root_pipeline_id() == root_pipeline_id);
        self.new_pipeline(root_pipeline_id, None, Some(window_size), None, LoadData::new(url.clone()));
        self.handle_load_start_msg(&root_pipeline_id);
        self.push_pending_frame(root_pipeline_id, None);
        self.compositor_proxy.send(ToCompositorMsg::ChangePageUrl(root_pipeline_id, url));
    }

    fn handle_frame_size_msg(&mut self,
                             pipeline_id: PipelineId,
                             size: &TypedSize2D<PagePx, f32>) {
        // Store the new rect inside the pipeline
        let script_chan = {
            // Find the pipeline that corresponds to this rectangle. It's possible that this
            // pipeline may have already exited before we process this message, so just
            // early exit if that occurs.
            match self.pipelines.get_mut(&pipeline_id) {
                Some(pipeline) => {
                    pipeline.size = Some(*size);
                    pipeline.script_chan.clone()
                }
                None => return,
            }
        };

        script_chan.send(ConstellationControlMsg::Resize(pipeline_id, WindowSizeData {
            visible_viewport: *size,
            initial_viewport: *size * ScaleFactor::new(1.0),
            device_pixel_ratio: self.window_size.device_pixel_ratio,
        })).unwrap_or_else(|_| self.handle_send_error(pipeline_id));
    }

    fn handle_subframe_loaded(&mut self, pipeline_id: PipelineId) {
        let parent_info = match self.pipelines.get(&pipeline_id) {
            Some(pipeline) => pipeline.parent_info,
            None => return debug!("Pipeline {:?} loaded after closure.", pipeline_id),
        };
        let subframe_parent_id = match parent_info {
            Some(ref parent) => parent.0,
            None => return debug!("Pipeline {:?} has no parent.", pipeline_id),
        };
        let script_chan = match self.pipelines.get(&subframe_parent_id) {
            Some(pipeline) => pipeline.script_chan.clone(),
            None => return debug!("Pipeline {:?} subframe loaded after closure.", subframe_parent_id),
        };
        let msg = ConstellationControlMsg::DispatchFrameLoadEvent {
            target: pipeline_id,
            parent: subframe_parent_id,
        };
        script_chan.send(msg).unwrap_or_else(|_| self.handle_send_error(subframe_parent_id));
    }

    // The script thread associated with pipeline_id has loaded a URL in an iframe via script. This
    // will result in a new pipeline being spawned and a frame tree being added to
    // containing_page_pipeline_id's frame tree's children. This message is never the result of a
    // page navigation.
    fn handle_script_loaded_url_in_iframe_msg(&mut self, load_info: IFrameLoadInfo) {

        let old_pipeline_id = load_info.old_subpage_id
            .and_then(|old_subpage_id| self.subpage_map.get(&(load_info.containing_pipeline_id, old_subpage_id)))
            .cloned();

        let (new_url, script_chan, window_size) = {

            let old_pipeline = old_pipeline_id
                .and_then(|old_pipeline_id| self.pipelines.get(&old_pipeline_id));

            let source_pipeline = self.pipelines.get(&load_info.containing_pipeline_id);

            // If no url is specified, reload.
            let new_url = load_info.url.clone()
                .or_else(|| old_pipeline.map(|old_pipeline| old_pipeline.url.clone()))
                .unwrap_or_else(|| url!("about:blank"));

            // Compare the pipeline's url to the new url. If the origin is the same,
            // then reuse the script thread in creating the new pipeline
            let script_chan = source_pipeline.and_then(|source_pipeline| {
                let source_url = source_pipeline.url.clone();

                let same_script = (source_url.host() == new_url.host() &&
                                   source_url.port() == new_url.port()) &&
                    load_info.sandbox == IFrameSandboxState::IFrameUnsandboxed;

                // FIXME(tkuehn): Need to follow the standardized spec for checking same-origin
                // Reuse the script thread if the URL is same-origin
                if same_script {
                    debug!("Constellation: loading same-origin iframe, \
                            parent url {:?}, iframe url {:?}", source_url, new_url);
                    Some(source_pipeline.script_chan.clone())
                } else {
                    debug!("Constellation: loading cross-origin iframe, \
                            parent url {:?}, iframe url {:?}", source_url, new_url);
                    None
                }
            });

            let window_size = old_pipeline.and_then(|old_pipeline| old_pipeline.size);

            if let Some(old_pipeline) = old_pipeline {
                old_pipeline.freeze();
            }

            (new_url, script_chan, window_size)

        };

        // Create the new pipeline, attached to the parent and push to pending frames
        self.new_pipeline(load_info.new_pipeline_id,
                          Some((load_info.containing_pipeline_id, load_info.new_subpage_id)),
                          window_size,
                          script_chan,
                          LoadData::new(new_url));

        self.subpage_map.insert((load_info.containing_pipeline_id, load_info.new_subpage_id),
                                load_info.new_pipeline_id);

        self.push_pending_frame(load_info.new_pipeline_id, old_pipeline_id);
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
        let (layout_chan, script_chan) = match self.pipelines.get(&pipeline_id) {
            Some(pipeline) => (pipeline.layout_chan.0.clone(), pipeline.script_chan.clone()),
            None => return debug!("Pipeline {:?} got tick after closure.", pipeline_id),
        };
        match tick_type {
            AnimationTickType::Script => {
                script_chan
                    .send(ConstellationControlMsg::TickAllAnimations(pipeline_id))
                    .unwrap_or_else(|_| self.handle_send_error(pipeline_id));
            }
            AnimationTickType::Layout => {
                layout_chan
                    .send(LayoutControlMsg::TickAnimations)
                    .unwrap_or_else(|_| self.handle_send_error(pipeline_id));
            }
        }
    }

    fn handle_load_url_msg(&mut self, source_id: PipelineId, load_data: LoadData) {
        self.load_url(source_id, load_data);
    }

    fn load_url(&mut self, source_id: PipelineId, load_data: LoadData) -> Option<PipelineId> {
        let (parent_info, window_size) = {
            let source = self.pipelines.get(&source_id);
            let parent_info = source.and_then(|source| source.parent_info);
            let window_size = source.and_then(|source| source.size);
            (parent_info, window_size)
        };

        // If this load targets an iframe, its framing element may exist
        // in a separate script thread than the framed document that initiated
        // the new load. The framing element must be notified about the
        // requested change so it can update its internal state.
        match parent_info {
            Some((parent_pipeline_id, subpage_id)) => {
                self.handle_load_start_msg(&source_id);
                // Message the constellation to find the script thread for this iframe
                // and issue an iframe load through there.
                let msg = ConstellationControlMsg::Navigate(parent_pipeline_id, subpage_id, load_data);
                let script_chan = match self.pipelines.get(&parent_pipeline_id) {
                    Some(parent_pipeline) => parent_pipeline.script_chan.clone(),
                    None => {
                        debug!("Pipeline {:?} child loaded after closure", parent_pipeline_id);
                        return None;
                    },
                };
                script_chan.send(msg).unwrap_or_else(|_| self.handle_send_error(parent_pipeline_id));
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

                self.handle_load_start_msg(&source_id);
                // Being here means either there are no pending frames, or none of the pending
                // changes would be overridden by changing the subframe associated with source_id.

                // Create the new pipeline
                let new_pipeline_id = PipelineId::new();
                self.new_pipeline(new_pipeline_id, None, window_size, None, load_data);
                self.push_pending_frame(new_pipeline_id, Some(source_id));

                // Send message to ScriptThread that will suspend all timers
                match self.pipelines.get(&source_id) {
                    Some(source) => source.freeze(),
                    None => debug!("Pipeline {:?} loaded after closure", source_id),
                };
                Some(new_pipeline_id)
            }
        }
    }

    fn handle_load_start_msg(&mut self, pipeline_id: &PipelineId) {
        if let Some(frame_id) = self.pipeline_to_frame_map.get(pipeline_id) {
            if let Some(frame) = self.frames.get(frame_id) {
                let forward = !frame.next.is_empty();
                let back = !frame.prev.is_empty();
                self.compositor_proxy.send(ToCompositorMsg::LoadStart(back, forward));
            }
        }
    }

    fn handle_load_complete_msg(&mut self, pipeline_id: &PipelineId) {
        if let Some(&frame_id) = self.pipeline_to_frame_map.get(pipeline_id) {
            if let Some(frame) = self.frames.get(&frame_id) {
                let forward = frame.next.is_empty();
                let back = frame.prev.is_empty();
                let root = self.root_frame_id.is_none() || self.root_frame_id == Some(frame_id);
                self.compositor_proxy.send(ToCompositorMsg::LoadComplete(back, forward, root));
            }
        }
    }

    fn handle_dom_load(&mut self, pipeline_id: PipelineId) {
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

        self.handle_subframe_loaded(pipeline_id);
    }

    fn handle_navigate_msg(&mut self,
                           pipeline_info: Option<(PipelineId, SubpageId)>,
                           direction: constellation_msg::NavigationDirection) {
        debug!("received message to navigate {:?}", direction);

        // Get the frame id associated with the pipeline that sent
        // the navigate message, or use root frame id by default.
        let frame_id = pipeline_info
            .and_then(|info| self.subpage_map.get(&info))
            .and_then(|pipeline_id| self.pipeline_to_frame_map.get(&pipeline_id))
            .cloned()
            .or(self.root_frame_id);

        // If the frame_id lookup fails, then we are in the middle of tearing down
        // the root frame, so it is reasonable to silently ignore the navigation.
        let frame_id = match frame_id {
            None => return debug!("Navigation after root's closure."),
            Some(frame_id) => frame_id,
        };

        // Check if the currently focused pipeline is the pipeline being replaced
        // (or a child of it). This has to be done here, before the current
        // frame tree is modified below.
        let update_focus_pipeline = self.focused_pipeline_in_tree(frame_id);

        // Get the ids for the previous and next pipelines.
        let (prev_pipeline_id, next_pipeline_id) = match self.frames.get_mut(&frame_id) {
            Some(frame) => {
                let next = match direction {
                    NavigationDirection::Forward => {
                        match frame.next.pop() {
                            None => {
                                debug!("no next page to navigate to");
                                return;
                            },
                            Some(next) => {
                                frame.prev.push(frame.current);
                                next
                            },
                        }
                    }
                    NavigationDirection::Back => {
                        match frame.prev.pop() {
                            None => {
                                debug!("no previous page to navigate to");
                                return;
                            },
                            Some(prev) => {
                                frame.next.push(frame.current);
                                prev
                            },
                        }
                    }
                };
                let prev = frame.current;
                frame.current = next;
                (prev, next)
            },
            None => {
                debug!("no frame to navigate from");
                return;
            },
        };

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
        self.revoke_paint_permission(prev_pipeline_id);
        self.send_frame_tree_and_grant_paint_permission();

        // Update the owning iframe to point to the new subpage id.
        // This makes things like contentDocument work correctly.
        if let Some((parent_pipeline_id, subpage_id)) = pipeline_info {
            let script_chan = match self.pipelines.get(&parent_pipeline_id) {
                None => return debug!("Pipeline {:?} child navigated after closure.", parent_pipeline_id),
                Some(pipeline) => pipeline.script_chan.clone(),
            };
            let new_subpage_id = match self.pipelines.get(&next_pipeline_id) {
                None => return debug!("Pipeline {:?} navigated to after closure.", next_pipeline_id),
                Some(pipeline) => match pipeline.parent_info {
                    None => return debug!("Pipeline {:?} has no parent info.", next_pipeline_id),
                    Some((_, new_subpage_id)) => new_subpage_id,
                },
            };
            let msg = ConstellationControlMsg::UpdateSubpageId(parent_pipeline_id, subpage_id, new_subpage_id);
            script_chan.send(msg).unwrap_or_else(|_| self.handle_send_error(parent_pipeline_id));

            // If this is an iframe, send a mozbrowser location change event.
            // This is the result of a back/forward navigation.
            self.trigger_mozbrowserlocationchange(next_pipeline_id);
        }
    }

    fn handle_key_msg(&mut self, key: Key, state: KeyState, mods: KeyModifiers) {
        // Send to the explicitly focused pipeline (if it exists), or the root
        // frame's current pipeline. If neither exist, fall back to sending to
        // the compositor below.
        let root_pipeline_id = self.root_frame_id
            .and_then(|root_frame_id| self.frames.get(&root_frame_id))
            .map(|root_frame| root_frame.current);
        let pipeline_id = self.focus_pipeline_id.or(root_pipeline_id);
        let script_chan = pipeline_id
            .and_then(|pipeline_id| self.pipelines.get(&pipeline_id))
            .map(|pipeline| pipeline.script_chan.clone());

        match (pipeline_id, script_chan) {
            (Some(pipeline_id), Some(script_chan)) => {
                let event = CompositorEvent::KeyEvent(key, state, mods);
                let msg = ConstellationControlMsg::SendEvent(pipeline_id, event);
                script_chan.send(msg).unwrap_or_else(|_| self.handle_send_error(pipeline_id));
            }
            _ => {
                let event = ToCompositorMsg::KeyEvent(key, state, mods);
                self.compositor_proxy.clone_compositor_proxy().send(event);
            }
        }
    }

    fn handle_get_pipeline_title_msg(&mut self, pipeline_id: PipelineId) {
        let result = match self.pipelines.get(&pipeline_id) {
            None => return self.compositor_proxy.send(ToCompositorMsg::ChangePageTitle(pipeline_id, None)),
            Some(pipeline) => pipeline.script_chan.send(ConstellationControlMsg::GetTitle(pipeline_id)),
        };
        result.unwrap_or_else(|_| self.handle_send_error(pipeline_id));
    }

    fn handle_mozbrowser_event_msg(&mut self,
                                   containing_pipeline_id: PipelineId,
                                   subpage_id: SubpageId,
                                   event: MozBrowserEvent) {
        assert!(prefs::get_pref("dom.mozbrowser.enabled").as_boolean().unwrap_or(false));

        // Find the script channel for the given parent pipeline,
        // and pass the event to that script thread.
        // If the pipeline lookup fails, it is because we have torn down the pipeline,
        // so it is reasonable to silently ignore the event.
        match self.pipelines.get(&containing_pipeline_id) {
            Some(pipeline) => pipeline.trigger_mozbrowser_event(subpage_id, event),
            None => debug!("Pipeline {:?} handling mozbrowser event after closure.", containing_pipeline_id),
        }
    }

    fn handle_get_pipeline(&mut self, frame_id: Option<FrameId>,
                           resp_chan: IpcSender<Option<PipelineId>>) {
        let current_pipeline_id = frame_id.or(self.root_frame_id)
            .and_then(|frame_id| self.frames.get(&frame_id))
            .map(|frame| frame.current);
        let pipeline_id = self.pending_frames.iter().rev()
            .find(|x| x.old_pipeline_id == current_pipeline_id)
            .map(|x| x.new_pipeline_id).or(current_pipeline_id);
        resp_chan.send(pipeline_id)
            .unwrap_or_else(|_| debug!("Failed get_pipeline response."));
    }

    fn handle_get_frame(&mut self,
                        pipeline_id: PipelineId,
                        resp_chan: IpcSender<Option<FrameId>>) {
        let frame_id = self.pipeline_to_frame_map.get(&pipeline_id).map(|x| *x);
        resp_chan.send(frame_id)
            .unwrap_or_else(|_| debug!("Failed get_pipeline response."));
    }

    fn focus_parent_pipeline(&mut self, pipeline_id: PipelineId) {
        let parent_info = match self.pipelines.get(&pipeline_id) {
            Some(pipeline) => pipeline.parent_info,
            None => return debug!("Pipeline {:?} focus parent after closure.", pipeline_id),
        };
        let (containing_pipeline_id, subpage_id) = match parent_info {
            Some(info) => info,
            None => return debug!("Pipeline {:?} focus has no parent.", pipeline_id),
        };
        let script_chan = match self.pipelines.get(&containing_pipeline_id) {
            Some(pipeline) => pipeline.script_chan.clone(),
            None => return debug!("Pipeline {:?} focus after closure.", containing_pipeline_id),
        };

        // Send a message to the parent of the provided pipeline (if it exists)
        // telling it to mark the iframe element as focused.
        let msg = ConstellationControlMsg::FocusIFrame(containing_pipeline_id, subpage_id);
        script_chan.send(msg).unwrap_or_else(|_| self.handle_send_error(containing_pipeline_id));
        self.focus_parent_pipeline(containing_pipeline_id);
    }

    fn handle_focus_msg(&mut self, pipeline_id: PipelineId) {
        self.focus_pipeline_id = Some(pipeline_id);

        // Focus parent iframes recursively
        self.focus_parent_pipeline(pipeline_id);
    }

    fn handle_remove_iframe_msg(&mut self, pipeline_id: PipelineId) {
        let frame_id = self.pipeline_to_frame_map.get(&pipeline_id).map(|id| *id);
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

    fn handle_create_canvas_paint_thread_msg(
            &mut self,
            size: &Size2D<i32>,
            response_sender: IpcSender<IpcSender<CanvasMsg>>) {
        let webrender_api = self.webrender_api_sender.clone();
        let sender = CanvasPaintThread::start(*size, webrender_api);
        response_sender.send(sender)
            .unwrap_or_else(|e| debug!("Create canvas paint thread response failed ({})", e))
    }

    fn handle_create_webgl_paint_thread_msg(
            &mut self,
            size: &Size2D<i32>,
            attributes: GLContextAttributes,
            response_sender: IpcSender<Result<IpcSender<CanvasMsg>, String>>) {
        let webrender_api = self.webrender_api_sender.clone();
        let sender = WebGLPaintThread::start(*size, attributes, webrender_api);

        response_sender.send(sender)
            .unwrap_or_else(|e| debug!("Create WebGL paint thread response failed ({})", e))
    }

    fn handle_webdriver_msg(&mut self, msg: WebDriverCommandMsg) {
        // Find the script channel for the given parent pipeline,
        // and pass the event to that script thread.
        match msg {
            WebDriverCommandMsg::LoadUrl(pipeline_id, load_data, reply) => {
                self.load_url_for_webdriver(pipeline_id, load_data, reply);
            },
            WebDriverCommandMsg::Refresh(pipeline_id, reply) => {
                let load_data = match self.pipelines.get(&pipeline_id) {
                    Some(pipeline) => LoadData::new(pipeline.url.clone()),
                    None => return debug!("Pipeline {:?} Refresh after closure.", pipeline_id),
                };
                self.load_url_for_webdriver(pipeline_id, load_data, reply);
            }
            WebDriverCommandMsg::ScriptCommand(pipeline_id, cmd) => {
                let script_channel = match self.pipelines.get(&pipeline_id) {
                    Some(pipeline) => pipeline.script_chan.clone(),
                    None => return debug!("Pipeline {:?} ScriptCommand after closure.", pipeline_id),
                };
                let control_msg = ConstellationControlMsg::WebDriverScriptCommand(pipeline_id, cmd);
                script_channel.send(control_msg)
                    .unwrap_or_else(|_| self.handle_send_error(pipeline_id));
            },
            WebDriverCommandMsg::SendKeys(pipeline_id, cmd) => {
                let script_channel = match self.pipelines.get(&pipeline_id) {
                    Some(pipeline) => pipeline.script_chan.clone(),
                    None => return debug!("Pipeline {:?} SendKeys after closure.", pipeline_id),
                };
                for (key, mods, state) in cmd {
                    let event = CompositorEvent::KeyEvent(key, state, mods);
                    let control_msg = ConstellationControlMsg::SendEvent(pipeline_id, event);
                    script_channel.send(control_msg)
                        .unwrap_or_else(|_| self.handle_send_error(pipeline_id));
                }
            },
            WebDriverCommandMsg::TakeScreenshot(pipeline_id, reply) => {
                let current_pipeline_id = self.root_frame_id
                    .and_then(|root_frame_id| self.frames.get(&root_frame_id))
                    .map(|root_frame| root_frame.current);
                if Some(pipeline_id) == current_pipeline_id {
                    self.compositor_proxy.send(ToCompositorMsg::CreatePng(reply));
                } else {
                    reply.send(None).unwrap_or_else(|e| debug!("Screenshot reply failed ({})", e));
                }
            },
        }
    }

    fn load_url_for_webdriver(&mut self,
                              pipeline_id: PipelineId,
                              load_data: LoadData,
                              reply: IpcSender<webdriver_msg::LoadStatus>) {
        let new_pipeline_id = self.load_url(pipeline_id, load_data);
        if let Some(id) = new_pipeline_id {
            self.webdriver.load_channel = Some((id, reply));
        }
    }

    fn add_or_replace_pipeline_in_frame_tree(&mut self, frame_change: FrameChange) {

        // If the currently focused pipeline is the one being changed (or a child
        // of the pipeline being changed) then update the focus pipeline to be
        // the replacement.
        if let Some(old_pipeline_id) = frame_change.old_pipeline_id {
            if let Some(&old_frame_id) = self.pipeline_to_frame_map.get(&old_pipeline_id) {
                if self.focused_pipeline_in_tree(old_frame_id) {
                    self.focus_pipeline_id = Some(frame_change.new_pipeline_id);
                }
            }
        }

        let evicted_frames = frame_change.old_pipeline_id.and_then(|old_pipeline_id| {
            // The new pipeline is replacing an old one.
            // Remove paint permissions for the pipeline being replaced.
            self.revoke_paint_permission(old_pipeline_id);

            // Add new pipeline to navigation frame, and return frames evicted from history.
            self.pipeline_to_frame_map.get(&old_pipeline_id).cloned()
                .and_then(|frame_id| {
                    self.pipeline_to_frame_map.insert(frame_change.new_pipeline_id, frame_id);
                    self.frames.get_mut(&frame_id)
                }).map(|frame| frame.load(frame_change.new_pipeline_id))
        });

        if let None = evicted_frames {
            // The new pipeline is in a new frame with no history
            let frame_id = self.new_frame(frame_change.new_pipeline_id);

            // If a child frame, add it to the parent pipeline. Otherwise
            // it must surely be the root frame being created!
            match self.pipelines.get(&frame_change.new_pipeline_id).and_then(|pipeline| pipeline.parent_info) {
                Some((parent_id, _)) => {
                    if let Some(parent) = self.pipelines.get_mut(&parent_id) {
                        parent.add_child(frame_id);
                    }
                }
                None => {
                    assert!(self.root_frame_id.is_none());
                    self.root_frame_id = Some(frame_id);
                }
            }
        }

        // Build frame tree and send permission
        self.send_frame_tree_and_grant_paint_permission();

        // If this is an iframe, send a mozbrowser location change event.
        // This is the result of a link being clicked and a navigation completing.
        self.trigger_mozbrowserlocationchange(frame_change.new_pipeline_id);

        // Remove any evicted frames
        for pipeline_id in evicted_frames.unwrap_or_default() {
            self.close_pipeline(pipeline_id, ExitPipelineMode::Normal);
        }

    }

    fn handle_activate_document_msg(&mut self, pipeline_id: PipelineId) {
        debug!("Document ready to activate {:?}", pipeline_id);

        if let Some(ref child_pipeline) = self.pipelines.get(&pipeline_id) {
            if let Some(ref parent_info) = child_pipeline.parent_info {
                if let Some(parent_pipeline) = self.pipelines.get(&parent_info.0) {
                    let _ = parent_pipeline.script_chan
                                           .send(ConstellationControlMsg::FramedContentChanged(
                                               parent_info.0,
                                               parent_info.1));
                }
            }
        }

        // If this pipeline is already part of the current frame tree,
        // we don't need to do anything.
        if self.pipeline_is_in_current_frame(pipeline_id) {
            return;
        }

        // Find the pending frame change whose new pipeline id is pipeline_id.
        // If it is found, mark this pending frame as ready to be enabled.
        let pending_index = self.pending_frames.iter().rposition(|frame_change| {
            frame_change.new_pipeline_id == pipeline_id
        });
        if let Some(pending_index) = pending_index {
            self.pending_frames[pending_index].document_ready = true;
        }

        // This is a bit complex. We need to loop through pending frames and find
        // ones that can be swapped. A frame can be swapped (enabled) once it is
        // ready to layout (has document_ready set), and also has no dependencies
        // (i.e. the pipeline it is replacing has been enabled and now has a frame).
        // The outer loop is required because any time a pipeline is enabled, that
        // may affect whether other pending frames are now able to be enabled. On the
        // other hand, if no frames can be enabled after looping through all pending
        // frames, we can safely exit the loop, knowing that we will need to wait on
        // a dependent pipeline to be ready to paint.
        while let Some(valid_frame_change) = self.pending_frames.iter().rposition(|frame_change| {
            let waiting_on_dependency = frame_change.old_pipeline_id.map_or(false, |old_pipeline_id| {
                self.pipeline_to_frame_map.get(&old_pipeline_id).is_none()
            });
            frame_change.document_ready && !waiting_on_dependency
        }) {
            let frame_change = self.pending_frames.swap_remove(valid_frame_change);
            self.add_or_replace_pipeline_in_frame_tree(frame_change);
        }
    }

    /// Called when the window is resized.
    fn handle_resized_window_msg(&mut self, new_size: WindowSizeData) {
        debug!("handle_resized_window_msg: {:?} {:?}", new_size.initial_viewport.to_untyped(),
                                                       new_size.visible_viewport.to_untyped());

        if let Some(root_frame_id) = self.root_frame_id {
            // Send Resize (or ResizeInactive) messages to each
            // pipeline in the frame tree.
            let frame = match self.frames.get(&root_frame_id) {
                None => return debug!("Frame {:?} resized after closing.", root_frame_id),
                Some(frame) => frame,
            };
            let pipeline_id = frame.current;
            let pipeline = match self.pipelines.get(&pipeline_id) {
                None => return debug!("Pipeline {:?} resized after closing.", pipeline_id),
                Some(pipeline) => pipeline,
            };
            let _ = pipeline.script_chan.send(ConstellationControlMsg::Resize(pipeline.id, new_size));
            for pipeline_id in frame.prev.iter().chain(&frame.next) {
                let pipeline = match self.pipelines.get(&pipeline_id) {
                    None => { debug!("Inactive pipeline {:?} resized after closing.", pipeline_id); continue; },
                    Some(pipeline) => pipeline,
                };
                let _ = pipeline.script_chan.send(ConstellationControlMsg::ResizeInactive(pipeline.id,
                                                                                          new_size));
            }
        }

        // Send resize message to any pending pipelines that aren't loaded yet.
        for pending_frame in &self.pending_frames {
            let pipeline_id = pending_frame.new_pipeline_id;
            let pipeline = match self.pipelines.get(&pipeline_id) {
                None => { debug!("Pending pipeline {:?} is closed", pipeline_id); continue; }
                Some(pipeline) => pipeline,
            };
            if pipeline.parent_info.is_none() {
                let _ = pipeline.script_chan.send(ConstellationControlMsg::Resize(pipeline.id,
                                                                                  new_size));
            }
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
        if self.root_frame_id.is_none() {
            return ReadyToSave::NoRootFrame;
        }

        // If there are pending loads, wait for those to complete.
        if self.pending_frames.len() > 0 {
            return ReadyToSave::PendingFrames;
        }

        // Step through the current frame tree, checking that the script
        // thread is idle, and that the current epoch of the layout thread
        // matches what the compositor has painted. If all these conditions
        // are met, then the output image should not change and a reftest
        // screenshot can safely be written.
        for frame in self.current_frame_tree_iter(self.root_frame_id) {

            let pipeline_id = frame.current;

            let pipeline = match self.pipelines.get(&pipeline_id) {
                None => { debug!("Pipeline {:?} screenshot while closing.", pipeline_id); continue; },
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
            let (sender, receiver) = ipc::channel().unwrap();
            let msg = LayoutControlMsg::GetWebFontLoadState(sender);
            pipeline.layout_chan.0.send(msg)
                .unwrap_or_else(|e| debug!("Get web font failed ({})", e));
            if receiver.recv().unwrap_or(true) {
                return ReadyToSave::WebFontNotLoaded;
            }

            // See if this pipeline has reached idle script state yet.
            match self.document_states.get(&frame.current) {
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
                if size == Size2D::zero() {
                    continue;
                }

                // Get the epoch that the compositor has drawn for this pipeline.
                let compositor_epoch = pipeline_states.get(&frame.current);
                match compositor_epoch {
                    Some(compositor_epoch) => {
                        // Synchronously query the layout thread to see if the current
                        // epoch matches what the compositor has drawn. If they match
                        // (and script is idle) then this pipeline won't change again
                        // and can be considered stable.
                        let (sender, receiver) = ipc::channel().unwrap();
                        let LayoutControlChan(ref layout_chan) = pipeline.layout_chan;
                        layout_chan.send(LayoutControlMsg::GetCurrentEpoch(sender)).unwrap();
                        let layout_thread_epoch = receiver.recv().unwrap();
                        if layout_thread_epoch != *compositor_epoch {
                            return ReadyToSave::EpochMismatch;
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

    /// Checks whether the pipeline or its ancestors are private
    #[allow(dead_code)]
    fn check_is_pipeline_private(&self, pipeline_id: PipelineId) -> bool {
        let mut pipeline_id = Some(pipeline_id);
        while let Some(pipeline) = pipeline_id.and_then(|id| self.pipelines.get(&id)) {
            if pipeline.is_private {
                return true;
            }
            pipeline_id = pipeline.parent_info.map(|(parent_pipeline_id, _)| parent_pipeline_id);
        }
        false
    }

    // Close a frame (and all children)
    fn close_frame(&mut self, frame_id: FrameId, exit_mode: ExitPipelineMode) {
        // Store information about the pipelines to be closed. Then close the
        // pipelines, before removing ourself from the frames hash map. This
        // ordering is vital - so that if close_pipeline() ends up closing
        // any child frames, they can be removed from the parent frame correctly.
        let parent_info = self.frames.get(&frame_id)
            .and_then(|frame| self.pipelines.get(&frame.current))
            .and_then(|pipeline| pipeline.parent_info);

        let pipelines_to_close = {
            let mut pipelines_to_close = vec!();

            if let Some(frame) = self.frames.get(&frame_id) {
                pipelines_to_close.extend_from_slice(&frame.next);
                pipelines_to_close.push(frame.current);
                pipelines_to_close.extend_from_slice(&frame.prev);
            }

            pipelines_to_close
        };

        for pipeline_id in &pipelines_to_close {
            self.close_pipeline(*pipeline_id, exit_mode);
        }

        if let None = self.frames.remove(&frame_id) {
            debug!("Closing frame {:?} twice.", frame_id);
        }

        if let Some((parent_pipeline_id, _)) = parent_info {
            let parent_pipeline = match self.pipelines.get_mut(&parent_pipeline_id) {
                None => return debug!("Pipeline {:?} child closed after parent.", parent_pipeline_id),
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

        let pipeline = match self.pipelines.remove(&pipeline_id) {
            Some(pipeline) => pipeline,
            None => return debug!("Closing pipeline {:?} twice.", pipeline_id),
        };

        // If a child pipeline, remove from subpage map
        if let Some(info) = pipeline.parent_info {
            self.subpage_map.remove(&info);
        }

        // Remove assocation between this pipeline and its holding frame
        self.pipeline_to_frame_map.remove(&pipeline_id);

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
                    // Note that we deliberately do not do any of the tidying up
                    // associated with closing a pipeline. The constellation should cope!
                    info!("Randomly closing pipeline {}.", pipeline_id);
                    pipeline.force_exit();
                }
            }
        }
    }

    // Convert a frame to a sendable form to pass to the compositor
    fn frame_to_sendable(&self, frame_id: FrameId) -> Option<SendableFrameTree> {
        self.frames.get(&frame_id).and_then(|frame: &Frame| {
            self.pipelines.get(&frame.current).map(|pipeline: &Pipeline| {
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

    // Revoke paint permission from a pipeline, and all children.
    fn revoke_paint_permission(&self, pipeline_id: PipelineId) {
        let frame_id = self.pipeline_to_frame_map.get(&pipeline_id).map(|frame_id| *frame_id);
        for frame in self.current_frame_tree_iter(frame_id) {
            self.pipelines.get(&frame.current).map(|pipeline| pipeline.revoke_paint_permission());
        }
    }

    // Send the current frame tree to compositor, and grant paint
    // permission to each pipeline in the current frame tree.
    fn send_frame_tree_and_grant_paint_permission(&mut self) {
        // Note that this function can panic, due to ipc-channel creation failure.
        // avoiding this panic would require a mechanism for dealing
        // with low-resource scenarios.
        if let Some(root_frame_id) = self.root_frame_id {
            if let Some(frame_tree) = self.frame_to_sendable(root_frame_id) {

                let (chan, port) = ipc::channel().unwrap();
                self.compositor_proxy.send(ToCompositorMsg::SetFrameTree(frame_tree,
                                                                         chan,
                                                                         self.compositor_sender.clone()));
                if port.recv().is_err() {
                    debug!("Compositor has discarded SetFrameTree");
                    return; // Our message has been discarded, probably shutting down.
                }
            }
        }

        for frame in self.current_frame_tree_iter(self.root_frame_id) {
            self.pipelines.get(&frame.current).map(|pipeline| pipeline.grant_paint_permission());
        }
    }

    // https://developer.mozilla.org/en-US/docs/Web/Events/mozbrowserlocationchange
    fn trigger_mozbrowserlocationchange(&self, pipeline_id: PipelineId) {
        if prefs::get_pref("dom.mozbrowser.enabled").as_boolean().unwrap_or(false) {
            let event_info = self.pipelines.get(&pipeline_id).and_then(|pipeline| {
                pipeline.parent_info.map(|(containing_pipeline_id, subpage_id)| {
                    (containing_pipeline_id, subpage_id, pipeline.url.serialize())
                })
            });

            // If this is an iframe, then send the event with new url
            if let Some((containing_pipeline_id, subpage_id, url)) = event_info {
                if let Some(parent_pipeline) = self.pipelines.get(&containing_pipeline_id) {
                    if let Some(frame_id) = self.pipeline_to_frame_map.get(&pipeline_id) {
                        if let Some(frame) = self.frames.get(&frame_id) {
                            let can_go_backward = !frame.prev.is_empty();
                            let can_go_forward = !frame.next.is_empty();
                            let event = MozBrowserEvent::LocationChange(url, can_go_backward, can_go_forward);
                            parent_pipeline.trigger_mozbrowser_event(subpage_id, event);
                        }
                    }
                }
            }
        }
    }

    fn focused_pipeline_in_tree(&self, frame_id: FrameId) -> bool {
        self.focus_pipeline_id.map_or(false, |pipeline_id| {
            self.pipeline_exists_in_tree(pipeline_id, Some(frame_id))
        })
    }

    fn pipeline_is_in_current_frame(&self, pipeline_id: PipelineId) -> bool {
        self.pipeline_exists_in_tree(pipeline_id, self.root_frame_id)
    }

    fn pipeline_exists_in_tree(&self,
                               pipeline_id: PipelineId,
                               root_frame_id: Option<FrameId>) -> bool {
        self.current_frame_tree_iter(root_frame_id)
            .any(|current_frame| current_frame.current == pipeline_id)
    }
}
