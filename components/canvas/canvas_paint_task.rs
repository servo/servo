/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use azure::azure::{AzFloat, AzColor};
use azure::azure_hl::{ColorPattern, PathBuilder, DrawSurfaceOptions, Filter};
use azure::azure_hl::{DrawTarget, SurfaceFormat, BackendType, StrokeOptions, DrawOptions, Pattern};
use azure::azure_hl::{JoinStyle, CapStyle, CompositionOp, AntialiasMode};
use canvas_traits::*;
use euclid::matrix2d::Matrix2D;
use euclid::point::Point2D;
use euclid::rect::Rect;
use euclid::size::Size2D;
use gfx_traits::color;
use ipc_channel::ipc::IpcSharedMemory;
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use layers::platform::surface::NativeSurface;
use num::ToPrimitive;
use util::opts;
use util::task::spawn_named;
use util::vec::byte_swap;

use std::borrow::ToOwned;
use std::mem;
use std::sync::mpsc::{channel, Sender};

impl<'a> CanvasPaintTask<'a> {
    /// It reads image data from the canvas
    /// canvas_size: The size of the canvas we're reading from
    /// read_rect: The area of the canvas we want to read from
    fn read_pixels(&self, read_rect: Rect<i32>, canvas_size: Size2D<f64>) -> Vec<u8>{
        let canvas_size = canvas_size.to_i32();
        let canvas_rect = Rect::new(Point2D::new(0i32, 0i32), canvas_size);
        let src_read_rect = canvas_rect.intersection(&read_rect).unwrap_or(Rect::zero());

        let mut image_data = Vec::new();
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
            image_data.push_all(row);
            src += stride as usize;
        }

        image_data
    }
}

pub struct CanvasPaintTask<'a> {
    drawtarget: DrawTarget,
    /// TODO(pcwalton): Support multiple paths.
    path_builder: PathBuilder,
    state: CanvasPaintState<'a>,
    saved_states: Vec<CanvasPaintState<'a>>,
}

#[derive(Clone)]
struct CanvasPaintState<'a> {
    draw_options: DrawOptions,
    fill_style: Pattern,
    stroke_style: Pattern,
    stroke_opts: StrokeOptions<'a>,
    /// The current 2D transform matrix.
    transform: Matrix2D<f32>,
    shadow_offset_x: f64,
    shadow_offset_y: f64,
    shadow_blur: f64,
    shadow_color: AzColor,
}

impl<'a> CanvasPaintState<'a> {
    fn new() -> CanvasPaintState<'a> {
        let antialias = if opts::get().enable_canvas_antialiasing {
            AntialiasMode::Default
        } else {
            AntialiasMode::None
        };

        CanvasPaintState {
            draw_options: DrawOptions::new(1.0, CompositionOp::Over, antialias),
            fill_style: Pattern::Color(ColorPattern::new(color::black())),
            stroke_style: Pattern::Color(ColorPattern::new(color::black())),
            stroke_opts: StrokeOptions::new(1.0, JoinStyle::MiterOrBevel, CapStyle::Butt, 10.0, &[]),
            transform: Matrix2D::identity(),
            shadow_offset_x: 0.0,
            shadow_offset_y: 0.0,
            shadow_blur: 0.0,
            shadow_color: color::transparent(),
        }
    }
}

impl<'a> CanvasPaintTask<'a> {
    fn new(size: Size2D<i32>) -> CanvasPaintTask<'a> {
        let draw_target = CanvasPaintTask::create(size);
        let path_builder = draw_target.create_path_builder();
        CanvasPaintTask {
            drawtarget: draw_target,
            path_builder: path_builder,
            state: CanvasPaintState::new(),
            saved_states: Vec::new(),
        }
    }

    /// Creates a new `CanvasPaintTask` and returns the out-of-process sender and the in-process
    /// sender for it.
    pub fn start(size: Size2D<i32>) -> (IpcSender<CanvasMsg>, Sender<CanvasMsg>) {
        // TODO(pcwalton): Ask the pipeline to create this for us instead of spawning it directly.
        // This will be needed for multiprocess Servo.
        let (out_of_process_chan, out_of_process_port) = ipc::channel::<CanvasMsg>().unwrap();
        let (in_process_chan, in_process_port) = channel();
        ROUTER.route_ipc_receiver_to_mpsc_sender(out_of_process_port, in_process_chan.clone());
        spawn_named("CanvasTask".to_owned(), move || {
            let mut painter = CanvasPaintTask::new(size);
            loop {
                let msg = in_process_port.recv();
                match msg.unwrap() {
                    CanvasMsg::Canvas2d(message) => {
                        match message {
                            Canvas2dMsg::FillRect(ref rect) => painter.fill_rect(rect),
                            Canvas2dMsg::StrokeRect(ref rect) => painter.stroke_rect(rect),
                            Canvas2dMsg::ClearRect(ref rect) => painter.clear_rect(rect),
                            Canvas2dMsg::BeginPath => painter.begin_path(),
                            Canvas2dMsg::ClosePath => painter.close_path(),
                            Canvas2dMsg::Fill => painter.fill(),
                            Canvas2dMsg::Stroke => painter.stroke(),
                            Canvas2dMsg::Clip => painter.clip(),
                            Canvas2dMsg::DrawImage(imagedata, image_size, dest_rect, source_rect,
                                                   smoothing_enabled) => {
                                painter.draw_image(imagedata, image_size, dest_rect, source_rect, smoothing_enabled)
                            }
                            Canvas2dMsg::DrawImageSelf(image_size, dest_rect, source_rect, smoothing_enabled) => {
                                painter.draw_image_self(image_size, dest_rect, source_rect, smoothing_enabled)
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
                                => painter.get_image_data(dest_rect, canvas_size, chan),
                            Canvas2dMsg::PutImageData(imagedata, offset, image_data_size, dirty_rect)
                                => painter.put_image_data(imagedata, offset, image_data_size, dirty_rect),
                            Canvas2dMsg::SetShadowOffsetX(value) => painter.set_shadow_offset_x(value),
                            Canvas2dMsg::SetShadowOffsetY(value) => painter.set_shadow_offset_y(value),
                            Canvas2dMsg::SetShadowBlur(value) => painter.set_shadow_blur(value),
                            Canvas2dMsg::SetShadowColor(ref color) => painter.set_shadow_color(color.to_azcolor()),
                        }
                    },
                    CanvasMsg::Common(message) => {
                        match message {
                            CanvasCommonMsg::Close => break,
                            CanvasCommonMsg::Recreate(size) => painter.recreate(size),
                        }
                    },
                    CanvasMsg::FromLayout(message) => {
                        match message {
                            FromLayoutMsg::SendPixelContents(chan) => {
                                painter.send_pixel_contents(chan)
                            }
                        }
                    }
                    CanvasMsg::FromPaint(message) => {
                        match message {
                            FromPaintMsg::SendNativeSurface(chan) => {
                                painter.send_native_surface(chan)
                            }
                        }
                    }
                    CanvasMsg::WebGL(_) => panic!("Wrong message sent to Canvas2D task"),
                }
            }
        });

        (out_of_process_chan, in_process_chan)
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

    fn set_fill_style(&mut self, style: FillOrStrokeStyle) {
        self.state.fill_style = style.to_azure_pattern(&self.drawtarget)
    }

    fn set_stroke_style(&mut self, style: FillOrStrokeStyle) {
        self.state.stroke_style = style.to_azure_pattern(&self.drawtarget)
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

    fn set_transform(&mut self, transform: &Matrix2D<f32>) {
        self.state.transform = *transform;
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
        self.drawtarget = CanvasPaintTask::create(size);
    }

    fn send_pixel_contents(&mut self, chan: IpcSender<IpcSharedMemory>) {
        self.drawtarget.snapshot().get_data_surface().with_data(|element| {
            chan.send(IpcSharedMemory::from_bytes(element)).unwrap();
        })
    }

    fn send_native_surface(&self, _chan: Sender<NativeSurface>) {
        // FIXME(mrobinson): We need a handle on the NativeDisplay to create compatible
        // NativeSurfaces for the compositor.
        unimplemented!()
    }

    fn get_image_data(&self,
                      dest_rect: Rect<i32>,
                      canvas_size: Size2D<f64>,
                      chan: IpcSender<Vec<u8>>) {
        let mut dest_data = self.read_pixels(dest_rect, canvas_size);

        // bgra -> rgba
        byte_swap(&mut dest_data);
        chan.send(dest_data).unwrap();
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-putimagedata
    fn put_image_data(&mut self, imagedata: Vec<u8>,
                      offset: Point2D<f64>,
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
                // Premultiply alpha and swap RGBA -> BGRA.
                // TODO: may want a precomputed premultiply table to make this fast. (#6969)
                let alpha = imagedata[src_offset + 3] as f32 / 255.;
                dest.push((imagedata[src_offset + 2] as f32 * alpha) as u8);
                dest.push((imagedata[src_offset + 1] as f32 * alpha) as u8);
                dest.push((imagedata[src_offset + 0] as f32 * alpha) as u8);
                dest.push(imagedata[src_offset + 3]);
                src_offset += 4;
            }
            src_line += (image_size.width * 4) as usize;
        }

        let source_surface = self.drawtarget.create_source_surface_from_data(
            &dest,
            dest_rect.size, dest_rect.size.width * 4, SurfaceFormat::B8G8R8A8);

        self.drawtarget.copy_surface(source_surface,
                                     Rect::new(Point2D::new(0, 0), dest_rect.size),
                                     dest_rect.origin);
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

    fn set_shadow_color(&mut self, value: AzColor) {
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
        let matrix = Matrix2D::identity().translate(-source_rect.origin.x as AzFloat,
                                                    -source_rect.origin.y as AzFloat)
                                         .mul(&self.state.transform);
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
                                                 &Point2D::new(self.state.shadow_offset_x as AzFloat,
                                                               self.state.shadow_offset_y as AzFloat),
                                                 (self.state.shadow_blur / 2.0f64) as AzFloat,
                                                 self.state.draw_options.composition);
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
    let crop_area_bytes_length = crop_rect.size.height * crop_rect.size.height * 4;
    // If the image size is less or equal than the crop area we do nothing
    if image_bytes_length <= crop_area_bytes_length {
        return image_data;
    }

    let mut new_image_data = Vec::new();
    let mut src = (crop_rect.origin.y * stride + crop_rect.origin.x * 4) as usize;
    for _ in (0..crop_rect.size.height) {
        let row = &image_data[src .. src + (4 * crop_rect.size.width) as usize];
        new_image_data.push_all(row);
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

    let source_surface = draw_target.create_source_surface_from_data(
        &image_data,
        image_size, image_size.width * 4, SurfaceFormat::B8G8R8A8);

    let draw_surface_options = DrawSurfaceOptions::new(filter, true);
    let draw_options = DrawOptions::new(global_alpha, composition_op, AntialiasMode::None);

    draw_target.draw_surface(source_surface,
                             dest_rect.to_azfloat(),
                             image_rect.to_azfloat(),
                             draw_surface_options,
                             draw_options);
}

fn is_zero_size_gradient(pattern: &Pattern) -> bool {
    if let &Pattern::LinearGradient(ref gradient) = pattern {
        if gradient.is_zero_size() {
            return true;
        }
    }
    return false;
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

pub trait ToAzFloat {
    fn to_azfloat(&self) -> Rect<AzFloat>;
}

impl ToAzFloat for Rect<f64> {
    fn to_azfloat(&self) -> Rect<AzFloat> {
        Rect::new(Point2D::new(self.origin.x as AzFloat, self.origin.y as AzFloat),
                  Size2D::new(self.size.width as AzFloat, self.size.height as AzFloat))
    }
}
