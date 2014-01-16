/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// The task that handles all rendering/painting.

use azure::azure_hl::{B8G8R8A8, Color, DrawTarget, StolenGLResources};
use azure::AzFloat;
use geom::matrix2d::Matrix2D;
use geom::rect::Rect;
use geom::size::Size2D;
use layers::platform::surface::{NativePaintingGraphicsContext, NativeSurface};
use layers::platform::surface::{NativeSurfaceMethods};
use layers;
use servo_msg::compositor_msg::{Epoch, IdleRenderState, LayerBuffer, LayerBufferSet};
use servo_msg::compositor_msg::{RenderListener, RenderingRenderState};
use servo_msg::constellation_msg::{ConstellationChan, PipelineId, RendererReadyMsg};
use servo_msg::platform::surface::NativeSurfaceAzureMethods;
use servo_util::time::{ProfilerChan, profile};
use servo_util::time;
use servo_util::task::spawn_named;

use std::comm::{Chan, Port, SharedChan};
use extra::arc::Arc;

use buffer_map::BufferMap;
use display_list::DisplayList;
use font_context::FontContext;
use opts::Opts;
use render_context::RenderContext;

pub struct RenderLayer<T> {
    display_list: Arc<DisplayList<T>>,
    size: Size2D<uint>,
    color: Color
}

pub enum Msg<T> {
    RenderMsg(RenderLayer<T>),
    ReRenderMsg(~[BufferRequest], f32, Epoch),
    UnusedBufferMsg(~[~LayerBuffer]),
    PaintPermissionGranted,
    PaintPermissionRevoked,
    ExitMsg(Chan<()>),
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

// FIXME(rust#9155): this should be a newtype struct, but
// generic newtypes ICE when compiled cross-crate
pub struct RenderChan<T> {
    chan: SharedChan<Msg<T>>,
}

impl<T: Send> Clone for RenderChan<T> {
    fn clone(&self) -> RenderChan<T> {
        RenderChan {
            chan: self.chan.clone(),
        }
    }
}

impl<T: Send> RenderChan<T> {
    pub fn new() -> (Port<Msg<T>>, RenderChan<T>) {
        let (port, chan) = SharedChan::new();
        let render_chan = RenderChan {
            chan: chan,
        };
        (port, render_chan)
    }

    pub fn send(&self, msg: Msg<T>) {
        assert!(self.try_send(msg), "RenderChan.send: render port closed")
    }

    pub fn try_send(&self, msg: Msg<T>) -> bool {
        self.chan.try_send(msg)
    }
}

/// If we're using GPU rendering, this provides the metadata needed to create a GL context that
/// is compatible with that of the main thread.
enum GraphicsContext {
    CpuGraphicsContext,
    GpuGraphicsContext,
}

pub struct RenderTask<C,T> {
    id: PipelineId,
    port: Port<Msg<T>>,
    compositor: C,
    constellation_chan: ConstellationChan,
    font_ctx: ~FontContext,
    opts: Opts,

    /// A channel to the profiler.
    profiler_chan: ProfilerChan,

    /// The graphics context to use.
    graphics_context: GraphicsContext,

    /// The native graphics context.
    native_graphics_context: Option<NativePaintingGraphicsContext>,

    /// The layer to be rendered
    render_layer: Option<RenderLayer<T>>,

    /// Permission to send paint messages to the compositor
    paint_permission: bool,

    /// A counter for epoch messages
    epoch: Epoch,

    /// A data structure to store unused LayerBuffers
    buffer_map: BufferMap<~LayerBuffer>,
}

// If we implement this as a function, we get borrowck errors from borrowing
// the whole RenderTask struct.
macro_rules! native_graphics_context(
    ($task:expr) => (
        $task.native_graphics_context.as_ref().expect("Need a graphics context to do rendering")
    )
)

impl<C: RenderListener + Send,T:Send+Freeze> RenderTask<C,T> {
    pub fn create(id: PipelineId,
                  port: Port<Msg<T>>,
                  compositor: C,
                  constellation_chan: ConstellationChan,
                  opts: Opts,
                  profiler_chan: ProfilerChan,
                  shutdown_chan: Chan<()>) {
        spawn_named("RenderTask", proc() {

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
                    font_ctx: ~FontContext::new(opts.render_backend.clone(),
                                                false,
                                                profiler_chan.clone()),
                    opts: opts,
                    profiler_chan: profiler_chan,

                    graphics_context: if cpu_painting {
                        CpuGraphicsContext
                    } else {
                        GpuGraphicsContext
                    },

                    native_graphics_context: native_graphics_context,

                    render_layer: None,

                    paint_permission: false,
                    epoch: Epoch(0),
                    buffer_map: BufferMap::new(10000000),
                };

                render_task.start();

                // Destroy all the buffers.
                render_task.native_graphics_context.as_ref().map(
                    |ctx| render_task.buffer_map.clear(ctx));
            }

            debug!("render_task: shutdown_chan send");
            shutdown_chan.send(());
        });
    }

    fn start(&mut self) {
        debug!("render_task: beginning rendering loop");

        loop {
            match self.port.recv() {
                RenderMsg(render_layer) => {
                    if self.paint_permission {
                        self.epoch.next();
                        self.compositor.set_layer_page_size_and_color(self.id, render_layer.size, self.epoch, render_layer.color);
                    } else {
                        debug!("render_task: render ready msg");
                        self.constellation_chan.send(RendererReadyMsg(self.id));
                    }
                    self.render_layer = Some(render_layer);
                }
                ReRenderMsg(tiles, scale, epoch) => {
                    if self.epoch == epoch {
                        self.render(tiles, scale);
                    } else {
                        debug!("renderer epoch mismatch: {:?} != {:?}", self.epoch, epoch);
                    }
                }
                UnusedBufferMsg(unused_buffers) => {
                    // move_rev_iter is more efficient
                    for buffer in unused_buffers.move_rev_iter() {
                        self.buffer_map.insert(native_graphics_context!(self), buffer);
                    }
                }
                PaintPermissionGranted => {
                    self.paint_permission = true;
                    match self.render_layer {
                        Some(ref render_layer) => {
                            self.epoch.next();
                            self.compositor.set_layer_page_size_and_color(self.id, render_layer.size, self.epoch, render_layer.color);
                        }
                        None => {}
                    }
                }
                PaintPermissionRevoked => {
                    self.paint_permission = false;
                }
                ExitMsg(response_ch) => {
                    debug!("render_task: exitmsg response send");
                    response_ch.send(());
                    break;
                }
            }
        }
    }

    fn render(&mut self, tiles: ~[BufferRequest], scale: f32) {
        let render_layer;
        match self.render_layer {
            Some(ref r_layer) => {
                render_layer = r_layer;
            }
            _ => return, // nothing to do
        }

        self.compositor.set_render_state(RenderingRenderState);
        time::profile(time::RenderingCategory, self.profiler_chan.clone(), || {
            // FIXME: Try not to create a new array here.
            let mut new_buffers = ~[];

            // Divide up the layer into tiles.
            time::profile(time::RenderingPrepBuffCategory, self.profiler_chan.clone(), || {
                for tile in tiles.iter() {
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
                        let matrix = matrix.translate(-(tile.page_rect.origin.x) as AzFloat,
                                                      -(tile.page_rect.origin.y) as AzFloat);
                        
                        ctx.draw_target.set_transform(&matrix);
                        
                        // Clear the buffer.
                        ctx.clear();
                        
                        // Draw the display list.
                        profile(time::RenderingDrawingCategory, self.profiler_chan.clone(), || {
                            render_layer.display_list.get().draw_into_context(&mut ctx);
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
                            let buffer = match self.buffer_map.find(tile.screen_rect.size) {
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
                                            native_graphics_context!(self),
                                            Size2D(width as i32, height as i32),
                                            width as i32 * 4);
                                    native_surface.mark_wont_leak();

                                    ~LayerBuffer {
                                        native_surface: native_surface,
                                        rect: tile.page_rect,
                                        screen_pos: tile.screen_rect,
                                        resolution: scale,
                                        stride: (width * 4) as uint
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

                            ~LayerBuffer {
                                native_surface: native_surface,
                                rect: tile.page_rect,
                                screen_pos: tile.screen_rect,
                                resolution: scale,
                                stride: (width * 4) as uint
                            }
                        }
                    };
                    
                    new_buffers.push(buffer);
                }
            });

            let layer_buffer_set = ~LayerBufferSet {
                buffers: new_buffers,
            };

            debug!("render_task: returning surface");
            if self.paint_permission {
                self.compositor.paint(self.id, layer_buffer_set, self.epoch);
            } else {
                debug!("render_task: RendererReadyMsg send");
                self.constellation_chan.send(RendererReadyMsg(self.id));
            }
            self.compositor.set_render_state(IdleRenderState);
        })
    }
}

