/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositor_layer::{CompositorData, CompositorLayer, DoesntWantScrollEvents};
use compositor_layer::WantsScrollEvents;
use compositor_task::{ChangeReadyState, ChangeRenderState, CompositorEventListener};
use compositor_task::{CompositorProxy, CompositorReceiver, CompositorTask};
use compositor_task::{CreateOrUpdateDescendantLayer, CreateOrUpdateRootLayer, Exit};
use compositor_task::{FrameTreeUpdateMsg, GetGraphicsMetadata, LayerProperties};
use compositor_task::{LoadComplete, Msg, Paint, RenderMsgDiscarded, ScrollFragmentPoint};
use compositor_task::{ScrollTimeout, SetIds, SetLayerOrigin, ShutdownComplete};
use constellation::{SendableFrameTree, FrameTreeDiff};
use pipeline::CompositionPipeline;
use scrolling::ScrollingTimerProxy;
use windowing;
use windowing::{IdleWindowEvent, LoadUrlWindowEvent, MouseWindowClickEvent};
use windowing::{MouseWindowEvent, MouseWindowEventClass, MouseWindowMouseDownEvent};
use windowing::{MouseWindowMouseUpEvent, MouseWindowMoveEventClass, NavigationWindowEvent};
use windowing::{QuitWindowEvent, RefreshWindowEvent, ResizeWindowEvent, ScrollWindowEvent};
use windowing::{WindowEvent, WindowMethods, WindowNavigateMsg, ZoomWindowEvent};
use windowing::{PinchZoomWindowEvent, KeyEvent};

use azure::azure_hl;
use std::cmp;
use std::mem;
use std::num::Zero;
use geom::point::{Point2D, TypedPoint2D};
use geom::rect::{Rect, TypedRect};
use geom::size::TypedSize2D;
use geom::scale_factor::ScaleFactor;
use gfx::render_task::{RenderChan, RenderMsg, RenderRequest, UnusedBufferMsg};
use layers::geometry::{DevicePixel, LayerPixel};
use layers::layers::{BufferRequest, Layer, LayerBufferSet};
use layers::rendergl;
use layers::rendergl::RenderContext;
use layers::scene::Scene;
use png;
use gleam::gl::types::{GLint, GLsizei};
use gleam::gl;
use script_traits::{ViewportMsg, ScriptControlChan};
use servo_msg::compositor_msg::{Blank, Epoch, FinishedLoading, IdleRenderState, LayerId};
use servo_msg::compositor_msg::{ReadyState, RenderingRenderState, RenderState, Scrollable};
use servo_msg::constellation_msg::{ConstellationChan, ExitMsg, LoadUrlMsg};
use servo_msg::constellation_msg::{NavigateMsg, LoadData, PipelineId, ResizedWindowMsg};
use servo_msg::constellation_msg::{WindowSizeData, KeyState, Key, KeyModifiers};
use servo_msg::constellation_msg;
use servo_util::geometry::{PagePx, ScreenPx, ViewportPx};
use servo_util::memory::MemoryProfilerChan;
use servo_util::opts;
use servo_util::time::{profile, TimeProfilerChan};
use servo_util::{memory, time};
use std::collections::HashMap;
use std::collections::hash_map::{Occupied, Vacant};
use std::path::Path;
use std::rc::Rc;
use std::slice::bytes::copy_memory;
use time::{precise_time_ns, precise_time_s};
use url::Url;

pub struct IOCompositor<Window: WindowMethods> {
    /// The application window.
    window: Rc<Window>,

    /// The port on which we receive messages.
    port: Box<CompositorReceiver>,

    /// The render context.
    context: RenderContext,

    /// The root pipeline.
    root_pipeline: Option<CompositionPipeline>,

    /// The canvas to paint a page.
    scene: Scene<CompositorData>,

    /// The application window size.
    window_size: TypedSize2D<DevicePixel, uint>,

    /// "Mobile-style" zoom that does not reflow the page.
    viewport_zoom: ScaleFactor<PagePx, ViewportPx, f32>,

    /// "Desktop-style" zoom that resizes the viewport to fit the window.
    /// See `ViewportPx` docs in util/geom.rs for details.
    page_zoom: ScaleFactor<ViewportPx, ScreenPx, f32>,

    /// The device pixel ratio for this window.
    hidpi_factor: ScaleFactor<ScreenPx, DevicePixel, f32>,

    /// A handle to the scrolling timer.
    scrolling_timer: ScrollingTimerProxy,

    /// Tracks whether we should composite this frame.
    composition_request: CompositionRequest,

    /// Tracks whether we are in the process of shutting down, or have shut down and should close
    /// the compositor.
    shutdown_state: ShutdownState,

    /// Tracks outstanding render_msg's sent to the render tasks.
    outstanding_render_msgs: uint,

    /// Tracks the last composite time.
    last_composite_time: u64,

    /// Tracks whether the zoom action has happened recently.
    zoom_action: bool,

    /// The time of the last zoom action has started.
    zoom_time: f64,

    /// Current display/reflow status of each pipeline.
    ready_states: HashMap<PipelineId, ReadyState>,

    /// Current render status of each pipeline.
    render_states: HashMap<PipelineId, RenderState>,

    /// Whether the page being rendered has loaded completely.
    /// Differs from ReadyState because we can finish loading (ready)
    /// many times for a single page.
    got_load_complete_message: bool,

    /// Whether we have gotten a `SetIds` message.
    got_set_ids_message: bool,

    /// The channel on which messages can be sent to the constellation.
    constellation_chan: ConstellationChan,

    /// The channel on which messages can be sent to the time profiler.
    time_profiler_chan: TimeProfilerChan,

    /// The channel on which messages can be sent to the memory profiler.
    memory_profiler_chan: MemoryProfilerChan,

    /// Pending scroll to fragment event, if any
    fragment_point: Option<Point2D<f32>>,

    /// Pending scroll events.
    pending_scroll_events: Vec<ScrollEvent>,
}

pub struct ScrollEvent {
    delta: TypedPoint2D<DevicePixel,f32>,
    cursor: TypedPoint2D<DevicePixel,i32>,
}

#[deriving(PartialEq)]
enum CompositionRequest {
    NoCompositingNecessary,
    CompositeOnScrollTimeout(u64),
    CompositeNow,
}

#[deriving(PartialEq, Show)]
enum ShutdownState {
    NotShuttingDown,
    ShuttingDown,
    FinishedShuttingDown,
}

struct HitTestResult {
    layer: Rc<Layer<CompositorData>>,
    point: TypedPoint2D<LayerPixel, f32>,
}

impl<Window: WindowMethods> IOCompositor<Window> {
    fn new(window: Rc<Window>,
           sender: Box<CompositorProxy+Send>,
           receiver: Box<CompositorReceiver>,
           constellation_chan: ConstellationChan,
           time_profiler_chan: TimeProfilerChan,
           memory_profiler_chan: MemoryProfilerChan)
           -> IOCompositor<Window> {
        // Create an initial layer tree.
        //
        // TODO: There should be no initial layer tree until the renderer creates one from the
        // display list. This is only here because we don't have that logic in the renderer yet.
        let window_size = window.framebuffer_size();
        let hidpi_factor = window.hidpi_factor();
        let context = CompositorTask::create_graphics_context(&window.native_metadata());

        let show_debug_borders = opts::get().show_debug_borders;
        IOCompositor {
            window: window,
            port: receiver,
            context: rendergl::RenderContext::new(context, show_debug_borders),
            root_pipeline: None,
            scene: Scene::new(Rect {
                origin: Zero::zero(),
                size: window_size.as_f32(),
            }),
            window_size: window_size,
            hidpi_factor: hidpi_factor,
            scrolling_timer: ScrollingTimerProxy::new(sender),
            composition_request: NoCompositingNecessary,
            pending_scroll_events: Vec::new(),
            shutdown_state: NotShuttingDown,
            page_zoom: ScaleFactor(1.0),
            viewport_zoom: ScaleFactor(1.0),
            zoom_action: false,
            zoom_time: 0f64,
            ready_states: HashMap::new(),
            render_states: HashMap::new(),
            got_load_complete_message: false,
            got_set_ids_message: false,
            constellation_chan: constellation_chan,
            time_profiler_chan: time_profiler_chan,
            memory_profiler_chan: memory_profiler_chan,
            fragment_point: None,
            outstanding_render_msgs: 0,
            last_composite_time: 0,
        }
    }

    pub fn create(window: Rc<Window>,
                  sender: Box<CompositorProxy+Send>,
                  receiver: Box<CompositorReceiver>,
                  constellation_chan: ConstellationChan,
                  time_profiler_chan: TimeProfilerChan,
                  memory_profiler_chan: MemoryProfilerChan)
                  -> IOCompositor<Window> {
        let mut compositor = IOCompositor::new(window,
                                               sender,
                                               receiver,
                                               constellation_chan,
                                               time_profiler_chan,
                                               memory_profiler_chan);

        // Set the size of the root layer.
        compositor.update_zoom_transform();

        // Tell the constellation about the initial window size.
        compositor.send_window_size();

        compositor
    }

    fn handle_browser_message(&mut self, msg: Msg) -> bool {
        match (msg, self.shutdown_state) {
            (_, FinishedShuttingDown) =>
                panic!("compositor shouldn't be handling messages after shutting down"),

            (Exit(chan), _) => {
                debug!("shutting down the constellation");
                let ConstellationChan(ref con_chan) = self.constellation_chan;
                con_chan.send(ExitMsg);
                chan.send(());
                self.shutdown_state = ShuttingDown;
            }

            (ShutdownComplete, _) => {
                debug!("constellation completed shutdown");
                self.shutdown_state = FinishedShuttingDown;
                return false;
            }

            (ChangeReadyState(pipeline_id, ready_state), NotShuttingDown) => {
                self.change_ready_state(pipeline_id, ready_state);
            }

            (ChangeRenderState(pipeline_id, render_state), NotShuttingDown) => {
                self.change_render_state(pipeline_id, render_state);
            }

            (RenderMsgDiscarded, NotShuttingDown) => {
                self.remove_outstanding_render_msg();
            }

            (SetIds(frame_tree, response_chan, new_constellation_chan), NotShuttingDown) => {
                self.set_frame_tree(&frame_tree,
                                    response_chan,
                                    new_constellation_chan);
                self.send_viewport_rects_for_all_layers();
            }

            (FrameTreeUpdateMsg(frame_tree_diff, response_channel), NotShuttingDown) => {
                self.update_frame_tree(&frame_tree_diff);
                response_channel.send(());
            }

            (CreateOrUpdateRootLayer(layer_properties), NotShuttingDown) => {
                self.create_or_update_root_layer(layer_properties);
            }

            (CreateOrUpdateDescendantLayer(layer_properties), NotShuttingDown) => {
                self.create_or_update_descendant_layer(layer_properties);
            }

            (GetGraphicsMetadata(chan), NotShuttingDown) => {
                chan.send(Some(self.window.native_metadata()));
            }

            (SetLayerOrigin(pipeline_id, layer_id, origin), NotShuttingDown) => {
                self.set_layer_origin(pipeline_id, layer_id, origin);
            }

            (Paint(pipeline_id, epoch, replies), NotShuttingDown) => {
                for (layer_id, new_layer_buffer_set) in replies.into_iter() {
                    self.paint(pipeline_id, layer_id, new_layer_buffer_set, epoch);
                }
                self.remove_outstanding_render_msg();
            }

            (ScrollFragmentPoint(pipeline_id, layer_id, point), NotShuttingDown) => {
                self.scroll_fragment_to_point(pipeline_id, layer_id, point);
            }

            (LoadComplete(..), NotShuttingDown) => {
                self.got_load_complete_message = true;

                // If we're rendering in headless mode, schedule a recomposite.
                if opts::get().output_file.is_some() {
                    self.composite_if_necessary();
                }
            }

            (ScrollTimeout(timestamp), NotShuttingDown) => {
                debug!("scroll timeout, drawing unrendered content!");
                match self.composition_request {
                    CompositeOnScrollTimeout(this_timestamp) if timestamp == this_timestamp => {
                        self.composition_request = CompositeNow
                    }
                    _ => {}
                }
            }

            // When we are shutting_down, we need to avoid performing operations
            // such as Paint that may crash because we have begun tearing down
            // the rest of our resources.
            (_, ShuttingDown) => { }
        }

        true
    }

    fn change_ready_state(&mut self, pipeline_id: PipelineId, ready_state: ReadyState) {
        match self.ready_states.entry(pipeline_id) {
            Occupied(entry) => {
                *entry.into_mut() = ready_state;
            }
            Vacant(entry) => {
                entry.set(ready_state);
            }
        }
        self.window.set_ready_state(self.get_earliest_pipeline_ready_state());

        // If we're rendering in headless mode, schedule a recomposite.
        if opts::get().output_file.is_some() {
            self.composite_if_necessary()
        }
    }

    fn get_earliest_pipeline_ready_state(&self) -> ReadyState {
        if self.ready_states.len() == 0 {
            return Blank;
        }
        return self.ready_states.values().fold(FinishedLoading, |a, &b| cmp::min(a, b));

    }

    fn change_render_state(&mut self, pipeline_id: PipelineId, render_state: RenderState) {
        match self.render_states.entry(pipeline_id) {
            Occupied(entry) => {
                *entry.into_mut() = render_state;
            }
            Vacant(entry) => {
                entry.set(render_state);
            }
        }

        self.window.set_render_state(render_state);
    }

    fn all_pipelines_in_idle_render_state(&self) -> bool {
        if self.ready_states.len() == 0 {
            return false;
        }
        return self.render_states.values().all(|&value| value == IdleRenderState);
    }

    fn has_render_msg_tracking(&self) -> bool {
        // only track RenderMsg's if the compositor outputs to a file.
        opts::get().output_file.is_some()
    }

    fn has_outstanding_render_msgs(&self) -> bool {
        self.has_render_msg_tracking() && self.outstanding_render_msgs > 0
    }

    fn add_outstanding_render_msg(&mut self, count: uint) {
        // return early if not tracking render_msg's
        if !self.has_render_msg_tracking() {
            return;
        }
        debug!("add_outstanding_render_msg {}", self.outstanding_render_msgs);
        self.outstanding_render_msgs += count;
    }

    fn remove_outstanding_render_msg(&mut self) {
        if !self.has_render_msg_tracking() {
            return;
        }
        if self.outstanding_render_msgs > 0 {
            self.outstanding_render_msgs -= 1;
        } else {
            debug!("too many rerender msgs completed");
        }
    }

    fn set_frame_tree(&mut self,
                      frame_tree: &SendableFrameTree,
                      response_chan: Sender<()>,
                      new_constellation_chan: ConstellationChan) {
        response_chan.send(());

        self.root_pipeline = Some(frame_tree.pipeline.clone());

        // If we have an old root layer, release all old tiles before replacing it.
        match self.scene.root {
            Some(ref mut layer) => layer.clear_all_tiles(),
            None => { }
        }
        self.scene.root = Some(self.create_frame_tree_root_layers(frame_tree, None));
        self.scene.set_root_layer_size(self.window_size.as_f32());

        // Initialize the new constellation channel by sending it the root window size.
        self.constellation_chan = new_constellation_chan;
        self.send_window_size();

        self.got_set_ids_message = true;
        self.composite_if_necessary();
    }

    fn create_frame_tree_root_layers(&mut self,
                                     frame_tree: &SendableFrameTree,
                                     frame_rect: Option<TypedRect<PagePx, f32>>)
                                     -> Rc<Layer<CompositorData>> {
        // Initialize the ReadyState and RenderState for this pipeline.
        self.ready_states.insert(frame_tree.pipeline.id, Blank);
        self.render_states.insert(frame_tree.pipeline.id, RenderingRenderState);

        let root_layer = create_root_layer_for_pipeline_and_rect(&frame_tree.pipeline, frame_rect);
        for kid in frame_tree.children.iter() {
            root_layer.add_child(self.create_frame_tree_root_layers(&kid.frame_tree, kid.rect));
        }
        return root_layer;
    }

    fn update_frame_tree(&mut self, frame_tree_diff: &FrameTreeDiff) {
        let parent_layer = self.find_pipeline_root_layer(frame_tree_diff.parent_pipeline.id);
        parent_layer.add_child(
            create_root_layer_for_pipeline_and_rect(&frame_tree_diff.pipeline,
                                                    frame_tree_diff.rect));
    }

    fn find_pipeline_root_layer(&self, pipeline_id: PipelineId) -> Rc<Layer<CompositorData>> {
        match self.find_layer_with_pipeline_and_layer_id(pipeline_id, LayerId::null()) {
            Some(ref layer) => layer.clone(),
            None => panic!("Tried to create or update layer for unknown pipeline"),
        }
    }

    fn update_layer_if_exists(&mut self, properties: LayerProperties) -> bool {
        match self.find_layer_with_pipeline_and_layer_id(properties.pipeline_id, properties.id) {
            Some(existing_layer) => {
                existing_layer.update_layer(properties);
                true
            }
            None => false,
        }
    }

    fn create_or_update_root_layer(&mut self, layer_properties: LayerProperties) {
        let need_new_root_layer = !self.update_layer_if_exists(layer_properties);
        if need_new_root_layer {
            let root_layer = self.find_pipeline_root_layer(layer_properties.pipeline_id);
            root_layer.update_layer_except_size(layer_properties);

            let root_layer_pipeline = root_layer.extra_data.borrow().pipeline.clone();
            let first_child = CompositorData::new_layer(root_layer_pipeline.clone(),
                                                        layer_properties,
                                                        DoesntWantScrollEvents,
                                                        opts::get().tile_size);

            // Add the first child / base layer to the front of the child list, so that
            // child iframe layers are rendered on top of the base layer. These iframe
            // layers were added previously when creating the layer tree skeleton in
            // create_frame_tree_root_layers.
            root_layer.children().insert(0, first_child);
        }

        self.scroll_layer_to_fragment_point_if_necessary(layer_properties.pipeline_id,
                                                         layer_properties.id);
        self.send_buffer_requests_for_all_layers();
    }

    fn create_or_update_descendant_layer(&mut self, layer_properties: LayerProperties) {
        if !self.update_layer_if_exists(layer_properties) {
            self.create_descendant_layer(layer_properties);
        }
        self.scroll_layer_to_fragment_point_if_necessary(layer_properties.pipeline_id,
                                                         layer_properties.id);
        self.send_buffer_requests_for_all_layers();
    }

    fn create_descendant_layer(&self, layer_properties: LayerProperties) {
        let root_layer = self.find_pipeline_root_layer(layer_properties.pipeline_id);
        let root_layer_pipeline = root_layer.extra_data.borrow().pipeline.clone();
        let new_layer = CompositorData::new_layer(root_layer_pipeline,
                                                  layer_properties,
                                                  DoesntWantScrollEvents,
                                                  root_layer.tile_size);
        root_layer.add_child(new_layer);
    }

    fn send_window_size(&self) {
        let dppx = self.page_zoom * self.device_pixels_per_screen_px();
        let initial_viewport = self.window_size.as_f32() / dppx;
        let visible_viewport = initial_viewport / self.viewport_zoom;

        let ConstellationChan(ref chan) = self.constellation_chan;
        chan.send(ResizedWindowMsg(WindowSizeData {
            device_pixel_ratio: dppx,
            initial_viewport: initial_viewport,
            visible_viewport: visible_viewport,
        }));
    }

    pub fn move_layer(&self,
                      pipeline_id: PipelineId,
                      layer_id: LayerId,
                      origin: TypedPoint2D<LayerPixel, f32>)
                      -> bool {
        match self.find_layer_with_pipeline_and_layer_id(pipeline_id, layer_id) {
            Some(ref layer) => {
                if layer.extra_data.borrow().wants_scroll_events == WantsScrollEvents {
                    layer.clamp_scroll_offset_and_scroll_layer(TypedPoint2D(0f32, 0f32) - origin);
                }
                true
            }
            None => false,
        }
    }

    fn scroll_layer_to_fragment_point_if_necessary(&mut self,
                                                   pipeline_id: PipelineId,
                                                   layer_id: LayerId) {
        match self.fragment_point.take() {
            Some(point) => {
                if !self.move_layer(pipeline_id, layer_id, Point2D::from_untyped(&point)) {
                    panic!("Compositor: Tried to scroll to fragment with unknown layer.");
                }

                self.start_scrolling_timer_if_necessary();
            }
            None => {}
        }
    }

    fn start_scrolling_timer_if_necessary(&mut self) {
        match self.composition_request {
            CompositeNow | CompositeOnScrollTimeout(_) => return,
            NoCompositingNecessary => {}
        }

        let timestamp = precise_time_ns();
        self.scrolling_timer.scroll_event_processed(timestamp);
        self.composition_request = CompositeOnScrollTimeout(timestamp);
    }

    fn set_layer_origin(&mut self,
                        pipeline_id: PipelineId,
                        layer_id: LayerId,
                        new_origin: Point2D<f32>) {
        match self.find_layer_with_pipeline_and_layer_id(pipeline_id, layer_id) {
            Some(ref layer) => {
                layer.bounds.borrow_mut().origin = Point2D::from_untyped(&new_origin)
            }
            None => panic!("Compositor received SetLayerOrigin for nonexistent layer"),
        };

        self.send_buffer_requests_for_all_layers();
    }

    fn paint(&mut self,
             pipeline_id: PipelineId,
             layer_id: LayerId,
             new_layer_buffer_set: Box<LayerBufferSet>,
             epoch: Epoch) {
        debug!("compositor received new frame at size {}x{}",
               self.window_size.width.get(),
               self.window_size.height.get());

        // From now on, if we destroy the buffers, they will leak.
        let mut new_layer_buffer_set = new_layer_buffer_set;
        new_layer_buffer_set.mark_will_leak();

        match self.find_layer_with_pipeline_and_layer_id(pipeline_id, layer_id) {
            Some(ref layer) => {
                // FIXME(pcwalton): This is going to cause problems with inconsistent frames since
                // we only composite one layer at a time.
                assert!(layer.add_buffers(new_layer_buffer_set, epoch));
                self.composite_if_necessary();
            }
            None => {
                // FIXME: This may potentially be triggered by a race condition where a
                // buffers are being rendered but the layer is removed before rendering
                // completes.
                panic!("compositor given paint command for non-existent layer");
            }
        }
    }

    fn scroll_fragment_to_point(&mut self,
                                pipeline_id: PipelineId,
                                layer_id: LayerId,
                                point: Point2D<f32>) {
        if self.move_layer(pipeline_id, layer_id, Point2D::from_untyped(&point)) {
            if self.send_buffer_requests_for_all_layers() {
                self.start_scrolling_timer_if_necessary();
            }
        } else {
            self.fragment_point = Some(point);
        }
    }

    fn handle_window_message(&mut self, event: WindowEvent) {
        match event {
            IdleWindowEvent => {}

            RefreshWindowEvent => {
                self.composite_if_necessary()
            }

            ResizeWindowEvent(size) => {
                self.on_resize_window_event(size);
            }

            LoadUrlWindowEvent(url_string) => {
                self.on_load_url_window_event(url_string);
            }

            MouseWindowEventClass(mouse_window_event) => {
                self.on_mouse_window_event_class(mouse_window_event);
            }

            MouseWindowMoveEventClass(cursor) => {
                self.on_mouse_window_move_event_class(cursor);
            }

            ScrollWindowEvent(delta, cursor) => {
                self.on_scroll_window_event(delta, cursor);
            }

            ZoomWindowEvent(magnification) => {
                self.on_zoom_window_event(magnification);
            }

            PinchZoomWindowEvent(magnification) => {
                self.on_pinch_zoom_window_event(magnification);
            }

            NavigationWindowEvent(direction) => {
                self.on_navigation_window_event(direction);
            }

            KeyEvent(key, state, modifiers) => {
                self.on_key_event(key, state, modifiers);
            }

            QuitWindowEvent => {
                debug!("shutting down the constellation for QuitWindowEvent");
                let ConstellationChan(ref chan) = self.constellation_chan;
                chan.send(ExitMsg);
                self.shutdown_state = ShuttingDown;
            }
        }
    }

    fn on_resize_window_event(&mut self, new_size: TypedSize2D<DevicePixel, uint>) {
        // A size change could also mean a resolution change.
        let new_hidpi_factor = self.window.hidpi_factor();
        if self.hidpi_factor != new_hidpi_factor {
            self.hidpi_factor = new_hidpi_factor;
            self.update_zoom_transform();
        }

        if self.window_size == new_size {
            return;
        }

        self.window_size = new_size;

        self.scene.set_root_layer_size(new_size.as_f32());
        self.send_window_size();
    }

    fn on_load_url_window_event(&mut self, url_string: String) {
        debug!("osmain: loading URL `{:s}`", url_string);
        self.got_load_complete_message = false;
        let root_pipeline_id = match self.scene.root {
            Some(ref layer) => layer.extra_data.borrow().pipeline.id.clone(),
            None => panic!("Compositor: Received LoadUrlWindowEvent without initialized compositor \
                           layers"),
        };

        let msg = LoadUrlMsg(root_pipeline_id,
                             LoadData::new(Url::parse(url_string.as_slice()).unwrap()));
        let ConstellationChan(ref chan) = self.constellation_chan;
        chan.send(msg);
    }

    fn on_mouse_window_event_class(&self, mouse_window_event: MouseWindowEvent) {
        let point = match mouse_window_event {
            MouseWindowClickEvent(_, p) => p,
            MouseWindowMouseDownEvent(_, p) => p,
            MouseWindowMouseUpEvent(_, p) => p,
        };
        match self.find_topmost_layer_at_point(point / self.scene.scale) {
            Some(result) => result.layer.send_mouse_event(mouse_window_event, result.point),
            None => {},
        }
    }

    fn on_mouse_window_move_event_class(&self, cursor: TypedPoint2D<DevicePixel, f32>) {
        match self.find_topmost_layer_at_point(cursor / self.scene.scale) {
            Some(result) => result.layer.send_mouse_move_event(result.point),
            None => {},
        }
    }

    fn on_scroll_window_event(&mut self,
                              delta: TypedPoint2D<DevicePixel, f32>,
                              cursor: TypedPoint2D<DevicePixel, i32>) {
        self.pending_scroll_events.push(ScrollEvent {
            delta: delta,
            cursor: cursor,
        });

        self.composite_if_necessary();
    }

    fn process_pending_scroll_events(&mut self) {
        let had_scroll_events = self.pending_scroll_events.len() > 0;
        for scroll_event in mem::replace(&mut self.pending_scroll_events, Vec::new()).into_iter() {
            let delta = scroll_event.delta / self.scene.scale;
            let cursor = scroll_event.cursor.as_f32() / self.scene.scale;

            match self.scene.root {
                Some(ref mut layer) => {
                    layer.handle_scroll_event(delta, cursor);
                }
                None => {}
            }

            self.start_scrolling_timer_if_necessary();
            self.send_buffer_requests_for_all_layers();
        }

        if had_scroll_events {
            self.send_viewport_rects_for_all_layers();
        }
    }

    fn device_pixels_per_screen_px(&self) -> ScaleFactor<ScreenPx, DevicePixel, f32> {
        match opts::get().device_pixels_per_px {
            Some(device_pixels_per_px) => device_pixels_per_px,
            None => match opts::get().output_file {
                Some(_) => ScaleFactor(1.0),
                None => self.hidpi_factor
            }
        }
    }

    fn device_pixels_per_page_px(&self) -> ScaleFactor<PagePx, DevicePixel, f32> {
        self.viewport_zoom * self.page_zoom * self.device_pixels_per_screen_px()
    }

    fn update_zoom_transform(&mut self) {
        let scale = self.device_pixels_per_page_px();
        self.scene.scale = ScaleFactor(scale.get());

        // We need to set the size of the root layer again, since the window size
        // has changed in unscaled layer pixels.
        self.scene.set_root_layer_size(self.window_size.as_f32());
    }

    fn on_zoom_window_event(&mut self, magnification: f32) {
        self.page_zoom = ScaleFactor((self.page_zoom.get() * magnification).max(1.0));
        self.update_zoom_transform();
        self.send_window_size();
    }

    // TODO(pcwalton): I think this should go through the same queuing as scroll events do.
    fn on_pinch_zoom_window_event(&mut self, magnification: f32) {
        self.zoom_action = true;
        self.zoom_time = precise_time_s();
        let old_viewport_zoom = self.viewport_zoom;

        self.viewport_zoom = ScaleFactor((self.viewport_zoom.get() * magnification).max(1.0));
        let viewport_zoom = self.viewport_zoom;

        self.update_zoom_transform();

        // Scroll as needed
        let window_size = self.window_size.as_f32();
        let page_delta: TypedPoint2D<LayerPixel, f32> = TypedPoint2D(
            window_size.width.get() * (viewport_zoom.inv() - old_viewport_zoom.inv()).get() * 0.5,
            window_size.height.get() * (viewport_zoom.inv() - old_viewport_zoom.inv()).get() * 0.5);

        let cursor = TypedPoint2D(-1f32, -1f32);  // Make sure this hits the base layer.
        match self.scene.root {
            Some(ref mut layer) => {
                layer.handle_scroll_event(page_delta, cursor);
            }
            None => { }
        }

        self.send_viewport_rects_for_all_layers();
        self.composite_if_necessary();
    }

    fn on_navigation_window_event(&self, direction: WindowNavigateMsg) {
        let direction = match direction {
            windowing::Forward => constellation_msg::Forward,
            windowing::Back => constellation_msg::Back,
        };
        let ConstellationChan(ref chan) = self.constellation_chan;
        chan.send(NavigateMsg(direction))
    }

    fn on_key_event(&self, key: Key, state: KeyState, modifiers: KeyModifiers) {
        let ConstellationChan(ref chan) = self.constellation_chan;
        chan.send(constellation_msg::KeyEvent(key, state, modifiers))
    }

    fn convert_buffer_requests_to_pipeline_requests_map(&self,
                                                        requests: Vec<(Rc<Layer<CompositorData>>,
                                                                       Vec<BufferRequest>)>) ->
                                                        HashMap<PipelineId, (RenderChan,
                                                                             Vec<RenderRequest>)> {
        let scale = self.device_pixels_per_page_px();
        let mut results:
            HashMap<PipelineId, (RenderChan, Vec<RenderRequest>)> = HashMap::new();

        for (layer, mut layer_requests) in requests.into_iter() {
            let &(_, ref mut vec) =
                match results.entry(layer.extra_data.borrow().pipeline.id) {
                    Occupied(mut entry) => {
                        *entry.get_mut() =
                            (layer.extra_data.borrow().pipeline.render_chan.clone(), vec!());
                        entry.into_mut()
                    }
                    Vacant(entry) => {
                        entry.set((layer.extra_data.borrow().pipeline.render_chan.clone(), vec!()))
                    }
                };

            // All the BufferRequests are in layer/device coordinates, but the render task
            // wants to know the page coordinates. We scale them before sending them.
            for request in layer_requests.iter_mut() {
                request.page_rect = request.page_rect / scale.get();
            }

            vec.push(RenderRequest {
                buffer_requests: layer_requests,
                scale: scale.get(),
                layer_id: layer.extra_data.borrow().id,
                epoch: layer.extra_data.borrow().epoch,
            });
        }

        return results;
    }

    fn send_back_unused_buffers(&mut self) {
        match self.root_pipeline {
            Some(ref pipeline) => {
                let unused_buffers = self.scene.collect_unused_buffers();
                if unused_buffers.len() != 0 {
                    let message = UnusedBufferMsg(unused_buffers);
                    let _ = pipeline.render_chan.send_opt(message);
                }
            },
            None => {}
        }
    }

    fn send_viewport_rect_for_layer(&self, layer: Rc<Layer<CompositorData>>) {
        if layer.extra_data.borrow().id == LayerId::null() {
            let layer_rect = Rect(-layer.extra_data.borrow().scroll_offset.to_untyped(),
                                  layer.bounds.borrow().size.to_untyped());
            let pipeline = &layer.extra_data.borrow().pipeline;
            let ScriptControlChan(ref chan) = pipeline.script_chan;
            chan.send(ViewportMsg(pipeline.id.clone(), layer_rect));
        }

        for kid in layer.children().iter() {
            self.send_viewport_rect_for_layer(kid.clone());
        }
    }

    fn send_viewport_rects_for_all_layers(&self) {
        match self.scene.root {
            Some(ref root) => self.send_viewport_rect_for_layer(root.clone()),
            None => {},
        }
    }

    /// Returns true if any buffer requests were sent or false otherwise.
    fn send_buffer_requests_for_all_layers(&mut self) -> bool {
        let mut layers_and_requests = Vec::new();
        self.scene.get_buffer_requests(&mut layers_and_requests,
                                       Rect(TypedPoint2D(0f32, 0f32), self.window_size.as_f32()));

        // Return unused tiles first, so that they can be reused by any new BufferRequests.
        self.send_back_unused_buffers();

        if layers_and_requests.len() == 0 {
            return false;
        }

        // We want to batch requests for each pipeline to avoid race conditions
        // when handling the resulting BufferRequest responses.
        let pipeline_requests =
            self.convert_buffer_requests_to_pipeline_requests_map(layers_and_requests);

        let mut num_render_msgs_sent = 0;
        for (_pipeline_id, (chan, requests)) in pipeline_requests.into_iter() {
            num_render_msgs_sent += 1;
            let _ = chan.send_opt(RenderMsg(requests));
        }

        self.add_outstanding_render_msg(num_render_msgs_sent);
        true
    }

    fn is_ready_to_render_image_output(&self) -> bool {
        if !self.got_load_complete_message {
            return false;
        }

        if self.get_earliest_pipeline_ready_state() != FinishedLoading {
            return false;
        }

        if self.has_outstanding_render_msgs() {
            return false;
        }

        if !self.all_pipelines_in_idle_render_state() {
            return false;
        }

        if !self.got_set_ids_message {
            return false;
        }

        return true;
    }

    fn composite(&mut self) {
        let output_image = opts::get().output_file.is_some() &&
                            self.is_ready_to_render_image_output();

        let mut framebuffer_ids = vec!();
        let mut texture_ids = vec!();
        let (width, height) = (self.window_size.width.get(), self.window_size.height.get());

        if output_image {
            framebuffer_ids = gl::gen_framebuffers(1);
            gl::bind_framebuffer(gl::FRAMEBUFFER, framebuffer_ids[0]);

            texture_ids = gl::gen_textures(1);
            gl::bind_texture(gl::TEXTURE_2D, texture_ids[0]);

            gl::tex_image_2d(gl::TEXTURE_2D, 0, gl::RGB as GLint, width as GLsizei,
                             height as GLsizei, 0, gl::RGB, gl::UNSIGNED_BYTE, None);
            gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
            gl::tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);

            gl::framebuffer_texture_2d(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D,
                                       texture_ids[0], 0);

            gl::bind_texture(gl::TEXTURE_2D, 0);
        }

        profile(time::CompositingCategory, None, self.time_profiler_chan.clone(), || {
            debug!("compositor: compositing");
            // Adjust the layer dimensions as necessary to correspond to the size of the window.
            self.scene.viewport = Rect {
                origin: Zero::zero(),
                size: self.window_size.as_f32(),
            };
            // Render the scene.
            match self.scene.root {
                Some(ref layer) => {
                    rendergl::render_scene(layer.clone(), self.context, &self.scene);
                }
                None => {}
            }
        });

        if output_image {
            let path =
                from_str::<Path>(opts::get().output_file.as_ref().unwrap().as_slice()).unwrap();
            let mut pixels = gl::read_pixels(0, 0,
                                             width as gl::GLsizei,
                                             height as gl::GLsizei,
                                             gl::RGB, gl::UNSIGNED_BYTE);

            gl::bind_framebuffer(gl::FRAMEBUFFER, 0);

            gl::delete_buffers(texture_ids.as_slice());
            gl::delete_frame_buffers(framebuffer_ids.as_slice());

            // flip image vertically (texture is upside down)
            let orig_pixels = pixels.clone();
            let stride = width * 3;
            for y in range(0, height) {
                let dst_start = y * stride;
                let src_start = (height - y - 1) * stride;
                let src_slice = orig_pixels.slice(src_start, src_start + stride);
                copy_memory(pixels.slice_mut(dst_start, dst_start + stride),
                            src_slice.slice_to(stride));
            }
            let mut img = png::Image {
                width: width as u32,
                height: height as u32,
                pixels: png::RGB8(pixels),
            };
            let res = png::store_png(&mut img, &path);
            assert!(res.is_ok());

            debug!("shutting down the constellation after generating an output file");
            let ConstellationChan(ref chan) = self.constellation_chan;
            chan.send(ExitMsg);
            self.shutdown_state = ShuttingDown;
        }

        // Perform the page flip. This will likely block for a while.
        self.window.present();

        self.last_composite_time = precise_time_ns();

        self.composition_request = NoCompositingNecessary;
        self.process_pending_scroll_events();
    }

    fn composite_if_necessary(&mut self) {
        if self.composition_request == NoCompositingNecessary {
            self.composition_request = CompositeNow
        }
    }

    fn find_topmost_layer_at_point_for_layer(&self,
                                             layer: Rc<Layer<CompositorData>>,
                                             point: TypedPoint2D<LayerPixel, f32>)
                                             -> Option<HitTestResult> {
        let child_point = point - layer.bounds.borrow().origin;
        for child in layer.children().iter().rev() {
            let result = self.find_topmost_layer_at_point_for_layer(child.clone(), child_point);
            if result.is_some() {
                return result;
            }
        }

        let point = point - *layer.content_offset.borrow();
        if !layer.bounds.borrow().contains(&point) {
            return None;
        }

        return Some(HitTestResult { layer: layer, point: point });
    }

    fn find_topmost_layer_at_point(&self,
                                   point: TypedPoint2D<LayerPixel, f32>)
                                   -> Option<HitTestResult> {
        match self.scene.root {
            Some(ref layer) => self.find_topmost_layer_at_point_for_layer(layer.clone(), point),
            None => None,
        }
    }

    fn find_layer_with_pipeline_and_layer_id(&self,
                                             pipeline_id: PipelineId,
                                             layer_id: LayerId)
                                             -> Option<Rc<Layer<CompositorData>>> {
        match self.scene.root {
            Some(ref layer) =>
                find_layer_with_pipeline_and_layer_id_for_layer(layer.clone(),
                                                                pipeline_id,
                                                                layer_id),

            None => None,
        }
    }
}

fn find_layer_with_pipeline_and_layer_id_for_layer(layer: Rc<Layer<CompositorData>>,
                                                   pipeline_id: PipelineId,
                                                   layer_id: LayerId)
                                                   -> Option<Rc<Layer<CompositorData>>> {
    if layer.extra_data.borrow().pipeline.id == pipeline_id &&
       layer.extra_data.borrow().id == layer_id {
        return Some(layer);
    }

    for kid in layer.children().iter() {
        let result = find_layer_with_pipeline_and_layer_id_for_layer(kid.clone(),
                                                                     pipeline_id,
                                                                     layer_id);
        if result.is_some() {
            return result;
        }
    }

    return None;
}

fn create_root_layer_for_pipeline_and_rect(pipeline: &CompositionPipeline,
                                           frame_rect: Option<TypedRect<PagePx, f32>>)
                                           -> Rc<Layer<CompositorData>> {
    let layer_properties = LayerProperties {
        pipeline_id: pipeline.id,
        epoch: Epoch(0),
        id: LayerId::null(),
        rect: Rect::zero(),
        background_color: azure_hl::Color::new(0., 0., 0., 0.),
        scroll_policy: Scrollable,
    };

    let root_layer = CompositorData::new_layer(pipeline.clone(),
                                               layer_properties,
                                               WantsScrollEvents,
                                               opts::get().tile_size);

    match frame_rect {
        Some(ref frame_rect) => {
            *root_layer.masks_to_bounds.borrow_mut() = true;

            let frame_rect = frame_rect.to_untyped();
            *root_layer.bounds.borrow_mut() = Rect::from_untyped(&frame_rect);
        }
        None => {}
    }

    return root_layer;
}

impl<Window> CompositorEventListener for IOCompositor<Window> where Window: WindowMethods {
    fn handle_event(&mut self, msg: WindowEvent) -> bool {
        // Check for new messages coming from the other tasks in the system.
        loop {
            match self.port.try_recv_compositor_msg() {
                None => break,
                Some(msg) => {
                    if !self.handle_browser_message(msg) {
                        break
                    }
                }
            }
        }

        if self.shutdown_state == FinishedShuttingDown {
            // We have exited the compositor and passing window
            // messages to script may crash.
            debug!("Exiting the compositor due to a request from script.");
            return false;
        }

        // Handle the message coming from the windowing system.
        self.handle_window_message(msg);

        // If a pinch-zoom happened recently, ask for tiles at the new resolution
        if self.zoom_action && precise_time_s() - self.zoom_time > 0.3 {
            self.zoom_action = false;
            self.scene.mark_layer_contents_as_changed_recursively();
            self.send_buffer_requests_for_all_layers();
        }

        match self.composition_request {
            NoCompositingNecessary | CompositeOnScrollTimeout(_) => {}
            CompositeNow => self.composite(),
        }

        self.shutdown_state != FinishedShuttingDown
    }

    /// Repaints and recomposites synchronously. You must be careful when calling this, as if a
    /// paint is not scheduled the compositor will hang forever.
    ///
    /// This is used when resizing the window.
    fn repaint_synchronously(&mut self) {
        while self.shutdown_state != ShuttingDown {
            let msg = self.port.recv_compositor_msg();
            let is_paint = match msg {
                Paint(..) => true,
                _ => false,
            };
            let keep_going = self.handle_browser_message(msg);
            if is_paint {
                self.composite();
                break
            }
            if !keep_going {
                break
            }
        }
    }

    fn shutdown(&mut self) {
        // Clear out the compositor layers so that painting tasks can destroy the buffers.
        match self.scene.root {
            None => {}
            Some(ref layer) => layer.forget_all_tiles(),
        }

        // Drain compositor port, sometimes messages contain channels that are blocking
        // another task from finishing (i.e. SetIds)
        while self.port.try_recv_compositor_msg().is_some() {}

        // Tell the profiler, memory profiler, and scrolling timer to shut down.
        let TimeProfilerChan(ref time_profiler_chan) = self.time_profiler_chan;
        time_profiler_chan.send(time::ExitMsg);

        let MemoryProfilerChan(ref memory_profiler_chan) = self.memory_profiler_chan;
        memory_profiler_chan.send(memory::ExitMsg);

        self.scrolling_timer.shutdown();
    }
}
