use harfbuzz;

export shape_text;

import libc::types::common::c99::int32_t;
import libc::{c_uint, c_int, c_void};
import font::font;
import glyph::{glyph, glyph_pos};
import ptr::{null, addr_of, offset};
import gfx::geom::{point, px_to_au};

import unsafe::reinterpret_cast;
import harfbuzz::{HB_MEMORY_MODE_READONLY,
                  HB_DIRECTION_LTR};
import harfbuzz::{hb_blob_t, hb_face_t, hb_font_t, hb_buffer_t,
                  hb_codepoint_t, hb_bool_t, hb_glyph_position_t,
				  hb_var_int_t};
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
fn shape_text(_font: &font, text: str) -> [glyph] {
    let mut glyphs = [];
    let mut cur_x = 0u;
    for text.each_char {
        |ch|
        // TODO: Use HarfBuzz!
        let hb_pos = {
            x_advance: 10 as int32_t,
            y_advance: 0 as int32_t,
            x_offset: cur_x as int32_t,
            y_offset: 0 as int32_t,
            var: 0 as hb_var_int_t
        };

        let pos = hb_glyph_pos_to_servo_glyph_pos(hb_pos);
        vec::push(glyphs, glyph(ch as uint, pos));
        cur_x += 10u;
    };

    ret glyphs;
}

fn shape_text2(font: &font, text: str) -> [glyph] unsafe {
    #debug("shaping text '%s'", text);

    let face_blob = vec::as_buf(*(*font).buf()) { |buf|
        hb_blob_create(reinterpret_cast(buf),
                       (*(*font).buf()).len() as c_uint,
                       HB_MEMORY_MODE_READONLY,
                       null(),
                       null())
    };

    let face = hb_face_create(face_blob, 0 as c_uint);
    let font = hb_font_create(face);

    hb_font_set_ppem(font, 10 as c_uint, 10 as c_uint);
    hb_font_set_scale(font, 10 as c_int, 10 as c_int);

    let funcs = hb_font_funcs_create();
    hb_font_funcs_set_glyph_func(funcs, glyph_func, null(), null());
    hb_font_set_funcs(font, funcs, addr_of(*font), null());

    let buffer = hb_buffer_create();

    hb_buffer_set_direction(buffer, HB_DIRECTION_LTR);

    str::as_c_str(text) { |ctext|
        hb_buffer_add_utf8(buffer, ctext,
                           text.len() as c_int,
                           0 as c_uint,
                           text.len() as c_int);
    }

    hb_shape(font, buffer, null(), 0 as c_uint);

    let info_len = 0 as c_uint;
    let info_ = hb_buffer_get_glyph_infos(buffer, addr_of(info_len));
    assert info_.is_not_null();
    let pos_len = 0 as c_uint;
    let pos = hb_buffer_get_glyph_positions(buffer, addr_of(pos_len));
    assert pos.is_not_null();

    assert info_len == pos_len;

    for uint::range(0u, info_len as uint) { |i|
        let info_ = offset(info_, i);
        let pos = offset(pos, i);
        #debug("glyph %?: codep %?, cluster %?,\
                x_adv %?, y_adv %?, x_off %?, y_of %?",
               i, (*info_).codepoint, (*info_).cluster,
               (*pos).x_advance, (*pos).y_advance,
               (*pos).x_offset, (*pos).y_offset);
    }

    hb_buffer_destroy(buffer);
    hb_font_funcs_destroy(funcs);
    hb_font_destroy(font);
    hb_face_destroy(face);
    hb_blob_destroy(face_blob);

    ret [];
}

crust fn glyph_func(_font: *hb_font_t,
                    _font_data: *c_void,
                    _unicode: hb_codepoint_t,
                    _variant_selector: hb_codepoint_t,
                    glyph: *mut hb_codepoint_t,
                    _user_data: *c_void) -> hb_bool_t unsafe {

    *glyph = 40 as hb_codepoint_t;
    ret true as hb_bool_t;
}

fn hb_glyph_pos_to_servo_glyph_pos(hb_pos: hb_glyph_position_t) -> glyph_pos {
    glyph_pos(point(px_to_au(hb_pos.x_advance as int),
                    px_to_au(hb_pos.y_advance as int)),
              point(px_to_au(hb_pos.x_offset as int),
                    px_to_au(hb_pos.y_offset as int)))
}

#[test]
fn test_shape_basic() {
    let font = font::create();
    shape_text2(&font, "firecracker");
}
