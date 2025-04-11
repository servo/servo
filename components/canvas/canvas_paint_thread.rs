/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

use canvas_traits::ConstellationCanvasMsg;
use canvas_traits::canvas::*;
use compositing_traits::CrossProcessCompositorApi;
use crossbeam_channel::{Sender, select, unbounded};
use euclid::default::Size2D;
use fonts::{FontContext, SystemFontServiceProxy};
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use log::warn;
use net_traits::ResourceThreads;
use webrender_api::ImageKey;

use crate::canvas_data::*;

pub struct CanvasPaintThread<'a> {
    canvases: HashMap<CanvasId, CanvasData<'a>>,
    next_canvas_id: CanvasId,
    compositor_api: CrossProcessCompositorApi,
    font_context: Arc<FontContext>,
}

impl<'a> CanvasPaintThread<'a> {
    fn new(
        compositor_api: CrossProcessCompositorApi,
        system_font_service: Arc<SystemFontServiceProxy>,
        resource_threads: ResourceThreads,
    ) -> CanvasPaintThread<'a> {
        CanvasPaintThread {
            canvases: HashMap::new(),
            next_canvas_id: CanvasId(0),
            compositor_api: compositor_api.clone(),
            font_context: Arc::new(FontContext::new(
                system_font_service,
                compositor_api,
                resource_threads,
            )),
        }
    }

    /// Creates a new `CanvasPaintThread` and returns an `IpcSender` to
    /// communicate with it.
    pub fn start(
        compositor_api: CrossProcessCompositorApi,
        system_font_service: Arc<SystemFontServiceProxy>,
        resource_threads: ResourceThreads,
    ) -> (Sender<ConstellationCanvasMsg>, IpcSender<CanvasMsg>) {
        let (ipc_sender, ipc_receiver) = ipc::channel::<CanvasMsg>().unwrap();
        let msg_receiver = ROUTER.route_ipc_receiver_to_new_crossbeam_receiver(ipc_receiver);
        let (create_sender, create_receiver) = unbounded();
        thread::Builder::new()
            .name("Canvas".to_owned())
            .spawn(move || {
                let mut canvas_paint_thread = CanvasPaintThread::new(
                    compositor_api, system_font_service, resource_threads);
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
                                        chan.send(canvas_paint_thread
                                            .canvas(canvas_id)
                                            .read_pixels(None, None)
                                            .as_ipc()
                                        ).unwrap();
                                    },
                                },
                                Err(e) => {
                                    warn!("Error on CanvasPaintThread receive ({})", e);
                                },
                            }
                        }
                        recv(create_receiver) -> msg => {
                            match msg {
                                Ok(ConstellationCanvasMsg::Create { sender: creator, size }) => {
                                    let canvas_data = canvas_paint_thread.create_canvas(size);
                                    creator.send(canvas_data).unwrap();
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

    pub fn create_canvas(&mut self, size: Size2D<u64>) -> (CanvasId, ImageKey) {
        let canvas_id = self.next_canvas_id;
        self.next_canvas_id.0 += 1;

        let canvas_data =
            CanvasData::new(size, self.compositor_api.clone(), self.font_context.clone());
        let image_key = canvas_data.image_key();
        self.canvases.insert(canvas_id, canvas_data);

        (canvas_id, image_key)
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
            Canvas2dMsg::FillPath(style, path) => {
                self.canvas(canvas_id).set_fill_style(style);
                self.canvas(canvas_id).fill_path(&path[..]);
            },
            Canvas2dMsg::Stroke(style) => {
                self.canvas(canvas_id).set_stroke_style(style);
                self.canvas(canvas_id).stroke();
            },
            Canvas2dMsg::StrokePath(style, path) => {
                self.canvas(canvas_id).set_stroke_style(style);
                self.canvas(canvas_id).stroke_path(&path[..]);
            },
            Canvas2dMsg::Clip => self.canvas(canvas_id).clip(),
            Canvas2dMsg::ClipPath(path) => self.canvas(canvas_id).clip_path(&path[..]),
            Canvas2dMsg::IsPointInCurrentPath(x, y, fill_rule, chan) => self
                .canvas(canvas_id)
                .is_point_in_path(x, y, fill_rule, chan),
            Canvas2dMsg::IsPointInPath(path, x, y, fill_rule, chan) => self
                .canvas(canvas_id)
                .is_point_in_path_(&path[..], x, y, fill_rule, chan),
            Canvas2dMsg::DrawImage(snapshot, dest_rect, source_rect, smoothing_enabled) => {
                let snapshot = snapshot.to_owned();
                self.canvas(canvas_id).draw_image(
                    snapshot.data(),
                    snapshot.size(),
                    dest_rect,
                    source_rect,
                    smoothing_enabled,
                    !snapshot.alpha_mode().is_premultiplied(),
                )
            },
            Canvas2dMsg::DrawEmptyImage(image_size, dest_rect, source_rect) => {
                self.canvas(canvas_id).draw_image(
                    &vec![0; image_size.area() as usize * 4],
                    image_size.to_u64(),
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
                    .read_pixels(Some(source_rect.to_u64()), Some(image_size.to_u64()));
                self.canvas(other_canvas_id).draw_image(
                    image_data.data(),
                    source_rect.size.to_u64(),
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
            Canvas2dMsg::SetLineDash(items) => self.canvas(canvas_id).set_line_dash(items),
            Canvas2dMsg::SetLineDashOffset(offset) => {
                self.canvas(canvas_id).set_line_dash_offset(offset)
            },
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
                let snapshot = self
                    .canvas(canvas_id)
                    .read_pixels(Some(dest_rect), Some(canvas_size));
                sender.send(snapshot.as_ipc()).unwrap();
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
            Canvas2dMsg::UpdateImage(sender) => {
                self.canvas(canvas_id).update_image_rendering();
                sender.send(()).unwrap();
            },
        }
    }

    fn canvas(&mut self, canvas_id: CanvasId) -> &mut CanvasData<'a> {
        self.canvases.get_mut(&canvas_id).expect("Bogus canvas id")
    }
}
