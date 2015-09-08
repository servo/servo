/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate harfbuzz;

use font::{DISABLE_KERNING_SHAPING_FLAG, Font, FontHandleMethods, FontTableMethods, FontTableTag};
use font::{IGNORE_LIGATURES_SHAPING_FLAG, RTL_FLAG, ShapingOptions};
use platform::font::FontTable;
use text::glyph::{CharIndex, GlyphStore, GlyphId, GlyphData};
use text::shaping::ShaperMethods;
use text::util::{float_to_fixed, fixed_to_float, is_bidi_control};

use euclid::Point2D;
use harfbuzz::{HB_MEMORY_MODE_READONLY, HB_DIRECTION_LTR, HB_DIRECTION_RTL};
use harfbuzz::{RUST_hb_blob_create, RUST_hb_face_create_for_tables};
use harfbuzz::{RUST_hb_buffer_add_utf8};
use harfbuzz::{RUST_hb_buffer_destroy};
use harfbuzz::{RUST_hb_buffer_get_glyph_positions};
use harfbuzz::{RUST_hb_buffer_get_length};
use harfbuzz::{RUST_hb_buffer_set_direction};
use harfbuzz::{RUST_hb_face_destroy};
use harfbuzz::{RUST_hb_font_create};
use harfbuzz::{RUST_hb_font_destroy, RUST_hb_buffer_create};
use harfbuzz::{RUST_hb_font_funcs_create};
use harfbuzz::{RUST_hb_font_funcs_set_glyph_func};
use harfbuzz::{RUST_hb_font_funcs_set_glyph_h_advance_func};
use harfbuzz::{RUST_hb_font_funcs_set_glyph_h_kerning_func};
use harfbuzz::{RUST_hb_font_set_funcs};
use harfbuzz::{RUST_hb_font_set_ppem};
use harfbuzz::{RUST_hb_font_set_scale};
use harfbuzz::{RUST_hb_shape, RUST_hb_buffer_get_glyph_infos};
use harfbuzz::{hb_blob_t};
use harfbuzz::{hb_bool_t};
use harfbuzz::{hb_face_t, hb_font_t};
use harfbuzz::{hb_feature_t};
use harfbuzz::{hb_font_funcs_t, hb_buffer_t, hb_codepoint_t};
use harfbuzz::{hb_glyph_info_t};
use harfbuzz::{hb_glyph_position_t};
use harfbuzz::{hb_position_t, hb_tag_t};
use libc::{c_uint, c_int, c_void, c_char};
use std::char;
use std::cmp;
use std::ptr;
use util::geometry::Au;
use util::range::Range;

macro_rules! hb_tag {
    ($t1:expr, $t2:expr, $t3:expr, $t4:expr) => (
        (($t1 as u32) << 24) | (($t2 as u32) << 16) | (($t3 as u32) << 8) | ($t4 as u32)
    );
}

static NO_GLYPH: i32 = -1;
static CONTINUATION_BYTE: i32 = -2;

static KERN: u32 = hb_tag!('k', 'e', 'r', 'n');
static LIGA: u32 = hb_tag!('l', 'i', 'g', 'a');

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
            let glyph_infos = RUST_hb_buffer_get_glyph_infos(buffer, &mut glyph_count);
            assert!(!glyph_infos.is_null());
            let mut pos_count = 0;
            let pos_infos = RUST_hb_buffer_get_glyph_positions(buffer, &mut pos_count);
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

struct FontAndShapingOptions {
    font: *mut Font,
    options: ShapingOptions,
}

pub struct Shaper {
    hb_face: *mut hb_face_t,
    hb_font: *mut hb_font_t,
    font_and_shaping_options: Box<FontAndShapingOptions>,
}

impl Drop for Shaper {
    fn drop(&mut self) {
        unsafe {
            assert!(!self.hb_face.is_null());
            RUST_hb_face_destroy(self.hb_face);

            assert!(!self.hb_font.is_null());
            RUST_hb_font_destroy(self.hb_font);
        }
    }
}

impl Shaper {
    pub fn new(font: &mut Font, options: &ShapingOptions) -> Shaper {
        unsafe {
            let mut font_and_shaping_options = box FontAndShapingOptions {
                font: font,
                options: *options,
            };
            let hb_face: *mut hb_face_t =
                RUST_hb_face_create_for_tables(font_table_func,
                                          (&mut *font_and_shaping_options)
                                            as *mut FontAndShapingOptions
                                            as *mut c_void,
                                          None);
            let hb_font: *mut hb_font_t = RUST_hb_font_create(hb_face);

            // Set points-per-em. if zero, performs no hinting in that direction.
            let pt_size = font.actual_pt_size.to_f64_px();
            RUST_hb_font_set_ppem(hb_font, pt_size as c_uint, pt_size as c_uint);

            // Set scaling. Note that this takes 16.16 fixed point.
            RUST_hb_font_set_scale(hb_font,
                                   Shaper::float_to_fixed(pt_size) as c_int,
                                   Shaper::float_to_fixed(pt_size) as c_int);

            // configure static function callbacks.
            RUST_hb_font_set_funcs(hb_font, **HB_FONT_FUNCS, font as *mut Font as *mut c_void, None);

            Shaper {
                hb_face: hb_face,
                hb_font: hb_font,
                font_and_shaping_options: font_and_shaping_options,
            }
        }
    }

    pub fn set_options(&mut self, options: &ShapingOptions) {
        self.font_and_shaping_options.options = *options
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
            let hb_buffer: *mut hb_buffer_t = RUST_hb_buffer_create();
            RUST_hb_buffer_set_direction(hb_buffer, if options.flags.contains(RTL_FLAG) {
                HB_DIRECTION_RTL
            } else {
                HB_DIRECTION_LTR
            });

            RUST_hb_buffer_add_utf8(hb_buffer,
                                    text.as_ptr() as *const c_char,
                                    text.len() as c_int,
                                    0,
                                    text.len() as c_int);

            let mut features = Vec::new();
            if options.flags.contains(IGNORE_LIGATURES_SHAPING_FLAG) {
                features.push(hb_feature_t {
                    _tag: LIGA,
                    _value: 0,
                    _start: 0,
                    _end: RUST_hb_buffer_get_length(hb_buffer),
                })
            }
            if options.flags.contains(DISABLE_KERNING_SHAPING_FLAG) {
                features.push(hb_feature_t {
                    _tag: KERN,
                    _value: 0,
                    _start: 0,
                    _end: RUST_hb_buffer_get_length(hb_buffer),
                })
            }

            RUST_hb_shape(self.hb_font, hb_buffer, features.as_mut_ptr(), features.len() as u32);
            self.save_glyph_results(text, options, glyphs, hb_buffer);
            RUST_hb_buffer_destroy(hb_buffer);
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
        let char_max = text.chars().count();

        // GlyphStore records are indexed by character, not byte offset.
        // so, we must be careful to increment this when saving glyph entries.
        let (mut char_idx, char_step) = if options.flags.contains(RTL_FLAG) {
            (CharIndex(char_max as isize - 1), CharIndex(-1))
        } else {
            (CharIndex(0), CharIndex(1))
        };

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
            byte_to_glyph = vec![NO_GLYPH; byte_max];
        } else {
            byte_to_glyph = vec![CONTINUATION_BYTE; byte_max];
            for (i, _) in text.char_indices() {
                byte_to_glyph[i] = NO_GLYPH;
            }
        }

        debug!("(glyph idx) -> (text byte offset)");
        for i in 0..glyph_data.len() {
            // loc refers to a *byte* offset within the utf8 string.
            let loc = glyph_data.byte_offset_of_glyph(i) as usize;
            if loc < byte_max {
                assert!(byte_to_glyph[loc] != CONTINUATION_BYTE);
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

        // some helpers
        let mut glyph_span: Range<usize> = Range::empty();
        // this span contains first byte of first char, to last byte of last char in range.
        // so, end() points to first byte of last+1 char, if it's less than byte_max.
        let mut char_byte_span: Range<usize> = Range::empty();
        let mut y_pos = Au(0);

        // main loop over each glyph. each iteration usually processes 1 glyph and 1+ chars.
        // in cases with complex glyph-character associations, 2+ glyphs and 1+ chars can be
        // processed.
        while glyph_span.begin() < glyph_count {
            // start by looking at just one glyph.
            glyph_span.extend_by(1);
            debug!("Processing glyph at idx={}", glyph_span.begin());

            let char_byte_start = glyph_data.byte_offset_of_glyph(glyph_span.begin());
            char_byte_span.reset(char_byte_start as usize, 0);

            // find a range of chars corresponding to this glyph, plus
            // any trailing chars that do not have associated glyphs.
            while char_byte_span.end() < byte_max {
                let range = text.char_range_at(char_byte_span.end());
                drop(range.ch);
                char_byte_span.extend_to(range.next);

                debug!("Processing char byte span: off={}, len={} for glyph idx={}",
                       char_byte_span.begin(), char_byte_span.length(), glyph_span.begin());

                while char_byte_span.end() != byte_max &&
                        byte_to_glyph[char_byte_span.end()] == NO_GLYPH {
                    debug!("Extending char byte span to include byte offset={} with no associated \
                            glyph", char_byte_span.end());
                    let range = text.char_range_at(char_byte_span.end());
                    drop(range.ch);
                    char_byte_span.extend_to(range.next);
                }

                // extend glyph range to max glyph index covered by char_span,
                // in cases where one char made several glyphs and left some unassociated chars.
                let mut max_glyph_idx = glyph_span.end();
                for i in char_byte_span.each_index() {
                    if byte_to_glyph[i] > NO_GLYPH {
                        max_glyph_idx = cmp::max(byte_to_glyph[i] as usize + 1, max_glyph_idx);
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
                    if !char_byte_span.contains(loc as usize) {
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
                    && byte_to_glyph[covered_byte_span.end()] == NO_GLYPH {
                let range = text.char_range_at(covered_byte_span.end());
                drop(range.ch);
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
            covered_byte_span.extend_to(cmp::min(end, byte_max));

            // fast path: 1-to-1 mapping of single char and single glyph.
            if glyph_span.length() == 1 {
                // TODO(Issue #214): cluster ranges need to be computed before
                // shaping, and then consulted here.
                // for now, just pretend that every character is a cluster start.
                // (i.e., pretend there are no combining character sequences).
                // 1-to-1 mapping of character to glyph also treated as ligature start.
                //
                // NB: When we acquire the ability to handle ligatures that cross word boundaries,
                // we'll need to do something special to handle `word-spacing` properly.
                let character = text.char_at(char_byte_span.begin());
                if is_bidi_control(character) {
                    glyphs.add_nonglyph_for_char_index(char_idx, false, false);
                } else {
                    let shape = glyph_data.entry_for_glyph(glyph_span.begin(), &mut y_pos);
                    let advance = self.advance_for_shaped_glyph(shape.advance, character, options);
                    let data = GlyphData::new(shape.codepoint,
                                              advance,
                                              shape.offset,
                                              false,
                                              true,
                                              true);
                    glyphs.add_glyph_for_char_index(char_idx, Some(character), &data);
                }
            } else {
                // collect all glyphs to be assigned to the first character.
                let mut datas = vec!();

                for glyph_i in glyph_span.each_index() {
                    let shape = glyph_data.entry_for_glyph(glyph_i, &mut y_pos);
                    datas.push(GlyphData::new(shape.codepoint,
                                              shape.advance,
                                              shape.offset,
                                              false, // not missing
                                              true,  // treat as cluster start
                                              glyph_i > glyph_span.begin()));
                                              // all but first are ligature continuations
                }

                // now add the detailed glyph entry.
                glyphs.add_glyphs_for_char_index(char_idx, &datas);

                // set the other chars, who have no glyphs
                let mut i = covered_byte_span.begin();
                loop {
                    let range = text.char_range_at(i);
                    drop(range.ch);
                    i = range.next;
                    if i >= covered_byte_span.end() { break; }
                    char_idx = char_idx + char_step;
                    glyphs.add_nonglyph_for_char_index(char_idx, false, false);
                }
            }

            // shift up our working spans past things we just handled.
            let end = glyph_span.end(); // FIXME: borrow checker workaround
            glyph_span.reset(end, 0);
            let end = char_byte_span.end();; // FIXME: borrow checker workaround
            char_byte_span.reset(end, 0);
            char_idx = char_idx + char_step;
        }

        // this must be called after adding all glyph data; it sorts the
        // lookup table for finding detailed glyphs by associated char index.
        glyphs.finalize_changes();
    }

    fn advance_for_shaped_glyph(&self, mut advance: Au, character: char, options: &ShapingOptions)
                                -> Au {
        match options.letter_spacing {
            None => {}
            Some(letter_spacing) => advance = advance + letter_spacing,
        };

        // CSS 2.1 § 16.4 states that "word spacing affects each space (U+0020) and non-breaking
        // space (U+00A0) left in the text after the white space processing rules have been
        // applied. The effect of the property on other word-separator characters is undefined."
        // We elect to only space the two required code points.
        if character == ' ' || character == '\u{a0}' {
            advance = advance + options.word_spacing
        } else if character == '\t' {
            let tab_size = 8f64;
            advance = Au::from_f64_px(tab_size * glyph_space_advance(self.font_and_shaping_options.font));
        }

        advance
    }
}

// Callbacks from Harfbuzz when font map and glyph advance lookup needed.
lazy_static! {
    static ref HB_FONT_FUNCS: ptr::Unique<hb_font_funcs_t> = unsafe {
        let hb_funcs = RUST_hb_font_funcs_create();
        RUST_hb_font_funcs_set_glyph_func(hb_funcs, glyph_func, ptr::null_mut(), None);
        RUST_hb_font_funcs_set_glyph_h_advance_func(
            hb_funcs, glyph_h_advance_func, ptr::null_mut(), None);
        RUST_hb_font_funcs_set_glyph_h_kerning_func(
            hb_funcs, glyph_h_kerning_func, ptr::null_mut(), ptr::null_mut());

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

fn glyph_space_advance(font: *mut Font) -> f64 {
    let space_unicode = ' ';
    let space_glyph: hb_codepoint_t;
    match unsafe { (*font).glyph_index(space_unicode) } {
        Some(g) => {
            space_glyph = g as hb_codepoint_t;
        }
        None => panic!("No space info")
    }
    let space_advance = unsafe { (*font).glyph_h_advance(space_glyph as GlyphId) };
    space_advance
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
        let font_and_shaping_options: *const FontAndShapingOptions =
            user_data as *const FontAndShapingOptions;
        assert!(!font_and_shaping_options.is_null());
        assert!(!(*font_and_shaping_options).font.is_null());

        // TODO(Issue #197): reuse font table data, which will change the unsound trickery here.
        match (*(*font_and_shaping_options).font).table_for_tag(tag as FontTableTag) {
            None => ptr::null_mut(),
            Some(font_table) => {
                // `Box::into_raw` intentionally leaks the FontTable so we don't destroy the buffer
                // while HarfBuzz is using it.  When HarfBuzz is done with the buffer, it will pass
                // this raw pointer back to `destroy_blob_func` which will deallocate the Box.
                let font_table_ptr = Box::into_raw(font_table);

                let mut blob: *mut hb_blob_t = ptr::null_mut();
                (*font_table_ptr).with_buffer(|buf: *const u8, len: usize| {
                    // HarfBuzz calls `destroy_blob_func` when the buffer is no longer needed.
                    blob = RUST_hb_blob_create(buf as *const c_char,
                                               len as c_uint,
                                               HB_MEMORY_MODE_READONLY,
                                               font_table_ptr as *mut c_void,
                                               destroy_blob_func);
                });

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
