/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use app_units::Au;
use euclid::default::Point2D;
use harfrust::{
    Feature, FontRef as HarfRustFontRef, GlyphBuffer, Script, ShaperData, ShaperInstance, Tag,
    UnicodeBuffer, Variation,
};
use num_traits::Zero as _;
use read_fonts::TableProvider;
use read_fonts::types::BigEndian;
use smallvec::SmallVec;

use super::{HarfBuzzShapedGlyphData, ShapedGlyphEntry, unicode_script_to_iso15924_tag};
use crate::{Font, FontBaseline, FontData, GlyphStore, ShapingFlags, ShapingOptions};

/// Convert a `webrender_api::FontVariation` to a `harfrust::Variation`
fn wr_variation_to_hr_varation(wr_variation: webrender_api::FontVariation) -> harfrust::Variation {
    Variation {
        tag: Tag::from_u32(wr_variation.tag),
        value: wr_variation.value,
    }
}

pub(crate) struct ShapedGlyphData {
    data: GlyphBuffer,
    scale: f64,
}

impl HarfBuzzShapedGlyphData for ShapedGlyphData {
    fn len(&self) -> usize {
        self.data.len()
    }

    fn byte_offset_of_glyph(&self, i: usize) -> u32 {
        self.data.glyph_infos()[i].cluster
    }

    fn entry_for_glyph(&self, i: usize, y_pos: &mut app_units::Au) -> super::ShapedGlyphEntry {
        let glyph_info_i = self.data.glyph_infos()[i];
        let pos_info_i = self.data.glyph_positions()[i];
        let x_offset = Au::from_f64_px(pos_info_i.x_offset as f64 * self.scale);
        let y_offset = Au::from_f64_px(pos_info_i.y_offset as f64 * self.scale);
        let x_advance = Au::from_f64_px(pos_info_i.x_advance as f64 * self.scale);
        let y_advance = Au::from_f64_px(pos_info_i.y_advance as f64 * self.scale);

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
            codepoint: glyph_info_i.glyph_id,
            advance: x_advance,
            offset,
        }
    }
}

pub(crate) struct Shaper {
    font: *const Font,
    /// The raw byte data of the font
    font_data: FontData,
    /// The index of a font in it's collection (.ttc)
    /// If the font file is not a collection then this is 0
    font_index: u32,
    // Used for scaling HarfRust's output
    scale: f64,

    /// Pre-computed data for shaping a font
    shaper_data: ShaperData,

    /// Pre-computed data for shaping a variable font with a particular set of variations.
    /// If there are no variations then we don't create a ShaperInstance.
    shaper_instance: Option<ShaperInstance>,
}

// `Font` and `FontData` are both threadsafe, so we can make the data structures here as thread-safe as well.
#[allow(unsafe_code)]
unsafe impl Sync for Shaper {}
#[allow(unsafe_code)]
unsafe impl Send for Shaper {}

impl Shaper {
    pub(crate) fn new(font: &Font) -> Self {
        let raw_font = font
            .font_data_and_index()
            .expect("Error creating shaper for font");
        let font_data = raw_font.data.clone();
        let font_index = raw_font.index;

        // Set points-per-em. if zero, performs no hinting in that direction
        let ppem = font.descriptor.pt_size.to_f64_px();
        let font_ref = read_fonts::FontRef::from_index(font_data.as_ref(), font_index).unwrap();
        let units_per_em = font_ref.head().unwrap().units_per_em();
        let scale = ppem / (units_per_em as f64);

        // Create cached shaping data for the font
        let hr_font = HarfRustFontRef::from_index(font_data.as_ref(), font_index).unwrap();
        let shaper_data = ShaperData::new(&hr_font);

        // If variable fonts are enabled and the font has variations, create a ShaperInstance to set on the shaper.
        let variations = font.variations();
        let shaper_instance =
            if servo_config::pref!(layout_variable_fonts_enabled) && !variations.is_empty() {
                let variations_iter = variations.iter().copied().map(wr_variation_to_hr_varation);
                Some(ShaperInstance::from_variations(&hr_font, variations_iter))
            } else {
                None
            };

        Self {
            font: font as *const Font,
            font_data,
            font_index,
            scale,
            shaper_data,
            shaper_instance,
        }
    }
}

impl Shaper {
    fn shaped_glyph_data(&self, text: &str, options: &crate::ShapingOptions) -> ShapedGlyphData {
        let mut buffer = UnicodeBuffer::new();

        // Set direction
        buffer.set_direction(if options.flags.contains(ShapingFlags::RTL_FLAG) {
            harfrust::Direction::RightToLeft
        } else {
            harfrust::Direction::LeftToRight
        });

        // Set script
        let script_tag = Tag::from_u32(unicode_script_to_iso15924_tag(options.script));
        let script = Script::from_iso15924_tag(script_tag).unwrap();
        buffer.set_script(script);

        // Push text
        buffer.push_str(text);

        // Features
        let mut features = SmallVec::<[Feature; 2]>::new();
        if options
            .flags
            .contains(ShapingFlags::IGNORE_LIGATURES_SHAPING_FLAG)
        {
            features.push(Feature::new(Tag::new(b"liga"), 0, ..));
        }
        if options
            .flags
            .contains(ShapingFlags::DISABLE_KERNING_SHAPING_FLAG)
        {
            features.push(Feature::new(Tag::new(b"kern"), 0, ..));
        }

        let hr_font =
            HarfRustFontRef::from_index(self.font_data.as_ref(), self.font_index).unwrap();
        let shaper = self
            .shaper_data
            .shaper(&hr_font)
            .instance(self.shaper_instance.as_ref())
            .build();

        let glyph_buffer = shaper.shape(buffer, &features);

        ShapedGlyphData {
            data: glyph_buffer,
            scale: self.scale,
        }
    }

    #[allow(unsafe_code)]
    fn font(&self) -> &Font {
        // SAFETY: the font actually owns this shaper so it cannot have been dropped
        unsafe { &(*self.font) }
    }

    pub(crate) fn shape_text(&self, text: &str, options: &ShapingOptions, glyphs: &mut GlyphStore) {
        let glyph_data = self.shaped_glyph_data(text, options);
        let font = self.font();
        super::shape_text_harfbuzz(&glyph_data, font, text, options, glyphs);
    }

    pub(crate) fn baseline(&self) -> Option<crate::FontBaseline> {
        let font_ref =
            read_fonts::FontRef::from_index(self.font_data.as_ref(), self.font_index).unwrap();

        // Load the horizontal axis of the BASE table
        let base_table = font_ref.base().ok()?;
        let horiz_axis = base_table.horiz_axis()?.ok()?;

        // Get the index of each baseline tag in the tag list
        let tag_list = horiz_axis.base_tag_list()?.ok()?;
        let baseline_tags = tag_list.baseline_tags();
        let romn_tag_idx = baseline_tags
            .binary_search(&BigEndian::from(Tag::new(b"romn")))
            .ok();
        let hang_tag_idx = baseline_tags
            .binary_search(&BigEndian::from(Tag::new(b"hang")))
            .ok();
        let ideo_tag_idx = baseline_tags
            .binary_search(&BigEndian::from(Tag::new(b"ideo")))
            .ok();

        // Bail early if none of the baseline tags exist in the tag list
        if romn_tag_idx.is_none() && hang_tag_idx.is_none() && ideo_tag_idx.is_none() {
            return None;
        }

        // Find the DFLT (default) script record
        let script_list = horiz_axis.base_script_list().ok()?;
        let script_records = script_list.base_script_records();
        let default_script_record_idx = script_records
            .binary_search_by_key(&Tag::from_be_bytes(*b"DFLT"), |record| {
                record.base_script_tag()
            })
            .ok()?;
        let default_script_record = &script_records[default_script_record_idx];

        // Lookup the baseline coordinates DFLT script record
        let base_script = default_script_record
            .base_script(script_list.offset_data())
            .ok()?;
        let base_script_values = base_script.base_values()?.ok()?;
        let base_coords = base_script_values.base_coords();

        // Search for the baseline corresponding to a given baseline index
        let get_coord = |idx: usize| -> Option<f32> {
            base_coords
                .get(idx)
                .ok()
                .map(|coord| coord.coordinate() as f32 * self.scale as f32)
        };

        Some(FontBaseline {
            ideographic_baseline: ideo_tag_idx.and_then(get_coord).unwrap_or(0.0),
            alphabetic_baseline: romn_tag_idx.and_then(get_coord).unwrap_or(0.0),
            hanging_baseline: hang_tag_idx.and_then(get_coord).unwrap_or(0.0),
        })
    }
}
