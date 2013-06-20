/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use servo_msg::compositor::LayerBuffer;
use font_context::FontContext;
use geometry::Au;
use opts::Opts;

use azure::azure_hl::{B8G8R8A8, Color, ColorPattern, DrawOptions};
use azure::azure_hl::{DrawSurfaceOptions, DrawTarget, Linear, StrokeOptions};
use azure::AzFloat;
use core::libc::types::common::c99::uint16_t;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use servo_net::image::base::Image;
use std::arc;
use std::arc::ARC;

pub struct RenderContext<'self> {
    canvas: &'self LayerBuffer,
    font_ctx: @mut FontContext,
    opts: &'self Opts
}

pub impl<'self> RenderContext<'self>  {
    pub fn get_draw_target(&self) -> &'self DrawTarget {
        &self.canvas.draw_target
    }

    pub fn draw_solid_color(&self, bounds: &Rect<Au>, color: Color) {
        self.canvas.draw_target.make_current();
        self.canvas.draw_target.fill_rect(&bounds.to_azure_rect(), &ColorPattern(color));
    }

    pub fn draw_border(&self, bounds: &Rect<Au>, width: Au, color: Color) {
        let pattern = ColorPattern(color);
        let stroke_fields = 2; // CAP_SQUARE
        let width_px = width.to_px();
        let rect = if width_px % 2 == 0 {
            bounds.to_azure_rect()
        } else {
            bounds.to_azure_snapped_rect()
        };
            
        let stroke_opts = StrokeOptions(width_px as AzFloat, 10 as AzFloat, stroke_fields);
        let draw_opts = DrawOptions(1 as AzFloat, 0 as uint16_t);

        self.canvas.draw_target.make_current();
        self.canvas.draw_target.stroke_rect(&rect, &pattern, &stroke_opts, &draw_opts);
    }

    pub fn draw_image(&self, bounds: Rect<Au>, image: ARC<~Image>) {
        let image = arc::get(&image);
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

    fn clear(&self) {
        let pattern = ColorPattern(Color(1.0, 1.0, 1.0, 1.0));
        let rect = Rect(Point2D(self.canvas.rect.origin.x as AzFloat,
                                self.canvas.rect.origin.y as AzFloat),
                        Size2D(self.canvas.rect.size.width as AzFloat,
                               self.canvas.rect.size.height as AzFloat));
        self.canvas.draw_target.make_current();
        self.canvas.draw_target.fill_rect(&rect, &pattern);
    }
}

trait to_float {
    fn to_float(self) -> float;
}

impl to_float for u8 {
    fn to_float(self) -> float {
        (self as float) / 255f
    }
}

trait ToAzureRect {
    fn to_azure_rect(&self) -> Rect<AzFloat>;
    fn to_azure_snapped_rect(&self) -> Rect<AzFloat>;
}

impl ToAzureRect for Rect<Au> {
    fn to_azure_rect(&self) -> Rect<AzFloat> {
        Rect(Point2D(self.origin.x.to_px() as AzFloat, self.origin.y.to_px() as AzFloat),
             Size2D(self.size.width.to_px() as AzFloat, self.size.height.to_px() as AzFloat))
    }

    fn to_azure_snapped_rect(&self) -> Rect<AzFloat> {
        Rect(Point2D(self.origin.x.to_px() as AzFloat + 0.5f as AzFloat,
					 self.origin.y.to_px() as AzFloat + 0.5f as AzFloat),
             Size2D(self.size.width.to_px() as AzFloat,
			 		self.size.height.to_px() as AzFloat))
    }
}
