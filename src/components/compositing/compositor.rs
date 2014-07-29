/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositor_data::{CompositorData, WantsScrollEvents};
use compositor_task::{Msg, CompositorTask, Exit, ChangeReadyState, SetIds, LayerProperties};
use compositor_task::{GetGraphicsMetadata, CreateOrUpdateRootLayer, CreateOrUpdateDescendantLayer};
use compositor_task::{SetLayerOrigin, SetLayerSize, Paint, ScrollFragmentPoint, LoadComplete};
use compositor_task::{ShutdownComplete, ChangeRenderState, ReRenderMsgDiscarded};
use constellation::SendableFrameTree;
use events;
use pipeline::CompositionPipeline;
use platform::{Application, Window};
use windowing;
use windowing::{FinishedWindowEvent, IdleWindowEvent, LoadUrlWindowEvent, MouseWindowClickEvent};
use windowing::{MouseWindowEvent, MouseWindowEventClass, MouseWindowMouseDownEvent};
use windowing::{MouseWindowMouseUpEvent, MouseWindowMoveEventClass, NavigationWindowEvent};
use windowing::{QuitWindowEvent, RefreshWindowEvent, ResizeWindowEvent, ScrollWindowEvent};
use windowing::{WindowEvent, WindowMethods, WindowNavigateMsg, ZoomWindowEvent};
use windowing::PinchZoomWindowEvent;

use azure::azure_hl::SourceSurfaceMethods;
use azure::{azure_hl, AzFloat};
use geom::matrix::identity;
use geom::point::{Point2D, TypedPoint2D};
use geom::rect::Rect;
use geom::size::{Size2D, TypedSize2D};
use geom::scale_factor::ScaleFactor;
use gfx::render_task::ReRenderMsg;
use layers::layers::{Layer, LayerBufferSet};
use layers::rendergl;
use layers::rendergl::RenderContext;
use layers::scene::Scene;
use opengles::gl2;
use png;
use servo_msg::compositor_msg::{Blank, Epoch, FixedPosition, FinishedLoading, IdleRenderState};
use servo_msg::compositor_msg::{LayerId, ReadyState, RenderingRenderState, RenderState};
use servo_msg::constellation_msg::{ConstellationChan, ExitMsg, LoadUrlMsg, NavigateMsg};
use servo_msg::constellation_msg::{PipelineId, ResizedWindowMsg, WindowSizeData};
use servo_msg::constellation_msg;
use servo_util::geometry::{DevicePixel, PagePx, ScreenPx, ViewportPx};
use servo_util::memory::MemoryProfilerChan;
use servo_util::opts::Opts;
use servo_util::time::{profile, TimeProfilerChan};
use servo_util::{memory, time};
use std::io::timer::sleep;
use std::collections::hashmap::HashMap;
use std::path::Path;
use std::rc::Rc;
use std::cmp;
use time::precise_time_s;
use url::Url;


pub struct IOCompositor {
    /// The application window.
    window: Rc<Window>,

    /// The port on which we receive messages.
    port: Receiver<Msg>,

    /// The render context.
    context: RenderContext,

    /// The root pipeline tree.
    pipeline_tree: Option<CompositionPipelineTree>,

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

    /// Tracks whether the renderer has finished its first rendering
    composite_ready: bool,

    /// Tracks whether we are in the process of shutting down, or have shut down and should close
    /// the compositor.
    shutdown_state: ShutdownState,

    /// Tracks whether we need to re-composite a page.
    recomposite: bool,

    /// Tracks outstanding ReRenderMsg's sent to the render tasks.
    outstanding_rerendermsgs: uint,

    /// Tracks whether the zoom action has happend recently.
    zoom_action: bool,

    /// The time of the last zoom action has started.
    zoom_time: f64,

    /// Current display/reflow status of the page
    ready_states: HashMap<PipelineId, ReadyState>,
    render_states: HashMap<PipelineId, RenderState>,

    /// Whether the page being rendered has loaded completely.
    /// Differs from ReadyState because we can finish loading (ready)
    /// many times for a single page.
    load_complete: bool,

    /// The command line option flags.
    opts: Opts,

    /// The channel on which messages can be sent to the constellation.
    constellation_chan: ConstellationChan,

    /// The channel on which messages can be sent to the time profiler.
    time_profiler_chan: TimeProfilerChan,

    /// The channel on which messages can be sent to the memory profiler.
    memory_profiler_chan: MemoryProfilerChan,

    /// Pending scroll to fragment event, if any
    fragment_point: Option<Point2D<f32>>
}

#[deriving(PartialEq)]
enum ShutdownState {
    NotShuttingDown,
    ShuttingDown,
    FinishedShuttingDown,
}


#[deriving(Clone)]
struct CompositionPipelineTree {
    pipeline: CompositionPipeline,
    children: Vec<CompositionPipelineTree>,
}

impl CompositionPipelineTree {
    pub fn new(frame_tree: SendableFrameTree) -> CompositionPipelineTree {
        let mut tree = CompositionPipelineTree {
            pipeline: frame_tree.pipeline,
            children: Vec::new(),
        };
        tree.children.push_all_move(frame_tree.children.iter().map(
                |child| CompositionPipelineTree::new(child.frame_tree.clone())).collect());
        return tree;
    }
}

impl IOCompositor {
    fn new(app: &Application,
               opts: Opts,
               port: Receiver<Msg>,
               constellation_chan: ConstellationChan,
               time_profiler_chan: TimeProfilerChan,
               memory_profiler_chan: MemoryProfilerChan) -> IOCompositor {
        let window: Rc<Window> = WindowMethods::new(app, opts.output_file.is_none());

        // Create an initial layer tree.
        //
        // TODO: There should be no initial layer tree until the renderer creates one from the
        // display list. This is only here because we don't have that logic in the renderer yet.
        let window_size = window.framebuffer_size();
        let hidpi_factor = window.hidpi_factor();

        let show_debug_borders = opts.show_debug_borders;
        IOCompositor {
            window: window,
            port: port,
            opts: opts,
            context: rendergl::RenderContext::new(CompositorTask::create_graphics_context(),
                                                  show_debug_borders),
            pipeline_tree: None,
            scene: Scene::new(window_size.as_f32().to_untyped(), identity()),
            window_size: window_size,
            hidpi_factor: hidpi_factor,
            composite_ready: false,
            shutdown_state: NotShuttingDown,
            recomposite: false,
            page_zoom: ScaleFactor(1.0),
            viewport_zoom: ScaleFactor(1.0),
            zoom_action: false,
            zoom_time: 0f64,
            ready_states: HashMap::new(),
            render_states: HashMap::new(),
            load_complete: false,
            constellation_chan: constellation_chan,
            time_profiler_chan: time_profiler_chan,
            memory_profiler_chan: memory_profiler_chan,
            fragment_point: None,
            outstanding_rerendermsgs: 0,
        }
    }

    pub fn create(app: &Application,
                  opts: Opts,
                  port: Receiver<Msg>,
                  constellation_chan: ConstellationChan,
                  time_profiler_chan: TimeProfilerChan,
                  memory_profiler_chan: MemoryProfilerChan) {
        let mut compositor = IOCompositor::new(app,
                                               opts,
                                               port,
                                               constellation_chan,
                                               time_profiler_chan,
                                               memory_profiler_chan);
        compositor.update_zoom_transform();

        // Starts the compositor, which listens for messages on the specified port.
        compositor.run();
    }

    fn run (&mut self) {
        // Tell the constellation about the initial window size.
        self.send_window_size();

        // Enter the main event loop.
        while self.shutdown_state != FinishedShuttingDown {
            // Check for new messages coming from the rendering task.
            self.handle_message();

            if self.shutdown_state == FinishedShuttingDown {
                // We have exited the compositor and passing window
                // messages to script may crash.
                debug!("Exiting the compositor due to a request from script.");
                break;
            }

            // Check for messages coming from the windowing system.
            let msg = self.window.recv();
            self.handle_window_message(msg);

            // If asked to recomposite and renderer has run at least once
            if self.recomposite && self.composite_ready {
                self.recomposite = false;
                self.composite();
            }

            sleep(10);

            // If a pinch-zoom happened recently, ask for tiles at the new resolution
            if self.zoom_action && precise_time_s() - self.zoom_time > 0.3 {
                self.zoom_action = false;
                self.ask_for_tiles();
            }

        }

        // Clear out the compositor layers so that painting tasks can destroy the buffers.
        match self.scene.root {
            None => {}
            Some(ref layer) => CompositorData::forget_all_tiles(layer.clone()),
        }

        // Drain compositor port, sometimes messages contain channels that are blocking
        // another task from finishing (i.e. SetIds)
        loop {
            match self.port.try_recv() {
                Err(_) => break,
                Ok(_) => {},
            }
        }

        // Tell the profiler and memory profiler to shut down.
        let TimeProfilerChan(ref time_profiler_chan) = self.time_profiler_chan;
        time_profiler_chan.send(time::ExitMsg);

        let MemoryProfilerChan(ref memory_profiler_chan) = self.memory_profiler_chan;
        memory_profiler_chan.send(memory::ExitMsg);
    }

    fn handle_message(&mut self) {
        loop {
            match (self.port.try_recv(), self.shutdown_state) {
                (_, FinishedShuttingDown) =>
                    fail!("compositor shouldn't be handling messages after shutting down"),

                (Err(_), _) => break,

                (Ok(Exit(chan)), _) => {
                    debug!("shutting down the constellation");
                    let ConstellationChan(ref con_chan) = self.constellation_chan;
                    con_chan.send(ExitMsg);
                    chan.send(());
                    self.shutdown_state = ShuttingDown;
                }

                (Ok(ShutdownComplete), _) => {
                    debug!("constellation completed shutdown");
                    self.shutdown_state = FinishedShuttingDown;
                    break;
                }

                (Ok(ChangeReadyState(pipeline_id, ready_state)), NotShuttingDown) => {
                    self.change_ready_state(pipeline_id, ready_state);
                }

                (Ok(ChangeRenderState(pipeline_id, render_state)), NotShuttingDown) => {
                    self.change_render_state(pipeline_id, render_state);
                }

                (Ok(ReRenderMsgDiscarded), NotShuttingDown) => {
                    self.remove_outstanding_rerendermsg();
                }

                (Ok(SetIds(frame_tree, response_chan, new_constellation_chan)), _) => {
                    self.set_ids(frame_tree, response_chan, new_constellation_chan);
                }

                (Ok(GetGraphicsMetadata(chan)), NotShuttingDown) => {
                    chan.send(Some(azure_hl::current_graphics_metadata()));
                }

                (Ok(CreateOrUpdateRootLayer(layer_properties)),
                 NotShuttingDown) => {
                    let scene_root = self.scene.root.clone();
                    self.create_or_update_root_layer(&scene_root, layer_properties);
                }

                (Ok(CreateOrUpdateDescendantLayer(layer_properties)),
                 NotShuttingDown) => {
                    match self.find_parent_pipeline_id(self.get_root_pipeline_tree(),
                                                       layer_properties.pipeline_id) {
                        Some(parent_pipeline_id) =>
                            self.create_or_update_descendant_layer(layer_properties,
                                                                   parent_pipeline_id),
                        None if self.is_root_pipeline(layer_properties.pipeline_id) =>
                            self.create_or_update_descendant_layer(layer_properties,
                                                                   layer_properties.pipeline_id),
                        None =>
                            fail!("didn't find pipeline for parent layer for pipeline id {:?}",
                                  layer_properties.pipeline_id),
                    }
                }

                (Ok(SetLayerOrigin(pipeline_id, layer_id, origin)), NotShuttingDown) => {
                    self.update_layer_clip_rect(pipeline_id,
                                                layer_id,
                                                |rect| -> Rect<f32> { Rect (origin, rect.size) });
                }

                (Ok(SetLayerSize(pipeline_id, layer_id, size)), NotShuttingDown) => {
                    self.update_layer_clip_rect(pipeline_id,
                                                layer_id,
                                                |rect| -> Rect<f32> { Rect (rect.origin, size) });
                }

                (Ok(Paint(pipeline_id, epoch, replies)), NotShuttingDown) => {
                    for (layer_id, new_layer_buffer_set) in replies.move_iter() {
                        self.paint(pipeline_id, layer_id, new_layer_buffer_set, epoch);
                    }
                    self.remove_outstanding_rerendermsg();
                }

                (Ok(ScrollFragmentPoint(pipeline_id, layer_id, point)), NotShuttingDown) => {
                    self.scroll_fragment_to_point(pipeline_id, layer_id, point);
                }

                (Ok(LoadComplete(..)), NotShuttingDown) => {
                    self.load_complete = true;
                }

                // When we are shutting_down, we need to avoid performing operations
                // such as Paint that may crash because we have begun tearing down
                // the rest of our resources.
                (_, ShuttingDown) => { }
            }
        }
    }

    fn change_ready_state(&mut self, pipeline_id: PipelineId, ready_state: ReadyState) {
        self.ready_states.insert_or_update_with(pipeline_id,
                                                ready_state,
                                                |_key, val| *val = ready_state);

        self.window.set_ready_state(self.get_ready_state());
    }

    fn get_ready_state(&self) -> ReadyState {
        let mut ready_state = FinishedLoading;
        for (_, state) in self.ready_states.iter() {
            ready_state = cmp::min(ready_state, *state);
            if ready_state == Blank { break; };
        }
        return ready_state;
    }

    fn change_render_state(&mut self, pipeline_id: PipelineId, render_state: RenderState) {
        self.render_states.insert_or_update_with(pipeline_id,
                                                 render_state,
                                                 |_key, val| *val = render_state);


        let new_render_state = self.get_render_state();
        self.window.set_render_state(new_render_state);
        if self.do_synchronous_wait_for_render() {
            self.composite_ready = new_render_state == IdleRenderState;
        } else {
            self.composite_ready |= new_render_state == IdleRenderState;
        }
    }

    fn get_render_state(&self) -> RenderState {
        let mut render_state = IdleRenderState;
        for (_, state) in self.render_states.iter() {
            render_state = cmp::min(render_state, *state);
            if render_state == RenderingRenderState { break; };
        }
        return render_state;
    }

    fn do_synchronous_wait_for_render(&self) -> bool {
        // only synchronously wait if the compositor outputs to a file.
        self.opts.output_file.is_some()
    }

    fn has_rerendermsg_tracking(&self) -> bool {
        // only track ReRenderMsg's if the compositor outputs to a file.
        self.opts.output_file.is_some()
    }

    fn has_outstanding_rerendermsgs(&self) -> bool {
        self.has_rerendermsg_tracking() && self.outstanding_rerendermsgs > 0
    }

    fn add_outstanding_rerendermsg(&mut self, count: uint) {
        // return early if not tracking ReRenderMsg's
        if !self.has_rerendermsg_tracking() {
            return;
        }
        debug!("add_outstanding_rerendermsg {}", self.outstanding_rerendermsgs);
        self.outstanding_rerendermsgs += count;
    }

    fn remove_outstanding_rerendermsg(&mut self) {
        if !self.has_rerendermsg_tracking() {
            return;
        }
        if self.outstanding_rerendermsgs > 0 {
            self.outstanding_rerendermsgs -= 1;
        } else {
            debug!("too many rerender msgs completed");
        }
    }

    fn set_ids(&mut self,
               frame_tree: SendableFrameTree,
               response_chan: Sender<()>,
               new_constellation_chan: ConstellationChan) {
        response_chan.send(());

        let tree = CompositionPipelineTree::new(frame_tree);
        self.pipeline_tree = Some(tree.clone());
        self.scene.root = Some(self.create_pipeline_tree_root_layers(&tree.clone()));

        // Initialize the new constellation channel by sending it the root window size.
        self.constellation_chan = new_constellation_chan;
        self.send_window_size();
    }

    fn create_pipeline_tree_root_layers(&mut self,
                                        pipeline_tree: &CompositionPipelineTree)
                                        -> Rc<Layer<CompositorData>> {
        let layer_properties = LayerProperties {
            pipeline_id: pipeline_tree.pipeline.id,
            epoch: Epoch(0),
            id: LayerId::null(),
            rect: Rect(Point2D(0f32,
                               0f32),
                       Size2D(0f32,
                              0f32)),
            background_color: azure_hl::Color::new(0 as AzFloat,
                                                   0 as AzFloat,
                                                   0 as AzFloat,
                                                   0 as AzFloat),
            scroll_policy: FixedPosition,
        };
        let root_layer = CompositorData::new_layer(pipeline_tree.pipeline.clone(),
                                                   layer_properties,
                                                   WantsScrollEvents,
                                                   self.opts.tile_size);
        self.ready_states.insert(pipeline_tree.pipeline.id, Blank);
        self.render_states.insert(pipeline_tree.pipeline.id, RenderingRenderState);
        for child_pipeline in pipeline_tree.children.iter() {
            root_layer.add_child(self.create_pipeline_tree_root_layers(child_pipeline));
        }
        return root_layer;
    }

    fn update_layer_if_exists(&mut self, properties: LayerProperties) -> bool {
        match self.scene.root {
            Some(ref root_layer) => {
                match CompositorData::find_layer_with_pipeline_and_layer_id(root_layer.clone(),
                                                                            properties.pipeline_id,
                                                                            properties.id) {
                    Some(existing_layer) => {
                        CompositorData::update_layer(existing_layer.clone(),
                                                     self.page_window(),
                                                     properties);
                        true
                    }
                    None => false,
               }
            }
            None => false,
        }
    }

    fn create_or_update_root_layer(&mut self,
                                   root: &Option<Rc<Layer<CompositorData>>>,
                                   layer_properties: LayerProperties) {

        let need_new_root_layer = !self.update_layer_if_exists(layer_properties);
        let page_window = self.page_window();
        if need_new_root_layer {
            if self.is_root_pipeline(layer_properties.pipeline_id) {
                let scene_root = match self.scene.root {
                    // Release all tiles from the layer before dropping it.
                    Some(ref mut layer) if layer.extra_data.borrow().id == LayerId::null() => {
                        CompositorData::clear_all_tiles(layer.clone());
                        layer
                    },
                    // root layer is already initialized
                    Some(_) => fail!("scene root layer already initialized"),
                    None => fail!("No scene root layer"),
                };
                CompositorData::update_layer(scene_root.clone(), page_window, layer_properties);
            } else {
                let root_layer =
                    match self.find_pipeline_root_layer(root, layer_properties.pipeline_id) {
                    // Release all tiles from the layer before dropping it.
                    Some(ref mut layer) => {
                        CompositorData::clear_all_tiles(layer.clone());
                        layer.clone()
                    },
                    None => fail!("No pipeline root layer"),
                };
                CompositorData::update_layer(root_layer.clone(), page_window, layer_properties);
             }
        }
        self.scroll_layer_to_fragment_point_if_necessary(layer_properties.pipeline_id,
                                                         layer_properties.id);
        self.ask_for_tiles();
    }

    fn create_or_update_descendant_layer(&mut self,
                                         layer_properties: LayerProperties,
                                         parent_layer_pipeline_id: PipelineId) {

        if !self.update_layer_if_exists(layer_properties) {
            self.create_descendant_layer(layer_properties, parent_layer_pipeline_id);
        }
        self.scroll_layer_to_fragment_point_if_necessary(layer_properties.pipeline_id,
                                                         layer_properties.id);
        self.ask_for_tiles();
    }

    fn create_descendant_layer(&self,
                               layer_properties: LayerProperties,
                               parent_layer_pipeline_id: PipelineId) {

        match self.scene.root {
            Some(ref root_layer) => {
                let x = Some(root_layer.clone());
                let parent_layer = match self.find_pipeline_root_layer(&x,
                                                                       parent_layer_pipeline_id) {
                    Some(l) => l,
                    None => fail!("couldn't find parent layer"),
                };
                CompositorData::add_child(parent_layer.clone(), layer_properties);
            }
            None => fail!("Compositor: Received new layer without initialized pipeline")
        }
    }

    fn is_root_pipeline(&mut self, pipeline_id: PipelineId) -> bool {
        match self.pipeline_tree {
            Some(ref root_pipeline) => root_pipeline.pipeline.id == pipeline_id,
            None => fail!("Compositor: Uninitialized pipeline tree"),
        }
    }

    fn get_root_pipeline_tree<'a>(&'a self) -> Option<&'a CompositionPipelineTree> {
        match self.pipeline_tree {
            Some(ref tree) => Some(tree),
            None => None,
        }
    }

    fn find_parent_pipeline_id(&self,
                               tree: Option<&CompositionPipelineTree>,
                               child_pipeline_id: PipelineId)
                               -> Option<PipelineId> {
        match tree {
            Some(ref pipeline_tree) => {
                for child_pipeline in pipeline_tree.children.iter() {
                    if child_pipeline.pipeline.id == child_pipeline_id {
                        return Some(pipeline_tree.pipeline.id);
                    }
                }
                for child_tree in pipeline_tree.children.iter() {
                    match self.find_parent_pipeline_id(Some(child_tree),
                                                       child_pipeline_id) {
                        Some(parent_pipeline_id) => return Some(parent_pipeline_id),
                        None => {},
                    }
                }
                None
            },
            None => None,
        }
    }

    fn find_pipeline_root_layer(&self,
                                root: &Option<Rc<Layer<CompositorData>>>,
                                pipeline_id: PipelineId)
                                -> Option<Rc<Layer<CompositorData>>> {
        let root_layer = match *root {
            Some(ref layer) => layer.clone(),
            None => fail!("no layer"),
        };
        if root_layer.extra_data.borrow().pipeline.id == pipeline_id {
            return Some(root_layer);
        } else {
            for child in root_layer.children().iter() {
                match self.find_pipeline_root_layer(&Some(child.clone()), pipeline_id) {
                    Some(l) => return Some(l),
                    None => { },
                }
            }
            return None;
        }
    }

    /// The size of the content area in CSS px at the current zoom level
    fn page_window(&self) -> TypedSize2D<PagePx, f32> {
        self.window_size.as_f32() / self.device_pixels_per_page_px()
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

    fn scroll_layer_to_fragment_point_if_necessary(&mut self,
                                                   pipeline_id: PipelineId,
                                                   layer_id: LayerId) {
        let page_window = self.page_window();
        let needs_recomposite = match self.scene.root {
            Some(ref mut root_layer) => {
                self.fragment_point.take().map_or(false, |fragment_point| {
                    events::move(root_layer.clone(),
                                 pipeline_id,
                                 layer_id,
                                 fragment_point,
                                 page_window)
                })
            }
            None => fail!("Compositor: Tried to scroll to fragment without root layer."),
        };

        self.recomposite_if(needs_recomposite);
    }

    fn update_layer_clip_rect(&mut self,
                              pipeline_id: PipelineId,
                              layer_id: LayerId,
                              f: |rect: Rect<f32>| ->Rect<f32>) {
        let root_layer = match self.scene.root {
            Some(ref root) => Some(root.clone()),
            None => None,
        };
        let layer_id = match (layer_id == LayerId::null(),
                              self.find_pipeline_root_layer(&root_layer, pipeline_id)) {
            (false, _)  => layer_id,
            (true, Some(l)) => l.extra_data.borrow().id,
            (true, None) => fail!("No root layer for pipeline")};

        let should_ask_for_tiles = match self.scene.root {
            Some(ref root_layer) => {
                match CompositorData::find_layer_with_pipeline_and_layer_id(root_layer.clone(),
                                                                            pipeline_id,
                                                                            layer_id) {
                    Some(ref layer) => {
                        let new_rect = { f(*layer.bounds.borrow()) };
                        *layer.bounds.borrow_mut() = new_rect;
                        true
                    }
                    None => {
                        fail!("compositor received SetLayerClipRect for nonexistent layer");
                    }
                }
            }
            None => false
        };

        if should_ask_for_tiles {
            self.ask_for_tiles();
        }
    }

    fn paint(&mut self,
             pipeline_id: PipelineId,
             layer_id: LayerId,
             new_layer_buffer_set: Box<LayerBufferSet>,
             epoch: Epoch) {
        debug!("compositor received new frame");

        // From now on, if we destroy the buffers, they will leak.
        let mut new_layer_buffer_set = new_layer_buffer_set;
        new_layer_buffer_set.mark_will_leak();

        match self.scene.root {
            Some(ref root_layer) => {
                match CompositorData::find_layer_with_pipeline_and_layer_id(root_layer.clone(),
                                                                            pipeline_id,
                                                                            layer_id) {
                    Some(ref layer) => {
                        assert!(CompositorData::add_buffers(layer.clone(),
                                                            new_layer_buffer_set,
                                                            epoch));
                        self.recomposite = true;
                    }
                    None => {
                        // FIXME: This may potentially be triggered by a race condition where a
                        // buffers are being rendered but the layer is removed before rendering
                        // completes.
                        fail!("compositor given paint command for non-existent layer");
                    }
                }
            }
            None => {
                fail!("compositor given paint command with no root layer initialized");
            }
        }

        // TODO: Recycle the old buffers; send them back to the renderer to reuse if
        // it wishes.
    }

    fn scroll_fragment_to_point(&mut self,
                                pipeline_id: PipelineId,
                                layer_id: LayerId,
                                point: Point2D<f32>) {
        let page_window = self.page_window();
        let (ask, move): (bool, bool) = match self.scene.root {
            Some(ref layer) if layer.extra_data.borrow().pipeline.id == pipeline_id => {
                (true,
                 events::move(layer.clone(), pipeline_id, layer_id, point, page_window))
            }
            Some(_) | None => {
                self.fragment_point = Some(point);

                (false, false)
            }
        };

        if ask {
            self.recomposite_if(move);
            self.ask_for_tiles();
        }
    }

    fn handle_window_message(&mut self, event: WindowEvent) {
        match event {
            IdleWindowEvent => {}

            RefreshWindowEvent => {
                self.recomposite = true;
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

            FinishedWindowEvent => {
                let exit = self.opts.exit_after_load;
                if exit {
                    debug!("shutting down the constellation for FinishedWindowEvent");
                    let ConstellationChan(ref chan) = self.constellation_chan;
                    chan.send(ExitMsg);
                    self.shutdown_state = ShuttingDown;
                }
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
        if self.window_size != new_size {
            debug!("osmain: window resized to {:?}", new_size);
            self.window_size = new_size;
            self.send_window_size();
        } else {
            debug!("osmain: dropping window resize since size is still {:?}", new_size);
        }
    }

    fn on_load_url_window_event(&mut self, url_string: String) {
        debug!("osmain: loading URL `{:s}`", url_string);
        self.load_complete = false;
        let root_pipeline_id = match self.scene.root {
            Some(ref layer) => layer.extra_data.borrow().pipeline.id.clone(),
            None => fail!("Compositor: Received LoadUrlWindowEvent without initialized compositor \
                           layers"),
        };

        let msg = LoadUrlMsg(root_pipeline_id, Url::parse(url_string.as_slice()).unwrap());
        let ConstellationChan(ref chan) = self.constellation_chan;
        chan.send(msg);
    }

    fn on_mouse_window_event_class(&self, mouse_window_event: MouseWindowEvent) {
        let scale = self.device_pixels_per_page_px();
        let point = match mouse_window_event {
            MouseWindowClickEvent(_, p) => p / scale,
            MouseWindowMouseDownEvent(_, p) => p / scale,
            MouseWindowMouseUpEvent(_, p) => p / scale,
        };
        for layer in self.scene.root.iter() {
            events::send_mouse_event(layer.clone(), mouse_window_event, point);
        }
    }

    fn on_mouse_window_move_event_class(&self, cursor: TypedPoint2D<DevicePixel, f32>) {
        let scale = self.device_pixels_per_page_px();
        for layer in self.scene.root.iter() {
            events::send_mouse_move_event(layer.clone(), cursor / scale);
        }
    }

    fn on_scroll_window_event(&mut self,
                              delta: TypedPoint2D<DevicePixel, f32>,
                              cursor: TypedPoint2D<DevicePixel, i32>) {
        let scale = self.device_pixels_per_page_px();
        // TODO: modify delta to snap scroll to pixels.
        let page_delta = delta / scale;
        let page_cursor = cursor.as_f32() / scale;
        let page_window = self.page_window();
        let mut scroll = false;
        match self.scene.root {
            Some(ref mut layer) => {
                scroll = events::handle_scroll_event(layer.clone(),
                                                     page_delta,
                                                     page_cursor,
                                                     page_window) || scroll;
            }
            None => { }
        }
        self.recomposite_if(scroll);
        self.ask_for_tiles();
    }

    fn device_pixels_per_screen_px(&self) -> ScaleFactor<ScreenPx, DevicePixel, f32> {
        match self.opts.device_pixels_per_px {
            Some(device_pixels_per_px) => device_pixels_per_px,
            None => match self.opts.output_file {
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
        self.scene.transform = identity().scale(scale.get(), scale.get(), 1f32);
    }

    fn on_zoom_window_event(&mut self, magnification: f32) {
        self.page_zoom = ScaleFactor((self.page_zoom.get() * magnification).max(1.0));
        self.update_zoom_transform();
        self.send_window_size();
    }

    fn on_pinch_zoom_window_event(&mut self, magnification: f32) {
        self.zoom_action = true;
        self.zoom_time = precise_time_s();
        let old_viewport_zoom = self.viewport_zoom;
        let window_size = self.window_size.as_f32();

        self.viewport_zoom = ScaleFactor((self.viewport_zoom.get() * magnification).max(1.0));
        let viewport_zoom = self.viewport_zoom;

        self.update_zoom_transform();

        // Scroll as needed
        let page_delta = TypedPoint2D(
            window_size.width.get() * (viewport_zoom.inv() - old_viewport_zoom.inv()).get() * 0.5,
            window_size.height.get() * (viewport_zoom.inv() - old_viewport_zoom.inv()).get() * 0.5);
        // TODO: modify delta to snap scroll to pixels.
        let page_cursor = TypedPoint2D(-1f32, -1f32); // Make sure this hits the base layer
        let page_window = self.page_window();

        match self.scene.root {
            Some(ref mut layer) => {
                events::handle_scroll_event(layer.clone(),
                                            page_delta,
                                            page_cursor,
                                            page_window);
            }
            None => { }
        }

        self.recomposite = true;
    }

    fn on_navigation_window_event(&self, direction: WindowNavigateMsg) {
        let direction = match direction {
            windowing::Forward => constellation_msg::Forward,
            windowing::Back => constellation_msg::Back,
        };
        let ConstellationChan(ref chan) = self.constellation_chan;
        chan.send(NavigateMsg(direction))
    }

    /// Get BufferRequests from each layer.
    fn ask_for_tiles(&mut self) {
        let scale = self.device_pixels_per_page_px();
        let page_window = self.page_window();
        let mut num_rerendermsgs_sent = 0;
        match self.scene.root {
            Some(ref layer) => {
                let rect = Rect(Point2D(0f32, 0f32), page_window.to_untyped());
                let mut request_map = HashMap::new();
                let recomposite =
                    CompositorData::get_buffer_requests_recursively(&mut request_map,
                                                                    layer.clone(),
                                                                    rect,
                                                                    scale.get());
                for (_pipeline_id, (chan, requests)) in request_map.move_iter() {
                    num_rerendermsgs_sent += 1;
                    let _ = chan.send_opt(ReRenderMsg(requests));
                }
                self.recomposite = self.recomposite || recomposite;
            }
            None => { }
        }
        self.add_outstanding_rerendermsg(num_rerendermsgs_sent);
    }

    fn composite(&mut self) {
        profile(time::CompositingCategory, self.time_profiler_chan.clone(), || {
            debug!("compositor: compositing");
            // Adjust the layer dimensions as necessary to correspond to the size of the window.
            self.scene.size = self.window_size.as_f32().to_untyped();
            // Render the scene.
            match self.scene.root {
                Some(ref layer) => {
                    self.scene.background_color.r = layer.extra_data.borrow().background_color.r;
                    self.scene.background_color.g = layer.extra_data.borrow().background_color.g;
                    self.scene.background_color.b = layer.extra_data.borrow().background_color.b;
                    self.scene.background_color.a = layer.extra_data.borrow().background_color.a;
                    rendergl::render_scene(layer.clone(), self.context, &self.scene);
                }
                None => {}
            }
        });

        // Render to PNG. We must read from the back buffer (ie, before
        // self.window.present()) as OpenGL ES 2 does not have glReadBuffer().
        if self.load_complete && self.get_ready_state() == FinishedLoading
            && self.opts.output_file.is_some() && !self.has_outstanding_rerendermsgs() {
            let (width, height) = (self.window_size.width.get(), self.window_size.height.get());
            let path = from_str::<Path>(self.opts.output_file.get_ref().as_slice()).unwrap();
            let mut pixels = gl2::read_pixels(0, 0,
                                              width as gl2::GLsizei,
                                              height as gl2::GLsizei,
                                              gl2::RGB, gl2::UNSIGNED_BYTE);
            // flip image vertically (texture is upside down)
            let orig_pixels = pixels.clone();
            let stride = width * 3;
            for y in range(0, height) {
                let dst_start = y * stride;
                let src_start = (height - y - 1) * stride;
                unsafe {
                    let src_slice = orig_pixels.slice(src_start, src_start + stride);
                    pixels.mut_slice(dst_start, dst_start + stride)
                          .copy_memory(src_slice.slice_to(stride));
                }
            }
            let img = png::Image {
                width: width as u32,
                height: height as u32,
                color_type: png::RGB8,
                pixels: pixels,
            };
            let res = png::store_png(&img, &path);
            assert!(res.is_ok());

            debug!("shutting down the constellation after generating an output file");
            let ConstellationChan(ref chan) = self.constellation_chan;
            chan.send(ExitMsg);
            self.shutdown_state = ShuttingDown;
        }

        self.window.present();

        let exit = self.opts.exit_after_load;
        if exit {
            debug!("shutting down the constellation for exit_after_load");
            let ConstellationChan(ref chan) = self.constellation_chan;
            chan.send(ExitMsg);
        }
    }

    fn recomposite_if(&mut self, result: bool) {
        self.recomposite = result || self.recomposite;
    }
}
