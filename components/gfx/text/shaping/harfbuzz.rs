/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate harfbuzz;

use font::{Font, FontHandleMethods, FontTableMethods, FontTableTag};
use platform::font::FontTable;
use text::glyph::{CharIndex, GlyphStore, GlyphId, GlyphData};
use text::shaping::ShaperMethods;
use text::util::{float_to_fixed, fixed_to_float};

use geom::Point2D;
use harfbuzz::{HB_MEMORY_MODE_READONLY, HB_DIRECTION_LTR};
use harfbuzz::{hb_blob_create, hb_face_create_for_tables};
use harfbuzz::{hb_blob_t};
use harfbuzz::{hb_bool_t};
use harfbuzz::{hb_buffer_add_utf8};
use harfbuzz::{hb_buffer_destroy};
use harfbuzz::{hb_buffer_get_glyph_positions};
use harfbuzz::{hb_buffer_set_direction};
use harfbuzz::{hb_face_destroy};
use harfbuzz::{hb_face_t, hb_font_t};
use harfbuzz::{hb_font_create};
use harfbuzz::{hb_font_destroy, hb_buffer_create};
use harfbuzz::{hb_font_funcs_create};
use harfbuzz::{hb_font_funcs_destroy};
use harfbuzz::{hb_font_funcs_set_glyph_func};
use harfbuzz::{hb_font_funcs_set_glyph_h_advance_func};
use harfbuzz::{hb_font_funcs_set_glyph_h_kerning_func};
use harfbuzz::{hb_font_funcs_t, hb_buffer_t, hb_codepoint_t};
use harfbuzz::{hb_font_set_funcs};
use harfbuzz::{hb_font_set_ppem};
use harfbuzz::{hb_font_set_scale};
use harfbuzz::{hb_glyph_info_t};
use harfbuzz::{hb_glyph_position_t};
use harfbuzz::{hb_position_t, hb_tag_t};
use harfbuzz::{hb_shape, hb_buffer_get_glyph_infos};
use libc::{c_uint, c_int, c_void, c_char};
use servo_util::geometry::Au;
use servo_util::range::Range;
use std::mem;
use std::char;
use std::cmp;
use std::ptr;

static NO_GLYPH: i32 = -1;
static CONTINUATION_BYTE: i32 = -2;

pub struct ShapedGlyphData {
    count: int,
    glyph_infos: *mut hb_glyph_info_t,
    pos_infos: *mut hb_glyph_position_t,
}

pub struct ShapedGlyphEntry {
    codepoint: GlyphId,
    advance: Au,
    offset: Option<Point2D<Au>>,
}

impl ShapedGlyphData {
    pub fn new(buffer: *mut hb_buffer_t) -> ShapedGlyphData {
        unsafe {
            let mut glyph_count = 0;
            let glyph_infos = hb_buffer_get_glyph_infos(buffer, &mut glyph_count);
            let glyph_count = glyph_count as int;
            assert!(glyph_infos.is_not_null());
            let mut pos_count = 0;
            let pos_infos = hb_buffer_get_glyph_positions(buffer, &mut pos_count);
            let pos_count = pos_count as int;
            assert!(pos_infos.is_not_null());
            assert!(glyph_count == pos_count);

            ShapedGlyphData {
                count: glyph_count,
                glyph_infos: glyph_infos,
                pos_infos: pos_infos,
            }
        }
    }

    #[inline(always)]
    fn byte_offset_of_glyph(&self, i: int) -> int {
        assert!(i < self.count);

        unsafe {
            let glyph_info_i = self.glyph_infos.offset(i);
            (*glyph_info_i).cluster as int
        }
    }

    pub fn len(&self) -> int {
        self.count
    }

    /// Returns shaped glyph data for one glyph, and updates the y-position of the pen.
    pub fn get_entry_for_glyph(&self, i: int, y_pos: &mut Au) -> ShapedGlyphEntry {
        assert!(i < self.count);

        unsafe {
            let glyph_info_i = self.glyph_infos.offset(i);
            let pos_info_i = self.pos_infos.offset(i);
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
                codepoint: (*glyph_info_i).codepoint as GlyphId,
                advance: x_advance,
                offset: offset,
            }
        }
    }
}

pub struct Shaper {
    hb_face: *mut hb_face_t,
    hb_font: *mut hb_font_t,
    hb_funcs: *mut hb_font_funcs_t,
}

#[unsafe_destructor]
impl Drop for Shaper {
    fn drop(&mut self) {
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
    pub fn new(font: &mut Font) -> Shaper {
        unsafe {
            // Indirection for Rust Issue #6248, dynamic freeze scope artifically extended
            let font_ptr = font as *mut Font;
            let hb_face: *mut hb_face_t = hb_face_create_for_tables(get_font_table_func,
                                                                    font_ptr as *mut c_void,
                                                                    None);
            let hb_font: *mut hb_font_t = hb_font_create(hb_face);

            // Set points-per-em. if zero, performs no hinting in that direction.
            let pt_size = font.pt_size;
            hb_font_set_ppem(hb_font, pt_size as c_uint, pt_size as c_uint);

            // Set scaling. Note that this takes 16.16 fixed point.
            hb_font_set_scale(hb_font,
                              Shaper::float_to_fixed(pt_size) as c_int,
                              Shaper::float_to_fixed(pt_size) as c_int);

            // configure static function callbacks.
            // NB. This funcs structure could be reused globally, as it never changes.
            let hb_funcs: *mut hb_font_funcs_t = hb_font_funcs_create();
            hb_font_funcs_set_glyph_func(hb_funcs, glyph_func, ptr::null_mut(), None);
            hb_font_funcs_set_glyph_h_advance_func(hb_funcs, glyph_h_advance_func, ptr::null_mut(), None);
            hb_font_funcs_set_glyph_h_kerning_func(hb_funcs, glyph_h_kerning_func, ptr::null_mut(), ptr::null_mut());
            hb_font_set_funcs(hb_font, hb_funcs, font_ptr as *mut c_void, None);

            Shaper {
                hb_face: hb_face,
                hb_font: hb_font,
                hb_funcs: hb_funcs,
            }
        }
    }

    fn float_to_fixed(f: f64) -> i32 {
        float_to_fixed(16, f)
    }

    fn fixed_to_float(i: hb_position_t) -> f64 {
        fixed_to_float(16, i)
    }
}

impl ShaperMethods for Shaper {
    /// Calculate the layout metrics associated with the given text when rendered in a specific
    /// font.
    fn shape_text(&self, text: &str, glyphs: &mut GlyphStore) {
        unsafe {
            let hb_buffer: *mut hb_buffer_t = hb_buffer_create();
            hb_buffer_set_direction(hb_buffer, HB_DIRECTION_LTR);

            hb_buffer_add_utf8(hb_buffer,
                               text.as_ptr() as *const c_char,
                               text.len() as c_int,
                               0,
                               text.len() as c_int);

            hb_shape(self.hb_font, hb_buffer, ptr::null_mut(), 0);
            self.save_glyph_results(text, glyphs, hb_buffer);
            hb_buffer_destroy(hb_buffer);
        }
    }
}

impl Shaper {
    fn save_glyph_results(&self, text: &str, glyphs: &mut GlyphStore, buffer: *mut hb_buffer_t) {
        let glyph_data = ShapedGlyphData::new(buffer);
        let glyph_count = glyph_data.len();
        let byte_max = text.len() as int;
        let char_max = text.char_len() as int;

        // GlyphStore records are indexed by character, not byte offset.
        // so, we must be careful to increment this when saving glyph entries.
        let mut char_idx = CharIndex(0);

        assert!(glyph_count <= char_max);

        debug!("Shaped text[char count={}], got back {} glyph info records.",
               char_max,
               glyph_count);

        if char_max != glyph_count {
            debug!("NOTE: Since these are not equal, we probably have been given some complex \
                    glyphs.");
        }

        // make map of what chars have glyphs
        let mut byte_to_glyph: Vec<i32>;

        // fast path: all chars are single-byte.
        if byte_max == char_max {
            byte_to_glyph = Vec::from_elem(byte_max as uint, NO_GLYPH);
        } else {
            byte_to_glyph = Vec::from_elem(byte_max as uint, CONTINUATION_BYTE);
            for (i, _) in text.char_indices() {
                *byte_to_glyph.get_mut(i) = NO_GLYPH;
            }
        }

        debug!("(glyph idx) -> (text byte offset)");
        for i in range(0, glyph_data.len()) {
            // loc refers to a *byte* offset within the utf8 string.
            let loc = glyph_data.byte_offset_of_glyph(i);
            if loc < byte_max {
                assert!(*byte_to_glyph.get(loc as uint) != CONTINUATION_BYTE);
                *byte_to_glyph.get_mut(loc as uint) = i as i32;
            } else {
                debug!("ERROR: tried to set out of range byte_to_glyph: idx={}, glyph idx={}",
                       loc,
                       i);
            }
            debug!("{} -> {}", i, loc);
        }

        debug!("text: {:s}", text);
        debug!("(char idx): char->(glyph index):");
        for (i, ch) in text.char_indices() {
            debug!("{}: {} --> {:d}", i, ch, *byte_to_glyph.get(i) as int);
        }

        // some helpers
        let mut glyph_span: Range<int> = Range::empty();
        // this span contains first byte of first char, to last byte of last char in range.
        // so, end() points to first byte of last+1 char, if it's less than byte_max.
        let mut char_byte_span: Range<int> = Range::empty();
        let mut y_pos = Au(0);

        // main loop over each glyph. each iteration usually processes 1 glyph and 1+ chars.
        // in cases with complex glyph-character assocations, 2+ glyphs and 1+ chars can be
        // processed.
        while glyph_span.begin() < glyph_count {
            // start by looking at just one glyph.
            glyph_span.extend_by(1);
            debug!("Processing glyph at idx={}", glyph_span.begin());

            let char_byte_start = glyph_data.byte_offset_of_glyph(glyph_span.begin());
            char_byte_span.reset(char_byte_start, 0);

            // find a range of chars corresponding to this glyph, plus
            // any trailing chars that do not have associated glyphs.
            while char_byte_span.end() < byte_max {
                let range = text.char_range_at(char_byte_span.end() as uint);
                drop(range.ch);
                char_byte_span.extend_to(range.next as int);

                debug!("Processing char byte span: off={}, len={} for glyph idx={}",
                       char_byte_span.begin(), char_byte_span.length(), glyph_span.begin());

                while char_byte_span.end() != byte_max &&
                        byte_to_glyph[char_byte_span.end() as uint] == NO_GLYPH {
                    debug!("Extending char byte span to include byte offset={} with no associated \
                            glyph", char_byte_span.end());
                    let range = text.char_range_at(char_byte_span.end() as uint);
                    drop(range.ch);
                    char_byte_span.extend_to(range.next as int);
                }

                // extend glyph range to max glyph index covered by char_span,
                // in cases where one char made several glyphs and left some unassociated chars.
                let mut max_glyph_idx = glyph_span.end();
                for i in char_byte_span.each_index() {
                    if byte_to_glyph[i as uint] > NO_GLYPH {
                        max_glyph_idx = cmp::max(byte_to_glyph[i as uint] as int + 1, max_glyph_idx);
                    }
                }

                if max_glyph_idx > glyph_span.end() {
                    glyph_span.extend_to(max_glyph_idx);
                    debug!("Extended glyph span (off={}, len={}) to cover char byte span's max \
                            glyph index",
                           glyph_span.begin(), glyph_span.length());
                }


                // if there's just one glyph, then we don't need further checks.
                if glyph_span.length() == 1 { break; }

                // if no glyphs were found yet, extend the char byte range more.
                if glyph_span.length() == 0 { continue; }

                debug!("Complex (multi-glyph to multi-char) association found. This case \
                        probably doesn't work.");

                let mut all_glyphs_are_within_cluster: bool = true;
                for j in glyph_span.each_index() {
                    let loc = glyph_data.byte_offset_of_glyph(j);
                    if !char_byte_span.contains(loc) {
                        all_glyphs_are_within_cluster = false;
                        break
                    }
                }

                debug!("All glyphs within char_byte_span cluster?: {}",
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
                    && byte_to_glyph[covered_byte_span.end() as uint] == NO_GLYPH {
                let range = text.char_range_at(covered_byte_span.end() as uint);
                drop(range.ch);
                covered_byte_span.extend_to(range.next as int);
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
            covered_byte_span.extend_to(cmp::min(end, byte_max));

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
                let mut datas = vec!();

                for glyph_i in glyph_span.each_index() {
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
                glyphs.add_glyphs_for_char_index(char_idx, datas.as_slice());

                // set the other chars, who have no glyphs
                let mut i = covered_byte_span.begin();
                loop {
                    let range = text.char_range_at(i as uint);
                    drop(range.ch);
                    i = range.next as int;
                    if i >= covered_byte_span.end() { break; }
                    char_idx = char_idx + CharIndex(1);
                    glyphs.add_nonglyph_for_char_index(char_idx, false, false);
                }
            }

            // shift up our working spans past things we just handled.
            let end = glyph_span.end(); // FIXME: borrow checker workaround
            glyph_span.reset(end, 0);
            let end = char_byte_span.end();; // FIXME: borrow checker workaround
            char_byte_span.reset(end, 0);
            char_idx = char_idx + CharIndex(1);
        }

        // this must be called after adding all glyph data; it sorts the
        // lookup table for finding detailed glyphs by associated char index.
        glyphs.finalize_changes();
    }
}

/// Callbacks from Harfbuzz when font map and glyph advance lookup needed.
extern fn glyph_func(_: *mut hb_font_t,
                     font_data: *mut c_void,
                     unicode: hb_codepoint_t,
                     _: hb_codepoint_t,
                     glyph: *mut hb_codepoint_t,
                     _: *mut c_void)
                  -> hb_bool_t {
    let font: *const Font = font_data as *const Font;
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

extern fn glyph_h_advance_func(_: *mut hb_font_t,
                               font_data: *mut c_void,
                               glyph: hb_codepoint_t,
                               _: *mut c_void)
                            -> hb_position_t {
    let font: *mut Font = font_data as *mut Font;
    assert!(font.is_not_null());

    unsafe {
        let advance = (*font).glyph_h_advance(glyph as GlyphId);
        Shaper::float_to_fixed(advance)
    }
}

extern fn glyph_h_kerning_func(_: *mut hb_font_t,
                               font_data: *mut c_void,
                               first_glyph: hb_codepoint_t,
                               second_glyph: hb_codepoint_t,
                               _: *mut c_void)
                            -> hb_position_t {
    let font: *mut Font = font_data as *mut Font;
    assert!(font.is_not_null());

    unsafe {
        let advance = (*font).glyph_h_kerning(first_glyph as GlyphId, second_glyph as GlyphId);
        Shaper::float_to_fixed(advance)
    }
}

// Callback to get a font table out of a font.
extern fn get_font_table_func(_: *mut hb_face_t, tag: hb_tag_t, user_data: *mut c_void) -> *mut hb_blob_t {
    unsafe {
        let font: *const Font = user_data as *const Font;
        assert!(font.is_not_null());

        // TODO(Issue #197): reuse font table data, which will change the unsound trickery here.
        match (*font).get_table_for_tag(tag as FontTableTag) {
            None => ptr::null_mut(),
            Some(ref font_table) => {
                let skinny_font_table_ptr: *const FontTable = font_table;   // private context

                let mut blob: *mut hb_blob_t = ptr::null_mut();
                (*skinny_font_table_ptr).with_buffer(|buf: *const u8, len: uint| {
                    // HarfBuzz calls `destroy_blob_func` when the buffer is no longer needed.
                    blob = hb_blob_create(buf as *const c_char,
                                          len as c_uint,
                                          HB_MEMORY_MODE_READONLY,
                                          mem::transmute(skinny_font_table_ptr),
                                          destroy_blob_func);
                });

                assert!(blob.is_not_null());
                blob
            }
        }
    }
}

// TODO(Issue #197): reuse font table data, which will change the unsound trickery here.
// In particular, we'll need to cast to a boxed, rather than owned, FontTable.

// even better, should cache the harfbuzz blobs directly instead of recreating a lot.
extern fn destroy_blob_func(_: *mut c_void) {
    // TODO: Previous code here was broken. Rewrite.
}
