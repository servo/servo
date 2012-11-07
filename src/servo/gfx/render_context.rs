use compositor::LayerBuffer;
use gfx::au;
use gfx::{
    Au,
    Font,
    FontContext,
    TextRun,
};
use image::base::Image;
use util::range::Range;

use cairo::cairo_hl::ImageSurface;
use core::libc::types::common::c99::uint16_t;
use core::ptr::to_unsafe_ptr;
use core::dvec::DVec;
use std::arc::ARC;
use geom::size::Size2D;
use geom::point::Point2D;
use geom::rect::Rect;
use azure::{AzDrawOptions, AzFloat};
use azure::azure_hl::{AsAzureRect, B8G8R8A8, Color, ColorPattern, DrawOptions, DrawSurfaceOptions, StrokeOptions};
use azure::azure_hl::{DrawTarget, Linear};

struct RenderContext {
    canvas: &LayerBuffer,
    font_ctx: @FontContext,
}

impl RenderContext  {
    pub fn get_draw_target(&self) -> &self/DrawTarget {
        &self.canvas.draw_target
    }

    pub fn draw_solid_color(&self, bounds: &Rect<Au>, color: Color) {
        self.canvas.draw_target.fill_rect(&bounds.to_azure_rect(), &ColorPattern(color));
    }

    pub fn draw_border(&self, bounds: &Rect<Au>, width: Au, color: Color) {
        let pattern = ColorPattern(color);
        let stroke_fields = 2; // CAP_SQUARE
        let width_px = au::to_px(width);
        let rect = if width_px % 2 == 0 {
            bounds.to_azure_rect()
        } else {
            bounds.to_azure_snapped_rect()
        };
            
        let stroke_opts = StrokeOptions(width_px as AzFloat, 10 as AzFloat, stroke_fields);
        let draw_opts = DrawOptions(1 as AzFloat, 0 as uint16_t);

        self.canvas.draw_target.stroke_rect(&rect, &pattern, &stroke_opts, &draw_opts);
    }

    pub fn draw_image(&self, bounds: Rect<Au>, image: ARC<~Image>) {
        let image = std::arc::get(&image);
        let size = Size2D(image.width as i32, image.height as i32);
        let stride = image.width * 4;

        let draw_target_ref = &self.canvas.draw_target;
        let azure_surface = draw_target_ref.create_source_surface_from_data(image.data, size,
                                                                            stride as i32, B8G8R8A8);
        let source_rect = Rect(Point2D(0 as AzFloat, 0 as AzFloat),
                               Size2D(image.width as AzFloat, image.height as AzFloat));
        let dest_rect = bounds.to_azure_rect();
        let draw_surface_options = DrawSurfaceOptions(Linear, true);
        let draw_options = DrawOptions(1.0f as AzFloat, 0);
        draw_target_ref.draw_surface(move azure_surface, dest_rect, source_rect,
                                     draw_surface_options, draw_options);
    }

    fn clear(&self) {
        let pattern = ColorPattern(Color(1f as AzFloat, 1f as AzFloat, 1f as AzFloat, 1f as AzFloat));
        let rect = Rect(Point2D(self.canvas.rect.origin.x as AzFloat,
                                self.canvas.rect.origin.y as AzFloat),
                        Size2D(self.canvas.rect.size.width as AzFloat,
                               self.canvas.rect.size.height as AzFloat));
        self.canvas.draw_target.fill_rect(&rect, &pattern);
    }
}

trait to_float {
    fn to_float() -> float;
}

impl u8 : to_float {
    fn to_float() -> float {
        (self as float) / 255f
    }
}

trait ToAzureRect {
    fn to_azure_rect() -> Rect<AzFloat>;
    fn to_azure_snapped_rect() -> Rect<AzFloat>;
}

impl Rect<Au> : ToAzureRect {
    fn to_azure_rect() -> Rect<AzFloat> {
        Rect(Point2D(au::to_px(self.origin.x) as AzFloat, au::to_px(self.origin.y) as AzFloat),
             Size2D(au::to_px(self.size.width) as AzFloat, au::to_px(self.size.height) as AzFloat))
    }

    fn to_azure_snapped_rect() -> Rect<AzFloat> {
        Rect(Point2D(au::to_px(self.origin.x) as AzFloat + 0.5f as AzFloat, au::to_px(self.origin.y) as AzFloat + 0.5f as AzFloat),
             Size2D(au::to_px(self.size.width) as AzFloat, au::to_px(self.size.height) as AzFloat))
    }
}
