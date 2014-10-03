/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The task that handles all rendering/painting.

use buffer_map::BufferMap;
use display_list::optimizer::DisplayListOptimizer;
use display_list::DisplayList;
use font_context::FontContext;
use render_context::RenderContext;

use azure::azure_hl::{B8G8R8A8, Color, DrawTarget, StolenGLResources};
use azure::AzFloat;
use geom::matrix2d::Matrix2D;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use layers::platform::surface::{NativePaintingGraphicsContext, NativeSurface};
use layers::platform::surface::{NativeSurfaceMethods};
use layers::layers::{BufferRequest, LayerBuffer, LayerBufferSet};
use layers;
use servo_msg::compositor_msg::{Epoch, IdleRenderState, LayerId};
use servo_msg::compositor_msg::{LayerMetadata, RenderListener, RenderingRenderState, ScrollPolicy};
use servo_msg::constellation_msg::{ConstellationChan, Failure, FailureMsg, PipelineId};
use servo_msg::constellation_msg::{RendererReadyMsg};
use servo_msg::platform::surface::NativeSurfaceAzureMethods;
use servo_util::geometry;
use servo_util::opts::Opts;
use servo_util::smallvec::{SmallVec, SmallVec1};
use servo_util::task::spawn_named_with_send_on_failure;
use servo_util::time::{TimeProfilerChan, profile};
use servo_util::time;
use std::comm::{Receiver, Sender, channel};
use sync::Arc;
use font_cache_task::FontCacheTask;

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
    font_ctx: Box<FontContext>,
    opts: Opts,

    /// A channel to the time profiler.
    time_profiler_chan: TimeProfilerChan,

    /// The graphics context to use.
    graphics_context: GraphicsContext,

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
}

// If we implement this as a function, we get borrowck errors from borrowing
// the whole RenderTask struct.
macro_rules! native_graphics_context(
    ($task:expr) => (
        $task.native_graphics_context.as_ref().expect("Need a graphics context to do rendering")
    )
)

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
                  font_cache_task: FontCacheTask,
                  failure_msg: Failure,
                  opts: Opts,
                  time_profiler_chan: TimeProfilerChan,
                  shutdown_chan: Sender<()>) {

        let ConstellationChan(c) = constellation_chan.clone();
        let fc = font_cache_task.clone();

        spawn_named_with_send_on_failure("RenderTask", proc() {
            { // Ensures RenderTask and graphics context are destroyed before shutdown msg
                let native_graphics_context = compositor.get_graphics_metadata().map(
                    |md| NativePaintingGraphicsContext::from_metadata(&md));
                let cpu_painting = opts.cpu_painting;

                // FIXME: rust/#5967
                let mut render_task = RenderTask {
                    id: id,
                    port: port,
                    compositor: compositor,
                    constellation_chan: constellation_chan,
                    font_ctx: box FontContext::new(fc.clone()),
                    opts: opts,
                    time_profiler_chan: time_profiler_chan,

                    graphics_context: if cpu_painting {
                        CpuGraphicsContext
                    } else {
                        GpuGraphicsContext
                    },

                    native_graphics_context: native_graphics_context,

                    render_layers: SmallVec1::new(),

                    paint_permission: false,
                    epoch: Epoch(0),
                    buffer_map: BufferMap::new(10000000),
                };

                render_task.start();

                // Destroy all the buffers.
                match render_task.native_graphics_context.as_ref() {
                    Some(ctx) => render_task.buffer_map.clear(ctx),
                    None => (),
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

    /// Renders one layer and sends the tiles back to the layer.
    fn render(&mut self,
              replies: &mut Vec<(LayerId, Box<LayerBufferSet>)>,
              tiles: Vec<BufferRequest>,
              scale: f32,
              layer_id: LayerId) {
        time::profile(time::RenderingCategory, None, self.time_profiler_chan.clone(), || {
            // FIXME: Try not to create a new array here.
            let mut new_buffers = vec!();

            // Find the appropriate render layer.
            let render_layer = match self.render_layers.iter().find(|layer| layer.id == layer_id) {
                Some(render_layer) => render_layer,
                None => return,
            };

            // Divide up the layer into tiles.
            for tile in tiles.iter() {
                // page_rect is in coordinates relative to the layer origin, but all display list
                // components are relative to the page origin. We make page_rect relative to
                // the page origin before passing it to the optimizer.
                let page_rect =
                     tile.page_rect.translate(&Point2D(render_layer.position.origin.x as f32,
                                                       render_layer.position.origin.y as f32));
                let page_rect_au = geometry::f32_rect_to_au_rect(page_rect);

                // Optimize the display list for this tile.
                let optimizer = DisplayListOptimizer::new(render_layer.display_list.clone(),
                                                          page_rect_au);
                let display_list = optimizer.optimize();

                let width = tile.screen_rect.size.width;
                let height = tile.screen_rect.size.height;

                let size = Size2D(width as i32, height as i32);
                let draw_target = match self.graphics_context {
                    CpuGraphicsContext => {
                        DrawTarget::new(self.opts.render_backend, size, B8G8R8A8)
                    }
                    GpuGraphicsContext => {
                        // FIXME(pcwalton): Cache the components of draw targets
                        // (texture color buffer, renderbuffers) instead of recreating them.
                        let draw_target =
                            DrawTarget::new_with_fbo(self.opts.render_backend,
                                                     native_graphics_context!(self),
                                                     size,
                                                     B8G8R8A8);
                        draw_target.make_current();
                        draw_target
                    }
                };

                {
                    // Build the render context.
                    let mut ctx = RenderContext {
                        draw_target: &draw_target,
                        font_ctx: &mut self.font_ctx,
                        opts: &self.opts,
                        page_rect: tile.page_rect,
                        screen_rect: tile.screen_rect,
                    };

                    // Apply the translation to render the tile we want.
                    let matrix: Matrix2D<AzFloat> = Matrix2D::identity();
                    let matrix = matrix.scale(scale as AzFloat, scale as AzFloat);
                    let matrix = matrix.translate(-page_rect.origin.x as AzFloat,
                                                  -page_rect.origin.y as AzFloat);

                    ctx.draw_target.set_transform(&matrix);

                    // Clear the buffer.
                    ctx.clear();

                    // Draw the display list.
                    profile(time::RenderingDrawingCategory, None, self.time_profiler_chan.clone(), || {
                        display_list.draw_into_context(&mut ctx, &matrix);
                        ctx.draw_target.flush();
                    });
                }

                // Extract the texture from the draw target and place it into its slot in the
                // buffer. If using CPU rendering, upload it first.
                //
                // FIXME(pcwalton): We should supply the texture and native surface *to* the
                // draw target in GPU rendering mode, so that it doesn't have to recreate it.
                let buffer = match self.graphics_context {
                    CpuGraphicsContext => {
                        let mut buffer = match self.buffer_map.find(tile.screen_rect.size) {
                            Some(buffer) => {
                                let mut buffer = buffer;
                                buffer.rect = tile.page_rect;
                                buffer.screen_pos = tile.screen_rect;
                                buffer.resolution = scale;
                                buffer.native_surface.mark_wont_leak();
                                buffer.painted_with_cpu = true;
                                buffer.content_age = tile.content_age;
                                buffer
                            }
                            None => {
                                // Create an empty native surface. We mark it as not leaking
                                // in case it dies in transit to the compositor task.
                                let mut native_surface: NativeSurface =
                                    layers::platform::surface::NativeSurfaceMethods::new(
                                        native_graphics_context!(self),
                                        Size2D(width as i32, height as i32),
                                        width as i32 * 4);
                                native_surface.mark_wont_leak();

                                box LayerBuffer {
                                    native_surface: native_surface,
                                    rect: tile.page_rect,
                                    screen_pos: tile.screen_rect,
                                    resolution: scale,
                                    stride: (width * 4) as uint,
                                    painted_with_cpu: true,
                                    content_age: tile.content_age,
                                }
                            }
                        };

                        draw_target.snapshot().get_data_surface().with_data(|data| {
                            buffer.native_surface.upload(native_graphics_context!(self), data);
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
                            painted_with_cpu: false,
                            content_age: tile.content_age,
                        }
                    }
                };

                new_buffers.push(buffer);
            }

            let layer_buffer_set = box LayerBufferSet {
                buffers: new_buffers,
            };

            replies.push((render_layer.id, layer_buffer_set));
        })
    }
}
