extern mod harfbuzz;

use geom::Point2D;

use au = geometry;
use au::Au;

use font::{
    Font,
    FontTable,
    FontTableTag,
};

use glyph::{GlyphStore, GlyphIndex, GlyphData};
use servo_util::range;
use range::MutableRange;

use core::libc::types::common::c99::int32_t;
use core::libc::{c_uint, c_int, c_void, c_char};
use std::arc;
use dvec::DVec;

use harfbuzz::{HB_MEMORY_MODE_READONLY, HB_DIRECTION_LTR, hb_blob_t, hb_face_t, hb_font_t};
use harfbuzz::{hb_font_funcs_t, hb_buffer_t, hb_codepoint_t, hb_bool_t, hb_glyph_position_t};
use harfbuzz::{hb_glyph_info_t, hb_var_int_t, hb_position_t};
use harfbuzz::bindgen::{hb_blob_create, hb_blob_destroy, hb_face_create, hb_face_destroy};
use harfbuzz::bindgen::{hb_font_create, hb_font_destroy, hb_buffer_create, hb_buffer_destroy};
use harfbuzz::bindgen::{hb_buffer_add_utf8, hb_shape, hb_buffer_get_glyph_infos};
use harfbuzz::bindgen::{hb_buffer_get_glyph_positions, hb_font_set_ppem, hb_font_set_scale};
use harfbuzz::bindgen::{hb_buffer_set_direction, hb_font_funcs_create, hb_font_funcs_destroy};
use harfbuzz::bindgen::{hb_font_set_funcs, hb_font_funcs_set_glyph_h_advance_func};
use harfbuzz::bindgen::{hb_font_funcs_set_glyph_func, hb_font_funcs_set_glyph_h_kerning_func};

use harfbuzz::{HB_MEMORY_MODE_READONLY,
                  HB_DIRECTION_LTR};
use harfbuzz::{hb_blob_t, hb_face_t, hb_font_t, hb_font_funcs_t, hb_buffer_t,
                  hb_codepoint_t, hb_bool_t, hb_glyph_position_t,
		  hb_glyph_info_t, hb_var_int_t, hb_position_t, hb_tag_t};
use harfbuzz::bindgen::{hb_blob_create, hb_blob_destroy,
                           hb_face_create_for_tables, hb_face_destroy,
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
    font: @Font,
    priv hb_face: *hb_face_t,
    priv hb_font: *hb_font_t,
    priv hb_funcs: *hb_font_funcs_t,

    drop {
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
        let hb_face: *hb_face_t = hb_face_create_for_tables(get_font_table_func, ptr::to_unsafe_ptr(font) as *c_void, ptr::null());
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
        hb_font_funcs_set_glyph_func(hb_funcs, glyph_func, ptr::null(), ptr::null());
        hb_font_funcs_set_glyph_h_advance_func(hb_funcs, glyph_h_advance_func, ptr::null(), ptr::null());
        unsafe {
            let font_data: *c_void = core::ptr::addr_of(font) as *c_void;
            hb_font_set_funcs(hb_font, hb_funcs, font_data, ptr::null());
        };

        HarfbuzzShaper { 
            font: font,
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

        hb_shape(self.hb_font, hb_buffer, ptr::null(), 0 as c_uint);
        self.save_glyph_results(text, glyphs, hb_buffer);
        hb_buffer_destroy(hb_buffer);
    }

    priv fn save_glyph_results(text: &str, glyphs: &GlyphStore, buffer: *hb_buffer_t) {
        // TODO: We probably aren't handling bytes-to-chars mapping
        // correctly in this routine.  it will probably explode with
        // multi-byte utf8 codepoints.

        let char_max = str::char_len(text);

        // get the results out of the hb_buffer_t
        let glyph_count = 0 as c_uint;
        let glyph_infos = hb_buffer_get_glyph_infos(buffer, ptr::to_unsafe_ptr(&glyph_count));
        let glyph_count = glyph_count as uint;
        assert glyph_infos.is_not_null();
        let pos_count = 0 as c_uint;
        let pos_infos = hb_buffer_get_glyph_positions(buffer, ptr::to_unsafe_ptr(&pos_count));
        assert pos_infos.is_not_null();
        assert glyph_count == pos_count as uint;

        // wohoo
        debug!("Shaped text[char count=%u], got back %u glyph info records.", char_max, glyph_count);
        if char_max != glyph_count {
            debug!("Since these are not equal, we probably have been given some complex glyphs!");
        }

        // make map of what chars have glyphs
        const NO_GLYPH : i32 = -1;
        let mut charToGlyph : ~[i32] = vec::from_elem(char_max, NO_GLYPH);
        debug!("(glyph idx) -> (char cluster)");
        for i32::range(0, glyph_count as i32) |i| {
            let info_i = ptr::offset(glyph_infos, i as uint);
            // loc refers to a *byte* offset within the utf8 string.
            let loc: uint = unsafe { (*info_i).cluster as uint };
            debug!("%u -> %u", i as uint, loc);
            if loc < char_max { charToGlyph[loc] = i; }
            else { debug!("Tried to set out of range charToGlyph: idx=%u, glyph idx=%u", loc, i as uint); }
        }

        debug!("text: %s", text);
        debug!("(char idx): char->(glyph index):");
        for str::each_chari(text) |i, ch| {
            debug!("%u: %? --> %d", i, ch, charToGlyph[i] as int);
        }

        // some helpers
        let glyph_span : MutableRange = range::empty_mut();
        let char_span : MutableRange = range::empty_mut();
        let mut y_pos = Au(0);

        // main loop over each glyph. each iteration usually processes 1 glyph and 1+ chars.
        // in cases with complex glyph-character assocations, 2+ glyphs and 1+ chars can be processed.
        while glyph_span.begin() < glyph_count {
            // start by looking at just one glyph.
            glyph_span.extend_by(1);
            debug!("Processing glyph at idx=%u", glyph_span.begin());

            let glyph_info_i = ptr::offset(glyph_infos, glyph_span.begin());
            let pos_info_i = ptr::offset(pos_infos, glyph_span.begin());
            let char_end = unsafe { (*glyph_info_i).cluster as uint };

            char_span.extend_to(char_end);

            // find a range of chars corresponding to this glyph, plus
            // any trailing chars that do not have associated glyphs.
            while char_span.end() < char_max {
                char_span.extend_by(1);

                debug!("Processing char span: off=%u, len=%u for glyph idx=%u",
                       char_span.begin(), char_span.length(), glyph_span.begin());

                while char_span.end() != char_max && charToGlyph[char_span.end()] == NO_GLYPH {
                    debug!("Extending char span to include char idx=%u with no associated glyph", char_span.end());
                    char_span.extend_by(1);
                }

                // extend glyph range to max glyph index covered by char_span,
                // in cases where one char made several glyphs and left some unassociated chars.
                let mut max_glyph_idx = glyph_span.end();
                for char_span.eachi |i| {
                    if charToGlyph[i] != NO_GLYPH {
                        max_glyph_idx = uint::max(charToGlyph[i] as uint, max_glyph_idx);
                    }
                }

                if max_glyph_idx > glyph_span.end() {
                    glyph_span.extend_to(max_glyph_idx);
                    debug!("Extended glyph span (off=%u, len=%u) to cover char span's max glyph index",
                           glyph_span.begin(), glyph_span.length());
                }

            
                // if there's just one glyph, then we don't need further checks.
                if glyph_span.length() == 1 { break; }

                // if no glyphs were found yet, extend the char range more.
                if glyph_span.length() == 0 { loop; }

                debug!("Complex (multi-glyph to multi-char) association found. This case probably doesn't work.");

                let mut all_glyphs_are_within_cluster: bool = true;
                do char_span.eachi |j| {
                    let glyph_info_j = ptr::offset(glyph_infos, j);
                    let cluster_idx = unsafe { (*glyph_info_j).cluster as uint };
                    if cluster_idx < char_span.begin() || cluster_idx > char_span.end() {
                        all_glyphs_are_within_cluster = false;
                    }
                    all_glyphs_are_within_cluster // if true, keep checking. else, stop.
                }

                debug!("All glyphs within char_span cluster?: %?", all_glyphs_are_within_cluster);

                // found a valid range; stop extending char_span.
                if all_glyphs_are_within_cluster { break; }
            }

            // character/glyph clump must contain characters.
            assert char_span.length() > 0;
            // character/glyph clump must contain glyphs.
            assert glyph_span.length() > 0;

            // now char_span is a ligature clump, formed by the glyphs in glyph_span.
            // we need to find the chars that correspond to actual glyphs (char_extended_span),
            //and set glyph info for those and empty infos for the chars that are continuations.

            // a simple example:
            // chars:  'f'     't'   't'   
            // glyphs: 'ftt'   ''    ''
            // cgmap:  t        f     f
            // gspan:  [-]
            // cspan:  [-]
            // covsp:  [---------------]
            let chars_covered_span = copy char_span;
            // extend, clipping at end of text range.
            while chars_covered_span.end() < char_max 
                && charToGlyph[chars_covered_span.end()] == NO_GLYPH {
                chars_covered_span.extend_by(1);
            }

            if chars_covered_span.begin() >= char_max {
                // oops, out of range. clip and forget this clump.
                glyph_span.reset(glyph_span.end(), 0);
                char_span.reset(char_span.end(), 0);
            }

            // clamp to end of text. (I don't think this will be necessary, but..)
            let covered_end = uint::min(chars_covered_span.end(), char_max);
            chars_covered_span.extend_to(covered_end);

            // TODO: extract this into a struct passed by reference to helper function
            let mut codepoint = unsafe { (*glyph_info_i).codepoint as GlyphIndex };
            let mut x_offset = unsafe { Au::from_frac_px(HarfbuzzShaper::fixed_to_float((*pos_info_i).x_offset)) };
            let mut y_offset = unsafe { Au::from_frac_px(HarfbuzzShaper::fixed_to_float((*pos_info_i).y_offset)) };
            let mut x_advance = unsafe { Au::from_frac_px(HarfbuzzShaper::fixed_to_float((*pos_info_i).x_advance)) };
            let mut y_advance = unsafe { Au::from_frac_px(HarfbuzzShaper::fixed_to_float((*pos_info_i).y_advance)) };
            let mut offset = Point2D(x_offset, y_pos - y_offset);
            // adjust our pen..
            if y_advance > Au(0) {
                y_pos -= y_advance;
            }

            // fast path: 1-to-1 mapping of single char and single glyph.
            if glyph_span.length() == 1 {
                // TODO(Issue #214): cluster ranges need to be computed before
                // shaping, and then consulted here.
                // for now, just pretend that every character is a cluster start.
                // (i.e., pretend there are no combining character sequences)
                let used_offset = if offset == Au::zero_point() { None } else { Some(offset) };
                let data = GlyphData(codepoint, x_advance, used_offset, false, true, true);
                glyphs.add_glyph_for_index(glyph_span.begin(), &data);
            } else {
                // collect all glyphs to be assigned to the first character.
                let datas = DVec();

                // there is at least one, and its advance was already
                // measured. So, the loop condition is placed weirdly.
                loop {
                    let used_offset = if offset == Au::zero_point() { None } else { Some(offset) };
                    datas.push(GlyphData(codepoint, x_advance, used_offset, false, true, true));

                    glyph_span.adjust_by(1,-1);
                    if glyph_span.length() == 0 { break; }

                    let glyph_info_j = ptr::offset(glyph_infos, glyph_span.begin());
                    let pos_info_j = ptr::offset(pos_infos, glyph_span.begin());
                    codepoint = unsafe { (*glyph_info_j).codepoint as GlyphIndex };
                    x_offset = unsafe { Au::from_frac_px(HarfbuzzShaper::fixed_to_float((*pos_info_j).x_offset)) };
                    y_offset = unsafe { Au::from_frac_px(HarfbuzzShaper::fixed_to_float((*pos_info_j).y_offset)) };
                    x_advance = unsafe { Au::from_frac_px(HarfbuzzShaper::fixed_to_float((*pos_info_j).x_advance)) };
                    y_advance = unsafe { Au::from_frac_px(HarfbuzzShaper::fixed_to_float((*pos_info_j).y_advance)) };
                    offset = Point2D(x_offset, y_pos - y_offset);
                    // adjust our pen..
                    if y_advance > Au(0) {
                        y_pos -= y_advance;
                    }
                }

                // now add the actual entry.
                glyphs.add_glyphs_for_index(glyph_span.begin(), dvec::unwrap(move datas));
                
                chars_covered_span.adjust_by(1, -1);
                // set the other chars, who have no glyphs
                for chars_covered_span.eachi |covered_j| {
                    glyphs.add_nonglyph_for_index(covered_j, false, false);
                }

            }

            // shift up our working spans past things we just handled.
            glyph_span.reset(glyph_span.end(), 0);
            char_span.reset(char_span.end(), 0);
        }
    }

/*
        for uint::range(0u, glyph_count as uint) |i| { unsafe {
            let hb_info: hb_glyph_info_t = *ptr::offset(glyph_infos, i);
            let hb_pos: hb_glyph_position_t = *ptr::offset(pos_infos, i);
            let codepoint = hb_info.codepoint as GlyphIndex;
            let advance: Au = Au::from_frac_px(HarfbuzzShaper::fixed_to_float(hb_pos.x_advance));
            let offset = match (hb_pos.x_offset, hb_pos.y_offset) {
                (0, 0) => None,
                (x, y) => Some(Point2D(Au::from_frac_px(HarfbuzzShaper::fixed_to_float(x)),
                                       Au::from_frac_px(HarfbuzzShaper::fixed_to_float(y))))
            };
            // TODO: convert pos.y_advance into offset adjustment
            // TODO(#95): handle multiple glyphs per char, ligatures, etc.
            // NB. this debug statement is commented out, as it must be checked for every shaped char.
            debug!("glyph %?: index %?, advance %?, offset %?", i, codepoint, advance, offset);

            let data = GlyphData(codepoint, advance, offset, false, false, false);
            glyphs.add_glyph_for_index(i, &data);
} } */

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

// Callback to get a font table out of a font.
extern fn get_font_table_func(_face: *hb_face_t, tag: hb_tag_t, user_data: *c_void) -> *hb_blob_t unsafe {
    let font: *Font = user_data as *Font;
    assert font.is_not_null();

    // TODO(Issue #197): reuse font table data, which will change the unsound trickery here.
    match (*font).get_table_for_tag(tag as FontTableTag) {
        None => return ptr::null(),
        Some(ref font_table) => {
            let skinny_font_table = ~font_table;
            let skinny_font_table_ptr = ptr::to_unsafe_ptr(skinny_font_table);
            let mut blob: *hb_blob_t = ptr::null();
            (*skinny_font_table_ptr).with_buffer(|buf: *u8, len: uint| {
                blob = hb_blob_create(buf as *c_char,
                                      len as c_uint,
                                      HB_MEMORY_MODE_READONLY,
                                      cast::transmute(skinny_font_table_ptr), // private context for below.
                                      destroy_blob_func); // HarfBuzz calls this when blob not needed.
            });
            assert blob.is_not_null();
            return blob;
        }
    }
}

// TODO(Issue #197): reuse font table data, which will change the unsound trickery here.
// In particular, we'll need to cast to a boxed, rather than owned, FontTable.

// even better, should cache the harfbuzz blobs directly instead of recreating a lot.
extern fn destroy_blob_func(user_data: *c_void) unsafe {
    // this will cause drop to run.
    let _wrapper : &~FontTable = cast::transmute(user_data);
}
