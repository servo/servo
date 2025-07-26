/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::default::Default;
use std::str::FromStr;

use euclid::Angle;
use euclid::approxeq::ApproxEq;
use euclid::default::{Point2D, Rect, Size2D, Transform2D};
use ipc_channel::ipc::IpcSender;
use kurbo::{BezPath, ParamCurveNearest as _, PathEl, Point, Shape, Triangle};
use malloc_size_of::MallocSizeOf;
use malloc_size_of_derive::MallocSizeOf;
use pixels::IpcSnapshot;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use style::color::AbsoluteColor;
use style::properties::style_structs::Font as FontStyleStruct;
use style::servo_arc::Arc as ServoArc;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Path(pub BezPath);

impl MallocSizeOf for Path {
    fn size_of(&self, _ops: &mut malloc_size_of::MallocSizeOfOps) -> usize {
        std::mem::size_of_val(self.0.elements())
    }
}

pub struct IndexSizeError;

impl Path {
    pub fn new() -> Self {
        Self(BezPath::new())
    }

    pub fn from_svg(s: &str) -> Self {
        Self(BezPath::from_svg(s).unwrap_or_default())
    }

    pub fn transform(&mut self, transform: Transform2D<f64>) {
        self.0.apply_affine(transform.into());
    }

    /// <https://html.spec.whatwg.org/multipage/#ensure-there-is-a-subpath>
    pub fn ensure_there_is_a_subpath(&mut self, x: f64, y: f64) {
        // The user agent must check to see if the path has its need new subpath flag set.
        if self.0.elements().is_empty() {
            // If it does, then the user agent must create a new subpath with the point (x, y)
            // as its first (and only) point,
            // as if the moveTo() method had been called,
            // and must then unset the path's need new subpath flag.
            self.0.move_to((x, y));
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-closepath>
    pub fn close_path(&mut self) {
        // must do nothing if the object's path has no subpaths
        if matches!(self.0.elements().last(), None | Some(PathEl::ClosePath)) {
            return;
        }
        // Otherwise, it must mark the last subpath as closed,
        // create a new subpath whose first point is the same as the previous subpath's first point,
        // and finally add this new subpath to the path.
        self.0.close_path();
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-moveto>
    pub fn move_to(&mut self, x: f64, y: f64) {
        // Step 1. If either of the arguments are infinite or NaN, then return.
        if !(x.is_finite() && y.is_finite()) {
            return;
        }

        // Step 2. Create a new subpath with the specified point as its first (and only) point.
        self.0.move_to((x, y));
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-lineto>
    pub fn line_to(&mut self, x: f64, y: f64) {
        // Step 1. If either of the arguments are infinite or NaN, then return.
        if !(x.is_finite() && y.is_finite()) {
            return;
        }

        // Step 2. If the object's path has no subpaths, then ensure there is a subpath for (x, y).
        self.ensure_there_is_a_subpath(x, y);

        // Step 3. Otherwise, connect the last point in the subpath to the given point (x, y) using a straight line,
        // and then add the given point (x, y) to the subpath.
        self.0.line_to((x, y));
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-quadraticcurveto>
    pub fn quadratic_curve_to(&mut self, cpx: f64, cpy: f64, x: f64, y: f64) {
        // Step 1. If any of the arguments are infinite or NaN, then return.
        if !(cpx.is_finite() && cpy.is_finite() && x.is_finite() && y.is_finite()) {
            return;
        }

        // Step 2. Ensure there is a subpath for (cpx, cpy).
        self.ensure_there_is_a_subpath(cpx, cpy);

        // 3. Connect the last point in the subpath to the given point (x, y)
        // using a quadratic Bézier curve with control point (cpx, cpy). [BEZIER]
        // 4. Add the given point (x, y) to the subpath.
        self.0.quad_to((cpx, cpy), (x, y));
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-beziercurveto>
    pub fn bezier_curve_to(&mut self, cp1x: f64, cp1y: f64, cp2x: f64, cp2y: f64, x: f64, y: f64) {
        // Step 1. If any of the arguments are infinite or NaN, then return.
        if !(cp1x.is_finite() &&
            cp1y.is_finite() &&
            cp2x.is_finite() &&
            cp2y.is_finite() &&
            x.is_finite() &&
            y.is_finite())
        {
            return;
        }

        // Step 2. Ensure there is a subpath for (cp1x, cp1y).
        self.ensure_there_is_a_subpath(cp1x, cp1y);

        // Step 3. Connect the last point in the subpath to the given point (x, y)
        // using a cubic Bézier curve with control points (cp1x, cp1y) and (cp2x, cp2y). [BEZIER]
        // Step 4. Add the point (x, y) to the subpath.
        self.0.curve_to((cp1x, cp1y), (cp2x, cp2y), (x, y));
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-arcto>
    pub fn arc_to(
        &mut self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        radius: f64,
    ) -> Result<(), IndexSizeError> {
        // Step 1. If any of the arguments are infinite or NaN, then return.
        if !(x1.is_finite() &&
            y1.is_finite() &&
            x2.is_finite() &&
            y2.is_finite() &&
            radius.is_finite())
        {
            return Ok(());
        }

        // Step 2. Ensure there is a subpath for (x1, y1).
        self.ensure_there_is_a_subpath(x1, y1);

        // Step 3. If either radius is negative, then throw an "IndexSizeError" DOMException.
        if radius < 0.0 {
            return Err(IndexSizeError);
        }

        // Step 4. Let the point (x0, y0) be the last point in the subpath.
        let Point { x: x0, y: y0 } = self.last_point().unwrap();

        // Step 5. If the point (x0, y0) is equal to the point (x1, y1),
        // or if the point (x1, y1) is equal to the point (x2, y2),
        // or if radius is zero, then add the point (x1, y1) to the subpath,
        // and connect that point to the previous point (x0, y0) by a straight line.
        if ((x0, y0) == (x1, y1)) || ((x1, y1) == (x2, y2)) || radius.approx_eq(&0.0) {
            self.0.line_to((x1, y1));
            return Ok(());
        }

        // Step 6. Otherwise, if the points (x0, y0), (x1, y1), and (x2, y2)
        // all lie on a single straight line, then add the point (x1, y1) to the subpath,
        // and connect that point to the previous point (x0, y0) by a straight line.
        let direction = Triangle::from_coords((x0, y0), (x1, y1), (x2, y2)).area();
        if direction.approx_eq(&0.0) {
            self.0.line_to((x1, y1));
            return Ok(());
        }

        // Step 7. Otherwise, let The Arc be the shortest arc given by circumference of the circle
        // that has radius radius, and that has one point tangent to the half-infinite line
        // that crosses the point (x0, y0) and ends at the point (x1, y1),
        // and that has a different point tangent to the half-infinite line that ends at the point (x1, y1)
        // and crosses the point (x2, y2).
        // The points at which this circle touches these two lines are called the start
        // and end tangent points respectively.
        // Connect the point (x0, y0) to the start tangent point by a straight line,
        // adding the start tangent point to the subpath,
        // and then connect the start tangent point to the end tangent point by The Arc,
        // adding the end tangent point to the subpath.

        let a2 = (x0 - x1).powi(2) + (y0 - y1).powi(2);
        let b2 = (x1 - x2).powi(2) + (y1 - y2).powi(2);
        let d = {
            let c2 = (x0 - x2).powi(2) + (y0 - y2).powi(2);
            let cosx = (a2 + b2 - c2) / (2.0 * (a2 * b2).sqrt());
            let sinx = (1.0 - cosx.powi(2)).sqrt();
            radius / ((1.0 - cosx) / sinx)
        };

        // first tangent point
        let anx = (x1 - x0) / a2.sqrt();
        let any = (y1 - y0) / a2.sqrt();
        let tp1 = Point2D::new(x1 - anx * d, y1 - any * d);

        // second tangent point
        let bnx = (x1 - x2) / b2.sqrt();
        let bny = (y1 - y2) / b2.sqrt();
        let tp2 = Point2D::new(x1 - bnx * d, y1 - bny * d);

        // arc center and angles
        let anticlockwise = direction < 0.0;
        let cx = tp1.x + any * radius * if anticlockwise { 1.0 } else { -1.0 };
        let cy = tp1.y - anx * radius * if anticlockwise { 1.0 } else { -1.0 };
        let angle_start = (tp1.y - cy).atan2(tp1.x - cx);
        let angle_end = (tp2.y - cy).atan2(tp2.x - cx);

        self.0.line_to((tp1.x, tp1.y));

        self.arc(cx, cy, radius, angle_start, angle_end, anticlockwise)
    }

    pub fn last_point(&mut self) -> Option<Point> {
        self.0.current_position()
    }

    #[allow(clippy::too_many_arguments)]
    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-arc>
    pub fn arc(
        &mut self,
        x: f64,
        y: f64,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
        counterclockwise: bool,
    ) -> Result<(), IndexSizeError> {
        // ellipse() with both radii are equal and rotation is 0.
        self.ellipse(
            x,
            y,
            radius,
            radius,
            0.,
            start_angle,
            end_angle,
            counterclockwise,
        )
    }

    #[allow(clippy::too_many_arguments)]
    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-ellipse>
    pub fn ellipse(
        &mut self,
        x: f64,
        y: f64,
        radius_x: f64,
        radius_y: f64,
        rotation_angle: f64,
        start_angle: f64,
        end_angle: f64,
        counterclockwise: bool,
    ) -> Result<(), IndexSizeError> {
        // Step 1. If any of the arguments are infinite or NaN, then return.
        if !(x.is_finite() &&
            y.is_finite() &&
            radius_x.is_finite() &&
            radius_y.is_finite() &&
            rotation_angle.is_finite() &&
            start_angle.is_finite() &&
            end_angle.is_finite())
        {
            return Ok(());
        }

        // Step 2. If either radiusX or radiusY are negative, then throw an "IndexSizeError" DOMException.
        if radius_x < 0.0 || radius_y < 0.0 {
            return Err(IndexSizeError);
        }

        let mut start = Angle::radians(start_angle);
        let mut end = Angle::radians(end_angle);

        // Wrap angles mod 2 * PI if necessary
        if !counterclockwise && start > end + Angle::two_pi() ||
            counterclockwise && end > start + Angle::two_pi()
        {
            start = start.positive();
            end = end.positive();
        }

        // Calculate the total arc we're going to sweep.
        let sweep = match counterclockwise {
            true => {
                if end - start == Angle::two_pi() {
                    -Angle::two_pi()
                } else if end > start {
                    -(Angle::two_pi() - (end - start))
                } else {
                    -(start - end)
                }
            },
            false => {
                if start - end == Angle::two_pi() {
                    Angle::two_pi()
                } else if start > end {
                    Angle::two_pi() - (start - end)
                } else {
                    end - start
                }
            },
        };

        let arc = kurbo::Arc::new(
            (x, y),
            (radius_x, radius_y),
            start.radians,
            sweep.radians,
            rotation_angle,
        );

        let mut iter = arc.path_elements(0.01);
        let kurbo::PathEl::MoveTo(start_point) = iter.next().unwrap() else {
            unreachable!()
        };

        self.line_to(start_point.x, start_point.y);

        if sweep.radians.abs() > 1e-3 {
            self.0.extend(iter);
        }

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-rect>
    pub fn rect(&mut self, x: f64, y: f64, w: f64, h: f64) {
        // Step 1. If any of the arguments are infinite or NaN, then return.
        if !(x.is_finite() && y.is_finite() && w.is_finite() && h.is_finite()) {
            return;
        }

        // Step 2. Create a new subpath containing just the four points
        // (x, y), (x+w, y), (x+w, y+h), (x, y+h), in that order,
        // with those four points connected by straight lines.
        self.0.move_to((x, y));
        self.0.line_to((x + w, y));
        self.0.line_to((x + w, y + h));
        self.0.line_to((x, y + h));

        // Step 3. Mark the subpath as closed.
        self.0.close_path();

        // Step 4. Create a new subpath with the point (x, y) as the only point in the subpath.
        self.0.move_to((x, y));
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-ispointinpath>
    pub fn is_point_in_path(&self, x: f64, y: f64, fill_rule: FillRule) -> bool {
        let p = Point::new(x, y);
        // Step 1. If x or y are infinite or NaN, then return false.
        if !p.is_finite() {
            return false;
        }

        // Step 2. If the point given by the x and y coordinates,
        // when treated as coordinates in the canvas coordinate space unaffected by the current transformation,
        // is inside the intended path for path as determined by the fill rule indicated by fillRule,
        // then return true.
        // Open subpaths must be implicitly closed when computing the area inside the path,
        // without affecting the actual subpaths.
        let mut path = self.clone();
        path.close_path();
        let winding = path.0.winding(p);
        let is_inside = match fill_rule {
            FillRule::Nonzero => winding != 0,
            FillRule::Evenodd => (winding % 2) != 0,
        };
        if is_inside {
            return true;
        }
        // Points on the path itself must be considered to be inside the path.
        path.0
            .segments()
            .any(|seg| seg.nearest(p, 0.00001).distance_sq < 0.00001)
    }

    pub fn bounding_box(&self) -> Rect<f64> {
        self.0.control_box().into()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum FillRule {
    Nonzero,
    Evenodd,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
pub struct CanvasId(pub u64);

#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct CompositionOptions {
    pub alpha: f64,
    pub composition_operation: CompositionOrBlending,
}

#[derive(Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct ShadowOptions {
    pub offset_x: f64,
    pub offset_y: f64,
    pub blur: f64,
    pub color: AbsoluteColor,
}

impl ShadowOptions {
    /// <https://html.spec.whatwg.org/multipage/#when-shadows-are-drawn>
    pub fn need_to_draw_shadow(&self) -> bool {
        // Shadows are only drawn if the opacity component of the alpha component of the shadow color is nonzero
        self.color.alpha != 0.0 &&
        // and either the shadowBlur is nonzero, or the shadowOffsetX is nonzero, or the shadowOffsetY is nonzero.
            (self.offset_x != 0.0 ||
                self.offset_y != 0.0 ||
                self.blur != 0.0)
    }
}

#[derive(Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct LineOptions {
    pub width: f64,
    pub cap_style: LineCapStyle,
    pub join_style: LineJoinStyle,
    pub miter_limit: f64,
    pub dash: Vec<f32>,
    pub dash_offset: f64,
}

#[derive(Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct TextOptions {
    #[ignore_malloc_size_of = "Arc"]
    pub font: Option<ServoArc<FontStyleStruct>>,
    pub align: TextAlign,
    pub baseline: TextBaseline,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Deserialize, Serialize)]
pub enum CanvasMsg {
    Canvas2d(Canvas2dMsg, CanvasId),
    Recreate(Option<Size2D<u64>>, CanvasId),
    Close(CanvasId),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Canvas2dMsg {
    DrawImage(
        IpcSnapshot,
        Rect<f64>,
        Rect<f64>,
        bool,
        ShadowOptions,
        CompositionOptions,
        Transform2D<f32>,
    ),
    DrawEmptyImage(
        Size2D<u32>,
        Rect<f64>,
        Rect<f64>,
        ShadowOptions,
        CompositionOptions,
        Transform2D<f32>,
    ),
    DrawImageInOther(
        CanvasId,
        Rect<f64>,
        Rect<f64>,
        bool,
        ShadowOptions,
        CompositionOptions,
        Transform2D<f32>,
    ),
    ClearRect(Rect<f32>, Transform2D<f32>),
    ClipPath(Path, FillRule, Transform2D<f32>),
    PopClip,
    FillPath(
        FillOrStrokeStyle,
        Path,
        FillRule,
        ShadowOptions,
        CompositionOptions,
        Transform2D<f32>,
    ),
    FillText(
        String,
        f64,
        f64,
        Option<f64>,
        FillOrStrokeStyle,
        bool,
        TextOptions,
        ShadowOptions,
        CompositionOptions,
        Transform2D<f32>,
    ),
    FillRect(
        Rect<f32>,
        FillOrStrokeStyle,
        ShadowOptions,
        CompositionOptions,
        Transform2D<f32>,
    ),
    GetImageData(Option<Rect<u32>>, IpcSender<IpcSnapshot>),
    MeasureText(String, IpcSender<TextMetrics>, TextOptions),
    PutImageData(Rect<u32>, IpcSnapshot),
    StrokeRect(
        Rect<f32>,
        FillOrStrokeStyle,
        LineOptions,
        ShadowOptions,
        CompositionOptions,
        Transform2D<f32>,
    ),
    StrokePath(
        Path,
        FillOrStrokeStyle,
        LineOptions,
        ShadowOptions,
        CompositionOptions,
        Transform2D<f32>,
    ),
    UpdateImage(IpcSender<()>),
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
    pub surface_data: IpcSnapshot,
    pub surface_size: Size2D<u32>,
    pub repeat_x: bool,
    pub repeat_y: bool,
    pub transform: Transform2D<f32>,
}

impl SurfaceStyle {
    pub fn new(
        surface_data: IpcSnapshot,
        surface_size: Size2D<u32>,
        repeat_x: bool,
        repeat_y: bool,
        transform: Transform2D<f32>,
    ) -> Self {
        Self {
            surface_data,
            surface_size,
            repeat_x,
            repeat_y,
            transform,
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

impl FillOrStrokeStyle {
    pub fn is_zero_size_gradient(&self) -> bool {
        match self {
            Self::RadialGradient(pattern) => {
                let centers_equal = (pattern.x0, pattern.y0) == (pattern.x1, pattern.y1);
                let radii_equal = pattern.r0 == pattern.r1;
                (centers_equal && radii_equal) || pattern.stops.is_empty()
            },
            Self::LinearGradient(pattern) => {
                (pattern.x0, pattern.y0) == (pattern.x1, pattern.y1) || pattern.stops.is_empty()
            },
            Self::Color(..) | Self::Surface(..) => false,
        }
    }

    pub fn x_bound(&self) -> Option<u32> {
        match self {
            Self::Surface(pattern) => {
                if pattern.repeat_x {
                    None
                } else {
                    Some(pattern.surface_size.width)
                }
            },
            Self::Color(..) | Self::LinearGradient(..) | Self::RadialGradient(..) => None,
        }
    }

    pub fn y_bound(&self) -> Option<u32> {
        match self {
            Self::Surface(pattern) => {
                if pattern.repeat_y {
                    None
                } else {
                    Some(pattern.surface_size.height)
                }
            },
            Self::Color(..) | Self::LinearGradient(..) | Self::RadialGradient(..) => None,
        }
    }
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
