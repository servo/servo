/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The `Constellation`, Servo's Grand Central Station
//!
//! The primary duty of a `Constellation` is to mediate between the
//! graphics compositor and the many `Pipeline`s in the browser's
//! navigation context, each `Pipeline` encompassing a `ScriptTask`,
//! `LayoutTask`, and `PaintTask`.

use pipeline::{Pipeline, CompositionPipeline};

use canvas::canvas_paint_task::CanvasPaintTask;
use canvas::webgl_paint_task::WebGLPaintTask;
use canvas_traits::CanvasMsg;
use clipboard::ClipboardContext;
use compositor_task::CompositorProxy;
use compositor_task::Msg as CompositorMsg;
use devtools_traits::{ChromeToDevtoolsControlMsg, DevtoolsControlMsg};
use euclid::point::Point2D;
use euclid::rect::{Rect, TypedRect};
use euclid::scale_factor::ScaleFactor;
use euclid::size::Size2D;
use gfx::font_cache_task::FontCacheTask;
use ipc_channel::ipc::{self, IpcSender};
use layout_traits::{LayoutControlChan, LayoutTaskFactory};
use msg::compositor_msg::{Epoch, LayerId};
use msg::constellation_msg::AnimationState;
use msg::constellation_msg::Msg as ConstellationMsg;
use msg::constellation_msg::WebDriverCommandMsg;
use msg::constellation_msg::{FrameId, PipelineExitType, PipelineId};
use msg::constellation_msg::{IFrameSandboxState, MozBrowserEvent, NavigationDirection};
use msg::constellation_msg::{Key, KeyState, KeyModifiers, LoadData};
use msg::constellation_msg::{SubpageId, WindowSizeData};
use msg::constellation_msg::{self, ConstellationChan, Failure};
use msg::webdriver_msg;
use net_traits::image_cache_task::ImageCacheTask;
use net_traits::storage_task::{StorageTask, StorageTaskMsg};
use net_traits::{self, ResourceTask};
use offscreen_gl_context::GLContextAttributes;
use profile_traits::mem;
use profile_traits::time;
use script_traits::{CompositorEvent, ConstellationControlMsg, LayoutControlMsg};
use script_traits::{ScriptState, ScriptTaskFactory};
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::io::{self, Write};
use std::marker::PhantomData;
use std::mem::replace;
use std::process;
use std::sync::mpsc::{Receiver, Sender, channel};
use style_traits::viewport::ViewportConstraints;
use url::Url;
use util::cursor::Cursor;
use util::geometry::PagePx;
use util::task::spawn_named;
use util::{opts, prefs};

/// Maintains the pipelines and navigation context and grants permission to composite.
///
/// It is parameterized over a `LayoutTaskFactory` and a
/// `ScriptTaskFactory` (which in practice are implemented by
/// `LayoutTask` in the `layout` crate, and `ScriptTask` in
/// the `script` crate).
pub struct Constellation<LTF, STF> {
    /// A channel through which messages can be sent to this object.
    pub chan: ConstellationChan,

    /// Receives messages.
    pub request_port: Receiver<ConstellationMsg>,

    /// A channel (the implementation of which is port-specific) through which messages can be sent
    /// to the compositor.
    pub compositor_proxy: Box<CompositorProxy>,

    /// A channel through which messages can be sent to the resource task.
    pub resource_task: ResourceTask,

    /// A channel through which messages can be sent to the image cache task.
    pub image_cache_task: ImageCacheTask,

    /// A channel through which messages can be sent to the developer tools.
    devtools_chan: Option<Sender<DevtoolsControlMsg>>,

    /// A channel through which messages can be sent to the storage task.
    storage_task: StorageTask,

    /// A list of all the pipelines. (See the `pipeline` module for more details.)
    pipelines: HashMap<PipelineId, Pipeline>,

    /// A list of all the frames
    frames: HashMap<FrameId, Frame>,

    /// Maps from pipeline ID to the frame that contains it.
    pipeline_to_frame_map: HashMap<PipelineId, FrameId>,

    /// Maps from a (parent pipeline, subpage) to the actual child pipeline ID.
    subpage_map: HashMap<(PipelineId, SubpageId), PipelineId>,

    /// A channel through which messages can be sent to the font cache.
    font_cache_task: FontCacheTask,

    /// ID of the root frame.
    root_frame_id: Option<FrameId>,

    /// The next free ID to assign to a pipeline.
    next_pipeline_id: PipelineId,

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

    /// A list of in-process senders to `CanvasPaintTask`s.
    canvas_paint_tasks: Vec<Sender<CanvasMsg>>,

    /// A list of in-process senders to `WebGLPaintTask`s.
    webgl_paint_tasks: Vec<Sender<CanvasMsg>>,
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
    painter_ready: bool,
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
        self.stack.pop().map(|next| {
            let frame = self.frames.get(&next).unwrap();
            let pipeline = self.pipelines.get(&frame.current).unwrap();
            self.stack.extend(pipeline.children.iter().map(|&c| c));
            frame
        })
    }
}

pub struct SendableFrameTree {
    pub pipeline: CompositionPipeline,
    pub rect: Option<TypedRect<PagePx, f32>>,
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

impl<LTF: LayoutTaskFactory, STF: ScriptTaskFactory> Constellation<LTF, STF> {
    pub fn start(compositor_proxy: Box<CompositorProxy + Send>,
                 resource_task: ResourceTask,
                 image_cache_task: ImageCacheTask,
                 font_cache_task: FontCacheTask,
                 time_profiler_chan: time::ProfilerChan,
                 mem_profiler_chan: mem::ProfilerChan,
                 devtools_chan: Option<Sender<DevtoolsControlMsg>>,
                 storage_task: StorageTask,
                 supports_clipboard: bool)
                 -> ConstellationChan {
        let (constellation_port, constellation_chan) = ConstellationChan::new();
        let constellation_chan_clone = constellation_chan.clone();
        spawn_named("Constellation".to_owned(), move || {
            let mut constellation: Constellation<LTF, STF> = Constellation {
                chan: constellation_chan_clone,
                request_port: constellation_port,
                compositor_proxy: compositor_proxy,
                devtools_chan: devtools_chan,
                resource_task: resource_task,
                image_cache_task: image_cache_task,
                font_cache_task: font_cache_task,
                storage_task: storage_task,
                pipelines: HashMap::new(),
                frames: HashMap::new(),
                pipeline_to_frame_map: HashMap::new(),
                subpage_map: HashMap::new(),
                pending_frames: vec!(),
                next_pipeline_id: PipelineId(0),
                root_frame_id: None,
                next_frame_id: FrameId(0),
                focus_pipeline_id: None,
                time_profiler_chan: time_profiler_chan,
                mem_profiler_chan: mem_profiler_chan,
                window_size: WindowSizeData {
                    visible_viewport: opts::get().initial_window_size.as_f32() *
                                          ScaleFactor::new(1.0),
                    initial_viewport: opts::get().initial_window_size.as_f32() *
                        ScaleFactor::new(1.0),
                    device_pixel_ratio: ScaleFactor::new(1.0),
                },
                phantom: PhantomData,
                clipboard_ctx: if supports_clipboard {
                    ClipboardContext::new().ok()
                } else {
                    None
                },
                webdriver: WebDriverData::new(),
                canvas_paint_tasks: Vec::new(),
                webgl_paint_tasks: Vec::new(),
            };
            constellation.run();
        });
        constellation_chan
    }

    fn run(&mut self) {
        loop {
            let request = self.request_port.recv().unwrap();
            if !self.handle_request(request) {
                break;
            }
        }
    }

    /// Helper function for creating a pipeline
    fn new_pipeline(&mut self,
                    parent_info: Option<(PipelineId, SubpageId)>,
                    initial_window_rect: Option<TypedRect<PagePx, f32>>,
                    script_channel: Option<Sender<ConstellationControlMsg>>,
                    load_data: LoadData)
                    -> PipelineId {
        let pipeline_id = self.next_pipeline_id;
        let PipelineId(ref mut i) = self.next_pipeline_id;
        *i += 1;

        let spawning_paint_only = script_channel.is_some();
        let (pipeline, mut pipeline_content) =
            Pipeline::create::<LTF, STF>(pipeline_id,
                                         parent_info,
                                         self.chan.clone(),
                                         self.compositor_proxy.clone_compositor_proxy(),
                                         self.devtools_chan.clone(),
                                         self.image_cache_task.clone(),
                                         self.font_cache_task.clone(),
                                         self.resource_task.clone(),
                                         self.storage_task.clone(),
                                         self.time_profiler_chan.clone(),
                                         self.mem_profiler_chan.clone(),
                                         initial_window_rect,
                                         script_channel,
                                         load_data,
                                         self.window_size.device_pixel_ratio);

        // TODO(pcwalton): In multiprocess mode, send that `PipelineContent` instance over to
        // the content process and call this over there.
        if spawning_paint_only {
            pipeline_content.start_paint_task();
        } else {
            pipeline_content.start_all::<LTF, STF>();
        }

        assert!(!self.pipelines.contains_key(&pipeline_id));
        self.pipelines.insert(pipeline_id, pipeline);
        pipeline_id
    }

    // Push a new (loading) pipeline to the list of pending frame changes
    fn push_pending_frame(&mut self, new_pipeline_id: PipelineId,
                          old_pipeline_id: Option<PipelineId>) {
        self.pending_frames.push(FrameChange {
            old_pipeline_id: old_pipeline_id,
            new_pipeline_id: new_pipeline_id,
            painter_ready: false,
        });
    }

    // Get an iterator for the current frame tree. Specify self.root_frame_id to
    // iterate the entire tree, or a specific frame id to iterate only that sub-tree.
    fn current_frame_tree_iter(&self, frame_id_root: Option<FrameId>) -> FrameTreeIterator {
        let mut stack = vec!();
        if let Some(frame_id_root) = frame_id_root {
            stack.push(frame_id_root);
        }
        FrameTreeIterator {
            stack: stack,
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
    fn handle_request(&mut self, request: ConstellationMsg) -> bool {
        match request {
            ConstellationMsg::Exit => {
                debug!("constellation exiting");
                self.handle_exit();
                return false;
            }
            ConstellationMsg::Failure(Failure { pipeline_id, parent_info }) => {
                self.handle_failure_msg(pipeline_id, parent_info);
            }
            // This should only be called once per constellation, and only by the browser
            ConstellationMsg::InitLoadUrl(url) => {
                debug!("constellation got init load URL message");
                self.handle_init_load(url);
            }
            // A layout assigned a size and position to a subframe. This needs to be reflected by
            // all frame trees in the navigation context containing the subframe.
            ConstellationMsg::FrameRect(pipeline_id, subpage_id, rect) => {
                debug!("constellation got frame rect message");
                self.handle_frame_rect_msg(pipeline_id, subpage_id, Rect::from_untyped(&rect));
            }
            ConstellationMsg::ScriptLoadedURLInIFrame(url,
                                                      source_pipeline_id,
                                                      new_subpage_id,
                                                      old_subpage_id,
                                                      sandbox) => {
                debug!("constellation got iframe URL load message {:?} {:?} {:?}",
                       source_pipeline_id,
                       old_subpage_id,
                       new_subpage_id);
                self.handle_script_loaded_url_in_iframe_msg(url,
                                                            source_pipeline_id,
                                                            new_subpage_id,
                                                            old_subpage_id,
                                                            sandbox);
            }
            ConstellationMsg::SetCursor(cursor) => self.handle_set_cursor_msg(cursor),
            ConstellationMsg::ChangeRunningAnimationsState(pipeline_id, animation_state) => {
                self.handle_change_running_animations_state(pipeline_id, animation_state)
            }
            ConstellationMsg::TickAnimation(pipeline_id) => {
                self.handle_tick_animation(pipeline_id)
            }
            // Load a new page, usually -- but not always -- from a mouse click or typed url
            // If there is already a pending page (self.pending_frames), it will not be overridden;
            // However, if the id is not encompassed by another change, it will be.
            ConstellationMsg::LoadUrl(source_id, load_data) => {
                debug!("constellation got URL load message");
                self.handle_load_url_msg(source_id, load_data);
            }
            // A page loaded through one of several methods above has completed all parsing,
            // script, and reflow messages have been sent.
            ConstellationMsg::LoadComplete(pipeline_id) => {
                debug!("constellation got load complete message");
                self.handle_load_complete_msg(&pipeline_id)
            }
            // The DOM load event fired on a document
            ConstellationMsg::DOMLoad(pipeline_id) => {
                debug!("constellation got dom load message");
                self.handle_dom_load(pipeline_id)
            }
            // Handle a forward or back request
            ConstellationMsg::Navigate(pipeline_info, direction) => {
                debug!("constellation got navigation message");
                self.handle_navigate_msg(pipeline_info, direction);
            }
            // Notification that painting has finished and is requesting permission to paint.
            ConstellationMsg::PainterReady(pipeline_id) => {
                debug!("constellation got painter ready message");
                self.handle_painter_ready_msg(pipeline_id);
            }
            ConstellationMsg::ResizedWindow(new_size) => {
                debug!("constellation got window resize message");
                self.handle_resized_window_msg(new_size);
            }
            ConstellationMsg::KeyEvent(key, state, modifiers) => {
                debug!("constellation got key event message");
                self.handle_key_msg(key, state, modifiers);
            }
            ConstellationMsg::GetPipelineTitle(pipeline_id) => {
                debug!("constellation got get-pipeline-title message");
                self.handle_get_pipeline_title_msg(pipeline_id);
            }
            ConstellationMsg::MozBrowserEvent(pipeline_id,
                                              subpage_id,
                                              event) => {
                debug!("constellation got mozbrowser event message");
                self.handle_mozbrowser_event_msg(pipeline_id,
                                                 subpage_id,
                                                 event);
            }
            ConstellationMsg::GetPipeline(frame_id, resp_chan) => {
                debug!("constellation got get root pipeline message");
                self.handle_get_pipeline(frame_id, resp_chan);
            }
            ConstellationMsg::GetFrame(parent_pipeline_id, subpage_id, resp_chan) => {
                debug!("constellation got get root pipeline message");
                self.handle_get_frame(parent_pipeline_id, subpage_id, resp_chan);
            }
            ConstellationMsg::Focus(pipeline_id) => {
                debug!("constellation got focus message");
                self.handle_focus_msg(pipeline_id);
            }
            ConstellationMsg::GetClipboardContents(sender) => {
                let result = self.clipboard_ctx.as_ref().map_or(
                    "".to_string(),
                    |ctx| ctx.get_contents().unwrap_or_else(|e| {
                        debug!("Error getting clipboard contents ({}), defaulting to empty string", e);
                        "".to_string()
                    })
                );
                sender.send(result).unwrap();
            }
            ConstellationMsg::SetClipboardContents(s) => {
                if let Some(ref mut ctx) = self.clipboard_ctx {
                    if let Err(e) = ctx.set_contents(s) {
                        debug!("Error setting clipboard contents ({})", e);
                    }
                }
            }
            ConstellationMsg::WebDriverCommand(command) => {
                debug!("constellation got webdriver command message");
                self.handle_webdriver_msg(command);
            }
            ConstellationMsg::ViewportConstrained(pipeline_id, constraints) => {
                debug!("constellation got viewport-constrained event message");
                self.handle_viewport_constrained_msg(pipeline_id, constraints);
            }
            ConstellationMsg::IsReadyToSaveImage(pipeline_states) => {
                let is_ready = self.handle_is_ready_to_save_image(pipeline_states);
                self.compositor_proxy.send(CompositorMsg::IsReadyToSaveImageReply(is_ready));
            }
            ConstellationMsg::RemoveIFrame(containing_pipeline_id, subpage_id) => {
                debug!("constellation got remove iframe message");
                self.handle_remove_iframe_msg(containing_pipeline_id, subpage_id);
            }
            ConstellationMsg::NewFavicon(url) => {
                debug!("constellation got new favicon message");
                self.compositor_proxy.send(CompositorMsg::NewFavicon(url));
            }
            ConstellationMsg::HeadParsed => {
                debug!("constellation got head parsed message");
                self.compositor_proxy.send(CompositorMsg::HeadParsed);
            }
            ConstellationMsg::CreateCanvasPaintTask(size, sender) => {
                debug!("constellation got create-canvas-paint-task message");
                self.handle_create_canvas_paint_task_msg(&size, sender)
            }
            ConstellationMsg::CreateWebGLPaintTask(size, attributes, sender) => {
                debug!("constellation got create-WebGL-paint-task message");
                self.handle_create_webgl_paint_task_msg(&size, attributes, sender)
            }
            ConstellationMsg::NodeStatus(message) => {
                debug!("constellation got NodeStatus message");
                self.compositor_proxy.send(CompositorMsg::Status(message));
            }
        }
        true
    }

    fn handle_exit(&mut self) {
        for (_id, ref pipeline) in &self.pipelines {
            pipeline.exit(PipelineExitType::Complete);
        }
        self.image_cache_task.exit();
        self.resource_task.send(net_traits::ControlMsg::Exit).unwrap();
        self.devtools_chan.as_ref().map(|chan| {
            chan.send(DevtoolsControlMsg::FromChrome(
                    ChromeToDevtoolsControlMsg::ServerExitMsg)).unwrap();
        });
        self.storage_task.send(StorageTaskMsg::Exit).unwrap();
        self.font_cache_task.exit();
        self.compositor_proxy.send(CompositorMsg::ShutdownComplete);
    }

    fn handle_failure_msg(&mut self,
                          pipeline_id: PipelineId,
                          parent_info: Option<(PipelineId, SubpageId)>) {
        debug!("handling failure message from pipeline {:?}, {:?}", pipeline_id, parent_info);

        if opts::get().hard_fail {
            // It's quite difficult to make Servo exit cleanly if some tasks have failed.
            // Hard fail exists for test runners so we crash and that's good enough.
            let mut stderr = io::stderr();
            stderr.write_all("Pipeline failed in hard-fail mode.  Crashing!\n".as_bytes()).unwrap();
            process::exit(1);
        }

        self.close_pipeline(pipeline_id, ExitPipelineMode::Force);

        loop {
            let pending_pipeline_id = self.pending_frames.iter().find(|pending| {
                pending.old_pipeline_id == Some(pipeline_id)
            }).map(|frame| frame.new_pipeline_id);
            match pending_pipeline_id {
                Some(pending_pipeline_id) => {
                    debug!("removing pending frame change for failed pipeline");
                    self.close_pipeline(pending_pipeline_id, ExitPipelineMode::Force);
                },
                None => break,
            }
        }
        debug!("creating replacement pipeline for about:failure");

        let window_rect = self.pipeline(pipeline_id).rect;
        let new_pipeline_id =
            self.new_pipeline(parent_info,
                              window_rect,
                              None,
                              LoadData::new(Url::parse("about:failure").unwrap()));

        self.push_pending_frame(new_pipeline_id, Some(pipeline_id));
    }

    fn handle_init_load(&mut self, url: Url) {
        let window_rect = Rect::new(Point2D::zero(), self.window_size.visible_viewport);
        let root_pipeline_id =
            self.new_pipeline(None, Some(window_rect), None, LoadData::new(url.clone()));
        self.handle_load_start_msg(&root_pipeline_id);
        self.push_pending_frame(root_pipeline_id, None);
        self.compositor_proxy.send(CompositorMsg::ChangePageUrl(root_pipeline_id, url));
    }

    fn handle_frame_rect_msg(&mut self, containing_pipeline_id: PipelineId, subpage_id: SubpageId,
                             rect: TypedRect<PagePx, f32>) {
        // Store the new rect inside the pipeline
        let (pipeline_id, script_chan) = {
            // Find the pipeline that corresponds to this rectangle. It's possible that this
            // pipeline may have already exited before we process this message, so just
            // early exit if that occurs.
            let pipeline_id = self.subpage_map.get(&(containing_pipeline_id, subpage_id)).map(|id| *id);
            let pipeline = match pipeline_id {
                Some(pipeline_id) => self.mut_pipeline(pipeline_id),
                None => return,
            };
            pipeline.rect = Some(rect);
            (pipeline.id, pipeline.script_chan.clone())
        };

        script_chan.send(ConstellationControlMsg::Resize(pipeline_id, WindowSizeData {
            visible_viewport: rect.size,
            initial_viewport: rect.size * ScaleFactor::new(1.0),
            device_pixel_ratio: self.window_size.device_pixel_ratio,
        })).unwrap();

        // If this pipeline is in the current frame tree,
        // send the updated rect to the script and compositor tasks
        if self.pipeline_is_in_current_frame(pipeline_id) {
            self.compositor_proxy.send(CompositorMsg::SetLayerRect(
                pipeline_id,
                LayerId::null(),
                rect.to_untyped()));
        }
    }

    // The script task associated with pipeline_id has loaded a URL in an iframe via script. This
    // will result in a new pipeline being spawned and a frame tree being added to
    // containing_page_pipeline_id's frame tree's children. This message is never the result of a
    // page navigation.
    fn handle_script_loaded_url_in_iframe_msg(&mut self,
                                              url: Url,
                                              containing_pipeline_id: PipelineId,
                                              new_subpage_id: SubpageId,
                                              old_subpage_id: Option<SubpageId>,
                                              sandbox: IFrameSandboxState) {
        // Compare the pipeline's url to the new url. If the origin is the same,
        // then reuse the script task in creating the new pipeline
        let script_chan = {
            let source_pipeline = self.pipeline(containing_pipeline_id);

            let source_url = source_pipeline.url.clone();

            let same_script = (source_url.host() == url.host() &&
                               source_url.port() == url.port()) &&
                               sandbox == IFrameSandboxState::IFrameUnsandboxed;

            // FIXME(tkuehn): Need to follow the standardized spec for checking same-origin
            // Reuse the script task if the URL is same-origin
            if same_script {
                debug!("Constellation: loading same-origin iframe, \
                        parent url {:?}, iframe url {:?}", source_url, url);
                Some(source_pipeline.script_chan.clone())
            } else {
                debug!("Constellation: loading cross-origin iframe, \
                        parent url {:?}, iframe url {:?}", source_url, url);
                None
            }
        };

        // Create the new pipeline, attached to the parent and push to pending frames
        let old_pipeline_id = old_subpage_id.map(|old_subpage_id| {
            self.find_subpage(containing_pipeline_id, old_subpage_id).id
        });
        let window_rect = old_pipeline_id.and_then(|old_pipeline_id| {
            self.pipeline(old_pipeline_id).rect
        });
        let new_pipeline_id = self.new_pipeline(Some((containing_pipeline_id, new_subpage_id)),
                                                window_rect,
                                                script_chan,
                                                LoadData::new(url));
        self.subpage_map.insert((containing_pipeline_id, new_subpage_id), new_pipeline_id);
        self.push_pending_frame(new_pipeline_id, old_pipeline_id);
    }

    fn handle_set_cursor_msg(&mut self, cursor: Cursor) {
        self.compositor_proxy.send(CompositorMsg::SetCursor(cursor))
    }

    fn handle_change_running_animations_state(&mut self,
                                              pipeline_id: PipelineId,
                                              animation_state: AnimationState) {
        self.compositor_proxy.send(CompositorMsg::ChangeRunningAnimationsState(pipeline_id,
                                                                               animation_state))
    }

    fn handle_tick_animation(&mut self, pipeline_id: PipelineId) {
        self.pipeline(pipeline_id)
            .layout_chan
            .0
            .send(LayoutControlMsg::TickAnimations)
            .unwrap();
    }

    fn handle_load_url_msg(&mut self, source_id: PipelineId, load_data: LoadData) {
        self.load_url(source_id, load_data);
    }

    fn load_url(&mut self, source_id: PipelineId, load_data: LoadData) -> Option<PipelineId> {
        // If this load targets an iframe, its framing element may exist
        // in a separate script task than the framed document that initiated
        // the new load. The framing element must be notified about the
        // requested change so it can update its internal state.
        match self.pipeline(source_id).parent_info {
            Some((parent_pipeline_id, subpage_id)) => {
                self.handle_load_start_msg(&source_id);
                // Message the constellation to find the script task for this iframe
                // and issue an iframe load through there.
                let parent_pipeline = self.pipeline(parent_pipeline_id);
                let script_channel = &parent_pipeline.script_chan;
                script_channel.send(ConstellationControlMsg::Navigate(parent_pipeline_id,
                                                                      subpage_id,
                                                                      load_data)).unwrap();
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

                self.handle_load_start_msg(&source_id);
                // Being here means either there are no pending frames, or none of the pending
                // changes would be overridden by changing the subframe associated with source_id.

                // Create the new pipeline
                let window_rect = self.pipeline(source_id).rect;
                let new_pipeline_id = self.new_pipeline(None, window_rect, None, load_data);
                self.push_pending_frame(new_pipeline_id, Some(source_id));

                // Send message to ScriptTask that will suspend all timers
                let old_pipeline = self.pipelines.get(&source_id).unwrap();
                old_pipeline.freeze();
                Some(new_pipeline_id)
            }
        }
    }

    fn handle_load_start_msg(&mut self, pipeline_id: &PipelineId) {
        if let Some(id) = self.pipeline_to_frame_map.get(pipeline_id) {
            let forward = !self.frame(*id).next.is_empty();
            let back = !self.frame(*id).prev.is_empty();
            self.compositor_proxy.send(CompositorMsg::LoadStart(back, forward));
        }
    }

    fn handle_load_complete_msg(&mut self, pipeline_id: &PipelineId) {
        let frame_id = match self.pipeline_to_frame_map.get(pipeline_id) {
            Some(frame) => *frame,
            None => {
                debug!("frame not found for pipeline id {:?}", pipeline_id);
                return
            }
        };

        let forward = !self.frame(frame_id).next.is_empty();
        let back = !self.frame(frame_id).prev.is_empty();
        self.compositor_proxy.send(CompositorMsg::LoadComplete(back, forward));
    }

    fn handle_dom_load(&mut self,
                       pipeline_id: PipelineId) {
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
    }

    fn handle_navigate_msg(&mut self,
                           pipeline_info: Option<(PipelineId, SubpageId)>,
                           direction: constellation_msg::NavigationDirection) {
        debug!("received message to navigate {:?}", direction);

        // Get the frame id associated with the pipeline that sent
        // the navigate message, or use root frame id by default.
        let frame_id = pipeline_info.map_or(self.root_frame_id, |(containing_pipeline_id, subpage_id)| {
            let pipeline_id = self.find_subpage(containing_pipeline_id, subpage_id).id;
            self.pipeline_to_frame_map.get(&pipeline_id).map(|id| *id)
        }).unwrap();

        // Check if the currently focused pipeline is the pipeline being replaced
        // (or a child of it). This has to be done here, before the current
        // frame tree is modified below.
        let update_focus_pipeline = self.focused_pipeline_in_tree(frame_id);

        // Get the ids for the previous and next pipelines.
        let (prev_pipeline_id, next_pipeline_id) = {
            let frame = self.mut_frame(frame_id);

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
        };

        // If the currently focused pipeline is the one being changed (or a child
        // of the pipeline being changed) then update the focus pipeline to be
        // the replacement.
        if update_focus_pipeline {
            self.focus_pipeline_id = Some(next_pipeline_id);
        }

        // Suspend the old pipeline, and resume the new one.
        self.pipeline(prev_pipeline_id).freeze();
        self.pipeline(next_pipeline_id).thaw();

        // Set paint permissions correctly for the compositor layers.
        self.revoke_paint_permission(prev_pipeline_id);
        self.send_frame_tree_and_grant_paint_permission();

        // Update the owning iframe to point to the new subpage id.
        // This makes things like contentDocument work correctly.
        if let Some((parent_pipeline_id, subpage_id)) = pipeline_info {
            let script_chan = &self.pipeline(parent_pipeline_id).script_chan;
            let (_, new_subpage_id) = self.pipeline(next_pipeline_id).parent_info.unwrap();
            script_chan.send(ConstellationControlMsg::UpdateSubpageId(parent_pipeline_id,
                                                                      subpage_id,
                                                                      new_subpage_id)).unwrap();

            // If this is an iframe, send a mozbrowser location change event.
            // This is the result of a back/forward navigation.
            self.trigger_mozbrowserlocationchange(next_pipeline_id);
        }
    }

    fn handle_key_msg(&self, key: Key, state: KeyState, mods: KeyModifiers) {
        // Send to the explicitly focused pipeline (if it exists), or the root
        // frame's current pipeline. If neither exist, fall back to sending to
        // the compositor below.
        let target_pipeline_id = self.focus_pipeline_id.or(self.root_frame_id.map(|frame_id| {
            self.frame(frame_id).current
        }));

        match target_pipeline_id {
            Some(target_pipeline_id) => {
                let pipeline = self.pipeline(target_pipeline_id);
                let event = CompositorEvent::KeyEvent(key, state, mods);
                pipeline.script_chan.send(
                    ConstellationControlMsg::SendEvent(pipeline.id, event)).unwrap();
            }
            None => {
                let event = CompositorMsg::KeyEvent(key, state, mods);
                self.compositor_proxy.clone_compositor_proxy().send(event);
            }
        }
    }

    fn handle_get_pipeline_title_msg(&mut self, pipeline_id: PipelineId) {
        match self.pipelines.get(&pipeline_id) {
            None => self.compositor_proxy.send(CompositorMsg::ChangePageTitle(pipeline_id, None)),
            Some(pipeline) => {
                pipeline.script_chan.send(ConstellationControlMsg::GetTitle(pipeline_id)).unwrap();
            }
        }
    }

    fn handle_mozbrowser_event_msg(&mut self,
                                   containing_pipeline_id: PipelineId,
                                   subpage_id: SubpageId,
                                   event: MozBrowserEvent) {
        assert!(prefs::get_pref("dom.mozbrowser.enabled", false));

        // Find the script channel for the given parent pipeline,
        // and pass the event to that script task.
        let pipeline = self.pipeline(containing_pipeline_id);
        pipeline.trigger_mozbrowser_event(subpage_id, event);
    }

    fn handle_get_pipeline(&mut self, frame_id: Option<FrameId>,
                           resp_chan: IpcSender<Option<PipelineId>>) {
        let current_pipeline_id = frame_id.or(self.root_frame_id).map(|frame_id| {
            let frame = self.frames.get(&frame_id).unwrap();
            frame.current
        });
        let pipeline_id = self.pending_frames.iter().rev()
            .find(|x| x.old_pipeline_id == current_pipeline_id)
            .map(|x| x.new_pipeline_id).or(current_pipeline_id);
        resp_chan.send(pipeline_id).unwrap();
    }

    fn handle_get_frame(&mut self,
                        containing_pipeline_id: PipelineId,
                        subpage_id: SubpageId,
                        resp_chan: IpcSender<Option<FrameId>>) {
        let frame_id = self.subpage_map.get(&(containing_pipeline_id, subpage_id)).and_then(
            |x| self.pipeline_to_frame_map.get(&x)).map(|x| *x);
        resp_chan.send(frame_id).unwrap();
    }

    fn focus_parent_pipeline(&self, pipeline_id: PipelineId) {
        // Send a message to the parent of the provided pipeline (if it exists)
        // telling it to mark the iframe element as focused.
        if let Some((containing_pipeline_id, subpage_id)) = self.pipeline(pipeline_id).parent_info {
            let pipeline = self.pipeline(containing_pipeline_id);
            let event = ConstellationControlMsg::FocusIFrame(containing_pipeline_id,
                                                                subpage_id);
            pipeline.script_chan.send(event).unwrap();

            self.focus_parent_pipeline(containing_pipeline_id);
        }
    }

    fn handle_focus_msg(&mut self, pipeline_id: PipelineId) {
        self.focus_pipeline_id = Some(pipeline_id);

        // Focus parent iframes recursively
        self.focus_parent_pipeline(pipeline_id);
    }

    fn handle_remove_iframe_msg(&mut self, containing_pipeline_id: PipelineId, subpage_id: SubpageId) {
        let pipeline_id = self.find_subpage(containing_pipeline_id, subpage_id).id;
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

    fn handle_create_canvas_paint_task_msg(
            &mut self,
            size: &Size2D<i32>,
            response_sender: IpcSender<(IpcSender<CanvasMsg>, usize)>) {
        let id = self.canvas_paint_tasks.len();
        let (out_of_process_sender, in_process_sender) = CanvasPaintTask::start(*size);
        self.canvas_paint_tasks.push(in_process_sender);
        response_sender.send((out_of_process_sender, id)).unwrap()
    }

    fn handle_create_webgl_paint_task_msg(
            &mut self,
            size: &Size2D<i32>,
            attributes: GLContextAttributes,
            response_sender: IpcSender<Result<(IpcSender<CanvasMsg>, usize), String>>) {
        let response = match WebGLPaintTask::start(*size, attributes) {
            Ok((out_of_process_sender, in_process_sender)) => {
                let id = self.webgl_paint_tasks.len();
                self.webgl_paint_tasks.push(in_process_sender);
                Ok((out_of_process_sender, id))
            },
            Err(msg) => Err(msg.to_owned()),
        };

        response_sender.send(response).unwrap()
    }

    fn handle_webdriver_msg(&mut self, msg: WebDriverCommandMsg) {
        // Find the script channel for the given parent pipeline,
        // and pass the event to that script task.
        match msg {
            WebDriverCommandMsg::LoadUrl(pipeline_id, load_data, reply) => {
                self.load_url_for_webdriver(pipeline_id, load_data, reply);
            },
            WebDriverCommandMsg::Refresh(pipeline_id, reply) => {
                let load_data = {
                    let pipeline = self.pipeline(pipeline_id);
                    LoadData::new(pipeline.url.clone())
                };
                self.load_url_for_webdriver(pipeline_id, load_data, reply);
            }
            WebDriverCommandMsg::ScriptCommand(pipeline_id, cmd) => {
                let pipeline = self.pipeline(pipeline_id);
                let control_msg = ConstellationControlMsg::WebDriverScriptCommand(pipeline_id, cmd);
                pipeline.script_chan.send(control_msg).unwrap();
            },
            WebDriverCommandMsg::TakeScreenshot(pipeline_id, reply) => {
                let current_pipeline_id = self.root_frame_id.map(|frame_id| {
                    let frame = self.frames.get(&frame_id).unwrap();
                    frame.current
                });
                if Some(pipeline_id) == current_pipeline_id {
                    self.compositor_proxy.send(CompositorMsg::CreatePng(reply));
                } else {
                    reply.send(None).unwrap();
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
            let old_frame_id = *self.pipeline_to_frame_map.get(&old_pipeline_id).unwrap();
            if self.focused_pipeline_in_tree(old_frame_id) {
                self.focus_pipeline_id = Some(frame_change.new_pipeline_id);
            }
        }

        let evicted_frames = match frame_change.old_pipeline_id {
            Some(old_pipeline_id) => {
                // The new pipeline is replacing an old one.
                // Remove paint permissions for the pipeline being replaced.
                self.revoke_paint_permission(old_pipeline_id);

                // Add new pipeline to navigation frame, and return frames evicted from history.
                let frame_id = *self.pipeline_to_frame_map.get(&old_pipeline_id).unwrap();
                let evicted_frames = self.mut_frame(frame_id).load(frame_change.new_pipeline_id);
                self.pipeline_to_frame_map.insert(frame_change.new_pipeline_id, frame_id);

                Some(evicted_frames)
            }
            None => {
                // The new pipeline is in a new frame with no history
                let frame_id = self.new_frame(frame_change.new_pipeline_id);

                // If a child frame, add it to the parent pipeline. Otherwise
                // it must surely be the root frame being created!
                match self.pipeline(frame_change.new_pipeline_id).parent_info {
                    Some((parent_id, _)) => {
                        self.mut_pipeline(parent_id).add_child(frame_id);
                    }
                    None => {
                        assert!(self.root_frame_id.is_none());
                        self.root_frame_id = Some(frame_id);
                    }
                }

                // No evicted frames if a new frame was created
                None
            }
        };

        // Build frame tree and send permission
        self.send_frame_tree_and_grant_paint_permission();

        // If this is an iframe, send a mozbrowser location change event.
        // This is the result of a link being clicked and a navigation completing.
        self.trigger_mozbrowserlocationchange(frame_change.new_pipeline_id);

        // Remove any evicted frames
        if let Some(evicted_frames) = evicted_frames {
            for pipeline_id in &evicted_frames {
                self.close_pipeline(*pipeline_id, ExitPipelineMode::Normal);
            }
        }
    }

    fn handle_painter_ready_msg(&mut self, pipeline_id: PipelineId) {
        debug!("Painter {:?} ready to send paint msg", pipeline_id);
        // This message could originate from a pipeline in the navigation context or
        // from a pending frame. The only time that we will grant paint permission is
        // when the message originates from a pending frame or the current frame.

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
            self.pending_frames[pending_index].painter_ready = true;
        }

        // This is a bit complex. We need to loop through pending frames and find
        // ones that can be swapped. A frame can be swapped (enabled) once it is
        // ready to paint (has painter_ready set), and also has no dependencies
        // (i.e. the pipeline it is replacing has been enabled and now has a frame).
        // The outer loop is required because any time a pipeline is enabled, that
        // may affect whether other pending frames are now able to be enabled. On the
        // other hand, if no frames can be enabled after looping through all pending
        // frames, we can safely exit the loop, knowing that we will need to wait on
        // a dependent pipeline to be ready to paint.
        loop {
            let valid_frame_change = self.pending_frames.iter().rposition(|frame_change| {
                let waiting_on_dependency = frame_change.old_pipeline_id.map_or(false, |old_pipeline_id| {
                    self.pipeline_to_frame_map.get(&old_pipeline_id).is_none()
                });
                frame_change.painter_ready && !waiting_on_dependency
            });

            if let Some(valid_frame_change) = valid_frame_change {
                let frame_change = self.pending_frames.swap_remove(valid_frame_change);
                self.add_or_replace_pipeline_in_frame_tree(frame_change);
            } else {
                break;
            }
        }
    }

    /// Called when the window is resized.
    fn handle_resized_window_msg(&mut self, new_size: WindowSizeData) {
        debug!("handle_resized_window_msg: {:?} {:?}", new_size.initial_viewport.to_untyped(),
                                                       new_size.visible_viewport.to_untyped());

        if let Some(root_frame_id) = self.root_frame_id {
            // Send Resize (or ResizeInactive) messages to each
            // pipeline in the frame tree.
            let frame = self.frames.get(&root_frame_id).unwrap();

            let pipeline = self.pipelines.get(&frame.current).unwrap();
            let _ = pipeline.script_chan.send(ConstellationControlMsg::Resize(pipeline.id, new_size));

            for pipeline_id in frame.prev.iter().chain(&frame.next) {
                let pipeline = self.pipelines.get(pipeline_id).unwrap();
                let _ = pipeline.script_chan.send(ConstellationControlMsg::ResizeInactive(pipeline.id, new_size));
            }
        }

        // Send resize message to any pending pipelines that aren't loaded yet.
        for pending_frame in &self.pending_frames {
            let pipeline = self.pipelines.get(&pending_frame.new_pipeline_id).unwrap();
            if pipeline.parent_info.is_none() {
                let _ = pipeline.script_chan.send(ConstellationControlMsg::Resize(pipeline.id, new_size));
            }
        }

        self.window_size = new_size;
    }

    /// Handle updating actual viewport / zoom due to @viewport rules
    fn handle_viewport_constrained_msg(&mut self, pipeline_id: PipelineId, constraints: ViewportConstraints) {
        self.compositor_proxy.send(CompositorMsg::ViewportConstrained(pipeline_id, constraints));
    }

    /// Checks the state of all script and layout pipelines to see if they are idle
    /// and compares the current layout state to what the compositor has. This is used
    /// to check if the output image is "stable" and can be written as a screenshot
    /// for reftests.
    fn handle_is_ready_to_save_image(&mut self,
                                     pipeline_states: HashMap<PipelineId, Epoch>) -> bool {
        // If there is no root frame yet, the initial page has
        // not loaded, so there is nothing to save yet.
        if self.root_frame_id.is_none() {
            return false;
        }

        // If there are pending changes to the current frame
        // tree, the image is not stable yet.
        if self.pending_frames.len() > 0 {
            return false;
        }

        // Step through the current frame tree, checking that the script
        // task is idle, and that the current epoch of the layout task
        // matches what the compositor has painted. If all these conditions
        // are met, then the output image should not change and a reftest
        // screenshot can safely be written.
        for frame in self.current_frame_tree_iter(self.root_frame_id) {
            let pipeline = self.pipeline(frame.current);

            // Synchronously query the script task for this pipeline
            // to see if it is idle.
            let (sender, receiver) = channel();
            let msg = ConstellationControlMsg::GetCurrentState(sender, frame.current);
            pipeline.script_chan.send(msg).unwrap();
            if receiver.recv().unwrap() == ScriptState::DocumentLoading {
                return false;
            }

            // Check the visible rectangle for this pipeline. If the constellation
            // hasn't received a rectangle for this pipeline yet, then assume
            // that the output image isn't stable yet.
            match pipeline.rect {
                Some(rect) => {
                    // If the rectangle for this pipeline is zero sized, it will
                    // never be painted. In this case, don't query the layout
                    // task as it won't contribute to the final output image.
                    if rect.size == Size2D::zero() {
                        continue;
                    }

                    // Get the epoch that the compositor has drawn for this pipeline.
                    let compositor_epoch = pipeline_states.get(&frame.current);
                    match compositor_epoch {
                        Some(compositor_epoch) => {
                            // Synchronously query the layout task to see if the current
                            // epoch matches what the compositor has drawn. If they match
                            // (and script is idle) then this pipeline won't change again
                            // and can be considered stable.
                            let (sender, receiver) = ipc::channel().unwrap();
                            let LayoutControlChan(ref layout_chan) = pipeline.layout_chan;
                            layout_chan.send(LayoutControlMsg::GetCurrentEpoch(sender)).unwrap();
                            let layout_task_epoch = receiver.recv().unwrap();
                            if layout_task_epoch != *compositor_epoch {
                                return false;
                            }
                        }
                        None => {
                            // The compositor doesn't know about this pipeline yet.
                            // Assume it hasn't rendered yet.
                            return false;
                        }
                    }
                }
                None => {
                    return false;
                }
            }
        }

        // All script tasks are idle and layout epochs match compositor, so output image!
        true
    }

    // Close a frame (and all children)
    fn close_frame(&mut self, frame_id: FrameId, exit_mode: ExitPipelineMode) {
        // Store information about the pipelines to be closed. Then close the
        // pipelines, before removing ourself from the frames hash map. This
        // ordering is vital - so that if close_pipeline() ends up closing
        // any child frames, they can be removed from the parent frame correctly.
        let parent_info = self.pipeline(self.frame(frame_id).current).parent_info;
        let pipelines_to_close = {
            let mut pipelines_to_close = vec!();

            let frame = self.frame(frame_id);
            pipelines_to_close.push_all(&frame.next);
            pipelines_to_close.push(frame.current);
            pipelines_to_close.push_all(&frame.prev);

            pipelines_to_close
        };

        for pipeline_id in &pipelines_to_close {
            self.close_pipeline(*pipeline_id, exit_mode);
        }

        self.frames.remove(&frame_id).unwrap();

        if let Some((parent_pipeline_id, _)) = parent_info {
            let parent_pipeline = self.mut_pipeline(parent_pipeline_id);
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

            let pipeline = self.pipeline(pipeline_id);
            frames_to_close.push_all(&pipeline.children);

            frames_to_close
        };

        // Remove any child frames
        for child_frame in &frames_to_close {
            self.close_frame(*child_frame, exit_mode);
        }

        let pipeline = self.pipelines.remove(&pipeline_id).unwrap();

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
            ExitPipelineMode::Normal => pipeline.exit(PipelineExitType::PipelineOnly),
            ExitPipelineMode::Force => pipeline.force_exit(),
        }
    }

    // Convert a frame to a sendable form to pass to the compositor
    fn frame_to_sendable(&self, frame_id: FrameId) -> SendableFrameTree {
        let pipeline = self.pipeline(self.frame(frame_id).current);

        let mut frame_tree = SendableFrameTree {
            pipeline: pipeline.to_sendable(),
            rect: pipeline.rect,
            children: vec!(),
        };

        for child_frame_id in &pipeline.children {
            frame_tree.children.push(self.frame_to_sendable(*child_frame_id));
        }

        frame_tree
    }

    // Revoke paint permission from a pipeline, and all children.
    fn revoke_paint_permission(&self, pipeline_id: PipelineId) {
        let frame_id = self.pipeline_to_frame_map.get(&pipeline_id).map(|frame_id| *frame_id);
        for frame in self.current_frame_tree_iter(frame_id) {
            self.pipeline(frame.current).revoke_paint_permission();
        }
    }

    // Send the current frame tree to compositor, and grant paint
    // permission to each pipeline in the current frame tree.
    fn send_frame_tree_and_grant_paint_permission(&mut self) {
        if let Some(root_frame_id) = self.root_frame_id {
            let frame_tree = self.frame_to_sendable(root_frame_id);

            let (chan, port) = channel();
            self.compositor_proxy.send(CompositorMsg::SetFrameTree(frame_tree,
                                                                   chan,
                                                                   self.chan.clone()));
            if port.recv().is_err() {
                debug!("Compositor has discarded SetFrameTree");
                return; // Our message has been discarded, probably shutting down.
            }
        }

        for frame in self.current_frame_tree_iter(self.root_frame_id) {
            self.pipeline(frame.current).grant_paint_permission();
        }
    }

    // https://developer.mozilla.org/en-US/docs/Web/Events/mozbrowserlocationchange
    fn trigger_mozbrowserlocationchange(&self, pipeline_id: PipelineId) {
        if prefs::get_pref("dom.mozbrowser.enabled", false) {
            // Work around borrow checker
            let event_info = {
                let pipeline = self.pipeline(pipeline_id);

                pipeline.parent_info.map(|(containing_pipeline_id, subpage_id)| {
                    (containing_pipeline_id, subpage_id, pipeline.url.serialize())
                })
            };

            // If this is an iframe, then send the event with new url
            if let Some((containing_pipeline_id, subpage_id, url)) = event_info {
                let parent_pipeline = self.pipeline(containing_pipeline_id);
                parent_pipeline.trigger_mozbrowser_event(subpage_id, MozBrowserEvent::LocationChange(url));
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

    #[inline(always)]
    fn frame(&self, frame_id: FrameId) -> &Frame {
        self.frames.get(&frame_id).expect("unable to find frame - this is a bug")
    }

    #[inline(always)]
    fn mut_frame(&mut self, frame_id: FrameId) -> &mut Frame {
        self.frames.get_mut(&frame_id).expect("unable to find frame - this is a bug")
    }

    #[inline(always)]
    fn pipeline(&self, pipeline_id: PipelineId) -> &Pipeline {
        self.pipelines.get(&pipeline_id).expect("unable to find pipeline - this is a bug")
    }

    #[inline(always)]
    fn mut_pipeline(&mut self, pipeline_id: PipelineId) -> &mut Pipeline {
        self.pipelines.get_mut(&pipeline_id).expect("unable to find pipeline - this is a bug")
    }

    fn find_subpage(&mut self, containing_pipeline_id: PipelineId, subpage_id: SubpageId) -> &mut Pipeline {
        let pipeline_id = *self.subpage_map
                               .get(&(containing_pipeline_id, subpage_id))
                               .expect("no subpage pipeline_id");
        self.mut_pipeline(pipeline_id)
    }
}
