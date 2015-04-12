use canvas_paint_task::FillOrStrokeStyle;
use geom::matrix2d::Matrix2D;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use std::sync::mpsc::{Sender};

#[derive(Clone)]
pub enum CanvasMsg {
    // 2dContext
    FillRect(Rect<f32>),
    ClearRect(Rect<f32>),
    StrokeRect(Rect<f32>),
    BeginPath,
    ClosePath,
    Fill,
    Stroke,
    DrawImage(Vec<u8>, Rect<i32>, Rect<i32>, bool),
    DrawImageSelf(Size2D<i32>, Rect<i32>, Rect<i32>, bool),
    MoveTo(Point2D<f32>),
    LineTo(Point2D<f32>),
    QuadraticCurveTo(Point2D<f32>, Point2D<f32>),
    BezierCurveTo(Point2D<f32>, Point2D<f32>, Point2D<f32>),
    Arc(Point2D<f32>, f32, f32, f32, bool),
    ArcTo(Point2D<f32>, Point2D<f32>, f32),
    SetFillStyle(FillOrStrokeStyle),
    SetStrokeStyle(FillOrStrokeStyle),
    SetLineWidth(f32),
    SetMiterLimit(f32),
    SetTransform(Matrix2D<f32>),
    SetGlobalAlpha(f32),
    SetTransform(Matrix2D<f32>),
    Recreate(Size2D<i32>),
    GetImageData(Rect<i32>, Size2D<i32>, Sender<Vec<u8>>),
    PutImageData(Vec<u8>, Rect<i32>, Option<Rect<i32>>),
    Close,
    // WebGL
    Clear(u32),
    ClearColor(f32, f32, f32, f32),
    Render,
    // Common
    SendPixelContents(Sender<Vec<u8>>),
}
