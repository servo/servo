/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use azure::azure::AzFloat;
use azure::azure_hl::{AntialiasMode, CapStyle, CompositionOp, JoinStyle};
use azure::azure_hl::{BackendType, DrawOptions, DrawTarget, Pattern, StrokeOptions, SurfaceFormat};
use azure::azure_hl::{Color, ColorPattern, DrawSurfaceOptions, Filter, PathBuilder};
use azure::azure_hl::{ExtendMode, GradientStop, LinearGradientPattern, RadialGradientPattern};
use azure::azure_hl::SurfacePattern;
use canvas_traits::canvas::*;
use cssparser::RGBA;
use euclid::{Transform2D, Point2D, Vector2D, Rect, Size2D};
use ipc_channel::ipc::{self, IpcSender, IpcReceiver};
use num_traits::ToPrimitive;
use std::borrow::ToOwned;
use std::mem;
use std::sync::Arc;
use std::thread;
use webrender_api;

impl<'a> CanvasPaintThread<'a> {
    /// It reads image data from the canvas
    /// canvas_size: The size of the canvas we're reading from
    /// read_rect: The area of the canvas we want to read from
    fn read_pixels(&self, read_rect: Rect<i32>, canvas_size: Size2D<f64>) -> Vec<u8>{
        let canvas_size = canvas_size.to_i32();
        let canvas_rect = Rect::new(Point2D::new(0i32, 0i32), canvas_size);
        let src_read_rect = canvas_rect.intersection(&read_rect).unwrap_or(Rect::zero());

        let mut image_data = vec![];
        if src_read_rect.is_empty() || canvas_size.width <= 0 && canvas_size.height <= 0 {
          return image_data;
        }

        let data_surface = self.drawtarget.snapshot().get_data_surface();
        let mut src_data = Vec::new();
        data_surface.with_data(|element| { src_data = element.to_vec(); });
        let stride = data_surface.stride();

        //start offset of the copyable rectangle
        let mut src = (src_read_rect.origin.y * stride + src_read_rect.origin.x * 4) as usize;
        //copy the data to the destination vector
        for _ in 0..src_read_rect.size.height {
            let row = &src_data[src .. src + (4 * src_read_rect.size.width) as usize];
            image_data.extend_from_slice(row);
            src += stride as usize;
        }

        image_data
    }
}

pub struct CanvasPaintThread<'a> {
    drawtarget: DrawTarget,
    /// TODO(pcwalton): Support multiple paths.
    path_builder: PathBuilder,
    state: CanvasPaintState<'a>,
    saved_states: Vec<CanvasPaintState<'a>>,
    webrender_api: webrender_api::RenderApi,
    image_key: Option<webrender_api::ImageKey>,
    /// An old webrender image key that can be deleted when the next epoch ends.
    old_image_key: Option<webrender_api::ImageKey>,
    /// An old webrender image key that can be deleted when the current epoch ends.
    very_old_image_key: Option<webrender_api::ImageKey>,
}

#[derive(Clone)]
struct CanvasPaintState<'a> {
    draw_options: DrawOptions,
    fill_style: Pattern,
    stroke_style: Pattern,
    stroke_opts: StrokeOptions<'a>,
    /// The current 2D transform matrix.
    transform: Transform2D<f32>,
    shadow_offset_x: f64,
    shadow_offset_y: f64,
    shadow_blur: f64,
    shadow_color: Color,
}

impl<'a> CanvasPaintState<'a> {
    fn new(antialias: AntialiasMode) -> CanvasPaintState<'a> {
        CanvasPaintState {
            draw_options: DrawOptions::new(1.0, CompositionOp::Over, antialias),
            fill_style: Pattern::Color(ColorPattern::new(Color::black())),
            stroke_style: Pattern::Color(ColorPattern::new(Color::black())),
            stroke_opts: StrokeOptions::new(1.0, JoinStyle::MiterOrBevel, CapStyle::Butt, 10.0, &[]),
            transform: Transform2D::identity(),
            shadow_offset_x: 0.0,
            shadow_offset_y: 0.0,
            shadow_blur: 0.0,
            shadow_color: Color::transparent(),
        }
    }
}

impl<'a> CanvasPaintThread<'a> {
    fn new(size: Size2D<i32>,
           webrender_api_sender: webrender_api::RenderApiSender,
           antialias: AntialiasMode) -> CanvasPaintThread<'a> {
        let draw_target = CanvasPaintThread::create(size);
        let path_builder = draw_target.create_path_builder();
        let webrender_api = webrender_api_sender.create_api();
        CanvasPaintThread {
            drawtarget: draw_target,
            path_builder: path_builder,
            state: CanvasPaintState::new(antialias),
            saved_states: vec![],
            webrender_api: webrender_api,
            image_key: None,
            old_image_key: None,
            very_old_image_key: None,
        }
    }

    /// Creates a new `CanvasPaintThread` and returns an `IpcSender` to
    /// communicate with it.
    pub fn start(size: Size2D<i32>,
                 webrender_api_sender: webrender_api::RenderApiSender,
                 antialias: bool,
                 receiver: IpcReceiver<CanvasMsg>) {
        let antialias = if antialias {
            AntialiasMode::Default
        } else {
            AntialiasMode::None
        };
        thread::Builder::new().name("CanvasThread".to_owned()).spawn(move || {
            let mut painter = CanvasPaintThread::new(size, webrender_api_sender, antialias);
            loop {
                let msg = receiver.recv();
                match msg.unwrap() {
                    CanvasMsg::Canvas2d(message) => {
                        match message {
                            Canvas2dMsg::FillText(text, x, y, max_width) => painter.fill_text(text, x, y, max_width),
                            Canvas2dMsg::FillRect(ref rect) => painter.fill_rect(rect),
                            Canvas2dMsg::StrokeRect(ref rect) => painter.stroke_rect(rect),
                            Canvas2dMsg::ClearRect(ref rect) => painter.clear_rect(rect),
                            Canvas2dMsg::BeginPath => painter.begin_path(),
                            Canvas2dMsg::ClosePath => painter.close_path(),
                            Canvas2dMsg::Fill => painter.fill(),
                            Canvas2dMsg::Stroke => painter.stroke(),
                            Canvas2dMsg::Clip => painter.clip(),
                            Canvas2dMsg::IsPointInPath(x, y, fill_rule, chan) => {
                                painter.is_point_in_path(x, y, fill_rule, chan)
                            },
                            Canvas2dMsg::DrawImage(imagedata, image_size, dest_rect, source_rect,
                                                   smoothing_enabled) => {
                                painter.draw_image(imagedata, image_size, dest_rect, source_rect, smoothing_enabled)
                            }
                            Canvas2dMsg::DrawImageSelf(image_size, dest_rect, source_rect, smoothing_enabled) => {
                                painter.draw_image_self(image_size, dest_rect, source_rect, smoothing_enabled)
                            }
                            Canvas2dMsg::DrawImageInOther(
                                renderer, image_size, dest_rect, source_rect, smoothing, sender
                            ) => {
                                painter.draw_image_in_other(
                                    renderer, image_size, dest_rect, source_rect, smoothing, sender)
                            }
                            Canvas2dMsg::MoveTo(ref point) => painter.move_to(point),
                            Canvas2dMsg::LineTo(ref point) => painter.line_to(point),
                            Canvas2dMsg::Rect(ref rect) => painter.rect(rect),
                            Canvas2dMsg::QuadraticCurveTo(ref cp, ref pt) => {
                                painter.quadratic_curve_to(cp, pt)
                            }
                            Canvas2dMsg::BezierCurveTo(ref cp1, ref cp2, ref pt) => {
                                painter.bezier_curve_to(cp1, cp2, pt)
                            }
                            Canvas2dMsg::Arc(ref center, radius, start, end, ccw) => {
                                painter.arc(center, radius, start, end, ccw)
                            }
                            Canvas2dMsg::ArcTo(ref cp1, ref cp2, radius) => {
                                painter.arc_to(cp1, cp2, radius)
                            }
                            Canvas2dMsg::Ellipse(ref center, radius_x, radius_y, rotation, start, end, ccw) => {
                                painter.ellipse(center, radius_x, radius_y, rotation, start, end, ccw)
                            }
                            Canvas2dMsg::RestoreContext => painter.restore_context_state(),
                            Canvas2dMsg::SaveContext => painter.save_context_state(),
                            Canvas2dMsg::SetFillStyle(style) => painter.set_fill_style(style),
                            Canvas2dMsg::SetStrokeStyle(style) => painter.set_stroke_style(style),
                            Canvas2dMsg::SetLineWidth(width) => painter.set_line_width(width),
                            Canvas2dMsg::SetLineCap(cap) => painter.set_line_cap(cap),
                            Canvas2dMsg::SetLineJoin(join) => painter.set_line_join(join),
                            Canvas2dMsg::SetMiterLimit(limit) => painter.set_miter_limit(limit),
                            Canvas2dMsg::SetTransform(ref matrix) => painter.set_transform(matrix),
                            Canvas2dMsg::SetGlobalAlpha(alpha) => painter.set_global_alpha(alpha),
                            Canvas2dMsg::SetGlobalComposition(op) => painter.set_global_composition(op),
                            Canvas2dMsg::GetImageData(dest_rect, canvas_size, chan)
                                => painter.image_data(dest_rect, canvas_size, chan),
                            Canvas2dMsg::PutImageData(imagedata, offset, image_data_size, dirty_rect)
                                => painter.put_image_data(imagedata, offset, image_data_size, dirty_rect),
                            Canvas2dMsg::SetShadowOffsetX(value) => painter.set_shadow_offset_x(value),
                            Canvas2dMsg::SetShadowOffsetY(value) => painter.set_shadow_offset_y(value),
                            Canvas2dMsg::SetShadowBlur(value) => painter.set_shadow_blur(value),
                            Canvas2dMsg::SetShadowColor(ref color) => painter.set_shadow_color(color.to_azure_style()),
                        }
                    },
                    CanvasMsg::Close => break,
                    CanvasMsg::Recreate(size) => painter.recreate(size),
                    CanvasMsg::FromScript(message) => {
                        match message {
                            FromScriptMsg::SendPixels(chan) => {
                                painter.send_pixels(chan)
                            }
                        }
                    }
                    CanvasMsg::FromLayout(message) => {
                        match message {
                            FromLayoutMsg::SendData(chan) => {
                                painter.send_data(chan)
                            }
                        }
                    }
                }
            }
        }).expect("Thread spawning failed");
    }

    fn save_context_state(&mut self) {
        self.saved_states.push(self.state.clone());
    }

    fn restore_context_state(&mut self) {
        if let Some(state) = self.saved_states.pop() {
            mem::replace(&mut self.state, state);
            self.drawtarget.set_transform(&self.state.transform);
            self.drawtarget.pop_clip();
        }
    }

    fn fill_text(&self, text: String, x: f64, y: f64, max_width: Option<f64>) {
        error!("Unimplemented canvas2d.fillText. Values received: {}, {}, {}, {:?}.", text, x, y, max_width);
    }

    fn fill_rect(&self, rect: &Rect<f32>) {
        if is_zero_size_gradient(&self.state.fill_style) {
            return; // Paint nothing if gradient size is zero.
        }

        let draw_rect = Rect::new(rect.origin,
            match self.state.fill_style {
                Pattern::Surface(ref surface) => {
                    let surface_size = surface.size();
                    match (surface.repeat_x, surface.repeat_y) {
                        (true, true) => rect.size,
                        (true, false) => Size2D::new(rect.size.width, surface_size.height as f32),
                        (false, true) => Size2D::new(surface_size.width as f32, rect.size.height),
                        (false, false) => Size2D::new(surface_size.width as f32, surface_size.height as f32),
                    }
                },
                _ => rect.size,
            }
        );

        if self.need_to_draw_shadow() {
            self.draw_with_shadow(&draw_rect, |new_draw_target: &DrawTarget| {
                new_draw_target.fill_rect(&draw_rect, self.state.fill_style.to_pattern_ref(),
                                          Some(&self.state.draw_options));
            });
        } else {
            self.drawtarget.fill_rect(&draw_rect, self.state.fill_style.to_pattern_ref(),
                                      Some(&self.state.draw_options));
        }
    }

    fn clear_rect(&self, rect: &Rect<f32>) {
        self.drawtarget.clear_rect(rect);
    }

    fn stroke_rect(&self, rect: &Rect<f32>) {
        if is_zero_size_gradient(&self.state.stroke_style) {
            return; // Paint nothing if gradient size is zero.
        }

        if self.need_to_draw_shadow() {
            self.draw_with_shadow(&rect, |new_draw_target: &DrawTarget| {
                new_draw_target.stroke_rect(rect, self.state.stroke_style.to_pattern_ref(),
                                            &self.state.stroke_opts, &self.state.draw_options);
            });
        } else if rect.size.width == 0. || rect.size.height == 0. {
            let cap = match self.state.stroke_opts.line_join {
                JoinStyle::Round => CapStyle::Round,
                _ => CapStyle::Butt
            };

            let stroke_opts =
                StrokeOptions::new(self.state.stroke_opts.line_width,
                                   self.state.stroke_opts.line_join,
                                   cap,
                                   self.state.stroke_opts.miter_limit,
                                   self.state.stroke_opts.mDashPattern);
            self.drawtarget.stroke_line(rect.origin, rect.bottom_right(),
                                        self.state.stroke_style.to_pattern_ref(),
                                        &stroke_opts, &self.state.draw_options);
        } else {
            self.drawtarget.stroke_rect(rect, self.state.stroke_style.to_pattern_ref(),
                                        &self.state.stroke_opts, &self.state.draw_options);
        }
    }

    fn begin_path(&mut self) {
        self.path_builder = self.drawtarget.create_path_builder()
    }

    fn close_path(&self) {
        self.path_builder.close()
    }

    fn fill(&self) {
        if is_zero_size_gradient(&self.state.fill_style) {
            return; // Paint nothing if gradient size is zero.
        }

        self.drawtarget.fill(&self.path_builder.finish(),
                             self.state.fill_style.to_pattern_ref(),
                             &self.state.draw_options);
    }

    fn stroke(&self) {
        if is_zero_size_gradient(&self.state.stroke_style) {
            return; // Paint nothing if gradient size is zero.
        }

        self.drawtarget.stroke(&self.path_builder.finish(),
                               self.state.stroke_style.to_pattern_ref(),
                               &self.state.stroke_opts,
                               &self.state.draw_options);
    }

    fn clip(&self) {
        self.drawtarget.push_clip(&self.path_builder.finish());
    }

    fn is_point_in_path(&mut self, x: f64, y: f64,
                        _fill_rule: FillRule, chan: IpcSender<bool>) {
        let path = self.path_builder.finish();
        let result = path.contains_point(x, y, &self.state.transform);
        self.path_builder = path.copy_to_builder();
        chan.send(result).unwrap();
    }

    fn draw_image(&self, image_data: Vec<u8>, image_size: Size2D<f64>,
                  dest_rect: Rect<f64>, source_rect: Rect<f64>, smoothing_enabled: bool) {
        // We round up the floating pixel values to draw the pixels
        let source_rect = source_rect.ceil();
        // It discards the extra pixels (if any) that won't be painted
        let image_data = crop_image(image_data, image_size, source_rect);

        if self.need_to_draw_shadow() {
            let rect = Rect::new(Point2D::new(dest_rect.origin.x as f32, dest_rect.origin.y as f32),
                                 Size2D::new(dest_rect.size.width as f32, dest_rect.size.height as f32));

            self.draw_with_shadow(&rect, |new_draw_target: &DrawTarget| {
                write_image(&new_draw_target, image_data, source_rect.size, dest_rect,
                            smoothing_enabled, self.state.draw_options.composition,
                            self.state.draw_options.alpha);
            });
        } else {
            write_image(&self.drawtarget, image_data, source_rect.size, dest_rect,
                        smoothing_enabled, self.state.draw_options.composition,
                        self.state.draw_options.alpha);
        }
    }

    fn draw_image_self(&self, image_size: Size2D<f64>,
                       dest_rect: Rect<f64>, source_rect: Rect<f64>,
                       smoothing_enabled: bool) {
        // Reads pixels from source image
        // In this case source and target are the same canvas
        let image_data = self.read_pixels(source_rect.to_i32(), image_size);

        if self.need_to_draw_shadow() {
            let rect = Rect::new(Point2D::new(dest_rect.origin.x as f32, dest_rect.origin.y as f32),
                                 Size2D::new(dest_rect.size.width as f32, dest_rect.size.height as f32));

            self.draw_with_shadow(&rect, |new_draw_target: &DrawTarget| {
                write_image(&new_draw_target, image_data, source_rect.size, dest_rect,
                            smoothing_enabled, self.state.draw_options.composition,
                            self.state.draw_options.alpha);
            });
        } else {
            // Writes on target canvas
            write_image(&self.drawtarget, image_data, image_size, dest_rect,
                        smoothing_enabled, self.state.draw_options.composition,
                        self.state.draw_options.alpha);
        }
    }

    fn draw_image_in_other(&self,
                           renderer: IpcSender<CanvasMsg>,
                           image_size: Size2D<f64>,
                           dest_rect: Rect<f64>,
                           source_rect: Rect<f64>,
                           smoothing_enabled: bool,
                           sender: IpcSender<()>) {
        let mut image_data = self.read_pixels(source_rect.to_i32(), image_size);
        // TODO: avoid double byte_swap.
        byte_swap(&mut image_data);

        let msg = CanvasMsg::Canvas2d(Canvas2dMsg::DrawImage(
            image_data, source_rect.size, dest_rect, source_rect, smoothing_enabled));
        renderer.send(msg).unwrap();
        // We acknowledge to the caller here that the data was sent to the
        // other canvas so that if JS immediately afterwards try to get the
        // pixels of the other one, it won't retrieve the other values.
        sender.send(()).unwrap();
    }

    fn move_to(&self, point: &Point2D<AzFloat>) {
        self.path_builder.move_to(*point)
    }

    fn line_to(&self, point: &Point2D<AzFloat>) {
        self.path_builder.line_to(*point)
    }

    fn rect(&self, rect: &Rect<f32>) {
        self.path_builder.move_to(Point2D::new(rect.origin.x, rect.origin.y));
        self.path_builder.line_to(Point2D::new(rect.origin.x + rect.size.width, rect.origin.y));
        self.path_builder.line_to(Point2D::new(rect.origin.x + rect.size.width,
                                               rect.origin.y + rect.size.height));
        self.path_builder.line_to(Point2D::new(rect.origin.x, rect.origin.y + rect.size.height));
        self.path_builder.close();
    }

    fn quadratic_curve_to(&self,
                          cp: &Point2D<AzFloat>,
                          endpoint: &Point2D<AzFloat>) {
        self.path_builder.quadratic_curve_to(cp, endpoint)
    }

    fn bezier_curve_to(&self,
                       cp1: &Point2D<AzFloat>,
                       cp2: &Point2D<AzFloat>,
                       endpoint: &Point2D<AzFloat>) {
        self.path_builder.bezier_curve_to(cp1, cp2, endpoint)
    }

    fn arc(&self,
           center: &Point2D<AzFloat>,
           radius: AzFloat,
           start_angle: AzFloat,
           end_angle: AzFloat,
           ccw: bool) {
        self.path_builder.arc(*center, radius, start_angle, end_angle, ccw)
    }

    fn arc_to(&self,
              cp1: &Point2D<AzFloat>,
              cp2: &Point2D<AzFloat>,
              radius: AzFloat) {
        let cp0 = self.path_builder.get_current_point();
        let cp1 = *cp1;
        let cp2 = *cp2;

        if (cp0.x == cp1.x && cp0.y == cp1.y) || cp1 == cp2 || radius == 0.0 {
            self.line_to(&cp1);
            return;
        }

        // if all three control points lie on a single straight line,
        // connect the first two by a straight line
        let direction = (cp2.x - cp1.x) * (cp0.y - cp1.y) + (cp2.y - cp1.y) * (cp1.x - cp0.x);
        if direction == 0.0 {
            self.line_to(&cp1);
            return;
        }

        // otherwise, draw the Arc
        let a2 = (cp0.x - cp1.x).powi(2) + (cp0.y - cp1.y).powi(2);
        let b2 = (cp1.x - cp2.x).powi(2) + (cp1.y - cp2.y).powi(2);
        let d = {
            let c2 = (cp0.x - cp2.x).powi(2) + (cp0.y - cp2.y).powi(2);
            let cosx = (a2 + b2 - c2) / (2.0 * (a2 * b2).sqrt());
            let sinx = (1.0 - cosx.powi(2)).sqrt();
            radius / ((1.0 - cosx) / sinx)
        };

        // first tangent point
        let anx = (cp1.x - cp0.x) / a2.sqrt();
        let any = (cp1.y - cp0.y) / a2.sqrt();
        let tp1 = Point2D::new(cp1.x - anx * d, cp1.y - any * d);

        // second tangent point
        let bnx = (cp1.x - cp2.x) / b2.sqrt();
        let bny = (cp1.y - cp2.y) / b2.sqrt();
        let tp2 = Point2D::new(cp1.x - bnx * d, cp1.y - bny * d);

        // arc center and angles
        let anticlockwise = direction < 0.0;
        let cx = tp1.x + any * radius * if anticlockwise { 1.0 } else { -1.0 };
        let cy = tp1.y - anx * radius * if anticlockwise { 1.0 } else { -1.0 };
        let angle_start = (tp1.y - cy).atan2(tp1.x - cx);
        let angle_end = (tp2.y - cy).atan2(tp2.x - cx);

        self.line_to(&tp1);
        if [cx, cy, angle_start, angle_end].iter().all(|x| x.is_finite()) {
            self.arc(&Point2D::new(cx, cy), radius,
                     angle_start, angle_end, anticlockwise);
        }
    }

    fn ellipse(&mut self,
           center: &Point2D<AzFloat>,
           radius_x: AzFloat,
           radius_y: AzFloat,
           rotation_angle: AzFloat,
           start_angle: AzFloat,
           end_angle: AzFloat,
           ccw: bool) {
        self.path_builder.ellipse(*center, radius_x, radius_y, rotation_angle, start_angle, end_angle, ccw);
    }

    fn set_fill_style(&mut self, style: FillOrStrokeStyle) {
        if let Some(pattern) = style.to_azure_pattern(&self.drawtarget) {
            self.state.fill_style = pattern
        }
    }

    fn set_stroke_style(&mut self, style: FillOrStrokeStyle) {
        if let Some(pattern) = style.to_azure_pattern(&self.drawtarget) {
            self.state.stroke_style = pattern
        }
    }

    fn set_line_width(&mut self, width: f32) {
        self.state.stroke_opts.line_width = width;
    }

    fn set_line_cap(&mut self, cap: LineCapStyle) {
        self.state.stroke_opts.line_cap = cap.to_azure_style();
    }

    fn set_line_join(&mut self, join: LineJoinStyle) {
        self.state.stroke_opts.line_join = join.to_azure_style();
    }

    fn set_miter_limit(&mut self, limit: f32) {
        self.state.stroke_opts.miter_limit = limit;
    }

    fn set_transform(&mut self, transform: &Transform2D<f32>) {
        self.state.transform = transform.clone();
        self.drawtarget.set_transform(transform)
    }

    fn set_global_alpha(&mut self, alpha: f32) {
        self.state.draw_options.alpha = alpha;
    }

    fn set_global_composition(&mut self, op: CompositionOrBlending) {
        self.state.draw_options.set_composition_op(op.to_azure_style());
    }

    fn create(size: Size2D<i32>) -> DrawTarget {
        DrawTarget::new(BackendType::Skia, size, SurfaceFormat::B8G8R8A8)
    }

    fn recreate(&mut self, size: Size2D<i32>) {
        // TODO: clear the thread state. https://github.com/servo/servo/issues/17533
        self.drawtarget = CanvasPaintThread::create(size);
        self.state = CanvasPaintState::new(self.state.draw_options.antialias);
        self.saved_states.clear();
        // Webrender doesn't let images change size, so we clear the webrender image key.
        // TODO: there is an annying race condition here: the display list builder
        // might still be using the old image key. Really, we should be scheduling the image
        // for later deletion, not deleting it immediately.
        // https://github.com/servo/servo/issues/17534
        if let Some(image_key) = self.image_key.take() {
            // If this executes, then we are in a new epoch since we last recreated the canvas,
            // so `old_image_key` must be `None`.
            debug_assert!(self.old_image_key.is_none());
            self.old_image_key = Some(image_key);
        }
    }

    fn send_pixels(&mut self, chan: IpcSender<Option<Vec<u8>>>) {
        self.drawtarget.snapshot().get_data_surface().with_data(|element| {
            chan.send(Some(element.into())).unwrap();
        })
    }

    fn send_data(&mut self, chan: IpcSender<CanvasImageData>) {
        self.drawtarget.snapshot().get_data_surface().with_data(|element| {
            let size = self.drawtarget.get_size();

            let descriptor = webrender_api::ImageDescriptor {
                width: size.width as u32,
                height: size.height as u32,
                stride: None,
                format: webrender_api::ImageFormat::BGRA8,
                offset: 0,
                is_opaque: false,
            };
            let data = webrender_api::ImageData::Raw(Arc::new(element.into()));

            let mut updates = webrender_api::ResourceUpdates::new();

            match self.image_key {
                Some(image_key) => {
                    debug!("Updating image {:?}.", image_key);
                    updates.update_image(image_key,
                                         descriptor,
                                         data,
                                         None);
                }
                None => {
                    self.image_key = Some(self.webrender_api.generate_image_key());
                    debug!("New image {:?}.", self.image_key);
                    updates.add_image(self.image_key.unwrap(),
                                      descriptor,
                                      data,
                                      None);
                }
            }

            if let Some(image_key) = mem::replace(&mut self.very_old_image_key, self.old_image_key.take()) {
                updates.delete_image(image_key);
            }

            self.webrender_api.update_resources(updates);

            let data = CanvasImageData {
                image_key: self.image_key.unwrap(),
            };
            chan.send(data).unwrap();
        })
    }

    fn image_data(&self, dest_rect: Rect<i32>, canvas_size: Size2D<f64>, chan: IpcSender<Vec<u8>>) {
        let mut dest_data = self.read_pixels(dest_rect, canvas_size);

        // bgra -> rgba
        byte_swap(&mut dest_data);
        chan.send(dest_data).unwrap();
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-putimagedata
    fn put_image_data(&mut self, imagedata: Vec<u8>,
                      offset: Vector2D<f64>,
                      image_data_size: Size2D<f64>,
                      mut dirty_rect: Rect<f64>) {
        if image_data_size.width <= 0.0 || image_data_size.height <= 0.0 {
            return
        }

        assert!(image_data_size.width * image_data_size.height * 4.0 == imagedata.len() as f64);

        // Step 1. TODO (neutered data)

        // Step 2.
        if dirty_rect.size.width < 0.0f64 {
            dirty_rect.origin.x += dirty_rect.size.width;
            dirty_rect.size.width = -dirty_rect.size.width;
        }

        if dirty_rect.size.height < 0.0f64 {
            dirty_rect.origin.y += dirty_rect.size.height;
            dirty_rect.size.height = -dirty_rect.size.height;
        }

        // Step 3.
        if dirty_rect.origin.x < 0.0f64 {
            dirty_rect.size.width += dirty_rect.origin.x;
            dirty_rect.origin.x = 0.0f64;
        }

        if dirty_rect.origin.y < 0.0f64 {
            dirty_rect.size.height += dirty_rect.origin.y;
            dirty_rect.origin.y = 0.0f64;
        }

        // Step 4.
        if dirty_rect.max_x() > image_data_size.width {
            dirty_rect.size.width = image_data_size.width - dirty_rect.origin.x;
        }

        if dirty_rect.max_y() > image_data_size.height {
            dirty_rect.size.height = image_data_size.height - dirty_rect.origin.y;
        }

        // 5) If either dirtyWidth or dirtyHeight is negative or zero,
        // stop without affecting any bitmaps
        if dirty_rect.size.width <= 0.0 || dirty_rect.size.height <= 0.0 {
            return
        }

        // Step 6.
        let dest_rect = dirty_rect.translate(&offset).to_i32();

        // azure_hl operates with integers. We need to cast the image size
        let image_size = image_data_size.to_i32();

        let first_pixel = dest_rect.origin - offset.to_i32();
        let mut src_line = (first_pixel.y * (image_size.width * 4) + first_pixel.x * 4) as usize;

        let mut dest =
            Vec::with_capacity((dest_rect.size.width * dest_rect.size.height * 4) as usize);

        for _ in 0 .. dest_rect.size.height {
            let mut src_offset = src_line;
            for _ in 0 .. dest_rect.size.width {
                let alpha = imagedata[src_offset + 3] as u16;
                // add 127 before dividing for more accurate rounding
                let premultiply_channel = |channel: u8| (((channel as u16 * alpha) + 127) / 255) as u8;
                dest.push(premultiply_channel(imagedata[src_offset + 2]));
                dest.push(premultiply_channel(imagedata[src_offset + 1]));
                dest.push(premultiply_channel(imagedata[src_offset + 0]));
                dest.push(imagedata[src_offset + 3]);
                src_offset += 4;
            }
            src_line += (image_size.width * 4) as usize;
        }

        if let Some(source_surface) = self.drawtarget.create_source_surface_from_data(
                &dest,
                dest_rect.size,
                dest_rect.size.width * 4,
                SurfaceFormat::B8G8R8A8) {
            self.drawtarget.copy_surface(source_surface,
                                         Rect::new(Point2D::new(0, 0), dest_rect.size),
                                         dest_rect.origin);
        }
    }

    fn set_shadow_offset_x(&mut self, value: f64) {
        self.state.shadow_offset_x = value;
    }

    fn set_shadow_offset_y(&mut self, value: f64) {
        self.state.shadow_offset_y = value;
    }

    fn set_shadow_blur(&mut self, value: f64) {
        self.state.shadow_blur = value;
    }

    fn set_shadow_color(&mut self, value: Color) {
        self.state.shadow_color = value;
    }

    // https://html.spec.whatwg.org/multipage/#when-shadows-are-drawn
    fn need_to_draw_shadow(&self) -> bool {
        self.state.shadow_color.a != 0.0f32 &&
        (self.state.shadow_offset_x != 0.0f64 ||
         self.state.shadow_offset_y != 0.0f64 ||
         self.state.shadow_blur != 0.0f64)
    }

    fn create_draw_target_for_shadow(&self, source_rect: &Rect<f32>) -> DrawTarget {
        let draw_target = self.drawtarget.create_similar_draw_target(&Size2D::new(source_rect.size.width as i32,
                                                                                  source_rect.size.height as i32),
                                                                     self.drawtarget.get_format());
        let matrix = Transform2D::identity()
            .pre_translate(-source_rect.origin.to_vector().cast().unwrap())
            .pre_mul(&self.state.transform);
        draw_target.set_transform(&matrix);
        draw_target
    }

    fn draw_with_shadow<F>(&self, rect: &Rect<f32>, draw_shadow_source: F)
        where F: FnOnce(&DrawTarget)
    {
        let shadow_src_rect = self.state.transform.transform_rect(rect);
        let new_draw_target = self.create_draw_target_for_shadow(&shadow_src_rect);
        draw_shadow_source(&new_draw_target);
        self.drawtarget.draw_surface_with_shadow(new_draw_target.snapshot(),
                                                 &Point2D::new(shadow_src_rect.origin.x as AzFloat,
                                                               shadow_src_rect.origin.y as AzFloat),
                                                 &self.state.shadow_color,
                                                 &Vector2D::new(self.state.shadow_offset_x as AzFloat,
                                                                self.state.shadow_offset_y as AzFloat),
                                                 (self.state.shadow_blur / 2.0f64) as AzFloat,
                                                 self.state.draw_options.composition);
    }
}

impl<'a> Drop for CanvasPaintThread<'a> {
    fn drop(&mut self) {
        let mut updates = webrender_api::ResourceUpdates::new();

        if let Some(image_key) = self.old_image_key.take() {
            updates.delete_image(image_key);
        }
        if let Some(image_key) = self.very_old_image_key.take() {
            updates.delete_image(image_key);
        }

        self.webrender_api.update_resources(updates);
    }
}

/// Used by drawImage to get rid of the extra pixels of the image data that
/// won't be copied to the canvas
/// image_data: Color pixel data of the image
/// image_size: Image dimensions
/// crop_rect: It determines the area of the image we want to keep
fn crop_image(image_data: Vec<u8>,
              image_size: Size2D<f64>,
              crop_rect: Rect<f64>) -> Vec<u8>{
    // We're going to iterate over a pixel values array so we need integers
    let crop_rect = crop_rect.to_i32();
    let image_size = image_size.to_i32();
    // Assuming 4 bytes per pixel and row-major order for storage
    // (consecutive elements in a pixel row of the image are contiguous in memory)
    let stride = image_size.width * 4;
    let image_bytes_length = image_size.height * image_size.width * 4;
    let crop_area_bytes_length = crop_rect.size.height * crop_rect.size.width * 4;
    // If the image size is less or equal than the crop area we do nothing
    if image_bytes_length <= crop_area_bytes_length {
        return image_data;
    }

    let mut new_image_data = Vec::new();
    let mut src = (crop_rect.origin.y * stride + crop_rect.origin.x * 4) as usize;
    for _ in 0..crop_rect.size.height {
        let row = &image_data[src .. src + (4 * crop_rect.size.width) as usize];
        new_image_data.extend_from_slice(row);
        src += stride as usize;
    }
    new_image_data
}

/// It writes an image to the destination target
/// draw_target: the destination target where the image_data will be copied
/// image_data: Pixel information of the image to be written. It takes RGBA8
/// image_size: The size of the image to be written
/// dest_rect: Area of the destination target where the pixels will be copied
/// smoothing_enabled: It determines if smoothing is applied to the image result
fn write_image(draw_target: &DrawTarget,
               mut image_data: Vec<u8>,
               image_size: Size2D<f64>,
               dest_rect: Rect<f64>,
               smoothing_enabled: bool,
               composition_op: CompositionOp,
               global_alpha: f32) {
    if image_data.is_empty() {
        return
    }
    let image_rect = Rect::new(Point2D::zero(), image_size);
    // rgba -> bgra
    byte_swap(&mut image_data);

    // From spec https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage
    // When scaling up, if the imageSmoothingEnabled attribute is set to true, the user agent should attempt
    // to apply a smoothing algorithm to the image data when it is scaled.
    // Otherwise, the image must be rendered using nearest-neighbor interpolation.
    let filter = if smoothing_enabled {
        Filter::Linear
    } else {
        Filter::Point
    };
    // azure_hl operates with integers. We need to cast the image size
    let image_size = image_size.to_i32();

    if let Some(source_surface) =
            draw_target.create_source_surface_from_data(&image_data,
                                                        image_size,
                                                        image_size.width * 4,
                                                        SurfaceFormat::B8G8R8A8) {
        let draw_surface_options = DrawSurfaceOptions::new(filter, true);
        let draw_options = DrawOptions::new(global_alpha, composition_op, AntialiasMode::None);

        draw_target.draw_surface(source_surface,
                                 dest_rect.to_azure_style(),
                                 image_rect.to_azure_style(),
                                 draw_surface_options,
                                 draw_options);
    }
}

fn is_zero_size_gradient(pattern: &Pattern) -> bool {
    if let &Pattern::LinearGradient(ref gradient) = pattern {
        if gradient.is_zero_size() {
            return true;
        }
    }
    false
}

pub trait PointToi32 {
    fn to_i32(&self) -> Point2D<i32>;
}

impl PointToi32 for Point2D<f64> {
    fn to_i32(&self) -> Point2D<i32> {
        Point2D::new(self.x.to_i32().unwrap(),
                     self.y.to_i32().unwrap())
    }
}

pub trait SizeToi32 {
    fn to_i32(&self) -> Size2D<i32>;
}

impl SizeToi32 for Size2D<f64> {
    fn to_i32(&self) -> Size2D<i32> {
        Size2D::new(self.width.to_i32().unwrap(),
                    self.height.to_i32().unwrap())
    }
}

pub trait RectToi32 {
    fn to_i32(&self) -> Rect<i32>;
    fn ceil(&self) -> Rect<f64>;
}

impl RectToi32 for Rect<f64> {
    fn to_i32(&self) -> Rect<i32> {
        Rect::new(Point2D::new(self.origin.x.to_i32().unwrap(),
                               self.origin.y.to_i32().unwrap()),
                  Size2D::new(self.size.width.to_i32().unwrap(),
                              self.size.height.to_i32().unwrap()))
    }

    fn ceil(&self) -> Rect<f64> {
        Rect::new(Point2D::new(self.origin.x.ceil(),
                               self.origin.y.ceil()),
                  Size2D::new(self.size.width.ceil(),
                              self.size.height.ceil()))
    }

}

pub trait ToAzureStyle {
    type Target;
    fn to_azure_style(self) -> Self::Target;
}

impl ToAzureStyle for Rect<f64> {
    type Target = Rect<AzFloat>;

    fn to_azure_style(self) -> Rect<AzFloat> {
        Rect::new(Point2D::new(self.origin.x as AzFloat, self.origin.y as AzFloat),
                  Size2D::new(self.size.width as AzFloat, self.size.height as AzFloat))
    }
}


impl ToAzureStyle for LineCapStyle {
    type Target = CapStyle;

    fn to_azure_style(self) -> CapStyle {
        match self {
            LineCapStyle::Butt => CapStyle::Butt,
            LineCapStyle::Round => CapStyle::Round,
            LineCapStyle::Square => CapStyle::Square,
        }
    }
}

impl ToAzureStyle for LineJoinStyle {
    type Target = JoinStyle;

    fn to_azure_style(self) -> JoinStyle {
        match self {
            LineJoinStyle::Round => JoinStyle::Round,
            LineJoinStyle::Bevel => JoinStyle::Bevel,
            LineJoinStyle::Miter => JoinStyle::Miter,
        }
    }
}

impl ToAzureStyle for CompositionStyle {
    type Target = CompositionOp;

    fn to_azure_style(self) -> CompositionOp {
        match self {
            CompositionStyle::SrcIn    => CompositionOp::In,
            CompositionStyle::SrcOut   => CompositionOp::Out,
            CompositionStyle::SrcOver  => CompositionOp::Over,
            CompositionStyle::SrcAtop  => CompositionOp::Atop,
            CompositionStyle::DestIn   => CompositionOp::DestIn,
            CompositionStyle::DestOut  => CompositionOp::DestOut,
            CompositionStyle::DestOver => CompositionOp::DestOver,
            CompositionStyle::DestAtop => CompositionOp::DestAtop,
            CompositionStyle::Copy     => CompositionOp::Source,
            CompositionStyle::Lighter  => CompositionOp::Add,
            CompositionStyle::Xor      => CompositionOp::Xor,
        }
    }
}

impl ToAzureStyle for BlendingStyle {
    type Target = CompositionOp;

    fn to_azure_style(self) -> CompositionOp {
        match self {
            BlendingStyle::Multiply   => CompositionOp::Multiply,
            BlendingStyle::Screen     => CompositionOp::Screen,
            BlendingStyle::Overlay    => CompositionOp::Overlay,
            BlendingStyle::Darken     => CompositionOp::Darken,
            BlendingStyle::Lighten    => CompositionOp::Lighten,
            BlendingStyle::ColorDodge => CompositionOp::ColorDodge,
            BlendingStyle::ColorBurn  => CompositionOp::ColorBurn,
            BlendingStyle::HardLight  => CompositionOp::HardLight,
            BlendingStyle::SoftLight  => CompositionOp::SoftLight,
            BlendingStyle::Difference => CompositionOp::Difference,
            BlendingStyle::Exclusion  => CompositionOp::Exclusion,
            BlendingStyle::Hue        => CompositionOp::Hue,
            BlendingStyle::Saturation => CompositionOp::Saturation,
            BlendingStyle::Color      => CompositionOp::Color,
            BlendingStyle::Luminosity => CompositionOp::Luminosity,
        }
    }
}

impl ToAzureStyle for CompositionOrBlending {
    type Target = CompositionOp;

    fn to_azure_style(self) -> CompositionOp {
        match self {
            CompositionOrBlending::Composition(op) => op.to_azure_style(),
            CompositionOrBlending::Blending(op) => op.to_azure_style(),
        }
    }
}

pub trait ToAzurePattern {
    fn to_azure_pattern(&self, drawtarget: &DrawTarget) -> Option<Pattern>;
}

impl ToAzurePattern for FillOrStrokeStyle {
    fn to_azure_pattern(&self, drawtarget: &DrawTarget) -> Option<Pattern> {
        match *self {
            FillOrStrokeStyle::Color(ref color) => {
                Some(Pattern::Color(ColorPattern::new(color.to_azure_style())))
            },
            FillOrStrokeStyle::LinearGradient(ref linear_gradient_style) => {
                let gradient_stops: Vec<GradientStop> = linear_gradient_style.stops.iter().map(|s| {
                    GradientStop {
                        offset: s.offset as AzFloat,
                        color: s.color.to_azure_style()
                    }
                }).collect();

                Some(Pattern::LinearGradient(LinearGradientPattern::new(
                    &Point2D::new(linear_gradient_style.x0 as AzFloat, linear_gradient_style.y0 as AzFloat),
                    &Point2D::new(linear_gradient_style.x1 as AzFloat, linear_gradient_style.y1 as AzFloat),
                    drawtarget.create_gradient_stops(&gradient_stops, ExtendMode::Clamp),
                    &Transform2D::identity())))
            },
            FillOrStrokeStyle::RadialGradient(ref radial_gradient_style) => {
                let gradient_stops: Vec<GradientStop> = radial_gradient_style.stops.iter().map(|s| {
                    GradientStop {
                        offset: s.offset as AzFloat,
                        color: s.color.to_azure_style()
                    }
                }).collect();

                Some(Pattern::RadialGradient(RadialGradientPattern::new(
                    &Point2D::new(radial_gradient_style.x0 as AzFloat, radial_gradient_style.y0 as AzFloat),
                    &Point2D::new(radial_gradient_style.x1 as AzFloat, radial_gradient_style.y1 as AzFloat),
                    radial_gradient_style.r0 as AzFloat, radial_gradient_style.r1 as AzFloat,
                    drawtarget.create_gradient_stops(&gradient_stops, ExtendMode::Clamp),
                    &Transform2D::identity())))
            },
            FillOrStrokeStyle::Surface(ref surface_style) => {
                drawtarget.create_source_surface_from_data(&surface_style.surface_data,
                                                           surface_style.surface_size,
                                                           surface_style.surface_size.width * 4,
                                                           SurfaceFormat::B8G8R8A8)
                          .map(|source_surface| {
                    Pattern::Surface(SurfacePattern::new(
                        source_surface.azure_source_surface,
                        surface_style.repeat_x,
                        surface_style.repeat_y,
                        &Transform2D::identity()))
                    })
            }
        }
    }
}

impl ToAzureStyle for RGBA {
    type Target = Color;

    fn to_azure_style(self) -> Color {
        Color::rgba(self.red_f32() as AzFloat,
                    self.green_f32() as AzFloat,
                    self.blue_f32() as AzFloat,
                    self.alpha_f32() as AzFloat)
    }
}
