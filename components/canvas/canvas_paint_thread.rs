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
use euclid::default::{Point2D, Rect, Size2D, Transform2D};
use fonts::{FontContext, SystemFontServiceProxy};
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use log::warn;
use net_traits::ResourceThreads;
use style::color::AbsoluteColor;
use style::properties::style_structs::Font as FontStyleStruct;
use webrender_api::ImageKey;

use crate::canvas_data::*;
use crate::raqote_backend::RaqoteBackend;

pub struct CanvasPaintThread<'a> {
    canvases: HashMap<CanvasId, Canvas<'a>>,
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

        let canvas_data = CanvasData::new(
            size,
            self.compositor_api.clone(),
            self.font_context.clone(),
            RaqoteBackend,
        );
        let image_key = canvas_data.image_key();
        self.canvases.insert(canvas_id, Canvas::Raqote(canvas_data));

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

    fn canvas(&mut self, canvas_id: CanvasId) -> &mut Canvas<'a> {
        self.canvases.get_mut(&canvas_id).expect("Bogus canvas id")
    }
}

enum Canvas<'a> {
    Raqote(CanvasData<'a, RaqoteBackend>),
}

impl Canvas<'_> {
    fn set_fill_style(&mut self, style: FillOrStrokeStyle) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.set_fill_style(style),
        }
    }

    fn fill(&mut self) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.fill(),
        }
    }

    fn fill_text(&mut self, text: String, x: f64, y: f64, max_width: Option<f64>, is_rtl: bool) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.fill_text(text, x, y, max_width, is_rtl),
        }
    }

    fn fill_rect(&mut self, rect: &Rect<f32>) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.fill_rect(rect),
        }
    }

    fn set_stroke_style(&mut self, style: FillOrStrokeStyle) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.set_stroke_style(style),
        }
    }

    fn stroke_rect(&mut self, rect: &Rect<f32>) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.stroke_rect(rect),
        }
    }

    fn begin_path(&mut self) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.begin_path(),
        }
    }

    fn close_path(&mut self) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.close_path(),
        }
    }

    fn fill_path(&mut self, path: &[PathSegment]) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.fill_path(path),
        }
    }

    fn stroke(&mut self) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.stroke(),
        }
    }

    fn stroke_path(&mut self, path: &[PathSegment]) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.stroke_path(path),
        }
    }

    fn clip(&mut self) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.clip(),
        }
    }

    fn is_point_in_path(&mut self, x: f64, y: f64, fill_rule: FillRule, chan: IpcSender<bool>) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.is_point_in_path(x, y, fill_rule, chan),
        }
    }

    fn is_point_in_path_(
        &mut self,
        path: &[PathSegment],
        x: f64,
        y: f64,
        fill_rule: FillRule,
        chan: IpcSender<bool>,
    ) {
        match self {
            Canvas::Raqote(canvas_data) => {
                canvas_data.is_point_in_path_(path, x, y, fill_rule, chan)
            },
        }
    }

    fn clear_rect(&mut self, rect: &Rect<f32>) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.clear_rect(rect),
        }
    }

    fn draw_image(
        &mut self,
        data: &[u8],
        size: Size2D<u64>,
        dest_rect: Rect<f64>,
        source_rect: Rect<f64>,
        smoothing_enabled: bool,
        is_premultiplied: bool,
    ) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.draw_image(
                data,
                size,
                dest_rect,
                source_rect,
                smoothing_enabled,
                is_premultiplied,
            ),
        }
    }

    fn read_pixels(
        &mut self,
        read_rect: Option<Rect<u64>>,
        canvas_size: Option<Size2D<u64>>,
    ) -> snapshot::Snapshot {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.read_pixels(read_rect, canvas_size),
        }
    }

    fn move_to(&mut self, point: &Point2D<f32>) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.move_to(point),
        }
    }

    fn line_to(&mut self, point: &Point2D<f32>) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.line_to(point),
        }
    }

    fn rect(&mut self, rect: &Rect<f32>) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.rect(rect),
        }
    }

    fn quadratic_curve_to(&mut self, cp: &Point2D<f32>, pt: &Point2D<f32>) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.quadratic_curve_to(cp, pt),
        }
    }

    fn bezier_curve_to(&mut self, cp1: &Point2D<f32>, cp2: &Point2D<f32>, pt: &Point2D<f32>) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.bezier_curve_to(cp1, cp2, pt),
        }
    }

    fn arc(&mut self, center: &Point2D<f32>, radius: f32, start: f32, end: f32, ccw: bool) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.arc(center, radius, start, end, ccw),
        }
    }

    fn arc_to(&mut self, cp1: &Point2D<f32>, cp2: &Point2D<f32>, radius: f32) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.arc_to(cp1, cp2, radius),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn ellipse(
        &mut self,
        center: &Point2D<f32>,
        radius_x: f32,
        radius_y: f32,
        rotation: f32,
        start: f32,
        end: f32,
        ccw: bool,
    ) {
        match self {
            Canvas::Raqote(canvas_data) => {
                canvas_data.ellipse(center, radius_x, radius_y, rotation, start, end, ccw)
            },
        }
    }

    fn restore_context_state(&mut self) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.restore_context_state(),
        }
    }

    fn save_context_state(&mut self) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.save_context_state(),
        }
    }

    fn set_line_width(&mut self, width: f32) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.set_line_width(width),
        }
    }

    fn set_line_cap(&mut self, cap: LineCapStyle) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.set_line_cap(cap),
        }
    }

    fn set_line_join(&mut self, join: LineJoinStyle) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.set_line_join(join),
        }
    }

    fn set_miter_limit(&mut self, limit: f32) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.set_miter_limit(limit),
        }
    }

    fn set_line_dash(&mut self, items: Vec<f32>) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.set_line_dash(items),
        }
    }

    fn set_line_dash_offset(&mut self, offset: f32) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.set_line_dash_offset(offset),
        }
    }

    fn set_transform(&mut self, matrix: &Transform2D<f32>) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.set_transform(matrix),
        }
    }

    fn set_global_alpha(&mut self, alpha: f32) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.set_global_alpha(alpha),
        }
    }

    fn set_global_composition(&mut self, op: CompositionOrBlending) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.set_global_composition(op),
        }
    }

    fn set_shadow_offset_x(&mut self, value: f64) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.set_shadow_offset_x(value),
        }
    }

    fn set_shadow_offset_y(&mut self, value: f64) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.set_shadow_offset_y(value),
        }
    }

    fn set_shadow_blur(&mut self, value: f64) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.set_shadow_blur(value),
        }
    }

    fn set_shadow_color(&mut self, color: AbsoluteColor) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.set_shadow_color(color),
        }
    }

    fn set_font(&mut self, font_style: FontStyleStruct) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.set_font(font_style),
        }
    }

    fn set_text_align(&mut self, text_align: TextAlign) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.set_text_align(text_align),
        }
    }

    fn set_text_baseline(&mut self, text_baseline: TextBaseline) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.set_text_baseline(text_baseline),
        }
    }

    fn measure_text(&mut self, text: String) -> TextMetrics {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.measure_text(text),
        }
    }

    fn clip_path(&mut self, path: &[PathSegment]) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.clip_path(path),
        }
    }

    fn get_transform(&self) -> Transform2D<f32> {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.get_transform(),
        }
    }

    fn put_image_data(&mut self, unwrap: Vec<u8>, rect: Rect<u64>) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.put_image_data(unwrap, rect),
        }
    }

    fn update_image_rendering(&mut self) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.update_image_rendering(),
        }
    }

    fn recreate(&mut self, size: Option<Size2D<u64>>) {
        match self {
            Canvas::Raqote(canvas_data) => canvas_data.recreate(size),
        }
    }
}
