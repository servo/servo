use harfbuzz;

export shape_text;

import libc::types::common::c99::int32_t;
import libc::{c_uint, c_int, c_void};
import font::Font;
import glyph::{Glyph, GlyphPos};
import ptr::{null, addr_of, offset};
import gfx::geometry::{au, px_to_au};
import geom::point::Point2D;

import unsafe::reinterpret_cast;
import harfbuzz::{HB_MEMORY_MODE_READONLY,
                  HB_DIRECTION_LTR};
import harfbuzz::{hb_blob_t, hb_face_t, hb_font_t, hb_buffer_t,
                  hb_codepoint_t, hb_bool_t, hb_glyph_position_t,
		  hb_var_int_t, hb_position_t};
import harfbuzz::bindgen::{hb_blob_create, hb_blob_destroy,
                           hb_face_create, hb_face_destroy,
                           hb_font_create, hb_font_destroy,
                           hb_buffer_create, hb_buffer_destroy,
                           hb_buffer_add_utf8, hb_shape,
                           hb_buffer_get_glyph_infos,
                           hb_buffer_get_glyph_positions,
                           hb_font_set_ppem, hb_font_set_scale,
                           hb_buffer_set_direction,
                           hb_font_funcs_create, hb_font_funcs_destroy,
                           hb_font_set_funcs,
                           hb_font_funcs_set_glyph_h_advance_func,
                           hb_font_funcs_set_glyph_func,
                           hb_font_funcs_set_glyph_h_kerning_func};

#[doc = "
Calculate the layout metrics associated with a some given text
when rendered in a specific font.
"]
fn shape_text(font: &Font, text: ~str) -> ~[Glyph] unsafe {
    #debug("shaping text '%s'", text);

    let face_blob = vec::as_buf(*(*font).buf(), |buf, len| {
        hb_blob_create(reinterpret_cast(buf),
                       len as c_uint,
                       HB_MEMORY_MODE_READONLY,
                       null(),
                       null())
    });

    let hbface = hb_face_create(face_blob, 0 as c_uint);
    let hbfont = hb_font_create(hbface);

    hb_font_set_ppem(hbfont, 10 as c_uint, 10 as c_uint);
    hb_font_set_scale(hbfont, 10 as c_int, 10 as c_int);

    let funcs = hb_font_funcs_create();
    hb_font_funcs_set_glyph_func(funcs, glyph_func, null(), null());
    hb_font_funcs_set_glyph_h_advance_func(funcs, glyph_h_advance_func, null(), null());
    hb_font_set_funcs(hbfont, funcs, reinterpret_cast(addr_of(*font)), null());

    let buffer = hb_buffer_create();

    hb_buffer_set_direction(buffer, HB_DIRECTION_LTR);

    str::as_c_str(text, |ctext| {
        hb_buffer_add_utf8(buffer, ctext,
                           text.len() as c_int,
                           0 as c_uint,
                           text.len() as c_int);
    });

    hb_shape(hbfont, buffer, null(), 0 as c_uint);

    let info_len = 0 as c_uint;
    let info_ = hb_buffer_get_glyph_infos(buffer, addr_of(info_len));
    assert info_.is_not_null();
    let pos_len = 0 as c_uint;
    let pos = hb_buffer_get_glyph_positions(buffer, addr_of(pos_len));
    assert pos.is_not_null();

    assert info_len == pos_len;

    let mut glyphs = ~[];

    for uint::range(0u, info_len as uint) |i| {
        let info_ = offset(info_, i);
        let pos = offset(pos, i);
        let codepoint = (*info_).codepoint as uint;
        let pos = hb_glyph_pos_to_servo_glyph_pos(&*pos);
        #debug("glyph %?: codep %?, x_adv %?, y_adv %?, x_off %?, y_of %?",
               i, codepoint, pos.advance.x, pos.advance.y, pos.offset.x, pos.offset.y);

        glyphs += ~[Glyph(codepoint, pos)];
    }

    hb_buffer_destroy(buffer);
    hb_font_funcs_destroy(funcs);
    hb_font_destroy(hbfont);
    hb_face_destroy(hbface);
    hb_blob_destroy(face_blob);

    return glyphs;
}

extern fn glyph_func(_font: *hb_font_t,
                     font_data: *c_void,
                     unicode: hb_codepoint_t,
                     _variant_selector: hb_codepoint_t,
                     glyph: *mut hb_codepoint_t,
                     _user_data: *c_void) -> hb_bool_t unsafe {

    let font: *Font = reinterpret_cast(font_data);
    assert font.is_not_null();

    return alt (*font).glyph_idx(unicode as char) {
           some(g) {
               *glyph = g as hb_codepoint_t;
               true
           }
           none {
               false
           }
    } as hb_bool_t;
}

extern fn glyph_h_advance_func(_font: *hb_font_t,
                               font_data: *c_void,
                               glyph: hb_codepoint_t,
                               _user_data: *c_void) -> hb_position_t unsafe {
    let font: *Font = reinterpret_cast(font_data);
    assert font.is_not_null();

    let h_advance = (*font).glyph_h_advance(glyph as uint);
    #debug("h_advance for codepoint %? is %?", glyph, h_advance);
    return h_advance as hb_position_t;
}

fn hb_glyph_pos_to_servo_glyph_pos(hb_pos: &hb_glyph_position_t) -> GlyphPos {
    GlyphPos(Point2D(px_to_au(hb_pos.x_advance as int),
                     px_to_au(hb_pos.y_advance as int)),
             Point2D(px_to_au(hb_pos.x_offset as int),
                     px_to_au(hb_pos.y_offset as int)))
}

fn should_get_glyph_indexes() {
    #[test];
    #[ignore(reason = "random failures")];

    let font = font::create_test_font();
    let glyphs = shape_text(font, ~"firecracker");
    let idxs = glyphs.map(|glyph| glyph.index);
    assert idxs == ~[32u, 8u, 13u, 14u, 10u, 13u, 201u, 10u, 37u, 14u, 13u];
}

fn should_get_glyph_h_advance() {
    #[test];
    #[ignore(reason = "random failures")];

    let font = font::create_test_font();
    let glyphs = shape_text(font, ~"firecracker");
    let actual = glyphs.map(|g| g.pos.advance.x);
    let expected = (~[6, 4, 7, 9, 8, 7, 10, 8, 9, 9, 7]).map(|a| px_to_au(a));
    assert expected == actual;
}
