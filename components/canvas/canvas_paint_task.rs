/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use azure::azure::AzFloat;
use azure::azure_hl::{DrawTarget, SurfaceFormat, BackendType, StrokeOptions, DrawOptions, Pattern};
use azure::azure_hl::{ColorPattern, PathBuilder, JoinStyle, CapStyle, DrawSurfaceOptions, Filter};
use azure::azure_hl::{GradientStop, LinearGradientPattern, RadialGradientPattern, ExtendMode};
use canvas_msg::{CanvasMsg, Canvas2dMsg, CanvasCommonMsg};
use geom::matrix2d::Matrix2D;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use gfx::color;
use util::task::spawn_named;
use util::vec::byte_swap;

use cssparser::RGBA;
use std::borrow::ToOwned;
use std::num::{Float, ToPrimitive};
use std::sync::mpsc::{channel, Sender};

impl<'a> CanvasPaintTask<'a> {
    /// It reads image data from the canvas
    /// canvas_size: The size of the canvas we're reading from
    /// read_rect: The area of the canvas we want to read from
    fn read_pixels(&self, read_rect: Rect<f64>, canvas_size: Size2D<f64>) -> Vec<u8>{
        let read_rect = read_rect.to_i32();
        let canvas_size = canvas_size.to_i32();
        let canvas_rect = Rect(Point2D(0i32, 0i32), canvas_size);
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

    /// It writes image data to the canvas
    /// source_rect: the area of the image data to be written
    /// dest_rect: The area of the canvas where the imagedata will be copied
    /// smoothing_enabled: if smoothing is applied to the copied pixels
    fn write_pixels(&self, imagedata: &[u8],
                    image_size: Size2D<f64>,
                    source_rect: Rect<f64>,
                    dest_rect: Rect<f64>,
                    smoothing_enabled: bool) {
        // From spec https://html.spec.whatwg.org/multipage/scripting.html#dom-context-2d-drawimage
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

        let source_surface = self.drawtarget.create_source_surface_from_data(
            &imagedata,
            image_size, image_size.width * 4, SurfaceFormat::B8G8R8A8);

        let draw_surface_options = DrawSurfaceOptions::new(filter, true);
        let draw_options = DrawOptions::new(self.draw_options.alpha, 0);

        self.drawtarget.draw_surface(source_surface,
                                     dest_rect.to_azfloat(),
                                     source_rect.to_azfloat(),
                                     draw_surface_options, draw_options);
    }

    /// dirty_rect: original dirty_rect provided by the putImageData call
    /// image_data_rect: the area of the image to be copied
    /// Result: It retuns the modified dirty_rect by the rules described in
    /// the spec https://html.spec.whatwg.org/#dom-context-2d-putimagedata
    fn calculate_dirty_rect(&self,
                            mut dirty_rect: Rect<f64>,
                            image_data_rect: Rect<f64>) -> Rect<f64>{
        // 1) If dirtyWidth is negative,
        // let dirtyX be dirtyX+dirtyWidth,
        // and let dirtyWidth be equal to the absolute magnitude of dirtyWidth.
        if dirty_rect.size.width < 0.0f64 {
            dirty_rect.origin.x = dirty_rect.origin.x + dirty_rect.size.width;
            dirty_rect.size.width = -dirty_rect.size.width;
        }

        // 2) If dirtyHeight is negative, let dirtyY be dirtyY+dirtyHeight,
        // and let dirtyHeight be equal to the absolute magnitude of dirtyHeight.
        if dirty_rect.size.height < 0.0f64 {
            dirty_rect.origin.y = dirty_rect.origin.y + dirty_rect.size.height;
            dirty_rect.size.height = -dirty_rect.size.height;
        }

        // 3) If dirtyX is negative, let dirtyWidth be dirtyWidth+dirtyX, and let dirtyX be zero.
        if dirty_rect.origin.x < 0.0f64 {
            dirty_rect.size.width += dirty_rect.origin.x;
            dirty_rect.origin.x = 0.0f64;
        }

        // 3) If dirtyY is negative, let dirtyHeight be dirtyHeight+dirtyY, and let dirtyY be zero.
        if dirty_rect.origin.y < 0.0f64 {
            dirty_rect.size.height += dirty_rect.origin.y;
            dirty_rect.origin.y = 0.0f64;
        }

        // 4) If dirtyX+dirtyWidth is greater than the width attribute of the imagedata argument,
        // let dirtyWidth be the value of that width attribute, minus the value of dirtyX.
        if dirty_rect.origin.x + dirty_rect.size.width > image_data_rect.size.width {
            dirty_rect.size.width = image_data_rect.size.width - dirty_rect.origin.x;
        }

        // 4) If dirtyY+dirtyHeight is greater than the height attribute of the imagedata argument,
        // let dirtyHeight be the value of that height attribute, minus the value of dirtyY.
        if dirty_rect.origin.y + dirty_rect.size.height > image_data_rect.size.height {
            dirty_rect.size.height = image_data_rect.size.height - dirty_rect.origin.y;
        }

        dirty_rect
    }

    /// It writes an image to the destination canvas
    /// imagedata: Pixel information of the image to be written. It takes RGBA8
    /// image_size: The size of the image to be written
    /// dest_rect: Area of the destination canvas where the pixels will be copied
    /// smoothing_enabled: It determines if smoothing is applied to the image result
    fn write_image(&self, mut imagedata: Vec<u8>,
                   image_size: Size2D<f64>, dest_rect: Rect<f64>, smoothing_enabled: bool) {
        if imagedata.len() == 0 {
            return
        }
        let image_rect = Rect(Point2D(0f64, 0f64), image_size);
        // rgba -> bgra
        byte_swap(imagedata.as_mut_slice());
        self.write_pixels(&imagedata, image_size, image_rect, dest_rect, smoothing_enabled);
    }

}

pub struct CanvasPaintTask<'a> {
    drawtarget: DrawTarget,
    draw_options: DrawOptions,
    fill_style: Pattern,
    stroke_style: Pattern,
    stroke_opts: StrokeOptions<'a>,
    /// TODO(pcwalton): Support multiple paths.
    path_builder: PathBuilder,
    /// The current 2D transform matrix.
    transform: Matrix2D<f32>,
}

impl<'a> CanvasPaintTask<'a> {
    fn new(size: Size2D<i32>) -> CanvasPaintTask<'a> {
        let draw_target = CanvasPaintTask::create(size);
        let path_builder = draw_target.create_path_builder();
        CanvasPaintTask {
            drawtarget: draw_target,
            draw_options: DrawOptions::new(1.0, 0),
            fill_style: Pattern::Color(ColorPattern::new(color::black())),
            stroke_style: Pattern::Color(ColorPattern::new(color::black())),
            stroke_opts: StrokeOptions::new(1.0, JoinStyle::MiterOrBevel, CapStyle::Butt, 10.0, &[]),
            path_builder: path_builder,
            transform: Matrix2D::identity(),
        }
    }

    pub fn start(size: Size2D<i32>) -> Sender<CanvasMsg> {
        let (chan, port) = channel::<CanvasMsg>();
        spawn_named("CanvasTask".to_owned(), move || {
            let mut painter = CanvasPaintTask::new(size);

            loop {
                match port.recv().unwrap() {
                    CanvasMsg::Canvas2d(message) => {
                        match message {
                            Canvas2dMsg::FillRect(ref rect) => painter.fill_rect(rect),
                            Canvas2dMsg::StrokeRect(ref rect) => painter.stroke_rect(rect),
                            Canvas2dMsg::ClearRect(ref rect) => painter.clear_rect(rect),
                            Canvas2dMsg::BeginPath => painter.begin_path(),
                            Canvas2dMsg::ClosePath => painter.close_path(),
                            Canvas2dMsg::Fill => painter.fill(),
                            Canvas2dMsg::Stroke => painter.stroke(),
                            Canvas2dMsg::DrawImage(imagedata, image_size, dest_rect, source_rect, smoothing_enabled) => {
                                painter.draw_image(imagedata, image_size, dest_rect, source_rect, smoothing_enabled)
                            }
                            Canvas2dMsg::DrawImageSelf(image_size, dest_rect, source_rect, smoothing_enabled) => {
                                painter.draw_image_self(image_size, dest_rect, source_rect, smoothing_enabled)
                            }
                            Canvas2dMsg::MoveTo(ref point) => painter.move_to(point),
                            Canvas2dMsg::LineTo(ref point) => painter.line_to(point),
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
                            Canvas2dMsg::SetFillStyle(style) => painter.set_fill_style(style),
                            Canvas2dMsg::SetStrokeStyle(style) => painter.set_stroke_style(style),
                            Canvas2dMsg::SetLineWidth(width) => painter.set_line_width(width),
                            Canvas2dMsg::SetLineCap(cap) => painter.set_line_cap(cap),
                            Canvas2dMsg::SetLineJoin(join) => painter.set_line_join(join),
                            Canvas2dMsg::SetMiterLimit(limit) => painter.set_miter_limit(limit),
                            Canvas2dMsg::SetTransform(ref matrix) => painter.set_transform(matrix),
                            Canvas2dMsg::SetGlobalAlpha(alpha) => painter.set_global_alpha(alpha),
                            Canvas2dMsg::GetImageData(dest_rect, canvas_size, chan) => painter.get_image_data(dest_rect, canvas_size, chan),
                            Canvas2dMsg::PutImageData(imagedata, image_data_rect, dirty_rect)
                                => painter.put_image_data(imagedata, image_data_rect, dirty_rect),
                        }
                    },
                    CanvasMsg::Common(message) => {
                        match message {
                            CanvasCommonMsg::Close => break,
                            CanvasCommonMsg::Recreate(size) => painter.recreate(size),
                            CanvasCommonMsg::SendPixelContents(chan) =>
                                painter.send_pixel_contents(chan),
                        }
                    },
                    CanvasMsg::WebGL(_) => panic!("Wrong message sent to Canvas2D task"),
                }
            }
        });
        chan
    }

    fn fill_rect(&self, rect: &Rect<f32>) {
        self.drawtarget.fill_rect(rect, self.fill_style.to_pattern_ref(), Some(&self.draw_options));
    }

    fn clear_rect(&self, rect: &Rect<f32>) {
        self.drawtarget.clear_rect(rect);
    }

    fn stroke_rect(&self, rect: &Rect<f32>) {
        match self.stroke_style {
            Pattern::Color(ref color) => {
                self.drawtarget.stroke_rect(rect, color, &self.stroke_opts, &self.draw_options)
            }
            _ => {
                // TODO(pcwalton)
            }
        };
    }

    fn begin_path(&mut self) {
        self.path_builder = self.drawtarget.create_path_builder()
    }

    fn close_path(&self) {
        self.path_builder.close()
    }

    fn fill(&self) {
        match self.fill_style {
            Pattern::Color(ref color) => {
                self.drawtarget.fill(&self.path_builder.finish(), color, &self.draw_options);
            }
            _ => {
                // TODO(pcwalton)
            }
        };
    }

    fn stroke(&self) {
        match self.stroke_style {
            Pattern::Color(ref color) => {
                self.drawtarget.stroke(&self.path_builder.finish(),
                                       color, &self.stroke_opts, &self.draw_options);
            }
            _ => {
                // TODO
            }
        };
    }

    fn draw_image(&self, image_data: Vec<u8>, image_size: Size2D<f64>,
                  dest_rect: Rect<f64>, source_rect: Rect<f64>, smoothing_enabled: bool) {
        // We round up the floating pixel values to draw the pixels
        let source_rect = source_rect.ceil();
        // It discards the extra pixels (if any) that won't be painted
        let image_data = crop_image(image_data, image_size, source_rect);
        self.write_image(image_data, source_rect.size, dest_rect, smoothing_enabled);
    }

    fn draw_image_self(&self, image_size: Size2D<f64>,
                       dest_rect: Rect<f64>, source_rect: Rect<f64>,
                       smoothing_enabled: bool) {
        // Reads pixels from source image
        // In this case source and target are the same canvas
        let imagedata = self.read_pixels(source_rect, image_size);
        // Writes on target canvas
        self.write_image(imagedata, image_size, dest_rect, smoothing_enabled);
    }

    fn move_to(&self, point: &Point2D<AzFloat>) {
        self.path_builder.move_to(*point)
    }

    fn line_to(&self, point: &Point2D<AzFloat>) {
        self.path_builder.line_to(*point)
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
        let tp1 = Point2D::<AzFloat>(cp1.x - anx * d, cp1.y - any * d);

        // second tangent point
        let bnx = (cp1.x - cp2.x) / b2.sqrt();
        let bny = (cp1.y - cp2.y) / b2.sqrt();
        let tp2 = Point2D::<AzFloat>(cp1.x - bnx * d, cp1.y - bny * d);

        // arc center and angles
        let anticlockwise = direction < 0.0;
        let cx = tp1.x + any * radius * if anticlockwise { 1.0 } else { -1.0 };
        let cy = tp1.y - anx * radius * if anticlockwise { 1.0 } else { -1.0 };
        let angle_start = (tp1.y - cy).atan2(tp1.x - cx);
        let angle_end = (tp2.y - cy).atan2(tp2.x - cx);

        self.line_to(&tp1);
        if [cx, cy, angle_start, angle_end].iter().all(|x| x.is_finite()) {
            self.arc(&Point2D::<AzFloat>(cx, cy), radius,
                     angle_start, angle_end, anticlockwise);
        }
    }

    fn set_fill_style(&mut self, style: FillOrStrokeStyle) {
        self.fill_style = style.to_azure_pattern(&self.drawtarget)
    }

    fn set_stroke_style(&mut self, style: FillOrStrokeStyle) {
        self.stroke_style = style.to_azure_pattern(&self.drawtarget)
    }

    fn set_line_width(&mut self, width: f32) {
        self.stroke_opts.line_width = width;
    }

    fn set_line_cap(&mut self, cap: LineCapStyle) {
        self.stroke_opts.line_cap = cap.to_azure_style();
    }

    fn set_line_join(&mut self, join: LineJoinStyle) {
        self.stroke_opts.line_join = join.to_azure_style();
    }

    fn set_miter_limit(&mut self, limit: f32) {
        self.stroke_opts.miter_limit = limit;
    }

    fn set_transform(&mut self, transform: &Matrix2D<f32>) {
        self.transform = *transform;
        self.drawtarget.set_transform(transform)
    }

    fn set_global_alpha(&mut self, alpha: f32) {
        self.draw_options.alpha = alpha;
    }

    fn create(size: Size2D<i32>) -> DrawTarget {
        DrawTarget::new(BackendType::Skia, size, SurfaceFormat::B8G8R8A8)
    }

    fn recreate(&mut self, size: Size2D<i32>) {
        self.drawtarget = CanvasPaintTask::create(size);
    }

    fn send_pixel_contents(&mut self, chan: Sender<Vec<u8>>) {
        self.drawtarget.snapshot().get_data_surface().with_data(|element| {
            chan.send(element.to_vec()).unwrap();
        })
    }

    fn get_image_data(&self, mut dest_rect: Rect<f64>, canvas_size: Size2D<f64>, chan: Sender<Vec<u8>>) {
        if dest_rect.size.width < 0.0 {
            dest_rect.size.width = -dest_rect.size.width;
            dest_rect.origin.x -= dest_rect.size.width;
        }
        if dest_rect.size.height < 0.0 {
            dest_rect.size.height = -dest_rect.size.height;
            dest_rect.origin.y -= dest_rect.size.height;
        }
        if dest_rect.size.width == 0.0 {
            dest_rect.size.width = 1.0;
        }
        if dest_rect.size.height == 0.0 {
            dest_rect.size.height = 1.0;
        }

        let mut dest_data = self.read_pixels(dest_rect, canvas_size);

        // bgra -> rgba
        byte_swap(dest_data.as_mut_slice());
        chan.send(dest_data).unwrap();
    }

    fn put_image_data(&mut self, mut imagedata: Vec<u8>,
                      image_data_rect: Rect<f64>,
                      dirty_rect: Option<Rect<f64>>) {

        if image_data_rect.size.width <= 0.0 || image_data_rect.size.height <= 0.0 {
            return
        }

        assert!(image_data_rect.size.width * image_data_rect.size.height * 4.0 == imagedata.len() as f64);
        // rgba -> bgra
        byte_swap(imagedata.as_mut_slice());

        let image_rect = Rect(Point2D(0f64, 0f64),
                               Size2D(image_data_rect.size.width, image_data_rect.size.height));

        // Dirty rectangle defines the area of the source image to be copied
        // on the destination canvas
        let source_rect = match dirty_rect {
            Some(dirty_rect) =>
                self.calculate_dirty_rect(dirty_rect, image_data_rect),
            // If no dirty area is provided we consider the whole source image
            // as the area to be copied to the canvas
            None => image_rect,
        };

        // 5) If either dirtyWidth or dirtyHeight is negative or zero,
        // stop without affecting any bitmaps
        if source_rect.size.width <= 0.0 || source_rect.size.height <= 0.0 {
            return
        }

        // 6) For all integer values of x and y where dirtyX ≤ x < dirty
        // X+dirtyWidth and dirtyY ≤ y < dirtyY+dirtyHeight, copy the
        // four channels of the pixel with coordinate (x, y) in the imagedata
        // data structure's Canvas Pixel ArrayBuffer to the pixel with coordinate
        // (dx+x, dy+y) in the rendering context's scratch bitmap.
        // It also clips the destination rectangle to the canvas area
        let dest_rect = Rect(
            Point2D(image_data_rect.origin.x + source_rect.origin.x,
                    image_data_rect.origin.y + source_rect.origin.y),
            Size2D(source_rect.size.width, source_rect.size.height));

        self.write_pixels(&imagedata, image_data_rect.size, source_rect, dest_rect, true)
    }
}

#[derive(Clone)]
pub struct CanvasGradientStop {
    pub offset: f64,
    pub color: RGBA,
}

#[derive(Clone)]
pub struct LinearGradientStyle {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
    pub stops: Vec<CanvasGradientStop>
}

impl LinearGradientStyle {
    pub fn new(x0: f64, y0: f64, x1: f64, y1: f64, stops: Vec<CanvasGradientStop>)
        -> LinearGradientStyle {
        LinearGradientStyle {
            x0: x0,
            y0: y0,
            x1: x1,
            y1: y1,
            stops: stops,
        }
    }
}

#[derive(Clone)]
pub struct RadialGradientStyle {
    pub x0: f64,
    pub y0: f64,
    pub r0: f64,
    pub x1: f64,
    pub y1: f64,
    pub r1: f64,
    pub stops: Vec<CanvasGradientStop>
}

impl RadialGradientStyle {
    pub fn new(x0: f64, y0: f64, r0: f64, x1: f64, y1: f64, r1: f64, stops: Vec<CanvasGradientStop>)
        -> RadialGradientStyle {
        RadialGradientStyle {
            x0: x0,
            y0: y0,
            r0: r0,
            x1: x1,
            y1: y1,
            r1: r1,
            stops: stops,
        }
    }
}

#[derive(Clone)]
pub enum FillOrStrokeStyle {
    Color(RGBA),
    LinearGradient(LinearGradientStyle),
    RadialGradient(RadialGradientStyle),
}

impl FillOrStrokeStyle {
    fn to_azure_pattern(&self, drawtarget: &DrawTarget) -> Pattern {
        match *self {
            FillOrStrokeStyle::Color(ref color) => {
                Pattern::Color(ColorPattern::new(color::new(color.red,
                                                            color.green,
                                                            color.blue,
                                                            color.alpha)))
            },
            FillOrStrokeStyle::LinearGradient(ref linear_gradient_style) => {
                let gradient_stops: Vec<GradientStop> = linear_gradient_style.stops.iter().map(|s| {
                    GradientStop {
                        offset: s.offset as AzFloat,
                        color: color::new(s.color.red, s.color.green, s.color.blue, s.color.alpha)
                    }
                }).collect();

                Pattern::LinearGradient(LinearGradientPattern::new(
                    &Point2D(linear_gradient_style.x0 as AzFloat, linear_gradient_style.y0 as AzFloat),
                    &Point2D(linear_gradient_style.x1 as AzFloat, linear_gradient_style.y1 as AzFloat),
                    drawtarget.create_gradient_stops(&gradient_stops, ExtendMode::Clamp),
                    &Matrix2D::identity()))
            },
            FillOrStrokeStyle::RadialGradient(ref radial_gradient_style) => {
                let gradient_stops: Vec<GradientStop> = radial_gradient_style.stops.iter().map(|s| {
                    GradientStop {
                        offset: s.offset as AzFloat,
                        color: color::new(s.color.red, s.color.green, s.color.blue, s.color.alpha)
                    }
                }).collect();

                Pattern::RadialGradient(RadialGradientPattern::new(
                    &Point2D(radial_gradient_style.x0 as AzFloat, radial_gradient_style.y0 as AzFloat),
                    &Point2D(radial_gradient_style.x1 as AzFloat, radial_gradient_style.y1 as AzFloat),
                    radial_gradient_style.r0 as AzFloat, radial_gradient_style.r1 as AzFloat,
                    drawtarget.create_gradient_stops(&gradient_stops, ExtendMode::Clamp),
                    &Matrix2D::identity()))
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum LineCapStyle {
    Butt = 0,
    Round = 1,
    Square = 2,
}

impl LineCapStyle {
    fn to_azure_style(&self) -> CapStyle {
        match *self {
            LineCapStyle::Butt => CapStyle::Butt,
            LineCapStyle::Round => CapStyle::Round,
            LineCapStyle::Square => CapStyle::Square,
        }
    }

    pub fn from_str(string: &str) -> Option<LineCapStyle> {
        match string {
            "butt" => Some(LineCapStyle::Butt),
            "round" => Some(LineCapStyle::Round),
            "square" => Some(LineCapStyle::Square),
            _ => None
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum LineJoinStyle {
    Round = 0,
    Bevel = 1,
    Miter = 2,
}

impl LineJoinStyle {
    fn to_azure_style(&self) -> JoinStyle {
        match *self {
            LineJoinStyle::Round => JoinStyle::Round,
            LineJoinStyle::Bevel => JoinStyle::Bevel,
            LineJoinStyle::Miter => JoinStyle::Miter,
        }
    }

    pub fn from_str(string: &str) -> Option<LineJoinStyle> {
        match string {
            "round" => Some(LineJoinStyle::Round),
            "bevel" => Some(LineJoinStyle::Bevel),
            "miter" => Some(LineJoinStyle::Miter),
            _ => None
        }
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

pub trait SizeToi32 {
    fn to_i32(&self) -> Size2D<i32>;
}

impl SizeToi32 for Size2D<f64> {
    fn to_i32(&self) -> Size2D<i32> {
        Size2D(self.width.to_i32().unwrap(),
               self.height.to_i32().unwrap())
    }
}

pub trait RectToi32 {
    fn to_i32(&self) -> Rect<i32>;
    fn ceil(&self) -> Rect<f64>;
}

impl RectToi32 for Rect<f64> {
    fn to_i32(&self) -> Rect<i32> {
        Rect(Point2D(self.origin.x.to_i32().unwrap(),
                     self.origin.y.to_i32().unwrap()),
             Size2D(self.size.width.to_i32().unwrap(),
                    self.size.height.to_i32().unwrap()))
    }

    fn ceil(&self) -> Rect<f64> {
        Rect(Point2D(self.origin.x.ceil(),
                     self.origin.y.ceil()),
             Size2D(self.size.width.ceil(),
                    self.size.height.ceil()))
    }

}

pub trait ToAzFloat {
    fn to_azfloat(&self) -> Rect<AzFloat>;
}

impl ToAzFloat for Rect<f64> {
    fn to_azfloat(&self) -> Rect<AzFloat> {
        Rect(Point2D(self.origin.x as AzFloat, self.origin.y as AzFloat),
             Size2D(self.size.width as AzFloat, self.size.height as AzFloat))
    }
}
