import platform::osmain;
import geometry::*;
import comm::*;
import image::base::image;
import dl = layout::display_list;
import azure::*;
import azure::bindgen::*;
import libc::size_t;
import text::text_run::TextRun;
import dom::event::{Event, ResizeEvent};

type Renderer = chan<Msg>;

enum Msg {
    RenderMsg(dl::display_list),
    ExitMsg(comm::chan<()>)
}

#[doc = "
The interface used to by the renderer to aquire draw targets for
each rendered frame and submit them to be drawn to the display

FIXME: Change this name to Compositor.
"]
iface Sink {
    fn begin_drawing(next_dt: chan<AzDrawTargetRef>);
    fn draw(next_dt: chan<AzDrawTargetRef>, draw_me: AzDrawTargetRef);
    fn add_event_listener(listener: chan<Event>);
}

fn Renderer<S: Sink send copy>(sink: S) -> chan<Msg> {
    task::spawn_listener::<Msg>(|po| {
        listen(|draw_target_ch| {
            #debug("renderer: beginning rendering loop");
            sink.begin_drawing(draw_target_ch);

            loop {
                alt po.recv() {
                  RenderMsg(display_list) {
                    #debug("renderer: got render request");
                    let draw_target = draw_target_ch.recv();
                    #debug("renderer: rendering");

                    do util::time::time(~"rendering") {
                        clear(draw_target);
                        draw_display_list(draw_target, display_list);
                        #debug("renderer: returning surface");
                        sink.draw(draw_target_ch, draw_target);
                    }
                  }
                  ExitMsg(response_ch) {
                    response_ch.send(());
                    break;
                  }
                }
            }
        })
    })
}

impl to_float for u8 {
    fn to_float() -> float {
        (self as float) / 255f
    }
}

fn draw_display_list(draw_target: AzDrawTargetRef, display_list: dl::display_list) {
    for display_list.each |item| {
        #debug["drawing %?", item];

        alt item.item_type {
          dl::display_item_solid_color(r, g, b) {
            draw_solid_color(draw_target, item, r, g, b);
          }
          dl::display_item_image(image) {
            draw_image(draw_target, item, copy image);
          }
          dl::display_item_text(text_run) {
            draw_text(draw_target, item, text_run);
          }
          dl::padding(*) {
            fail ~"should never see padding";
          }
        }
    }
}

fn draw_solid_color(draw_target: AzDrawTargetRef, item: dl::display_item,
                    r: u8, g: u8, b: u8) {
    let bounds = copy (*item).bounds;

    let red_color = {
        r: r.to_float() as AzFloat,
        g: g.to_float() as AzFloat,
        b: b.to_float() as AzFloat,
        a: 1f as AzFloat
    };
    let red_pattern = AzCreateColorPattern(ptr::addr_of(red_color));

    let red_rect = {
        x: au_to_px(bounds.origin.x) as AzFloat,
        y: au_to_px(bounds.origin.y) as AzFloat,
        width: au_to_px(bounds.size.width) as AzFloat,
        height: au_to_px(bounds.size.height) as AzFloat
    };
    AzDrawTargetFillRect(
        draw_target,
        ptr::addr_of(red_rect),
        unsafe { unsafe::reinterpret_cast(red_pattern) }
    );

    AzReleaseColorPattern(red_pattern);
}

fn draw_image(draw_target: AzDrawTargetRef, item: dl::display_item,
              -image: ~image) {
    // FIXME: This is hideously inefficient.

    let bounds = copy (*item).bounds;

    if (image.depth < 3u) {
        #debug("TODO: can't draw images with depth less than 3 yet");
        ret;
    }

    let stride = image.width * image.depth;
    for uint::range(0u, image.height) |y| {
        for uint::range(0u, image.width) |x| {
            let color = {
                r: image.data[y * stride + x * image.depth].to_float()
                    as AzFloat,
                g: image.data[y * stride + x * image.depth + 1u].to_float()
                    as AzFloat,
                b: image.data[y * stride + x * image.depth + 2u].to_float()
                    as AzFloat,
                a: 1f as AzFloat
            };
            let pattern = AzCreateColorPattern(ptr::addr_of(color));

            let pixel_rect = {
                x: (au_to_px(bounds.origin.x) + (x as int)) as AzFloat,
                y: (au_to_px(bounds.origin.y) + (y as int)) as AzFloat,
                width: 1f as AzFloat,
                height: 1f as AzFloat
            };
            AzDrawTargetFillRect(
                draw_target,
                ptr::addr_of(pixel_rect),
                unsafe { unsafe::reinterpret_cast(pattern) }
            );

            AzReleaseColorPattern(pattern);
        }
    }
}

fn draw_text(draw_target: AzDrawTargetRef, item: dl::display_item, text_run: TextRun) {

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

    let bounds = copy (*item).bounds;
    // FIXME: The font library should not be created here
    let flib = FontLibrary();
    let font = flib.get_test_font();

    let nfont: AzNativeFont = {
        mType: AZ_NATIVE_FONT_CAIRO_FONT_FACE,
        mFont: null()
    };

    let azfont = AzCreateScaledFontWithCairo(addr_of(nfont), 1f as AzFloat, font.cairo_font);
    assert azfont.is_not_null();

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
