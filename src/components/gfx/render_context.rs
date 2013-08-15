/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use servo_msg::compositor_msg::LayerBuffer;
use font_context::FontContext;
use geometry::Au;
use opts::Opts;

use azure::azure_hl::{B8G8R8A8, Color, ColorPattern, DrawOptions};
use azure::azure_hl::{DrawSurfaceOptions, DrawTarget, Linear, StrokeOptions};
use azure::AzFloat;
use std::libc::types::common::c99::uint16_t;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use geom::side_offsets::SideOffsets2D;
use servo_net::image::base::Image;
use extra::arc::ARC;

pub struct RenderContext<'self> {
    canvas: &'self LayerBuffer,
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
                       color: Color) {
        let pattern = ColorPattern(color);
        let draw_opts = DrawOptions(1 as AzFloat, 0 as uint16_t);
        let stroke_fields = 2; // CAP_SQUARE
        let mut stroke_opts = StrokeOptions(0 as AzFloat, 10 as AzFloat, stroke_fields);

        let rect = bounds.to_azure_rect();
        let border = border.to_float_px();

        self.canvas.draw_target.make_current();

        // draw top border
        stroke_opts.line_width = border.top;
        let y = rect.origin.y + border.top * 0.5;
        let start = Point2D(rect.origin.x, y);
        let end = Point2D(rect.origin.x + rect.size.width, y);
        self.canvas.draw_target.stroke_line(start, end, &pattern, &stroke_opts, &draw_opts);

        // draw bottom border
        stroke_opts.line_width = border.bottom;
        let y = rect.origin.y + rect.size.height - border.bottom * 0.5;
        let start = Point2D(rect.origin.x, y);
        let end = Point2D(rect.origin.x + rect.size.width, y);
        self.canvas.draw_target.stroke_line(start, end, &pattern, &stroke_opts, &draw_opts);

        // draw left border
        stroke_opts.line_width = border.left;
        let x = rect.origin.x + border.left * 0.5;
        let start = Point2D(x, rect.origin.y);
        let end = Point2D(x, rect.origin.y + rect.size.height);
        self.canvas.draw_target.stroke_line(start, end, &pattern, &stroke_opts, &draw_opts);

        // draw right border
        stroke_opts.line_width = border.right;
        let x = rect.origin.x + rect.size.width - border.right * 0.5;
        let start = Point2D(x, rect.origin.y);
        let end = Point2D(x, rect.origin.y + rect.size.height);
        self.canvas.draw_target.stroke_line(start, end, &pattern, &stroke_opts, &draw_opts);
    }

    pub fn draw_image(&self, bounds: Rect<Au>, image: ARC<~Image>) {
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
        Rect(Point2D(self.origin.x.to_px() as AzFloat, self.origin.y.to_px() as AzFloat),
             Size2D(self.size.width.to_px() as AzFloat, self.size.height.to_px() as AzFloat))
    }
}

trait ToSideOffsetsPx {
    fn to_float_px(&self) -> SideOffsets2D<AzFloat>;
}

impl ToSideOffsetsPx for SideOffsets2D<Au> {
    fn to_float_px(&self) -> SideOffsets2D<AzFloat> {
        SideOffsets2D::new(self.top.to_px() as AzFloat,
                           self.right.to_px() as AzFloat,
                           self.bottom.to_px() as AzFloat,
                           self.left.to_px() as AzFloat)
    }
}
