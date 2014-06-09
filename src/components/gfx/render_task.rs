/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The task that handles all rendering/painting.

use buffer_map::BufferMap;
use display_list::optimizer::DisplayListOptimizer;
use display_list::DisplayList;
use font_context::{FontContext, FontContextInfo};
use render_context::RenderContext;

use azure::azure_hl::{B8G8R8A8, Color, DrawTarget, StolenGLResources};
use azure::AzFloat;
use geom::matrix2d::Matrix2D;
use geom::rect::Rect;
use geom::size::Size2D;
use layers::platform::surface::{NativePaintingGraphicsContext, NativeSurface};
use layers::platform::surface::{NativeSurfaceMethods};
use layers;
use servo_msg::compositor_msg::{Epoch, IdleRenderState, LayerBuffer, LayerBufferSet, LayerId};
use servo_msg::compositor_msg::{LayerMetadata, RenderListener, RenderingRenderState, ScrollPolicy};
use servo_msg::constellation_msg::{ConstellationChan, Failure, FailureMsg, PipelineId};
use servo_msg::constellation_msg::{RendererReadyMsg};
use servo_msg::platform::surface::NativeSurfaceAzureMethods;
use servo_util::geometry;
use servo_util::opts::Opts;
use servo_util::smallvec::{SmallVec, SmallVec1};
use servo_util::task::send_on_failure;
use servo_util::time::{ProfilerChan, profile};
use servo_util::time;
use std::comm::{Receiver, Sender, channel};
use std::task::TaskBuilder;
use sync::Arc;

/// Information about a layer that layout sends to the painting task.
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

pub enum Msg {
    RenderMsg(SmallVec1<RenderLayer>),
    ReRenderMsg(Vec<BufferRequest>, f32, LayerId, Epoch),
    UnusedBufferMsg(Vec<Box<LayerBuffer>>),
    PaintPermissionGranted,
    PaintPermissionRevoked,
    ExitMsg(Option<Sender<()>>),
}

/// A request from the compositor to the renderer for tiles that need to be (re)displayed.
#[deriving(Clone)]
pub struct BufferRequest {
    // The rect in pixels that will be drawn to the screen
    screen_rect: Rect<uint>,

    // The rect in page coordinates that this tile represents
    page_rect: Rect<f32>,
}

pub fn BufferRequest(screen_rect: Rect<uint>, page_rect: Rect<f32>) -> BufferRequest {
    BufferRequest {
        screen_rect: screen_rect,
        page_rect: page_rect,
    }
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

/// If we're using GPU rendering, this provides the metadata needed to create a GL context that
/// is compatible with that of the main thread.
pub enum GraphicsContext {
    CpuGraphicsContext,
    GpuGraphicsContext,
}

pub struct RenderTask<C> {
    id: PipelineId,
    port: Receiver<Msg>,
    compositor: C,
    constellation_chan: ConstellationChan,
    opts: Opts,

    /// A channel to the profiler.
    profiler_chan: ProfilerChan,

    /// The graphics context to use.
    graphics_context: GraphicsContext,

    /// The layers to be rendered.
    render_layers: SmallVec1<RenderLayer>,

    /// Permission to send paint messages to the compositor
    paint_permission: bool,

    /// A counter for epoch messages
    epoch: Epoch,

    /// Renderer workers
    worker_txs: Vec<Sender<WorkerMsg>>,

    /// The receiver on which we receive rendered buffers from the workers
    worker_result_rx: Receiver<Box<LayerBuffer>>
}

// If we implement this as a function, we get borrowck errors from borrowing
// the whole RenderTask struct.
macro_rules! native_graphics_context(
    ($task:expr) => (
        $task.native_graphics_context.as_ref().expect("Need a graphics context to do rendering")
    )
)

enum WorkerMsg {
    // This is tupled so all the data can be pulled out of the message as one variable
    WorkerRender((BufferRequest, Arc<DisplayList>, uint, uint, f32)),
    WorkerUnusedBuffer(Box<LayerBuffer>),
    WorkerExit(Sender<()>)
}

fn initialize_layers<C:RenderListener>(
                     compositor: &mut C,
                     pipeline_id: PipelineId,
                     epoch: Epoch,
                     render_layers: &[RenderLayer]) {
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

impl<C:RenderListener + Send> RenderTask<C> {
    pub fn create(id: PipelineId,
                  port: Receiver<Msg>,
                  compositor: C,
                  constellation_chan: ConstellationChan,
                  failure_msg: Failure,
                  opts: Opts,
                  profiler_chan: ProfilerChan,
                  shutdown_chan: Sender<()>) {
        let mut builder = TaskBuilder::new().named("RenderTask");
        let ConstellationChan(c) = constellation_chan.clone();
        send_on_failure(&mut builder, FailureMsg(failure_msg), c);
        builder.spawn(proc() {

            {
                let cpu_painting = opts.cpu_painting;

                let (worker_result_tx, worker_result_rx) = channel();

                // FIXME: rust/#5967
                let mut render_task = RenderTask {
                    id: id,
                    port: port,
                    compositor: compositor,
                    constellation_chan: constellation_chan,
                    opts: opts,
                    profiler_chan: profiler_chan,

                    graphics_context: if cpu_painting {
                        CpuGraphicsContext
                    } else {
                        GpuGraphicsContext
                    },

                    render_layers: SmallVec1::new(),

                    paint_permission: false,
                    epoch: Epoch(0),
                    worker_txs: vec![],
                    worker_result_rx: worker_result_rx
                };

                // Now spawn the workers. We're only doing this after creating
                // the RenderTask object because spawn_workers was originally
                // written to be run afterward, and was refactored like so.
                let worker_txs = render_task.spawn_workers(worker_result_tx);
                render_task.worker_txs = worker_txs;                

                render_task.start();
            }

            debug!("render_task: shutdown_chan send");
            shutdown_chan.send(());
        });
    }

    fn start(&mut self) {
        debug!("render_task: beginning rendering loop");

        loop {
            match self.port.recv() {
                RenderMsg(render_layers) => {
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
                ReRenderMsg(tiles, scale, layer_id, epoch) => {
                    if self.epoch == epoch {
                        self.render(tiles, scale, layer_id);
                    } else {
                        debug!("renderer epoch mismatch: {:?} != {:?}", self.epoch, epoch);
                    }
                }
                UnusedBufferMsg(unused_buffers) => {
                    for buffer in unused_buffers.move_iter().rev() {
                        self.worker_txs.get(buffer.render_idx).send(WorkerUnusedBuffer(buffer));
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
                    for worker_tx in self.worker_txs.iter() {
                        let (tx, rx) = channel();
                        worker_tx.send(WorkerExit(tx));
                        rx.recv();
                    }
                    debug!("render_task: exitmsg response send");
                    response_ch.map(|ch| ch.send(()));
                    break;
                }
            }
        }
    }

    fn spawn_workers(&mut self, result_tx: Sender<Box<LayerBuffer>>) -> Vec<Sender<WorkerMsg>> {
        let mut worker_chans = vec![];
        for render_idx in range(0, self.opts.n_render_threads) {
            let (tx, rx) = channel();
            let result_tx = result_tx.clone();

            let opts = self.opts.clone();
            let graphics_context = self.graphics_context;
            let render_backend = self.opts.render_backend;
            let native_graphics_context = self.compositor.get_graphics_metadata().map(
                |md| NativePaintingGraphicsContext::from_metadata(&md));
            let native_graphics_context = native_graphics_context.expect("need native graphics context");
            let native_graphics_context = Some(native_graphics_context);
            let font_ctx_info = FontContextInfo {
                backend: self.opts.render_backend,
                needs_font_list: false,
                profiler_chan: self.profiler_chan.clone(),
            };
            let profiler_chan = self.profiler_chan.clone();
            let buffer_map: BufferMap<Box<LayerBuffer>> = BufferMap::new(10000000);

            spawn(proc() {
                let mut buffer_map = buffer_map;
                let mut native_graphics_context = native_graphics_context;
                loop {
                    let render_msg: WorkerMsg = rx.recv();
                    let render_data = match render_msg {
                        WorkerRender(render_data) => render_data,
                        WorkerUnusedBuffer(buffer) => {
                            buffer_map.insert(native_graphics_context.get_ref(), buffer);
                            continue
                        }
                        WorkerExit(tx) => {
                            // Cleanup and tell the RenderTask we're done
                            buffer_map.clear(native_graphics_context.get_ref());
                            drop(native_graphics_context.take_unwrap());
                            tx.send(());
                            break
                        }
                    };
                    let (tile,
                         display_list,
                         layer_position_x,
                         layer_position_y,
                         scale) = render_data;

                    // Optimize the display list for this tile.
                    let page_rect_au = geometry::f32_rect_to_au_rect(tile.page_rect);
                    let optimizer = DisplayListOptimizer::new(display_list,
                                                              page_rect_au);
                    let display_list = optimizer.optimize();

                    let width = tile.screen_rect.size.width;
                    let height = tile.screen_rect.size.height;

                    let size = Size2D(width as i32, height as i32);
                    let draw_target = match graphics_context {
                        CpuGraphicsContext => {
                            DrawTarget::new(render_backend, size, B8G8R8A8)
                        }
                        GpuGraphicsContext => {
                            // FIXME(pcwalton): Cache the components of draw targets
                            // (texture color buffer, renderbuffers) instead of recreating them.
                            let draw_target =
                                DrawTarget::new_with_fbo(render_backend,
                                                         native_graphics_context.get_ref(),
                                                         size,
                                                         B8G8R8A8);
                            draw_target.make_current();
                            draw_target
                        }
                    };

                    {
                        let mut font_ctx = box FontContext::new(font_ctx_info.clone());
                        // Build the render context.
                        let mut ctx = RenderContext {
                            draw_target: &draw_target,
                            font_ctx: &mut font_ctx,
                            opts: &opts,
                            page_rect: tile.page_rect,
                            screen_rect: tile.screen_rect,
                        };

                        // Apply the translation to render the tile we want.
                        let matrix: Matrix2D<AzFloat> = Matrix2D::identity();
                        let matrix = matrix.scale(scale as AzFloat, scale as AzFloat);
                        let matrix = matrix.translate(-(tile.page_rect.origin.x) as AzFloat,
                                                      -(tile.page_rect.origin.y) as AzFloat);
                        let matrix = matrix.translate(-(layer_position_x as AzFloat),
                                                      -(layer_position_y as AzFloat));

                        ctx.draw_target.set_transform(&matrix);

                        // Clear the buffer.
                        ctx.clear();

                        // Draw the display list.
                        profile(time::RenderingDrawingCategory, profiler_chan.clone(), || {
                            display_list.draw_into_context(&mut ctx);
                            ctx.draw_target.flush();
                        });
                    }

                    // Extract the texture from the draw target and place it into its slot in the
                    // buffer. If using CPU rendering, upload it first.
                    //
                    // FIXME(pcwalton): We should supply the texture and native surface *to* the
                    // draw target in GPU rendering mode, so that it doesn't have to recreate it.
                    let buffer = match graphics_context {
                        CpuGraphicsContext => {
                            let maybe_buffer = buffer_map.find(tile.screen_rect.size);
                            let buffer = match maybe_buffer {
                                Some(buffer) => {
                                    let mut buffer = buffer;
                                    buffer.rect = tile.page_rect;
                                    buffer.screen_pos = tile.screen_rect;
                                    buffer.resolution = scale;
                                    buffer.native_surface.mark_wont_leak();
                                    buffer
                                }
                                None => {
                                    // Create an empty native surface. We mark it as not leaking
                                    // in case it dies in transit to the compositor task.
                                    let mut native_surface: NativeSurface =
                                        layers::platform::surface::NativeSurfaceMethods::new(
                                            native_graphics_context.get_ref(),
                                            Size2D(width as i32, height as i32),
                                            width as i32 * 4);
                                    native_surface.mark_wont_leak();

                                    box LayerBuffer {
                                        native_surface: native_surface,
                                        rect: tile.page_rect,
                                        screen_pos: tile.screen_rect,
                                        resolution: scale,
                                        stride: (width * 4) as uint,
                                        render_idx: render_idx
                                    }
                                }
                            };

                            draw_target.snapshot().get_data_surface().with_data(|data| {
                                buffer.native_surface.upload(native_graphics_context.get_ref(), data);
                                debug!("RENDERER uploading to native surface {:d}",
                                       buffer.native_surface.get_id() as int);
                            });

                            buffer
                        }
                        GpuGraphicsContext => {
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
                                stride: (width * 4) as uint,
                                render_idx: render_idx
                            }
                        }
                    };

                    result_tx.send(buffer);
                }
            });
            worker_chans.push(tx)
        }

        return worker_chans;
    }

    /// Renders one layer and sends the tiles back to the layer.
    ///
    /// FIXME(pcwalton): We will probably want to eventually send all layers belonging to a page in
    /// one transaction, to avoid the user seeing inconsistent states.
    fn render(&mut self, tiles: Vec<BufferRequest>, scale: f32, layer_id: LayerId) {
        let mut tiles = Some(tiles);
        time::profile(time::RenderingCategory, self.profiler_chan.clone(), || {
            let tiles = tiles.take_unwrap();
            // FIXME: Try not to create a new array here.
            let mut new_buffers = vec!();

            // Find the appropriate render layer.
            let render_layer = match self.render_layers.iter().find(|layer| layer.id == layer_id) {
                Some(render_layer) => render_layer,
                None => return,
            };

            self.compositor.set_render_state(RenderingRenderState);

            // Distribute the tiles to the workers
            let num_tiles = tiles.len();
            let mut worker_idx = 0;

            for tile in tiles.move_iter() {

                let display_list = render_layer.display_list.clone();
                let layer_position_x = render_layer.position.origin.x;
                let layer_position_y = render_layer.position.origin.y;

                self.worker_txs.get(worker_idx).send(WorkerRender((tile, display_list, layer_position_x, layer_position_y, scale)));

                // Round-robin the work
                worker_idx = (worker_idx + 1) % self.worker_txs.len();
            }

            for _ in range(0, num_tiles) {
                new_buffers.push(self.worker_result_rx.recv());
            }

            let layer_buffer_set = box LayerBufferSet {
                buffers: new_buffers,
            };

            debug!("render_task: returning surface");
            if self.paint_permission {
                self.compositor.paint(self.id, render_layer.id, layer_buffer_set, self.epoch);
            } else {
                debug!("render_task: RendererReadyMsg send");
                let ConstellationChan(ref mut c) = self.constellation_chan;
                c.send(RendererReadyMsg(self.id));
            }
            self.compositor.set_render_state(IdleRenderState);
        })
    }
}
