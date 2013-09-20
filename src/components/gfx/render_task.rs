/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// The task that handles all rendering/painting.

use azure::{AzFloat, AzGLContext};
use azure::azure_hl::{B8G8R8A8, DrawTarget};
use display_list::DisplayList;
use servo_msg::compositor_msg::{RenderListener, IdleRenderState, RenderingRenderState, LayerBuffer};
use servo_msg::compositor_msg::{LayerBufferSet, Epoch};
use servo_msg::constellation_msg::{ConstellationChan, PipelineId, RendererReadyMsg};
use font_context::FontContext;
use geom::matrix2d::Matrix2D;
use geom::size::Size2D;
use geom::rect::Rect;
use opts::Opts;
use render_context::RenderContext;

use std::comm::{Chan, Port, SharedChan};
use std::task::spawn_with;
use extra::arc::Arc;

use servo_util::time::{ProfilerChan, profile};
use servo_util::time;

use buffer_map::BufferMap;


pub struct RenderLayer<T> {
    display_list: Arc<DisplayList<T>>,
    size: Size2D<uint>
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
#[deriving(Clone)]
pub struct RenderChan<T> {
    chan: SharedChan<Msg<T>>,
}
impl<T: Send> RenderChan<T> {
    pub fn new(chan: Chan<Msg<T>>) -> RenderChan<T> {
        RenderChan {
            chan: SharedChan::new(chan),
        }
    }
}
impl<T: Send> GenericChan<Msg<T>> for RenderChan<T> {
    fn send(&self, msg: Msg<T>) {
        assert!(self.try_send(msg), "RenderChan.send: render port closed")
    }
}
impl<T: Send> GenericSmartChan<Msg<T>> for RenderChan<T> {
    fn try_send(&self, msg: Msg<T>) -> bool {
        self.chan.try_send(msg)
    }
}

struct RenderTask<C,T> {
    id: PipelineId,
    port: Port<Msg<T>>,
    compositor: C,
    constellation_chan: ConstellationChan,
    font_ctx: @mut FontContext,
    opts: Opts,

    /// A channel to the profiler.
    profiler_chan: ProfilerChan,

    share_gl_context: AzGLContext,

    /// The layer to be rendered
    render_layer: Option<RenderLayer<T>>,
    /// Permission to send paint messages to the compositor
    paint_permission: bool,
    /// Cached copy of last layers rendered
    last_paint_msg: Option<~LayerBufferSet>,
    /// A counter for epoch messages
    epoch: Epoch,
    /// A data structure to store unused LayerBuffers
    buffer_map: BufferMap<~LayerBuffer>,
}

impl<C: RenderListener + Send,T:Send+Freeze> RenderTask<C,T> {
    pub fn create(id: PipelineId,
                  port: Port<Msg<T>>,
                  compositor: C,
                  constellation_chan: ConstellationChan,
                  opts: Opts,
                  profiler_chan: ProfilerChan) {

        do spawn_with((port, compositor, constellation_chan, opts, profiler_chan))
            |(port, compositor, constellation_chan, opts, profiler_chan)| {

            let share_gl_context = compositor.get_gl_context();

            // FIXME: rust/#5967
            let mut render_task = RenderTask {
                id: id,
                port: port,
                compositor: compositor,
                constellation_chan: constellation_chan,
                font_ctx: @mut FontContext::new(opts.render_backend.clone(),
                                                false,
                                                profiler_chan.clone()),
                opts: opts,
                profiler_chan: profiler_chan,
                share_gl_context: share_gl_context,
                render_layer: None,

                paint_permission: false,
                last_paint_msg: None,
                epoch: Epoch(0),
                buffer_map: BufferMap::new(10000000),
            };

            render_task.start();
        }
    }

    fn start(&mut self) {
        debug!("render_task: beginning rendering loop");

        loop {
            match self.port.recv() {
                RenderMsg(render_layer) => {
                    if self.paint_permission {
                        self.epoch.next();
                        self.compositor.set_layer_page_size(self.id, render_layer.size, self.epoch);
                    } else {
                        self.constellation_chan.send(RendererReadyMsg(self.id));
                    }
                    self.render_layer = Some(render_layer);
                    self.last_paint_msg = None;
                }
                ReRenderMsg(tiles, scale, epoch) => {
                    if self.epoch == epoch {
                        self.render(tiles, scale);
                    } else {
                        debug!("renderer epoch mismatch: %? != %?", self.epoch, epoch);
                    }
                }
                UnusedBufferMsg(unused_buffers) => {
                    // move_rev_iter is more efficient
                    for buffer in unused_buffers.move_rev_iter() {
                        self.buffer_map.insert(buffer);
                    }
                }
                PaintPermissionGranted => {
                    self.paint_permission = true;
                    match self.render_layer {
                        Some(ref render_layer) => {
                            self.epoch.next();
                            self.compositor.set_layer_page_size(self.id, render_layer.size, self.epoch);
                        }
                        None => {}
                    }
                    // FIXME: This sends the last paint request, anticipating what
                    // the compositor will ask for. However, even if it sends the right
                    // tiles, the compositor still asks for them, and they will be
                    // re-rendered redundantly.
                    match self.last_paint_msg {
                        Some(ref layer_buffer_set) => {
                            self.compositor.paint(self.id, layer_buffer_set.clone(), self.epoch);
                        }
                        None => {} // Nothing to do
                    }
                }
                PaintPermissionRevoked => {
                    self.paint_permission = false;
                }
                ExitMsg(response_ch) => {
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
        do time::profile(time::RenderingCategory, self.profiler_chan.clone()) {

            // FIXME: Try not to create a new array here.
            let mut new_buffers = ~[];

            // Divide up the layer into tiles.
            do time::profile(time::RenderingPrepBuffCategory, self.profiler_chan.clone()) {
                for tile in tiles.iter() {
                    let width = tile.screen_rect.size.width;
                    let height = tile.screen_rect.size.height;
                    
                    let buffer = match self.buffer_map.find(tile.screen_rect.size) {
                        Some(buffer) => {
                            let mut buffer = buffer;
                            buffer.rect = tile.page_rect;
                            buffer.screen_pos = tile.screen_rect;
                            buffer.resolution = scale;
                            buffer
                        }
                        None => ~LayerBuffer {
                            draw_target: DrawTarget::new_with_fbo(self.opts.render_backend,
                                                                  self.share_gl_context,
                                                                  Size2D(width as i32, height as i32),
                                                                  B8G8R8A8),
                            rect: tile.page_rect,
                            screen_pos: tile.screen_rect,
                            resolution: scale,
                            stride: (width * 4) as uint
                        }
                    };
                    
                    
                    {
                        // Build the render context.
                        let ctx = RenderContext {
                            canvas: &buffer,
                            font_ctx: self.font_ctx,
                            opts: &self.opts
                        };

                        // Apply the translation to render the tile we want.
                        let matrix: Matrix2D<AzFloat> = Matrix2D::identity();
                        let matrix = matrix.scale(scale as AzFloat, scale as AzFloat);
                        let matrix = matrix.translate(-(buffer.rect.origin.x) as AzFloat,
                                                      -(buffer.rect.origin.y) as AzFloat);
                        
                        ctx.canvas.draw_target.set_transform(&matrix);
                        
                        // Clear the buffer.
                        ctx.clear();
                        
                        // Draw the display list.
                        do profile(time::RenderingDrawingCategory, self.profiler_chan.clone()) {
                            render_layer.display_list.get().draw_into_context(&ctx);
                            ctx.canvas.draw_target.flush();
                        }
                    }
                    
                    new_buffers.push(buffer);
                    
                }

            }

            let layer_buffer_set = ~LayerBufferSet {
                buffers: new_buffers,
            };

            debug!("render_task: returning surface");
            if self.paint_permission {
                self.compositor.paint(self.id, layer_buffer_set.clone(), self.epoch);
            } else {
                self.constellation_chan.send(RendererReadyMsg(self.id));
            }
            debug!("caching paint msg");
            self.last_paint_msg = Some(layer_buffer_set);
            self.compositor.set_render_state(IdleRenderState);
        }
    }
}

