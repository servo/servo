/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use azure::azure_hl::AntialiasMode;
use canvas_data::*;
use canvas_traits::canvas::*;
use euclid::Size2D;
use ipc_channel::ipc::{self, IpcSender};
use pixels;
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::thread;
use webrender_api;

pub struct CanvasPaintThread <'a> {
    canvases: HashMap<CanvasId, CanvasData<'a>>,
    next_canvas_id: CanvasId,
}

impl<'a> CanvasPaintThread <'a> {
    fn new() -> CanvasPaintThread <'a> {
        CanvasPaintThread {
            canvases: HashMap::new(),
            next_canvas_id: CanvasId(0),
        }
    }

    /// Creates a new `CanvasPaintThread` and returns an `IpcSender` to
    /// communicate with it.
    pub fn start() -> IpcSender<CanvasMsg> {
        let (sender, receiver) = ipc::channel::<CanvasMsg>().unwrap();
        thread::Builder::new().name("CanvasThread".to_owned()).spawn(move || {
            let mut canvas_paint_thread = CanvasPaintThread::new();
            loop {
                match receiver.recv() {
                    Ok(msg) => {
                        match msg {
                            CanvasMsg::Canvas2d(message, canvas_id) => {
                                canvas_paint_thread.process_canvas_2d_message(message, canvas_id);
                            },
                            CanvasMsg::Close(canvas_id) =>{
                                canvas_paint_thread.canvases.remove(&canvas_id);
                            },
                            CanvasMsg::Create(creator, size, webrenderer_api_sender, antialias) => {
                                let canvas_id = canvas_paint_thread.create_canvas(
                                    size,
                                    webrenderer_api_sender,
                                    antialias
                                );
                                creator.send(canvas_id).unwrap();
                            },
                            CanvasMsg::Recreate(size, canvas_id) =>{
                                canvas_paint_thread.canvas(canvas_id).recreate(size);
                            },
                            CanvasMsg::FromScript(message, canvas_id) => {
                                match message {
                                    FromScriptMsg::SendPixels(chan) => {
                                        canvas_paint_thread.canvas(canvas_id).send_pixels(chan);
                                    }
                                }
                            },
                            CanvasMsg::FromLayout(message, canvas_id) => {
                                match message {
                                    FromLayoutMsg::SendData(chan) => {
                                        canvas_paint_thread.canvas(canvas_id).send_data(chan);
                                    }
                                }
                            },
                        }
                    },
                    Err(e) => {
                        warn!("Error on CanvasPaintThread receive ({})", e);
                    }
                }
            }
        }).expect("Thread spawning failed");

        sender
    }

    pub fn create_canvas(
        &mut self,
        size: Size2D<u32>,
        webrender_api_sender: webrender_api::RenderApiSender,
        antialias: bool
    ) -> CanvasId {
        let antialias = if antialias {
            AntialiasMode::Default
        } else {
            AntialiasMode::None
        };

        let canvas_id = self.next_canvas_id.clone();
        self.next_canvas_id.0 += 1;

        let canvas_data = CanvasData::new(size, webrender_api_sender, antialias, canvas_id.clone());
        self.canvases.insert(canvas_id.clone(), canvas_data);

        canvas_id
    }

    fn process_canvas_2d_message(&mut self, message: Canvas2dMsg, canvas_id: CanvasId) {
        match message {
            Canvas2dMsg::FillText(text, x, y, max_width) => {
                self.canvas(canvas_id).fill_text(text, x, y, max_width)
            },
            Canvas2dMsg::FillRect(ref rect) => {
                self.canvas(canvas_id).fill_rect(rect)
            },
            Canvas2dMsg::StrokeRect(ref rect) => {
                self.canvas(canvas_id).stroke_rect(rect)
            },
            Canvas2dMsg::ClearRect(ref rect) => {
                self.canvas(canvas_id).clear_rect(rect)
            },
            Canvas2dMsg::BeginPath => {
                self.canvas(canvas_id).begin_path()
            },
            Canvas2dMsg::ClosePath => {
                self.canvas(canvas_id).close_path()
            },
            Canvas2dMsg::Fill => {
                self.canvas(canvas_id).fill()
            },
            Canvas2dMsg::Stroke => {
                self.canvas(canvas_id).stroke()
            },
            Canvas2dMsg::Clip => {
                self.canvas(canvas_id).clip()
            },
            Canvas2dMsg::IsPointInPath(x, y, fill_rule, chan) => {
                self.canvas(canvas_id).is_point_in_path(x, y, fill_rule, chan)
            },
            Canvas2dMsg::DrawImage(
                imagedata,
                image_size,
                dest_rect,
                source_rect,
                smoothing_enabled,
            ) => {
                let data = match imagedata {
                    None => vec![0; image_size.width as usize * image_size.height as usize * 4],
                    Some(mut data) => {
                        pixels::byte_swap_colors_inplace(&mut data);
                        data.into()
                    },
                };
                self.canvas(canvas_id).draw_image(
                    data,
                    image_size,
                    dest_rect,
                    source_rect,
                    smoothing_enabled,
                )
            },
            Canvas2dMsg::DrawImageInOther(
                other_canvas_id,
                image_size,
                dest_rect,
                source_rect,
                smoothing
            ) => {
                let image_data = self.canvas(canvas_id).read_pixels(
                    source_rect.to_u32(),
                    image_size.to_u32(),
                );
                self.canvas(other_canvas_id).draw_image(
                    image_data.into(),
                    source_rect.size,
                    dest_rect,
                    source_rect,
                    smoothing,
                );
            },
            Canvas2dMsg::MoveTo(ref point) => {
                self.canvas(canvas_id).move_to(point)
            },
            Canvas2dMsg::LineTo(ref point) => {
                self.canvas(canvas_id).line_to(point)
            },
            Canvas2dMsg::Rect(ref rect) => {
                self.canvas(canvas_id).rect(rect)
            },
            Canvas2dMsg::QuadraticCurveTo(ref cp, ref pt) => {
                 self.canvas(canvas_id).quadratic_curve_to(cp, pt)
             }
            Canvas2dMsg::BezierCurveTo(ref cp1, ref cp2, ref pt) => {
                 self.canvas(canvas_id).bezier_curve_to(cp1, cp2, pt)
             }
            Canvas2dMsg::Arc(ref center, radius, start, end, ccw) => {
                 self.canvas(canvas_id).arc(center, radius, start, end, ccw)
             }
            Canvas2dMsg::ArcTo(ref cp1, ref cp2, radius) => {
                self.canvas(canvas_id).arc_to(cp1, cp2, radius)
            }
            Canvas2dMsg::Ellipse(ref center, radius_x, radius_y, rotation, start, end, ccw) => {
                self.canvas(canvas_id).ellipse(
                    center,
                    radius_x,
                    radius_y,
                    rotation,
                    start,
                    end,
                    ccw
                )
            }
            Canvas2dMsg::RestoreContext => {
                self.canvas(canvas_id).restore_context_state()
            },
            Canvas2dMsg::SaveContext => {
                self.canvas(canvas_id).save_context_state()
            },
            Canvas2dMsg::SetFillStyle(style) => {
                self.canvas(canvas_id).set_fill_style(style)
            },
            Canvas2dMsg::SetStrokeStyle(style) => {
                self.canvas(canvas_id).set_stroke_style(style)
            },
            Canvas2dMsg::SetLineWidth(width) => {
                self.canvas(canvas_id).set_line_width(width)
            },
            Canvas2dMsg::SetLineCap(cap) => {
                self.canvas(canvas_id).set_line_cap(cap)
            },
            Canvas2dMsg::SetLineJoin(join) => {
                self.canvas(canvas_id).set_line_join(join)
            },
            Canvas2dMsg::SetMiterLimit(limit) => {
                self.canvas(canvas_id).set_miter_limit(limit)
            },
            Canvas2dMsg::SetTransform(ref matrix) => {
                self.canvas(canvas_id).set_transform(matrix)
            },
            Canvas2dMsg::SetGlobalAlpha(alpha) => {
                self.canvas(canvas_id).set_global_alpha(alpha)
            },
            Canvas2dMsg::SetGlobalComposition(op) => {
                self.canvas(canvas_id).set_global_composition(op)
            },
            Canvas2dMsg::GetImageData(dest_rect, canvas_size, sender) => {
                let pixels = self.canvas(canvas_id).read_pixels(dest_rect, canvas_size);
                sender.send(&pixels).unwrap();
            },
            Canvas2dMsg::PutImageData(rect, receiver) => {
                self.canvas(canvas_id).put_image_data(receiver.recv().unwrap(), rect);
            },
            Canvas2dMsg::SetShadowOffsetX(value) => {
                self.canvas(canvas_id).set_shadow_offset_x(value)
            },
            Canvas2dMsg::SetShadowOffsetY(value) => {
                self.canvas(canvas_id).set_shadow_offset_y(value)
            },
            Canvas2dMsg::SetShadowBlur(value) => {
                self.canvas(canvas_id).set_shadow_blur(value)
            },
            Canvas2dMsg::SetShadowColor(ref color) => {
                self.canvas(canvas_id).set_shadow_color(color.to_azure_style())
            },
        }
    }

    fn canvas(&mut self, canvas_id: CanvasId) -> &mut CanvasData<'a> {
        self.canvases.get_mut(&canvas_id).expect("Bogus canvas id")
    }
}
