/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Painting of display lists using Moz2D/Azure.

use app_units::Au;
use azure::azure::AzIntSize;
use azure::azure_hl::{AntialiasMode, Color, ColorPattern, CompositionOp};
use azure::azure_hl::{CapStyle, JoinStyle};
use azure::azure_hl::{DrawOptions, DrawSurfaceOptions, DrawTarget, ExtendMode, FilterType};
use azure::azure_hl::{Filter, FilterNode, GaussianBlurInput, GradientStop, LinearGradientPattern};
use azure::azure_hl::{GaussianBlurAttribute, StrokeOptions, SurfaceFormat};
use azure::azure_hl::{Path, PathBuilder, Pattern, PatternRef, SurfacePattern};
use azure::scaled_font::ScaledFont;
use azure::{AzDrawTargetFillGlyphs, struct__AzGlyphBuffer, struct__AzPoint};
use azure::{AzFloat, struct__AzDrawOptions, struct__AzGlyph};
use display_list::TextOrientation::{SidewaysLeft, SidewaysRight, Upright};
use display_list::{BLUR_INFLATION_FACTOR, BorderRadii, BoxShadowClipMode, ClippingRegion};
use display_list::{TextDisplayItem, WebRenderImageInfo};
use euclid::matrix2d::Matrix2D;
use euclid::point::Point2D;
use euclid::rect::{Rect, TypedRect};
use euclid::scale_factor::ScaleFactor;
use euclid::side_offsets::SideOffsets2D;
use euclid::size::Size2D;
use filters;
use font_context::FontContext;
use gfx_traits::{color, LayerKind};
use net_traits::image::base::PixelFormat;
use range::Range;
use std::default::Default;
use std::{f32, mem, ptr};
use style::computed_values::{border_style, filter, image_rendering, mix_blend_mode};
use style_traits::PagePx;
use text::TextRun;
use text::glyph::ByteIndex;
use util::geometry::{self, MAX_RECT, ScreenPx};
use util::opts;

pub struct PaintContext<'a> {
    pub draw_target: DrawTarget,
    pub font_context: &'a mut Box<FontContext>,
    /// The rectangle that this context encompasses in page coordinates.
    pub page_rect: TypedRect<PagePx, f32>,
    /// The rectangle that this context encompasses in screen coordinates (pixels).
    pub screen_rect: TypedRect<ScreenPx, usize>,
    /// The clipping rect for the stacking context as a whole.
    pub clip_rect: Option<Rect<Au>>,
    /// The current transient clipping region, if any. A "transient clipping region" is the
    /// clipping region used by the last display item. We cache the last value so that we avoid
    /// pushing and popping clipping regions unnecessarily.
    pub transient_clip: Option<ClippingRegion>,
    /// A temporary hack to disable clipping optimizations on 3d layers.
    pub layer_kind: LayerKind,
}

#[derive(Copy, Clone)]
enum Direction {
    Top,
    Left,
    Right,
    Bottom
}

#[derive(Copy, Clone)]
enum BorderCorner {
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}

#[derive(Copy, Clone)]
enum DashSize {
    DottedBorder = 1,
    DashedBorder = 3
}

#[derive(Copy, Clone, Debug)]
struct Ellipse {
    origin: Point2D<f32>,
    width: f32,
    height: f32,
}

/// When `Line::new` creates a new `Line` it ensures `start.x <= end.x` for that line.
#[derive(Copy, Clone, Debug)]
struct Line {
    start: Point2D<f32>,
    end: Point2D<f32>,
}

impl Line {
    /// Guarantees that `start.x <= end.x` for the returned `Line`.
    fn new(start: Point2D<f32>, end: Point2D<f32>) -> Line {
        let line = if start.x <= end.x {
            Line { start: start, end: end }
        } else {
            Line { start: end, end: start }
        };
        debug_assert!(line.length_squared() > f32::EPSILON);
        line
    }

    fn length_squared(&self) -> f32 {
        let width = (self.end.x - self.start.x).abs();
        let height = (self.end.y - self.start.y).abs();
        width * width + height * height
    }
}

struct CornerOrigin {
        top_left: Point2D<f32>,
        top_right: Point2D<f32>,
        bottom_right: Point2D<f32>,
        bottom_left: Point2D<f32>,
}

impl<'a> PaintContext<'a> {
    pub fn screen_pixels_per_px(&self) -> ScaleFactor<PagePx, ScreenPx, f32> {
        self.screen_rect.as_f32().size.width / self.page_rect.size.width
    }

    pub fn draw_target(&self) -> &DrawTarget {
        &self.draw_target
    }

    pub fn draw_solid_color(&self, bounds: &Rect<Au>, color: Color) {
        self.draw_target.make_current();
        self.draw_target.fill_rect(&bounds.to_nearest_azure_rect(self.screen_pixels_per_px()),
                                   PatternRef::Color(&ColorPattern::new(color)),
                                   None);
    }

    pub fn draw_border(&self,
                       bounds: &Rect<Au>,
                       border: &SideOffsets2D<Au>,
                       radius: &BorderRadii<Au>,
                       color: &SideOffsets2D<Color>,
                       style: &SideOffsets2D<border_style::T>) {
        let scale = self.screen_pixels_per_px();
        let border = border.to_float_pixels(scale);
        let radius = radius.to_radii_pixels(scale);

        self.draw_border_segment(Direction::Top, bounds, &border, &radius, color, style);
        self.draw_border_segment(Direction::Right, bounds, &border, &radius, color, style);
        self.draw_border_segment(Direction::Bottom, bounds, &border, &radius, color, style);
        self.draw_border_segment(Direction::Left, bounds, &border, &radius, color, style);
    }

    pub fn draw_line(&self, bounds: &Rect<Au>, color: Color, style: border_style::T) {
        self.draw_target.make_current();

        self.draw_line_segment(bounds, &Default::default(), color, style);
    }

    pub fn draw_push_clip(&self, bounds: &Rect<Au>) {
        let rect = bounds.to_nearest_azure_rect(self.screen_pixels_per_px());
        let path_builder = self.draw_target.create_path_builder();

        let left_top = Point2D::new(rect.origin.x, rect.origin.y);
        let right_top = Point2D::new(rect.origin.x + rect.size.width, rect.origin.y);
        let left_bottom = Point2D::new(rect.origin.x, rect.origin.y + rect.size.height);
        let right_bottom = Point2D::new(rect.origin.x + rect.size.width,
                                        rect.origin.y + rect.size.height);

        path_builder.move_to(left_top);
        path_builder.line_to(right_top);
        path_builder.line_to(right_bottom);
        path_builder.line_to(left_bottom);

        let path = path_builder.finish();
        self.draw_target.push_clip(&path);
    }

    pub fn draw_pop_clip(&self) {
        self.draw_target.pop_clip();
    }

    pub fn draw_image(&self,
                      bounds: &Rect<Au>,
                      stretch_size: &Size2D<Au>,
                      image_info: &WebRenderImageInfo,
                      image_data: &[u8],
                      image_rendering: image_rendering::T) {
        let size = Size2D::new(image_info.width as i32, image_info.height as i32);
        let (pixel_width, source_format) = match image_info.format {
            PixelFormat::RGBA8 => (4, SurfaceFormat::B8G8R8A8),
            PixelFormat::K8 => (1, SurfaceFormat::A8),
            PixelFormat::RGB8 => panic!("RGB8 color type not supported"),
            PixelFormat::KA8 => panic!("KA8 color type not supported"),
        };
        let stride = image_info.width * pixel_width;
        let scale = self.screen_pixels_per_px();

        self.draw_target.make_current();
        let draw_target_ref = &self.draw_target;
        let azure_surface = match draw_target_ref.create_source_surface_from_data(image_data,
                                                                                  size,
                                                                                  stride as i32,
                                                                                  source_format) {
            Some(azure_surface) => azure_surface,
            None => return,
        };

        let source_rect = Rect::new(Point2D::new(0.0, 0.0),
                                    Size2D::new(image_info.width as AzFloat,
                                                image_info.height as AzFloat));
        let dest_rect = bounds.to_nearest_azure_rect(scale);

        // TODO(pcwalton): According to CSS-IMAGES-3 § 5.3, nearest-neighbor interpolation is a
        // conforming implementation of `crisp-edges`, but it is not the best we could do.
        // Something like Scale2x would be ideal.
        let draw_surface_filter = match image_rendering {
            image_rendering::T::Auto => Filter::Linear,
            image_rendering::T::CrispEdges | image_rendering::T::Pixelated => Filter::Point,
        };

        let draw_surface_options = DrawSurfaceOptions::new(draw_surface_filter, true);
        let draw_options = DrawOptions::new(1.0, CompositionOp::Over, AntialiasMode::None);

        // Fast path: No need to create a pattern.
        if bounds.size == *stretch_size {
            draw_target_ref.draw_surface(azure_surface,
                                         dest_rect,
                                         source_rect,
                                         draw_surface_options,
                                         draw_options);
            return
        }

        // Slightly slower path: No need to stretch.
        //
        // Annoyingly, surface patterns in Azure/Skia are relative to the top left of the *canvas*,
        // not the rectangle we're drawing to. So we need to translate it explicitly.
        let matrix = Matrix2D::identity().translate(dest_rect.origin.x, dest_rect.origin.y);
        let stretch_size = stretch_size.to_nearest_azure_size(scale);
        if source_rect.size == stretch_size {
            let pattern = SurfacePattern::new(azure_surface.azure_source_surface,
                                              true,
                                              true,
                                              &matrix);
            draw_target_ref.fill_rect(&dest_rect,
                                      PatternRef::Surface(&pattern),
                                      Some(&draw_options));
            return
        }

        // Slow path: Both stretch and a pattern are needed.
        let draw_surface_options = DrawSurfaceOptions::new(draw_surface_filter, true);
        let draw_options = DrawOptions::new(1.0, CompositionOp::Over, AntialiasMode::None);
        let temporary_draw_target =
            self.draw_target.create_similar_draw_target(&stretch_size.to_azure_int_size(),
                                                        self.draw_target.get_format());
        let temporary_dest_rect = Rect::new(Point2D::new(0.0, 0.0), stretch_size);
        temporary_draw_target.draw_surface(azure_surface,
                                           temporary_dest_rect,
                                           source_rect,
                                           draw_surface_options,
                                           draw_options);

        let temporary_surface = temporary_draw_target.snapshot();
        let pattern = SurfacePattern::new(temporary_surface.azure_source_surface,
                                          true,
                                          true,
                                          &matrix);
        draw_target_ref.fill_rect(&dest_rect, PatternRef::Surface(&pattern), None);
    }

    pub fn clear(&self) {
        let pattern = ColorPattern::new(color::transparent());
        let page_rect = self.page_rect.to_untyped();
        let screen_rect = self.screen_rect.to_untyped();
        let rect = Rect::new(Point2D::new(page_rect.origin.x as AzFloat,
                                          page_rect.origin.y as AzFloat),
                             Size2D::new(screen_rect.size.width as AzFloat,
                                         screen_rect.size.height as AzFloat));
        let mut draw_options = DrawOptions::new(1.0, CompositionOp::Over, AntialiasMode::None);
        draw_options.set_composition_op(CompositionOp::Source);
        self.draw_target.make_current();
        self.draw_target.fill_rect(&rect, PatternRef::Color(&pattern), Some(&draw_options));
    }

    fn draw_border_segment(&self,
                           direction: Direction,
                           bounds: &Rect<Au>,
                           border: &SideOffsets2D<f32>,
                           radius: &BorderRadii<AzFloat>,
                           color: &SideOffsets2D<Color>,
                           style: &SideOffsets2D<border_style::T>) {
        let (style_select, color_select) = match direction {
            Direction::Top => (style.top, color.top),
            Direction::Left => (style.left, color.left),
            Direction::Right => (style.right, color.right),
            Direction::Bottom => (style.bottom, color.bottom)
        };

        match style_select {
            border_style::T::none | border_style::T::hidden => {}
            border_style::T::dotted => {
                // FIXME(sammykim): This doesn't work well with dash_pattern and cap_style.
                self.draw_dashed_border_segment(direction,
                                                bounds,
                                                border,
                                                radius,
                                                color_select,
                                                DashSize::DottedBorder);
            }
            border_style::T::dashed => {
                self.draw_dashed_border_segment(direction,
                                                bounds,
                                                border,
                                                radius,
                                                color_select,
                                                DashSize::DashedBorder);
            }
            border_style::T::solid => {
                self.draw_solid_border_segment(direction, bounds, border, radius, color_select);
            }
            border_style::T::double => {
                self.draw_double_border_segment(direction, bounds, border, radius, color_select);
            }
            border_style::T::groove | border_style::T::ridge => {
                self.draw_groove_ridge_border_segment(direction,
                                                      bounds,
                                                      border,
                                                      radius,
                                                      color_select,
                                                      style_select);
            }
            border_style::T::inset | border_style::T::outset => {
                self.draw_inset_outset_border_segment(direction,
                                                      bounds,
                                                      border,
                                                      radius,
                                                      color_select,
                                                      style_select);
            }
        }
    }

    fn draw_line_segment(&self,
                         bounds: &Rect<Au>,
                         radius: &BorderRadii<AzFloat>,
                         color: Color,
                         style: border_style::T) {
        let scale = self.screen_pixels_per_px();
        let border = SideOffsets2D::new_all_same(bounds.size.width).to_float_pixels(scale);
        match style {
            border_style::T::none | border_style::T::hidden => {}
            border_style::T::dotted => {
                self.draw_dashed_border_segment(Direction::Right,
                                                bounds,
                                                &border,
                                                radius,
                                                color,
                                                DashSize::DottedBorder);
            }
            border_style::T::dashed => {
                self.draw_dashed_border_segment(Direction::Right,
                                                bounds,
                                                &border,
                                                radius,
                                                color,
                                                DashSize::DashedBorder);
            }
            border_style::T::solid => {
                self.draw_solid_border_segment(Direction::Right, bounds, &border, radius, color)
            }
            border_style::T::double => {
                self.draw_double_border_segment(Direction::Right, bounds, &border, radius, color)
            }
            border_style::T::groove | border_style::T::ridge => {
                self.draw_groove_ridge_border_segment(Direction::Right,
                                                      bounds,
                                                      &border,
                                                      radius,
                                                      color,
                                                      style);
            }
            border_style::T::inset | border_style::T::outset => {
                self.draw_inset_outset_border_segment(Direction::Right,
                                                      bounds,
                                                      &border,
                                                      radius,
                                                      color,
                                                      style);
            }
        }
    }

    fn draw_border_path(&self,
                        bounds: &Rect<f32>,
                        direction: Direction,
                        border: &SideOffsets2D<f32>,
                        radii: &BorderRadii<AzFloat>,
                        color: Color) {
        let mut path_builder = self.draw_target.create_path_builder();
        self.create_border_path_segment(&mut path_builder,
                                        bounds,
                                        direction,
                                        border,
                                        radii,
                                        BorderPathDrawingMode::EntireBorder);
        let draw_options = DrawOptions::new(1.0, CompositionOp::Over, AntialiasMode::None);
        self.draw_target.fill(&path_builder.finish(),
                              Pattern::Color(ColorPattern::new(color)).to_pattern_ref(),
                              &draw_options);
    }

    fn push_rounded_rect_clip(&self, bounds: &Rect<f32>, radii: &BorderRadii<AzFloat>) {
        let mut path_builder = self.draw_target.create_path_builder();
        self.create_rounded_rect_path(&mut path_builder, bounds, radii);
        self.draw_target.push_clip(&path_builder.finish());
    }

    fn solve_quadratic(a: f32, b: f32, c: f32) -> (Option<f32>, Option<f32>) {
        let discriminant = b * b - 4. * a * c;
        if discriminant < 0. {
            return (None, None);
        }
        let x1 = (-b + discriminant.sqrt())/(2. * a);
        let x2 = (-b - discriminant.sqrt())/(2. * a);
        if discriminant == 0. {
            return (Some(x1), None);
        }
        (Some(x1), Some(x2))
    }

    fn intersect_ellipse_line(mut e: Ellipse, mut line: Line) -> (Option<Point2D<f32>>,
                                                                  Option<Point2D<f32>>) {
        let mut rotated_axes = false;
        fn rotate_axes(point: Point2D<f32>, clockwise: bool) -> Point2D<f32> {
            if clockwise {
                // rotate clockwise by 90 degrees
                Point2D::new(point.y, -point.x)
            } else {
                // rotate counter clockwise by 90 degrees
                Point2D::new(-point.y, point.x)
            }
        }

        // if line height is greater than its width then rotate the axes by 90 degrees,
        // i.e. (x, y) -> (y, -x).
        if (line.end.x - line.start.x).abs() < (line.end.y - line.start.y).abs() {
            rotated_axes = true;
            line = Line::new(rotate_axes(line.start, true), rotate_axes(line.end, true));
            e = Ellipse { origin: rotate_axes(e.origin, true),
                          width: e.height, height: e.width };
        }
        debug_assert!(line.end.x - line.start.x > f32::EPSILON,
                      "Error line segment end.x ({}) <= start.x ({})!", line.end.x, line.start.x);
        // shift the origin to center of the ellipse.
        line = Line::new(line.start - e.origin, line.end - e.origin);
        let a = (line.end.y - line.start.y)/(line.end.x - line.start.x);
        let b = line.start.y - (a * line.start.x);
        // given the equation of a line,
        // y = a * x + b,
        // and the equation of an ellipse,
        // x^2/w^2 + y^2/h^2 = 1,
        // substitute y = a * x + b, giving
        // x^2/w^2 + (a^2x^2 + 2abx + b^2)/h^2 = 1
        // then simplify to
        // (h^2 + w^2a^2)x^2 + 2abw^2x + (b^2w^2 - w^2h^2) = 0
        // finally solve for w using the quadratic equation.
        let w = e.width;
        let h = e.height;
        let quad_a = h * h + w * w * a * a;
        let quad_b = 2. * a * b * w * w;
        let quad_c = b * b * w * w - w * w * h * h;
        let intersections = PaintContext::solve_quadratic(quad_a, quad_b, quad_c);
        match intersections {
            (Some(x0), Some(x1)) => {
                let mut p0 = Point2D::new(x0, a * x0 + b) + e.origin;
                let mut p1 = Point2D::new(x1, a * x1 + b) + e.origin;
                if x0 > x1 {
                    mem::swap(&mut p0, &mut p1);
                }
                if rotated_axes {
                    p0 = rotate_axes(p0, false);
                    p1 = rotate_axes(p1, false);
                }
                (Some(p0), Some(p1))
            },
            (Some(x0), None) | (None, Some(x0)) => {
                let mut p = Point2D::new(x0, a * x0 + b) + e.origin;
                if rotated_axes {
                    p = rotate_axes(p, false);
                }
                (Some(p), None)
            },
            (None, None) => (None, None),
        }
    }

    // Given an ellipse and line segment, the line segment may intersect the
    // ellipse at 0, 1, or 2 points. We compute those intersection points.
    // For each intersection point the angle of the point on the ellipse relative to
    // the top|bottom of the ellipse is computed.
    // Examples:
    // - intersection at ellipse.center + (0, ellipse.height), the angle is 0 rad.
    // - intersection at ellipse.center + (0, -ellipse.height), the angle is 0 rad.
    // - intersection at ellipse.center + (+-ellipse.width, 0), the angle is pi/2.
    fn ellipse_line_intersection_angles(e: Ellipse, l: Line)
                                        -> (Option<(Point2D<f32>, f32)>, Option<(Point2D<f32>, f32)>) {
        fn point_angle(e: Ellipse, intersect_point: Point2D<f32>) -> f32 {
            ((intersect_point.y - e.origin.y).abs() / e.height).asin()
        }

        let intersection = PaintContext::intersect_ellipse_line(e, l);
        match intersection {
            (Some(p0), Some(p1)) => (Some((p0, point_angle(e, p0))), Some((p1, point_angle(e, p1)))),
            (Some(p0), None) => (Some((p0, point_angle(e, p0))), None),
            (None, Some(p1)) => (None, Some((p1, point_angle(e, p1)))),
            (None, None) => (None, None),
        }
    }

    fn ellipse_rightmost_intersection(e: Ellipse, l: Line) -> Option<f32> {
        match PaintContext::ellipse_line_intersection_angles(e, l) {
            (Some((p0, angle0)), Some((p1, _))) if p0.x > p1.x => Some(angle0),
            (_, Some((_, angle1))) => Some(angle1),
            (Some((_, angle0)), None) => Some(angle0),
            (None, None) => None,
        }
    }

    fn ellipse_leftmost_intersection(e: Ellipse, l: Line) -> Option<f32> {
        match PaintContext::ellipse_line_intersection_angles(e, l) {
            (Some((p0, angle0)), Some((p1, _))) if p0.x < p1.x => Some(angle0),
            (_, Some((_, angle1))) => Some(angle1),
            (Some((_, angle0)), None) => Some(angle0),
            (None, None) => None,
        }
    }

    fn is_zero_radius(radius: &Size2D<AzFloat>) -> bool {
        radius.width <= 0. || radius.height <= 0.
    }

    // The following comment is wonderful, and stolen from
    // gecko:gfx/thebes/gfxContext.cpp:RoundedRectangle for reference.
    // ---------------------------------------------------------------
    //
    // For CW drawing, this looks like:
    //
    //  ...******0**      1    C
    //              ****
    //                  ***    2
    //                     **
    //                       *
    //                        *
    //                         3
    //                         *
    //                         *
    //
    // Where 0, 1, 2, 3 are the control points of the Bezier curve for
    // the corner, and C is the actual corner point.
    //
    // At the start of the loop, the current point is assumed to be
    // the point adjacent to the top left corner on the top
    // horizontal.  Note that corner indices start at the top left and
    // continue clockwise, whereas in our loop i = 0 refers to the top
    // right corner.
    //
    // When going CCW, the control points are swapped, and the first
    // corner that's drawn is the top left (along with the top segment).
    //
    // There is considerable latitude in how one chooses the four
    // control points for a Bezier curve approximation to an ellipse.
    // For the overall path to be continuous and show no corner at the
    // endpoints of the arc, points 0 and 3 must be at the ends of the
    // straight segments of the rectangle; points 0, 1, and C must be
    // collinear; and points 3, 2, and C must also be collinear.  This
    // leaves only two free parameters: the ratio of the line segments
    // 01 and 0C, and the ratio of the line segments 32 and 3C.  See
    // the following papers for extensive discussion of how to choose
    // these ratios:
    //
    //   Dokken, Tor, et al. "Good approximation of circles by
    //      curvature-continuous Bezier curves."  Computer-Aided
    //      Geometric Design 7(1990) 33--41.
    //   Goldapp, Michael. "Approximation of circular arcs by cubic
    //      polynomials." Computer-Aided Geometric Design 8(1991) 227--238.
    //   Maisonobe, Luc. "Drawing an elliptical arc using polylines,
    //      quadratic, or cubic Bezier curves."
    //      http://www.spaceroots.org/documents/ellipse/elliptical-arc.pdf
    //
    // We follow the approach in section 2 of Goldapp (least-error,
    // Hermite-type approximation) and make both ratios equal to
    //
    //          2   2 + n - sqrt(2n + 28)
    //  alpha = - * ---------------------
    //          3           n - 4
    //
    // where n = 3( cbrt(sqrt(2)+1) - cbrt(sqrt(2)-1) ).
    //
    // This is the result of Goldapp's equation (10b) when the angle
    // swept out by the arc is pi/2, and the parameter "a-bar" is the
    // expression given immediately below equation (21).
    //
    // Using this value, the maximum radial error for a circle, as a
    // fraction of the radius, is on the order of 0.2 x 10^-3.
    // Neither Dokken nor Goldapp discusses error for a general
    // ellipse; Maisonobe does, but his choice of control points
    // follows different constraints, and Goldapp's expression for
    // 'alpha' gives much smaller radial error, even for very flat
    // ellipses, than Maisonobe's equivalent.
    //
    // For the various corners and for each axis, the sign of this
    // constant changes, or it might be 0 -- it's multiplied by the
    // appropriate multiplier from the list before using.
    // ---------------------------------------------------------------
    //
    // Code adapted from gecko:gfx/2d/PathHelpers.h:EllipseToBezier
    fn ellipse_to_bezier(path_builder: &mut PathBuilder,
                         origin: Point2D<AzFloat>,
                         radius: Size2D<AzFloat>,
                         start_angle: f32,
                         end_angle: f32) {
        if PaintContext::is_zero_radius(&radius) {
            return;
        }

        // Calculate kappa constant for partial curve. The sign of angle in the
        // tangent will actually ensure this is negative for a counter clockwise
        // sweep, so changing signs later isn't needed.
        let kappa_factor: f32 = (4.0f32 / 3.0f32) * ((end_angle - start_angle) / 4.).tan();
        let kappa_x: f32 = kappa_factor * radius.width;
        let kappa_y: f32 = kappa_factor * radius.height;

        // We guarantee here the current point is the start point of the next
        // curve segment.
        let start_point = Point2D::new(origin.x + start_angle.cos() * radius.width,
                                       origin.y + start_angle.sin() * radius.height);

        path_builder.line_to(start_point);
        let end_point = Point2D::new(origin.x + end_angle.cos() * radius.width,
                                     origin.y + end_angle.sin() * radius.height);

        let tangent_start = Point2D::new(-start_angle.sin(), start_angle.cos());

        let cp1 = Point2D::new(start_point.x + tangent_start.x * kappa_x,
                               start_point.y + tangent_start.y * kappa_y);

        let rev_tangent_end = Point2D::new(end_angle.sin(), -end_angle.cos());

        let cp2 = Point2D::new(end_point.x + rev_tangent_end.x * kappa_x,
                               end_point.y + rev_tangent_end.y * kappa_y);

        path_builder.bezier_curve_to(&cp1, &cp2, &end_point);
    }

    #[allow(non_snake_case)]
    fn inner_border_bounds(bounds: &Rect<f32>, border: &SideOffsets2D<f32>) -> Rect<f32> {
        // T = top, B = bottom, L = left, R = right
        let inner_TL = bounds.origin + Point2D::new(border.left, border.top);
        let inner_BR = bounds.bottom_right() + Point2D::new(-border.right, -border.bottom);
        Rect::new(inner_TL, Size2D::new(inner_BR.x - inner_TL.x, inner_BR.y - inner_TL.y))
    }

    #[allow(non_snake_case)]
    fn corner_bounds(bounds: &Rect<f32>,
                     border: &SideOffsets2D<f32>,
                     radii: &BorderRadii<AzFloat>) -> (CornerOrigin, SideOffsets2D<Size2D<f32>>) {
        fn distance_to_elbow(radius: &Size2D<AzFloat>,
                             corner_width: f32,
                             corner_height: f32) -> Size2D<f32> {
            if corner_width >= radius.width || corner_height >= radius.height {
                Size2D::zero()
            } else {
                Size2D::new(radius.width - corner_width, radius.height - corner_height)
            }
        }

        // T = top, B = bottom, L = left, R = right
        let origin_TL = bounds.origin + Point2D::new(radii.top_left.width, radii.top_left.height);
        let origin_TR = bounds.top_right() + Point2D::new(-radii.top_right.width,
                                                          radii.top_right.height);
        let origin_BR = bounds.bottom_right() + Point2D::new(-radii.bottom_right.width,
                                                             -radii.bottom_right.height);
        let origin_BL = bounds.bottom_left() + Point2D::new(radii.bottom_left.width,
                                                            -radii.bottom_left.height);

        let elbow_TL = distance_to_elbow(&radii.top_left, border.left, border.top);
        let elbow_TR = distance_to_elbow(&radii.top_right, border.right, border.top);
        let elbow_BR = distance_to_elbow(&radii.bottom_right, border.right, border.bottom);
        let elbow_BL = distance_to_elbow(&radii.bottom_left, border.left, border.bottom);
        (CornerOrigin { top_left: origin_TL,
                        top_right: origin_TR,
                        bottom_right: origin_BR,
                        bottom_left: origin_BL },
         SideOffsets2D::new(elbow_TL, elbow_TR, elbow_BR, elbow_BL))
    }

    /// `origin` is the origin point when drawing the corner e.g. it's the circle center
    ///  when drawing radial borders.
    ///
    /// `corner` indicates which corner to draw e.g. top left or top right etc.
    ///
    /// `radius` is the border-radius width and height. If `radius.width == radius.height` then
    ///  an arc from a circle is drawn instead of an arc from an ellipse.
    ///
    /// `inner_border` & `outer_border` are the inner and outer points on the border corner
    ///  respectively. ASCII diagram:
    ///      ---------------* =====> ("*" is the `outer_border` point)
    ///                     |
    ///                     |
    ///                     |
    ///      --------* ============> ("*" is the `inner_border` point)
    ///              |      |
    ///              |      |
    ///
    ///
    ///  `dist_elbow` is the distance from `origin` to the inner part of the border corner.
    ///  `clockwise` indicates direction to draw the border curve.
    #[allow(non_snake_case)]
    fn draw_corner(path_builder: &mut PathBuilder,
                   corner: BorderCorner,
                   origin: &Point2D<AzFloat>,
                   radius: &Size2D<AzFloat>,
                   inner_border: &Point2D<AzFloat>,
                   outer_border: &Point2D<AzFloat>,
                   dist_elbow: &Size2D<AzFloat>,
                   clockwise: bool) {
        let rad_R: AzFloat = 0.;
        let rad_BR = rad_R  + f32::consts::FRAC_PI_4;
        let rad_B  = rad_BR + f32::consts::FRAC_PI_4;
        let rad_BL = rad_B  + f32::consts::FRAC_PI_4;
        let rad_L  = rad_BL + f32::consts::FRAC_PI_4;
        let rad_TL = rad_L  + f32::consts::FRAC_PI_4;
        let rad_T  = rad_TL + f32::consts::FRAC_PI_4;

        // Returns true if the angular size for this border corner
        // is PI/4.
        fn simple_border_corner(border_corner_radius: &Size2D<f32>,
                                border1_width: f32,
                                border2_width: f32) -> bool {
            (border_corner_radius.width - border_corner_radius.height).abs() <= f32::EPSILON &&
                (border1_width - border2_width).abs() <= f32::EPSILON
        }

        if PaintContext::is_zero_radius(radius) {
            return;
        }
        let ellipse = Ellipse { origin: *origin, width: radius.width, height: radius.height };
        let simple_border = simple_border_corner(&radius,
                                                 (outer_border.x - inner_border.x).abs(),
                                                 (outer_border.y - inner_border.y).abs());
        let corner_angle = if simple_border {
            f32::consts::FRAC_PI_4
        } else {
            let corner_line = Line::new(*inner_border, *outer_border);
            match corner {
                BorderCorner::TopLeft | BorderCorner::BottomLeft =>
                    PaintContext::ellipse_leftmost_intersection(ellipse, corner_line).unwrap(),
                BorderCorner::TopRight | BorderCorner::BottomRight =>
                    PaintContext::ellipse_rightmost_intersection(ellipse, corner_line).unwrap(),
            }
        };
        let (start_angle, end_angle) = match corner {
            // TR corner - top border & right border
            BorderCorner::TopRight =>
                if clockwise { (-rad_B, rad_R - corner_angle) } else { (rad_R - corner_angle, rad_R) },
            // BR corner - right border & bottom border
            BorderCorner::BottomRight =>
                if clockwise { (rad_R, rad_R + corner_angle) } else { (rad_R + corner_angle, rad_B) },
            // TL corner - left border & top border
            BorderCorner::TopLeft =>
                if clockwise { (rad_L, rad_L + corner_angle) } else { (rad_L + corner_angle, rad_T) },
            // BL corner - bottom border & left border
            BorderCorner::BottomLeft =>
                if clockwise { (rad_B, rad_L - corner_angle) } else { (rad_L - corner_angle, rad_L) },
        };
        if clockwise {
            PaintContext::ellipse_to_bezier(path_builder, *origin, *radius, start_angle, end_angle);
            PaintContext::ellipse_to_bezier(path_builder, *origin, *dist_elbow, end_angle, start_angle);
        } else {
            PaintContext::ellipse_to_bezier(path_builder, *origin, *dist_elbow, end_angle, start_angle);
            PaintContext::ellipse_to_bezier(path_builder, *origin, *radius, start_angle, end_angle);
        }
    }

    #[allow(non_snake_case)]
    fn create_border_path_segment(&self,
                                  path_builder: &mut PathBuilder,
                                  bounds: &Rect<f32>,
                                  direction: Direction,
                                  border: &SideOffsets2D<f32>,
                                  radii: &BorderRadii<AzFloat>,
                                  mode: BorderPathDrawingMode) {
        // T = top, B = bottom, L = left, R = right
        let inner = PaintContext::inner_border_bounds(bounds, &border);
        let (box_TL, inner_TL,
             box_TR, inner_TR,
             box_BR, inner_BR,
             box_BL, inner_BL) = (bounds.origin, inner.origin,
                                  bounds.top_right(), inner.top_right(),
                                  bounds.bottom_right(), inner.bottom_right(),
                                  bounds.bottom_left(), inner.bottom_left());

        fn dx(x: AzFloat) -> Point2D<AzFloat> {
            Point2D::new(x, 0.)
        }

        fn dy(y: AzFloat) -> Point2D<AzFloat> {
            Point2D::new(0., y)
        }

        fn dx_if(cond: bool, dx: AzFloat) -> Point2D<AzFloat> {
            Point2D::new(if cond { dx } else { 0. }, 0.)
        }

        fn dy_if(cond: bool, dy: AzFloat) -> Point2D<AzFloat> {
            Point2D::new(0., if cond { dy } else { 0. })
        }

        let (corner_origin, elbow) =
            PaintContext::corner_bounds(bounds, border, radii);

        let (elbow_TL, elbow_TR, elbow_BR, elbow_BL) =
            (elbow.top, elbow.right, elbow.bottom, elbow.left);

        match direction {
            Direction::Top => {
                let edge_TL = box_TL + dx(radii.top_left.width.max(border.left));
                let edge_TR = box_TR + dx(-radii.top_right.width.max(border.right));
                let edge_BR = box_TR + dx(-border.right - elbow_TR.width) + dy(border.top);
                let edge_BL = box_TL + dx(border.left + elbow_TL.width) + dy(border.top);

                let corner_TL = edge_TL + dx_if(PaintContext::is_zero_radius(&radii.top_left),
                                                -border.left);
                let corner_TR = edge_TR + dx_if(PaintContext::is_zero_radius(&radii.top_right),
                                                border.right);

                match mode {
                    BorderPathDrawingMode::EntireBorder => {
                        path_builder.move_to(corner_TL);
                        path_builder.line_to(corner_TR);
                    }
                    BorderPathDrawingMode::CornersOnly => path_builder.move_to(corner_TR),
                }
                PaintContext::draw_corner(path_builder,
                                          BorderCorner::TopRight,
                                          &corner_origin.top_right,
                                          &radii.top_right,
                                          &inner_TR,
                                          &box_TR,
                                          &elbow_TR,
                                          true);
                match mode {
                    BorderPathDrawingMode::EntireBorder => {
                        path_builder.line_to(edge_BR);
                        path_builder.line_to(edge_BL);
                    }
                    BorderPathDrawingMode::CornersOnly => path_builder.move_to(edge_BL),
                }
                PaintContext::draw_corner(path_builder,
                                          BorderCorner::TopLeft,
                                          &corner_origin.top_left,
                                          &radii.top_left,
                                          &inner_TL,
                                          &box_TL,
                                          &elbow_TL,
                                          false);
            }
            Direction::Left => {
                let edge_TL = box_TL + dy(radii.top_left.height.max(border.top));
                let edge_BL = box_BL + dy(-radii.bottom_left.height.max(border.bottom));
                let edge_TR = box_TL + dx(border.left) + dy(border.top + elbow_TL.height);
                let edge_BR = box_BL + dx(border.left) + dy(-border.bottom -
                                                            elbow_BL.height);

                let corner_TL = edge_TL + dy_if(PaintContext::is_zero_radius(&radii.top_left),
                                                -border.top);
                let corner_BL = edge_BL + dy_if(PaintContext::is_zero_radius(&radii.bottom_left),
                                                border.bottom);

                match mode {
                    BorderPathDrawingMode::EntireBorder => {
                        path_builder.move_to(corner_BL);
                        path_builder.line_to(corner_TL);
                    }
                    BorderPathDrawingMode::CornersOnly => path_builder.move_to(corner_TL),
                }
                PaintContext::draw_corner(path_builder,
                                          BorderCorner::TopLeft,
                                          &corner_origin.top_left,
                                          &radii.top_left,
                                          &inner_TL,
                                          &box_TL,
                                          &elbow_TL,
                                          true);
                match mode {
                    BorderPathDrawingMode::EntireBorder => {
                        path_builder.line_to(edge_TR);
                        path_builder.line_to(edge_BR);
                    }
                    BorderPathDrawingMode::CornersOnly => path_builder.move_to(edge_BR),
                }
                PaintContext::draw_corner(path_builder,
                                          BorderCorner::BottomLeft,
                                          &corner_origin.bottom_left,
                                          &radii.bottom_left,
                                          &inner_BL,
                                          &box_BL,
                                          &elbow_BL,
                                          false);
            }
            Direction::Right => {
                let edge_TR = box_TR + dy(radii.top_right.height.max(border.top));
                let edge_BR = box_BR + dy(-radii.bottom_right.height.max(border.bottom));
                let edge_TL = box_TR + dx(-border.right) + dy(border.top + elbow_TR.height);
                let edge_BL = box_BR + dx(-border.right) + dy(-border.bottom -
                                                              elbow_BR.height);

                let corner_TR = edge_TR + dy_if(PaintContext::is_zero_radius(&radii.top_right),
                                                -border.top);
                let corner_BR = edge_BR + dy_if(PaintContext::is_zero_radius(&radii.bottom_right),
                                                border.bottom);

                match mode {
                    BorderPathDrawingMode::EntireBorder => {
                        path_builder.move_to(edge_BL);
                        path_builder.line_to(edge_TL);
                    }
                    BorderPathDrawingMode::CornersOnly => path_builder.move_to(edge_TL),
                }
                PaintContext::draw_corner(path_builder,
                                          BorderCorner::TopRight,
                                          &corner_origin.top_right,
                                          &radii.top_right,
                                          &inner_TR,
                                          &box_TR,
                                          &elbow_TR,
                                          false);
                match mode {
                    BorderPathDrawingMode::EntireBorder => {
                        path_builder.line_to(corner_TR);
                        path_builder.line_to(corner_BR);
                    }
                    BorderPathDrawingMode::CornersOnly => path_builder.move_to(corner_BR),
                }
                PaintContext::draw_corner(path_builder,
                                          BorderCorner::BottomRight,
                                          &corner_origin.bottom_right,
                                          &radii.bottom_right,
                                          &inner_BR,
                                          &box_BR,
                                          &elbow_BR,
                                          true);

            }
            Direction::Bottom => {
                let edge_BL = box_BL + dx(radii.bottom_left.width.max(border.left));
                let edge_BR = box_BR + dx(-radii.bottom_right.width.max(border.right));
                let edge_TL = box_BL + dy(-border.bottom) + dx(border.left +
                                                               elbow_BL.width);
                let edge_TR = box_BR + dy(-border.bottom) + dx(-border.right -
                                                               elbow_BR.width);

                let corner_BR = edge_BR + dx_if(PaintContext::is_zero_radius(&radii.bottom_right),
                                                border.right);
                let corner_BL = edge_BL + dx_if(PaintContext::is_zero_radius(&radii.bottom_left),
                                                -border.left);

                match mode {
                    BorderPathDrawingMode::EntireBorder => {
                        path_builder.move_to(edge_TL);
                        path_builder.line_to(edge_TR);
                    }
                    BorderPathDrawingMode::CornersOnly => path_builder.move_to(edge_TR),
                }
                PaintContext::draw_corner(path_builder,
                                          BorderCorner::BottomRight,
                                          &corner_origin.bottom_right,
                                          &radii.bottom_right,
                                          &inner_BR,
                                          &box_BR,
                                          &elbow_BR,
                                          false);
                match mode {
                    BorderPathDrawingMode::EntireBorder => {
                        path_builder.line_to(corner_BR);
                        path_builder.line_to(corner_BL);
                    }
                    BorderPathDrawingMode::CornersOnly => path_builder.move_to(corner_BL),
                }
                PaintContext::draw_corner(path_builder,
                                          BorderCorner::BottomLeft,
                                          &corner_origin.bottom_left,
                                          &radii.bottom_left,
                                          &inner_BL,
                                          &box_BL,
                                          &elbow_BL,
                                          true);
            }
        }
    }

    /// Creates a path representing the given rounded rectangle.
    ///
    /// TODO(pcwalton): Should we unify with the code above? It doesn't seem immediately obvious
    /// how to do that (especially without regressing performance) unless we have some way to
    /// efficiently intersect or union paths, since different border styles/colors can force us to
    /// slice through the rounded corners. My first attempt to unify with the above code resulted
    /// in making a mess of it, and the simplicity of this code path is appealing, so it may not
    /// be worth it… In any case, revisit this decision when we support elliptical radii.
    #[allow(non_snake_case)]
    fn create_rounded_rect_path(&self,
                                path_builder: &mut PathBuilder,
                                bounds: &Rect<f32>,
                                radii: &BorderRadii<AzFloat>) {
        //    +----------+
        //   / 1        2 \
        //  + 8          3 +
        //  |              |
        //  + 7          4 +
        //   \ 6        5 /
        //    +----------+
        let border = SideOffsets2D::new(radii.top_left.height.max(radii.top_right.height),
                                        radii.bottom_right.width.max(radii.top_right.width),
                                        radii.bottom_right.height.max(radii.bottom_left.height),
                                        radii.top_left.height.max(radii.bottom_left.height));

        // T = top, B = bottom, L = left, R = right
        let inner = PaintContext::inner_border_bounds(bounds, &border);
        let (outer_TL, inner_TL,
             outer_TR, inner_TR,
             outer_BR, inner_BR,
             outer_BL, inner_BL) = (bounds.origin, inner.origin,
                                    bounds.top_right(), inner.top_right(),
                                    bounds.bottom_right(), inner.bottom_right(),
                                    bounds.bottom_left(), inner.bottom_left());

        let (corner_origin, _) =
            PaintContext::corner_bounds(bounds, &border, radii);
        let (origin_TL, origin_TR, origin_BR, origin_BL) = (corner_origin.top_left,
                                                            corner_origin.top_right,
                                                            corner_origin.bottom_right,
                                                            corner_origin.bottom_left);
        let zero_elbow = Size2D::new(0., 0.);

        path_builder.move_to(Point2D::new(origin_TL.x - radii.top_left.width, origin_TL.y));
        path_builder.move_to(Point2D::new(bounds.origin.x + radii.top_left.width, bounds.origin.y));   // 1
        path_builder.line_to(Point2D::new(bounds.max_x() - radii.top_right.width, bounds.origin.y));   // 2
        PaintContext::draw_corner(path_builder,                                                        // 3
                                  BorderCorner::TopRight,
                                  &origin_TR,
                                  &radii.top_right,
                                  &inner_TR,
                                  &outer_TR,
                                  &zero_elbow,
                                  true);
        PaintContext::draw_corner(path_builder,                                                        // 3
                                  BorderCorner::TopRight,
                                  &origin_TR,
                                  &radii.top_right,
                                  &inner_TR,
                                  &outer_TR,
                                  &zero_elbow,
                                  false);
        path_builder.line_to(Point2D::new(bounds.max_x(), bounds.max_y() - radii.bottom_right.width)); // 4
        PaintContext::draw_corner(path_builder,                                                        // 5
                                  BorderCorner::BottomRight,
                                  &origin_BR,
                                  &radii.bottom_right,
                                  &inner_BR,
                                  &outer_BR,
                                  &zero_elbow,
                                  true);
        PaintContext::draw_corner(path_builder,                                                        // 5
                                  BorderCorner::BottomRight,
                                  &origin_BR,
                                  &radii.bottom_right,
                                  &inner_BR,
                                  &outer_BR,
                                  &zero_elbow,
                                  false);
        path_builder.line_to(Point2D::new(bounds.origin.x + radii.bottom_left.width,
                                          bounds.max_y()));                                            // 6
        PaintContext::draw_corner(path_builder,                                                        // 7
                                  BorderCorner::BottomLeft,
                                  &origin_BL,
                                  &radii.bottom_left,
                                  &inner_BL,
                                  &outer_BL,
                                  &zero_elbow,
                                  true);
        PaintContext::draw_corner(path_builder,                                                        // 7
                                  BorderCorner::BottomLeft,
                                  &origin_BL,
                                  &radii.bottom_left,
                                  &inner_BL,
                                  &outer_BL,
                                  &zero_elbow,
                                  false);
        path_builder.line_to(Point2D::new(bounds.origin.x,
                                          bounds.origin.y + radii.top_left.height));                    // 8
        PaintContext::draw_corner(path_builder,                                                         // 9
                                  BorderCorner::TopLeft,
                                  &origin_TL,
                                  &radii.top_left,
                                  &inner_TL,
                                  &outer_TL,
                                  &zero_elbow,
                                  true);
        PaintContext::draw_corner(path_builder,                                                         // 9
                                  BorderCorner::TopLeft,
                                  &origin_TL,
                                  &radii.top_left,
                                  &inner_TL,
                                  &outer_TL,
                                  &zero_elbow,
                                  false);
    }

    fn draw_dashed_border_segment(&self,
                                  direction: Direction,
                                  bounds: &Rect<Au>,
                                  border: &SideOffsets2D<f32>,
                                  radius: &BorderRadii<AzFloat>,
                                  color: Color,
                                  dash_size: DashSize) {
        let rect = bounds.to_nearest_azure_rect(self.screen_pixels_per_px());
        let draw_opts = DrawOptions::new(1.0, CompositionOp::Over, AntialiasMode::None);
        let border_width = match direction {
            Direction::Top => border.top,
            Direction::Left => border.left,
            Direction::Right => border.right,
            Direction::Bottom => border.bottom
        };
        let dash_pattern = [border_width * (dash_size as i32) as AzFloat,
                            border_width * (dash_size as i32) as AzFloat];
        let stroke_opts = StrokeOptions::new(border_width as AzFloat,
                                             JoinStyle::MiterOrBevel,
                                             CapStyle::Butt,
                                             10 as AzFloat,
                                             &dash_pattern);
        let (start, end)  = match direction {
            Direction::Top => {
                let y = rect.origin.y + border.top * 0.5;
                let start = Point2D::new(rect.origin.x + radius.top_left.width, y);
                let end = Point2D::new(rect.origin.x + rect.size.width - radius.top_right.width, y);
                (start, end)
            }
            Direction::Left => {
                let x = rect.origin.x + border.left * 0.5;
                let start = Point2D::new(x, rect.origin.y + rect.size.height - radius.bottom_left.height);
                let end = Point2D::new(x, rect.origin.y + border.top.max(radius.top_left.height));
                (start, end)
            }
            Direction::Right => {
                let x = rect.origin.x + rect.size.width - border.right * 0.5;
                let start = Point2D::new(x, rect.origin.y + radius.top_right.height);
                let end = Point2D::new(x, rect.origin.y + rect.size.height - radius.bottom_right.height);
                (start, end)
            }
            Direction::Bottom => {
                let y = rect.origin.y + rect.size.height - border.bottom * 0.5;
                let start = Point2D::new(rect.origin.x + rect.size.width - radius.bottom_right.width, y);
                let end = Point2D::new(rect.origin.x + border.left.max(radius.bottom_left.width), y);
                (start, end)
            }
        };

        self.draw_target.stroke_line(start,
                                     end,
                                     PatternRef::Color(&ColorPattern::new(color)),
                                     &stroke_opts,
                                     &draw_opts);

        if radii_apply_to_border_direction(direction, radius) {
            let mut path_builder = self.draw_target.create_path_builder();
            self.create_border_path_segment(&mut path_builder,
                                            &rect,
                                            direction,
                                            border,
                                            radius,
                                            BorderPathDrawingMode::CornersOnly);
            self.draw_target.fill(&path_builder.finish(),
                                  Pattern::Color(ColorPattern::new(color)).to_pattern_ref(),
                                  &draw_opts);
        }
    }

    fn draw_solid_border_segment(&self,
                                 direction: Direction,
                                 bounds: &Rect<Au>,
                                 border: &SideOffsets2D<f32>,
                                 radius: &BorderRadii<AzFloat>,
                                 color: Color) {
        let rect = bounds.to_nearest_azure_rect(self.screen_pixels_per_px());
        self.draw_border_path(&rect, direction, border, radius, color);
    }

    fn compute_scaled_bounds(&self,
                             bounds: &Rect<Au>,
                             border: &SideOffsets2D<f32>,
                             shrink_factor: f32) -> Rect<f32> {
        let rect            = bounds.to_nearest_azure_rect(self.screen_pixels_per_px());
        let scaled_border   = SideOffsets2D::new(shrink_factor * border.top,
                                                 shrink_factor * border.right,
                                                 shrink_factor * border.bottom,
                                                 shrink_factor * border.left);
        let left_top        = Point2D::new(rect.origin.x, rect.origin.y);
        let scaled_left_top = left_top + Point2D::new(scaled_border.left,
                                                      scaled_border.top);
        Rect::new(scaled_left_top,
                  Size2D::new(rect.size.width - 2.0 * scaled_border.right,
                              rect.size.height - 2.0 * scaled_border.bottom))
    }

    fn scale_color(&self, color: Color, scale_factor: f32) -> Color {
        color::new(color.r * scale_factor,
                   color.g * scale_factor,
                   color.b * scale_factor,
                   color.a)
    }

    fn draw_double_border_segment(&self,
                                  direction: Direction,
                                  bounds: &Rect<Au>,
                                  border: &SideOffsets2D<f32>,
                                  radius: &BorderRadii<AzFloat>,
                                  color: Color) {
        let scaled_border = SideOffsets2D::new((1.0 / 3.0) * border.top,
                                               (1.0 / 3.0) * border.right,
                                               (1.0 / 3.0) * border.bottom,
                                               (1.0 / 3.0) * border.left);
        let inner_scaled_bounds = self.compute_scaled_bounds(bounds, border, 2.0 / 3.0);
        // draw the outer portion of the double border.
        self.draw_solid_border_segment(direction, bounds, &scaled_border, radius, color);
        // draw the inner portion of the double border.
        self.draw_border_path(&inner_scaled_bounds, direction, &scaled_border, radius, color);
    }

    fn draw_groove_ridge_border_segment(&self,
                                        direction: Direction,
                                        bounds: &Rect<Au>,
                                        border: &SideOffsets2D<f32>,
                                        radius: &BorderRadii<AzFloat>,
                                        color: Color,
                                        style: border_style::T) {
        // original bounds as a Rect<f32>, with no scaling.
        let original_bounds            = self.compute_scaled_bounds(bounds, border, 0.0);
        // shrink the bounds by 1/2 of the border, leaving the innermost 1/2 of the border
        let inner_scaled_bounds        = self.compute_scaled_bounds(bounds, border, 0.5);
        let scaled_border              = SideOffsets2D::new(0.5 * border.top,
                                                            0.5 * border.right,
                                                            0.5 * border.bottom,
                                                            0.5 * border.left);
        let is_groove = match style {
                border_style::T::groove => true,
                border_style::T::ridge  => false,
                _ => panic!("invalid border style")
        };

        let lighter_color;
        let mut darker_color = color::black();
        if color != darker_color {
            darker_color = self.scale_color(color, if is_groove { 1.0 / 3.0 } else { 2.0 / 3.0 });
            lighter_color = color;
        } else {
            // You can't scale black color (i.e. 'scaled = 0 * scale', equals black).
            darker_color = color::new(0.3, 0.3, 0.3, color.a);
            lighter_color = color::new(0.7, 0.7, 0.7, color.a);
        }

        let (outer_color, inner_color) = match (direction, is_groove) {
            (Direction::Top, true) | (Direction::Left, true) |
            (Direction::Right, false) | (Direction::Bottom, false) => {
                (darker_color, lighter_color)
            }
            (Direction::Top, false) | (Direction::Left, false) |
            (Direction::Right, true) | (Direction::Bottom, true) => (lighter_color, darker_color),
        };
        // outer portion of the border
        self.draw_border_path(&original_bounds, direction, &scaled_border, radius, outer_color);
        // inner portion of the border
        self.draw_border_path(&inner_scaled_bounds,
                              direction,
                              &scaled_border,
                              radius,
                              inner_color);
    }

    fn draw_inset_outset_border_segment(&self,
                                        direction: Direction,
                                        bounds: &Rect<Au>,
                                        border: &SideOffsets2D<f32>,
                                        radius: &BorderRadii<AzFloat>,
                                        color: Color,
                                        style: border_style::T) {
        let is_inset = match style {
            border_style::T::inset  => true,
            border_style::T::outset => false,
            _ => panic!("invalid border style")
        };
        // original bounds as a Rect<f32>
        let original_bounds = self.compute_scaled_bounds(bounds, border, 0.0);

        // You can't scale black color (i.e. 'scaled = 0 * scale', equals black).
        let mut scaled_color = color::black();
        if color != scaled_color {
            scaled_color = match direction {
                Direction::Top | Direction::Left => {
                    self.scale_color(color, if is_inset { 2.0 / 3.0 } else { 1.0       })
                }
                Direction::Right | Direction::Bottom => {
                    self.scale_color(color, if is_inset { 1.0       } else { 2.0 / 3.0 })
                }
            };
        } else {
            scaled_color = match direction {
                Direction::Top | Direction::Left => {
                    if is_inset {
                        color::new(0.3, 0.3, 0.3, color.a)
                    } else {
                        color::new(0.7, 0.7, 0.7, color.a)
                    }
                }
                Direction::Right | Direction::Bottom => {
                    if is_inset {
                        color::new(0.7, 0.7, 0.7, color.a)
                    } else {
                        color::new(0.3, 0.3, 0.3, color.a)
                    }
                }
            };
        }

        self.draw_border_path(&original_bounds, direction, border, radius, scaled_color);
    }

    /// Draws the given text display item into the current context.
    pub fn draw_text(&mut self, text: &TextDisplayItem) {
        let draw_target_transform = self.draw_target.get_transform();

        // Optimization: Don’t set a transform matrix for upright text, and pass a start point to
        // `draw_text_into_context`.
        //
        // For sideways text, it’s easier to do the rotation such that its center (the baseline’s
        // start point) is at (0, 0) coordinates.
        let baseline_origin = match text.orientation {
            Upright => text.baseline_origin,
            SidewaysLeft => {
                let x = text.baseline_origin.x.to_f32_px();
                let y = text.baseline_origin.y.to_f32_px();
                self.draw_target.set_transform(&draw_target_transform.mul(&Matrix2D::new(0., -1.,
                                                                                         1., 0.,
                                                                                         x, y)));
                Point2D::zero()
            }
            SidewaysRight => {
                let x = text.baseline_origin.x.to_f32_px();
                let y = text.baseline_origin.y.to_f32_px();
                self.draw_target.set_transform(&draw_target_transform.mul(&Matrix2D::new(0., 1.,
                                                                                         -1., 0.,
                                                                                         x, y)));
                Point2D::zero()
            }
        };

        // Draw the text.
        let temporary_draw_target =
            self.create_draw_target_for_blur_if_necessary(&text.base.bounds, text.blur_radius);
        {
            // FIXME(https://github.com/rust-lang/rust/issues/23338)
            let font = self.font_context.paint_font_from_template(
                &text.text_run.font_template, text.text_run.actual_pt_size);
            font.borrow()
                .draw_text(&temporary_draw_target.draw_target,
                           &*text.text_run,
                           &text.range,
                           baseline_origin,
                           text.text_color,
                           opts::get().enable_text_antialiasing);
        }

        // Blur, if necessary.
        self.blur_if_necessary(temporary_draw_target, text.blur_radius);

        // Undo the transform, only when we did one.
        if text.orientation != Upright {
            self.draw_target.set_transform(&draw_target_transform)
        }
    }

    /// Draws a linear gradient in the given boundaries from the given start point to the given end
    /// point with the given stops.
    pub fn draw_linear_gradient(&self,
                                bounds: &Rect<Au>,
                                start_point: &Point2D<Au>,
                                end_point: &Point2D<Au>,
                                stops: &[GradientStop]) {
        self.draw_target.make_current();

        let stops = self.draw_target.create_gradient_stops(stops, ExtendMode::Clamp);
        let scale = self.screen_pixels_per_px();
        let pattern = LinearGradientPattern::new(&start_point.to_nearest_azure_point(scale),
                                                 &end_point.to_nearest_azure_point(scale),
                                                 stops,
                                                 &Matrix2D::identity());
        self.draw_target.fill_rect(&bounds.to_nearest_azure_rect(scale),
                                   PatternRef::LinearGradient(&pattern),
                                   None);
    }

    pub fn get_or_create_temporary_draw_target(&mut self,
                                               filters: &filter::T,
                                               blend_mode: mix_blend_mode::T)
                                               -> DrawTarget {
        // Determine if we need a temporary draw target.
        if !filters::temporary_draw_target_needed_for_style_filters(filters) &&
                blend_mode == mix_blend_mode::T::normal {
            // Reuse the draw target, but remove the transient clip. If we don't do the latter,
            // we'll be in a state whereby the paint subcontext thinks it has no transient clip
            // (see `StackingContext::optimize_and_draw_into_context`) but it actually does,
            // resulting in a situation whereby display items are seemingly randomly clipped out.
            self.remove_transient_clip_if_applicable();

            return self.draw_target.clone()
        }

        // FIXME(pcwalton): This surface might be bigger than necessary and waste memory.
        let size: AzIntSize = self.draw_target.get_size();
        let mut size = Size2D::new(size.width, size.height);

        // Pre-calculate if there is a blur expansion need.
        let accum_blur = filters::calculate_accumulated_blur(filters);
        let mut matrix = self.draw_target.get_transform();
        if accum_blur > Au(0) {
            // Set the correct size.
            let side_inflation = accum_blur * BLUR_INFLATION_FACTOR;
            size = Size2D::new(size.width + (side_inflation.to_nearest_px() * 2) as i32,
                               size.height + (side_inflation.to_nearest_px() * 2) as i32);

            // Calculate the transform matrix.
            let old_transform = self.draw_target.get_transform();
            let inflated_size = Rect::new(Point2D::new(0.0, 0.0),
                                          Size2D::new(size.width as AzFloat,
                                                      size.height as AzFloat));
            let temporary_draw_target_bounds = old_transform.transform_rect(&inflated_size);
            matrix = Matrix2D::identity().translate(
                -temporary_draw_target_bounds.origin.x as AzFloat,
                -temporary_draw_target_bounds.origin.y as AzFloat).mul(&old_transform);
        }

        let temporary_draw_target =
            self.draw_target.create_similar_draw_target(&size, self.draw_target.get_format());

        temporary_draw_target.set_transform(&matrix);
        temporary_draw_target
    }

    /// If we created a temporary draw target, then draw it to the main draw target. This is called
    /// after doing all the painting, and the temporary draw target must not be used afterward.
    pub fn draw_temporary_draw_target_if_necessary(&mut self,
                                                   temporary_draw_target: &DrawTarget,
                                                   filters: &filter::T,
                                                   blend_mode: mix_blend_mode::T) {
        if (*temporary_draw_target) == self.draw_target {
            // We're directly painting to the surface; nothing to do.
            return
        }

        // Set up transforms.
        let old_transform = self.draw_target.get_transform();
        self.draw_target.set_transform(&Matrix2D::identity());
        let rect = Rect::new(Point2D::new(0.0, 0.0), self.draw_target.get_size().to_azure_size());

        let rect_temporary = Rect::new(Point2D::new(0.0, 0.0),
                                       temporary_draw_target.get_size().to_azure_size());

        // Create the Azure filter pipeline.
        let mut accum_blur = Au(0);
        let (filter_node, opacity) = filters::create_filters(&self.draw_target,
                                                             temporary_draw_target,
                                                             filters,
                                                             &mut accum_blur);

        // Perform the blit operation.
        let mut draw_options = DrawOptions::new(opacity, CompositionOp::Over, AntialiasMode::None);
        draw_options.set_composition_op(blend_mode.to_azure_composition_op());

       // If there is a blur expansion, shift the transform and update the size.
        if accum_blur > Au(0) {
            // Remove both the transient clip and the stacking context clip, because we may need to
            // draw outside the stacking context's clip.
            self.remove_transient_clip_if_applicable();
            self.pop_clip_if_applicable();

            debug!("######### use expanded Rect.");
            self.draw_target.draw_filter(&filter_node,
                                         &rect_temporary,
                                         &rect_temporary.origin,
                                         draw_options);
            self.push_clip_if_applicable();
        } else {
            debug!("######### use regular Rect.");
            self.draw_target.draw_filter(&filter_node, &rect, &rect.origin, draw_options);
        }

        self.draw_target.set_transform(&old_transform);
    }

    /// Draws a box shadow with the given boundaries, color, offset, blur radius, and spread
    /// radius. `box_bounds` represents the boundaries of the box.
    pub fn draw_box_shadow(&mut self,
                           box_bounds: &Rect<Au>,
                           offset: &Point2D<Au>,
                           color: Color,
                           blur_radius: Au,
                           spread_radius: Au,
                           clip_mode: BoxShadowClipMode) {
        // Remove both the transient clip and the stacking context clip, because we may need to
        // draw outside the stacking context's clip.
        self.remove_transient_clip_if_applicable();
        self.pop_clip_if_applicable();

        // If we have blur, create a new draw target.
        let pixels_per_px = self.screen_pixels_per_px();
        let shadow_bounds = box_bounds.translate(offset).inflate(spread_radius, spread_radius);
        let side_inflation = blur_radius * BLUR_INFLATION_FACTOR;
        let inflated_shadow_bounds = shadow_bounds.inflate(side_inflation, side_inflation);
        let temporary_draw_target =
            self.create_draw_target_for_blur_if_necessary(&inflated_shadow_bounds, blur_radius);

        let path;
        match clip_mode {
            BoxShadowClipMode::Inset => {
                path = temporary_draw_target.draw_target
                                            .create_rectangular_border_path(&MAX_RECT,
                                                                            &shadow_bounds,
                                                                            pixels_per_px);
                self.draw_target.push_clip(
                    &self.draw_target.create_rectangular_path(box_bounds, pixels_per_px))
            }
            BoxShadowClipMode::Outset => {
                path = temporary_draw_target.draw_target.create_rectangular_path(&shadow_bounds,
                                                                                pixels_per_px);
                self.draw_target.push_clip(
                    &self.draw_target.create_rectangular_border_path(&MAX_RECT, box_bounds,
                                                                     pixels_per_px))
            }
            BoxShadowClipMode::None => {
                path = temporary_draw_target.draw_target.create_rectangular_path(&shadow_bounds,
                                                                                pixels_per_px)
            }
        }

        // Draw the shadow, and blur if we need to.
        temporary_draw_target.draw_target.fill(
            &path,
            Pattern::Color(ColorPattern::new(color)).to_pattern_ref(),
            &DrawOptions::new(1.0, CompositionOp::Over, AntialiasMode::None));
        self.blur_if_necessary(temporary_draw_target, blur_radius);

        // Undo the draw target's clip if we need to, and push back the stacking context clip.
        if clip_mode != BoxShadowClipMode::None {
            self.draw_target.pop_clip()
        }

        self.push_clip_if_applicable();
    }

    /// If we have blur, create a new draw target that's the same size as this tile, but with
    /// enough space around the edges to hold the entire blur. (If we don't do the latter, then
    /// there will be seams between tiles.)
    fn create_draw_target_for_blur_if_necessary(&self, box_bounds: &Rect<Au>, blur_radius: Au)
                                                -> TemporaryDrawTarget {
        if blur_radius == Au(0) {
            return TemporaryDrawTarget::from_main_draw_target(&self.draw_target)
        }

        // Intersect display item bounds with the tile bounds inflated by blur radius to get the
        // smallest possible rectangle that encompasses all the paint.
        let side_inflation = blur_radius * BLUR_INFLATION_FACTOR;
        let tile_box_bounds =
            geometry::f32_rect_to_au_rect(self.page_rect.to_untyped()).intersection(box_bounds)
                                                         .unwrap_or(Rect::zero())
                                                         .inflate(side_inflation, side_inflation);
        TemporaryDrawTarget::from_bounds(&self.draw_target, &tile_box_bounds)
    }

    /// Performs a blur using the draw target created in
    /// `create_draw_target_for_blur_if_necessary`.
    fn blur_if_necessary(&self, temporary_draw_target: TemporaryDrawTarget, blur_radius: Au) {
        if blur_radius == Au(0) {
            return
        }

        let blur_filter = self.draw_target.create_filter(FilterType::GaussianBlur);
        blur_filter.set_attribute(GaussianBlurAttribute::StdDeviation(blur_radius.to_f64_px() as
                                                                      AzFloat));
        blur_filter.set_input(GaussianBlurInput, &temporary_draw_target.draw_target.snapshot());
        temporary_draw_target.draw_filter(&self.draw_target, blur_filter);
    }

    pub fn push_clip_if_applicable(&self) {
        if let Some(ref clip_rect) = self.clip_rect {
            self.draw_push_clip(clip_rect)
        }
    }

    pub fn pop_clip_if_applicable(&self) {
        if self.clip_rect.is_some() {
            self.draw_pop_clip()
        }
    }

    pub fn remove_transient_clip_if_applicable(&mut self) {
        if let Some(old_transient_clip) = mem::replace(&mut self.transient_clip, None) {
            for _ in &old_transient_clip.complex {
                self.draw_pop_clip()
            }
            self.draw_pop_clip()
        }
    }

    /// Sets a new transient clipping region. Automatically calls
    /// `remove_transient_clip_if_applicable()` first.
    pub fn push_transient_clip(&mut self, clip_region: ClippingRegion) {
        let scale = self.screen_pixels_per_px();
        self.remove_transient_clip_if_applicable();

        self.draw_push_clip(&clip_region.main);
        for complex_region in &clip_region.complex {
            // FIXME(pcwalton): Actually draw a rounded rect.
            self.push_rounded_rect_clip(&complex_region.rect.to_nearest_azure_rect(scale),
                                        &complex_region.radii.to_radii_pixels(scale))
        }
        self.transient_clip = Some(clip_region)
    }
}

pub trait ToAzurePoint {
    fn to_nearest_azure_point(&self, pixels_per_px: ScaleFactor<PagePx, ScreenPx, f32>) -> Point2D<AzFloat>;
    fn to_azure_point(&self) -> Point2D<AzFloat>;
}

impl ToAzurePoint for Point2D<Au> {
    fn to_nearest_azure_point(&self, pixels_per_px: ScaleFactor<PagePx, ScreenPx, f32>) -> Point2D<AzFloat> {
        Point2D::new(self.x.to_nearest_pixel(pixels_per_px.get()) as AzFloat,
                     self.y.to_nearest_pixel(pixels_per_px.get()) as AzFloat)
    }
    fn to_azure_point(&self) -> Point2D<AzFloat> {
        Point2D::new(self.x.to_f32_px(), self.y.to_f32_px())
    }
}

pub trait ToAzureRect {
    fn to_nearest_azure_rect(&self, pixels_per_px: ScaleFactor<PagePx, ScreenPx, f32>) -> Rect<AzFloat>;
    fn to_nearest_non_empty_azure_rect(&self, pixels_per_px: ScaleFactor<PagePx, ScreenPx, f32>) -> Rect<AzFloat>;
    fn to_azure_rect(&self) -> Rect<AzFloat>;
}

impl ToAzureRect for Rect<Au> {
    /// Round rects to pixel coordinates, maintaining the invariant of non-overlap,
    /// assuming that before rounding rects don't overlap.
    fn to_nearest_azure_rect(&self, pixels_per_px: ScaleFactor<PagePx, ScreenPx, f32>) -> Rect<AzFloat> {
        // Rounding the top left corner to the nearest pixel with the size rounded
        // to the nearest pixel multiple would violate the non-overlap condition,
        // e.g.
        // 10px×9.60px at (0px,6.6px) & 10px×9.60px at (0px,16.2px)
        // would round to
        // 10px×10.0px at (0px,7.0px) & 10px×10.0px at (0px,16.0px), which overlap.
        //
        // Instead round each corner to the nearest pixel.
        let top_left = self.origin.to_nearest_azure_point(pixels_per_px);
        let bottom_right = self.bottom_right().to_nearest_azure_point(pixels_per_px);
        Rect::new(top_left, Size2D::new((bottom_right.x - top_left.x) as AzFloat,
                                        (bottom_right.y - top_left.y) as AzFloat))
    }

    /// For rects of width or height between 0.5px and 1px, rounding each rect corner to the
    /// nearest pixel can yield an empty rect e.g.
    /// 10px×0.6px at 0px,28.56px -> 10px×0px at 0px,29px
    /// Instead round the top left to the nearest pixel and the size to the nearest pixel
    /// multiple. It's possible for non-overlapping rects after this rounding to overlap.
    fn to_nearest_non_empty_azure_rect(&self, pixels_per_px: ScaleFactor<PagePx, ScreenPx, f32>) -> Rect<AzFloat> {
        Rect::new(self.origin.to_nearest_azure_point(pixels_per_px),
                  self.size.to_nearest_azure_size(pixels_per_px))
    }

    fn to_azure_rect(&self) -> Rect<AzFloat> {
        Rect::new(self.origin.to_azure_point(), self.size.to_azure_size())
    }
}

pub trait ToNearestAzureSize {
    fn to_nearest_azure_size(&self, pixels_per_px: ScaleFactor<PagePx, ScreenPx, f32>) -> Size2D<AzFloat>;
}

impl ToNearestAzureSize for Size2D<Au> {
    fn to_nearest_azure_size(&self, pixels_per_px: ScaleFactor<PagePx, ScreenPx, f32>) -> Size2D<AzFloat> {
        Size2D::new(self.width.to_nearest_pixel(pixels_per_px.get()) as AzFloat,
                    self.height.to_nearest_pixel(pixels_per_px.get()) as AzFloat)
    }
}

pub trait ToAzureSize {
    fn to_azure_size(&self) -> Size2D<AzFloat>;
}

impl ToAzureSize for Size2D<Au> {
    fn to_azure_size(&self) -> Size2D<AzFloat> {
        Size2D::new(self.width.to_f32_px(), self.height.to_f32_px())
    }
}

impl ToAzureSize for AzIntSize {
    fn to_azure_size(&self) -> Size2D<AzFloat> {
        Size2D::new(self.width as AzFloat, self.height as AzFloat)
    }
}

trait ToAzureIntSize {
    fn to_azure_int_size(&self) -> Size2D<i32>;
}

impl ToAzureIntSize for Size2D<AzFloat> {
    fn to_azure_int_size(&self) -> Size2D<i32> {
        Size2D::new(self.width as i32, self.height as i32)
    }
}

trait ToSideOffsetsPixels {
    fn to_float_pixels(&self, pixels_per_px: ScaleFactor<PagePx, ScreenPx, f32>) -> SideOffsets2D<AzFloat>;
}

impl ToSideOffsetsPixels for SideOffsets2D<Au> {
    fn to_float_pixels(&self, pixels_per_px: ScaleFactor<PagePx, ScreenPx, f32>) -> SideOffsets2D<AzFloat> {
        SideOffsets2D::new(self.top.to_nearest_pixel(pixels_per_px.get()) as AzFloat,
                           self.right.to_nearest_pixel(pixels_per_px.get()) as AzFloat,
                           self.bottom.to_nearest_pixel(pixels_per_px.get()) as AzFloat,
                           self.left.to_nearest_pixel(pixels_per_px.get()) as AzFloat)
    }
}

trait ToRadiiPixels {
    fn to_radii_pixels(&self, pixels_per_px: ScaleFactor<PagePx, ScreenPx, f32>) -> BorderRadii<AzFloat>;
}

impl ToRadiiPixels for BorderRadii<Au> {
    fn to_radii_pixels(&self, pixels_per_px: ScaleFactor<PagePx, ScreenPx, f32>) -> BorderRadii<AzFloat> {
        let to_nearest_px = |x: Au| -> AzFloat {
            x.to_nearest_pixel(pixels_per_px.get()) as AzFloat
        };

        BorderRadii {
            top_left: Size2D { width: to_nearest_px(self.top_left.width),
                               height: to_nearest_px(self.top_left.height) },
            top_right: Size2D { width: to_nearest_px(self.top_right.width),
                                height: to_nearest_px(self.top_right.height) },
            bottom_left: Size2D { width: to_nearest_px(self.bottom_left.width),
                                  height: to_nearest_px(self.bottom_left.height) },
            bottom_right: Size2D { width: to_nearest_px(self.bottom_right.width),
                                   height: to_nearest_px(self.bottom_right.height) },
       }
    }
}

trait ScaledFontExtensionMethods {
    fn draw_text(&self,
                 draw_target: &DrawTarget,
                 run: &TextRun,
                 range: &Range<ByteIndex>,
                 baseline_origin: Point2D<Au>,
                 color: Color,
                 antialias: bool);
}

impl ScaledFontExtensionMethods for ScaledFont {
    #[allow(unsafe_code)]
    fn draw_text(&self,
                 draw_target: &DrawTarget,
                 run: &TextRun,
                 range: &Range<ByteIndex>,
                 baseline_origin: Point2D<Au>,
                 color: Color,
                 antialias: bool) {
        let pattern = ColorPattern::new(color);
        let azure_pattern = pattern.azure_color_pattern;
        assert!(!azure_pattern.is_null());

        let mut options = struct__AzDrawOptions {
            mAlpha: 1f64 as AzFloat,
            mCompositionOp: CompositionOp::Over as u8,
            mAntialiasMode: if antialias {
                                AntialiasMode::Subpixel as u8
                            } else {
                                AntialiasMode::None as u8
                            }
        };

        let mut origin = baseline_origin.clone();
        let mut azglyphs = Vec::with_capacity(range.length().to_usize());

        for slice in run.natural_word_slices_in_visual_order(range) {
            for glyph in slice.glyphs.iter_glyphs_for_byte_range(&slice.range) {
                let glyph_advance = if glyph.char_is_space() {
                    glyph.advance() + run.extra_word_spacing
                } else {
                    glyph.advance()
                };
                if !slice.glyphs.is_whitespace() {
                    let glyph_offset = glyph.offset().unwrap_or(Point2D::zero());
                    let azglyph = struct__AzGlyph {
                        mIndex: glyph.id() as u32,
                        mPosition: struct__AzPoint {
                            x: (origin.x + glyph_offset.x).to_f32_px(),
                            y: (origin.y + glyph_offset.y).to_f32_px(),
                        }
                    };
                    azglyphs.push(azglyph)
                }
                origin.x = origin.x + glyph_advance;
            };
        }

        let azglyph_buf_len = azglyphs.len();
        if azglyph_buf_len == 0 { return; } // Otherwise the Quartz backend will assert.

        let mut glyphbuf = struct__AzGlyphBuffer {
            mGlyphs: azglyphs.as_mut_ptr(),
            mNumGlyphs: azglyph_buf_len as u32
        };

        unsafe {
            // TODO(Issue #64): this call needs to move into azure_hl.rs
            AzDrawTargetFillGlyphs(draw_target.azure_draw_target,
                                   self.get_ref(),
                                   &mut glyphbuf,
                                   azure_pattern,
                                   &mut options,
                                   ptr::null_mut());
        }
    }
}

trait DrawTargetExtensions {
    /// Creates and returns a path that represents a rectangular border. Like this:
    ///
    /// ```text
    ///     +--------------------------------+
    ///     |################################|
    ///     |#######+---------------------+##|
    ///     |#######|                     |##|
    ///     |#######+---------------------+##|
    ///     |################################|
    ///     +--------------------------------+
    /// ```
    fn create_rectangular_border_path(&self,
                                      outer_rect: &Rect<Au>,
                                      inner_rect: &Rect<Au>,
                                      pixels_per_px: ScaleFactor<PagePx, ScreenPx, f32>) -> Path;

    /// Creates and returns a path that represents a rectangle.
    fn create_rectangular_path(&self, rect: &Rect<Au>, pixels_per_px: ScaleFactor<PagePx, ScreenPx, f32>) -> Path;
}

impl DrawTargetExtensions for DrawTarget {
    fn create_rectangular_border_path(&self,
                                      outer_rect: &Rect<Au>,
                                      inner_rect: &Rect<Au>,
                                      pixels_per_px: ScaleFactor<PagePx, ScreenPx, f32>) -> Path {
        // +-----------+
        // |2          |1
        // |           |
        // |   +---+---+
        // |   |9  |6  |5, 10
        // |   |   |   |
        // |   +---+   |
        // |    8   7  |
        // |           |
        // +-----------+
        //  3           4

        let outer_rect = outer_rect.to_nearest_azure_rect(pixels_per_px);
        let inner_rect = inner_rect.to_nearest_azure_rect(pixels_per_px);
        let path_builder = self.create_path_builder();
        path_builder.move_to(Point2D::new(outer_rect.max_x(), outer_rect.origin.y));     // 1
        path_builder.line_to(Point2D::new(outer_rect.origin.x, outer_rect.origin.y));    // 2
        path_builder.line_to(Point2D::new(outer_rect.origin.x, outer_rect.max_y()));     // 3
        path_builder.line_to(Point2D::new(outer_rect.max_x(), outer_rect.max_y()));      // 4
        path_builder.line_to(Point2D::new(outer_rect.max_x(), inner_rect.origin.y));     // 5
        path_builder.line_to(Point2D::new(inner_rect.max_x(), inner_rect.origin.y));     // 6
        path_builder.line_to(Point2D::new(inner_rect.max_x(), inner_rect.max_y()));      // 7
        path_builder.line_to(Point2D::new(inner_rect.origin.x, inner_rect.max_y()));     // 8
        path_builder.line_to(inner_rect.origin);                                         // 9
        path_builder.line_to(Point2D::new(outer_rect.max_x(), inner_rect.origin.y));     // 10
        path_builder.finish()
    }

    fn create_rectangular_path(&self, rect: &Rect<Au>, pixels_per_px: ScaleFactor<PagePx, ScreenPx, f32>) -> Path {
        // Explicitly round to the nearest non-empty rect because when drawing
        // box-shadow the rect height can be between 0.5px & 1px and could
        // otherwise round to an empty rect.
        let rect = rect.to_nearest_non_empty_azure_rect(pixels_per_px);

        let path_builder = self.create_path_builder();
        path_builder.move_to(rect.origin);
        path_builder.line_to(Point2D::new(rect.max_x(), rect.origin.y));
        path_builder.line_to(Point2D::new(rect.max_x(), rect.max_y()));
        path_builder.line_to(Point2D::new(rect.origin.x, rect.max_y()));
        path_builder.finish()
    }
}

/// Converts a CSS blend mode (per CSS-COMPOSITING) to an Azure `CompositionOp`.
trait ToAzureCompositionOp {
    /// Converts a CSS blend mode (per CSS-COMPOSITING) to an Azure `CompositionOp`.
    fn to_azure_composition_op(&self) -> CompositionOp;
}

impl ToAzureCompositionOp for mix_blend_mode::T {
    fn to_azure_composition_op(&self) -> CompositionOp {
        match *self {
            mix_blend_mode::T::normal => CompositionOp::Over,
            mix_blend_mode::T::multiply => CompositionOp::Multiply,
            mix_blend_mode::T::screen => CompositionOp::Screen,
            mix_blend_mode::T::overlay => CompositionOp::Overlay,
            mix_blend_mode::T::darken => CompositionOp::Darken,
            mix_blend_mode::T::lighten => CompositionOp::Lighten,
            mix_blend_mode::T::color_dodge => CompositionOp::ColorDodge,
            mix_blend_mode::T::color_burn => CompositionOp::ColorBurn,
            mix_blend_mode::T::hard_light => CompositionOp::HardLight,
            mix_blend_mode::T::soft_light => CompositionOp::SoftLight,
            mix_blend_mode::T::difference => CompositionOp::Difference,
            mix_blend_mode::T::exclusion => CompositionOp::Exclusion,
            mix_blend_mode::T::hue => CompositionOp::Hue,
            mix_blend_mode::T::saturation => CompositionOp::Saturation,
            mix_blend_mode::T::color => CompositionOp::Color,
            mix_blend_mode::T::luminosity => CompositionOp::Luminosity,
        }
    }
}

/// Represents a temporary drawing surface. Some operations that perform complex compositing
/// operations need this.
struct TemporaryDrawTarget {
    /// The draw target.
    draw_target: DrawTarget,
    /// The distance from the top left of the main draw target to the top left of this temporary
    /// draw target.
    offset: Point2D<AzFloat>,
}

impl TemporaryDrawTarget {
    /// Creates a temporary draw target that simply draws to the main draw target.
    fn from_main_draw_target(main_draw_target: &DrawTarget) -> TemporaryDrawTarget {
        TemporaryDrawTarget {
            draw_target: main_draw_target.clone(),
            offset: Point2D::new(0.0, 0.0),
        }
    }

    /// Creates a temporary draw target large enough to encompass the given bounding rect in page
    /// coordinates. The temporary draw target will have the same transform as the tile we're
    /// drawing to.
    fn from_bounds(main_draw_target: &DrawTarget, bounds: &Rect<Au>) -> TemporaryDrawTarget {
        let draw_target_transform = main_draw_target.get_transform();
        let temporary_draw_target_bounds =
            draw_target_transform.transform_rect(&bounds.to_azure_rect());
        let temporary_draw_target_size =
            Size2D::new(temporary_draw_target_bounds.size.width.ceil() as i32,
                        temporary_draw_target_bounds.size.height.ceil() as i32);
        let temporary_draw_target =
            main_draw_target.create_similar_draw_target(&temporary_draw_target_size,
                                                        main_draw_target.get_format());
        let matrix =
            Matrix2D::identity().translate(-temporary_draw_target_bounds.origin.x as AzFloat,
                                           -temporary_draw_target_bounds.origin.y as AzFloat)
                                .mul(&draw_target_transform);
        temporary_draw_target.set_transform(&matrix);
        TemporaryDrawTarget {
            draw_target: temporary_draw_target,
            offset: temporary_draw_target_bounds.origin,
        }
    }

    /// Composites this temporary draw target onto the main surface, with the given Azure filter.
    fn draw_filter(self, main_draw_target: &DrawTarget, filter: FilterNode) {
        let main_draw_target_transform = main_draw_target.get_transform();
        let temporary_draw_target_size = self.draw_target.get_size();
        let temporary_draw_target_size = Size2D::new(temporary_draw_target_size.width as AzFloat,
                                                     temporary_draw_target_size.height as AzFloat);

        // Blit the blur onto the tile. We undo the transforms here because we want to directly
        // stack the temporary draw target onto the tile.
        main_draw_target.set_transform(&Matrix2D::identity());
        main_draw_target.draw_filter(&filter,
                                     &Rect::new(Point2D::new(0.0, 0.0), temporary_draw_target_size),
                                     &self.offset,
                                     DrawOptions::new(1.0, CompositionOp::Over, AntialiasMode::None));
        main_draw_target.set_transform(&main_draw_target_transform);

    }
}

#[derive(Copy, Clone, PartialEq)]
enum BorderPathDrawingMode {
    EntireBorder,
    CornersOnly,
}

fn radii_apply_to_border_direction(direction: Direction, radius: &BorderRadii<AzFloat>) -> bool {
    match (direction,
           radius.top_left.width,
           radius.top_right.width,
           radius.bottom_left.width,
           radius.bottom_right.width) {
        (Direction::Top, a, b, _, _) |
        (Direction::Right, _, a, _, b) |
        (Direction::Bottom, _, _, a, b) |
        (Direction::Left, a, _, b, _) => a != 0.0 || b != 0.0,
    }
}
