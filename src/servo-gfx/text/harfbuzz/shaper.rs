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
use text::shaper::ShaperMethods;

use servo_util::range;
use range::Range;

use core::libc::types::common::c99::int32_t;
use core::libc::{c_uint, c_int, c_void, c_char};
use core::util::ignore;
use dvec::DVec;
use std::arc;

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

pub struct ShapedGlyphData {
    count: uint,
    glyph_infos: *hb_glyph_info_t,
    pos_infos: *hb_glyph_position_t,
}

pub struct ShapedGlyphEntry {
    cluster: uint,
    codepoint: GlyphIndex,
    advance: Au,
    offset: Option<Point2D<Au>>,
}

pub impl ShapedGlyphData {
    static pure fn new(buffer: *hb_buffer_t) -> ShapedGlyphData unsafe {
        let glyph_count = 0 as c_uint;
        let glyph_infos = hb_buffer_get_glyph_infos(buffer, ptr::to_unsafe_ptr(&glyph_count));
        let glyph_count = glyph_count as uint;
        assert glyph_infos.is_not_null();
        let pos_count = 0 as c_uint;
        let pos_infos = hb_buffer_get_glyph_positions(buffer, ptr::to_unsafe_ptr(&pos_count));
        assert pos_infos.is_not_null();
        assert glyph_count == pos_count as uint;

        ShapedGlyphData {
            count: glyph_count,
            glyph_infos: glyph_infos,
            pos_infos: pos_infos,
        }
    }

    #[inline(always)]
    priv pure fn byte_offset_of_glyph(&const self, i: uint) -> uint unsafe {
        assert i < self.count;

        let glyph_info_i = ptr::offset(self.glyph_infos, i);
        return (*glyph_info_i).cluster as uint;
    }

    pure fn len() -> uint { self.count }

    // Returns shaped glyph data for one glyph, and updates the y-position of the pen.
    fn get_entry_for_glyph(i: uint, y_pos: &mut Au) -> ShapedGlyphEntry unsafe {
        assert i < self.count;

        let glyph_info_i = ptr::offset(self.glyph_infos, i);
        let pos_info_i = ptr::offset(self.pos_infos, i);
        let x_offset = unsafe { Au::from_frac_px(HarfbuzzShaper::fixed_to_float((*pos_info_i).x_offset)) };
        let y_offset = unsafe { Au::from_frac_px(HarfbuzzShaper::fixed_to_float((*pos_info_i).y_offset)) };
        let x_advance = unsafe { Au::from_frac_px(HarfbuzzShaper::fixed_to_float((*pos_info_i).x_advance)) };
        let y_advance = unsafe { Au::from_frac_px(HarfbuzzShaper::fixed_to_float((*pos_info_i).y_advance)) };
        let offset = if x_offset == Au(0)
                     && y_offset == Au(0)
                     && y_advance == Au(0) { None }
                     else {
                         // adjust the pen..
                         if y_advance > Au(0) {
                             *y_pos -= y_advance;
                         }

                         Some(Point2D(x_offset, y_pos - y_offset))
                     };

        ShapedGlyphEntry {
            cluster: (*glyph_info_i).cluster as uint, 
            codepoint: (*glyph_info_i).codepoint as GlyphIndex, 
            advance: x_advance,
            offset: move offset,
        }
    }
}

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

pub impl HarfbuzzShaper : ShaperMethods {    
    /**
    Calculate the layout metrics associated with a some given text
    when rendered in a specific font.
    */
    fn shape_text(text: &str, glyphs: &mut GlyphStore) {
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
}

pub impl HarfbuzzShaper {

    priv fn save_glyph_results(text: &str, glyphs: &mut GlyphStore, buffer: *hb_buffer_t) {
        let glyph_data = ShapedGlyphData::new(buffer);
        let glyph_count = glyph_data.len();
        let byte_max = text.len();
        let char_max = str::char_len(text);
        // GlyphStore records are indexed by character, not byte offset.
        // so, we must be careful to increment this when saving glyph entries.
        let mut char_idx = 0;

        assert glyph_count <= char_max;

        debug!("Shaped text[char count=%u], got back %u glyph info records.", char_max, glyph_count);
        if char_max != glyph_count {
            debug!("NOTE: Since these are not equal, we probably have been given some complex glyphs.");
        }

        // make map of what chars have glyphs
        const NO_GLYPH : i32 = -1;
        const CONTINUATION_BYTE : i32 = -2;
        let mut byteToGlyph : ~[i32];

        // fast path: all chars are single-byte.
        if byte_max == char_max {
            byteToGlyph = vec::from_elem(byte_max, NO_GLYPH);
        } else {
            byteToGlyph = vec::from_elem(byte_max, CONTINUATION_BYTE);
            let mut i = 0u;
            while i < byte_max {
                byteToGlyph[i] = NO_GLYPH;
                let range = str::char_range_at(text, i);
                ignore(range.ch);
                i = range.next;
            }
        }
        
        debug!("(glyph idx) -> (text byte offset)");
        for uint::range(0, glyph_data.len()) |i| {
            // loc refers to a *byte* offset within the utf8 string.
            let loc = glyph_data.byte_offset_of_glyph(i);
            if loc < byte_max {
                assert byteToGlyph[loc] != CONTINUATION_BYTE;
                byteToGlyph[loc] = i as i32;
            }
            else { debug!("ERROR: tried to set out of range byteToGlyph: idx=%u, glyph idx=%u", loc, i); }
            debug!("%u -> %u", i, loc);
        }

        debug!("text: %s", text);
        debug!("(char idx): char->(glyph index):");
        let mut i = 0u;
        while i < byte_max {
            let range = str::char_range_at(text, i);
            debug!("%u: %? --> %d", i, range.ch, byteToGlyph[i] as int);
            i = range.next;
        }

        // some helpers
        let mut glyph_span : Range = Range::empty();
        // this span contains first byte of first char, to last byte of last char in range.
        // so, end() points to first byte of last+1 char, if it's less than byte_max.
        let mut char_byte_span : Range = Range::empty();
        let mut y_pos = Au(0);

        // main loop over each glyph. each iteration usually processes 1 glyph and 1+ chars.
        // in cases with complex glyph-character assocations, 2+ glyphs and 1+ chars can be processed.
        while glyph_span.begin() < glyph_count {
            // start by looking at just one glyph.
            glyph_span.extend_by(1);
            debug!("Processing glyph at idx=%u", glyph_span.begin());

            let char_byte_start = glyph_data.byte_offset_of_glyph(glyph_span.begin());
            char_byte_span.reset(char_byte_start, 0);

            // find a range of chars corresponding to this glyph, plus
            // any trailing chars that do not have associated glyphs.
            while char_byte_span.end() < byte_max {
                let range = str::char_range_at(text, char_byte_span.end());
                ignore(range.ch);
                char_byte_span.extend_to(range.next);

                debug!("Processing char byte span: off=%u, len=%u for glyph idx=%u",
                       char_byte_span.begin(), char_byte_span.length(), glyph_span.begin());

                while char_byte_span.end() != byte_max && byteToGlyph[char_byte_span.end()] == NO_GLYPH {
                    debug!("Extending char byte span to include byte offset=%u with no associated glyph", char_byte_span.end());
                    let range = str::char_range_at(text, char_byte_span.end());
                    ignore(range.ch);
                    char_byte_span.extend_to(range.next);
                }

                // extend glyph range to max glyph index covered by char_span,
                // in cases where one char made several glyphs and left some unassociated chars.
                let mut max_glyph_idx = glyph_span.end();
                for char_byte_span.eachi |i| {
                    if byteToGlyph[i] > NO_GLYPH {
                        max_glyph_idx = uint::max(byteToGlyph[i] as uint, max_glyph_idx);
                    }
                }

                if max_glyph_idx > glyph_span.end() {
                    glyph_span.extend_to(max_glyph_idx);
                    debug!("Extended glyph span (off=%u, len=%u) to cover char byte span's max glyph index",
                           glyph_span.begin(), glyph_span.length());
                }

            
                // if there's just one glyph, then we don't need further checks.
                if glyph_span.length() == 1 { break; }

                // if no glyphs were found yet, extend the char byte range more.
                if glyph_span.length() == 0 { loop; }

                debug!("Complex (multi-glyph to multi-char) association found. This case probably doesn't work.");

                let mut all_glyphs_are_within_cluster: bool = true;
                do glyph_span.eachi |j| {
                    let loc = glyph_data.byte_offset_of_glyph(j);
                    if !char_byte_span.contains(loc) {
                        all_glyphs_are_within_cluster = false;
                    }
                    all_glyphs_are_within_cluster // if true, keep checking. else, stop.
                }

                debug!("All glyphs within char_byte_span cluster?: %?", all_glyphs_are_within_cluster);

                // found a valid range; stop extending char_span.
                if all_glyphs_are_within_cluster { break; }
            }

            // character/glyph clump must contain characters.
            assert char_byte_span.length() > 0;
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
            let mut covered_byte_span = copy char_byte_span;
            // extend, clipping at end of text range.
            while covered_byte_span.end() < byte_max 
                && byteToGlyph[covered_byte_span.end()] == NO_GLYPH {
                let range = str::char_range_at(text, covered_byte_span.end());
                ignore(range.ch);
                covered_byte_span.extend_to(range.next);
            }

            if covered_byte_span.begin() >= byte_max {
                // oops, out of range. clip and forget this clump.
                glyph_span.reset(glyph_span.end(), 0);
                char_byte_span.reset(char_byte_span.end(), 0);
            }

            // clamp to end of text. (I don't think this will be necessary, but..)
            covered_byte_span.extend_to(uint::min(covered_byte_span.end(), byte_max));

            // fast path: 1-to-1 mapping of single char and single glyph.
            if glyph_span.length() == 1 {
                // TODO(Issue #214): cluster ranges need to be computed before
                // shaping, and then consulted here.
                // for now, just pretend that every character is a cluster start.
                // (i.e., pretend there are no combining character sequences).
                // 1-to-1 mapping of character to glyph also treated as ligature start.
                let shape = glyph_data.get_entry_for_glyph(glyph_span.begin(), &mut y_pos);
                let data = GlyphData(shape.codepoint, shape.advance, shape.offset, false, true, true);
                glyphs.add_glyph_for_char_index(char_idx, &data);
            } else {
                // collect all glyphs to be assigned to the first character.
                let datas = DVec();

                for glyph_span.eachi |glyph_i| {
                    let shape = glyph_data.get_entry_for_glyph(glyph_i, &mut y_pos);
                    datas.push(GlyphData(shape.codepoint, 
                                         shape.advance, 
                                         shape.offset,
                                         false, // not missing
                                         true,  // treat as cluster start
                                         glyph_i > glyph_span.begin())); // all but first are ligature continuations
                    glyph_span.adjust_by(1,-1);
                }

                // now add the detailed glyph entry.
                glyphs.add_glyphs_for_char_index(char_idx, dvec::unwrap(move datas));
                
                // set the other chars, who have no glyphs
                let mut i = covered_byte_span.begin();
                loop {
                    let range = str::char_range_at(text, i);
                    ignore(range.ch);
                    i = range.next;
                    if i >= covered_byte_span.end() { break; }
                    char_idx += 1;
                    glyphs.add_nonglyph_for_char_index(char_idx, false, false);
                }
            }

            // shift up our working spans past things we just handled.
            glyph_span.reset(glyph_span.end(), 0);
            char_byte_span.reset(char_byte_span.end(), 0);
            char_idx += 1;
        }

        // this must be called after adding all glyph data; it sorts the
        // lookup table for finding detailed glyphs by associated char index.
        glyphs.finalize_changes();
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
