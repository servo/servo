/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use azure::azure::AzFloat;
use azure::azure_hl::{DrawTarget, Color, SurfaceFormat, BackendType, StrokeOptions, DrawOptions};
use azure::azure_hl::{ColorPattern, PathBuilder, Pattern};
use geom::matrix2d::Matrix2D;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use util::task::spawn_named;

use cssparser::RGBA;
use std::borrow::ToOwned;
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
    BezierCurveTo(Point2D<f32>, Point2D<f32>, Point2D<f32>),
    SetFillStyle(FillOrStrokeStyle),
    SetStrokeStyle(FillOrStrokeStyle),
    SetTransform(Matrix2D<f32>),
    Recreate(Size2D<i32>),
    SendPixelContents(Sender<Vec<u8>>),
    Close,
}

pub struct CanvasPaintTask {
    drawtarget: DrawTarget,
    fill_style: Pattern,
    stroke_style: Pattern,
    stroke_opts: StrokeOptions,
    /// TODO(pcwalton): Support multiple paths.
    path_builder: PathBuilder,
    /// The current 2D transform matrix.
    transform: Matrix2D<f32>,
}

impl CanvasPaintTask {
    fn new(size: Size2D<i32>) -> CanvasPaintTask {
        let draw_target = CanvasPaintTask::create(size);
        let path_builder = draw_target.create_path_builder();
        CanvasPaintTask {
            drawtarget: draw_target,
            fill_style: Pattern::Color(ColorPattern::new(Color::new(0., 0., 0., 1.))),
            stroke_style: Pattern::Color(ColorPattern::new(Color::new(0., 0., 0., 1.))),
            stroke_opts: StrokeOptions::new(1.0, 1.0),
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
                    CanvasMsg::BezierCurveTo(ref cp1, ref cp2, ref pt) => {
                        painter.bezier_curve_to(cp1, cp2, pt)
                    }
                    CanvasMsg::SetFillStyle(style) => painter.set_fill_style(style),
                    CanvasMsg::SetStrokeStyle(style) => painter.set_stroke_style(style),
                    CanvasMsg::SetTransform(ref matrix) => painter.set_transform(matrix),
                    CanvasMsg::Recreate(size) => painter.recreate(size),
                    CanvasMsg::SendPixelContents(chan) => painter.send_pixel_contents(chan),
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

    fn bezier_curve_to(&self,
                       cp1: &Point2D<AzFloat>,
                       cp2: &Point2D<AzFloat>,
                       endpoint: &Point2D<AzFloat>) {
        self.path_builder.bezier_curve_to(cp1, cp2, endpoint)
    }

    fn set_fill_style(&mut self, style: FillOrStrokeStyle) {
        self.fill_style = style.to_azure_pattern()
    }

    fn set_stroke_style(&mut self, style: FillOrStrokeStyle) {
        self.stroke_style = style.to_azure_pattern()
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
}

#[derive(Clone)]
pub enum FillOrStrokeStyle {
    Color(RGBA),
}

impl FillOrStrokeStyle {
    fn to_azure_pattern(&self) -> Pattern {
        match *self {
            FillOrStrokeStyle::Color(ref color) => {
                Pattern::Color(ColorPattern::new(Color::new(color.red,
                                                            color.green,
                                                            color.blue,
                                                            color.alpha)))
            }
        }
    }
}

