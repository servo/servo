/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use crate::font::{Font, FontTableMethods, FontTableTag, ShapingFlags, ShapingOptions, KERN};
use crate::platform::font::FontTable;
use crate::text::glyph::{ByteIndex, GlyphData, GlyphId, GlyphStore};
use crate::text::shaping::ShaperMethods;
use crate::text::util::{fixed_to_float, float_to_fixed, is_bidi_control};
use app_units::Au;
use euclid::default::Point2D;
// Eventually we would like the shaper to be pluggable, as many operating systems have their own
// shapers. For now, however, HarfBuzz is a hard dependency.
use harfbuzz_sys::hb_blob_t;
use harfbuzz_sys::hb_bool_t;
use harfbuzz_sys::hb_buffer_add_utf8;
use harfbuzz_sys::hb_buffer_destroy;
use harfbuzz_sys::hb_buffer_get_glyph_positions;
use harfbuzz_sys::hb_buffer_get_length;
use harfbuzz_sys::hb_face_destroy;
use harfbuzz_sys::hb_feature_t;
use harfbuzz_sys::hb_font_create;
use harfbuzz_sys::hb_font_funcs_create;
use harfbuzz_sys::hb_font_funcs_set_glyph_h_advance_func;
use harfbuzz_sys::hb_font_funcs_set_nominal_glyph_func;
use harfbuzz_sys::hb_font_set_funcs;
use harfbuzz_sys::hb_font_set_ppem;
use harfbuzz_sys::hb_font_set_scale;
use harfbuzz_sys::hb_glyph_info_t;
use harfbuzz_sys::hb_glyph_position_t;
use harfbuzz_sys::{hb_blob_create, hb_face_create_for_tables};
use harfbuzz_sys::{hb_buffer_create, hb_font_destroy};
use harfbuzz_sys::{hb_buffer_get_glyph_infos, hb_shape};
use harfbuzz_sys::{hb_buffer_set_direction, hb_buffer_set_script};
use harfbuzz_sys::{hb_buffer_t, hb_codepoint_t, hb_font_funcs_t};
use harfbuzz_sys::{hb_face_t, hb_font_t};
use harfbuzz_sys::{hb_position_t, hb_tag_t};
use harfbuzz_sys::{HB_DIRECTION_LTR, HB_DIRECTION_RTL, HB_MEMORY_MODE_READONLY};
use std::os::raw::{c_char, c_int, c_uint, c_void};
use std::{char, cmp, ptr};

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
            assert_eq!(glyph_count, pos_count);

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
            let hb_face: *mut hb_face_t = hb_face_create_for_tables(
                Some(font_table_func),
                font as *const c_void as *mut c_void,
                None,
            );
            let hb_font: *mut hb_font_t = hb_font_create(hb_face);

            // Set points-per-em. if zero, performs no hinting in that direction.
            let pt_size = (*font).actual_pt_size.to_f64_px();
            hb_font_set_ppem(hb_font, pt_size as c_uint, pt_size as c_uint);

            // Set scaling. Note that this takes 16.16 fixed point.
            hb_font_set_scale(
                hb_font,
                Shaper::float_to_fixed(pt_size) as c_int,
                Shaper::float_to_fixed(pt_size) as c_int,
            );

            // configure static function callbacks.
            hb_font_set_funcs(
                hb_font,
                HB_FONT_FUNCS.0,
                font as *mut Font as *mut c_void,
                None,
            );

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

pub fn unicode_to_hb_script(script: unicode_script::Script) -> harfbuzz_sys::hb_script_t {
    use harfbuzz_sys::*;
    use unicode_script::Script::*;
    match script {
        Adlam => HB_SCRIPT_ADLAM,
        Ahom => HB_SCRIPT_AHOM,
        Anatolian_Hieroglyphs => HB_SCRIPT_ANATOLIAN_HIEROGLYPHS,
        Arabic => HB_SCRIPT_ARABIC,
        Armenian => HB_SCRIPT_ARMENIAN,
        Avestan => HB_SCRIPT_AVESTAN,
        Balinese => HB_SCRIPT_BALINESE,
        Bamum => HB_SCRIPT_BAMUM,
        Bassa_Vah => HB_SCRIPT_BASSA_VAH,
        Batak => HB_SCRIPT_BATAK,
        Bengali => HB_SCRIPT_BENGALI,
        Bhaiksuki => HB_SCRIPT_BHAIKSUKI,
        Bopomofo => HB_SCRIPT_BOPOMOFO,
        Brahmi => HB_SCRIPT_BRAHMI,
        Braille => HB_SCRIPT_BRAILLE,
        Buginese => HB_SCRIPT_BUGINESE,
        Buhid => HB_SCRIPT_BUHID,
        Canadian_Aboriginal => HB_SCRIPT_CANADIAN_SYLLABICS,
        Carian => HB_SCRIPT_CARIAN,
        Caucasian_Albanian => HB_SCRIPT_CAUCASIAN_ALBANIAN,
        Chakma => HB_SCRIPT_CHAKMA,
        Cham => HB_SCRIPT_CHAM,
        Cherokee => HB_SCRIPT_CHEROKEE,
        Chorasmian => HB_SCRIPT_CHORASMIAN,
        Common => HB_SCRIPT_COMMON,
        Coptic => HB_SCRIPT_COPTIC,
        Cuneiform => HB_SCRIPT_CUNEIFORM,
        Cypriot => HB_SCRIPT_CYPRIOT,
        Cyrillic => HB_SCRIPT_CYRILLIC,
        Deseret => HB_SCRIPT_DESERET,
        Devanagari => HB_SCRIPT_DEVANAGARI,
        Dives_Akuru => HB_SCRIPT_DIVES_AKURU,
        Dogra => HB_SCRIPT_DOGRA,
        Duployan => HB_SCRIPT_DUPLOYAN,
        Egyptian_Hieroglyphs => HB_SCRIPT_EGYPTIAN_HIEROGLYPHS,
        Elbasan => HB_SCRIPT_ELBASAN,
        Elymaic => HB_SCRIPT_ELYMAIC,
        Ethiopic => HB_SCRIPT_ETHIOPIC,
        Georgian => HB_SCRIPT_GEORGIAN,
        Glagolitic => HB_SCRIPT_GLAGOLITIC,
        Gothic => HB_SCRIPT_GOTHIC,
        Grantha => HB_SCRIPT_GRANTHA,
        Greek => HB_SCRIPT_GREEK,
        Gujarati => HB_SCRIPT_GUJARATI,
        Gunjala_Gondi => HB_SCRIPT_GUNJALA_GONDI,
        Gurmukhi => HB_SCRIPT_GURMUKHI,
        Han => HB_SCRIPT_HAN,
        Hangul => HB_SCRIPT_HANGUL,
        Hanifi_Rohingya => HB_SCRIPT_HANIFI_ROHINGYA,
        Hanunoo => HB_SCRIPT_HANUNOO,
        Hatran => HB_SCRIPT_HATRAN,
        Hebrew => HB_SCRIPT_HEBREW,
        Hiragana => HB_SCRIPT_HIRAGANA,
        Imperial_Aramaic => HB_SCRIPT_IMPERIAL_ARAMAIC,
        Inherited => HB_SCRIPT_INHERITED,
        Inscriptional_Pahlavi => HB_SCRIPT_INSCRIPTIONAL_PAHLAVI,
        Inscriptional_Parthian => HB_SCRIPT_INSCRIPTIONAL_PARTHIAN,
        Javanese => HB_SCRIPT_JAVANESE,
        Kaithi => HB_SCRIPT_KAITHI,
        Kannada => HB_SCRIPT_KANNADA,
        Katakana => HB_SCRIPT_KATAKANA,
        Kayah_Li => HB_SCRIPT_KAYAH_LI,
        Kharoshthi => HB_SCRIPT_KHAROSHTHI,
        Khitan_Small_Script => HB_SCRIPT_KHITAN_SMALL_SCRIPT,
        Khmer => HB_SCRIPT_KHMER,
        Khojki => HB_SCRIPT_KHOJKI,
        Khudawadi => HB_SCRIPT_KHUDAWADI,
        Lao => HB_SCRIPT_LAO,
        Latin => HB_SCRIPT_LATIN,
        Lepcha => HB_SCRIPT_LEPCHA,
        Limbu => HB_SCRIPT_LIMBU,
        Linear_A => HB_SCRIPT_LINEAR_A,
        Linear_B => HB_SCRIPT_LINEAR_B,
        Lisu => HB_SCRIPT_LISU,
        Lycian => HB_SCRIPT_LYCIAN,
        Lydian => HB_SCRIPT_LYDIAN,
        Mahajani => HB_SCRIPT_MAHAJANI,
        Makasar => HB_SCRIPT_MAKASAR,
        Malayalam => HB_SCRIPT_MALAYALAM,
        Mandaic => HB_SCRIPT_MANDAIC,
        Manichaean => HB_SCRIPT_MANICHAEAN,
        Marchen => HB_SCRIPT_MARCHEN,
        Masaram_Gondi => HB_SCRIPT_MASARAM_GONDI,
        Medefaidrin => HB_SCRIPT_MEDEFAIDRIN,
        Meetei_Mayek => HB_SCRIPT_MEETEI_MAYEK,
        Mende_Kikakui => HB_SCRIPT_MENDE_KIKAKUI,
        Meroitic_Cursive => HB_SCRIPT_MEROITIC_CURSIVE,
        Meroitic_Hieroglyphs => HB_SCRIPT_MEROITIC_HIEROGLYPHS,
        Miao => HB_SCRIPT_MIAO,
        Modi => HB_SCRIPT_MODI,
        Mongolian => HB_SCRIPT_MONGOLIAN,
        Mro => HB_SCRIPT_MRO,
        Multani => HB_SCRIPT_MULTANI,
        Myanmar => HB_SCRIPT_MYANMAR,
        Nabataean => HB_SCRIPT_NABATAEAN,
        Nandinagari => HB_SCRIPT_NANDINAGARI,
        New_Tai_Lue => HB_SCRIPT_NEW_TAI_LUE,
        Newa => HB_SCRIPT_NEWA,
        Nko => HB_SCRIPT_NKO,
        Nushu => HB_SCRIPT_NUSHU,
        Nyiakeng_Puachue_Hmong => HB_SCRIPT_NYIAKENG_PUACHUE_HMONG,
        Ogham => HB_SCRIPT_OGHAM,
        Ol_Chiki => HB_SCRIPT_OL_CHIKI,
        Old_Hungarian => HB_SCRIPT_OLD_HUNGARIAN,
        Old_Italic => HB_SCRIPT_OLD_ITALIC,
        Old_North_Arabian => HB_SCRIPT_OLD_NORTH_ARABIAN,
        Old_Permic => HB_SCRIPT_OLD_PERMIC,
        Old_Persian => HB_SCRIPT_OLD_PERSIAN,
        Old_Sogdian => HB_SCRIPT_OLD_SOGDIAN,
        Old_South_Arabian => HB_SCRIPT_OLD_SOUTH_ARABIAN,
        Old_Turkic => HB_SCRIPT_OLD_TURKIC,
        Oriya => HB_SCRIPT_ORIYA,
        Osage => HB_SCRIPT_OSAGE,
        Osmanya => HB_SCRIPT_OSMANYA,
        Pahawh_Hmong => HB_SCRIPT_PAHAWH_HMONG,
        Palmyrene => HB_SCRIPT_PALMYRENE,
        Pau_Cin_Hau => HB_SCRIPT_PAU_CIN_HAU,
        Phags_Pa => HB_SCRIPT_PHAGS_PA,
        Phoenician => HB_SCRIPT_PHOENICIAN,
        Psalter_Pahlavi => HB_SCRIPT_PSALTER_PAHLAVI,
        Rejang => HB_SCRIPT_REJANG,
        Runic => HB_SCRIPT_RUNIC,
        Samaritan => HB_SCRIPT_SAMARITAN,
        Saurashtra => HB_SCRIPT_SAURASHTRA,
        Sharada => HB_SCRIPT_SHARADA,
        Shavian => HB_SCRIPT_SHAVIAN,
        Siddham => HB_SCRIPT_SIDDHAM,
        SignWriting => HB_SCRIPT_SIGNWRITING,
        Sinhala => HB_SCRIPT_SINHALA,
        Sogdian => HB_SCRIPT_SOGDIAN,
        Sora_Sompeng => HB_SCRIPT_SORA_SOMPENG,
        Soyombo => HB_SCRIPT_SOYOMBO,
        Sundanese => HB_SCRIPT_SUNDANESE,
        Syloti_Nagri => HB_SCRIPT_SYLOTI_NAGRI,
        Syriac => HB_SCRIPT_SYRIAC,
        Tagalog => HB_SCRIPT_TAGALOG,
        Tagbanwa => HB_SCRIPT_TAGBANWA,
        Tai_Le => HB_SCRIPT_TAI_LE,
        Tai_Tham => HB_SCRIPT_TAI_THAM,
        Tai_Viet => HB_SCRIPT_TAI_VIET,
        Takri => HB_SCRIPT_TAKRI,
        Tamil => HB_SCRIPT_TAMIL,
        Tangut => HB_SCRIPT_TANGUT,
        Telugu => HB_SCRIPT_TELUGU,
        Thaana => HB_SCRIPT_THAANA,
        Thai => HB_SCRIPT_THAI,
        Tibetan => HB_SCRIPT_TIBETAN,
        Tifinagh => HB_SCRIPT_TIFINAGH,
        Tirhuta => HB_SCRIPT_TIRHUTA,
        Ugaritic => HB_SCRIPT_UGARITIC,
        Unknown => HB_SCRIPT_UNKNOWN,
        Vai => HB_SCRIPT_VAI,
        Warang_Citi => HB_SCRIPT_WARANG_CITI,
        Wancho => HB_SCRIPT_WANCHO,
        Yezidi => HB_SCRIPT_YEZIDI,
        Yi => HB_SCRIPT_YI,
        Zanabazar_Square => HB_SCRIPT_ZANABAZAR_SQUARE,
        _ => HB_SCRIPT_UNKNOWN,
    }
}

impl ShaperMethods for Shaper {
    /// Calculate the layout metrics associated with the given text when painted in a specific
    /// font.
    fn shape_text(&self, text: &str, options: &ShapingOptions, glyphs: &mut GlyphStore) {
        unsafe {
            let hb_buffer: *mut hb_buffer_t = hb_buffer_create();
            hb_buffer_set_direction(
                hb_buffer,
                if options.flags.contains(ShapingFlags::RTL_FLAG) {
                    HB_DIRECTION_RTL
                } else {
                    HB_DIRECTION_LTR
                },
            );

            hb_buffer_set_script(hb_buffer, unicode_to_hb_script(options.script));

            hb_buffer_add_utf8(
                hb_buffer,
                text.as_ptr() as *const c_char,
                text.len() as c_int,
                0,
                text.len() as c_int,
            );

            let mut features = Vec::new();
            if options
                .flags
                .contains(ShapingFlags::IGNORE_LIGATURES_SHAPING_FLAG)
            {
                features.push(hb_feature_t {
                    tag: LIGA,
                    value: 0,
                    start: 0,
                    end: hb_buffer_get_length(hb_buffer),
                })
            }
            if options
                .flags
                .contains(ShapingFlags::DISABLE_KERNING_SHAPING_FLAG)
            {
                features.push(hb_feature_t {
                    tag: KERN,
                    value: 0,
                    start: 0,
                    end: hb_buffer_get_length(hb_buffer),
                })
            }

            hb_shape(
                self.hb_font,
                hb_buffer,
                features.as_mut_ptr(),
                features.len() as u32,
            );
            self.save_glyph_results(text, options, glyphs, hb_buffer);
            hb_buffer_destroy(hb_buffer);
        }
    }
}

impl Shaper {
    fn save_glyph_results(
        &self,
        text: &str,
        options: &ShapingOptions,
        glyphs: &mut GlyphStore,
        buffer: *mut hb_buffer_t,
    ) {
        let glyph_data = ShapedGlyphData::new(buffer);
        let glyph_count = glyph_data.len();
        let byte_max = text.len();

        debug!(
            "Shaped text[byte count={}], got back {} glyph info records.",
            byte_max, glyph_count
        );

        // make map of what chars have glyphs
        let mut byte_to_glyph = vec![NO_GLYPH; byte_max];

        debug!("(glyph idx) -> (text byte offset)");
        for i in 0..glyph_data.len() {
            let loc = glyph_data.byte_offset_of_glyph(i) as usize;
            if loc < byte_max {
                byte_to_glyph[loc] = i as i32;
            } else {
                debug!(
                    "ERROR: tried to set out of range byte_to_glyph: idx={}, glyph idx={}",
                    loc, i
                );
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
                if glyph_span.len() == 1 {
                    break;
                }

                // if no glyphs were found yet, extend the char byte range more.
                if glyph_span.len() == 0 {
                    continue;
                }

                // If byte_range now includes all the byte offsets found in glyph_span, then we
                // have found a contiguous "cluster" and can stop extending it.
                let mut all_glyphs_are_within_cluster: bool = true;
                for j in glyph_span.clone() {
                    let loc = glyph_data.byte_offset_of_glyph(j) as usize;
                    if !(byte_range.start <= loc && loc < byte_range.end) {
                        all_glyphs_are_within_cluster = false;
                        break;
                    }
                }
                if all_glyphs_are_within_cluster {
                    break;
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
                    let data =
                        GlyphData::new(space_glyph_id, advance, Default::default(), true, true);
                    glyphs.add_glyph_for_byte_index(byte_idx, character, &data);
                } else {
                    let shape = glyph_data.entry_for_glyph(glyph_span.start, &mut y_pos);
                    let advance = self.advance_for_shaped_glyph(shape.advance, character, options);
                    let data = GlyphData::new(shape.codepoint, advance, shape.offset, true, true);
                    glyphs.add_glyph_for_byte_index(byte_idx, character, &data);
                }
            } else {
                // collect all glyphs to be assigned to the first character.
                let mut datas = vec![];

                for glyph_i in glyph_span.clone() {
                    let shape = glyph_data.entry_for_glyph(glyph_i, &mut y_pos);
                    datas.push(GlyphData::new(
                        shape.codepoint,
                        shape.advance,
                        shape.offset,
                        true, // treat as cluster start
                        glyph_i > glyph_span.start,
                    ));
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

    fn advance_for_shaped_glyph(
        &self,
        mut advance: Au,
        character: char,
        options: &ShapingOptions,
    ) -> Au {
        if let Some(letter_spacing) = options.letter_spacing {
            advance = advance + letter_spacing;
        };

        // CSS 2.1 § 16.4 states that "word spacing affects each space (U+0020) and non-breaking
        // space (U+00A0) left in the text after the white space processing rules have been
        // applied. The effect of the property on other word-separator characters is undefined."
        // We elect to only space the two required code points.
        if character == ' ' || character == '\u{a0}' {
            // https://drafts.csswg.org/css-text-3/#word-spacing-property
            advance += options.word_spacing;
        }

        advance
    }
}

/// Callbacks from Harfbuzz when font map and glyph advance lookup needed.
struct FontFuncs(*mut hb_font_funcs_t);

unsafe impl Sync for FontFuncs {}

lazy_static! {
    static ref HB_FONT_FUNCS: FontFuncs = unsafe {
        let hb_funcs = hb_font_funcs_create();
        hb_font_funcs_set_nominal_glyph_func(hb_funcs, Some(glyph_func), ptr::null_mut(), None);
        hb_font_funcs_set_glyph_h_advance_func(
            hb_funcs,
            Some(glyph_h_advance_func),
            ptr::null_mut(),
            None,
        );

        FontFuncs(hb_funcs)
    };
}

extern "C" fn glyph_func(
    _: *mut hb_font_t,
    font_data: *mut c_void,
    unicode: hb_codepoint_t,
    glyph: *mut hb_codepoint_t,
    _: *mut c_void,
) -> hb_bool_t {
    let font: *const Font = font_data as *const Font;
    assert!(!font.is_null());

    unsafe {
        match (*font).glyph_index(char::from_u32(unicode).unwrap()) {
            Some(g) => {
                *glyph = g as hb_codepoint_t;
                true as hb_bool_t
            },
            None => false as hb_bool_t,
        }
    }
}

extern "C" fn glyph_h_advance_func(
    _: *mut hb_font_t,
    font_data: *mut c_void,
    glyph: hb_codepoint_t,
    _: *mut c_void,
) -> hb_position_t {
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
        },
        None => panic!("No space info"),
    }
    let space_advance = unsafe { (*font).glyph_h_advance(space_glyph as GlyphId) };
    (space_glyph, space_advance)
}

// Callback to get a font table out of a font.
extern "C" fn font_table_func(
    _: *mut hb_face_t,
    tag: hb_tag_t,
    user_data: *mut c_void,
) -> *mut hb_blob_t {
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
                let font_table_ptr = Box::into_raw(Box::new(font_table));

                let buf = (*font_table_ptr).buffer();
                // HarfBuzz calls `destroy_blob_func` when the buffer is no longer needed.
                let blob = hb_blob_create(
                    buf.as_ptr() as *const c_char,
                    buf.len() as c_uint,
                    HB_MEMORY_MODE_READONLY,
                    font_table_ptr as *mut c_void,
                    Some(destroy_blob_func),
                );

                assert!(!blob.is_null());
                blob
            },
        }
    }
}

extern "C" fn destroy_blob_func(font_table_ptr: *mut c_void) {
    unsafe {
        drop(Box::from_raw(font_table_ptr as *mut FontTable));
    }
}
