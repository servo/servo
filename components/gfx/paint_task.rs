/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The task that handles all painting.

use display_list::{self, StackingContext};
use font_cache_task::FontCacheTask;
use font_context::FontContext;
use paint_context::PaintContext;

use azure::AzFloat;
use azure::azure_hl::{SurfaceFormat, Color, DrawTarget, BackendType};
use canvas_traits::CanvasMsg;
use euclid::Matrix4;
use euclid::point::Point2D;
use euclid::rect::Rect;
use euclid::size::Size2D;
use ipc_channel::ipc::IpcSender;
use layers::layers::{BufferRequest, LayerBuffer, LayerBufferSet};
use layers::platform::surface::{NativeDisplay, NativeSurface};
use msg::compositor_msg::{Epoch, FrameTreeId, LayerId, LayerKind};
use msg::compositor_msg::{LayerProperties, PaintListener, ScrollPolicy};
use msg::constellation_msg::Msg as ConstellationMsg;
use msg::constellation_msg::PipelineExitType;
use msg::constellation_msg::{ConstellationChan, Failure, PipelineId};
use profile_traits::mem::{self, ReportsChan};
use profile_traits::time::{self, profile};
use rand::{self, Rng};
use skia::gl_context::GLContext;
use smallvec::SmallVec;
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::mem as std_mem;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Select, Sender, channel};
use url::Url;
use util::geometry::{Au, ZERO_POINT};
use util::opts;
use util::task::spawn_named;
use util::task::spawn_named_with_send_on_failure;
use util::task_state;

/// Information about a hardware graphics layer that layout sends to the painting task.
#[derive(Clone, Deserialize, Serialize)]
pub struct PaintLayer {
    /// A per-pipeline ID describing this layer that should be stable across reflows.
    pub id: LayerId,
    /// The color of the background in this layer. Used for unpainted content.
    pub background_color: Color,
    /// The scrolling policy of this layer.
    pub scroll_policy: ScrollPolicy,
}

impl PaintLayer {
    /// Creates a new `PaintLayer`.
    pub fn new(id: LayerId, background_color: Color, scroll_policy: ScrollPolicy) -> PaintLayer {
        PaintLayer {
            id: id,
            background_color: background_color,
            scroll_policy: scroll_policy,
        }
    }
}

pub struct PaintRequest {
    pub buffer_requests: Vec<BufferRequest>,
    pub scale: f32,
    pub layer_id: LayerId,
    pub epoch: Epoch,
    pub layer_kind: LayerKind,
}

pub enum Msg {
    FromLayout(LayoutToPaintMsg),
    FromChrome(ChromeToPaintMsg),
}

#[derive(Deserialize, Serialize)]
pub enum LayoutToPaintMsg {
    PaintInit(Epoch, Arc<StackingContext>),
    CanvasLayer(LayerId, IpcSender<CanvasMsg>),
    Exit(Option<IpcSender<()>>, PipelineExitType),
}

pub enum ChromeToPaintMsg {
    Paint(Vec<PaintRequest>, FrameTreeId),
    PaintPermissionGranted,
    PaintPermissionRevoked,
    CollectReports(ReportsChan),
    Exit(Option<IpcSender<()>>, PipelineExitType),
}

pub struct PaintTask<C> {
    id: PipelineId,
    _url: Url,
    layout_to_paint_port: Receiver<LayoutToPaintMsg>,
    chrome_to_paint_port: Receiver<ChromeToPaintMsg>,
    compositor: C,
    constellation_chan: ConstellationChan,

    /// A channel to the time profiler.
    time_profiler_chan: time::ProfilerChan,

    /// The root stacking context sent to us by the layout thread.
    root_stacking_context: Option<Arc<StackingContext>>,

    /// Permission to send paint messages to the compositor
    paint_permission: bool,

    /// The current epoch counter is passed by the layout task
    current_epoch: Option<Epoch>,

    /// Communication handles to each of the worker threads.
    worker_threads: Vec<WorkerThreadProxy>,

    /// A map to track the canvas specific layers
    canvas_map: HashMap<LayerId, IpcSender<CanvasMsg>>,
}

// If we implement this as a function, we get borrowck errors from borrowing
// the whole PaintTask struct.
macro_rules! native_display(
    ($task:expr) => (
        $task.native_display.as_ref().expect("Need a graphics context to do painting")
    )
);

impl<C> PaintTask<C> where C: PaintListener + Send + 'static {
    pub fn create(id: PipelineId,
                  url: Url,
                  chrome_to_paint_chan: Sender<ChromeToPaintMsg>,
                  layout_to_paint_port: Receiver<LayoutToPaintMsg>,
                  chrome_to_paint_port: Receiver<ChromeToPaintMsg>,
                  compositor: C,
                  constellation_chan: ConstellationChan,
                  font_cache_task: FontCacheTask,
                  failure_msg: Failure,
                  time_profiler_chan: time::ProfilerChan,
                  mem_profiler_chan: mem::ProfilerChan,
                  shutdown_chan: Sender<()>) {
        let ConstellationChan(c) = constellation_chan.clone();
        spawn_named_with_send_on_failure(format!("PaintTask {:?}", id), task_state::PAINT, move || {
            {
                // Ensures that the paint task and graphics context are destroyed before the
                // shutdown message.
                let mut compositor = compositor;
                let native_display = compositor.native_display().map(
                    |display| display);
                let worker_threads = WorkerThreadProxy::spawn(native_display.clone(),
                                                              font_cache_task,
                                                              time_profiler_chan.clone());

                // FIXME: rust/#5967
                let mut paint_task = PaintTask {
                    id: id,
                    _url: url,
                    layout_to_paint_port: layout_to_paint_port,
                    chrome_to_paint_port: chrome_to_paint_port,
                    compositor: compositor,
                    constellation_chan: constellation_chan,
                    time_profiler_chan: time_profiler_chan,
                    root_stacking_context: None,
                    paint_permission: false,
                    current_epoch: None,
                    worker_threads: worker_threads,
                    canvas_map: HashMap::new()
                };

                let reporter_name = format!("paint-reporter-{}", id.0);
                mem_profiler_chan.run_with_memory_reporting(|| {
                    paint_task.start();
                }, reporter_name, chrome_to_paint_chan, ChromeToPaintMsg::CollectReports);

                // Tell all the worker threads to shut down.
                for worker_thread in &mut paint_task.worker_threads {
                    worker_thread.exit()
                }
            }

            debug!("paint_task: shutdown_chan send");
            shutdown_chan.send(()).unwrap();
        }, ConstellationMsg::Failure(failure_msg), c);
    }

    fn start(&mut self) {
        debug!("PaintTask: beginning painting loop");

        loop {
            let message = {
                let select = Select::new();
                let mut layout_to_paint_handle = select.handle(&self.layout_to_paint_port);
                let mut chrome_to_paint_handle = select.handle(&self.chrome_to_paint_port);
                unsafe {
                    layout_to_paint_handle.add();
                    chrome_to_paint_handle.add();
                }
                let result = select.wait();
                if result == layout_to_paint_handle.id() {
                    Msg::FromLayout(self.layout_to_paint_port.recv().unwrap())
                } else if result == chrome_to_paint_handle.id() {
                    Msg::FromChrome(self.chrome_to_paint_port.recv().unwrap())
                } else {
                    panic!("unexpected select result")
                }
            };

            match message {
                Msg::FromLayout(LayoutToPaintMsg::PaintInit(epoch, stacking_context)) => {
                    self.current_epoch = Some(epoch);
                    self.root_stacking_context = Some(stacking_context.clone());

                    if !self.paint_permission {
                        debug!("PaintTask: paint ready msg");
                        let ConstellationChan(ref mut c) = self.constellation_chan;
                        c.send(ConstellationMsg::PainterReady(self.id)).unwrap();
                        continue;
                    }

                    self.initialize_layers();
                }
                // Inserts a new canvas renderer to the layer map
                Msg::FromLayout(LayoutToPaintMsg::CanvasLayer(layer_id, canvas_renderer)) => {
                    debug!("Renderer received for canvas with layer {:?}", layer_id);
                    self.canvas_map.insert(layer_id, canvas_renderer);
                }
                Msg::FromChrome(ChromeToPaintMsg::Paint(requests, frame_tree_id)) => {
                    if !self.paint_permission {
                        debug!("PaintTask: paint ready msg");
                        let ConstellationChan(ref mut c) = self.constellation_chan;
                        c.send(ConstellationMsg::PainterReady(self.id)).unwrap();
                        continue;
                    }

                    let mut replies = Vec::new();
                    for PaintRequest { buffer_requests, scale, layer_id, epoch, layer_kind }
                          in requests {
                        if self.current_epoch == Some(epoch) {
                            self.paint(&mut replies, buffer_requests, scale, layer_id, layer_kind);
                        } else {
                            debug!("PaintTask: Ignoring requests with epoch mismatch: {:?} != {:?}",
                                   self.current_epoch,
                                   epoch);
                            self.compositor.ignore_buffer_requests(buffer_requests);
                        }
                    }

                    debug!("PaintTask: returning surfaces");
                    self.compositor.assign_painted_buffers(self.id,
                                                           self.current_epoch.unwrap(),
                                                           replies,
                                                           frame_tree_id);
                }
                Msg::FromChrome(ChromeToPaintMsg::PaintPermissionGranted) => {
                    self.paint_permission = true;

                    if self.root_stacking_context.is_some() {
                        self.initialize_layers();
                    }
                }
                Msg::FromChrome(ChromeToPaintMsg::PaintPermissionRevoked) => {
                    self.paint_permission = false;
                }
                Msg::FromChrome(ChromeToPaintMsg::CollectReports(ref channel)) => {
                    // FIXME(njn): should eventually measure the paint task.
                    channel.send(Vec::new())
                }
                Msg::FromLayout(LayoutToPaintMsg::Exit(ref response_channel, _)) |
                Msg::FromChrome(ChromeToPaintMsg::Exit(ref response_channel, _)) => {
                    // Ask the compositor to remove any layers it is holding for this paint task.
                    // FIXME(mrobinson): This can probably move back to the constellation now.
                    self.compositor.notify_paint_task_exiting(self.id);

                    debug!("PaintTask: Exiting.");
                    response_channel.as_ref().map(|channel| channel.send(()));
                    break;
                }
            }
        }
    }

    /// Paints one layer and places the painted tiles in `replies`.
    fn paint(&mut self,
              replies: &mut Vec<(LayerId, Box<LayerBufferSet>)>,
              mut tiles: Vec<BufferRequest>,
              scale: f32,
              layer_id: LayerId,
              layer_kind: LayerKind) {
        time::profile(time::ProfilerCategory::Painting, None, self.time_profiler_chan.clone(), || {
            // Bail out if there is no appropriate stacking context.
            let stacking_context = if let Some(ref stacking_context) = self.root_stacking_context {
                match display_list::find_stacking_context_with_layer_id(stacking_context,
                                                                        layer_id) {
                    Some(stacking_context) => stacking_context,
                    None => return,
                }
            } else {
                return
            };

            // Divide up the layer into tiles and distribute them to workers via a simple round-
            // robin strategy.
            let tiles = std_mem::replace(&mut tiles, Vec::new());
            let tile_count = tiles.len();
            for (i, tile) in tiles.into_iter().enumerate() {
                let thread_id = i % self.worker_threads.len();
                self.worker_threads[thread_id].paint_tile(thread_id,
                                                          tile,
                                                          stacking_context.clone(),
                                                          scale,
                                                          layer_kind);
            }
            let new_buffers = (0..tile_count).map(|i| {
                let thread_id = i % self.worker_threads.len();
                self.worker_threads[thread_id].get_painted_tile_buffer()
            }).collect();

            let layer_buffer_set = box LayerBufferSet {
                buffers: new_buffers,
            };
            replies.push((layer_id, layer_buffer_set));
        })
    }

    fn initialize_layers(&mut self) {
        let root_stacking_context = match self.root_stacking_context {
            None => return,
            Some(ref root_stacking_context) => root_stacking_context,
        };

        let mut properties = Vec::new();
        build(&mut properties,
              &**root_stacking_context,
              &ZERO_POINT,
              &Matrix4::identity(),
              &Matrix4::identity(),
              None);
        self.compositor.initialize_layers_for_pipeline(self.id,
                                                       properties,
                                                       self.current_epoch.unwrap());

        fn build(properties: &mut Vec<LayerProperties>,
                 stacking_context: &StackingContext,
                 page_position: &Point2D<Au>,
                 transform: &Matrix4,
                 perspective: &Matrix4,
                 parent_id: Option<LayerId>) {
            let transform = transform.mul(&stacking_context.transform);
            let perspective = perspective.mul(&stacking_context.perspective);

            let (next_parent_id, page_position, transform, perspective) =
                match stacking_context.layer {
                Some(ref paint_layer) => {
                    let overflow_size =
                        Size2D::new(stacking_context.overflow.size.width.to_nearest_px() as f32,
                                    stacking_context.overflow.size.height.to_nearest_px() as f32);
                    let establishes_3d_context = stacking_context.establishes_3d_context;
                    let scrolls_overflow_area = stacking_context.scrolls_overflow_area;

                    // Layers start at the top left of their overflow rect, as far as the info
                    // we give to the compositor is concerned.
                    let overflow_relative_page_position = *page_position +
                                                          stacking_context.bounds.origin +
                                                          stacking_context.overflow.origin;
                    let layer_position = Rect::new(
                        Point2D::new(overflow_relative_page_position.x.to_nearest_px() as f32,
                                     overflow_relative_page_position.y.to_nearest_px() as f32),
                        overflow_size);

                    properties.push(LayerProperties {
                        id: paint_layer.id,
                        parent_id: parent_id,
                        rect: layer_position,
                        background_color: paint_layer.background_color,
                        scroll_policy: paint_layer.scroll_policy,
                        transform: transform,
                        perspective: perspective,
                        establishes_3d_context: establishes_3d_context,
                        scrolls_overflow_area: scrolls_overflow_area,
                    });

                    // When there is a new layer, the transforms and origin
                    // are handled by the compositor.
                    (Some(paint_layer.id),
                     -stacking_context.overflow.origin,
                     Matrix4::identity(),
                     Matrix4::identity())
                }
                None => {
                    (parent_id,
                     stacking_context.bounds.origin + *page_position,
                     transform,
                     perspective)
                }
            };

            // Sort positioned children according to z-index.
            let mut positioned_children: SmallVec<[Arc<StackingContext>; 8]> = SmallVec::new();
            for kid in &stacking_context.display_list.children {
                positioned_children.push((*kid).clone());
            }
            positioned_children.sort_by(|this, other| this.z_index.cmp(&other.z_index));

            for kid in positioned_children.iter() {
                build(properties, &**kid, &page_position, &transform, &perspective, next_parent_id)
            }
        }
    }
}

struct WorkerThreadProxy {
    sender: Sender<MsgToWorkerThread>,
    receiver: Receiver<MsgFromWorkerThread>,
}

impl WorkerThreadProxy {
    fn spawn(native_display: Option<NativeDisplay>,
             font_cache_task: FontCacheTask,
             time_profiler_chan: time::ProfilerChan)
             -> Vec<WorkerThreadProxy> {
        let thread_count = if opts::get().gpu_painting {
            1
        } else {
            opts::get().paint_threads
        };
        (0..thread_count).map(|_| {
            let (from_worker_sender, from_worker_receiver) = channel();
            let (to_worker_sender, to_worker_receiver) = channel();
            let font_cache_task = font_cache_task.clone();
            let time_profiler_chan = time_profiler_chan.clone();
            spawn_named("PaintWorker".to_owned(), move || {
                let mut worker_thread = WorkerThread::new(from_worker_sender,
                                                          to_worker_receiver,
                                                          native_display,
                                                          font_cache_task,
                                                          time_profiler_chan);
                worker_thread.main();
            });
            WorkerThreadProxy {
                receiver: from_worker_receiver,
                sender: to_worker_sender,
            }
        }).collect()
    }

    fn paint_tile(&mut self,
                  thread_id: usize,
                  tile: BufferRequest,
                  stacking_context: Arc<StackingContext>,
                  scale: f32,
                  layer_kind: LayerKind) {
        let msg = MsgToWorkerThread::PaintTile(thread_id,
                                               tile,
                                               stacking_context,
                                               scale,
                                               layer_kind);
        self.sender.send(msg).unwrap()
    }

    fn get_painted_tile_buffer(&mut self) -> Box<LayerBuffer> {
        match self.receiver.recv().unwrap() {
            MsgFromWorkerThread::PaintedTile(layer_buffer) => layer_buffer,
        }
    }

    fn exit(&mut self) {
        self.sender.send(MsgToWorkerThread::Exit).unwrap()
    }
}

struct WorkerThread {
    sender: Sender<MsgFromWorkerThread>,
    receiver: Receiver<MsgToWorkerThread>,
    native_display: Option<NativeDisplay>,
    font_context: Box<FontContext>,
    time_profiler_sender: time::ProfilerChan,
    gl_context: Option<Arc<GLContext>>,
}

fn create_gl_context(native_display: Option<NativeDisplay>) -> Option<Arc<GLContext>> {
    if !opts::get().gpu_painting {
        return None;
    }

    match native_display {
        Some(display) => {
            let tile_size = opts::get().tile_size as i32;
            GLContext::new(display.platform_display_data(), Size2D::new(tile_size, tile_size))
        }
        None => {
            warn!("Could not create GLContext, falling back to CPU rasterization");
            None
        }
    }
}

impl WorkerThread {
    fn new(sender: Sender<MsgFromWorkerThread>,
           receiver: Receiver<MsgToWorkerThread>,
           native_display: Option<NativeDisplay>,
           font_cache_task: FontCacheTask,
           time_profiler_sender: time::ProfilerChan)
           -> WorkerThread {
        let gl_context = create_gl_context(native_display);
        WorkerThread {
            sender: sender,
            receiver: receiver,
            native_display: native_display,
            font_context: box FontContext::new(font_cache_task.clone()),
            time_profiler_sender: time_profiler_sender,
            gl_context: gl_context,
        }
    }

    fn main(&mut self) {
        loop {
            match self.receiver.recv().unwrap() {
                MsgToWorkerThread::Exit => break,
                MsgToWorkerThread::PaintTile(thread_id, tile, stacking_context, scale, layer_kind) => {
                    let buffer = self.optimize_and_paint_tile(thread_id,
                                                              tile,
                                                              stacking_context,
                                                              scale,
                                                              layer_kind);
                    self.sender.send(MsgFromWorkerThread::PaintedTile(buffer)).unwrap()
                }
            }
        }
    }

    fn create_draw_target_for_layer_buffer(&self,
                                           size: Size2D<i32>,
                                           layer_buffer: &mut Box<LayerBuffer>)
                                           -> DrawTarget {
        match self.gl_context {
            Some(ref gl_context) => {
                match layer_buffer.native_surface.gl_rasterization_context(gl_context.clone()) {
                    Some(rasterization_context) => {
                        DrawTarget::new_with_gl_rasterization_context(rasterization_context,
                                                                      SurfaceFormat::B8G8R8A8)
                    }
                    None => panic!("Could not create GLRasterizationContext for LayerBuffer"),
                }
            },
            None => {
                // A missing GLContext means we want CPU rasterization.
                DrawTarget::new(BackendType::Skia, size, SurfaceFormat::B8G8R8A8)
            }
        }
   }

    fn optimize_and_paint_tile(&mut self,
                               thread_id: usize,
                               mut tile: BufferRequest,
                               stacking_context: Arc<StackingContext>,
                               scale: f32,
                               layer_kind: LayerKind)
                               -> Box<LayerBuffer> {
        let size = Size2D::new(tile.screen_rect.size.width as i32,
                               tile.screen_rect.size.height as i32);
        let mut buffer = self.create_layer_buffer(&mut tile, scale);
        let draw_target = self.create_draw_target_for_layer_buffer(size, &mut buffer);

        {
            // Build the paint context.
            let mut paint_context = PaintContext {
                draw_target: draw_target.clone(),
                font_context: &mut self.font_context,
                page_rect: tile.page_rect,
                screen_rect: tile.screen_rect,
                clip_rect: None,
                transient_clip: None,
                layer_kind: layer_kind,
            };

            // Apply a translation to start at the boundaries of the stacking context, since the
            // layer's origin starts at its overflow rect's origin.
            let tile_bounds = tile.page_rect.translate(
                &Point2D::new(stacking_context.overflow.origin.x.to_f32_px(),
                              stacking_context.overflow.origin.y.to_f32_px()));

            // Apply the translation to paint the tile we want.
            let matrix = Matrix4::identity();
            let matrix = matrix.scale(scale as AzFloat, scale as AzFloat, 1.0);
            let matrix = matrix.translate(-tile_bounds.origin.x as AzFloat,
                                          -tile_bounds.origin.y as AzFloat,
                                          0.0);

            // Clear the buffer.
            paint_context.clear();

            // Draw the display list.
            time::profile(time::ProfilerCategory::PaintingPerTile,
                          None,
                          self.time_profiler_sender.clone(),
                          || {
                stacking_context.optimize_and_draw_into_context(&mut paint_context,
                                                                &matrix,
                                                                None);
                paint_context.draw_target.flush();
                    });

            if opts::get().show_debug_parallel_paint {
                // Overlay a transparent solid color to identify the thread that
                // painted this tile.
                let color = THREAD_TINT_COLORS[thread_id % THREAD_TINT_COLORS.len()];
                paint_context.draw_solid_color(&Rect::new(Point2D::new(Au(0), Au(0)),
                                                          Size2D::new(Au::from_px(size.width),
                                                                      Au::from_px(size.height))),
                                               color);
            }
            if opts::get().paint_flashing {
                // Overlay a random transparent color.
                let color = *rand::thread_rng().choose(&THREAD_TINT_COLORS[..]).unwrap();
                paint_context.draw_solid_color(&Rect::new(Point2D::new(Au(0), Au(0)),
                                                          Size2D::new(Au::from_px(size.width),
                                                                      Au::from_px(size.height))),
                                               color);
            }
        }

        // Extract the texture from the draw target and place it into its slot in the buffer. If
        // using CPU painting, upload it first.
        if self.gl_context.is_none() {
            draw_target.snapshot().get_data_surface().with_data(|data| {
                buffer.native_surface.upload(native_display!(self), data);
                debug!("painting worker thread uploading to native surface {}",
                       buffer.native_surface.get_id());
            });
        }

        draw_target.finish();
        buffer
    }

    fn create_layer_buffer(&mut self,
                           tile: &mut BufferRequest,
                           scale: f32)
                           -> Box<LayerBuffer> {
        // Create an empty native surface. We mark it as not leaking
        // in case it dies in transit to the compositor task.
        let width = tile.screen_rect.size.width;
        let height = tile.screen_rect.size.height;
        let mut native_surface = tile.native_surface.take().unwrap_or_else(|| {
            NativeSurface::new(native_display!(self), Size2D::new(width as i32, height as i32))
        });
        native_surface.mark_wont_leak();

        box LayerBuffer {
            native_surface: native_surface,
            rect: tile.page_rect,
            screen_pos: tile.screen_rect,
            resolution: scale,
            painted_with_cpu: self.gl_context.is_none(),
            content_age: tile.content_age,
        }
    }
}

enum MsgToWorkerThread {
    Exit,
    PaintTile(usize, BufferRequest, Arc<StackingContext>, f32, LayerKind),
}

enum MsgFromWorkerThread {
    PaintedTile(Box<LayerBuffer>),
}

pub static THREAD_TINT_COLORS: [Color; 8] = [
    Color { r: 6.0 / 255.0, g: 153.0 / 255.0, b: 198.0 / 255.0, a: 0.7 },
    Color { r: 255.0 / 255.0, g: 212.0 / 255.0, b: 83.0 / 255.0, a: 0.7 },
    Color { r: 116.0 / 255.0, g: 29.0 / 255.0, b: 109.0 / 255.0, a: 0.7 },
    Color { r: 204.0 / 255.0, g: 158.0 / 255.0, b: 199.0 / 255.0, a: 0.7 },
    Color { r: 242.0 / 255.0, g: 46.0 / 255.0, b: 121.0 / 255.0, a: 0.7 },
    Color { r: 116.0 / 255.0, g: 203.0 / 255.0, b: 196.0 / 255.0, a: 0.7 },
    Color { r: 255.0 / 255.0, g: 249.0 / 255.0, b: 201.0 / 255.0, a: 0.7 },
    Color { r: 137.0 / 255.0, g: 196.0 / 255.0, b: 78.0 / 255.0, a: 0.7 },
];
