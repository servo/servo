/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Painting of display lists using Moz2D/Azure.

use gfx_traits::color;
use display_list::TextOrientation::{SidewaysLeft, SidewaysRight, Upright};
use display_list::{BLUR_INFLATION_FACTOR, BorderRadii, BoxShadowClipMode, ClippingRegion};
use display_list::{TextDisplayItem};
use filters;
use font_context::FontContext;
use text::TextRun;
use text::glyph::CharIndex;

use azure::azure::AzIntSize;
use azure::azure_hl::{AntialiasMode, Color, ColorPattern, CompositionOp};
use azure::azure_hl::{DrawOptions, DrawSurfaceOptions, DrawTarget, ExtendMode, FilterType};
use azure::azure_hl::{GaussianBlurAttribute, StrokeOptions, SurfaceFormat};
use azure::azure_hl::{GaussianBlurInput, GradientStop, Filter, FilterNode, LinearGradientPattern};
use azure::azure_hl::{JoinStyle, CapStyle};
use azure::azure_hl::{Pattern, PatternRef, Path, PathBuilder, SurfacePattern};
use azure::scaled_font::ScaledFont;
use azure::{AzFloat, struct__AzDrawOptions, struct__AzGlyph};
use azure::{struct__AzGlyphBuffer, struct__AzPoint, AzDrawTargetFillGlyphs};
use euclid::matrix2d::Matrix2D;
use euclid::point::Point2D;
use euclid::rect::Rect;
use euclid::side_offsets::SideOffsets2D;
use euclid::size::Size2D;
use libc::types::common::c99::uint32_t;
use msg::compositor_msg::LayerKind;
use net_traits::image::base::{Image, PixelFormat};
use std::default::Default;
use std::f32;
use std::mem;
use std::ptr;
use std::sync::Arc;
use style::computed_values::{border_style, filter, image_rendering, mix_blend_mode};
use util::geometry::{self, Au, MAX_RECT, ZERO_RECT};
use util::opts;
use util::range::Range;

pub struct PaintContext<'a> {
    pub draw_target: DrawTarget,
    pub font_context: &'a mut Box<FontContext>,
    /// The rectangle that this context encompasses in page coordinates.
    pub page_rect: Rect<f32>,
    /// The rectangle that this context encompasses in screen coordinates (pixels).
    pub screen_rect: Rect<usize>,
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
enum DashSize {
    DottedBorder = 1,
    DashedBorder = 3
}

impl<'a> PaintContext<'a> {
    pub fn draw_target(&self) -> &DrawTarget {
        &self.draw_target
    }

    pub fn draw_solid_color(&self, bounds: &Rect<Au>, color: Color) {
        self.draw_target.make_current();
        self.draw_target.fill_rect(&bounds.to_nearest_azure_rect(),
                                   PatternRef::Color(&ColorPattern::new(color)),
                                   None);
    }

    pub fn draw_border(&self,
                       bounds: &Rect<Au>,
                       border: &SideOffsets2D<Au>,
                       radius: &BorderRadii<Au>,
                       color: &SideOffsets2D<Color>,
                       style: &SideOffsets2D<border_style::T>) {
        let border = border.to_float_px();
        let radius = radius.to_radii_px();

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
        let rect = bounds.to_nearest_azure_rect();
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
                      image: Arc<Image>,
                      image_rendering: image_rendering::T) {
        let size = Size2D::new(image.width as i32, image.height as i32);
        let (pixel_width, source_format) = match image.format {
            PixelFormat::RGBA8 => (4, SurfaceFormat::B8G8R8A8),
            PixelFormat::K8 => (1, SurfaceFormat::A8),
            PixelFormat::RGB8 => panic!("RGB8 color type not supported"),
            PixelFormat::KA8 => panic!("KA8 color type not supported"),
        };
        let stride = image.width * pixel_width;

        self.draw_target.make_current();
        let draw_target_ref = &self.draw_target;
        let azure_surface = draw_target_ref.create_source_surface_from_data(&image.bytes,
                                                                            size,
                                                                            stride as i32,
                                                                            source_format);
        let source_rect = Rect::new(Point2D::new(0.0, 0.0),
                                    Size2D::new(image.width as AzFloat, image.height as AzFloat));
        let dest_rect = bounds.to_nearest_azure_rect();

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
        let stretch_size = stretch_size.to_nearest_azure_size();
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
        let rect = Rect::new(Point2D::new(self.page_rect.origin.x as AzFloat,
                                          self.page_rect.origin.y as AzFloat),
                             Size2D::new(self.screen_rect.size.width as AzFloat,
                                         self.screen_rect.size.height as AzFloat));
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
                                                color_select,
                                                DashSize::DottedBorder);
            }
            border_style::T::dashed => {
                self.draw_dashed_border_segment(direction,
                                                bounds,
                                                border,
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
        let border = SideOffsets2D::new_all_same(bounds.size.width).to_float_px();
        match style {
            border_style::T::none | border_style::T::hidden => {}
            border_style::T::dotted => {
                self.draw_dashed_border_segment(Direction::Right,
                                                bounds,
                                                &border,
                                                color,
                                                DashSize::DottedBorder);
            }
            border_style::T::dashed => {
                self.draw_dashed_border_segment(Direction::Right,
                                                bounds,
                                                &border,
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
        self.create_border_path_segment(&mut path_builder, bounds, direction, border, radii);
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
    fn create_border_path_segment(&self,
                                  path_builder: &mut PathBuilder,
                                  bounds: &Rect<f32>,
                                  direction: Direction,
                                  border: &SideOffsets2D<f32>,
                                  radius: &BorderRadii<AzFloat>) {
        // T = top, B = bottom, L = left, R = right

        let box_TL = bounds.origin;
        let box_TR = box_TL + Point2D::new(bounds.size.width, 0.0);
        let box_BL = box_TL + Point2D::new(0.0, bounds.size.height);
        let box_BR = box_TL + Point2D::new(bounds.size.width, bounds.size.height);

        let rad_R: AzFloat = 0.;
        let rad_BR = rad_R  + f32::consts::FRAC_PI_4;
        let rad_B  = rad_BR + f32::consts::FRAC_PI_4;
        let rad_BL = rad_B  + f32::consts::FRAC_PI_4;
        let rad_L  = rad_BL + f32::consts::FRAC_PI_4;
        let rad_TL = rad_L  + f32::consts::FRAC_PI_4;
        let rad_T  = rad_TL + f32::consts::FRAC_PI_4;
        let rad_TR = rad_T  + f32::consts::FRAC_PI_4;

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

        match direction {
            Direction::Top => {
                let edge_TL = box_TL + dx(radius.top_left.max(border.left));
                let edge_TR = box_TR + dx(-radius.top_right.max(border.right));
                let edge_BR = edge_TR + dy(border.top);
                let edge_BL = edge_TL + dy(border.top);

                let corner_TL = edge_TL + dx_if(radius.top_left == 0., -border.left);
                let corner_TR = edge_TR + dx_if(radius.top_right == 0., border.right);

                path_builder.move_to(corner_TL);
                path_builder.line_to(corner_TR);

                if radius.top_right != 0. {
                    // the origin is the center of the arcs we're about to draw.
                    let origin = edge_TR + Point2D::new((border.right - radius.top_right).max(0.),
                                                        radius.top_right);
                    // the elbow is the inside of the border's curve.
                    let distance_to_elbow = (radius.top_right - border.top).max(0.);

                    path_builder.arc(origin, radius.top_right,  rad_T, rad_TR, false);
                    path_builder.arc(origin, distance_to_elbow, rad_TR, rad_T, true);
                }

                path_builder.line_to(edge_BR);
                path_builder.line_to(edge_BL);

                if radius.top_left != 0. {
                    let origin = edge_TL + Point2D::new(-(border.left - radius.top_left).max(0.),
                                                   radius.top_left);
                    let distance_to_elbow = (radius.top_left - border.top).max(0.);

                    path_builder.arc(origin, distance_to_elbow, rad_T, rad_TL, true);
                    path_builder.arc(origin, radius.top_left,   rad_TL, rad_T, false);
                }
            }
            Direction::Left => {
                let edge_TL = box_TL + dy(radius.top_left.max(border.top));
                let edge_BL = box_BL + dy(-radius.bottom_left.max(border.bottom));
                let edge_TR = edge_TL + dx(border.left);
                let edge_BR = edge_BL + dx(border.left);

                let corner_TL = edge_TL + dy_if(radius.top_left == 0., -border.top);
                let corner_BL = edge_BL + dy_if(radius.bottom_left == 0., border.bottom);

                path_builder.move_to(corner_BL);
                path_builder.line_to(corner_TL);

                if radius.top_left != 0. {
                    let origin = edge_TL + Point2D::new(radius.top_left,
                                                   -(border.top - radius.top_left).max(0.));
                    let distance_to_elbow = (radius.top_left - border.left).max(0.);

                    path_builder.arc(origin, radius.top_left,   rad_L, rad_TL, false);
                    path_builder.arc(origin, distance_to_elbow, rad_TL, rad_L, true);
                }

                path_builder.line_to(edge_TR);
                path_builder.line_to(edge_BR);

                if radius.bottom_left != 0. {
                    let origin = edge_BL +
                        Point2D::new(radius.bottom_left,
                                (border.bottom - radius.bottom_left).max(0.));
                    let distance_to_elbow = (radius.bottom_left - border.left).max(0.);

                    path_builder.arc(origin, distance_to_elbow,  rad_L, rad_BL, true);
                    path_builder.arc(origin, radius.bottom_left, rad_BL, rad_L, false);
                }
            }
            Direction::Right => {
                let edge_TR = box_TR + dy(radius.top_right.max(border.top));
                let edge_BR = box_BR + dy(-radius.bottom_right.max(border.bottom));
                let edge_TL = edge_TR + dx(-border.right);
                let edge_BL = edge_BR + dx(-border.right);

                let corner_TR = edge_TR + dy_if(radius.top_right == 0., -border.top);
                let corner_BR = edge_BR + dy_if(radius.bottom_right == 0., border.bottom);

                path_builder.move_to(edge_BL);
                path_builder.line_to(edge_TL);

                if radius.top_right != 0. {
                    let origin = edge_TR + Point2D::new(-radius.top_right,
                                                        -(border.top - radius.top_right).max(0.));
                    let distance_to_elbow = (radius.top_right - border.right).max(0.);

                    path_builder.arc(origin, distance_to_elbow, rad_R, rad_TR, true);
                    path_builder.arc(origin, radius.top_right,  rad_TR, rad_R, false);
                }

                path_builder.line_to(corner_TR);
                path_builder.line_to(corner_BR);

                if radius.bottom_right != 0. {
                    let origin = edge_BR +
                        Point2D::new(-radius.bottom_right,
                                (border.bottom - radius.bottom_right).max(0.));
                    let distance_to_elbow = (radius.bottom_right - border.right).max(0.);

                    path_builder.arc(origin, radius.bottom_right, rad_R, rad_BR, false);
                    path_builder.arc(origin, distance_to_elbow,   rad_BR, rad_R, true);
                }
            }
            Direction::Bottom => {
                let edge_BL = box_BL + dx(radius.bottom_left.max(border.left));
                let edge_BR = box_BR + dx(-radius.bottom_right.max(border.right));
                let edge_TL = edge_BL + dy(-border.bottom);
                let edge_TR = edge_BR + dy(-border.bottom);

                let corner_BR = edge_BR + dx_if(radius.bottom_right == 0., border.right);
                let corner_BL = edge_BL + dx_if(radius.bottom_left == 0., -border.left);

                path_builder.move_to(edge_TL);
                path_builder.line_to(edge_TR);

                if radius.bottom_right != 0. {
                    let origin = edge_BR + Point2D::new((border.right - radius.bottom_right).max(0.),
                                                        -radius.bottom_right);
                    let distance_to_elbow = (radius.bottom_right - border.bottom).max(0.);

                    path_builder.arc(origin, distance_to_elbow,   rad_B, rad_BR, true);
                    path_builder.arc(origin, radius.bottom_right, rad_BR, rad_B, false);
                }

                path_builder.line_to(corner_BR);
                path_builder.line_to(corner_BL);

                if radius.bottom_left != 0. {
                    let origin = edge_BL - Point2D::new((border.left - radius.bottom_left).max(0.),
                                                   radius.bottom_left);
                    let distance_to_elbow = (radius.bottom_left - border.bottom).max(0.);

                    path_builder.arc(origin, radius.bottom_left, rad_B, rad_BL, false);
                    path_builder.arc(origin, distance_to_elbow,  rad_BL, rad_B, true);
                }
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

        path_builder.move_to(Point2D::new(bounds.origin.x + radii.top_left, bounds.origin.y));   // 1
        path_builder.line_to(Point2D::new(bounds.max_x() - radii.top_right, bounds.origin.y));   // 2
        path_builder.arc(Point2D::new(bounds.max_x() - radii.top_right,
                                      bounds.origin.y + radii.top_right),
                         radii.top_right,
                         1.5f32 * f32::consts::FRAC_PI_2,
                         2.0f32 * f32::consts::PI,
                         false);                                                                 // 3
        path_builder.line_to(Point2D::new(bounds.max_x(), bounds.max_y() - radii.bottom_right)); // 4
        path_builder.arc(Point2D::new(bounds.max_x() - radii.bottom_right,
                                      bounds.max_y() - radii.bottom_right),
                         radii.bottom_right,
                         0.0,
                         f32::consts::FRAC_PI_2,
                         false);                                                                 // 5
        path_builder.line_to(Point2D::new(bounds.origin.x + radii.bottom_left, bounds.max_y())); // 6
        path_builder.arc(Point2D::new(bounds.origin.x + radii.bottom_left,
                                 bounds.max_y() - radii.bottom_left),
                         radii.bottom_left,
                         f32::consts::FRAC_PI_2,
                         f32::consts::PI,
                         false);                                                                 // 7
        path_builder.line_to(Point2D::new(bounds.origin.x, bounds.origin.y + radii.top_left));   // 8
        path_builder.arc(Point2D::new(bounds.origin.x + radii.top_left,
                                      bounds.origin.y + radii.top_left),
                         radii.top_left,
                         f32::consts::PI,
                         1.5f32 * f32::consts::FRAC_PI_2,
                         false);                                                                 // 1
    }

    fn draw_dashed_border_segment(&self,
                                  direction: Direction,
                                  bounds: &Rect<Au>,
                                  border: &SideOffsets2D<f32>,
                                  color: Color,
                                  dash_size: DashSize) {
        let rect = bounds.to_nearest_azure_rect();
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
                let start = Point2D::new(rect.origin.x, y);
                let end = Point2D::new(rect.origin.x + rect.size.width, y);
                (start, end)
            }
            Direction::Left => {
                let x = rect.origin.x + border.left * 0.5;
                let start = Point2D::new(x, rect.origin.y + rect.size.height);
                let end = Point2D::new(x, rect.origin.y + border.top);
                (start, end)
            }
            Direction::Right => {
                let x = rect.origin.x + rect.size.width - border.right * 0.5;
                let start = Point2D::new(x, rect.origin.y);
                let end = Point2D::new(x, rect.origin.y + rect.size.height);
                (start, end)
            }
            Direction::Bottom => {
                let y = rect.origin.y + rect.size.height - border.bottom * 0.5;
                let start = Point2D::new(rect.origin.x + rect.size.width, y);
                let end = Point2D::new(rect.origin.x + border.left, y);
                (start, end)
            }
        };

        self.draw_target.stroke_line(start,
                                     end,
                                     PatternRef::Color(&ColorPattern::new(color)),
                                     &stroke_opts,
                                     &draw_opts);
    }

    fn draw_solid_border_segment(&self,
                                 direction: Direction,
                                 bounds: &Rect<Au>,
                                 border: &SideOffsets2D<f32>,
                                 radius: &BorderRadii<AzFloat>,
                                 color: Color) {
        let rect = bounds.to_nearest_azure_rect();
        self.draw_border_path(&rect, direction, border, radius, color);
    }

    fn compute_scaled_bounds(&self,
                             bounds: &Rect<Au>,
                             border: &SideOffsets2D<f32>,
                             shrink_factor: f32) -> Rect<f32> {
        let rect            = bounds.to_nearest_azure_rect();
        let scaled_border   = SideOffsets2D::new(shrink_factor * border.top,
                                                 shrink_factor * border.right,
                                                 shrink_factor * border.bottom,
                                                 shrink_factor * border.left);
        let left_top        = Point2D::new(rect.origin.x, rect.origin.y);
        let scaled_left_top = left_top + Point2D::new(scaled_border.left,
                                                      scaled_border.top);
        return Rect::new(scaled_left_top,
                         Size2D::new(rect.size.width - 2.0 * scaled_border.right,
                                     rect.size.height - 2.0 * scaled_border.bottom));
    }

    fn scale_color(&self, color: Color, scale_factor: f32) -> Color {
        return color::new(color.r * scale_factor,
                          color.g * scale_factor,
                          color.b * scale_factor,
                          color.a);
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
                border_style::T::groove =>  true,
                border_style::T::ridge  =>  false,
                _ =>  panic!("invalid border style")
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
            let font = self.font_context.get_paint_font_from_template(
                &text.text_run.font_template, text.text_run.actual_pt_size);
            font
            .borrow()
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
        let pattern = LinearGradientPattern::new(&start_point.to_nearest_azure_point(),
                                                 &end_point.to_nearest_azure_point(),
                                                 stops,
                                                 &Matrix2D::identity());
        self.draw_target.fill_rect(&bounds.to_nearest_azure_rect(),
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
        let size = self.draw_target.get_size(); //Az size.
        let mut size = Size2D::new(size.width, size.height); //Geom::Size.

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
            let inflated_size = Rect::new(Point2D::new(0.0, 0.0), Size2D::new(size.width as AzFloat,
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

        let rect_temporary = Rect::new(Point2D::new(0.0, 0.0), temporary_draw_target.get_size().to_azure_size());

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
            self.draw_target.draw_filter(&filter_node, &rect_temporary, &rect_temporary.origin, draw_options);
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
                                                                            &shadow_bounds);
                self.draw_target.push_clip(&self.draw_target.create_rectangular_path(box_bounds))
            }
            BoxShadowClipMode::Outset => {
                path = temporary_draw_target.draw_target.create_rectangular_path(&shadow_bounds);
                self.draw_target.push_clip(&self.draw_target
                                                .create_rectangular_border_path(&MAX_RECT,
                                                                                box_bounds))
            }
            BoxShadowClipMode::None => {
                path = temporary_draw_target.draw_target.create_rectangular_path(&shadow_bounds)
            }
        }

        // Draw the shadow, and blur if we need to.
        temporary_draw_target.draw_target.fill(&path,
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
            geometry::f32_rect_to_au_rect(self.page_rect).intersection(box_bounds)
                                                         .unwrap_or(ZERO_RECT)
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
        self.remove_transient_clip_if_applicable();

        self.draw_push_clip(&clip_region.main);
        for complex_region in &clip_region.complex {
            // FIXME(pcwalton): Actually draw a rounded rect.
            self.push_rounded_rect_clip(&complex_region.rect.to_nearest_azure_rect(),
                                        &complex_region.radii.to_radii_px())
        }
        self.transient_clip = Some(clip_region)
    }
}

pub trait ToAzurePoint {
    fn to_nearest_azure_point(&self) -> Point2D<AzFloat>;
    fn to_azure_point(&self) -> Point2D<AzFloat>;
}

impl ToAzurePoint for Point2D<Au> {
    fn to_nearest_azure_point(&self) -> Point2D<AzFloat> {
        Point2D::new(self.x.to_nearest_px() as AzFloat, self.y.to_nearest_px() as AzFloat)
    }
    fn to_azure_point(&self) -> Point2D<AzFloat> {
        Point2D::new(self.x.to_f32_px(), self.y.to_f32_px())
    }
}

pub trait ToAzureRect {
    fn to_nearest_azure_rect(&self) -> Rect<AzFloat>;
    fn to_azure_rect(&self) -> Rect<AzFloat>;
}

impl ToAzureRect for Rect<Au> {
    fn to_nearest_azure_rect(&self) -> Rect<AzFloat> {
        Rect::new(self.origin.to_nearest_azure_point(), self.size.to_nearest_azure_size())
    }
    fn to_azure_rect(&self) -> Rect<AzFloat> {
        Rect::new(self.origin.to_azure_point(), self.size.to_azure_size())
    }
}

pub trait ToAzureSize {
    fn to_nearest_azure_size(&self) -> Size2D<AzFloat>;
    fn to_azure_size(&self) -> Size2D<AzFloat>;
}

impl ToAzureSize for Size2D<Au> {
    fn to_nearest_azure_size(&self) -> Size2D<AzFloat> {
        Size2D::new(self.width.to_nearest_px() as AzFloat, self.height.to_nearest_px() as AzFloat)
    }
    fn to_azure_size(&self) -> Size2D<AzFloat> {
        Size2D::new(self.width.to_f32_px(), self.height.to_f32_px())
    }
}

impl ToAzureSize for AzIntSize {
    fn to_nearest_azure_size(&self) -> Size2D<AzFloat> {
        Size2D::new(self.width as AzFloat, self.height as AzFloat)
    }
    fn to_azure_size(&self) -> Size2D<AzFloat> {
        Size2D::new(self.width as AzFloat, self.height as AzFloat)
    }
}

trait ToAzureIntSize {
    fn to_azure_int_size(&self) -> Size2D<i32>;
}

impl ToAzureIntSize for Size2D<Au> {
    fn to_azure_int_size(&self) -> Size2D<i32> {
        Size2D::new(self.width.to_nearest_px() as i32, self.height.to_nearest_px() as i32)
    }
}

impl ToAzureIntSize for Size2D<AzFloat> {
    fn to_azure_int_size(&self) -> Size2D<i32> {
        Size2D::new(self.width as i32, self.height as i32)
    }
}

impl ToAzureIntSize for Size2D<i32> {
    fn to_azure_int_size(&self) -> Size2D<i32> {
        Size2D::new(self.width, self.height)
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
        fn to_nearest_px(x: Au) -> AzFloat {
            x.to_nearest_px() as AzFloat
        }

        BorderRadii {
            top_left: to_nearest_px(self.top_left),
            top_right: to_nearest_px(self.top_right),
            bottom_left: to_nearest_px(self.bottom_left),
            bottom_right: to_nearest_px(self.bottom_right),
        }
    }
}

trait ScaledFontExtensionMethods {
    fn draw_text(&self,
                 draw_target: &DrawTarget,
                 run: &TextRun,
                 range: &Range<CharIndex>,
                 baseline_origin: Point2D<Au>,
                 color: Color,
                 antialias: bool);
}

impl ScaledFontExtensionMethods for ScaledFont {
    fn draw_text(&self,
                 draw_target: &DrawTarget,
                 run: &TextRun,
                 range: &Range<CharIndex>,
                 baseline_origin: Point2D<Au>,
                 color: Color,
                 antialias: bool) {
        let pattern = ColorPattern::new(color);
        let azure_pattern = pattern.azure_color_pattern;
        assert!(!azure_pattern.is_null());

        let mut options = struct__AzDrawOptions {
            mAlpha: 1f64 as AzFloat,
            mCompositionOp: CompositionOp::Over as u8,
            mAntialiasMode: if antialias { AntialiasMode::Subpixel as u8 }
                            else { AntialiasMode::None as u8 }
        };

        let mut origin = baseline_origin.clone();
        let mut azglyphs = vec!();
        azglyphs.reserve(range.length().to_usize());

        for slice in run.natural_word_slices_in_visual_order(range) {
            for (_i, glyph) in slice.glyphs.iter_glyphs_for_char_range(&slice.range) {
                let glyph_advance = glyph.advance();
                let glyph_offset = glyph.offset().unwrap_or(Point2D::zero());
                let azglyph = struct__AzGlyph {
                    mIndex: glyph.id() as uint32_t,
                    mPosition: struct__AzPoint {
                        x: (origin.x + glyph_offset.x).to_f32_px(),
                        y: (origin.y + glyph_offset.y).to_f32_px(),
                    }
                };
                origin = Point2D::new(origin.x + glyph_advance, origin.y);
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
    fn create_rectangular_border_path<T>(&self, outer_rect: &T, inner_rect: &T)
                                         -> Path
                                         where T: ToAzureRect;

    /// Creates and returns a path that represents a rectangle.
    fn create_rectangular_path(&self, rect: &Rect<Au>) -> Path;
}

impl DrawTargetExtensions for DrawTarget {
    fn create_rectangular_border_path<T>(&self, outer_rect: &T, inner_rect: &T)
                                         -> Path
                                         where T: ToAzureRect {
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

        let (outer_rect, inner_rect) = (outer_rect.to_nearest_azure_rect(), inner_rect.to_nearest_azure_rect());
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

    fn create_rectangular_path(&self, rect: &Rect<Au>) -> Path {
        let rect = rect.to_nearest_azure_rect();
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
