/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

use canvas_traits::canvas::*;
use canvas_traits::ConstellationCanvasMsg;
use crossbeam_channel::{select, unbounded, Sender};
use euclid::default::Size2D;
use fonts::{FontCacheThread, FontContext};
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use log::warn;
use net_traits::ResourceThreads;
use webrender_api::ImageKey;
use webrender_traits::ImageUpdate;

use crate::canvas_data::*;

pub enum AntialiasMode {
    Default,
    None,
}

pub trait WebrenderApi {
    /// Attempt to generate an [`ImageKey`], returning `None` in case of failure.
    fn generate_key(&self) -> Option<ImageKey>;
    fn update_images(&self, updates: Vec<ImageUpdate>);
    fn clone(&self) -> Box<dyn WebrenderApi>;
}

pub struct CanvasPaintThread<'a> {
    canvases: HashMap<CanvasId, CanvasData<'a>>,
    next_canvas_id: CanvasId,
    webrender_api: Box<dyn WebrenderApi>,
    font_context: Arc<FontContext<FontCacheThread>>,
}

impl<'a> CanvasPaintThread<'a> {
    fn new(
        webrender_api: Box<dyn WebrenderApi>,
        font_cache_thread: FontCacheThread,
        resource_threads: ResourceThreads,
    ) -> CanvasPaintThread<'a> {
        CanvasPaintThread {
            canvases: HashMap::new(),
            next_canvas_id: CanvasId(0),
            webrender_api,
            font_context: Arc::new(FontContext::new(font_cache_thread, resource_threads)),
        }
    }

    /// Creates a new `CanvasPaintThread` and returns an `IpcSender` to
    /// communicate with it.
    pub fn start(
        webrender_api: Box<dyn WebrenderApi + Send>,
        font_cache_thread: FontCacheThread,
        resource_threads: ResourceThreads,
    ) -> (Sender<ConstellationCanvasMsg>, IpcSender<CanvasMsg>) {
        let (ipc_sender, ipc_receiver) = ipc::channel::<CanvasMsg>().unwrap();
        let msg_receiver = ROUTER.route_ipc_receiver_to_new_crossbeam_receiver(ipc_receiver);
        let (create_sender, create_receiver) = unbounded();
        thread::Builder::new()
            .name("Canvas".to_owned())
            .spawn(move || {
                let mut canvas_paint_thread = CanvasPaintThread::new(
                    webrender_api, font_cache_thread, resource_threads);
                loop {
                    select! {
                        recv(msg_receiver) -> msg => {
                            match msg {
                                Ok(CanvasMsg::Canvas2d(message, canvas_id)) => {
                                    canvas_paint_thread.process_canvas_2d_message(message, canvas_id);
                                },
                                Ok(CanvasMsg::Close(canvas_id)) => {
                                    canvas_paint_thread.canvases.remove(&canvas_id);
                                },
                                Ok(CanvasMsg::Recreate(size, canvas_id)) => {
                                    canvas_paint_thread.canvas(canvas_id).recreate(size);
                                },
                                Ok(CanvasMsg::FromScript(message, canvas_id)) => match message {
                                    FromScriptMsg::SendPixels(chan) => {
                                        canvas_paint_thread.canvas(canvas_id).send_pixels(chan);
                                    },
                                },
                                Ok(CanvasMsg::FromLayout(message, canvas_id)) => match message {
                                    FromLayoutMsg::SendData(chan) => {
                                        canvas_paint_thread.canvas(canvas_id).send_data(chan);
                                    },
                                },
                                Err(e) => {
                                    warn!("Error on CanvasPaintThread receive ({})", e);
                                },
                            }
                        }
                        recv(create_receiver) -> msg => {
                            match msg {
                                Ok(ConstellationCanvasMsg::Create {
                                    id_sender: creator,
                                    size,
                                    antialias
                                }) => {
                                    let canvas_id = canvas_paint_thread.create_canvas(size, antialias);
                                    creator.send(canvas_id).unwrap();
                                },
                                Ok(ConstellationCanvasMsg::Exit) => break,
                                Err(e) => {
                                    warn!("Error on CanvasPaintThread receive ({})", e);
                                    break;
                                },
                            }
                        }
                    }
                }
            })
            .expect("Thread spawning failed");

        (create_sender, ipc_sender)
    }

    pub fn create_canvas(&mut self, size: Size2D<u64>, antialias: bool) -> CanvasId {
        let antialias = if antialias {
            AntialiasMode::Default
        } else {
            AntialiasMode::None
        };

        let canvas_id = self.next_canvas_id;
        self.next_canvas_id.0 += 1;

        let canvas_data = CanvasData::new(
            size,
            self.webrender_api.clone(),
            antialias,
            self.font_context.clone(),
        );
        self.canvases.insert(canvas_id, canvas_data);

        canvas_id
    }

    fn process_canvas_2d_message(&mut self, message: Canvas2dMsg, canvas_id: CanvasId) {
        match message {
            Canvas2dMsg::FillText(text, x, y, max_width, style, is_rtl) => {
                self.canvas(canvas_id).set_fill_style(style);
                self.canvas(canvas_id)
                    .fill_text(text, x, y, max_width, is_rtl);
            },
            Canvas2dMsg::FillRect(rect, style) => {
                self.canvas(canvas_id).set_fill_style(style);
                self.canvas(canvas_id).fill_rect(&rect);
            },
            Canvas2dMsg::StrokeRect(rect, style) => {
                self.canvas(canvas_id).set_stroke_style(style);
                self.canvas(canvas_id).stroke_rect(&rect);
            },
            Canvas2dMsg::ClearRect(ref rect) => self.canvas(canvas_id).clear_rect(rect),
            Canvas2dMsg::BeginPath => self.canvas(canvas_id).begin_path(),
            Canvas2dMsg::ClosePath => self.canvas(canvas_id).close_path(),
            Canvas2dMsg::Fill(style) => {
                self.canvas(canvas_id).set_fill_style(style);
                self.canvas(canvas_id).fill();
            },
            Canvas2dMsg::Stroke(style) => {
                self.canvas(canvas_id).set_stroke_style(style);
                self.canvas(canvas_id).stroke();
            },
            Canvas2dMsg::Clip => self.canvas(canvas_id).clip(),
            Canvas2dMsg::IsPointInPath(x, y, fill_rule, chan) => self
                .canvas(canvas_id)
                .is_point_in_path(x, y, fill_rule, chan),
            Canvas2dMsg::DrawImage(
                ref image_data,
                image_size,
                dest_rect,
                source_rect,
                smoothing_enabled,
            ) => self.canvas(canvas_id).draw_image(
                image_data,
                image_size,
                dest_rect,
                source_rect,
                smoothing_enabled,
                true,
            ),
            Canvas2dMsg::DrawEmptyImage(image_size, dest_rect, source_rect) => {
                self.canvas(canvas_id).draw_image(
                    &vec![0; image_size.area() as usize * 4],
                    image_size,
                    dest_rect,
                    source_rect,
                    false,
                    false,
                )
            },
            Canvas2dMsg::DrawImageInOther(
                other_canvas_id,
                image_size,
                dest_rect,
                source_rect,
                smoothing,
            ) => {
                let image_data = self
                    .canvas(canvas_id)
                    .read_pixels(source_rect.to_u64(), image_size.to_u64());
                self.canvas(other_canvas_id).draw_image(
                    &image_data,
                    source_rect.size,
                    dest_rect,
                    source_rect,
                    smoothing,
                    false,
                );
            },
            Canvas2dMsg::MoveTo(ref point) => self.canvas(canvas_id).move_to(point),
            Canvas2dMsg::LineTo(ref point) => self.canvas(canvas_id).line_to(point),
            Canvas2dMsg::Rect(ref rect) => self.canvas(canvas_id).rect(rect),
            Canvas2dMsg::QuadraticCurveTo(ref cp, ref pt) => {
                self.canvas(canvas_id).quadratic_curve_to(cp, pt)
            },
            Canvas2dMsg::BezierCurveTo(ref cp1, ref cp2, ref pt) => {
                self.canvas(canvas_id).bezier_curve_to(cp1, cp2, pt)
            },
            Canvas2dMsg::Arc(ref center, radius, start, end, ccw) => {
                self.canvas(canvas_id).arc(center, radius, start, end, ccw)
            },
            Canvas2dMsg::ArcTo(ref cp1, ref cp2, radius) => {
                self.canvas(canvas_id).arc_to(cp1, cp2, radius)
            },
            Canvas2dMsg::Ellipse(ref center, radius_x, radius_y, rotation, start, end, ccw) => self
                .canvas(canvas_id)
                .ellipse(center, radius_x, radius_y, rotation, start, end, ccw),
            Canvas2dMsg::MeasureText(text, sender) => {
                let metrics = self.canvas(canvas_id).measure_text(text);
                sender.send(metrics).unwrap();
            },
            Canvas2dMsg::RestoreContext => self.canvas(canvas_id).restore_context_state(),
            Canvas2dMsg::SaveContext => self.canvas(canvas_id).save_context_state(),
            Canvas2dMsg::SetLineWidth(width) => self.canvas(canvas_id).set_line_width(width),
            Canvas2dMsg::SetLineCap(cap) => self.canvas(canvas_id).set_line_cap(cap),
            Canvas2dMsg::SetLineJoin(join) => self.canvas(canvas_id).set_line_join(join),
            Canvas2dMsg::SetMiterLimit(limit) => self.canvas(canvas_id).set_miter_limit(limit),
            Canvas2dMsg::GetTransform(sender) => {
                let transform = self.canvas(canvas_id).get_transform();
                sender.send(transform).unwrap();
            },
            Canvas2dMsg::SetTransform(ref matrix) => self.canvas(canvas_id).set_transform(matrix),
            Canvas2dMsg::SetGlobalAlpha(alpha) => self.canvas(canvas_id).set_global_alpha(alpha),
            Canvas2dMsg::SetGlobalComposition(op) => {
                self.canvas(canvas_id).set_global_composition(op)
            },
            Canvas2dMsg::GetImageData(dest_rect, canvas_size, sender) => {
                let pixels = self.canvas(canvas_id).read_pixels(dest_rect, canvas_size);
                sender.send(&pixels).unwrap();
            },
            Canvas2dMsg::PutImageData(rect, receiver) => {
                self.canvas(canvas_id)
                    .put_image_data(receiver.recv().unwrap(), rect);
            },
            Canvas2dMsg::SetShadowOffsetX(value) => {
                self.canvas(canvas_id).set_shadow_offset_x(value)
            },
            Canvas2dMsg::SetShadowOffsetY(value) => {
                self.canvas(canvas_id).set_shadow_offset_y(value)
            },
            Canvas2dMsg::SetShadowBlur(value) => self.canvas(canvas_id).set_shadow_blur(value),
            Canvas2dMsg::SetShadowColor(color) => self.canvas(canvas_id).set_shadow_color(color),
            Canvas2dMsg::SetFont(font_style) => self.canvas(canvas_id).set_font(font_style),
            Canvas2dMsg::SetTextAlign(text_align) => {
                self.canvas(canvas_id).set_text_align(text_align)
            },
            Canvas2dMsg::SetTextBaseline(text_baseline) => {
                self.canvas(canvas_id).set_text_baseline(text_baseline)
            },
        }
    }

    fn canvas(&mut self, canvas_id: CanvasId) -> &mut CanvasData<'a> {
        self.canvases.get_mut(&canvas_id).expect("Bogus canvas id")
    }
}
