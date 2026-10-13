/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub mod paint;
pub mod paint_servers;

pub use self::paint::{FillParams, MarkerRefs, StrokeParams};
use crate::model::document::{ClipPathDef, DefRef, FilterDef, MaskDef};
use crate::model::units::Opacity;

pub struct NodeStyle {
    pub visibility: Visibility,
    pub display: Display,
    pub fill: Option<FillParams>,
    pub stroke: Option<StrokeParams>,
    pub render_hints: Option<RenderHints>,
    pub clip_path: Option<DefRef<ClipPathDef>>,
    pub mask: Option<DefRef<MaskDef>>,
    pub filter: Option<DefRef<FilterDef>>,
    pub opacity: Opacity,
    pub markers: Option<MarkerRefs>,
}

impl Default for NodeStyle {
    fn default() -> Self {
        NodeStyle {
            visibility: Visibility::Visible,
            display: Display::Inline,
            fill: None,
            stroke: None,
            render_hints: None,
            clip_path: None,
            mask: None,
            filter: None,
            opacity: Opacity::ONE,
            markers: None,
        }
    }
}

pub enum Visibility {
    Visible,
    Hidden,
}

pub enum Display {
    Inline,
    Block,
    None,
}

pub struct RenderHints {
    pub vector_effect: Option<VectorEffect>,
    pub color_rendering: Option<ColorRendering>,
    pub color_interpolation: Option<ColorInterpolation>,
    pub shape_rendering: Option<ShapeRendering>,
    pub paint_order: Option<PaintOrder>,
    pub text_rendering: Option<TextRendering>,
    pub image_rendering: Option<ImageRendering>,
}

pub enum VectorEffect {
    None,
    NonScalingStroke,
}

pub enum ColorRendering {
    Auto,
    OptimizeSpeed,
    OptimizeQuality,
}

pub enum ColorInterpolation {
    Auto,
    Srgb,
    LinearRGB,
}

pub enum ShapeRendering {
    Auto,
    OptimizeSpeed,
    CrispEdges,
    GeometricPrecision,
}

pub struct PaintOrder {
    pub order: [PaintOperation; 3],
}

impl Default for PaintOrder {
    fn default() -> Self {
        PaintOrder {
            order: [
                PaintOperation::Fill,
                PaintOperation::Stroke,
                PaintOperation::Markers,
            ],
        }
    }
}

pub enum PaintOperation {
    Fill,
    Stroke,
    Markers,
}

pub enum TextRendering {
    Auto,
    OptimizeSpeed,
    OptimizeLegibility,
    GeometricPrecision,
}

pub enum ImageRendering {
    Auto,
    OptimizeSpeed,
    OptimizeQuality,
}
