/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod harfbuzz;

use app_units::Au;
use euclid::default::Point2D;
pub(crate) use harfbuzz::Shaper;
use read_fonts::types::Tag;
use rustc_hash::FxHashMap;
use style::computed_values::font_variant_position::T as FontVariantPosition;
use style::font_face::FontFaceRule;
use style::values::computed::{FontVariantEastAsian, FontVariantLigatures, FontVariantNumeric};

use crate::{
    AFRC, CALT, CLIG, DLIG, FRAC, FWID, GlyphId, HLIG, JP04, JP78, JP83, JP90, KERN, LIGA, LNUM,
    ONUM, ORDN, PNUM, PWID, RUBY, SMPL, SUBS, SUPS, ShapingFlags, ShapingOptions, TNUM, TRAD, ZERO,
};

/// Utility function to convert a `unicode_script::Script` enum into the corresponding `c_uint` tag that
/// harfbuzz uses to represent unicode scipts.
fn unicode_script_to_iso15924_tag(script: unicode_script::Script) -> u32 {
    let bytes: [u8; 4] = match script {
        unicode_script::Script::Unknown => *b"Zzzz",
        _ => {
            let short_name = script.short_name();
            short_name.as_bytes().try_into().unwrap()
        },
    };

    u32::from_be_bytes(bytes)
}

#[derive(Debug)]
pub(crate) struct ShapedGlyph {
    /// The actual glyph to render for this [`ShapedGlyph`].
    pub glyph_id: GlyphId,
    /// The original byte offset in the input buffer of the character that this
    /// glyph belongs to. More than one glyph can share the same character and
    /// one character can produce multiple glyphs.
    pub string_byte_offset: usize,
    /// The advance the direction of the writing mode that this glyph needs.
    pub advance: Au,
    /// An offset that should be applied when rendering this glyph.
    pub offset: Option<Point2D<Au>>,
}

/// Holds the results of shaping. Abstracts over HarfBuzz and HarfRust which return data in very similar
/// form but with different types
pub(crate) trait GlyphShapingResult {
    /// The number of shaped glyphs
    fn len(&self) -> usize;
    /// Whether or not the result is right-to-left.
    fn is_rtl(&self) -> bool;
    /// An iterator of the shaped glyphs of this data.
    fn iter(&self) -> impl Iterator<Item = ShapedGlyph>;
}

/// Determine which OpenType features are applied for the font.
///
/// The order of precedence for resolving font-specific font feature properties is specified in
/// <https://drafts.csswg.org/css-fonts-4/#apply-font-matching-variations>.
fn compute_used_font_features(
    options: &ShapingOptions,
    font_face_rule: Option<&FontFaceRule>,
) -> impl Iterator<Item = (Tag, u32)> {
    let mut features = FxHashMap::default();

    let mut add_feature = |tag, value| {
        features.entry(tag).insert_entry(value);
    };

    // Step 1. Font features enabled by default are applied, including features required
    // for a given script. See § 7.1 Default features for a description of these.
    add_feature(LIGA, 1);
    add_feature(CLIG, 1);

    // Step 7. If the font is defined via an @font-face rule, the font features implied
    // by the font-feature-settings descriptor in the @font-face rule are applied.
    if let Some(font_feature_settings) =
        font_face_rule.and_then(|rule| rule.descriptors.font_feature_settings.as_ref())
    {
        for feature_setting in font_feature_settings.0.iter() {
            add_feature(
                Tag::from_u32(feature_setting.tag.0),
                feature_setting.value.value() as u32,
            )
        }
    }

    // Step 10. Font features implied by the value of the font-variant property,
    // the related font-variant subproperties and any other CSS property that uses
    // OpenType features (e.g. the font-kerning property) are applied.
    if options.ligatures == FontVariantLigatures::NONE {
        add_feature(LIGA, 0);
        add_feature(CLIG, 0);
        add_feature(DLIG, 0);
        add_feature(HLIG, 0);
        add_feature(CALT, 0);
    } else {
        if options
            .ligatures
            .contains(FontVariantLigatures::COMMON_LIGATURES)
        {
            add_feature(LIGA, 1);
            add_feature(CLIG, 1);
        } else if options
            .ligatures
            .contains(FontVariantLigatures::NO_COMMON_LIGATURES)
        {
            add_feature(LIGA, 0);
            add_feature(CLIG, 0);
        }

        if options
            .ligatures
            .contains(FontVariantLigatures::DISCRETIONARY_LIGATURES)
        {
            add_feature(DLIG, 1);
        } else if options
            .ligatures
            .contains(FontVariantLigatures::NO_DISCRETIONARY_LIGATURES)
        {
            add_feature(DLIG, 0);
        }

        if options
            .ligatures
            .contains(FontVariantLigatures::HISTORICAL_LIGATURES)
        {
            add_feature(HLIG, 1);
        } else if options
            .ligatures
            .contains(FontVariantLigatures::NO_HISTORICAL_LIGATURES)
        {
            add_feature(HLIG, 0);
        }

        if options.ligatures.contains(FontVariantLigatures::CONTEXTUAL) {
            add_feature(CALT, 1);
        } else if options
            .ligatures
            .contains(FontVariantLigatures::NO_CONTEXTUAL)
        {
            add_feature(CALT, 0);
        }
    }

    if options.numeric != FontVariantNumeric::NORMAL {
        if options.numeric.contains(FontVariantNumeric::LINING_NUMS) {
            add_feature(LNUM, 1);
        } else if options.numeric.contains(FontVariantNumeric::OLDSTYLE_NUMS) {
            add_feature(ONUM, 1);
        }
        if options
            .numeric
            .contains(FontVariantNumeric::PROPORTIONAL_NUMS)
        {
            add_feature(PNUM, 1);
        } else if options.numeric.contains(FontVariantNumeric::TABULAR_NUMS) {
            add_feature(TNUM, 1);
        }
        if options
            .numeric
            .contains(FontVariantNumeric::DIAGONAL_FRACTIONS)
        {
            add_feature(FRAC, 1);
        } else if options
            .numeric
            .contains(FontVariantNumeric::STACKED_FRACTIONS)
        {
            add_feature(AFRC, 1);
        }
        if options.numeric.contains(FontVariantNumeric::ORDINAL) {
            add_feature(ORDN, 1);
        }
        if options.numeric.contains(FontVariantNumeric::SLASHED_ZERO) {
            add_feature(ZERO, 1);
        }
    }

    if options.east_asian != FontVariantEastAsian::NORMAL {
        if options.east_asian.contains(FontVariantEastAsian::JIS78) {
            add_feature(JP78, 1);
        } else if options.east_asian.contains(FontVariantEastAsian::JIS83) {
            add_feature(JP83, 1);
        } else if options.east_asian.contains(FontVariantEastAsian::JIS90) {
            add_feature(JP90, 1);
        } else if options.east_asian.contains(FontVariantEastAsian::JIS04) {
            add_feature(JP04, 1);
        } else if options
            .east_asian
            .contains(FontVariantEastAsian::SIMPLIFIED)
        {
            add_feature(SMPL, 1);
        } else if options
            .east_asian
            .contains(FontVariantEastAsian::TRADITIONAL)
        {
            add_feature(TRAD, 1);
        }

        if options
            .east_asian
            .contains(FontVariantEastAsian::FULL_WIDTH)
        {
            add_feature(FWID, 1);
        } else if options
            .east_asian
            .contains(FontVariantEastAsian::PROPORTIONAL_WIDTH)
        {
            add_feature(PWID, 1);
        }

        if options.east_asian.contains(FontVariantEastAsian::RUBY) {
            add_feature(RUBY, 1);
        }
    }

    match options.position {
        FontVariantPosition::Normal => {},
        FontVariantPosition::Sub => add_feature(SUBS, 1),
        FontVariantPosition::Super => add_feature(SUPS, 1),
    }

    if options
        .flags
        .contains(ShapingFlags::DISABLE_KERNING_SHAPING_FLAG)
    {
        add_feature(KERN, 0);
    }

    // Step 13. Font features implied by the value of font-feature-settings property are applied.
    for feature_setting in options.feature_settings.0.iter() {
        add_feature(
            Tag::from_u32(feature_setting.tag.0),
            feature_setting.value as u32,
        )
    }

    features.into_iter()
}
