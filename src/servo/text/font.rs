export font, create_test_font;

import libc::{ c_int, c_double };
import ptr::{ null, addr_of };
import azure::cairo::{
    cairo_font_face_t,
    cairo_scaled_font_t,
    cairo_glyph_t,
    CAIRO_STATUS_SUCCESS,
};
import azure::cairo::bindgen::{
    cairo_font_face_destroy,
    cairo_scaled_font_destroy,
    cairo_scaled_font_text_to_glyphs,
    cairo_glyph_free,
};

#[doc = "
A font handle. Layout can use this to calculate glyph metrics
and the renderer can use it to render text.
"]
class font/& {
    let fontbuf: [u8];
    let cairo_font: *cairo_scaled_font_t;
    let font_dtor: fn@();

    new(+fontbuf: [u8]) {

        let (cairo_font, font_dtor) = get_cairo_font(&fontbuf);
        assert cairo_font.is_not_null();

        self.fontbuf = fontbuf;
        self.cairo_font = cairo_font;
        self.font_dtor = font_dtor;
    }

    drop {
        self.font_dtor();
    }

    fn buf() -> &self.[u8] {
        &self.fontbuf
    }

    fn glyph_idx(codepoint: char) -> option<uint> {
        #debug("getting glyph for codepoint %u", codepoint as uint);
        let codepoint_str = str::from_char(codepoint);

        let mut glyphs: *cairo_glyph_t = null();
        let mut num_glyphs = 0 as c_int;

        let status = str::as_c_str(codepoint_str) { |codepoint_buf|
            cairo_scaled_font_text_to_glyphs(
                self.cairo_font,
                0.0 as c_double, 0.0 as c_double,
                codepoint_buf, codepoint_str.len() as c_int,
                addr_of(glyphs), addr_of(num_glyphs),
                null(), null(), null()
            )
        };

        ret if status == CAIRO_STATUS_SUCCESS {

            // This might not be true, but at least we'll know if it isn't
            assert num_glyphs == 1 as c_int;

            let glyph_index = unsafe { *glyphs }.index as uint;
            #debug("glyph index is %?", glyph_index);
            cairo_glyph_free(glyphs);
            some(glyph_index)
        } else {
            #error("cairo did not give me a glyph for %u", codepoint as uint);
            none
        }
    }

    fn glyph_advance(_glyph: uint) -> (int, int) {
        (10, 10)
    }
}

fn get_cairo_font(buf: &[u8]) -> (*cairo_scaled_font_t, fn@()) {

    import libc::c_double;
    import azure::cairo;
    import cairo::{ cairo_matrix_t };
    import cairo::bindgen::{
        cairo_matrix_init_identity,
        cairo_matrix_scale,
        cairo_font_options_create,
        cairo_font_options_destroy,
        cairo_scaled_font_create,
        cairo_scaled_font_destroy
    };
        
    import cairo::bindgen::cairo_scaled_font_create;

    let mut (face, dtor) = get_cairo_face(buf);

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
                       300f as c_double, 400f as c_double);

    let options = cairo_font_options_create();
    let cfont = cairo_scaled_font_create(face, addr_of(fontmatrix),
                                         addr_of(idmatrix), options);
    cairo_font_options_destroy(options);

    // FIXME: Need negative tests
    if cfont.is_null() {
        dtor();
        fail "unable to create cairo scaled font";
    }
    dtor = fn@(move dtor) { cairo_scaled_font_destroy(cfont); dtor() };

    (cfont, dtor)
}

#[cfg(target_os = "linux")]
fn get_cairo_face(buf: &[u8]) -> (*cairo_font_face_t, fn@()) {
    import freetype = azure::freetype;
    import freetype::{ FT_Error, FT_Library, FT_Face, FT_Long };
    import freetype::bindgen::{
        FT_Init_FreeType,
        FT_Done_FreeType,
        FT_New_Memory_Face,
        FT_Done_Face
    };
    import azure::cairo_ft;
    import cairo_ft::bindgen::cairo_ft_font_face_create_for_ft_face;

    impl methods for FT_Error {
        fn for_sure() { assert !self.failed() }
        fn failed() -> bool { self != 0 as FT_Error }
    }

    let mut dtor = fn@() { };

    let library: FT_Library = null();
    // FIXME: Need tests for failure case
    FT_Init_FreeType(addr_of(library)).for_sure();
    dtor = fn@(move dtor) { FT_Done_FreeType(library).for_sure(); dtor() };

    let face: FT_Face = null();
    vec::as_buf(*buf) { |cbuf|
        if FT_New_Memory_Face(library, cbuf, (*buf).len() as FT_Long,
                              0 as FT_Long, addr_of(face)).failed() {
            dtor();
            fail "unable to create FreeType face";
        }
    }
    dtor = fn@(move dtor) { FT_Done_Face(face).for_sure(); dtor() };

    let cface = cairo_ft_font_face_create_for_ft_face(face, 0 as c_int);
    if cface.is_null() {
        // FIXME: Need tests for failure case
        dtor();
        fail "unable to create cairo font face";
    }
    dtor = fn@(move dtor) { cairo_font_face_destroy(cface); dtor() };

    (cface, dtor)
}

#[cfg(target_os = "macos")]
fn get_cairo_face(buf: &[u8]) -> (*cairo_font_face_t, fn@()) {
    fail
}

fn create_test_font() -> font {
    let buf = #include_bin("JosefinSans-SemiBold.ttf");
    font(buf)
}

fn should_destruct_on_fail_without_leaking() {
    #[test];
    #[should_fail];

    let _font = create_test_font();
    fail;
}

fn should_get_glyph_indexes() {
    #[test];

    let font = create_test_font();
    let glyph_idx = font.glyph_idx('w');
    assert glyph_idx == some(40u);
}

fn should_get_glyph_advance() {
    #[test];

    let font = create_test_font();
    let (x, y) = font.glyph_advance(40u);
    // These are bogus numbers
    assert x == 10 && y == 10;
}

fn should_be_able_to_create_instances_in_multiple_threads() {
    #[test];

    iter::repeat(10u) {||
        task::spawn {||
            create_test_font();
        }
    }
}

fn get_cairo_face_should_fail_and_not_leak_if_font_cant_be_created() {
    #[test];
    #[should_fail];

    get_cairo_face(&[0u8, 1u8, 2u8, 3u8]);
}

fn get_cairo_face_should_return_a_new_face_and_dtor() {
    #[test];

    let buf = #include_bin("JosefinSans-SemiBold.ttf");
    let (face, dtor) = get_cairo_face(&buf);
    assert face.is_not_null();
    dtor();
}

fn get_cairo_font_should_return_a_new_font_and_dtor() {
    #[test];

    let buf = #include_bin("JosefinSans-SemiBold.ttf");
    let (font, dtor) = get_cairo_font(&buf);
    assert font.is_not_null();
    dtor();
}
