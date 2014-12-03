/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Painting of display lists using Moz2D/Azure.

use azure::azure::AzIntSize;
use azure::azure_hl::{B8G8R8A8, A8, Color, ColorPattern, ColorPatternRef, DrawOptions};
use azure::azure_hl::{DrawSurfaceOptions, DrawTarget, ExtendClamp, GradientStop, Linear};
use azure::azure_hl::{LinearGradientPattern, LinearGradientPatternRef, SourceOp, StrokeOptions};
use azure::scaled_font::ScaledFont;
use azure::{AZ_CAP_BUTT, AzFloat, struct__AzDrawOptions, struct__AzGlyph};
use azure::{struct__AzGlyphBuffer, struct__AzPoint, AzDrawTargetFillGlyphs};
use display_list::{SidewaysLeft, SidewaysRight, TextDisplayItem, Upright, BorderRadii};
use font_context::FontContext;
use geom::matrix2d::Matrix2D;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::side_offsets::SideOffsets2D;
use geom::size::Size2D;
use libc::size_t;
use libc::types::common::c99::{uint16_t, uint32_t};
use png::{RGB8, RGBA8, K8, KA8};
use servo_net::image::base::Image;
use servo_util::geometry::Au;
use servo_util::opts;
use servo_util::range::Range;
use std::default::Default;
use std::num::{Float, FloatMath, Zero};
use std::ptr;
use style::computed_values::border_style;
use sync::Arc;
use text::TextRun;
use text::glyph::CharIndex;

pub struct RenderContext<'a> {
    pub draw_target: DrawTarget,
    pub font_ctx: &'a mut Box<FontContext>,
    /// The rectangle that this context encompasses in page coordinates.
    pub page_rect: Rect<f32>,
    /// The rectangle that this context encompasses in screen coordinates (pixels).
    pub screen_rect: Rect<uint>,
}

enum Direction {
    Top,
    Left,
    Right,
    Bottom
}

enum DashSize {
    DottedBorder = 1,
    DashedBorder = 3
}

impl<'a> RenderContext<'a>  {
    pub fn get_draw_target(&self) -> &DrawTarget {
        &self.draw_target
    }

    pub fn draw_solid_color(&self, bounds: &Rect<Au>, color: Color) {
        self.draw_target.make_current();
        self.draw_target.fill_rect(&bounds.to_azure_rect(),
                                   ColorPatternRef(&ColorPattern::new(color)),
                                   None);
    }

    pub fn draw_border(&self,
                       bounds: &Rect<Au>,
                       border: SideOffsets2D<Au>,
                       radius: &BorderRadii<Au>,
                       color: SideOffsets2D<Color>,
                       style: SideOffsets2D<border_style::T>) {
        let border = border.to_float_px();
        let radius = radius.to_radii_px();

        self.draw_target.make_current();

        self.draw_border_segment(Top, bounds, border, &radius, color, style);
        self.draw_border_segment(Right, bounds, border, &radius, color, style);
        self.draw_border_segment(Bottom, bounds, border, &radius, color, style);
        self.draw_border_segment(Left, bounds, border, &radius, color, style);
    }

    pub fn draw_line(&self,
                     bounds: &Rect<Au>,
                     color: Color,
                     style: border_style::T) {
        self.draw_target.make_current();

        self.draw_line_segment(bounds, &Default::default(), color, style);
    }

    pub fn draw_push_clip(&self, bounds: &Rect<Au>) {
        let rect = bounds.to_azure_rect();
        let path_builder = self.draw_target.create_path_builder();

        let left_top = Point2D(rect.origin.x, rect.origin.y);
        let right_top = Point2D(rect.origin.x + rect.size.width, rect.origin.y);
        let left_bottom = Point2D(rect.origin.x, rect.origin.y + rect.size.height);
        let right_bottom = Point2D(rect.origin.x + rect.size.width, rect.origin.y + rect.size.height);

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

    pub fn draw_image(&self, bounds: Rect<Au>, image: Arc<Box<Image>>) {
        let size = Size2D(image.width as i32, image.height as i32);
        let (pixel_width, pixels, source_format) = match image.pixels {
            RGBA8(ref pixels) => (4, pixels.as_slice(), B8G8R8A8),
            K8(ref pixels) => (1, pixels.as_slice(), A8),
            RGB8(_) => panic!("RGB8 color type not supported"),
            KA8(_) => panic!("KA8 color type not supported"),
        };
        let stride = image.width * pixel_width;

        self.draw_target.make_current();
        let draw_target_ref = &self.draw_target;
        let azure_surface = draw_target_ref.create_source_surface_from_data(pixels,
                                                                            size,
                                                                            stride as i32,
                                                                            source_format);
        let source_rect = Rect(Point2D(0u as AzFloat, 0u as AzFloat),
                               Size2D(image.width as AzFloat, image.height as AzFloat));
        let dest_rect = bounds.to_azure_rect();
        let draw_surface_options = DrawSurfaceOptions::new(Linear, true);
        let draw_options = DrawOptions::new(1.0f64 as AzFloat, 0);
        draw_target_ref.draw_surface(azure_surface,
                                     dest_rect,
                                     source_rect,
                                     draw_surface_options,
                                     draw_options);
    }

    pub fn clear(&self) {
        let pattern = ColorPattern::new(Color::new(0.0, 0.0, 0.0, 0.0));
        let rect = Rect(Point2D(self.page_rect.origin.x as AzFloat,
                                self.page_rect.origin.y as AzFloat),
                        Size2D(self.screen_rect.size.width as AzFloat,
                               self.screen_rect.size.height as AzFloat));
        let mut draw_options = DrawOptions::new(1.0, 0);
        draw_options.set_composition_op(SourceOp);
        self.draw_target.make_current();
        self.draw_target.fill_rect(&rect, ColorPatternRef(&pattern), Some(&draw_options));
    }

    fn draw_border_segment(&self,
                           direction: Direction,
                           bounds: &Rect<Au>,
                           border: SideOffsets2D<f32>,
                           radius: &BorderRadii<AzFloat>,
                           color: SideOffsets2D<Color>,
                           style: SideOffsets2D<border_style::T>) {
        let (style_select, color_select) = match direction {
            Top => (style.top, color.top),
            Left => (style.left, color.left),
            Right => (style.right, color.right),
            Bottom => (style.bottom, color.bottom)
        };

        match style_select{
            border_style::none                         => {
            }
            border_style::hidden                       => {
            }
            //FIXME(sammykim): This doesn't work with dash_pattern and cap_style well. I referred firefox code.
            border_style::dotted                       => {
                self.draw_dashed_border_segment(direction, bounds, border, color_select, DottedBorder);
            }
            border_style::dashed                       => {
                self.draw_dashed_border_segment(direction, bounds, border, color_select, DashedBorder);
            }
            border_style::solid                        => {
                self.draw_solid_border_segment(direction,bounds,border,radius,color_select);
            }
            border_style::double                       => {
                self.draw_double_border_segment(direction, bounds, border, radius, color_select);
            }
            border_style::groove | border_style::ridge => {
                self.draw_groove_ridge_border_segment(direction, bounds, border, radius, color_select, style_select);
            }
            border_style::inset | border_style::outset => {
                self.draw_inset_outset_border_segment(direction, bounds, border, radius, color_select, style_select);
            }
        }
    }

    fn draw_line_segment(&self, bounds: &Rect<Au>, radius: &BorderRadii<AzFloat>, color: Color, style: border_style::T) {
        let border = SideOffsets2D::new_all_same(bounds.size.width).to_float_px();

        match style {
            border_style::none | border_style::hidden  => {}
            border_style::dotted                       => {
                self.draw_dashed_border_segment(Right, bounds, border, color, DottedBorder);
            }
            border_style::dashed                       => {
                self.draw_dashed_border_segment(Right, bounds, border, color, DashedBorder);
            }
            border_style::solid                        => {
                self.draw_solid_border_segment(Right, bounds, border, radius, color);
            }
            border_style::double                       => {
                self.draw_double_border_segment(Right, bounds, border, radius, color);
            }
            border_style::groove | border_style::ridge => {
                self.draw_groove_ridge_border_segment(Right, bounds, border, radius, color, style);
            }
            border_style::inset | border_style::outset => {
                self.draw_inset_outset_border_segment(Right, bounds, border, radius, color, style);
            }
        }
    }

    // The following comment is wonderful, and stolen from
    // gecko:gfx/thebes/gfxContext.cpp:RoundedRectangle for reference.
    //
    // It does not currently apply to the code, but will be extremely useful in
    // the future when the below TODO is addressed.
    //
    // TODO(cgaebel): Switch from arcs to beziers for drawing the corners.
    //                Then, add http://www.subcide.com/experiments/fail-whale/
    //                to the reftest suite.
    //
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

    #[allow(non_snake_case)]
    fn draw_border_path(&self,
                        bounds:    &Rect<f32>,
                        direction: Direction,
                        border:    SideOffsets2D<f32>,
                        radius:    &BorderRadii<AzFloat>,
                        color:     Color) {
        // T = top, B = bottom, L = left, R = right

        let box_TL = bounds.origin;
        let box_TR = box_TL + Point2D(bounds.size.width, 0.0);
        let box_BL = box_TL + Point2D(0.0, bounds.size.height);
        let box_BR = box_TL + Point2D(bounds.size.width, bounds.size.height);

        let draw_opts    = DrawOptions::new(1.0, 0);
        let path_builder = self.draw_target.create_path_builder();

        let rad_R: AzFloat = 0.;
        let rad_BR = rad_R  + Float::frac_pi_4();
        let rad_B  = rad_BR + Float::frac_pi_4();
        let rad_BL = rad_B  + Float::frac_pi_4();
        let rad_L  = rad_BL + Float::frac_pi_4();
        let rad_TL = rad_L  + Float::frac_pi_4();
        let rad_T  = rad_TL + Float::frac_pi_4();
        let rad_TR = rad_T  + Float::frac_pi_4();

        match direction {
            Top    => {
                let edge_TL = box_TL  + Point2D(radius.top_left.max(border.left), 0.);
                let edge_TR = box_TR  + Point2D(-radius.top_right.max(border.right), 0.);
                let edge_BR = edge_TR + Point2D(0., border.top);
                let edge_BL = edge_TL + Point2D(0., border.top);

                path_builder.move_to(edge_TL);
                path_builder.line_to(edge_TR);

                // the origin is the center of the arcs we're about to draw.
                let origin = edge_TR + Point2D((border.right - radius.top_right).max(0.), radius.top_right);
                // the elbow is the inside of the border's curve.
                let distance_to_elbow = (radius.top_right - border.top).max(0.);

                path_builder.arc(origin, radius.top_right,  rad_T, rad_TR, false);
                path_builder.arc(origin, distance_to_elbow, rad_TR, rad_T, true);

                path_builder.line_to(edge_BR);
                path_builder.line_to(edge_BL);

                let origin = edge_TL + Point2D(-(border.left - radius.top_left).max(0.), radius.top_left);
                let distance_to_elbow = (radius.top_left - border.top).max(0.);

                path_builder.arc(origin, distance_to_elbow, rad_T, rad_TL, true);
                path_builder.arc(origin, radius.top_left,   rad_TL, rad_T, false);
            }
            Left   => {
                let edge_TL = box_TL  + Point2D(0., radius.top_left.max(border.top));
                let edge_BL = box_BL  + Point2D(0., -radius.bottom_left.max(border.bottom));
                let edge_TR = edge_TL + Point2D(border.left, 0.);
                let edge_BR = edge_BL + Point2D(border.left, 0.);

                path_builder.move_to(edge_BL);
                path_builder.line_to(edge_TL);

                let origin = edge_TL + Point2D(radius.top_left, -(border.top - radius.top_left).max(0.));
                let distance_to_elbow = (radius.top_left - border.left).max(0.);

                path_builder.arc(origin, radius.top_left,   rad_L, rad_TL, false);
                path_builder.arc(origin, distance_to_elbow, rad_TL, rad_L, true);

                path_builder.line_to(edge_TR);
                path_builder.line_to(edge_BR);

                let origin = edge_BL + Point2D(radius.bottom_left, (border.bottom - radius.bottom_left).max(0.));
                let distance_to_elbow = (radius.bottom_left - border.left).max(0.);

                path_builder.arc(origin, distance_to_elbow,  rad_L, rad_BL, true);
                path_builder.arc(origin, radius.bottom_left, rad_BL, rad_L, false);
            }
            Right  => {
                let edge_TR = box_TR  + Point2D(0., radius.top_right.max(border.top));
                let edge_BR = box_BR  + Point2D(0., -radius.bottom_right.max(border.bottom));
                let edge_TL = edge_TR + Point2D(-border.right, 0.);
                let edge_BL = edge_BR + Point2D(-border.right, 0.);

                path_builder.move_to(edge_BL);
                path_builder.line_to(edge_TL);

                let origin = edge_TR + Point2D(-radius.top_right, -(border.top - radius.top_right).max(0.));
                let distance_to_elbow = (radius.top_right - border.right).max(0.);

                path_builder.arc(origin, distance_to_elbow, rad_R, rad_TR, true);
                path_builder.arc(origin, radius.top_right,  rad_TR, rad_R, false);

                path_builder.line_to(edge_TR);
                path_builder.line_to(edge_BR);

                let origin = edge_BR + Point2D(-radius.bottom_right, (border.bottom - radius.bottom_right).max(0.));
                let distance_to_elbow = (radius.bottom_right - border.right).max(0.);

                path_builder.arc(origin, radius.bottom_right, rad_R, rad_BR, false);
                path_builder.arc(origin, distance_to_elbow,   rad_BR, rad_R, true);
            }
            Bottom => {
                let edge_BL = box_BL  + Point2D(radius.bottom_left.max(border.left), 0.);
                let edge_BR = box_BR  + Point2D(-radius.bottom_right.max(border.right), 0.);
                let edge_TL = edge_BL + Point2D(0., -border.bottom);
                let edge_TR = edge_BR + Point2D(0., -border.bottom);

                path_builder.move_to(edge_TL);
                path_builder.line_to(edge_TR);

                let origin = edge_BR + Point2D((border.right - radius.bottom_right).max(0.), -radius.bottom_right);
                let distance_to_elbow = (radius.bottom_right - border.bottom).max(0.);

                path_builder.arc(origin, distance_to_elbow,   rad_B, rad_BR, true);
                path_builder.arc(origin, radius.bottom_right, rad_BR, rad_B, false);

                path_builder.line_to(edge_BR);
                path_builder.line_to(edge_BL);

                let origin = edge_BL - Point2D((border.left - radius.bottom_left).max(0.), radius.bottom_left);
                let distance_to_elbow = (radius.bottom_left - border.bottom).max(0.);

                path_builder.arc(origin, radius.bottom_left, rad_B, rad_BL, false);
                path_builder.arc(origin, distance_to_elbow,  rad_BL, rad_B, true);
            }
        }

        let path = path_builder.finish();
        self.draw_target.fill(&path, &ColorPattern::new(color), &draw_opts);
    }

    fn draw_dashed_border_segment(&self,
                                  direction: Direction,
                                  bounds:    &Rect<Au>,
                                  border:    SideOffsets2D<f32>,
                                  color:     Color,
                                  dash_size: DashSize) {
        let rect = bounds.to_azure_rect();
        let draw_opts = DrawOptions::new(1u as AzFloat, 0 as uint16_t);
        let mut stroke_opts = StrokeOptions::new(0u as AzFloat, 10u as AzFloat);
        let mut dash: [AzFloat, ..2] = [0u as AzFloat, 0u as AzFloat];

        stroke_opts.set_cap_style(AZ_CAP_BUTT as u8);

        let border_width = match direction {
            Top => border.top,
            Left => border.left,
            Right => border.right,
            Bottom => border.bottom
        };

        stroke_opts.line_width = border_width;
        dash[0] = border_width * (dash_size as int) as AzFloat;
        dash[1] = border_width * (dash_size as int) as AzFloat;
        stroke_opts.mDashPattern = dash.as_mut_ptr();
        stroke_opts.mDashLength = dash.len() as size_t;

        let (start, end)  = match direction {
            Top => {
                let y = rect.origin.y + border.top * 0.5;
                let start = Point2D(rect.origin.x, y);
                let end = Point2D(rect.origin.x + rect.size.width, y);
                (start, end)
            }
            Left => {
                let x = rect.origin.x + border.left * 0.5;
                let start = Point2D(x, rect.origin.y + rect.size.height);
                let end = Point2D(x, rect.origin.y + border.top);
                (start, end)
            }
            Right => {
                let x = rect.origin.x + rect.size.width - border.right * 0.5;
                let start = Point2D(x, rect.origin.y);
                let end = Point2D(x, rect.origin.y + rect.size.height);
                (start, end)
            }
            Bottom => {
                let y = rect.origin.y + rect.size.height - border.bottom * 0.5;
                let start = Point2D(rect.origin.x + rect.size.width, y);
                let end = Point2D(rect.origin.x + border.left, y);
                (start, end)
            }
        };

        self.draw_target.stroke_line(start,
                                     end,
                                     &ColorPattern::new(color),
                                     &stroke_opts,
                                     &draw_opts);
    }

    fn draw_solid_border_segment(&self, direction: Direction, bounds: &Rect<Au>, border: SideOffsets2D<f32>, radius: &BorderRadii<AzFloat>, color: Color) {
        let rect = bounds.to_azure_rect();
        self.draw_border_path(&rect, direction, border, radius, color);
    }

    fn get_scaled_bounds(&self,
                         bounds:        &Rect<Au>,
                         border:        SideOffsets2D<f32>,
                         shrink_factor: f32) -> Rect<f32> {
        let rect            = bounds.to_azure_rect();
        let scaled_border   = SideOffsets2D::new(shrink_factor * border.top,
                                                 shrink_factor * border.right,
                                                 shrink_factor * border.bottom,
                                                 shrink_factor * border.left);
        let left_top        = Point2D(rect.origin.x, rect.origin.y);
        let scaled_left_top = left_top + Point2D(scaled_border.left,
                                                 scaled_border.top);
        return Rect(scaled_left_top,
                    Size2D(rect.size.width - 2.0 * scaled_border.right, rect.size.height - 2.0 * scaled_border.bottom));
    }

    fn scale_color(&self, color: Color, scale_factor: f32) -> Color {
        return Color::new(color.r * scale_factor, color.g * scale_factor, color.b * scale_factor, color.a);
    }

    fn draw_double_border_segment(&self, direction: Direction, bounds: &Rect<Au>, border: SideOffsets2D<f32>, radius: &BorderRadii<AzFloat>, color: Color) {
        let scaled_border       = SideOffsets2D::new((1.0/3.0) * border.top,
                                                     (1.0/3.0) * border.right,
                                                     (1.0/3.0) * border.bottom,
                                                     (1.0/3.0) * border.left);
        let inner_scaled_bounds = self.get_scaled_bounds(bounds, border, 2.0/3.0);
        // draw the outer portion of the double border.
        self.draw_solid_border_segment(direction, bounds, scaled_border, radius, color);
        // draw the inner portion of the double border.
        self.draw_border_path(&inner_scaled_bounds, direction, scaled_border, radius, color);
    }

    fn draw_groove_ridge_border_segment(&self,
                                        direction: Direction,
                                        bounds:    &Rect<Au>,
                                        border:    SideOffsets2D<f32>,
                                        radius:    &BorderRadii<AzFloat>,
                                        color:     Color,
                                        style:     border_style::T) {
        // original bounds as a Rect<f32>, with no scaling.
        let original_bounds            = self.get_scaled_bounds(bounds, border, 0.0);
        // shrink the bounds by 1/2 of the border, leaving the innermost 1/2 of the border
        let inner_scaled_bounds        = self.get_scaled_bounds(bounds, border, 0.5);
        let scaled_border              = SideOffsets2D::new(0.5 * border.top,
                                                            0.5 * border.right,
                                                            0.5 * border.bottom,
                                                            0.5 * border.left);
        let is_groove = match style {
                border_style::groove =>  true,
                border_style::ridge  =>  false,
                _                    =>  panic!("invalid border style")
        };
        let darker_color               = self.scale_color(color, if is_groove { 1.0/3.0 } else { 2.0/3.0 });
        let (outer_color, inner_color) = match (direction, is_groove) {
            (Top, true)  | (Left, true)  | (Right, false) | (Bottom, false) => (darker_color, color),
            (Top, false) | (Left, false) | (Right, true)  | (Bottom, true)  => (color, darker_color)
        };
        // outer portion of the border
        self.draw_border_path(&original_bounds, direction, scaled_border, radius, outer_color);
        // inner portion of the border
        self.draw_border_path(&inner_scaled_bounds, direction, scaled_border, radius, inner_color);
    }

    fn draw_inset_outset_border_segment(&self,
                                        direction: Direction,
                                        bounds:    &Rect<Au>,
                                        border:    SideOffsets2D<f32>,
                                        radius:    &BorderRadii<AzFloat>,
                                        color:     Color,
                                        style:     border_style::T) {
        let is_inset = match style {
                border_style::inset  =>  true,
                border_style::outset =>  false,
                _                    =>  panic!("invalid border style")
        };
        // original bounds as a Rect<f32>
        let original_bounds = self.get_scaled_bounds(bounds, border, 0.0);
        // select and scale the color appropriately.
        let scaled_color    = match direction {
            Top             => self.scale_color(color, if is_inset { 2.0/3.0 } else { 1.0     }),
            Left            => self.scale_color(color, if is_inset { 1.0/6.0 } else { 0.5     }),
            Right | Bottom  => self.scale_color(color, if is_inset { 1.0     } else { 2.0/3.0 })
        };
        self.draw_border_path(&original_bounds, direction, border, radius, scaled_color);
    }

    pub fn draw_text(&mut self,
                     text: &TextDisplayItem,
                     current_transform: &Matrix2D<AzFloat>) {
        // Optimization: Don’t set a transform matrix for upright text, and pass a start point to
        // `draw_text_into_context`.
        //
        // For sideways text, it’s easier to do the rotation such that its center (the baseline’s
        // start point) is at (0, 0) coordinates.
        let baseline_origin = match text.orientation {
            Upright => text.baseline_origin,
            SidewaysLeft => {
                let x = text.baseline_origin.x.to_subpx() as AzFloat;
                let y = text.baseline_origin.y.to_subpx() as AzFloat;
                self.draw_target.set_transform(&current_transform.mul(&Matrix2D::new(0., -1.,
                                                                                     1., 0.,
                                                                                     x, y)));
                Zero::zero()
            }
            SidewaysRight => {
                let x = text.baseline_origin.x.to_subpx() as AzFloat;
                let y = text.baseline_origin.y.to_subpx() as AzFloat;
                self.draw_target.set_transform(&current_transform.mul(&Matrix2D::new(0., 1.,
                                                                                     -1., 0.,
                                                                                     x, y)));
                Zero::zero()
            }
        };

        self.font_ctx
            .get_render_font_from_template(&text.text_run.font_template,
                                           text.text_run.actual_pt_size)
            .borrow()
            .draw_text_into_context(self,
                                    &*text.text_run,
                                    &text.range,
                                    baseline_origin,
                                    text.text_color,
                                    opts::get().enable_text_antialiasing);

        // Undo the transform, only when we did one.
        if text.orientation != Upright {
            self.draw_target.set_transform(current_transform)
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

        let stops = self.draw_target.create_gradient_stops(stops, ExtendClamp);
        let pattern = LinearGradientPattern::new(&start_point.to_azure_point(),
                                                 &end_point.to_azure_point(),
                                                 stops,
                                                 &Matrix2D::identity());

        self.draw_target.fill_rect(&bounds.to_azure_rect(),
                                   LinearGradientPatternRef(&pattern),
                                   None);
    }

    pub fn get_or_create_temporary_draw_target(&mut self, opacity: AzFloat) -> DrawTarget {
        if opacity == 1.0 {
            return self.draw_target.clone()
        }

        // FIXME(pcwalton): This surface might be bigger than necessary and waste memory.
        let size = self.draw_target.get_size();
        let size = Size2D {
            width: size.width,
            height: size.height,
        };

        let temporary_draw_target =
            self.draw_target.create_similar_draw_target(&size, self.draw_target.get_format());
        temporary_draw_target.set_transform(&self.draw_target.get_transform());
        temporary_draw_target
    }

    /// If we created a temporary draw target, then draw it to the main draw target. This is called
    /// after doing all the painting, and the temporary draw target must not be used afterward.
    pub fn draw_temporary_draw_target_if_necessary(&mut self,
                                                   temporary_draw_target: &DrawTarget,
                                                   opacity: AzFloat) {
        if (*temporary_draw_target) == self.draw_target {
            // We're directly rendering to the surface; nothing to do.
            return
        }

        let old_transform = self.draw_target.get_transform();
        self.draw_target.set_transform(&Matrix2D::identity());
        temporary_draw_target.set_transform(&Matrix2D::identity());
        let rect = Rect(Point2D(0.0, 0.0), self.draw_target.get_size().to_azure_size());
        let source_surface = temporary_draw_target.snapshot();
        let draw_surface_options = DrawSurfaceOptions::new(Linear, true);
        let draw_options = DrawOptions::new(opacity, 0);
        self.draw_target.draw_surface(source_surface,
                                      rect,
                                      rect,
                                      draw_surface_options,
                                      draw_options);
        self.draw_target.set_transform(&old_transform);
    }
}

pub trait ToAzurePoint {
    fn to_azure_point(&self) -> Point2D<AzFloat>;
}

impl ToAzurePoint for Point2D<Au> {
    fn to_azure_point(&self) -> Point2D<AzFloat> {
        Point2D(self.x.to_nearest_px() as AzFloat, self.y.to_nearest_px() as AzFloat)
    }
}

pub trait ToAzureRect {
    fn to_azure_rect(&self) -> Rect<AzFloat>;
}

impl ToAzureRect for Rect<Au> {
    fn to_azure_rect(&self) -> Rect<AzFloat> {
        Rect(self.origin.to_azure_point(),
             Size2D(self.size.width.to_nearest_px() as AzFloat,
                    self.size.height.to_nearest_px() as AzFloat))
    }
}

pub trait ToAzureSize {
    fn to_azure_size(&self) -> Size2D<AzFloat>;
}

impl ToAzureSize for AzIntSize {
    fn to_azure_size(&self) -> Size2D<AzFloat> {
        Size2D(self.width as AzFloat, self.height as AzFloat)
    }
}

trait ToSideOffsetsPx {
    fn to_float_px(&self) -> SideOffsets2D<AzFloat>;
}

impl ToSideOffsetsPx for SideOffsets2D<Au> {
    fn to_float_px(&self) -> SideOffsets2D<AzFloat> {
        SideOffsets2D::new(self.top.to_nearest_px() as AzFloat,
                           self.right.to_nearest_px() as AzFloat,
                           self.bottom.to_nearest_px() as AzFloat,
                           self.left.to_nearest_px() as AzFloat)
    }
}

trait ToRadiiPx {
    fn to_radii_px(&self) -> BorderRadii<AzFloat>;
}

impl ToRadiiPx for BorderRadii<Au> {
    fn to_radii_px(&self) -> BorderRadii<AzFloat> {
        BorderRadii {
            top_left: self.top_left.to_nearest_px() as AzFloat,
            top_right: self.top_right.to_nearest_px() as AzFloat,
            bottom_left: self.bottom_left.to_nearest_px() as AzFloat,
            bottom_right: self.bottom_right.to_nearest_px() as AzFloat,
        }
    }
}

trait ScaledFontExtensionMethods {
    fn draw_text_into_context(&self,
                              rctx: &RenderContext,
                              run: &Box<TextRun>,
                              range: &Range<CharIndex>,
                              baseline_origin: Point2D<Au>,
                              color: Color,
                              antialias: bool);
}

impl ScaledFontExtensionMethods for ScaledFont {
    fn draw_text_into_context(&self,
                              rctx: &RenderContext,
                              run: &Box<TextRun>,
                              range: &Range<CharIndex>,
                              baseline_origin: Point2D<Au>,
                              color: Color,
                              antialias: bool) {
        let target = rctx.get_draw_target();
        let pattern = ColorPattern::new(color);
        let azure_pattern = pattern.azure_color_pattern;
        assert!(azure_pattern.is_not_null());

        let fields = if antialias {
            0x0200
        } else {
            0
        };

        let mut options = struct__AzDrawOptions {
            mAlpha: 1f64 as AzFloat,
            fields: fields,
        };

        let mut origin = baseline_origin.clone();
        let mut azglyphs = vec!();
        azglyphs.reserve(range.length().to_uint());

        for (glyphs, _offset, slice_range) in run.iter_slices_for_range(range) {
            for (_i, glyph) in glyphs.iter_glyphs_for_char_range(&slice_range) {
                let glyph_advance = glyph.advance();
                let glyph_offset = glyph.offset().unwrap_or(Zero::zero());
                let azglyph = struct__AzGlyph {
                    mIndex: glyph.id() as uint32_t,
                    mPosition: struct__AzPoint {
                        x: (origin.x + glyph_offset.x).to_subpx() as AzFloat,
                        y: (origin.y + glyph_offset.y).to_subpx() as AzFloat
                    }
                };
                origin = Point2D(origin.x + glyph_advance, origin.y);
                azglyphs.push(azglyph)
            };
        }

        let azglyph_buf_len = azglyphs.len();
        if azglyph_buf_len == 0 { return; } // Otherwise the Quartz backend will assert.

        let mut glyphbuf = struct__AzGlyphBuffer {
            mGlyphs: azglyphs.as_mut_ptr(),
            mNumGlyphs: azglyph_buf_len as uint32_t
        };

        unsafe {
            // TODO(Issue #64): this call needs to move into azure_hl.rs
            AzDrawTargetFillGlyphs(target.azure_draw_target,
                                   self.get_ref(),
                                   &mut glyphbuf,
                                   azure_pattern,
                                   &mut options,
                                   ptr::null_mut());
        }
    }
}
