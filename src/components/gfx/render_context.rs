/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use servo_msg::compositor_msg::LayerBuffer;
use font_context::FontContext;
use geometry::Au;
use newcss::values::CSSBorderStyle;
use newcss::values::{CSSBorderStyleNone, CSSBorderStyleHidden, CSSBorderStyleDotted, CSSBorderStyleDashed, CSSBorderStyleSolid, CSSBorderStyleDouble, CSSBorderStyleGroove, CSSBorderStyleRidge, CSSBorderStyleInset, CSSBorderStyleOutset};
use opts::Opts;

use azure::azure_hl::{B8G8R8A8, Color, ColorPattern, DrawOptions};
use azure::azure_hl::{DrawSurfaceOptions, DrawTarget, Linear, StrokeOptions};
use azure::{AZ_CAP_BUTT, AZ_CAP_ROUND};
use azure::AZ_JOIN_BEVEL;
use azure::AzFloat;
use std::vec;
use std::libc::types::common::c99::uint16_t;
use std::libc::size_t;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use geom::side_offsets::SideOffsets2D;
use servo_net::image::base::Image;
use extra::arc::Arc;

pub struct RenderContext<'self> {
    canvas: &'self ~LayerBuffer,
    font_ctx: @mut FontContext,
    opts: &'self Opts
}

impl<'self> RenderContext<'self>  {
    pub fn get_draw_target(&self) -> &'self DrawTarget {
        &self.canvas.draw_target
    }

    pub fn draw_solid_color(&self, bounds: &Rect<Au>, color: Color) {
        self.canvas.draw_target.make_current();
        self.canvas.draw_target.fill_rect(&bounds.to_azure_rect(), &ColorPattern(color));
    }

    pub fn draw_border(&self,
                       bounds: &Rect<Au>,
                       border: SideOffsets2D<Au>,
                       color: SideOffsets2D<Color>,
                       style: SideOffsets2D<CSSBorderStyle>) {
        let draw_opts = DrawOptions(1 as AzFloat, 0 as uint16_t);
        let rect = bounds.to_azure_rect();
        let border = border.to_float_px();

        self.canvas.draw_target.make_current();
        let mut dash: [AzFloat, ..2] = [0 as AzFloat, 0 as AzFloat];
        let mut stroke_opts = StrokeOptions(0 as AzFloat, 10 as AzFloat);
 
        // draw top border
        RenderContext::apply_border_style(style.top, border.top, dash, &mut stroke_opts);
        let y = rect.origin.y + border.top * 0.5;
        let start = Point2D(rect.origin.x, y);
        let end = Point2D(rect.origin.x + rect.size.width, y);
        self.canvas.draw_target.stroke_line(start, end, &ColorPattern(color.top), &stroke_opts, &draw_opts);

        // draw right border
        RenderContext::apply_border_style(style.right, border.right, dash,  &mut stroke_opts);
        let x = rect.origin.x + rect.size.width - border.right * 0.5;
        let start = Point2D(x, rect.origin.y);
        let end = Point2D(x, rect.origin.y + rect.size.height);
        self.canvas.draw_target.stroke_line(start, end, &ColorPattern(color.right), &stroke_opts, &draw_opts);

        // draw bottom border
        RenderContext::apply_border_style(style.bottom, border.bottom, dash, &mut stroke_opts);
        let y = rect.origin.y + rect.size.height - border.bottom * 0.5;
        let start = Point2D(rect.origin.x, y);
        let end = Point2D(rect.origin.x + rect.size.width, y);
        self.canvas.draw_target.stroke_line(start, end, &ColorPattern(color.bottom), &stroke_opts, &draw_opts);

        // draw left border
        RenderContext::apply_border_style(style.left, border.left, dash,  &mut stroke_opts);
        let x = rect.origin.x + border.left * 0.5;
        let start = Point2D(x, rect.origin.y);
        let end = Point2D(x, rect.origin.y + rect.size.height);
        self.canvas.draw_target.stroke_line(start, end, &ColorPattern(color.left), &stroke_opts, &draw_opts);
    }

    pub fn draw_image(&self, bounds: Rect<Au>, image: Arc<~Image>) {
        let image = image.get();
        let size = Size2D(image.width as i32, image.height as i32);
        let stride = image.width * 4;

        self.canvas.draw_target.make_current();
        let draw_target_ref = &self.canvas.draw_target;
        let azure_surface = draw_target_ref.create_source_surface_from_data(image.data, size,
                                                                            stride as i32, B8G8R8A8);
        let source_rect = Rect(Point2D(0 as AzFloat, 0 as AzFloat),
                               Size2D(image.width as AzFloat, image.height as AzFloat));
        let dest_rect = bounds.to_azure_rect();
        let draw_surface_options = DrawSurfaceOptions(Linear, true);
        let draw_options = DrawOptions(1.0f as AzFloat, 0);
        draw_target_ref.draw_surface(azure_surface,
                                     dest_rect,
                                     source_rect,
                                     draw_surface_options,
                                     draw_options);
    }

    pub fn clear(&self) {
        let pattern = ColorPattern(Color(1.0, 1.0, 1.0, 1.0));
        let rect = Rect(Point2D(self.canvas.rect.origin.x as AzFloat,
                                self.canvas.rect.origin.y as AzFloat),
                        Size2D(self.canvas.screen_pos.size.width as AzFloat,
                               self.canvas.screen_pos.size.height as AzFloat));
        self.canvas.draw_target.make_current();
        self.canvas.draw_target.fill_rect(&rect, &pattern);
    }

    fn apply_border_style(style: CSSBorderStyle, border_width: AzFloat, dash: &mut [AzFloat], stroke_opts: &mut StrokeOptions){
        match style{
            CSSBorderStyleNone => {
            }
            CSSBorderStyleHidden => {
            }
            //FIXME(sammykim): This doesn't work with dash_pattern and cap_style well. I referred firefox code.
            CSSBorderStyleDotted => {
                stroke_opts.line_width = border_width;
                
                if border_width > 2.0 {
                    dash[0] = 0 as AzFloat;
                    dash[1] = border_width * 2.0;

                    stroke_opts.set_cap_style(AZ_CAP_ROUND as u8);
                } else {
                    dash[0] = border_width;
                    dash[1] = border_width;
                }
                stroke_opts.mDashPattern = vec::raw::to_ptr(dash);
                stroke_opts.mDashLength = dash.len() as size_t;
            }
            CSSBorderStyleDashed => {
                stroke_opts.set_cap_style(AZ_CAP_BUTT as u8);
                stroke_opts.line_width = border_width;
                dash[0] = border_width*3 as AzFloat;
                dash[1] = border_width*3 as AzFloat;
                stroke_opts.mDashPattern = vec::raw::to_ptr(dash);
                stroke_opts.mDashLength = dash.len() as size_t;
            }
            //FIXME(sammykim): BorderStyleSolid doesn't show proper join-style with comparing firefox.
            CSSBorderStyleSolid => {
                stroke_opts.set_cap_style(AZ_CAP_BUTT as u8);
                stroke_opts.set_join_style(AZ_JOIN_BEVEL as u8);
                stroke_opts.line_width = border_width; 
                stroke_opts.mDashLength = 0 as size_t;
            }            
            //FIXME(sammykim): Five more styles should be implemented.
            CSSBorderStyleDouble => {

            }
            CSSBorderStyleGroove => {

            }
            CSSBorderStyleRidge => {

            }
            CSSBorderStyleInset => {

            }
            CSSBorderStyleOutset => {

            }
        }
    }
}

trait to_float {
    fn to_float(&self) -> float;
}

impl to_float for u8 {
    fn to_float(&self) -> float {
        (*self as float) / 255f
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
