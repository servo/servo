/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::default::Default;
use std::str::FromStr;

use euclid::default::{Point2D, Rect, Size2D, Transform2D};
use ipc_channel::ipc::{IpcBytesReceiver, IpcSender};
use malloc_size_of_derive::MallocSizeOf;
use pixels::IpcSnapshot;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use strum::{Display, EnumString};
use style::color::AbsoluteColor;
use style::properties::style_structs::Font as FontStyleStruct;

#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum PathSegment {
    ClosePath,
    MoveTo {
        x: f32,
        y: f32,
    },
    LineTo {
        x: f32,
        y: f32,
    },
    Quadratic {
        cpx: f32,
        cpy: f32,
        x: f32,
        y: f32,
    },
    Bezier {
        cp1x: f32,
        cp1y: f32,
        cp2x: f32,
        cp2y: f32,
        x: f32,
        y: f32,
    },
    ArcTo {
        cp1x: f32,
        cp1y: f32,
        cp2x: f32,
        cp2y: f32,
        radius: f32,
    },
    Ellipse {
        x: f32,
        y: f32,
        radius_x: f32,
        radius_y: f32,
        rotation: f32,
        start_angle: f32,
        end_angle: f32,
        anticlockwise: bool,
    },
    SvgArc {
        radius_x: f32,
        radius_y: f32,
        rotation: f32,
        large_arc: bool,
        sweep: bool,
        x: f32,
        y: f32,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum FillRule {
    Nonzero,
    Evenodd,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
pub struct CanvasId(pub u64);

#[derive(Debug, Deserialize, Serialize)]
pub enum CanvasMsg {
    Canvas2d(Canvas2dMsg, CanvasId),
    FromScript(FromScriptMsg, CanvasId),
    Recreate(Option<Size2D<u64>>, CanvasId),
    Close(CanvasId),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Canvas2dMsg {
    Arc(Point2D<f32>, f32, f32, f32, bool),
    ArcTo(Point2D<f32>, Point2D<f32>, f32),
    DrawImage(IpcSnapshot, Rect<f64>, Rect<f64>, bool),
    DrawEmptyImage(Size2D<u32>, Rect<f64>, Rect<f64>),
    DrawImageInOther(CanvasId, Size2D<u32>, Rect<f64>, Rect<f64>, bool),
    BeginPath,
    BezierCurveTo(Point2D<f32>, Point2D<f32>, Point2D<f32>),
    ClearRect(Rect<f32>),
    Clip,
    ClipPath(Vec<PathSegment>),
    ClosePath,
    Ellipse(Point2D<f32>, f32, f32, f32, f32, f32, bool),
    Fill(FillOrStrokeStyle),
    FillPath(FillOrStrokeStyle, Vec<PathSegment>),
    FillText(String, f64, f64, Option<f64>, FillOrStrokeStyle, bool),
    FillRect(Rect<f32>, FillOrStrokeStyle),
    GetImageData(Rect<u32>, Size2D<u32>, IpcSender<IpcSnapshot>),
    GetTransform(IpcSender<Transform2D<f32>>),
    IsPointInCurrentPath(f64, f64, FillRule, IpcSender<bool>),
    IsPointInPath(Vec<PathSegment>, f64, f64, FillRule, IpcSender<bool>),
    LineTo(Point2D<f32>),
    MoveTo(Point2D<f32>),
    MeasureText(String, IpcSender<TextMetrics>),
    PutImageData(Rect<u32>, IpcBytesReceiver),
    QuadraticCurveTo(Point2D<f32>, Point2D<f32>),
    Rect(Rect<f32>),
    RestoreContext,
    SaveContext,
    StrokeRect(Rect<f32>, FillOrStrokeStyle),
    Stroke(FillOrStrokeStyle),
    StrokePath(FillOrStrokeStyle, Vec<PathSegment>),
    SetLineWidth(f32),
    SetLineCap(LineCapStyle),
    SetLineJoin(LineJoinStyle),
    SetMiterLimit(f32),
    SetLineDash(Vec<f32>),
    SetLineDashOffset(f32),
    SetGlobalAlpha(f32),
    SetGlobalComposition(CompositionOrBlending),
    SetTransform(Transform2D<f32>),
    SetShadowOffsetX(f64),
    SetShadowOffsetY(f64),
    SetShadowBlur(f64),
    SetShadowColor(AbsoluteColor),
    SetFont(FontStyleStruct),
    SetTextAlign(TextAlign),
    SetTextBaseline(TextBaseline),
    UpdateImage(IpcSender<()>),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum FromScriptMsg {
    SendPixels(IpcSender<IpcSnapshot>),
}

#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct CanvasGradientStop {
    pub offset: f64,
    pub color: AbsoluteColor,
}

#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct LinearGradientStyle {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
    pub stops: Vec<CanvasGradientStop>,
}

impl LinearGradientStyle {
    pub fn new(
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
        stops: Vec<CanvasGradientStop>,
    ) -> LinearGradientStyle {
        LinearGradientStyle {
            x0,
            y0,
            x1,
            y1,
            stops,
        }
    }
}

#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct RadialGradientStyle {
    pub x0: f64,
    pub y0: f64,
    pub r0: f64,
    pub x1: f64,
    pub y1: f64,
    pub r1: f64,
    pub stops: Vec<CanvasGradientStop>,
}

impl RadialGradientStyle {
    pub fn new(
        x0: f64,
        y0: f64,
        r0: f64,
        x1: f64,
        y1: f64,
        r1: f64,
        stops: Vec<CanvasGradientStop>,
    ) -> RadialGradientStyle {
        RadialGradientStyle {
            x0,
            y0,
            r0,
            x1,
            y1,
            r1,
            stops,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SurfaceStyle {
    pub surface_data: ByteBuf,
    pub surface_size: Size2D<u32>,
    pub repeat_x: bool,
    pub repeat_y: bool,
}

impl SurfaceStyle {
    pub fn new(
        surface_data: Vec<u8>,
        surface_size: Size2D<u32>,
        repeat_x: bool,
        repeat_y: bool,
    ) -> Self {
        Self {
            surface_data: ByteBuf::from(surface_data),
            surface_size,
            repeat_x,
            repeat_y,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum FillOrStrokeStyle {
    Color(AbsoluteColor),
    LinearGradient(LinearGradientStyle),
    RadialGradient(RadialGradientStyle),
    Surface(SurfaceStyle),
}

#[derive(
    Clone, Copy, Debug, Display, Deserialize, EnumString, MallocSizeOf, PartialEq, Serialize,
)]
pub enum LineCapStyle {
    Butt = 0,
    Round = 1,
    Square = 2,
}

#[derive(
    Clone, Copy, Debug, Deserialize, Display, EnumString, MallocSizeOf, PartialEq, Serialize,
)]
pub enum LineJoinStyle {
    Round = 0,
    Bevel = 1,
    Miter = 2,
}

#[derive(Clone, Copy, Debug, Deserialize, Display, EnumString, PartialEq, Serialize)]
#[strum(serialize_all = "kebab-case")]
pub enum RepetitionStyle {
    Repeat,
    RepeatX,
    RepeatY,
    NoRepeat,
}

/// <https://drafts.fxtf.org/compositing/#compositemode>
#[derive(
    Clone, Copy, Debug, Deserialize, Display, EnumString, MallocSizeOf, PartialEq, Serialize,
)]
#[strum(serialize_all = "kebab-case")]
pub enum CompositionStyle {
    Clear,
    Copy,
    SourceOver,
    DestinationOver,
    SourceIn,
    DestinationIn,
    SourceOut,
    DestinationOut,
    SourceAtop,
    DestinationAtop,
    Xor,
    Lighter,
    // PlusDarker,
    // PlusLighter,
}

/// <https://drafts.fxtf.org/compositing/#ltblendmodegt>
#[derive(
    Clone, Copy, Debug, Deserialize, Display, EnumString, MallocSizeOf, PartialEq, Serialize,
)]
#[strum(serialize_all = "kebab-case")]
pub enum BlendingStyle {
    // Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum CompositionOrBlending {
    Composition(CompositionStyle),
    Blending(BlendingStyle),
}

impl Default for CompositionOrBlending {
    fn default() -> CompositionOrBlending {
        CompositionOrBlending::Composition(CompositionStyle::SourceOver)
    }
}

impl FromStr for CompositionOrBlending {
    type Err = ();

    fn from_str(string: &str) -> Result<CompositionOrBlending, ()> {
        if let Ok(op) = CompositionStyle::from_str(string) {
            return Ok(CompositionOrBlending::Composition(op));
        }

        if let Ok(op) = BlendingStyle::from_str(string) {
            return Ok(CompositionOrBlending::Blending(op));
        }

        Err(())
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    Display,
    EnumString,
    MallocSizeOf,
    PartialEq,
    Serialize,
)]
pub enum TextAlign {
    #[default]
    Start,
    End,
    Left,
    Right,
    Center,
}

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    Display,
    EnumString,
    MallocSizeOf,
    PartialEq,
    Serialize,
)]
pub enum TextBaseline {
    Top,
    Hanging,
    Middle,
    #[default]
    Alphabetic,
    Ideographic,
    Bottom,
}

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    Display,
    EnumString,
    MallocSizeOf,
    PartialEq,
    Serialize,
)]
pub enum Direction {
    Ltr,
    Rtl,
    #[default]
    Inherit,
}

#[derive(Clone, Debug, Default, Deserialize, MallocSizeOf, Serialize)]
pub struct TextMetrics {
    pub width: f32,
    pub actual_boundingbox_left: f32,
    pub actual_boundingbox_right: f32,
    pub actual_boundingbox_ascent: f32,
    pub actual_boundingbox_descent: f32,
    pub font_boundingbox_ascent: f32,
    pub font_boundingbox_descent: f32,
    pub em_height_ascent: f32,
    pub em_height_descent: f32,
    pub hanging_baseline: f32,
    pub alphabetic_baseline: f32,
    pub ideographic_baseline: f32,
}
