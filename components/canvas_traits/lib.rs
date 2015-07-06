/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![crate_name = "canvas_traits"]
#![crate_type = "rlib"]
#![feature(core)]
#![feature(nonzero)]
extern crate core;
extern crate azure;
extern crate euclid;
extern crate cssparser;
extern crate gfx_traits;
extern crate layers;
extern crate offscreen_gl_context;

use azure::azure::{AzFloat, AzColor};
use azure::azure_hl::{DrawTarget, Pattern, ColorPattern};
use azure::azure_hl::{GradientStop, LinearGradientPattern, RadialGradientPattern, ExtendMode};
use azure::azure_hl::{JoinStyle, CapStyle, CompositionOp};
use azure::azure_hl::{SurfacePattern, SurfaceFormat};
use cssparser::RGBA;
use euclid::matrix2d::Matrix2D;
use euclid::point::Point2D;
use euclid::rect::Rect;
use euclid::size::Size2D;
use gfx_traits::color;
use std::sync::mpsc::{Sender};
use layers::platform::surface::NativeSurface;
use offscreen_gl_context::GLContextAttributes;
use core::nonzero::NonZero;

#[derive(Clone)]
pub enum CanvasMsg {
    Canvas2d(Canvas2dMsg),
    Common(CanvasCommonMsg),
    WebGL(CanvasWebGLMsg),
}

#[derive(Clone)]
pub enum CanvasCommonMsg {
    Close,
    Recreate(Size2D<i32>),
    SendPixelContents(Sender<Vec<u8>>),
    SendNativeSurface(Sender<NativeSurface>),
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
    SetShadowOffsetX(f64),
    SetShadowOffsetY(f64),
    SetShadowBlur(f64),
    SetShadowColor(RGBA),
}

#[derive(Clone)]
pub enum CanvasWebGLMsg {
    GetContextAttributes(Sender<GLContextAttributes>),
    ActiveTexture(u32),
    BlendColor(f32, f32, f32, f32),
    BlendEquation(u32),
    BlendEquationSeparate(u32, u32),
    BlendFunc(u32, u32),
    BlendFuncSeparate(u32, u32, u32, u32),
    AttachShader(u32, u32),
    BufferData(u32, Vec<f32>, u32),
    Clear(u32),
    ClearColor(f32, f32, f32, f32),
    CompileShader(u32),
    CreateBuffer(Sender<Option<NonZero<u32>>>),
    CreateFramebuffer(Sender<Option<NonZero<u32>>>),
    CreateRenderbuffer(Sender<Option<NonZero<u32>>>),
    CreateTexture(Sender<Option<NonZero<u32>>>),
    CreateProgram(Sender<Option<NonZero<u32>>>),
    CreateShader(u32, Sender<Option<NonZero<u32>>>),
    DeleteBuffer(u32),
    DeleteFramebuffer(u32),
    DeleteRenderbuffer(u32),
    DeleteTexture(u32),
    DeleteProgram(u32),
    DeleteShader(u32),
    BindBuffer(u32, u32),
    BindFramebuffer(u32, WebGLFramebufferBindingRequest),
    BindRenderbuffer(u32, u32),
    BindTexture(u32, u32),
    DrawArrays(u32, i32, i32),
    EnableVertexAttribArray(u32),
    GetShaderInfoLog(u32, Sender<Option<String>>),
    GetShaderParameter(u32, u32, Sender<WebGLShaderParameter>),
    GetAttribLocation(u32, String, Sender<Option<i32>>),
    GetUniformLocation(u32, String, Sender<Option<i32>>),
    LinkProgram(u32),
    ShaderSource(u32, String),
    Uniform4fv(i32, Vec<f32>),
    UseProgram(u32),
    VertexAttribPointer2f(u32, i32, bool, i32, i64),
    Viewport(i32, i32, i32, i32),
    DrawingBufferWidth(Sender<i32>),
    DrawingBufferHeight(Sender<i32>),
}

#[derive(Clone, Copy, PartialEq)]
pub enum WebGLError {
    InvalidEnum,
    InvalidOperation,
    InvalidValue,
    OutOfMemory,
    ContextLost,
}

pub type WebGLResult<T> = Result<T, WebGLError>;

#[derive(Clone)]
pub enum WebGLFramebufferBindingRequest {
    Explicit(u32),
    Default,
}

#[derive(Clone)]
pub enum WebGLShaderParameter {
    Int(i32),
    Bool(bool),
    Invalid,
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


#[derive(Clone)]
pub enum FillOrStrokeStyle {
    Color(RGBA),
    LinearGradient(LinearGradientStyle),
    RadialGradient(RadialGradientStyle),
    Surface(SurfaceStyle),
}

impl FillOrStrokeStyle {
    pub fn to_azure_pattern(&self, drawtarget: &DrawTarget) -> Pattern {
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
                    &Point2D::new(linear_gradient_style.x0 as AzFloat, linear_gradient_style.y0 as AzFloat),
                    &Point2D::new(linear_gradient_style.x1 as AzFloat, linear_gradient_style.y1 as AzFloat),
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
                    &Point2D::new(radial_gradient_style.x0 as AzFloat, radial_gradient_style.y0 as AzFloat),
                    &Point2D::new(radial_gradient_style.x1 as AzFloat, radial_gradient_style.y1 as AzFloat),
                    radial_gradient_style.r0 as AzFloat, radial_gradient_style.r1 as AzFloat,
                    drawtarget.create_gradient_stops(&gradient_stops, ExtendMode::Clamp),
                    &Matrix2D::identity()))
            },
            FillOrStrokeStyle::Surface(ref surface_style) => {
                let source_surface = drawtarget.create_source_surface_from_data(
                    &surface_style.surface_data,
                    surface_style.surface_size,
                    surface_style.surface_size.width * 4,
                    SurfaceFormat::B8G8R8A8);

                Pattern::Surface(SurfacePattern::new(
                    source_surface.azure_source_surface,
                    surface_style.repeat_x,
                    surface_style.repeat_y))
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
    pub fn to_azure_style(&self) -> CapStyle {
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
    pub fn to_azure_style(&self) -> JoinStyle {
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

#[derive(Copy, Clone, PartialEq)]
pub enum RepetitionStyle {
    Repeat,
    RepeatX,
    RepeatY,
    NoRepeat,
}

impl RepetitionStyle {
    pub fn from_str(string: &str) -> Option<RepetitionStyle> {
        match string {
            "repeat" => Some(RepetitionStyle::Repeat),
            "repeat-x" => Some(RepetitionStyle::RepeatX),
            "repeat-y" => Some(RepetitionStyle::RepeatY),
            "no-repeat" => Some(RepetitionStyle::NoRepeat),
            _ => None
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
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

    pub fn from_str(string: &str) -> Option<CompositionStyle> {
        match string {
            "source-in"        => Some(CompositionStyle::SrcIn),
            "source-out"       => Some(CompositionStyle::SrcOut),
            "source-over"      => Some(CompositionStyle::SrcOver),
            "source-atop"      => Some(CompositionStyle::SrcAtop),
            "destination-in"   => Some(CompositionStyle::DestIn),
            "destination-out"  => Some(CompositionStyle::DestOut),
            "destination-over" => Some(CompositionStyle::DestOver),
            "destination-atop" => Some(CompositionStyle::DestAtop),
            "copy"             => Some(CompositionStyle::Copy),
            "lighter"          => Some(CompositionStyle::Lighter),
            "xor"              => Some(CompositionStyle::Xor),
            _ => None
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

#[derive(Copy, Clone, PartialEq)]
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

    pub fn from_str(string: &str) -> Option<BlendingStyle> {
        match string {
            "multiply"    => Some(BlendingStyle::Multiply),
            "screen"      => Some(BlendingStyle::Screen),
            "overlay"     => Some(BlendingStyle::Overlay),
            "darken"      => Some(BlendingStyle::Darken),
            "lighten"     => Some(BlendingStyle::Lighten),
            "color-dodge" => Some(BlendingStyle::ColorDodge),
            "color-burn"  => Some(BlendingStyle::ColorBurn),
            "hard-light"  => Some(BlendingStyle::HardLight),
            "soft-light"  => Some(BlendingStyle::SoftLight),
            "difference"  => Some(BlendingStyle::Difference),
            "exclusion"   => Some(BlendingStyle::Exclusion),
            "hue"         => Some(BlendingStyle::Hue),
            "saturation"  => Some(BlendingStyle::Saturation),
            "color"       => Some(BlendingStyle::Color),
            "luminosity"  => Some(BlendingStyle::Luminosity),
            _ => None
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

#[derive(Copy, Clone, PartialEq)]
pub enum CompositionOrBlending {
    Composition(CompositionStyle),
    Blending(BlendingStyle),
}

impl CompositionOrBlending {
    pub fn to_azure_style(&self) -> CompositionOp {
        match *self {
            CompositionOrBlending::Composition(op) => op.to_azure_style(),
            CompositionOrBlending::Blending(op) => op.to_azure_style(),
        }
    }

    pub fn default() -> CompositionOrBlending {
        CompositionOrBlending::Composition(CompositionStyle::SrcOver)
    }

    pub fn from_str(string: &str) -> Option<CompositionOrBlending> {
        if let Some(op) = CompositionStyle::from_str(string) {
            return Some(CompositionOrBlending::Composition(op));
        }

        if let Some(op) = BlendingStyle::from_str(string) {
            return Some(CompositionOrBlending::Blending(op));
        }

        None
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
