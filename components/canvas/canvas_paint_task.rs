/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use azure::azure::AzFloat;
use azure::azure_hl::{DrawTarget, SurfaceFormat, BackendType, StrokeOptions, DrawOptions, Pattern};
use azure::azure_hl::{ColorPattern, PathBuilder, JoinStyle, CapStyle, DrawSurfaceOptions, Filter};
use azure::azure_hl::{GradientStop, LinearGradientPattern, RadialGradientPattern, ExtendMode};
use geom::matrix2d::Matrix2D;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use gfx::color;
use util::task::spawn_named;
use util::vec::byte_swap;

use cssparser::RGBA;
use std::borrow::ToOwned;
use std::ops::Add;
use std::sync::mpsc::{channel, Sender};

#[derive(Clone)]
pub enum CanvasMsg {
    FillRect(Rect<f32>),
    ClearRect(Rect<f32>),
    StrokeRect(Rect<f32>),
    BeginPath,
    ClosePath,
    Fill,
    MoveTo(Point2D<f32>),
    LineTo(Point2D<f32>),
    QuadraticCurveTo(Point2D<f32>, Point2D<f32>),
    BezierCurveTo(Point2D<f32>, Point2D<f32>, Point2D<f32>),
    Arc(Point2D<f32>, f32, f32, f32, bool),
    SetFillStyle(FillOrStrokeStyle),
    SetStrokeStyle(FillOrStrokeStyle),
    SetTransform(Matrix2D<f32>),
    Recreate(Size2D<i32>),
    SendPixelContents(Sender<Vec<u8>>),
    GetImageData(Rect<i32>, Size2D<i32>, Sender<Vec<u8>>),
    PutImageData(Vec<u8>, Rect<i32>, Option<Rect<i32>>, Size2D<i32>),
    Close,
}

pub struct CanvasPaintTask<'a> {
    drawtarget: DrawTarget,
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
            fill_style: Pattern::Color(ColorPattern::new(color::black())),
            stroke_style: Pattern::Color(ColorPattern::new(color::black())),
            stroke_opts: StrokeOptions::new(1.0, JoinStyle::MiterOrBevel, CapStyle::Butt, 1.0, &[]),
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
                    CanvasMsg::FillRect(ref rect) => painter.fill_rect(rect),
                    CanvasMsg::StrokeRect(ref rect) => painter.stroke_rect(rect),
                    CanvasMsg::ClearRect(ref rect) => painter.clear_rect(rect),
                    CanvasMsg::BeginPath => painter.begin_path(),
                    CanvasMsg::ClosePath => painter.close_path(),
                    CanvasMsg::Fill => painter.fill(),
                    CanvasMsg::MoveTo(ref point) => painter.move_to(point),
                    CanvasMsg::LineTo(ref point) => painter.line_to(point),
                    CanvasMsg::QuadraticCurveTo(ref cp, ref pt) => {
                        painter.quadratic_curve_to(cp, pt)
                    }
                    CanvasMsg::BezierCurveTo(ref cp1, ref cp2, ref pt) => {
                        painter.bezier_curve_to(cp1, cp2, pt)
                    }
                    CanvasMsg::Arc(ref center, radius, start, end, ccw) => {
                        painter.arc(center, radius, start, end, ccw)
                    }
                    CanvasMsg::SetFillStyle(style) => painter.set_fill_style(style),
                    CanvasMsg::SetStrokeStyle(style) => painter.set_stroke_style(style),
                    CanvasMsg::SetTransform(ref matrix) => painter.set_transform(matrix),
                    CanvasMsg::Recreate(size) => painter.recreate(size),
                    CanvasMsg::SendPixelContents(chan) => painter.send_pixel_contents(chan),
                    CanvasMsg::GetImageData(dest_rect, canvas_size, chan) => painter.get_image_data(dest_rect, canvas_size, chan),
                    CanvasMsg::PutImageData(imagedata, image_data_rect, dirty_rect, canvas_size)
                        => painter.put_image_data(imagedata, image_data_rect, dirty_rect, canvas_size),
                    CanvasMsg::Close => break,
                }
            }
        });
        chan
    }

    fn fill_rect(&self, rect: &Rect<f32>) {
        let drawopts = DrawOptions::new(1.0, 0);
        self.drawtarget.fill_rect(rect, self.fill_style.to_pattern_ref(), Some(&drawopts));
    }

    fn clear_rect(&self, rect: &Rect<f32>) {
        self.drawtarget.clear_rect(rect);
    }

    fn stroke_rect(&self, rect: &Rect<f32>) {
        let drawopts = DrawOptions::new(1.0, 0);
        match self.stroke_style {
            Pattern::Color(ref color) => {
                self.drawtarget.stroke_rect(rect, color, &self.stroke_opts, &drawopts)
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
        let draw_options = DrawOptions::new(1.0, 0);
        match self.fill_style {
            Pattern::Color(ref color) => {
                self.drawtarget.fill(&self.path_builder.finish(), color, &draw_options);
            }
            _ => {
                // TODO(pcwalton)
            }
        };
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

    fn set_fill_style(&mut self, style: FillOrStrokeStyle) {
        self.fill_style = style.to_azure_pattern(&self.drawtarget)
    }

    fn set_stroke_style(&mut self, style: FillOrStrokeStyle) {
        self.stroke_style = style.to_azure_pattern(&self.drawtarget)
    }

    fn set_transform(&mut self, transform: &Matrix2D<f32>) {
        self.transform = *transform;
        self.drawtarget.set_transform(transform)
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

    fn get_image_data(&self, mut dest_rect: Rect<i32>, canvas_size: Size2D<i32>, chan: Sender<Vec<u8>>) {
        if dest_rect.size.width < 0 {
            dest_rect.size.width = -dest_rect.size.width;
            dest_rect.origin.x -= dest_rect.size.width;
        }
        if dest_rect.size.height < 0 {
            dest_rect.size.height = -dest_rect.size.height;
            dest_rect.origin.y -= dest_rect.size.height;
        }
        if dest_rect.size.width == 0 {
            dest_rect.size.width = 1;
        }
        if dest_rect.size.height == 0 {
            dest_rect.size.height = 1;
        }

        let canvas_rect = Rect(Point2D(0i32, 0i32), canvas_size);
        let src_read_rect = canvas_rect.intersection(&dest_rect).unwrap_or(Rect::zero());

        let mut dest_data = Vec::new();
        //load the canvas data to the source vector
        if !src_read_rect.is_empty() && canvas_size.width != 0 && canvas_size.height != 0 {
            let data_surface = self.drawtarget.snapshot().get_data_surface();
            let mut src_data = Vec::new();
            data_surface.with_data(|element| {
                src_data = element.to_vec();
            });

            let stride = data_surface.stride();

            //start offset of the copyable rectangle
            let mut src = (src_read_rect.origin.y * stride + src_read_rect.origin.x * 4) as usize;
            //copy the data to the destination vector
            for _ in 0..src_read_rect.size.height {
                let row = &src_data[src .. src + (4 * src_read_rect.size.width) as usize];
                dest_data.push_all(row);
                src += stride as usize;
            }
        }
        // bgra -> rgba
        byte_swap(dest_data.as_mut_slice());
        chan.send(dest_data).unwrap();
    }

    fn put_image_data(&mut self, mut imagedata: Vec<u8>, image_data_rect: Rect<i32>,
                      dirty_rect: Option<Rect<i32>>, canvas_size: Size2D<i32>) {

        if image_data_rect.size.width <= 0 || image_data_rect.size.height <= 0 {
            return
        }

        assert!(image_data_rect.size.width * image_data_rect.size.height * 4 == imagedata.len() as i32);
        // rgba -> bgra
        byte_swap(imagedata.as_mut_slice());

        let new_image_data_rect = Rect(Point2D(0i32, 0i32),
            Size2D(image_data_rect.size.width, image_data_rect.size.height));

        let new_dirty_rect = match dirty_rect {
            Some(mut dirty_rect) => {
                if dirty_rect.size.width < 0 {
                    dirty_rect.origin.x = dirty_rect.origin.x + dirty_rect.size.width;
                    dirty_rect.size.width = -dirty_rect.size.width;
                }
                if dirty_rect.size.height < 0 {
                    dirty_rect.origin.y = dirty_rect.origin.y + dirty_rect.size.height;
                    dirty_rect.size.height = -dirty_rect.size.height;
                }
                new_image_data_rect.intersection(&dirty_rect)
            },
            None => Some(new_image_data_rect)
        };

        if let Some(new_dirty_rect) = new_dirty_rect {
            let moved_dirty_rect = Rect(new_dirty_rect.origin.add(image_data_rect.origin),
                                        new_dirty_rect.size).intersection(&Rect(Point2D(0i32, 0i32),
                                        canvas_size)).unwrap_or(Rect::zero());
            if moved_dirty_rect.is_empty() {
                return
            }

            let source_surface = self.drawtarget.create_source_surface_from_data(&imagedata,
                image_data_rect.size, image_data_rect.size.width * 4, SurfaceFormat::B8G8R8A8);

            let draw_surface_options = DrawSurfaceOptions::new(Filter::Linear, true);
            let draw_options = DrawOptions::new(1.0f64 as AzFloat, 0);

            self.drawtarget.draw_surface(source_surface,
                Rect(Point2D(moved_dirty_rect.origin.x as AzFloat, moved_dirty_rect.origin.y as AzFloat),
                     Size2D(moved_dirty_rect.size.width as AzFloat, moved_dirty_rect.size.height as AzFloat)),
                Rect(Point2D((moved_dirty_rect.origin.x - image_data_rect.origin.x) as AzFloat,
                             (moved_dirty_rect.origin.y - image_data_rect.origin.y) as AzFloat),
                     Size2D(moved_dirty_rect.size.width as AzFloat, moved_dirty_rect.size.height as AzFloat)),
                draw_surface_options, draw_options);
        }
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
