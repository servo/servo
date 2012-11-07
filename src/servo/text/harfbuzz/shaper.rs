extern mod harfbuzz;

use geom::Point2D;

use gfx::au;
use gfx::{
    Au,
    Font,
};
use glyph::{GlyphStore, GlyphIndex, GlyphData};

use libc::types::common::c99::int32_t;
use libc::{c_uint, c_int, c_void, c_char};
use ptr::{null, to_unsafe_ptr, offset};
use std::arc;

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

pub struct HarfbuzzShaper {
    priv font: @Font,
    priv hb_blob: *hb_blob_t,
    priv hb_face: *hb_face_t,
    priv hb_font: *hb_font_t,
    priv hb_funcs: *hb_font_funcs_t,

    drop {
        assert self.hb_blob.is_not_null();
        hb_blob_destroy(self.hb_blob);

        assert self.hb_face.is_not_null();
        hb_face_destroy(self.hb_face);

        assert self.hb_font.is_not_null();
        hb_font_destroy(self.hb_font);

        assert self.hb_funcs.is_not_null();
        hb_font_funcs_destroy(self.hb_funcs);
    }
}

pub impl HarfbuzzShaper {
    static pub fn new(font: @Font) -> HarfbuzzShaper {
        // TODO(Issue #92): font tables should be stored in Font object and cached per-task
        let hb_blob: *hb_blob_t = vec::as_imm_buf(*(font).buf(), |buf: *u8, len: uint| {
            hb_blob_create(buf as *c_char,
                           len as c_uint,
                           HB_MEMORY_MODE_READONLY,
                           null(),
                           null())
        });

        let hb_face: *hb_face_t = hb_face_create(hb_blob, 0 as c_uint);
        let hb_font: *hb_font_t = hb_font_create(hb_face);
        // Set points-per-em. if zero, performs no hinting in that direction.
        let pt_size = font.style.pt_size;
        hb_font_set_ppem(hb_font, pt_size as c_uint, pt_size as c_uint);
        // Set scaling. Note that this takes 16.16 fixed point.
        hb_font_set_scale(hb_font, 
                          HarfbuzzShaper::float_to_fixed(pt_size) as c_int,
                          HarfbuzzShaper::float_to_fixed(pt_size) as c_int);

        // configure static function callbacks.
        // NB. This funcs structure could be reused globally, as it never changes.
        let hb_funcs: *hb_font_funcs_t = hb_font_funcs_create();
        hb_font_funcs_set_glyph_func(hb_funcs, glyph_func, null(), null());
        hb_font_funcs_set_glyph_h_advance_func(hb_funcs, glyph_h_advance_func, null(), null());
        unsafe {
            let font_data: *c_void = core::ptr::addr_of(font) as *c_void;
            hb_font_set_funcs(hb_font, hb_funcs, font_data, null());
        };

        HarfbuzzShaper { 
            font: font,
            hb_blob: hb_blob,
            hb_face: hb_face,
            hb_font: hb_font,
            hb_funcs: hb_funcs,
        }
    }
    
    /**
    Calculate the layout metrics associated with a some given text
    when rendered in a specific font.
    */
    pub fn shape_text(text: &str, glyphs: &GlyphStore) {
        debug!("shaping text '%s'", text);

        // TODO(Issue #94): harfbuzz fonts and faces should be cached on the 
        // Shaper object, which is owned by the Font instance.

        let hb_buffer: *hb_buffer_t = hb_buffer_create();
        hb_buffer_set_direction(hb_buffer, HB_DIRECTION_LTR);

        // Using as_buf because it never does a copy - we don't need the trailing null
        str::as_buf(text, |ctext: *u8, _l: uint| {
            hb_buffer_add_utf8(hb_buffer, 
                               ctext as *c_char,
                               text.len() as c_int,
                               0 as c_uint,
                               text.len() as c_int);
        });

        hb_shape(self.hb_font, hb_buffer, null(), 0 as c_uint);

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
            let advance: Au = au::from_frac_px(HarfbuzzShaper::fixed_to_float(hb_pos.x_advance));
            let offset = match (hb_pos.x_offset, hb_pos.y_offset) {
                (0, 0) => None,
                (x, y) => Some(Point2D(au::from_frac_px(HarfbuzzShaper::fixed_to_float(x)),
                                       au::from_frac_px(HarfbuzzShaper::fixed_to_float(y))))
            };
            // TODO: convert pos.y_advance into offset adjustment
            // TODO: handle multiple glyphs per char, ligatures, etc.
            // NB. this debug statement is commented out, as it must be checked for every shaped char.
            //debug!("glyph %?: index %?, advance %?, offset %?", i, codepoint, advance, offset);

            let data = GlyphData(codepoint, advance, offset, false, false, false);
            glyphs.add_glyph_for_index(i, &data);
        } /* unsafe */ }

        hb_buffer_destroy(hb_buffer);
    }

    static priv fn float_to_fixed(f: float) -> i32 {
        util::float_to_fixed(16, f)
    }

    static priv fn fixed_to_float(i: hb_position_t) -> float {
        util::fixed_to_float(16, i)
    }

    static priv fn fixed_to_rounded_int(f: hb_position_t) -> int {
        util::fixed_to_rounded_int(16, f)
    }
}

/// Callbacks from Harfbuzz when font map and glyph advance lookup needed.
extern fn glyph_func(_font: *hb_font_t,
                     font_data: *c_void,
                     unicode: hb_codepoint_t,
                     _variant_selector: hb_codepoint_t,
                     glyph: *mut hb_codepoint_t,
                     _user_data: *c_void) -> hb_bool_t unsafe {
    let font: *Font = font_data as *Font;
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
    let font: *Font = font_data as *Font;
    assert font.is_not_null();

    let advance = (*font).glyph_h_advance(glyph as GlyphIndex);
    HarfbuzzShaper::float_to_fixed(advance)
}
