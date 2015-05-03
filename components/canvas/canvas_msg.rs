/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_paint_task::{FillOrStrokeStyle, LineCapStyle, LineJoinStyle, CompositionOrBlending};
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
    ClearRect(Rect<f32>),
    Clip,
    ClosePath,
    Fill,
    FillRect(Rect<f32>),
    GetImageData(Rect<f64>, Size2D<f64>, Sender<Vec<u8>>),
    LineTo(Point2D<f32>),
    MoveTo(Point2D<f32>),
    PutImageData(Vec<u8>, Rect<f64>, Option<Rect<f64>>),
    QuadraticCurveTo(Point2D<f32>, Point2D<f32>),
    Rect(Rect<f32>),
    RestoreContext,
    SaveContext,
    StrokeRect(Rect<f32>),
    Stroke,
    SetFillStyle(FillOrStrokeStyle),
    SetStrokeStyle(FillOrStrokeStyle),
    SetLineWidth(f32),
    SetLineCap(LineCapStyle),
    SetLineJoin(LineJoinStyle),
    SetMiterLimit(f32),
    SetGlobalAlpha(f32),
    SetGlobalComposition(CompositionOrBlending),
    SetTransform(Matrix2D<f32>),
}

#[derive(Clone)]
pub enum CanvasWebGLMsg {
    AttachShader(u32, u32),
    BindBuffer(u32, u32),
    BufferData(u32, Vec<f32>, u32),
    Clear(u32),
    ClearColor(f32, f32, f32, f32),
    CompileShader(u32),
    CreateBuffer(Sender<u32>),
    CreateProgram(Sender<u32>),
    CreateShader(u32, Sender<u32>),
    DrawArrays(u32, i32, i32),
    EnableVertexAttribArray(u32),
    GetAttribLocation(u32, String, Sender<i32>),
    GetShaderInfoLog(u32, Sender<String>),
    GetShaderParameter(u32, u32, Sender<i32>),
    GetUniformLocation(u32, String, Sender<u32>),
    LinkProgram(u32),
    ShaderSource(u32, Vec<String>),
    Uniform4fv(u32, Vec<f32>),
    UseProgram(u32),
    VertexAttribPointer2f(u32, i32, bool, i32, i64),
    Viewport(i32, i32, i32, i32),
}

#[derive(Clone)]
pub enum CanvasCommonMsg {
    Close,
    Recreate(Size2D<i32>),
    SendPixelContents(Sender<Vec<u8>>),
}
