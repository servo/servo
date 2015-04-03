/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_paint_task::{FillOrStrokeStyle, LineCapStyle, LineJoinStyle};
use geom::matrix2d::Matrix2D;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use std::sync::mpsc::{Sender};

#[derive(Clone)]
pub enum CanvasMsg {
    Canvas2d(Canvas2dMsg),
    Common(CanvasCommonMsg),
    WebGL(CanvasWebGLMsg),
}

#[derive(Clone)]
pub enum Canvas2dMsg {
    Arc(Point2D<f32>, f32, f32, f32, bool),
    ArcTo(Point2D<f32>, Point2D<f32>, f32),
    DrawImage(Vec<u8>, Size2D<f64>, Rect<f64>, Rect<f64>, bool),
    DrawImageSelf(Size2D<f64>, Rect<f64>, Rect<f64>, bool),
    BeginPath,
    BezierCurveTo(Point2D<f32>, Point2D<f32>, Point2D<f32>),
    ClosePath,
    ClearRect(Rect<f32>),
    Fill,
    FillRect(Rect<f32>),
    GetImageData(Rect<f64>, Size2D<f64>, Sender<Vec<u8>>),
    LineTo(Point2D<f32>),
    MoveTo(Point2D<f32>),
    PutImageData(Vec<u8>, Rect<f64>, Option<Rect<f64>>),
    QuadraticCurveTo(Point2D<f32>, Point2D<f32>),
    StrokeRect(Rect<f32>),
    Stroke,
    SetFillStyle(FillOrStrokeStyle),
    SetStrokeStyle(FillOrStrokeStyle),
    SetLineWidth(f32),
    SetLineCap(LineCapStyle),
    SetLineJoin(LineJoinStyle),
    SetMiterLimit(f32),
    SetGlobalAlpha(f32),
    SetTransform(Matrix2D<f32>),
}

#[derive(Clone)]
pub enum CanvasWebGLMsg {
    Clear(u32),
    ClearColor(f32, f32, f32, f32),
}

#[derive(Clone)]
pub enum CanvasCommonMsg {
    Close,
    Recreate(Size2D<i32>),
    SendPixelContents(Sender<Vec<u8>>),
}
