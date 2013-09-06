/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern mod harfbuzz;

use font::{Font, FontHandleMethods, FontTableMethods, FontTableTag};
use geometry::Au;
use platform::font::FontTable;
use text::glyph::{GlyphStore, GlyphIndex, GlyphData};
use text::shaping::ShaperMethods;
use servo_util::range::Range;
use text::util::{float_to_fixed, fixed_to_float, fixed_to_rounded_int};

use std::cast::transmute;
use std::char;
use std::libc::{c_uint, c_int, c_void, c_char};
use std::ptr;
use std::ptr::null;
use std::uint;
use std::util::ignore;
use std::vec;
use geom::Point2D;
use harfbuzz::{hb_blob_create, hb_face_create_for_tables};
use harfbuzz::{hb_buffer_add_utf8};
use harfbuzz::{hb_buffer_get_glyph_positions};
use harfbuzz::{hb_buffer_set_direction};
use harfbuzz::{hb_buffer_destroy};
use harfbuzz::{hb_face_destroy};
use harfbuzz::{hb_font_create};
use harfbuzz::{hb_font_destroy, hb_buffer_create};
use harfbuzz::{hb_font_funcs_create};
use harfbuzz::{hb_font_funcs_destroy};
use harfbuzz::{hb_font_funcs_set_glyph_func};
use harfbuzz::{hb_font_funcs_set_glyph_h_advance_func};
use harfbuzz::{hb_font_set_funcs};
use harfbuzz::{hb_font_set_ppem};
use harfbuzz::{hb_font_set_scale};
use harfbuzz::{hb_shape, hb_buffer_get_glyph_infos};
use harfbuzz::{HB_MEMORY_MODE_READONLY, HB_DIRECTION_LTR};
use harfbuzz::{hb_blob_t};
use harfbuzz::{hb_bool_t};
use harfbuzz::{hb_face_t, hb_font_t};
use harfbuzz::{hb_font_funcs_t, hb_buffer_t, hb_codepoint_t};
use harfbuzz::{hb_glyph_info_t};
use harfbuzz::{hb_glyph_position_t};
use harfbuzz::{hb_position_t, hb_tag_t};

static NO_GLYPH: i32 = -1;
static CONTINUATION_BYTE: i32 = -2;

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

impl ShapedGlyphData {
    #[fixed_stack_segment]
    pub fn new(buffer: *hb_buffer_t) -> ShapedGlyphData {
        unsafe {
            let glyph_count = 0;
            let glyph_infos = hb_buffer_get_glyph_infos(buffer, &glyph_count);
            let glyph_count = glyph_count as uint;
            assert!(glyph_infos.is_not_null());
            let pos_count = 0;
            let pos_infos = hb_buffer_get_glyph_positions(buffer, &pos_count);
            assert!(pos_infos.is_not_null());
            assert!(glyph_count == pos_count as uint);

            ShapedGlyphData {
                count: glyph_count,
                glyph_infos: glyph_infos,
                pos_infos: pos_infos,
            }
        }
    }

    #[inline(always)]
    fn byte_offset_of_glyph(&self, i: uint) -> uint {
        assert!(i < self.count);

        unsafe {
            let glyph_info_i = ptr::offset(self.glyph_infos, i as int);
            (*glyph_info_i).cluster as uint
        }
    }

    pub fn len(&self) -> uint {
        self.count
    }

    /// Returns shaped glyph data for one glyph, and updates the y-position of the pen.
    pub fn get_entry_for_glyph(&self, i: uint, y_pos: &mut Au) -> ShapedGlyphEntry {
        assert!(i < self.count);

        unsafe {
            let glyph_info_i = ptr::offset(self.glyph_infos, i as int);
            let pos_info_i = ptr::offset(self.pos_infos, i as int);
            let x_offset = Shaper::fixed_to_float((*pos_info_i).x_offset);
            let y_offset = Shaper::fixed_to_float((*pos_info_i).y_offset);
            let x_advance = Shaper::fixed_to_float((*pos_info_i).x_advance);
            let y_advance = Shaper::fixed_to_float((*pos_info_i).y_advance);

            let x_offset = Au::from_frac_px(x_offset);
            let y_offset = Au::from_frac_px(y_offset);
            let x_advance = Au::from_frac_px(x_advance);
            let y_advance = Au::from_frac_px(y_advance);

            let offset = if x_offset == Au(0) && y_offset == Au(0) && y_advance == Au(0) {
                None
            } else {
                // adjust the pen..
                if y_advance > Au(0) {
                    *y_pos = *y_pos - y_advance;
                }

                Some(Point2D(x_offset, *y_pos - y_offset))
            };

            ShapedGlyphEntry {
                cluster: (*glyph_info_i).cluster as uint,
                codepoint: (*glyph_info_i).codepoint as GlyphIndex,
                advance: x_advance,
                offset: offset,
            }
        }
    }
}

pub struct Shaper {
    font: @mut Font,
    priv hb_face: *hb_face_t,
    priv hb_font: *hb_font_t,
    priv hb_funcs: *hb_font_funcs_t,
}

#[unsafe_destructor]
impl Drop for Shaper {
    #[fixed_stack_segment]
    fn drop(&self) {
        unsafe {
            assert!(self.hb_face.is_not_null());
            hb_face_destroy(self.hb_face);

            assert!(self.hb_font.is_not_null());
            hb_font_destroy(self.hb_font);

            assert!(self.hb_funcs.is_not_null());
            hb_font_funcs_destroy(self.hb_funcs);
        }
    }
}

impl Shaper {
    #[fixed_stack_segment]
    pub fn new(font: @mut Font) -> Shaper {
        unsafe {
            // Indirection for Rust Issue #6248, dynamic freeze scope artifically extended
            let font_ptr = {
                let borrowed_font= &mut *font;
                borrowed_font as *mut Font
            };
            let hb_face: *hb_face_t = hb_face_create_for_tables(get_font_table_func,
                                                                font_ptr as *c_void,
                                                                None);
            let hb_font: *hb_font_t = hb_font_create(hb_face);

            // Set points-per-em. if zero, performs no hinting in that direction.
            let pt_size = font.style.pt_size;
            hb_font_set_ppem(hb_font, pt_size as c_uint, pt_size as c_uint);

            // Set scaling. Note that this takes 16.16 fixed point.
            hb_font_set_scale(hb_font,
                              Shaper::float_to_fixed(pt_size) as c_int,
                              Shaper::float_to_fixed(pt_size) as c_int);

            // configure static function callbacks.
            // NB. This funcs structure could be reused globally, as it never changes.
            let hb_funcs: *hb_font_funcs_t = hb_font_funcs_create();
            hb_font_funcs_set_glyph_func(hb_funcs, glyph_func, null(), None);
            hb_font_funcs_set_glyph_h_advance_func(hb_funcs, glyph_h_advance_func, null(), None);
            hb_font_set_funcs(hb_font, hb_funcs, font_ptr as *c_void, None);

            Shaper {
                font: font,
                hb_face: hb_face,
                hb_font: hb_font,
                hb_funcs: hb_funcs,
            }
        }
    }

    fn float_to_fixed(f: float) -> i32 {
        float_to_fixed(16, f)
    }

    fn fixed_to_float(i: hb_position_t) -> float {
        fixed_to_float(16, i)
    }

    fn fixed_to_rounded_int(f: hb_position_t) -> int {
        fixed_to_rounded_int(16, f)
    }
}

impl ShaperMethods for Shaper {
    /// Calculate the layout metrics associated with the given text when rendered in a specific
    /// font.
    #[fixed_stack_segment]
    fn shape_text(&self, text: &str, glyphs: &mut GlyphStore) {
        unsafe {
            let hb_buffer: *hb_buffer_t = hb_buffer_create();
            hb_buffer_set_direction(hb_buffer, HB_DIRECTION_LTR);

            // Using as_imm_buf because it never does a copy - we don't need the trailing null
            do text.as_imm_buf |ctext: *u8, _: uint| {
                hb_buffer_add_utf8(hb_buffer,
                                   ctext as *c_char,
                                   text.len() as c_int,
                                   0,
                                   text.len() as c_int);
            }

            hb_shape(self.hb_font, hb_buffer, null(), 0);
            self.save_glyph_results(text, glyphs, hb_buffer);
            hb_buffer_destroy(hb_buffer);
        }
    }
}

impl Shaper {
    fn save_glyph_results(&self, text: &str, glyphs: &mut GlyphStore, buffer: *hb_buffer_t) {
        let glyph_data = ShapedGlyphData::new(buffer);
        let glyph_count = glyph_data.len();
        let byte_max = text.len();
        let char_max = text.char_len();

        // GlyphStore records are indexed by character, not byte offset.
        // so, we must be careful to increment this when saving glyph entries.
        let mut char_idx = 0;

        assert!(glyph_count <= char_max);

        debug!("Shaped text[char count=%u], got back %u glyph info records.",
               char_max,
               glyph_count);

        if char_max != glyph_count {
            debug!("NOTE: Since these are not equal, we probably have been given some complex \
                    glyphs.");
        }

        // make map of what chars have glyphs
        let mut byteToGlyph: ~[i32];

        // fast path: all chars are single-byte.
        if byte_max == char_max {
            byteToGlyph = vec::from_elem(byte_max, NO_GLYPH);
        } else {
            byteToGlyph = vec::from_elem(byte_max, CONTINUATION_BYTE);
            let mut i = 0;
            while i < byte_max {
                byteToGlyph[i] = NO_GLYPH;
                let range = text.char_range_at(i);
                ignore(range.ch);
                i = range.next;
            }
        }

        debug!("(glyph idx) -> (text byte offset)");
        for i in range(0, glyph_data.len()) {
            // loc refers to a *byte* offset within the utf8 string.
            let loc = glyph_data.byte_offset_of_glyph(i);
            if loc < byte_max {
                assert!(byteToGlyph[loc] != CONTINUATION_BYTE);
                byteToGlyph[loc] = i as i32;
            } else {
                debug!("ERROR: tried to set out of range byteToGlyph: idx=%u, glyph idx=%u",
                       loc,
                       i);
            }
            debug!("%u -> %u", i, loc);
        }

        debug!("text: %s", text);
        debug!("(char idx): char->(glyph index):");
        let mut i = 0u;
        while i < byte_max {
            let range = text.char_range_at(i);
            debug!("%u: %? --> %d", i, range.ch, byteToGlyph[i] as int);
            i = range.next;
        }

        // some helpers
        let mut glyph_span: Range = Range::empty();
        // this span contains first byte of first char, to last byte of last char in range.
        // so, end() points to first byte of last+1 char, if it's less than byte_max.
        let mut char_byte_span: Range = Range::empty();
        let mut y_pos = Au(0);

        // main loop over each glyph. each iteration usually processes 1 glyph and 1+ chars.
        // in cases with complex glyph-character assocations, 2+ glyphs and 1+ chars can be
        // processed.
        while glyph_span.begin() < glyph_count {
            // start by looking at just one glyph.
            glyph_span.extend_by(1);
            debug!("Processing glyph at idx=%u", glyph_span.begin());

            let char_byte_start = glyph_data.byte_offset_of_glyph(glyph_span.begin());
            char_byte_span.reset(char_byte_start, 0);

            // find a range of chars corresponding to this glyph, plus
            // any trailing chars that do not have associated glyphs.
            while char_byte_span.end() < byte_max {
                let range = text.char_range_at(char_byte_span.end());
                ignore(range.ch);
                char_byte_span.extend_to(range.next);

                debug!("Processing char byte span: off=%u, len=%u for glyph idx=%u",
                       char_byte_span.begin(), char_byte_span.length(), glyph_span.begin());

                while char_byte_span.end() != byte_max &&
                        byteToGlyph[char_byte_span.end()] == NO_GLYPH {
                    debug!("Extending char byte span to include byte offset=%u with no associated \
                            glyph", char_byte_span.end());
                    let range = text.char_range_at(char_byte_span.end());
                    ignore(range.ch);
                    char_byte_span.extend_to(range.next);
                }

                // extend glyph range to max glyph index covered by char_span,
                // in cases where one char made several glyphs and left some unassociated chars.
                let mut max_glyph_idx = glyph_span.end();
                for i in char_byte_span.eachi() {
                    if byteToGlyph[i] > NO_GLYPH {
                        max_glyph_idx = uint::max(byteToGlyph[i] as uint, max_glyph_idx);
                    }
                }

                if max_glyph_idx > glyph_span.end() {
                    glyph_span.extend_to(max_glyph_idx);
                    debug!("Extended glyph span (off=%u, len=%u) to cover char byte span's max \
                            glyph index",
                           glyph_span.begin(), glyph_span.length());
                }


                // if there's just one glyph, then we don't need further checks.
                if glyph_span.length() == 1 { break; }

                // if no glyphs were found yet, extend the char byte range more.
                if glyph_span.length() == 0 { loop; }

                debug!("Complex (multi-glyph to multi-char) association found. This case \
                        probably doesn't work.");

                let mut all_glyphs_are_within_cluster: bool = true;
                for j in glyph_span.eachi() {
                    let loc = glyph_data.byte_offset_of_glyph(j);
                    if !char_byte_span.contains(loc) {
                        all_glyphs_are_within_cluster = false;
                        break
                    }

                    // If true, keep checking. Else, stop.
                    if !all_glyphs_are_within_cluster {
                        break
                    }
                }

                debug!("All glyphs within char_byte_span cluster?: %?",
                       all_glyphs_are_within_cluster);

                // found a valid range; stop extending char_span.
                if all_glyphs_are_within_cluster {
                    break
                }
            }

            // character/glyph clump must contain characters.
            assert!(char_byte_span.length() > 0);
            // character/glyph clump must contain glyphs.
            assert!(glyph_span.length() > 0);

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

            let mut covered_byte_span = char_byte_span.clone();
            // extend, clipping at end of text range.
            while covered_byte_span.end() < byte_max
                    && byteToGlyph[covered_byte_span.end()] == NO_GLYPH {
                let range = text.char_range_at(covered_byte_span.end());
                ignore(range.ch);
                covered_byte_span.extend_to(range.next);
            }

            if covered_byte_span.begin() >= byte_max {
                // oops, out of range. clip and forget this clump.
                let end = glyph_span.end(); // FIXME: borrow checker workaround
                glyph_span.reset(end, 0);
                let end = char_byte_span.end(); // FIXME: borrow checker workaround
                char_byte_span.reset(end, 0);
            }

            // clamp to end of text. (I don't think this will be necessary, but..)
            let end = covered_byte_span.end(); // FIXME: borrow checker workaround
            covered_byte_span.extend_to(uint::min(end, byte_max));

            // fast path: 1-to-1 mapping of single char and single glyph.
            if glyph_span.length() == 1 {
                // TODO(Issue #214): cluster ranges need to be computed before
                // shaping, and then consulted here.
                // for now, just pretend that every character is a cluster start.
                // (i.e., pretend there are no combining character sequences).
                // 1-to-1 mapping of character to glyph also treated as ligature start.
                let shape = glyph_data.get_entry_for_glyph(glyph_span.begin(), &mut y_pos);
                let data = GlyphData::new(shape.codepoint,
                                          shape.advance,
                                          shape.offset,
                                          false,
                                          true,
                                          true);
                glyphs.add_glyph_for_char_index(char_idx, &data);
            } else {
                // collect all glyphs to be assigned to the first character.
                let mut datas = ~[];

                for glyph_i in glyph_span.eachi() {
                    let shape = glyph_data.get_entry_for_glyph(glyph_i, &mut y_pos);
                    datas.push(GlyphData::new(shape.codepoint,
                                              shape.advance,
                                              shape.offset,
                                              false, // not missing
                                              true,  // treat as cluster start
                                              glyph_i > glyph_span.begin()));
                                              // all but first are ligature continuations
                }

                // now add the detailed glyph entry.
                glyphs.add_glyphs_for_char_index(char_idx, datas);

                // set the other chars, who have no glyphs
                let mut i = covered_byte_span.begin();
                loop {
                    let range = text.char_range_at(i);
                    ignore(range.ch);
                    i = range.next;
                    if i >= covered_byte_span.end() { break; }
                    char_idx += 1;
                    glyphs.add_nonglyph_for_char_index(char_idx, false, false);
                }
            }

            // shift up our working spans past things we just handled.
            let end = glyph_span.end(); // FIXME: borrow checker workaround
            glyph_span.reset(end, 0);
            let end = char_byte_span.end();; // FIXME: borrow checker workaround
            char_byte_span.reset(end, 0);
            char_idx += 1;
        }

        // this must be called after adding all glyph data; it sorts the
        // lookup table for finding detailed glyphs by associated char index.
        glyphs.finalize_changes();
    }
}

/// Callbacks from Harfbuzz when font map and glyph advance lookup needed.
extern fn glyph_func(_: *hb_font_t,
                     font_data: *c_void,
                     unicode: hb_codepoint_t,
                     _: hb_codepoint_t,
                     glyph: *mut hb_codepoint_t,
                     _: *c_void)
                  -> hb_bool_t {
    let font: *Font = font_data as *Font;
    assert!(font.is_not_null());

    unsafe {
        match (*font).glyph_index(char::from_u32(unicode).unwrap()) {
            Some(g) => {
                *glyph = g as hb_codepoint_t;
                true as hb_bool_t
            }
            None => false as hb_bool_t
        }
    }
}

extern fn glyph_h_advance_func(_: *hb_font_t,
                               font_data: *c_void,
                               glyph: hb_codepoint_t,
                               _: *c_void)
                            -> hb_position_t {
    let font: *Font = font_data as *Font;
    assert!(font.is_not_null());

    unsafe {
        let advance = (*font).glyph_h_advance(glyph as GlyphIndex);
        Shaper::float_to_fixed(advance)
    }
}

// Callback to get a font table out of a font.
extern fn get_font_table_func(_: *hb_face_t, tag: hb_tag_t, user_data: *c_void) -> *hb_blob_t {
    unsafe {
        let font: *Font = user_data as *Font;
        assert!(font.is_not_null());

        // TODO(Issue #197): reuse font table data, which will change the unsound trickery here.
        match (*font).get_table_for_tag(tag as FontTableTag) {
            None => null(),
            Some(ref font_table) => {
                let skinny_font_table_ptr: *FontTable = font_table;   // private context

                let mut blob: *hb_blob_t = null();
                do (*skinny_font_table_ptr).with_buffer |buf: *u8, len: uint| {
                    // HarfBuzz calls `destroy_blob_func` when the buffer is no longer needed.
                    blob = hb_blob_create(buf as *c_char,
                                          len as c_uint,
                                          HB_MEMORY_MODE_READONLY,
                                          transmute(skinny_font_table_ptr),
                                          destroy_blob_func);
                }

                assert!(blob.is_not_null());
                blob
            }
        }
    }
}

// TODO(Issue #197): reuse font table data, which will change the unsound trickery here.
// In particular, we'll need to cast to a boxed, rather than owned, FontTable.

// even better, should cache the harfbuzz blobs directly instead of recreating a lot.
extern fn destroy_blob_func(_: *c_void) {
    // TODO: Previous code here was broken. Rewrite.
}

