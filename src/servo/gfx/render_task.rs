use mod azure::azure_hl;
use au = geometry;
use au::au;
use platform::osmain;
use comm::*;
use image::base::Image;
use dl = display_list;
use libc::size_t;
use text::font::Font;
use display_list::GlyphRun;
use geom::size::Size2D;
use geom::rect::Rect;
use geom::point::Point2D;
use azure::{AzDrawOptions, AzFloat, AzGlyph, AzGlyphBuffer};
use azure::bindgen::AzDrawTargetFillGlyphs;
use azure_hl::{AsAzureRect, B8G8R8A8, Color, ColorPattern, DrawOptions, DrawSurfaceOptions};
use azure_hl::{DrawTarget, Linear};
use ptr::to_unsafe_ptr;
use std::arc::ARC;
use azure::cairo::{cairo_font_face_t, cairo_scaled_font_t};
use std::cell::Cell;
use compositor::Compositor;
use servo_text::font_cache::FontCache;

use pipes::{Port, Chan};

pub type Renderer = comm::Chan<Msg>;

pub enum Msg {
    RenderMsg(dl::DisplayList),
    ExitMsg(pipes::Chan<()>)
}

struct RenderContext {
    canvas: &DrawTarget,
    font_cache: @FontCache,
}

pub type RenderTask = comm::Chan<Msg>;

pub fn RenderTask<C: Compositor Send>(+compositor: C) -> RenderTask {
    do task::spawn_listener |po: comm::Port<Msg>| {
        let (draw_target_ch, draw_target_po) = pipes::stream();
        let mut draw_target_ch = draw_target_ch;
        let mut draw_target_po = draw_target_po;

        let font_cache = FontCache();

        debug!("renderer: beginning rendering loop");

        compositor.begin_drawing(draw_target_ch);

        loop {
            match po.recv() {
              RenderMsg(display_list) => {
                #debug("renderer: got render request");
                let draw_target = Cell(draw_target_po.recv());
                let (ch, po) = pipes::stream();
                let mut draw_target_ch_ = Some(ch);
                draw_target_po = po;
                #debug("renderer: rendering");
                do util::time::time(~"rendering") {
                    let mut draw_target_ch = None;
                    draw_target_ch_ <-> draw_target_ch;
                    let draw_target_ch = option::unwrap(draw_target_ch);

                    do draw_target.with_ref |draw_target| {
                        let ctx = RenderContext {
                            canvas: draw_target,
                            font_cache: font_cache
                        };

                        clear(&ctx);
                        display_list.draw(&ctx)
                    }

                    #debug("renderer: returning surface");
                    compositor.draw(draw_target_ch, draw_target.take());
                }
              }
              ExitMsg(response_ch) => {
                response_ch.send(());
                break;
              }
            }
        }
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

impl Rect<au> : ToAzureRect {
    fn to_azure_rect() -> Rect<AzFloat> {
        Rect(Point2D(au::to_px(self.origin.x) as AzFloat, au::to_px(self.origin.y) as AzFloat),
             Size2D(au::to_px(self.size.width) as AzFloat, au::to_px(self.size.height) as AzFloat))
    }
}

pub fn draw_solid_color(ctx: &RenderContext, bounds: &Rect<au>, r: u8, g: u8, b: u8) {
    let color = Color(r.to_float() as AzFloat,
                      g.to_float() as AzFloat,
                      b.to_float() as AzFloat,
                      1f as AzFloat);

    ctx.canvas.fill_rect(bounds.to_azure_rect(), ColorPattern(color));
}

pub fn draw_image(ctx: &RenderContext, bounds: Rect<au>, image: ARC<~Image>) {
    let image = std::arc::get(&image);
    let size = Size2D(image.width as i32, image.height as i32);
    let stride = image.width * 4;

    let azure_surface = ctx.canvas.create_source_surface_from_data(image.data, size, stride as i32,
                                                                    B8G8R8A8);
    let source_rect = Rect(Point2D(0 as AzFloat, 0 as AzFloat),
                           Size2D(image.width as AzFloat, image.height as AzFloat));
    let dest_rect = bounds.to_azure_rect();
    let draw_surface_options = DrawSurfaceOptions(Linear, true);
    let draw_options = DrawOptions(1.0f as AzFloat, 0);
    ctx.canvas.draw_surface(azure_surface, dest_rect, source_rect, draw_surface_options,
                             draw_options);
}

pub fn draw_glyphs(ctx: &RenderContext, bounds: Rect<au>, text_run: &GlyphRun) {
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

    // FIXME: font should be accessible from GlyphRun
    let font = ctx.font_cache.get_test_font();

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
    let azglyphs = text_run.glyphs.map(|glyph| {
        let azglyph: AzGlyph = {
            mIndex: glyph.index as uint32_t,
            mPosition: {
                x: au::to_px(origin.x.add(&glyph.pos.offset.x)) as AzFloat,
                y: au::to_px(origin.y.add(&glyph.pos.offset.y)) as AzFloat
            }
        };
        origin = Point2D(origin.x.add(&glyph.pos.advance.x),
                         origin.y.add(&glyph.pos.advance.y));
        azglyph
    });

    let glyphbuf: AzGlyphBuffer = unsafe {{
        mGlyphs: to_ptr(azglyphs),
        mNumGlyphs: azglyphs.len() as uint32_t            
    }};

    // TODO: this call needs to move into azure_hl.rs
    AzDrawTargetFillGlyphs(ctx.canvas.azure_draw_target, azfont, to_unsafe_ptr(&glyphbuf),
                           pattern, to_unsafe_ptr(&options), null());

    AzReleaseColorPattern(pattern);
    AzReleaseScaledFont(azfont);
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

fn clear(ctx: &RenderContext) {
    let pattern = ColorPattern(Color(1f as AzFloat, 1f as AzFloat, 1f as AzFloat, 1f as AzFloat));
    let rect = Rect(Point2D(0 as AzFloat, 0 as AzFloat), Size2D(800 as AzFloat, 600 as AzFloat));
    ctx.canvas.fill_rect(rect, pattern);
}
