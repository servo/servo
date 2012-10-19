use au = geometry;

use compositor::LayerBuffer;
use text::font::Font;
use text::text_run::TextRun;
use text::font_cache::FontCache;
use image::base::Image;
use au::Au;
use util::range::Range;

use core::libc::types::common::c99::uint16_t;
use core::ptr::to_unsafe_ptr;
use core::dvec::DVec;
use std::arc::ARC;
use geom::size::Size2D;
use geom::point::Point2D;
use geom::rect::Rect;
use azure::bindgen::AzDrawTargetFillGlyphs;
use azure::cairo::{cairo_font_face_t, cairo_scaled_font_t};
use azure::cairo_hl::ImageSurface;
use azure::{AzDrawOptions, AzFloat, AzGlyph, AzGlyphBuffer};
use azure::azure_hl::{AsAzureRect, B8G8R8A8, Color, ColorPattern, DrawOptions, DrawSurfaceOptions, StrokeOptions};
use azure::azure_hl::{DrawTarget, Linear};

struct RenderContext {
    canvas: &LayerBuffer,
    font_cache: @FontCache,
}

impl RenderContext  {
    pub fn draw_solid_color(&self, bounds: &Rect<Au>, r: u8, g: u8, b: u8) {
        let color = Color(r.to_float() as AzFloat,
                          g.to_float() as AzFloat,
                          b.to_float() as AzFloat,
                          1f as AzFloat);

        self.canvas.draw_target.fill_rect(&bounds.to_azure_rect(), &ColorPattern(color));
    }

    pub fn draw_border(&self, bounds: &Rect<Au>, width: Au, r: u8, g: u8, b: u8) {
        let rect = bounds.to_azure_rect();
        let color = Color(r.to_float() as AzFloat,
                          g.to_float() as AzFloat,
                          b.to_float() as AzFloat,
                          1f as AzFloat);
        let pattern = ColorPattern(color);
        let stroke_fields = 2; // CAP_SQUARE
        let stroke_opts = StrokeOptions(au::to_px(width) as AzFloat, 10 as AzFloat, stroke_fields);
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

    pub fn draw_text(&self, bounds: Rect<Au>, run: &TextRun, range: Range) {
        use ptr::{null};
        use vec::raw::to_ptr;
        use libc::types::common::c99::{uint16_t, uint32_t};
        use geom::point::Point2D;
        use text::font_cache::FontCache;
        use text::font::Font;
        use azure::{AzNativeFont, AzFloat, AZ_NATIVE_FONT_CAIRO_FONT_FACE};
        use azure::bindgen::{AzCreateScaledFontWithCairo,
                             AzReleaseScaledFont,
                             AzCreateColorPattern,
                             AzReleaseColorPattern};
        use azure::cairo::bindgen::cairo_scaled_font_destroy;

        let font = run.font;

        let nfont: AzNativeFont = {
            mType: AZ_NATIVE_FONT_CAIRO_FONT_FACE,
            mFont: null()
        };

        let cfont = get_cairo_font(font);
        let azfont = AzCreateScaledFontWithCairo(to_unsafe_ptr(&nfont), 1f as AzFloat, cfont);
        assert azfont.is_not_null();
        cairo_scaled_font_destroy(cfont);

        let color = {
            r: 0f as AzFloat,
            g: 0f as AzFloat,
            b: 0f as AzFloat,
            a: 1f as AzFloat
        };
        let pattern = AzCreateColorPattern(to_unsafe_ptr(&color));
        assert pattern.is_not_null();

        let options: AzDrawOptions = {
            mAlpha: 1f as AzFloat,
            fields: 0 as uint16_t
        };

        let mut origin = Point2D(bounds.origin.x, bounds.origin.y.add(&bounds.size.height));
        let azglyphs = DVec();
        azglyphs.reserve(range.length());

        do run.glyphs.iter_glyphs_for_range(range) |_i, glyph| {
            let glyph_advance = glyph.advance();
            let glyph_offset = glyph.offset().get_default(au::zero_point());

            let azglyph: AzGlyph = {
                mIndex: glyph.index() as uint32_t,
                mPosition: {
                    x: au::to_px(origin.x + glyph_offset.x) as AzFloat,
                    y: au::to_px(origin.y + glyph_offset.y) as AzFloat
                }
            };
            origin = Point2D(origin.x + glyph_advance, origin.y);
            azglyphs.push(move azglyph)
        };

        let azglyph_buf_len = azglyphs.len();
        let azglyph_buf = dvec::unwrap(move azglyphs);
        let glyphbuf: AzGlyphBuffer = unsafe {{
            mGlyphs: to_ptr(azglyph_buf),
            mNumGlyphs: azglyph_buf_len as uint32_t            
        }};

        // TODO: this call needs to move into azure_hl.rs
        AzDrawTargetFillGlyphs(self.canvas.draw_target.azure_draw_target, azfont,
                               to_unsafe_ptr(&glyphbuf), pattern, to_unsafe_ptr(&options), null());

        AzReleaseColorPattern(pattern);
        AzReleaseScaledFont(azfont);
    }

    fn clear(&self) {
        let pattern = ColorPattern(Color(1f as AzFloat, 1f as AzFloat, 1f as AzFloat, 1f as AzFloat));
        let rect = Rect(Point2D(0 as AzFloat, 0 as AzFloat),
                        Size2D(self.canvas.size.width as AzFloat, self.canvas.size.height as AzFloat));
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
}

impl Rect<Au> : ToAzureRect {
    fn to_azure_rect() -> Rect<AzFloat> {
        Rect(Point2D(au::to_px(self.origin.x) as AzFloat, au::to_px(self.origin.y) as AzFloat),
             Size2D(au::to_px(self.size.width) as AzFloat, au::to_px(self.size.height) as AzFloat))
    }
}

#[cfg(target_os = "linux")]
fn get_cairo_face(font: &Font) -> *cairo_font_face_t {

    use libc::c_int;
    use azure::cairo_ft::bindgen::{cairo_ft_font_face_create_for_ft_face};

    let ftface = font.native_font.face;
    let cface = cairo_ft_font_face_create_for_ft_face(ftface, 0 as c_int);
    // FIXME: error handling
    return cface;
}

#[cfg(target_os = "macos")]
fn get_cairo_face(font: &Font) -> *cairo_font_face_t {
    use azure::cairo_quartz::bindgen::cairo_quartz_font_face_create_for_cgfont;

    let cgfont = font.native_font.cgfont;
    let face = cairo_quartz_font_face_create_for_cgfont(cgfont);
    // FIXME: error handling
    return face;
}

fn get_cairo_font(font: &Font) -> *cairo_scaled_font_t {

    use libc::c_double;
    use azure::cairo;
    use cairo::cairo_matrix_t;
    use cairo::bindgen::{cairo_matrix_init_identity,
                         cairo_matrix_scale,
                         cairo_font_options_create,
                         cairo_scaled_font_create,
                         cairo_font_options_destroy,
                         cairo_font_face_destroy};

    // FIXME: error handling

    let face = get_cairo_face(font);

    let idmatrix: cairo_matrix_t = {
        xx: 0 as c_double,
        yx: 0 as c_double,
        xy: 0 as c_double,
        yy: 0 as c_double,
        x0: 0 as c_double,
        y0: 0 as c_double
    };
    cairo_matrix_init_identity(to_unsafe_ptr(&idmatrix));

    let fontmatrix = idmatrix;
    cairo_matrix_scale(to_unsafe_ptr(&fontmatrix), 20f as c_double, 20f as c_double);
    let options = cairo_font_options_create();
    let cfont = cairo_scaled_font_create(face, to_unsafe_ptr(&fontmatrix),
                                         to_unsafe_ptr(&idmatrix), options);
    cairo_font_options_destroy(options);
    cairo_font_face_destroy(face);

    return cfont;
}

