/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use std::os::raw::{c_char, c_int, c_uint, c_void};
use std::sync::LazyLock;
use std::{char, ptr};

use app_units::Au;
use euclid::default::Point2D;
// Eventually we would like the shaper to be pluggable, as many operating systems have their own
// shapers. For now, however, HarfBuzz is a hard dependency.
use harfbuzz_sys::{
    HB_DIRECTION_LTR, HB_DIRECTION_RTL, HB_MEMORY_MODE_READONLY, HB_OT_LAYOUT_BASELINE_TAG_HANGING,
    HB_OT_LAYOUT_BASELINE_TAG_IDEO_EMBOX_BOTTOM_OR_LEFT, HB_OT_LAYOUT_BASELINE_TAG_ROMAN,
    hb_blob_create, hb_blob_t, hb_bool_t, hb_buffer_add_utf8, hb_buffer_create, hb_buffer_destroy,
    hb_buffer_get_glyph_infos, hb_buffer_get_glyph_positions, hb_buffer_get_length,
    hb_buffer_set_direction, hb_buffer_set_script, hb_buffer_t, hb_codepoint_t,
    hb_face_create_for_tables, hb_face_destroy, hb_face_t, hb_feature_t, hb_font_create,
    hb_font_destroy, hb_font_funcs_create, hb_font_funcs_set_glyph_h_advance_func,
    hb_font_funcs_set_nominal_glyph_func, hb_font_funcs_t, hb_font_set_funcs, hb_font_set_ppem,
    hb_font_set_scale, hb_font_set_variations, hb_font_t, hb_glyph_info_t, hb_glyph_position_t,
    hb_ot_layout_get_baseline, hb_position_t, hb_script_from_iso15924_tag, hb_shape, hb_tag_t,
    hb_variation_t,
};
use num_traits::Zero;
use read_fonts::types::Tag;

use super::{HarfBuzzShapedGlyphData, ShapedGlyphEntry, unicode_script_to_iso15924_tag};
use crate::platform::font::FontTable;
use crate::{
    BASE, Font, FontBaseline, FontTableMethods, GlyphId, GlyphStore, KERN, LIGA, ShapingFlags,
    ShapingOptions, fixed_to_float, float_to_fixed,
};

const HB_OT_TAG_DEFAULT_SCRIPT: hb_tag_t = u32::from_be_bytes(Tag::new(b"DFLT").to_be_bytes());
const HB_OT_TAG_DEFAULT_LANGUAGE: hb_tag_t = u32::from_be_bytes(Tag::new(b"dflt").to_be_bytes());

pub(crate) struct ShapedGlyphData {
    count: usize,
    buffer: *mut hb_buffer_t,
    glyph_infos: *mut hb_glyph_info_t,
    pos_infos: *mut hb_glyph_position_t,
}

impl ShapedGlyphData {
    /// Create a new [`ShapedGlyphData`] from the given HarfBuzz buffer.
    ///
    /// # Safety
    ///
    /// - Passing an invalid buffer pointer to this function results in undefined behavior.
    /// - This function takes ownership of the buffer and the ShapedGlyphData destroys the buffer when dropped
    ///   so the pointer must an owned pointer and must not be used after being passed to this function
    unsafe fn new(buffer: *mut hb_buffer_t) -> ShapedGlyphData {
        let mut glyph_count = 0;
        let glyph_infos = unsafe { hb_buffer_get_glyph_infos(buffer, &mut glyph_count) };
        assert!(!glyph_infos.is_null());
        let mut pos_count = 0;
        let pos_infos = unsafe { hb_buffer_get_glyph_positions(buffer, &mut pos_count) };
        assert!(!pos_infos.is_null());
        assert_eq!(glyph_count, pos_count);

        ShapedGlyphData {
            count: glyph_count as usize,
            buffer,
            glyph_infos,
            pos_infos,
        }
    }
}

impl Drop for ShapedGlyphData {
    fn drop(&mut self) {
        unsafe { hb_buffer_destroy(self.buffer) }
    }
}

impl HarfBuzzShapedGlyphData for ShapedGlyphData {
    #[inline]
    fn len(&self) -> usize {
        self.count
    }

    #[inline(always)]
    fn byte_offset_of_glyph(&self, i: usize) -> u32 {
        assert!(i < self.count);

        unsafe {
            let glyph_info_i = self.glyph_infos.add(i);
            (*glyph_info_i).cluster
        }
    }

    /// Returns shaped glyph data for one glyph, and updates the y-position of the pen.
    fn entry_for_glyph(&self, i: usize, y_pos: &mut Au) -> ShapedGlyphEntry {
        assert!(i < self.count);

        unsafe {
            let glyph_info_i = self.glyph_infos.add(i);
            let pos_info_i = self.pos_infos.add(i);
            let x_offset = Shaper::fixed_to_float((*pos_info_i).x_offset);
            let y_offset = Shaper::fixed_to_float((*pos_info_i).y_offset);
            let x_advance = Shaper::fixed_to_float((*pos_info_i).x_advance);
            let y_advance = Shaper::fixed_to_float((*pos_info_i).y_advance);

            let x_offset = Au::from_f64_px(x_offset);
            let y_offset = Au::from_f64_px(y_offset);
            let x_advance = Au::from_f64_px(x_advance);
            let y_advance = Au::from_f64_px(y_advance);

            let offset = if x_offset.is_zero() && y_offset.is_zero() && y_advance.is_zero() {
                None
            } else {
                // adjust the pen..
                if y_advance > Au::zero() {
                    *y_pos -= y_advance;
                }

                Some(Point2D::new(x_offset, *y_pos - y_offset))
            };

            ShapedGlyphEntry {
                codepoint: (*glyph_info_i).codepoint as GlyphId,
                advance: x_advance,
                offset,
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct Shaper {
    hb_face: *mut hb_face_t,
    hb_font: *mut hb_font_t,
    font: *const Font,
}

// The HarfBuzz API is thread safe as well as our `Font`, so we can make the data
// structures here as thread-safe as well. This doesn't seem to be documented,
// but was expressed as one of the original goals of the HarfBuzz API.
unsafe impl Sync for Shaper {}
unsafe impl Send for Shaper {}

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
    pub(crate) fn new(font: &Font) -> Shaper {
        unsafe {
            let hb_face: *mut hb_face_t = hb_face_create_for_tables(
                Some(font_table_func),
                font as *const Font as *mut c_void,
                None,
            );
            let hb_font: *mut hb_font_t = hb_font_create(hb_face);

            // Set points-per-em. if zero, performs no hinting in that direction.
            let pt_size = font.descriptor.pt_size.to_f64_px();
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
                font as *const Font as *mut c_void,
                None,
            );

            if servo_config::pref!(layout_variable_fonts_enabled) {
                let variations = &font.variations();
                if !variations.is_empty() {
                    let variations: Vec<_> = variations
                        .iter()
                        .map(|variation| hb_variation_t {
                            tag: variation.tag,

                            value: variation.value,
                        })
                        .collect();

                    hb_font_set_variations(hb_font, variations.as_ptr(), variations.len() as u32);
                }
            }

            Shaper {
                hb_face,
                hb_font,
                font,
            }
        }
    }

    /// Calculate the layout metrics associated with the given text with the [`Shaper`]s font.
    pub(crate) fn shaped_glyph_data(
        &self,
        text: &str,
        options: &ShapingOptions,
    ) -> ShapedGlyphData {
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

            let script =
                hb_script_from_iso15924_tag(unicode_script_to_iso15924_tag(options.script));
            hb_buffer_set_script(hb_buffer, script);

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
                    tag: u32::from_be_bytes(LIGA.to_be_bytes()),
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
                    tag: u32::from_be_bytes(KERN.to_be_bytes()),
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

            ShapedGlyphData::new(hb_buffer)
        }
    }

    pub(crate) fn font(&self) -> &Font {
        unsafe { &(*self.font) }
    }

    pub(crate) fn shape_text(&self, text: &str, options: &ShapingOptions, glyphs: &mut GlyphStore) {
        let glyph_data = self.shaped_glyph_data(text, options);
        let font = self.font();
        super::shape_text_harfbuzz(&glyph_data, font, text, options, glyphs);
    }

    pub(crate) fn baseline(&self) -> Option<FontBaseline> {
        unsafe { (*self.font).table_for_tag(BASE)? };

        let mut hanging_baseline = 0;
        let mut alphabetic_baseline = 0;
        let mut ideographic_baseline = 0;

        unsafe {
            hb_ot_layout_get_baseline(
                self.hb_font,
                HB_OT_LAYOUT_BASELINE_TAG_ROMAN,
                HB_DIRECTION_LTR,
                HB_OT_TAG_DEFAULT_SCRIPT,
                HB_OT_TAG_DEFAULT_LANGUAGE,
                &mut alphabetic_baseline as *mut _,
            );

            hb_ot_layout_get_baseline(
                self.hb_font,
                HB_OT_LAYOUT_BASELINE_TAG_HANGING,
                HB_DIRECTION_LTR,
                HB_OT_TAG_DEFAULT_SCRIPT,
                HB_OT_TAG_DEFAULT_LANGUAGE,
                &mut hanging_baseline as *mut _,
            );

            hb_ot_layout_get_baseline(
                self.hb_font,
                HB_OT_LAYOUT_BASELINE_TAG_IDEO_EMBOX_BOTTOM_OR_LEFT,
                HB_DIRECTION_LTR,
                HB_OT_TAG_DEFAULT_SCRIPT,
                HB_OT_TAG_DEFAULT_LANGUAGE,
                &mut ideographic_baseline as *mut _,
            );
        }

        Some(FontBaseline {
            ideographic_baseline: Shaper::fixed_to_float(ideographic_baseline) as f32,
            alphabetic_baseline: Shaper::fixed_to_float(alphabetic_baseline) as f32,
            hanging_baseline: Shaper::fixed_to_float(hanging_baseline) as f32,
        })
    }

    fn float_to_fixed(f: f64) -> i32 {
        float_to_fixed(16, f)
    }

    fn fixed_to_float(i: hb_position_t) -> f64 {
        fixed_to_float(16, i)
    }
}

/// Callbacks from Harfbuzz when font map and glyph advance lookup needed.
struct FontFuncs(*mut hb_font_funcs_t);

unsafe impl Sync for FontFuncs {}
unsafe impl Send for FontFuncs {}

static HB_FONT_FUNCS: LazyLock<FontFuncs> = LazyLock::new(|| unsafe {
    let hb_funcs = hb_font_funcs_create();
    hb_font_funcs_set_nominal_glyph_func(hb_funcs, Some(glyph_func), ptr::null_mut(), None);
    hb_font_funcs_set_glyph_h_advance_func(
        hb_funcs,
        Some(glyph_h_advance_func),
        ptr::null_mut(),
        None,
    );

    FontFuncs(hb_funcs)
});

extern "C" fn glyph_func(
    _: *mut hb_font_t,
    font_data: *mut c_void,
    unicode: hb_codepoint_t,
    glyph: *mut hb_codepoint_t,
    _: *mut c_void,
) -> hb_bool_t {
    let font: *const Font = font_data as *const Font;
    assert!(!font.is_null());

    match unsafe { (*font).glyph_index(char::from_u32(unicode).unwrap()) } {
        Some(g) => {
            unsafe { *glyph = g as hb_codepoint_t };
            true as hb_bool_t
        },
        None => false as hb_bool_t,
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

    let advance = unsafe { (*font).glyph_h_advance(glyph as GlyphId) };
    Shaper::float_to_fixed(advance)
}

/// Callback to get a font table out of a font.
extern "C" fn font_table_func(
    _: *mut hb_face_t,
    tag: hb_tag_t,
    user_data: *mut c_void,
) -> *mut hb_blob_t {
    // NB: These asserts have security implications.
    let font = user_data as *const Font;
    assert!(!font.is_null());

    // TODO(Issue #197): reuse font table data, which will change the unsound trickery here.
    let Some(font_table) = (unsafe { (*font).table_for_tag(Tag::from_u32(tag)) }) else {
        return ptr::null_mut();
    };

    // `Box::into_raw` intentionally leaks the FontTable so we don't destroy the buffer
    // while HarfBuzz is using it.  When HarfBuzz is done with the buffer, it will pass
    // this raw pointer back to `destroy_blob_func` which will deallocate the Box.
    let font_table_ptr = Box::into_raw(Box::new(font_table));

    let buf = unsafe { (*font_table_ptr).buffer() };
    // HarfBuzz calls `destroy_blob_func` when the buffer is no longer needed.
    let blob = unsafe {
        hb_blob_create(
            buf.as_ptr() as *const c_char,
            buf.len() as c_uint,
            HB_MEMORY_MODE_READONLY,
            font_table_ptr as *mut c_void,
            Some(destroy_blob_func),
        )
    };

    assert!(!blob.is_null());
    blob
}

extern "C" fn destroy_blob_func(font_table_ptr: *mut c_void) {
    unsafe {
        drop(Box::from_raw(font_table_ptr as *mut FontTable));
    }
}
