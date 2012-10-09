extern mod harfbuzz;

use au = gfx::geometry;
use au::au;
use core::num::from_int;
use font::Font;
use font_cache::FontCache;
use geom::point::Point2D;
use glyph::{GlyphStore, GlyphIndex, GlyphData};
use libc::types::common::c99::int32_t;
use libc::{c_uint, c_int, c_void, c_char};
use ptr::{null, to_unsafe_ptr, offset};
use std::arc;
use text_run::TextRun;
use util::*;


use cast::reinterpret_cast;
use harfbuzz::{HB_MEMORY_MODE_READONLY,
                  HB_DIRECTION_LTR};
use harfbuzz::{hb_blob_t, hb_face_t, hb_font_t, hb_font_funcs_t, hb_buffer_t,
                  hb_codepoint_t, hb_bool_t, hb_glyph_position_t,
		  hb_glyph_info_t, hb_var_int_t, hb_position_t};
use harfbuzz::bindgen::{hb_blob_create, hb_blob_destroy,
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

fn float_to_fixed_hb(f: float) -> i32 {
    util::float_to_fixed(16, f)
}

fn fixed_to_float_hb(i: hb_position_t) -> float {
    util::fixed_to_float(16, i)
}

fn fixed_to_rounded_int_hb(f: hb_position_t) -> int {
    util::fixed_to_rounded_int(16, f)
}

/**
Calculate the layout metrics associated with a some given text
when rendered in a specific font.
*/
pub fn shape_textrun(font: &Font, run: &TextRun) {
    debug!("shaping text '%s'", run.text);

    // TODO: harfbuzz fonts and faces should be cached on the Font object.
    // TODO: font tables should be stored in Font object and cached by FontCache (Issue #92)
    let face_blob: *hb_blob_t = vec::as_imm_buf(*(*font).fontbuf, |buf: *u8, len: uint| {
        hb_blob_create(buf as *c_char,
                       len as c_uint,
                       HB_MEMORY_MODE_READONLY,
                       null(),
                       null())
    });

    let hb_face: *hb_face_t = hb_face_create(face_blob, 0 as c_uint);
    let hb_font: *hb_font_t = hb_font_create(hb_face);

    // TODO: set font size here, based on Font's size
    // Set points-per-em. if zero, performs no hinting in that direction.
    hb_font_set_ppem(hb_font, 21 as c_uint, 21 as c_uint);
    // Set scaling. Note that this takes 16.16 fixed point.
    hb_font_set_scale(hb_font, float_to_fixed_hb(21f) as c_int, float_to_fixed_hb(21f) as c_int);

    let funcs: *hb_font_funcs_t = hb_font_funcs_create();
    hb_font_funcs_set_glyph_func(funcs, glyph_func, null(), null());
    hb_font_funcs_set_glyph_h_advance_func(funcs, glyph_h_advance_func, null(), null());

    unsafe {
        let font_data: *c_void = cast::transmute(font);
        hb_font_set_funcs(hb_font, funcs, font_data, null());
    };

    let hb_buffer: *hb_buffer_t = hb_buffer_create();
    hb_buffer_set_direction(hb_buffer, HB_DIRECTION_LTR);

    // Using as_buf because it never does a copy - we don't need the trailing null
    str::as_buf(run.text, |ctext: *u8, _l: uint| {
        hb_buffer_add_utf8(hb_buffer, 
                           ctext as *c_char,
                           run.text.len() as c_int,
                           0 as c_uint,
                           run.text.len() as c_int);
    });

    hb_shape(hb_font, hb_buffer, null(), 0 as c_uint);

    let info_buf_len = 0 as c_uint;
    let info_buf = hb_buffer_get_glyph_infos(hb_buffer, to_unsafe_ptr(&info_buf_len));
    assert info_buf.is_not_null();
    let pos_buf_len = 0 as c_uint;
    let pos_buf = hb_buffer_get_glyph_positions(hb_buffer, to_unsafe_ptr(&pos_buf_len));
    assert pos_buf.is_not_null();

    assert info_buf_len == pos_buf_len;

    for uint::range(0u, info_buf_len as uint) |i| { unsafe {
        let hb_info: hb_glyph_info_t = *offset(info_buf, i);
        let hb_pos: hb_glyph_position_t = *offset(pos_buf, i);
        let codepoint = hb_info.codepoint as GlyphIndex;
        let advance: au = au::from_frac_px(fixed_to_float_hb(hb_pos.x_advance));
        let offset = match (hb_pos.x_offset, hb_pos.y_offset) {
            (0, 0) => None,
            (x, y) => Some(Point2D(au::from_frac_px(fixed_to_float_hb(x)),
                                    au::from_frac_px(fixed_to_float_hb(y))))
        };
        // TODO: convert pos.y_advance into offset adjustment
        // TODO: handle multiple glyphs per char, ligatures, etc.
        // See Issue #
        debug!("glyph %?: index %?, advance %?, offset %?",
               i, codepoint, advance, offset);

        let data = GlyphData(codepoint, advance, offset, false, false, false);
        run.glyphs.add_glyph_for_index(i, &data);
    } /* unsafe */ }

    hb_buffer_destroy(hb_buffer);
    hb_font_funcs_destroy(funcs);
    hb_font_destroy(hb_font);
    hb_face_destroy(hb_face);
    hb_blob_destroy(face_blob);
}

extern fn glyph_func(_font: *hb_font_t,
                     font_data: *c_void,
                     unicode: hb_codepoint_t,
                     _variant_selector: hb_codepoint_t,
                     glyph: *mut hb_codepoint_t,
                     _user_data: *c_void) -> hb_bool_t unsafe {

    let font: *Font = cast::transmute(font_data);
    assert font.is_not_null();

    return match (*font).glyph_index(unicode as char) {
        Some(g) => { *glyph = g as hb_codepoint_t; true },
        None => false
    } as hb_bool_t;
}

extern fn glyph_h_advance_func(_font: *hb_font_t,
                               font_data: *c_void,
                               glyph: hb_codepoint_t,
                               _user_data: *c_void) -> hb_position_t unsafe {
    let font: *Font = cast::transmute(font_data);
    assert font.is_not_null();

    let advance = (*font).glyph_h_advance(glyph as GlyphIndex);
    float_to_fixed_hb(advance)
}

fn should_get_glyph_indexes() {
    #[test];
    #[ignore(cfg(target_os = "macos"), reason = "bad metrics")];

    let lib = FontCache();
    let font = lib.get_test_font();
    let glyphs = shape_text(font, ~"firecracker");
    let idxs = glyphs.map(|glyph| glyph.index);
    assert idxs == ~[32u32, 8u32, 13u32, 14u32, 10u32, 13u32, 201u32, 10u32, 37u32, 14u32, 13u32];
}

fn should_get_glyph_h_advance() {
    #[test];
    #[ignore(cfg(target_os = "macos"), reason = "bad metrics")];

    let lib = FontCache();
    let font = lib.get_test_font();
    let glyphs = shape_text(font, ~"firecracker");
    let actual = glyphs.map(|g| g.pos.advance.x);
    let expected = (~[6, 4, 7, 9, 8, 7, 10, 8, 9, 9, 7]).map(|a| au::from_px(*a));
    assert expected == actual;
}
