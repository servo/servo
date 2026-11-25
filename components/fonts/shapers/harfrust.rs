/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use app_units::Au;
use euclid::default::Point2D;
use fonts_traits::SkrifaFont;
use harfrust::{Buffer, ClusterLevel, Feature, Language, Script, ShapeOptions, ShaperFont, Tag};
use num_traits::Zero as _;
use read_fonts::types::GlyphId;

use super::{GlyphShapingResult, unicode_script_to_iso15924_tag};
use crate::{
    Font, ShapedGlyph, ShapedText, ShapingFlags, ShapingOptions, fixed_to_float, float_to_fixed,
};

pub(crate) struct HarfrustGlyphShapingResult {
    data: Buffer,
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
    /// The `skrifa::Font` instance to use for this [`Shaper`].
    skrifa_font: Arc<SkrifaFont>,
    /// Point per em (i.e. the font size).
    ppem: f64,
}

impl Shaper {
    pub(crate) fn new(font: &Font) -> Self {
        Self {
            // Set points-per-em. If zero, performs no hinting in that direction.
            ppem: font.descriptor.pt_size.to_f64_px(),
            skrifa_font: font.skrifa_font().clone(),
        }
    }
}

impl Shaper {
    pub(crate) fn shaped_glyph_data(
        &self,
        font: &Font,
        text: &str,
        options: &crate::ShapingOptions,
        font_features: &[(Tag, u32)],
    ) -> HarfrustGlyphShapingResult {
        let mut buffer = Buffer::new();

        buffer.set_cluster_level(ClusterLevel::MonotoneCharacters);
        buffer.set_direction(if options.flags.contains(ShapingFlags::RTL_FLAG) {
            harfrust::Direction::RightToLeft
        } else {
            harfrust::Direction::LeftToRight
        });

        let script_tag = Tag::from_u32(unicode_script_to_iso15924_tag(options.script));
        buffer.set_script(Script::from_iso15924_tag(script_tag));

        let language = match options.language_override {
            Some(language_override) => Language::from_tag(language_override),
            None => options.language.as_str().parse::<Language>().ok(),
        };
        buffer.set_language(language);

        buffer.push_str(text);

        let features: Vec<_> = font_features
            .iter()
            .map(|(tag, value)| Feature::new(*tag, *value, ..))
            .collect();

        let font_funcs = FontFuncs { font };
        let shaper_font = ShaperFont::new(&self.skrifa_font)
            .with_scale(Shaper::float_to_fixed(self.ppem))
            .with_font_funcs(Some(&font_funcs));

        harfrust::shape(
            &shaper_font,
            &mut buffer,
            ShapeOptions::new().features(&features),
        )
        .expect("Error shaping text with HarfRust");

        HarfrustGlyphShapingResult { data: buffer }
    }

    #[allow(dead_code)]
    pub(crate) fn shape_text(
        &self,
        font: &Font,
        text: &str,
        options: &ShapingOptions,
        font_features: &[(Tag, u32)],
    ) -> ShapedText {
        ShapedText::with_shaped_glyph_data(
            text,
            options,
            &self.shaped_glyph_data(font, text, options, font_features),
        )
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

impl harfrust::FontFuncs for FontFuncs<'_> {
    /// Nominal character-to-glyph mapping callback.
    fn nominal_glyph(&self, _font: &ShaperFont, c: u32) -> Option<GlyphId> {
        self.font.glyph_index(char::from_u32(c)?).map(GlyphId::new)
    }

    /// Horizontal advance callback.
    ///
    /// See "Metrics scaling" in the [trait-level docs](FontFuncs) for details
    /// on what value this method should return.
    fn glyph_h_advance(&self, _font: &ShaperFont, glyph: GlyphId) -> i32 {
        Shaper::float_to_fixed(self.font.glyph_h_advance(glyph.to_u32()))
    }
}
