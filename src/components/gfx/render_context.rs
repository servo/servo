/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use font_context::FontContext;
use style::computed_values::border_style;

use azure::azure_hl::{B8G8R8A8, Color, ColorPattern, DrawOptions, DrawSurfaceOptions, DrawTarget};
use azure::azure_hl::{Linear, SourceOp, StrokeOptions};
use azure::AZ_CAP_BUTT;
use azure::AzFloat;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use geom::side_offsets::SideOffsets2D;
use libc::types::common::c99::uint16_t;
use libc::size_t;
use png::{RGBA8, K8, KA8};
use servo_net::image::base::Image;
use servo_util::geometry::Au;
use servo_util::opts::Opts;
use sync::Arc;

pub struct RenderContext<'a> {
    pub draw_target: &'a DrawTarget,
    pub font_ctx: &'a mut ~FontContext,
    pub opts: &'a Opts,
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
    pub fn get_draw_target(&self) -> &'a DrawTarget {
        self.draw_target
    }

    pub fn draw_solid_color(&self, bounds: &Rect<Au>, color: Color) {
        self.draw_target.make_current();
        self.draw_target.fill_rect(&bounds.to_azure_rect(), &ColorPattern(color), None);
    }

    pub fn draw_border(&self,
                       bounds: &Rect<Au>,
                       border: SideOffsets2D<Au>,
                       color: SideOffsets2D<Color>,
                       style: SideOffsets2D<border_style::T>) {
        let border = border.to_float_px();
        self.draw_target.make_current();

        self.draw_border_segment(Top, bounds, border, color, style);
        self.draw_border_segment(Right, bounds, border, color, style);
        self.draw_border_segment(Bottom, bounds, border, color, style);
        self.draw_border_segment(Left, bounds, border, color, style);
    }

    pub fn draw_line(&self,
                     bounds: &Rect<Au>,
                     color: Color,
                     style: border_style::T) {
        self.draw_target.make_current();

        self.draw_line_segment(bounds, color, style);
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

    pub fn draw_image(&self, bounds: Rect<Au>, image: Arc<~Image>) {
        let size = Size2D(image.width as i32, image.height as i32);
        let pixel_width = match image.color_type {
            RGBA8 => 4,
            K8    => 1,
            KA8   => 2,
            _     => fail!("color type not supported"),
        };
        let stride = image.width * pixel_width;

        self.draw_target.make_current();
        let draw_target_ref = &self.draw_target;
        let azure_surface = draw_target_ref.create_source_surface_from_data(image.pixels.as_slice(), size,
                                                                            stride as i32, B8G8R8A8);
        let source_rect = Rect(Point2D(0 as AzFloat, 0 as AzFloat),
                               Size2D(image.width as AzFloat, image.height as AzFloat));
        let dest_rect = bounds.to_azure_rect();
        let draw_surface_options = DrawSurfaceOptions(Linear, true);
        let draw_options = DrawOptions(1.0f64 as AzFloat, 0);
        draw_target_ref.draw_surface(azure_surface,
                                     dest_rect,
                                     source_rect,
                                     draw_surface_options,
                                     draw_options);
    }

    pub fn clear(&self) {
        let pattern = ColorPattern(Color(0.0, 0.0, 0.0, 0.0));
        let rect = Rect(Point2D(self.page_rect.origin.x as AzFloat,
                                self.page_rect.origin.y as AzFloat),
                        Size2D(self.screen_rect.size.width as AzFloat,
                               self.screen_rect.size.height as AzFloat));
        let mut draw_options = DrawOptions(1.0, 0);
        draw_options.set_composition_op(SourceOp);
        self.draw_target.make_current();
        self.draw_target.fill_rect(&rect, &pattern, Some(&draw_options));
    }

    fn draw_border_segment(&self, direction: Direction, bounds: &Rect<Au>, border: SideOffsets2D<f32>, color: SideOffsets2D<Color>, style: SideOffsets2D<border_style::T>) {
        let (style_select, color_select) = match direction {
            Top => (style.top, color.top),
            Left => (style.left, color.left),
            Right => (style.right, color.right),
            Bottom => (style.bottom, color.bottom)
        };

        match style_select{
            border_style::none => {
            }
            border_style::hidden => {
            }
            //FIXME(sammykim): This doesn't work with dash_pattern and cap_style well. I referred firefox code.
            border_style::dotted => {
                self.draw_dashed_border_segment(direction, bounds, border, color_select, DottedBorder);
            }
            border_style::dashed => {
                self.draw_dashed_border_segment(direction, bounds, border, color_select, DashedBorder);
            }
            border_style::solid => {
                self.draw_solid_border_segment(direction,bounds,border,color_select);
            }
            //FIXME(sammykim): Five more styles should be implemented.
            //double, groove, ridge, inset, outset
        }
    }

    fn draw_line_segment(&self, bounds: &Rect<Au>, color: Color, style: border_style::T) {
        let border = SideOffsets2D::new_all_same(bounds.size.width).to_float_px();

        match style{
            border_style::none | border_style::hidden => {}
            border_style::dotted => {
                self.draw_dashed_border_segment(Right, bounds, border, color, DottedBorder);
            }
            border_style::dashed => {
                self.draw_dashed_border_segment(Right, bounds, border, color, DashedBorder);
            }
            border_style::solid => {
                self.draw_solid_border_segment(Right,bounds,border,color);
            }
            //FIXME(sankha93): Five more styles should be implemented.
            //double, groove, ridge, inset, outset
        }
    }

    fn draw_dashed_border_segment(&self, 
                                  direction: Direction, 
                                  bounds:    &Rect<Au>, 
                                  border:    SideOffsets2D<f32>, 
                                  color:     Color, 
                                  dash_size: DashSize) {
        let rect = bounds.to_azure_rect();
        let draw_opts = DrawOptions(1 as AzFloat, 0 as uint16_t);
        let mut stroke_opts = StrokeOptions(0 as AzFloat, 10 as AzFloat);
        let mut dash: [AzFloat, ..2] = [0 as AzFloat, 0 as AzFloat];

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
        stroke_opts.mDashPattern = dash.as_ptr();
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
                                     &ColorPattern(color),
                                     &stroke_opts,
                                     &draw_opts);
    }

    fn draw_solid_border_segment(&self, direction: Direction, bounds: &Rect<Au>, border: SideOffsets2D<f32>, color: Color) {
        let rect = bounds.to_azure_rect();
        let draw_opts = DrawOptions(1.0 , 0);
        let path_builder = self.draw_target.create_path_builder();

        let left_top = Point2D(rect.origin.x, rect.origin.y);
        let right_top = Point2D(rect.origin.x + rect.size.width, rect.origin.y);
        let left_bottom = Point2D(rect.origin.x, rect.origin.y + rect.size.height);
        let right_bottom = Point2D(rect.origin.x + rect.size.width, rect.origin.y + rect.size.height);

        match direction {
            Top => {
                path_builder.move_to(left_top);
                path_builder.line_to(right_top);
                path_builder.line_to(right_top + Point2D(-border.right, border.top));
                path_builder.line_to(left_top + Point2D(border.left, border.top));
            }
            Left => {
                path_builder.move_to(left_top);
                path_builder.line_to(left_top + Point2D(border.left, border.top));
                path_builder.line_to(left_bottom + Point2D(border.left, -border.bottom));
                path_builder.line_to(left_bottom);
            }
            Right => {
                path_builder.move_to(right_top);
                path_builder.line_to(right_bottom);
                path_builder.line_to(right_bottom + Point2D(-border.right, -border.bottom));
                path_builder.line_to(right_top + Point2D(-border.right, border.top));
            }
            Bottom => {
                path_builder.move_to(left_bottom);
                path_builder.line_to(left_bottom + Point2D(border.left, -border.bottom));
                path_builder.line_to(right_bottom + Point2D(-border.right, -border.bottom));
                path_builder.line_to(right_bottom);
            }
        }

        let path = path_builder.finish();
        self.draw_target.fill(&path, &ColorPattern(color), &draw_opts);
    }
}

trait ToAzureRect {
    fn to_azure_rect(&self) -> Rect<AzFloat>;
}

impl ToAzureRect for Rect<Au> {
    fn to_azure_rect(&self) -> Rect<AzFloat> {
        Rect(Point2D(self.origin.x.to_nearest_px() as AzFloat,
                     self.origin.y.to_nearest_px() as AzFloat),
             Size2D(self.size.width.to_nearest_px() as AzFloat,
                    self.size.height.to_nearest_px() as AzFloat))
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
