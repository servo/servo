import platform::osmain;
import geometry::*;
import comm::*;
import image::base::Image;
import dl = layout::display_list;
import azure::*;
import azure::bindgen::*;
import libc::size_t;
import text::font::Font;
import text::text_run::TextRun;
import dom::event::{Event, ResizeEvent};
import geom::size::Size2D;
import geom::rect::Rect;
import geom::point::Point2D;
import azure_hl::{AsAzureRect, Color, ColorPattern, DrawTarget};
import ptr::addr_of;
import std::arc::arc;
import azure::cairo::{cairo_font_face_t, cairo_scaled_font_t};

import pipes::{port, chan};

type Renderer = comm::Chan<Msg>;

enum Msg {
    RenderMsg(dl::display_list),
    ExitMsg(pipes::chan<()>)
}

#[doc = "
The interface used to by the renderer to aquire draw targets for
each rendered frame and submit them to be drawn to the display

FIXME: Change this name to Compositor.
"]
trait Sink {
    fn begin_drawing(+next_dt: pipes::chan<AzDrawTargetRef>);
    fn draw(+next_dt: pipes::chan<AzDrawTargetRef>, draw_me: AzDrawTargetRef);
    fn add_event_listener(listener: comm::Chan<Event>);
}

fn Renderer<S: Sink send copy>(sink: S) -> comm::Chan<Msg> {
    do task::spawn_listener |po: comm::Port<Msg>| {
        let (draw_target_ch, draw_target_po) = pipes::stream();
        let mut draw_target_ch = draw_target_ch;
        let mut draw_target_po = draw_target_po;

        debug!("renderer: beginning rendering loop");

        sink.begin_drawing(draw_target_ch);

        loop {
            match po.recv() {
              RenderMsg(display_list) => {
                #debug("renderer: got render request");
                let draw_target = draw_target_po.recv();
                let (ch, po) = pipes::stream();
                let mut draw_target_ch_ = some(ch);
                draw_target_po = po;
                #debug("renderer: rendering");
                do util::time::time(~"rendering") {
                    let mut draw_target_ch = none;
                    draw_target_ch_ <-> draw_target_ch;
                    let draw_target_ch = option::unwrap(draw_target_ch);

                    clear(draw_target);
                    draw_display_list(draw_target, display_list);

                    #debug("renderer: returning surface");
                    sink.draw(draw_target_ch, draw_target);
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

fn draw_display_list(azure_draw_target: AzDrawTargetRef, display_list: dl::display_list) {
    let draw_target = azure_hl::new_draw_target_from_azure_draw_target(azure_draw_target);
    for display_list.each |item| {
        #debug["drawing %?", item];

        match item.item_type {
          dl::display_item_solid_color(r, g, b) => draw_solid_color(&draw_target, item, r, g, b),
          dl::display_item_image(image) => draw_image(&draw_target, item, *image),
          dl::display_item_text(text_run) => draw_text(&draw_target, item, text_run),
          dl::padding(*) => fail ~"should never see padding"
        }
    }
}

fn draw_solid_color(draw_target: &DrawTarget, item: dl::display_item, r: u8, g: u8, b: u8) {
    let color = Color(r.to_float() as AzFloat,
                      g.to_float() as AzFloat,
                      b.to_float() as AzFloat,
                      1f as AzFloat);

    let pattern = ColorPattern(color);

    let rect = Rect(Point2D(au_to_px(item.bounds.origin.x) as AzFloat,
                            au_to_px(item.bounds.origin.y) as AzFloat),
                    Size2D(au_to_px(item.bounds.size.width) as AzFloat,
                           au_to_px(item.bounds.size.height) as AzFloat));

    draw_target.fill_rect(rect, pattern);
}

fn draw_image(draw_target: &DrawTarget, item: dl::display_item, image: arc<~Image>) unsafe {
    let draw_target = draw_target.azure_draw_target;

    let image = std::arc::get(&image);
    let size = Size2D(image.width as i32, image.height as i32);
    let stride = image.width * 4;

    // Do color space conversion :(
    let data = do vec::from_fn(image.width * image.height * 4) |i| {
        let color = i % 4;
        let pixel = i / 4;
        match color {
            0 => image.data[pixel * 3 + 2],
            1 => image.data[pixel * 3 + 1],
            2 => image.data[pixel * 3 + 0],
            3 => 0xffu8,
            _ => fail
        }
    };

    let azure_surface =
        AzDrawTargetCreateSourceSurfaceFromData(draw_target,
                                                addr_of(data[0]),
                                                unsafe::reinterpret_cast(addr_of(size)),
                                                stride as i32,
                                                0u32);
    let source_rect = Rect(Point2D(0.0f as AzFloat, 0.0f as AzFloat),
                           Size2D(image.width as AzFloat, image.height as AzFloat));
    let dest_rect = Rect(Point2D(au_to_px(item.bounds.origin.x) as AzFloat,
                                 au_to_px(item.bounds.origin.y) as AzFloat),
                         Size2D(au_to_px(item.bounds.size.width) as AzFloat,
                                au_to_px(item.bounds.size.height) as AzFloat));
    let draw_surface_options = azure_hl::DrawSurfaceOptions(azure_hl::Linear, true);
    let draw_options = azure_hl::DrawOptions(1.0f as AzFloat, 0);
    AzDrawTargetDrawSurface(draw_target,
                            azure_surface,
                            addr_of(dest_rect.as_azure_rect()),
                            addr_of(source_rect.as_azure_rect()),
                            addr_of(draw_surface_options.as_azure_draw_surface_options()),
                            addr_of(draw_options.as_azure_draw_options()));
    AzReleaseSourceSurface(azure_surface);
}

fn draw_text(draw_target: &DrawTarget, item: dl::display_item, text_run: TextRun) {
    import ptr::{addr_of, null};
    import vec::unsafe::to_ptr;
    import libc::types::common::c99::{uint16_t, uint32_t};
    import geom::point::Point2D;
    import text::font_library::FontLibrary;
    import text::font::Font;
    import azure::{AzNativeFont, AzFloat, AZ_NATIVE_FONT_CAIRO_FONT_FACE};
    import azure::bindgen::{AzCreateScaledFontWithCairo,
                            AzReleaseScaledFont,
                            AzCreateColorPattern,
                            AzReleaseColorPattern};
    import azure::cairo::bindgen::cairo_scaled_font_destroy;

    let draw_target = draw_target.azure_draw_target;

    let bounds = copy (*item).bounds;
    // FIXME: The font library should not be created here
    let flib = FontLibrary();
    let font = flib.get_test_font();

    let nfont: AzNativeFont = {
        mType: AZ_NATIVE_FONT_CAIRO_FONT_FACE,
        mFont: null()
    };

    let cfont = get_cairo_font(font);
    let azfont = AzCreateScaledFontWithCairo(addr_of(nfont), 1f as AzFloat, cfont);
    assert azfont.is_not_null();
    cairo_scaled_font_destroy(cfont);

    let color = {
        r: 0f as AzFloat,
        g: 0f as AzFloat,
        b: 0f as AzFloat,
        a: 1f as AzFloat
    };
    let pattern = AzCreateColorPattern(addr_of(color));
    assert pattern.is_not_null();

    let options: AzDrawOptions = {
        mAlpha: 1f as AzFloat,
        fields: 0 as uint16_t
    };

    let mut origin = Point2D(bounds.origin.x, bounds.origin.y.add(bounds.size.height));
    let azglyphs = text_run.glyphs.map(|glyph| {
        let azglyph: AzGlyph = {
            mIndex: glyph.index as uint32_t,
            mPosition: {
                x: au_to_px(origin.x.add(glyph.pos.offset.x)) as AzFloat,
                y: au_to_px(origin.y.add(glyph.pos.offset.y)) as AzFloat
            }
        };
        origin = Point2D(origin.x.add(glyph.pos.advance.x),
                         origin.y.add(glyph.pos.advance.y));
        azglyph
    });

    let glyphbuf: AzGlyphBuffer = unsafe {{
        mGlyphs: to_ptr(azglyphs),
        mNumGlyphs: azglyphs.len() as uint32_t            
    }};

    AzDrawTargetFillGlyphs(draw_target, azfont, addr_of(glyphbuf),
                           pattern, addr_of(options), null());

    AzReleaseColorPattern(pattern);
    AzReleaseScaledFont(azfont);
}

#[cfg(target_os = "linux")]
fn get_cairo_face(font: &Font) -> *cairo_font_face_t {

    import libc::c_int;
    import azure::cairo_ft::bindgen::{cairo_ft_font_face_create_for_ft_face};

    let ftface = font.native_font.face;
    let cface = cairo_ft_font_face_create_for_ft_face(ftface, 0 as c_int);
    // FIXME: error handling
    return cface;
}

#[cfg(target_os = "macos")]
fn get_cairo_face(font: &Font) -> *cairo_font_face_t {
    import azure::cairo_quartz::bindgen::cairo_quartz_font_face_create_for_cgfont;

    let cgfont = font.native_font.cgfont;
    let face = cairo_quartz_font_face_create_for_cgfont(cgfont);
    // FIXME: error handling
    return face;
}

fn get_cairo_font(font: &Font) -> *cairo_scaled_font_t {

    import libc::c_double;
    import azure::cairo;
    import cairo::cairo_matrix_t;
    import cairo::bindgen::{cairo_matrix_init_identity,
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
    cairo_matrix_init_identity(addr_of(idmatrix));

    let fontmatrix = idmatrix;
    cairo_matrix_scale(addr_of(fontmatrix),
                       20f as c_double, 20f as c_double);
    let options = cairo_font_options_create();
    let cfont = cairo_scaled_font_create(face, addr_of(fontmatrix),
                                         addr_of(idmatrix), options);
    cairo_font_options_destroy(options);
    cairo_font_face_destroy(face);

    return cfont;
}

fn clear(draw_target: AzDrawTargetRef) {

    let black_color = {
        r: 0f as AzFloat,
        g: 0f as AzFloat,
        b: 0f as AzFloat,
        a: 1f as AzFloat
    };
    let black_pattern = AzCreateColorPattern(ptr::addr_of(black_color));

    let black_rect = {
        x: 0 as AzFloat,
        y: 0 as AzFloat,
        width: 800 as AzFloat,
        height: 600 as AzFloat,
    };

    AzDrawTargetFillRect(
        draw_target,
        ptr::addr_of(black_rect),
        unsafe { unsafe::reinterpret_cast(black_pattern) }
    );

    AzReleaseColorPattern(black_pattern);
}
