import platform::osmain;
import geom::*;
import comm::*;
import image::base::image;
import dl = layout::display_list;
import azure::*;
import azure::bindgen::*;

enum msg {
    render(dl::display_list),
    exit(comm::chan<()>)
}

#[doc = "
The interface used to by the renderer to aquire draw targets for
each rendered frame and submit them to be drawn to the display
"]
iface sink {
    fn begin_drawing(next_dt: chan<AzDrawTargetRef>);
    fn draw(next_dt: chan<AzDrawTargetRef>, draw_me: AzDrawTargetRef);
}

fn renderer<S: sink send copy>(sink: S) -> chan<msg> {
    task::spawn_listener::<msg> {|po|
        listen {|draw_target_ch|
            #debug("renderer: beginning rendering loop");
            sink.begin_drawing(draw_target_ch);

            loop {
                alt po.recv() {
                  render(display_list) {
                    #debug("renderer: got render request");
                    let draw_target = draw_target_ch.recv();
                    #debug("renderer: rendering");
                    clear(draw_target);
                    draw_display_list(draw_target, display_list);
                    draw_some_text(draw_target);
                    #debug("renderer: returning surface");
                    sink.draw(draw_target_ch, draw_target);
                  }
                  exit(response_ch) {
                    response_ch.send(());
                    break;
                  }
                }
            }
        }
    }
}

impl to_float for u8 {
    fn to_float() -> float {
        (self as float) / 255f
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
    for uint::range(0u, image.height) {
        |y|
        for uint::range(0u, image.width) {
            |x|
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

fn draw_display_list(
    draw_target: AzDrawTargetRef,
    display_list: dl::display_list
) {
    for display_list.each {|item|
        #debug["drawing %?", item];

        alt item.item_type {
            dl::display_item_solid_color(r, g, b) {
                draw_solid_color(draw_target, item, r, g, b);
            }
            dl::display_item_image(image) {
                draw_image(draw_target, item, image);
            }
            dl::padding(*) {
                fail "should never see padding";
            }
        }
    }
}

#[cfg(target_os = "macos")]
fn draw_some_text(_draw_target: AzDrawTargetRef) {
   // FIXME: Don't know how to draw text on mac yet
}

#[cfg(target_os = "linux")]
fn draw_some_text(draw_target: AzDrawTargetRef) {

    import az = azure;
    import azbg = azure::bindgen;
    import ft = azure::freetype;
    import ftbg = azure::freetype::bindgen;
    import cairo = azure::cairo;
    import cairobg = azure::cairo::bindgen;
    import cairoftbg = azure::cairo_ft::bindgen;
    import libc::types::common::c99::uint16_t;
    import libc::types::common::c99::uint32_t;

    impl methods for ft::FT_Error {
        fn for_sure() { assert self == 0 as ft::FT_Error }
    }

    let library: ft::FT_Library = ptr::null();
    ftbg::FT_Init_FreeType(ptr::addr_of(library)).for_sure();

    let fontbin = #include_bin("JosefinSans-SemiBold.ttf");

    let face: ft::FT_Face = ptr::null();
    vec::as_buf(fontbin) {|buf|
        ftbg::FT_New_Memory_Face(library, buf, fontbin.len() as ft::FT_Long,
                                 0, ptr::addr_of(face)).for_sure();
    }

    unsafe {
        #debug("num_glyphs %?", (*face).num_glyphs);
        #debug("family_name %?",
               str::unsafe::from_c_str((*face).family_name));
        #debug("style_name %?",
               str::unsafe::from_c_str((*face).style_name));
    }

    let cface = cairoftbg::cairo_ft_font_face_create_for_ft_face(
        face, 0 as libc::c_int);
    assert cface.is_not_null();

    let fontmatrix: cairo::cairo_matrix_t = {
        xx: 0 as libc::c_double,
        yx: 0 as libc::c_double,
        xy: 0 as libc::c_double,
        yy: 0 as libc::c_double,
        x0: 0 as libc::c_double,
        y0: 0 as libc::c_double
    };
    cairobg::cairo_matrix_init_identity(ptr::addr_of(fontmatrix));
    cairobg::cairo_matrix_scale(ptr::addr_of(fontmatrix), 300f as libc::c_double, 400f as libc::c_double);

    let idmatrix: cairo::cairo_matrix_t = {
        xx: 0 as libc::c_double,
        yx: 0 as libc::c_double,
        xy: 0 as libc::c_double,
        yy: 0 as libc::c_double,
        x0: 0 as libc::c_double,
        y0: 0 as libc::c_double
    };
    cairobg::cairo_matrix_init_identity(ptr::addr_of(idmatrix));

    let options = cairobg::cairo_font_options_create();

    let cfont = cairobg::cairo_scaled_font_create(
        cface, ptr::addr_of(fontmatrix), ptr::addr_of(idmatrix), options);
    assert cfont.is_not_null();
    assert cairobg::cairo_scaled_font_status(cfont) == 0 as cairo::cairo_status_t;
    cairobg::cairo_font_face_destroy(cface);
    cairobg::cairo_font_options_destroy(options);

    let nfont: az::AzNativeFont = {
        mType: az::AZ_NATIVE_FONT_CAIRO_FONT_FACE,
        mFont: ptr::null()
    };

    let azfont = azbg::AzCreateScaledFontWithCairo(
        ptr::addr_of(nfont),
        20f as az::AzFloat,
        cfont);
    assert azfont.is_not_null();
    cairobg::cairo_scaled_font_destroy(cfont);

    let color = {
        r: 1f as AzFloat,
        g: 0f as AzFloat,
        b: 0.5f as AzFloat,
        a: 1f as AzFloat
    };
    let pattern = AzCreateColorPattern(ptr::addr_of(color));
    assert pattern.is_not_null();

    let options: az::AzDrawOptions = {
        mAlpha: 1f as az::AzFloat,
        fields: 0 as uint16_t
    };

    let glyphidx = ftbg::FT_Get_Char_Index(face, 'w' as ft::FT_ULong);

    #debug("glyph: %?", glyphidx);

    for int::range(0, 3) {|i|
        let glyphs: [az::AzGlyph] = [
            {
                mIndex: glyphidx,
                mPosition: {
                    x: (100 + 250 * i) as az::AzFloat,
                    y: 600 as az::AzFloat
                }
            }
        ];

        let glyphbuf: az::AzGlyphBuffer = unsafe {{
            mGlyphs: vec::unsafe::to_ptr(glyphs),
            mNumGlyphs: 1 as uint32_t
        }};

        AzDrawTargetFillGlyphs(draw_target,
                               azfont,
                               ptr::addr_of(glyphbuf),
                               pattern,
                               ptr::addr_of(options),
                               ptr::null());
    }

    azbg::AzReleaseColorPattern(pattern);
    azbg::AzReleaseScaledFont(azfont);
    ftbg::FT_Done_Face(face).for_sure();
    ftbg::FT_Done_FreeType(library).for_sure();
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
