/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The task that handles all rendering/painting.

use buffer_map::BufferMap;
use display_list::optimizer::DisplayListOptimizer;
use display_list::DisplayList;
use font_context::FontContext;
use render_context::RenderContext;

use azure::azure_hl::{B8G8R8A8, Color, DrawTarget, SkiaBackend, StolenGLResources};
use azure::AzFloat;
use geom::matrix2d::Matrix2D;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use layers::platform::surface::{NativeGraphicsMetadata, NativePaintingGraphicsContext};
use layers::platform::surface::{NativeSurface, NativeSurfaceMethods};
use layers::layers::{BufferRequest, LayerBuffer, LayerBufferSet};
use layers;
use native::task::NativeTaskBuilder;
use servo_msg::compositor_msg::{Epoch, IdleRenderState, LayerId};
use servo_msg::compositor_msg::{LayerMetadata, RenderListener, RenderingRenderState, ScrollPolicy};
use servo_msg::constellation_msg::{ConstellationChan, Failure, FailureMsg, PipelineId};
use servo_msg::constellation_msg::{RendererReadyMsg};
use servo_msg::platform::surface::NativeSurfaceAzureMethods;
use servo_util::geometry;
use servo_util::opts;
use servo_util::smallvec::{SmallVec, SmallVec1};
use servo_util::task::spawn_named_with_send_on_failure;
use servo_util::time::{TimeProfilerChan, profile};
use servo_util::time;
use std::comm::{Receiver, Sender, channel};
use std::fmt;
use std::mem;
use std::task::TaskBuilder;
use sync::Arc;
use font_cache_task::FontCacheTask;

/// Information about a layer that layout sends to the painting task.
#[deriving(Clone)]
pub struct RenderLayer {
    /// A per-pipeline ID describing this layer that should be stable across reflows.
    pub id: LayerId,
    /// The display list describing the contents of this layer.
    pub display_list: Arc<DisplayList>,
    /// The position of the layer in pixels.
    pub position: Rect<uint>,
    /// The color of the background in this layer. Used for unrendered content.
    pub background_color: Color,
    /// The scrolling policy of this layer.
    pub scroll_policy: ScrollPolicy,
}

impl fmt::Show for RenderLayer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} @ {} [{}]", self.id, self.position, self.scroll_policy)
    }
}

pub struct RenderRequest {
    pub buffer_requests: Vec<BufferRequest>,
    pub scale: f32,
    pub layer_id: LayerId,
    pub epoch: Epoch,
}

pub enum Msg {
    RenderInitMsg(SmallVec1<RenderLayer>),
    RenderMsg(Vec<RenderRequest>),
    UnusedBufferMsg(Vec<Box<LayerBuffer>>),
    PaintPermissionGranted,
    PaintPermissionRevoked,
    ExitMsg(Option<Sender<()>>),
}

#[deriving(Clone)]
pub struct RenderChan(Sender<Msg>);

impl RenderChan {
    pub fn new() -> (Receiver<Msg>, RenderChan) {
        let (chan, port) = channel();
        (port, RenderChan(chan))
    }

    pub fn send(&self, msg: Msg) {
        let &RenderChan(ref chan) = self;
        assert!(chan.send_opt(msg).is_ok(), "RenderChan.send: render port closed")
    }

    pub fn send_opt(&self, msg: Msg) -> Result<(), Msg> {
        let &RenderChan(ref chan) = self;
        chan.send_opt(msg)
    }
}

pub struct RenderTask<C> {
    id: PipelineId,
    port: Receiver<Msg>,
    compositor: C,
    constellation_chan: ConstellationChan,

    /// A channel to the time profiler.
    time_profiler_chan: TimeProfilerChan,

    /// The native graphics context.
    native_graphics_context: Option<NativePaintingGraphicsContext>,

    /// The layers to be rendered.
    render_layers: SmallVec1<RenderLayer>,

    /// Permission to send paint messages to the compositor
    paint_permission: bool,

    /// A counter for epoch messages
    epoch: Epoch,

    /// A data structure to store unused LayerBuffers
    buffer_map: BufferMap,

    /// Communication handles to each of the worker threads.
    worker_threads: Vec<WorkerThreadProxy>,
}

// If we implement this as a function, we get borrowck errors from borrowing
// the whole RenderTask struct.
macro_rules! native_graphics_context(
    ($task:expr) => (
        $task.native_graphics_context.as_ref().expect("Need a graphics context to do rendering")
    )
)

fn initialize_layers<C>(compositor: &mut C,
                        pipeline_id: PipelineId,
                        epoch: Epoch,
                        render_layers: &[RenderLayer])
                        where C: RenderListener {
    let metadata = render_layers.iter().map(|render_layer| {
        LayerMetadata {
            id: render_layer.id,
            position: render_layer.position,
            background_color: render_layer.background_color,
            scroll_policy: render_layer.scroll_policy,
        }
    }).collect();
    compositor.initialize_layers_for_pipeline(pipeline_id, metadata, epoch);
}

impl<C> RenderTask<C> where C: RenderListener + Send {
    pub fn create(id: PipelineId,
                  port: Receiver<Msg>,
                  compositor: C,
                  constellation_chan: ConstellationChan,
                  font_cache_task: FontCacheTask,
                  failure_msg: Failure,
                  time_profiler_chan: TimeProfilerChan,
                  shutdown_chan: Sender<()>) {
        let ConstellationChan(c) = constellation_chan.clone();
        spawn_named_with_send_on_failure("RenderTask", proc() {
            { // Ensures RenderTask and graphics context are destroyed before shutdown msg
                let native_graphics_context = compositor.get_graphics_metadata().map(
                    |md| NativePaintingGraphicsContext::from_metadata(&md));
                let worker_threads = WorkerThreadProxy::spawn(compositor.get_graphics_metadata(),
                                                              font_cache_task,
                                                              time_profiler_chan.clone());

                // FIXME: rust/#5967
                let mut render_task = RenderTask {
                    id: id,
                    port: port,
                    compositor: compositor,
                    constellation_chan: constellation_chan,
                    time_profiler_chan: time_profiler_chan,

                    native_graphics_context: native_graphics_context,

                    render_layers: SmallVec1::new(),

                    paint_permission: false,
                    epoch: Epoch(0),
                    buffer_map: BufferMap::new(10000000),
                    worker_threads: worker_threads,
                };

                render_task.start();

                // Destroy all the buffers.
                match render_task.native_graphics_context.as_ref() {
                    Some(ctx) => render_task.buffer_map.clear(ctx),
                    None => (),
                }

                // Tell all the worker threads to shut down.
                for worker_thread in render_task.worker_threads.iter_mut() {
                    worker_thread.exit()
                }
            }

            debug!("render_task: shutdown_chan send");
            shutdown_chan.send(());
        }, FailureMsg(failure_msg), c, true);
    }

    fn start(&mut self) {
        debug!("render_task: beginning rendering loop");

        loop {
            match self.port.recv() {
                RenderInitMsg(render_layers) => {
                    self.epoch.next();
                    self.render_layers = render_layers;

                    if !self.paint_permission {
                        debug!("render_task: render ready msg");
                        let ConstellationChan(ref mut c) = self.constellation_chan;
                        c.send(RendererReadyMsg(self.id));
                        continue;
                    }

                    initialize_layers(&mut self.compositor,
                                      self.id,
                                      self.epoch,
                                      self.render_layers.as_slice());
                }
                RenderMsg(requests) => {
                    if !self.paint_permission {
                        debug!("render_task: render ready msg");
                        let ConstellationChan(ref mut c) = self.constellation_chan;
                        c.send(RendererReadyMsg(self.id));
                        self.compositor.render_msg_discarded();
                        continue;
                    }

                    let mut replies = Vec::new();
                    self.compositor.set_render_state(self.id, RenderingRenderState);
                    for RenderRequest { buffer_requests, scale, layer_id, epoch }
                          in requests.into_iter() {
                        if self.epoch == epoch {
                            self.render(&mut replies, buffer_requests, scale, layer_id);
                        } else {
                            debug!("renderer epoch mismatch: {:?} != {:?}", self.epoch, epoch);
                        }
                    }

                    self.compositor.set_render_state(self.id, IdleRenderState);

                    debug!("render_task: returning surfaces");
                    self.compositor.paint(self.id, self.epoch, replies);
                }
                UnusedBufferMsg(unused_buffers) => {
                    for buffer in unused_buffers.into_iter().rev() {
                        self.buffer_map.insert(native_graphics_context!(self), buffer);
                    }
                }
                PaintPermissionGranted => {
                    self.paint_permission = true;

                    // Here we assume that the main layer—the layer responsible for the page size—
                    // is the first layer. This is a pretty fragile assumption. It will be fixed
                    // once we use the layers-based scrolling infrastructure for all scrolling.
                    if self.render_layers.len() > 1 {
                        self.epoch.next();
                        initialize_layers(&mut self.compositor,
                                          self.id,
                                          self.epoch,
                                          self.render_layers.as_slice());
                    }
                }
                PaintPermissionRevoked => {
                    self.paint_permission = false;
                }
                ExitMsg(response_ch) => {
                    debug!("render_task: exitmsg response send");
                    response_ch.map(|ch| ch.send(()));
                    break;
                }
            }
        }
    }

    /// Retrieves an appropriately-sized layer buffer from the cache to match the requirements of
    /// the given tile, or creates one if a suitable one cannot be found.
    fn find_or_create_layer_buffer_for_tile(&mut self, tile: &BufferRequest, scale: f32)
                                            -> Option<Box<LayerBuffer>> {
        let width = tile.screen_rect.size.width;
        let height = tile.screen_rect.size.height;
        if opts::get().gpu_painting {
            return None
        }

        match self.buffer_map.find(tile.screen_rect.size) {
            Some(mut buffer) => {
                buffer.rect = tile.page_rect;
                buffer.screen_pos = tile.screen_rect;
                buffer.resolution = scale;
                buffer.native_surface.mark_wont_leak();
                buffer.painted_with_cpu = true;
                buffer.content_age = tile.content_age;
                return Some(buffer)
            }
            None => {}
        }

        // Create an empty native surface. We mark it as not leaking
        // in case it dies in transit to the compositor task.
        let mut native_surface: NativeSurface =
            layers::platform::surface::NativeSurfaceMethods::new(native_graphics_context!(self),
                                                                 Size2D(width as i32,
                                                                        height as i32),
                                                                 width as i32 * 4);
        native_surface.mark_wont_leak();

        Some(box LayerBuffer {
            native_surface: native_surface,
            rect: tile.page_rect,
            screen_pos: tile.screen_rect,
            resolution: scale,
            stride: (width * 4) as uint,
            painted_with_cpu: true,
            content_age: tile.content_age,
        })
    }

    /// Renders one layer and sends the tiles back to the layer.
    fn render(&mut self,
              replies: &mut Vec<(LayerId, Box<LayerBufferSet>)>,
              mut tiles: Vec<BufferRequest>,
              scale: f32,
              layer_id: LayerId) {
        time::profile(time::PaintingCategory, None, self.time_profiler_chan.clone(), || {
            // Bail out if there is no appropriate render layer.
            let render_layer = match self.render_layers.iter().find(|layer| layer.id == layer_id) {
                Some(render_layer) => (*render_layer).clone(),
                None => return,
            };

            // Divide up the layer into tiles and distribute them to workers via a simple round-
            // robin strategy.
            let tiles = mem::replace(&mut tiles, Vec::new());
            let tile_count = tiles.len();
            for (i, tile) in tiles.into_iter().enumerate() {
                let thread_id = i % self.worker_threads.len();
                let layer_buffer = self.find_or_create_layer_buffer_for_tile(&tile, scale);
                self.worker_threads.get_mut(thread_id).paint_tile(tile,
                                                                  layer_buffer,
                                                                  render_layer.clone(),
                                                                  scale);
            }
            let new_buffers = Vec::from_fn(tile_count, |i| {
                let thread_id = i % self.worker_threads.len();
                self.worker_threads.get_mut(thread_id).get_painted_tile_buffer()
            });

            let layer_buffer_set = box LayerBufferSet {
                buffers: new_buffers,
            };
            replies.push((layer_id, layer_buffer_set));
        })
    }
}

struct WorkerThreadProxy {
    sender: Sender<MsgToWorkerThread>,
    receiver: Receiver<MsgFromWorkerThread>,
}

impl WorkerThreadProxy {
    fn spawn(native_graphics_metadata: Option<NativeGraphicsMetadata>,
             font_cache_task: FontCacheTask,
             time_profiler_chan: TimeProfilerChan)
             -> Vec<WorkerThreadProxy> {
        let thread_count = if opts::get().gpu_painting {
            1
        } else {
            opts::get().layout_threads
        };
        Vec::from_fn(thread_count, |_| {
            let (from_worker_sender, from_worker_receiver) = channel();
            let (to_worker_sender, to_worker_receiver) = channel();
            let native_graphics_metadata = native_graphics_metadata.clone();
            let font_cache_task = font_cache_task.clone();
            let time_profiler_chan = time_profiler_chan.clone();
            TaskBuilder::new().native().spawn(proc() {
                let mut worker_thread = WorkerThread::new(from_worker_sender,
                                                          to_worker_receiver,
                                                          native_graphics_metadata,
                                                          font_cache_task,
                                                          time_profiler_chan);
                worker_thread.main();
            });
            WorkerThreadProxy {
                receiver: from_worker_receiver,
                sender: to_worker_sender,
            }
        })
    }

    fn paint_tile(&mut self,
                  tile: BufferRequest,
                  layer_buffer: Option<Box<LayerBuffer>>,
                  render_layer: RenderLayer,
                  scale: f32) {
        self.sender.send(PaintTileMsgToWorkerThread(tile, layer_buffer, render_layer, scale))
    }

    fn get_painted_tile_buffer(&mut self) -> Box<LayerBuffer> {
        match self.receiver.recv() {
            PaintedTileMsgFromWorkerThread(layer_buffer) => layer_buffer,
        }
    }

    fn exit(&mut self) {
        self.sender.send(ExitMsgToWorkerThread)
    }
}

struct WorkerThread {
    sender: Sender<MsgFromWorkerThread>,
    receiver: Receiver<MsgToWorkerThread>,
    native_graphics_context: Option<NativePaintingGraphicsContext>,
    font_context: Box<FontContext>,
    time_profiler_sender: TimeProfilerChan,
}

impl WorkerThread {
    fn new(sender: Sender<MsgFromWorkerThread>,
           receiver: Receiver<MsgToWorkerThread>,
           native_graphics_metadata: Option<NativeGraphicsMetadata>,
           font_cache_task: FontCacheTask,
           time_profiler_sender: TimeProfilerChan)
           -> WorkerThread {
        WorkerThread {
            sender: sender,
            receiver: receiver,
            native_graphics_context: native_graphics_metadata.map(|metadata| {
                NativePaintingGraphicsContext::from_metadata(&metadata)
            }),
            font_context: box FontContext::new(font_cache_task.clone()),
            time_profiler_sender: time_profiler_sender,
        }
    }

    fn main(&mut self) {
        loop {
            match self.receiver.recv() {
                ExitMsgToWorkerThread => break,
                PaintTileMsgToWorkerThread(tile, layer_buffer, render_layer, scale) => {
                    let draw_target = self.optimize_and_paint_tile(&tile, render_layer, scale);
                    let buffer = self.create_layer_buffer_for_painted_tile(&tile,
                                                                           layer_buffer,
                                                                           draw_target,
                                                                           scale);
                    self.sender.send(PaintedTileMsgFromWorkerThread(buffer))
                }
            }
        }
    }

    fn optimize_and_paint_tile(&mut self,
                               tile: &BufferRequest,
                               render_layer: RenderLayer,
                               scale: f32)
                               -> DrawTarget {
        // page_rect is in coordinates relative to the layer origin, but all display list
        // components are relative to the page origin. We make page_rect relative to
        // the page origin before passing it to the optimizer.
        let page_rect = tile.page_rect.translate(&Point2D(render_layer.position.origin.x as f32,
                                                          render_layer.position.origin.y as f32));
        let page_rect_au = geometry::f32_rect_to_au_rect(page_rect);

        // Optimize the display list for this tile.
        let optimizer = DisplayListOptimizer::new(render_layer.display_list.clone(),
                                                  page_rect_au);
        let display_list = optimizer.optimize();

        let size = Size2D(tile.screen_rect.size.width as i32, tile.screen_rect.size.height as i32);
        let draw_target = if !opts::get().gpu_painting {
            DrawTarget::new(SkiaBackend, size, B8G8R8A8)
        } else {
            // FIXME(pcwalton): Cache the components of draw targets (texture color buffer,
            // renderbuffers) instead of recreating them.
            let draw_target = DrawTarget::new_with_fbo(SkiaBackend,
                                                       native_graphics_context!(self),
                                                       size,
                                                       B8G8R8A8);
            draw_target.make_current();
            draw_target
        };

        {
            // Build the render context.
            let mut render_context = RenderContext {
                draw_target: &draw_target,
                font_ctx: &mut self.font_context,
                page_rect: tile.page_rect,
                screen_rect: tile.screen_rect,
            };

            // Apply the translation to render the tile we want.
            let matrix: Matrix2D<AzFloat> = Matrix2D::identity();
            let matrix = matrix.scale(scale as AzFloat, scale as AzFloat);
            let matrix = matrix.translate(-page_rect.origin.x as AzFloat,
                                          -page_rect.origin.y as AzFloat);

            render_context.draw_target.set_transform(&matrix);

            // Clear the buffer.
            render_context.clear();

            // Draw the display list.
            profile(time::PaintingPerTileCategory, None, self.time_profiler_sender.clone(), || {
                let mut clip_stack = Vec::new();
                display_list.draw_into_context(&mut render_context, &matrix, &mut clip_stack);
                render_context.draw_target.flush();
            });
        }

        draw_target
    }

    fn create_layer_buffer_for_painted_tile(&mut self,
                                            tile: &BufferRequest,
                                            layer_buffer: Option<Box<LayerBuffer>>,
                                            draw_target: DrawTarget,
                                            scale: f32)
                                            -> Box<LayerBuffer> {
        // Extract the texture from the draw target and place it into its slot in the buffer. If
        // using CPU rendering, upload it first.
        //
        // FIXME(pcwalton): We should supply the texture and native surface *to* the draw target in
        // GPU rendering mode, so that it doesn't have to recreate it.
        if !opts::get().gpu_painting {
            let mut buffer = layer_buffer.unwrap();
            draw_target.snapshot().get_data_surface().with_data(|data| {
                buffer.native_surface.upload(native_graphics_context!(self), data);
                debug!("painting worker thread uploading to native surface {:d}",
                       buffer.native_surface.get_id() as int);
            });
            return buffer
        }

        // GPU painting path:
        draw_target.make_current();
        let StolenGLResources {
            surface: native_surface
        } = draw_target.steal_gl_resources().unwrap();

        // We mark the native surface as not leaking in case the surfaces
        // die on their way to the compositor task.
        let mut native_surface: NativeSurface =
            NativeSurfaceAzureMethods::from_azure_surface(native_surface);
        native_surface.mark_wont_leak();

        box LayerBuffer {
            native_surface: native_surface,
            rect: tile.page_rect,
            screen_pos: tile.screen_rect,
            resolution: scale,
            stride: (tile.screen_rect.size.width * 4) as uint,
            painted_with_cpu: false,
            content_age: tile.content_age,
        }
    }
}

enum MsgToWorkerThread {
    ExitMsgToWorkerThread,
    PaintTileMsgToWorkerThread(BufferRequest, Option<Box<LayerBuffer>>, RenderLayer, f32),
}

enum MsgFromWorkerThread {
    PaintedTileMsgFromWorkerThread(Box<LayerBuffer>),
}
