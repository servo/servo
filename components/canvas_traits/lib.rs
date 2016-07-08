/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![crate_name = "canvas_traits"]
#![crate_type = "rlib"]
#![feature(custom_derive)]
#![feature(plugin)]
#![plugin(heapsize_plugin, plugins, serde_macros)]

#![deny(unsafe_code)]

extern crate azure;
extern crate core;
extern crate cssparser;
extern crate euclid;
extern crate gfx_traits;
extern crate heapsize;
extern crate ipc_channel;
extern crate serde;
extern crate webrender_traits;

use azure::azure::{AzColor, AzFloat};
use azure::azure_hl::{CapStyle, CompositionOp, JoinStyle};
use azure::azure_hl::{ColorPattern, DrawTarget, Pattern};
use azure::azure_hl::{ExtendMode, GradientStop, LinearGradientPattern, RadialGradientPattern};
use azure::azure_hl::{SurfaceFormat, SurfacePattern};
use cssparser::RGBA;
use euclid::matrix2d::Matrix2D;
use euclid::point::Point2D;
use euclid::rect::Rect;
use euclid::size::Size2D;
use gfx_traits::color;
use ipc_channel::ipc::{IpcSender, IpcSharedMemory};
use std::default::Default;
use std::str::FromStr;
use webrender_traits::{WebGLCommand, WebGLContextId};

#[derive(Clone, Deserialize, Serialize)]
pub enum FillRule {
    Nonzero,
    Evenodd,
}

#[derive(Clone, Deserialize, Serialize)]
pub enum CanvasMsg {
    Canvas2d(Canvas2dMsg),
    Common(CanvasCommonMsg),
    FromLayout(FromLayoutMsg),
    WebGL(WebGLCommand),
}

#[derive(Clone, Deserialize, Serialize)]
pub enum CanvasCommonMsg {
    Close,
    Recreate(Size2D<i32>),
}

#[derive(Clone, Deserialize, Serialize)]
pub enum CanvasData {
    Pixels(CanvasPixelData),
    WebGL(WebGLContextId),
}

#[derive(Clone, Deserialize, Serialize)]
pub struct CanvasPixelData {
    pub image_data: IpcSharedMemory,
    pub image_key: Option<webrender_traits::ImageKey>,
}

#[derive(Clone, Deserialize, Serialize)]
pub enum FromLayoutMsg {
    SendData(IpcSender<CanvasData>),
}

#[derive(Clone, Deserialize, Serialize)]
pub enum Canvas2dMsg {
    Arc(Point2D<f32>, f32, f32, f32, bool),
    ArcTo(Point2D<f32>, Point2D<f32>, f32),
    DrawImage(Vec<u8>, Size2D<f64>, Rect<f64>, Rect<f64>, bool),
    DrawImageSelf(Size2D<f64>, Rect<f64>, Rect<f64>, bool),
    DrawImageInOther(
        IpcSender<CanvasMsg>, Size2D<f64>, Rect<f64>, Rect<f64>, bool, IpcSender<()>),
    BeginPath,
    BezierCurveTo(Point2D<f32>, Point2D<f32>, Point2D<f32>),
    ClearRect(Rect<f32>),
    Clip,
    ClosePath,
    Fill,
    FillRect(Rect<f32>),
    GetImageData(Rect<i32>, Size2D<f64>, IpcSender<Vec<u8>>),
    IsPointInPath(f64, f64, FillRule, IpcSender<bool>),
    LineTo(Point2D<f32>),
    MoveTo(Point2D<f32>),
    PutImageData(Vec<u8>, Point2D<f64>, Size2D<f64>, Rect<f64>),
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
    SetShadowOffsetX(f64),
    SetShadowOffsetY(f64),
    SetShadowBlur(f64),
    SetShadowColor(RGBA),
}

#[derive(Clone, Deserialize, Serialize, HeapSizeOf)]
pub struct CanvasGradientStop {
    pub offset: f64,
    pub color: RGBA,
}

#[derive(Clone, Deserialize, Serialize, HeapSizeOf)]
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

#[derive(Clone, Deserialize, Serialize, HeapSizeOf)]
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

#[derive(Clone, Deserialize, Serialize)]
pub struct SurfaceStyle {
    pub surface_data: Vec<u8>,
    pub surface_size: Size2D<i32>,
    pub repeat_x: bool,
    pub repeat_y: bool,
}

impl SurfaceStyle {
    pub fn new(surface_data: Vec<u8>, surface_size: Size2D<i32>, repeat_x: bool, repeat_y: bool)
        -> SurfaceStyle {
        SurfaceStyle {
            surface_data: surface_data,
            surface_size: surface_size,
            repeat_x: repeat_x,
            repeat_y: repeat_y,
        }
    }
}


#[derive(Clone, Deserialize, Serialize)]
pub enum FillOrStrokeStyle {
    Color(RGBA),
    LinearGradient(LinearGradientStyle),
    RadialGradient(RadialGradientStyle),
    Surface(SurfaceStyle),
}

impl FillOrStrokeStyle {
    pub fn to_azure_pattern(&self, drawtarget: &DrawTarget) -> Option<Pattern> {
        match *self {
            FillOrStrokeStyle::Color(ref color) => {
                Some(Pattern::Color(ColorPattern::new(color::new(color.red,
                                                                 color.green,
                                                                 color.blue,
                                                                 color.alpha))))
            },
            FillOrStrokeStyle::LinearGradient(ref linear_gradient_style) => {
                let gradient_stops: Vec<GradientStop> = linear_gradient_style.stops.iter().map(|s| {
                    GradientStop {
                        offset: s.offset as AzFloat,
                        color: color::new(s.color.red, s.color.green, s.color.blue, s.color.alpha)
                    }
                }).collect();

                Some(Pattern::LinearGradient(LinearGradientPattern::new(
                    &Point2D::new(linear_gradient_style.x0 as AzFloat, linear_gradient_style.y0 as AzFloat),
                    &Point2D::new(linear_gradient_style.x1 as AzFloat, linear_gradient_style.y1 as AzFloat),
                    drawtarget.create_gradient_stops(&gradient_stops, ExtendMode::Clamp),
                    &Matrix2D::identity())))
            },
            FillOrStrokeStyle::RadialGradient(ref radial_gradient_style) => {
                let gradient_stops: Vec<GradientStop> = radial_gradient_style.stops.iter().map(|s| {
                    GradientStop {
                        offset: s.offset as AzFloat,
                        color: color::new(s.color.red, s.color.green, s.color.blue, s.color.alpha)
                    }
                }).collect();

                Some(Pattern::RadialGradient(RadialGradientPattern::new(
                    &Point2D::new(radial_gradient_style.x0 as AzFloat, radial_gradient_style.y0 as AzFloat),
                    &Point2D::new(radial_gradient_style.x1 as AzFloat, radial_gradient_style.y1 as AzFloat),
                    radial_gradient_style.r0 as AzFloat, radial_gradient_style.r1 as AzFloat,
                    drawtarget.create_gradient_stops(&gradient_stops, ExtendMode::Clamp),
                    &Matrix2D::identity())))
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
                        &Matrix2D::identity()))
                    })
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Deserialize, Serialize, HeapSizeOf)]
pub enum LineCapStyle {
    Butt = 0,
    Round = 1,
    Square = 2,
}

impl FromStr for LineCapStyle {
    type Err = ();

    fn from_str(string: &str) -> Result<LineCapStyle, ()> {
        match string {
            "butt" => Ok(LineCapStyle::Butt),
            "round" => Ok(LineCapStyle::Round),
            "square" => Ok(LineCapStyle::Square),
            _ => Err(()),
        }
    }
}

impl LineCapStyle {
    pub fn to_azure_style(&self) -> CapStyle {
        match *self {
            LineCapStyle::Butt => CapStyle::Butt,
            LineCapStyle::Round => CapStyle::Round,
            LineCapStyle::Square => CapStyle::Square,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Deserialize, Serialize, HeapSizeOf)]
pub enum LineJoinStyle {
    Round = 0,
    Bevel = 1,
    Miter = 2,
}

impl FromStr for LineJoinStyle {
    type Err = ();

    fn from_str(string: &str) -> Result<LineJoinStyle, ()> {
        match string {
            "round" => Ok(LineJoinStyle::Round),
            "bevel" => Ok(LineJoinStyle::Bevel),
            "miter" => Ok(LineJoinStyle::Miter),
            _ => Err(()),
        }
    }
}

impl LineJoinStyle {
    pub fn to_azure_style(&self) -> JoinStyle {
        match *self {
            LineJoinStyle::Round => JoinStyle::Round,
            LineJoinStyle::Bevel => JoinStyle::Bevel,
            LineJoinStyle::Miter => JoinStyle::Miter,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Deserialize, Serialize)]
pub enum RepetitionStyle {
    Repeat,
    RepeatX,
    RepeatY,
    NoRepeat,
}

impl FromStr for RepetitionStyle {
    type Err = ();

    fn from_str(string: &str) -> Result<RepetitionStyle, ()> {
        match string {
            "repeat" => Ok(RepetitionStyle::Repeat),
            "repeat-x" => Ok(RepetitionStyle::RepeatX),
            "repeat-y" => Ok(RepetitionStyle::RepeatY),
            "no-repeat" => Ok(RepetitionStyle::NoRepeat),
            _ => Err(()),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Deserialize, Serialize, HeapSizeOf)]
pub enum CompositionStyle {
    SrcIn,
    SrcOut,
    SrcOver,
    SrcAtop,
    DestIn,
    DestOut,
    DestOver,
    DestAtop,
    Copy,
    Lighter,
    Xor,
}

impl FromStr for CompositionStyle {
    type Err = ();

    fn from_str(string: &str) -> Result<CompositionStyle, ()> {
        match string {
            "source-in"        => Ok(CompositionStyle::SrcIn),
            "source-out"       => Ok(CompositionStyle::SrcOut),
            "source-over"      => Ok(CompositionStyle::SrcOver),
            "source-atop"      => Ok(CompositionStyle::SrcAtop),
            "destination-in"   => Ok(CompositionStyle::DestIn),
            "destination-out"  => Ok(CompositionStyle::DestOut),
            "destination-over" => Ok(CompositionStyle::DestOver),
            "destination-atop" => Ok(CompositionStyle::DestAtop),
            "copy"             => Ok(CompositionStyle::Copy),
            "lighter"          => Ok(CompositionStyle::Lighter),
            "xor"              => Ok(CompositionStyle::Xor),
            _ => Err(())
        }
    }
}

impl CompositionStyle {
    pub fn to_azure_style(&self) -> CompositionOp {
        match *self {
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

    pub fn to_str(&self) -> &str {
        match *self {
            CompositionStyle::SrcIn    => "source-in",
            CompositionStyle::SrcOut   => "source-out",
            CompositionStyle::SrcOver  => "source-over",
            CompositionStyle::SrcAtop  => "source-atop",
            CompositionStyle::DestIn   => "destination-in",
            CompositionStyle::DestOut  => "destination-out",
            CompositionStyle::DestOver => "destination-over",
            CompositionStyle::DestAtop => "destination-atop",
            CompositionStyle::Copy     => "copy",
            CompositionStyle::Lighter  => "lighter",
            CompositionStyle::Xor      => "xor",
        }
    }
}

#[derive(Copy, Clone, PartialEq, Deserialize, Serialize, HeapSizeOf)]
pub enum BlendingStyle {
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

impl FromStr for BlendingStyle {
    type Err = ();

    fn from_str(string: &str) -> Result<BlendingStyle, ()> {
        match string {
            "multiply"    => Ok(BlendingStyle::Multiply),
            "screen"      => Ok(BlendingStyle::Screen),
            "overlay"     => Ok(BlendingStyle::Overlay),
            "darken"      => Ok(BlendingStyle::Darken),
            "lighten"     => Ok(BlendingStyle::Lighten),
            "color-dodge" => Ok(BlendingStyle::ColorDodge),
            "color-burn"  => Ok(BlendingStyle::ColorBurn),
            "hard-light"  => Ok(BlendingStyle::HardLight),
            "soft-light"  => Ok(BlendingStyle::SoftLight),
            "difference"  => Ok(BlendingStyle::Difference),
            "exclusion"   => Ok(BlendingStyle::Exclusion),
            "hue"         => Ok(BlendingStyle::Hue),
            "saturation"  => Ok(BlendingStyle::Saturation),
            "color"       => Ok(BlendingStyle::Color),
            "luminosity"  => Ok(BlendingStyle::Luminosity),
            _ => Err(())
        }
    }
}

impl BlendingStyle {
    pub fn to_azure_style(&self) -> CompositionOp {
        match *self {
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

    pub fn to_str(&self) -> &str {
        match *self {
            BlendingStyle::Multiply   => "multiply",
            BlendingStyle::Screen     => "screen",
            BlendingStyle::Overlay    => "overlay",
            BlendingStyle::Darken     => "darken",
            BlendingStyle::Lighten    => "lighten",
            BlendingStyle::ColorDodge => "color-dodge",
            BlendingStyle::ColorBurn  => "color-burn",
            BlendingStyle::HardLight  => "hard-light",
            BlendingStyle::SoftLight  => "soft-light",
            BlendingStyle::Difference => "difference",
            BlendingStyle::Exclusion  => "exclusion",
            BlendingStyle::Hue        => "hue",
            BlendingStyle::Saturation => "saturation",
            BlendingStyle::Color      => "color",
            BlendingStyle::Luminosity => "luminosity",
        }
    }
}

#[derive(Copy, Clone, PartialEq, Deserialize, Serialize, HeapSizeOf)]
pub enum CompositionOrBlending {
    Composition(CompositionStyle),
    Blending(BlendingStyle),
}

impl Default for CompositionOrBlending {
    fn default() -> CompositionOrBlending {
        CompositionOrBlending::Composition(CompositionStyle::SrcOver)
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

impl CompositionOrBlending {
    pub fn to_azure_style(&self) -> CompositionOp {
        match *self {
            CompositionOrBlending::Composition(op) => op.to_azure_style(),
            CompositionOrBlending::Blending(op) => op.to_azure_style(),
        }
    }
}

pub trait ToAzColor {
    fn to_azcolor(&self) -> AzColor;
}

impl ToAzColor for RGBA {
    fn to_azcolor(&self) -> AzColor {
        color::rgba(self.red as AzFloat,
                    self.green as AzFloat,
                    self.blue as AzFloat,
                    self.alpha as AzFloat)
    }
}

// TODO(pcwalton): Speed up with SIMD, or better yet, find some way to not do this.
pub fn byte_swap(data: &mut [u8]) {
    let length = data.len();
    // FIXME(rust #27741): Range::step_by is not stable yet as of this writing.
    let mut i = 0;
    while i < length {
        let r = data[i + 2];
        data[i + 2] = data[i + 0];
        data[i + 0] = r;
        i += 4;
    }
}
