/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use app_units::Au;
use euclid::default::Point2D;
use harfrust::font::BuiltinFontFuncs;
use harfrust::{
    BufferClusterLevel, Feature, FontRef as HarfRustFontRef, GlyphBuffer, Language, Script,
    ShapeOptions, ShaperData, ShaperInstance, Tag, UnicodeBuffer, Variation,
};
use num_traits::Zero as _;
use read_fonts::TableProvider;
use read_fonts::types::{BigEndian, GlyphId};

use super::{GlyphShapingResult, unicode_script_to_iso15924_tag};
use crate::{
    Font, FontBaseline, FontData, ShapedGlyph, ShapedText, ShapingFlags, ShapingOptions,
    fixed_to_float, float_to_fixed,
};

/// Convert a `webrender_api::FontVariation` to a `harfrust::Variation`
fn wr_variation_to_hr_varation(wr_variation: webrender_api::FontVariation) -> harfrust::Variation {
    Variation {
        tag: Tag::from_u32(wr_variation.tag),
        value: wr_variation.value,
    }
}

pub(crate) struct HarfrustGlyphShapingResult {
    data: GlyphBuffer,
}

struct ShapedGlyphIterator<'a> {
    shaped_glyph_data: &'a HarfrustGlyphShapingResult,
    current_glyph_offset: usize,
    y_position: Au,
}

impl<'a> Iterator for ShapedGlyphIterator<'a> {
    type Item = ShapedGlyph;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_glyph_offset >= self.shaped_glyph_data.len() {
            return None;
        }

        // Increment current_glyph_offset before the potential early returns
        // when accessing the glyph info below
        let offset = self.current_glyph_offset;
        self.current_glyph_offset += 1;

        let glyph_info_i = &self.shaped_glyph_data.data.glyph_infos().get(offset)?;
        let pos_info_i = &self.shaped_glyph_data.data.glyph_positions().get(offset)?;

        let x_offset = Au::from_f64_px(Shaper::fixed_to_float(pos_info_i.x_offset));
        let y_offset = Au::from_f64_px(Shaper::fixed_to_float(pos_info_i.y_offset));
        let x_advance = Au::from_f64_px(Shaper::fixed_to_float(pos_info_i.x_advance));
        let y_advance = Au::from_f64_px(Shaper::fixed_to_float(pos_info_i.y_advance));

        let offset = if x_offset.is_zero() && y_offset.is_zero() && y_advance.is_zero() {
            None
        } else {
            // adjust the pen..
            if y_advance > Au::zero() {
                self.y_position -= y_advance;
            }

            Some(Point2D::new(x_offset, self.y_position - y_offset))
        };

        Some(ShapedGlyph {
            glyph_id: glyph_info_i.glyph_id,
            string_byte_offset: glyph_info_i.cluster as usize,
            advance: x_advance,
            offset,
        })
    }
}

impl GlyphShapingResult for HarfrustGlyphShapingResult {
    fn len(&self) -> usize {
        self.data.len()
    }

    fn is_rtl(&self) -> bool {
        let glyph_infos = self.data.glyph_infos();
        if let Some(first) = glyph_infos.first() &&
            let Some(last) = glyph_infos.last()
        {
            last.cluster < first.cluster
        } else {
            false
        }
    }

    fn iter(&self) -> impl Iterator<Item = ShapedGlyph> {
        ShapedGlyphIterator {
            shaped_glyph_data: self,
            current_glyph_offset: 0,
            y_position: Au::zero(),
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
    // Point-per-em (i.e. the font size)
    ppem: f64,

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
            ppem,
            shaper_data,
            shaper_instance,
        }
    }
}

impl Shaper {
    pub(crate) fn shaped_glyph_data(
        &self,
        text: &str,
        options: &crate::ShapingOptions,
        font_features: &[(Tag, u32)],
    ) -> HarfrustGlyphShapingResult {
        let mut buffer = UnicodeBuffer::new();

        // Set cluster level
        buffer.set_cluster_level(BufferClusterLevel::MonotoneCharacters);

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

        // Set language
        let language = options.language.as_str().parse::<Language>().unwrap();
        buffer.set_language(language);

        // Push text
        buffer.push_str(text);

        let features: Vec<_> = font_features
            .iter()
            .map(|(tag, value)| Feature::new(*tag, *value, ..))
            .collect();

        let hr_font =
            HarfRustFontRef::from_index(self.font_data.as_ref(), self.font_index).unwrap();
        let shaper = self
            .shaper_data
            .shaper(&hr_font)
            .instance(self.shaper_instance.as_ref())
            .build();

        let mut font_funcs = self.font_funcs();
        let glyph_buffer = shaper.shape(
            buffer,
            ShapeOptions::new()
                .scale(Some(Shaper::float_to_fixed(self.ppem)))
                .features(&features)
                .font_funcs(Some(&mut font_funcs)),
        );

        HarfrustGlyphShapingResult { data: glyph_buffer }
    }

    fn font_funcs(&self) -> FontFuncs<'_> {
        FontFuncs { font: self.font() }
    }

    #[allow(unsafe_code)]
    pub(crate) fn font(&self) -> &Font {
        // SAFETY: the font actually owns this shaper so it cannot have been dropped
        assert!(!self.font.is_null());
        unsafe { &(*self.font) }
    }

    #[allow(dead_code)]
    pub(crate) fn shape_text(
        &self,
        text: &str,
        options: &ShapingOptions,
        font_features: &[(Tag, u32)],
    ) -> ShapedText {
        ShapedText::with_shaped_glyph_data(
            text,
            options,
            &self.shaped_glyph_data(text, options, font_features),
        )
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
                .map(|coord| fixed_to_float(8, coord.coordinate() as i32) as f32)
        };

        Some(FontBaseline {
            ideographic_baseline: ideo_tag_idx.and_then(get_coord).unwrap_or(0.0),
            alphabetic_baseline: romn_tag_idx.and_then(get_coord).unwrap_or(0.0),
            hanging_baseline: hang_tag_idx.and_then(get_coord).unwrap_or(0.0),
        })
    }

    fn float_to_fixed(f: f64) -> i32 {
        float_to_fixed(16, f)
    }

    fn fixed_to_float(i: i32) -> f64 {
        fixed_to_float(16, i)
    }
}

struct FontFuncs<'a> {
    font: &'a Font,
}

impl harfrust::font::FontFuncs for FontFuncs<'_> {
    /// Nominal character-to-glyph mapping callback.
    fn nominal_glyph(&mut self, _builtin: &BuiltinFontFuncs, c: u32) -> Option<GlyphId> {
        self.font
            .glyph_index(char::from_u32(c).unwrap())
            .map(GlyphId::new)
    }

    /// Horizontal advance callback.
    ///
    /// See "Metrics scaling" in the [trait-level docs](FontFuncs) for details
    /// on what value this method should return.
    fn advance_width(&mut self, _builtin: &BuiltinFontFuncs, glyph: GlyphId) -> i32 {
        Shaper::float_to_fixed(self.font.glyph_h_advance(glyph.to_u32()))
    }
}
