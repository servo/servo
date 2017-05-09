/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use app_units::Au;
use euclid::Point2D;
use font::{DISABLE_KERNING_SHAPING_FLAG, Font, FontTableMethods, FontTableTag};
use font::{IGNORE_LIGATURES_SHAPING_FLAG, KERN, RTL_FLAG, ShapingOptions};
use harfbuzz::{HB_DIRECTION_LTR, HB_DIRECTION_RTL, HB_MEMORY_MODE_READONLY};
use harfbuzz::{hb_blob_create, hb_face_create_for_tables};
use harfbuzz::{hb_buffer_create, hb_font_destroy};
use harfbuzz::{hb_buffer_get_glyph_infos, hb_shape};
use harfbuzz::{hb_buffer_set_direction, hb_buffer_set_script};
use harfbuzz::{hb_buffer_t, hb_codepoint_t, hb_font_funcs_t};
use harfbuzz::{hb_face_t, hb_font_t};
use harfbuzz::{hb_position_t, hb_tag_t};
use harfbuzz::hb_blob_t;
use harfbuzz::hb_bool_t;
use harfbuzz::hb_buffer_add_utf8;
use harfbuzz::hb_buffer_destroy;
use harfbuzz::hb_buffer_get_glyph_positions;
use harfbuzz::hb_buffer_get_length;
use harfbuzz::hb_face_destroy;
use harfbuzz::hb_feature_t;
use harfbuzz::hb_font_create;
use harfbuzz::hb_font_funcs_create;
use harfbuzz::hb_font_funcs_set_glyph_func;
use harfbuzz::hb_font_funcs_set_glyph_h_advance_func;
use harfbuzz::hb_font_funcs_set_glyph_h_kerning_func;
use harfbuzz::hb_font_set_funcs;
use harfbuzz::hb_font_set_ppem;
use harfbuzz::hb_font_set_scale;
use harfbuzz::hb_glyph_info_t;
use harfbuzz::hb_glyph_position_t;
use libc::{c_char, c_int, c_uint, c_void};
use platform::font::FontTable;
use std::{char, cmp, ptr};
use text::glyph::{ByteIndex, GlyphData, GlyphId, GlyphStore};
use text::shaping::ShaperMethods;
use text::util::{fixed_to_float, float_to_fixed, is_bidi_control};

const NO_GLYPH: i32 = -1;
const LIGA: u32 = ot_tag!('l', 'i', 'g', 'a');

pub struct ShapedGlyphData {
    count: usize,
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
            assert!(!glyph_infos.is_null());
            let mut pos_count = 0;
            let pos_infos = hb_buffer_get_glyph_positions(buffer, &mut pos_count);
            assert!(!pos_infos.is_null());
            assert!(glyph_count == pos_count);

            ShapedGlyphData {
                count: glyph_count as usize,
                glyph_infos: glyph_infos,
                pos_infos: pos_infos,
            }
        }
    }

    #[inline(always)]
    fn byte_offset_of_glyph(&self, i: usize) -> u32 {
        assert!(i < self.count);

        unsafe {
            let glyph_info_i = self.glyph_infos.offset(i as isize);
            (*glyph_info_i).cluster
        }
    }

    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns shaped glyph data for one glyph, and updates the y-position of the pen.
    pub fn entry_for_glyph(&self, i: usize, y_pos: &mut Au) -> ShapedGlyphEntry {
        assert!(i < self.count);

        unsafe {
            let glyph_info_i = self.glyph_infos.offset(i as isize);
            let pos_info_i = self.pos_infos.offset(i as isize);
            let x_offset = Shaper::fixed_to_float((*pos_info_i).x_offset);
            let y_offset = Shaper::fixed_to_float((*pos_info_i).y_offset);
            let x_advance = Shaper::fixed_to_float((*pos_info_i).x_advance);
            let y_advance = Shaper::fixed_to_float((*pos_info_i).y_advance);

            let x_offset = Au::from_f64_px(x_offset);
            let y_offset = Au::from_f64_px(y_offset);
            let x_advance = Au::from_f64_px(x_advance);
            let y_advance = Au::from_f64_px(y_advance);

            let offset = if x_offset == Au(0) && y_offset == Au(0) && y_advance == Au(0) {
                None
            } else {
                // adjust the pen..
                if y_advance > Au(0) {
                    *y_pos = *y_pos - y_advance;
                }

                Some(Point2D::new(x_offset, *y_pos - y_offset))
            };

            ShapedGlyphEntry {
                codepoint: (*glyph_info_i).codepoint as GlyphId,
                advance: x_advance,
                offset: offset,
            }
        }
    }
}

#[derive(Debug)]
pub struct Shaper {
    hb_face: *mut hb_face_t,
    hb_font: *mut hb_font_t,
    font: *const Font,
}

impl Drop for Shaper {
    fn drop(&mut self) {
        unsafe {
            assert!(!self.hb_face.is_null());
            hb_face_destroy(self.hb_face);

            assert!(!self.hb_font.is_null());
            hb_font_destroy(self.hb_font);
        }
    }
}

impl Shaper {
    pub fn new(font: *const Font) -> Shaper {
        unsafe {
            let hb_face: *mut hb_face_t =
                hb_face_create_for_tables(Some(font_table_func),
                                          font as *const c_void as *mut c_void,
                                          None);
            let hb_font: *mut hb_font_t = hb_font_create(hb_face);

            // Set points-per-em. if zero, performs no hinting in that direction.
            let pt_size = (*font).actual_pt_size.to_f64_px();
            hb_font_set_ppem(hb_font, pt_size as c_uint, pt_size as c_uint);

            // Set scaling. Note that this takes 16.16 fixed point.
            hb_font_set_scale(hb_font,
                              Shaper::float_to_fixed(pt_size) as c_int,
                              Shaper::float_to_fixed(pt_size) as c_int);

            // configure static function callbacks.
            hb_font_set_funcs(hb_font, HB_FONT_FUNCS.as_ptr(), font as *mut Font as *mut c_void, None);

            Shaper {
                hb_face: hb_face,
                hb_font: hb_font,
                font: font,
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
    /// Calculate the layout metrics associated with the given text when painted in a specific
    /// font.
    fn shape_text(&self, text: &str, options: &ShapingOptions, glyphs: &mut GlyphStore) {
        unsafe {
            let hb_buffer: *mut hb_buffer_t = hb_buffer_create();
            hb_buffer_set_direction(hb_buffer, if options.flags.contains(RTL_FLAG) {
                HB_DIRECTION_RTL
            } else {
                HB_DIRECTION_LTR
            });

            hb_buffer_set_script(hb_buffer, options.script.to_hb_script());

            hb_buffer_add_utf8(hb_buffer,
                               text.as_ptr() as *const c_char,
                               text.len() as c_int,
                               0,
                               text.len() as c_int);

            let mut features = Vec::new();
            if options.flags.contains(IGNORE_LIGATURES_SHAPING_FLAG) {
                features.push(hb_feature_t {
                    tag: LIGA,
                    value: 0,
                    start: 0,
                    end: hb_buffer_get_length(hb_buffer),
                })
            }
            if options.flags.contains(DISABLE_KERNING_SHAPING_FLAG) {
                features.push(hb_feature_t {
                    tag: KERN,
                    value: 0,
                    start: 0,
                    end: hb_buffer_get_length(hb_buffer),
                })
            }

            hb_shape(self.hb_font, hb_buffer, features.as_mut_ptr(), features.len() as u32);
            self.save_glyph_results(text, options, glyphs, hb_buffer);
            hb_buffer_destroy(hb_buffer);
        }
    }
}

impl Shaper {
    fn save_glyph_results(&self,
                          text: &str,
                          options: &ShapingOptions,
                          glyphs: &mut GlyphStore,
                          buffer: *mut hb_buffer_t) {
        let glyph_data = ShapedGlyphData::new(buffer);
        let glyph_count = glyph_data.len();
        let byte_max = text.len();

        debug!("Shaped text[byte count={}], got back {} glyph info records.",
               byte_max,
               glyph_count);

        // make map of what chars have glyphs
        let mut byte_to_glyph = vec![NO_GLYPH; byte_max];

        debug!("(glyph idx) -> (text byte offset)");
        for i in 0..glyph_data.len() {
            let loc = glyph_data.byte_offset_of_glyph(i) as usize;
            if loc < byte_max {
                byte_to_glyph[loc] = i as i32;
            } else {
                debug!("ERROR: tried to set out of range byte_to_glyph: idx={}, glyph idx={}",
                       loc,
                       i);
            }
            debug!("{} -> {}", i, loc);
        }

        debug!("text: {:?}", text);
        debug!("(char idx): char->(glyph index):");
        for (i, ch) in text.char_indices() {
            debug!("{}: {:?} --> {}", i, ch, byte_to_glyph[i]);
        }

        let mut glyph_span = 0..0;
        let mut byte_range = 0..0;

        let mut y_pos = Au(0);

        // main loop over each glyph. each iteration usually processes 1 glyph and 1+ chars.
        // in cases with complex glyph-character associations, 2+ glyphs and 1+ chars can be
        // processed.
        while glyph_span.start < glyph_count {
            debug!("Processing glyph at idx={}", glyph_span.start);
            glyph_span.end = glyph_span.start;
            byte_range.end = glyph_data.byte_offset_of_glyph(glyph_span.start) as usize;

            while byte_range.end < byte_max {
                byte_range.end += 1;
                // Extend the byte range to include any following byte without its own glyph.
                while byte_range.end < byte_max && byte_to_glyph[byte_range.end] == NO_GLYPH {
                    byte_range.end += 1;
                }

                // Extend the glyph range to include all glyphs covered by bytes processed so far.
                let mut max_glyph_idx = glyph_span.end;
                for glyph_idx in &byte_to_glyph[byte_range.clone()] {
                    if *glyph_idx != NO_GLYPH {
                        max_glyph_idx = cmp::max(*glyph_idx as usize + 1, max_glyph_idx);
                    }
                }
                if max_glyph_idx > glyph_span.end {
                    glyph_span.end = max_glyph_idx;
                    debug!("Extended glyph span to {:?}", glyph_span);
                }

                // if there's just one glyph, then we don't need further checks.
                if glyph_span.len() == 1 { break; }

                // if no glyphs were found yet, extend the char byte range more.
                if glyph_span.len() == 0 { continue; }

                // If byte_range now includes all the byte offsets found in glyph_span, then we
                // have found a contiguous "cluster" and can stop extending it.
                let mut all_glyphs_are_within_cluster: bool = true;
                for j in glyph_span.clone() {
                    let loc = glyph_data.byte_offset_of_glyph(j);
                    if !byte_range.contains(loc as usize) {
                        all_glyphs_are_within_cluster = false;
                        break
                    }
                }
                if all_glyphs_are_within_cluster {
                    break
                }

                // Otherwise, the bytes we have seen so far correspond to a non-contiguous set of
                // glyphs.  Keep extending byte_range until we fill in all the holes in the glyph
                // span or reach the end of the text.
            }

            assert!(byte_range.len() > 0);
            assert!(glyph_span.len() > 0);

            // Now byte_range is the ligature clump formed by the glyphs in glyph_span.
            // We will save these glyphs to the glyph store at the index of the first byte.
            let byte_idx = ByteIndex(byte_range.start as isize);

            if glyph_span.len() == 1 {
                // Fast path: 1-to-1 mapping of byte offset to single glyph.
                //
                // TODO(Issue #214): cluster ranges need to be computed before
                // shaping, and then consulted here.
                // for now, just pretend that every character is a cluster start.
                // (i.e., pretend there are no combining character sequences).
                // 1-to-1 mapping of character to glyph also treated as ligature start.
                //
                // NB: When we acquire the ability to handle ligatures that cross word boundaries,
                // we'll need to do something special to handle `word-spacing` properly.
                let character = text[byte_range.clone()].chars().next().unwrap();
                if is_bidi_control(character) {
                    // Don't add any glyphs for bidi control chars
                } else if character == '\t' {
                    // Treat tabs in pre-formatted text as a fixed number of spaces.
                    //
                    // TODO: Proper tab stops.
                    const TAB_COLS: i32 = 8;
                    let (space_glyph_id, space_advance) = glyph_space_advance(self.font);
                    let advance = Au::from_f64_px(space_advance) * TAB_COLS;
                    let data = GlyphData::new(space_glyph_id,
                                              advance,
                                              Default::default(),
                                              true,
                                              true);
                    glyphs.add_glyph_for_byte_index(byte_idx, character, &data);
                } else {
                    let shape = glyph_data.entry_for_glyph(glyph_span.start, &mut y_pos);
                    let advance = self.advance_for_shaped_glyph(shape.advance, character, options);
                    let data = GlyphData::new(shape.codepoint,
                                              advance,
                                              shape.offset,
                                              true,
                                              true);
                    glyphs.add_glyph_for_byte_index(byte_idx, character, &data);
                }
            } else {
                // collect all glyphs to be assigned to the first character.
                let mut datas = vec!();

                for glyph_i in glyph_span.clone() {
                    let shape = glyph_data.entry_for_glyph(glyph_i, &mut y_pos);
                    datas.push(GlyphData::new(shape.codepoint,
                                              shape.advance,
                                              shape.offset,
                                              true,  // treat as cluster start
                                              glyph_i > glyph_span.start));
                                              // all but first are ligature continuations
                }
                // now add the detailed glyph entry.
                glyphs.add_glyphs_for_byte_index(byte_idx, &datas);
            }

            glyph_span.start = glyph_span.end;
            byte_range.start = byte_range.end;
        }

        // this must be called after adding all glyph data; it sorts the
        // lookup table for finding detailed glyphs by associated char index.
        glyphs.finalize_changes();
    }

    fn advance_for_shaped_glyph(&self, mut advance: Au, character: char, options: &ShapingOptions)
                                -> Au {
        if let Some(letter_spacing) = options.letter_spacing {
            advance = advance + letter_spacing;
        };

        // CSS 2.1 § 16.4 states that "word spacing affects each space (U+0020) and non-breaking
        // space (U+00A0) left in the text after the white space processing rules have been
        // applied. The effect of the property on other word-separator characters is undefined."
        // We elect to only space the two required code points.
        if character == ' ' || character == '\u{a0}' {
            // https://drafts.csswg.org/css-text-3/#word-spacing-property
            let (length, percent) = options.word_spacing;
            advance = (advance + length) + Au((advance.0 as f32 * percent.into_inner()) as i32);
        }

        advance
    }
}

// Callbacks from Harfbuzz when font map and glyph advance lookup needed.
lazy_static! {
    static ref HB_FONT_FUNCS: ptr::Unique<hb_font_funcs_t> = unsafe {
        let hb_funcs = hb_font_funcs_create();
        hb_font_funcs_set_glyph_func(hb_funcs, Some(glyph_func), ptr::null_mut(), None);
        hb_font_funcs_set_glyph_h_advance_func(
            hb_funcs, Some(glyph_h_advance_func), ptr::null_mut(), None);
        hb_font_funcs_set_glyph_h_kerning_func(
            hb_funcs, Some(glyph_h_kerning_func), ptr::null_mut(), None);

        ptr::Unique::new(hb_funcs)
    };
}

extern fn glyph_func(_: *mut hb_font_t,
                     font_data: *mut c_void,
                     unicode: hb_codepoint_t,
                     _: hb_codepoint_t,
                     glyph: *mut hb_codepoint_t,
                     _: *mut c_void)
                  -> hb_bool_t {
    let font: *const Font = font_data as *const Font;
    assert!(!font.is_null());

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
    assert!(!font.is_null());

    unsafe {
        let advance = (*font).glyph_h_advance(glyph as GlyphId);
        Shaper::float_to_fixed(advance)
    }
}

fn glyph_space_advance(font: *const Font) -> (hb_codepoint_t, f64) {
    let space_unicode = ' ';
    let space_glyph: hb_codepoint_t;
    match unsafe { (*font).glyph_index(space_unicode) } {
        Some(g) => {
            space_glyph = g as hb_codepoint_t;
        }
        None => panic!("No space info")
    }
    let space_advance = unsafe { (*font).glyph_h_advance(space_glyph as GlyphId) };
    (space_glyph, space_advance)
}

extern fn glyph_h_kerning_func(_: *mut hb_font_t,
                               font_data: *mut c_void,
                               first_glyph: hb_codepoint_t,
                               second_glyph: hb_codepoint_t,
                               _: *mut c_void)
                            -> hb_position_t {
    let font: *mut Font = font_data as *mut Font;
    assert!(!font.is_null());

    unsafe {
        let advance = (*font).glyph_h_kerning(first_glyph as GlyphId, second_glyph as GlyphId);
        Shaper::float_to_fixed(advance)
    }
}

// Callback to get a font table out of a font.
extern fn font_table_func(_: *mut hb_face_t,
                              tag: hb_tag_t,
                              user_data: *mut c_void)
                              -> *mut hb_blob_t {
    unsafe {
        // NB: These asserts have security implications.
        let font = user_data as *const Font;
        assert!(!font.is_null());

        // TODO(Issue #197): reuse font table data, which will change the unsound trickery here.
        match (*font).table_for_tag(tag as FontTableTag) {
            None => ptr::null_mut(),
            Some(font_table) => {
                // `Box::into_raw` intentionally leaks the FontTable so we don't destroy the buffer
                // while HarfBuzz is using it.  When HarfBuzz is done with the buffer, it will pass
                // this raw pointer back to `destroy_blob_func` which will deallocate the Box.
                let font_table_ptr = Box::into_raw(box font_table);

                let buf = (*font_table_ptr).buffer();
                // HarfBuzz calls `destroy_blob_func` when the buffer is no longer needed.
                let blob = hb_blob_create(buf.as_ptr() as *const c_char,
                                          buf.len() as c_uint,
                                          HB_MEMORY_MODE_READONLY,
                                          font_table_ptr as *mut c_void,
                                          Some(destroy_blob_func));

                assert!(!blob.is_null());
                blob
            }
        }
    }
}

extern fn destroy_blob_func(font_table_ptr: *mut c_void) {
    unsafe {
        drop(Box::from_raw(font_table_ptr as *mut FontTable));
    }
}
