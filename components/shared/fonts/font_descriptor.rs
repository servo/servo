/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::{Deref, RangeInclusive};

use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};
use style::computed_values::font_variant_caps;
use style::font_face::{FontFaceRuleData, FontStyle as FontFaceStyle};
use style::properties::style_structs::Font as FontStyleStruct;
use style::values::computed::font::{FixedPoint, FontStyleFixedPoint};
use style::values::computed::{Au, FontStretch, FontStyle, FontWeight, FontSynthesis};
use style::values::specified::FontStretch as SpecifiedFontStretch;
use webrender_api::FontVariation;

/// `FontDescriptor` describes the parameters of a `Font`. It represents rendering a given font
/// template at a particular size, with a particular font-variant-caps applied, etc. This contrasts
/// with `FontTemplateDescriptor` in that the latter represents only the parameters inherent in the
/// font data (weight, stretch, etc.).
#[derive(Clone, Debug, Deserialize, Hash, MallocSizeOf, PartialEq, Serialize)]
pub struct FontDescriptor {
    pub weight: FontWeight,
    pub stretch: FontStretch,
    pub style: FontStyle,
    pub variant: font_variant_caps::T,
    pub pt_size: Au,
    pub variation_settings: Vec<FontVariation>,
    pub synthesis_weight: FontSynthesis,
}

impl Eq for FontDescriptor {}

impl<'a> From<&'a FontStyleStruct> for FontDescriptor {
    fn from(style: &'a FontStyleStruct) -> Self {
        let variation_settings = style
            .clone_font_variation_settings()
            .0
            .into_iter()
            .map(|setting| FontVariation {
                tag: setting.tag.0,
                value: setting.value,
            })
            .collect();
        let synthesis_weight = style.clone_font_synthesis_weight();
        FontDescriptor {
            weight: style.font_weight,
            stretch: style.font_stretch,
            style: style.font_style,
            variant: style.font_variant_caps,
            pt_size: Au::from_f32_px(style.font_size.computed_size().px()),
            variation_settings,
            synthesis_weight,
        }
    }
}

/// A version of `FontStyle` from Stylo that is serializable. Normally this is not
/// because the specified version of `FontStyle` contains floats.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ComputedFontStyleDescriptor {
    Italic,
    Oblique(FontStyleFixedPoint, FontStyleFixedPoint),
}

/// This data structure represents the various optional descriptors that can be
/// applied to a `@font-face` rule in CSS. These are used to create a [`FontTemplate`]
/// from the given font data used as the source of the `@font-face` rule. If values
/// like weight, stretch, and style are not specified they are initialized based
/// on the contents of the font itself.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CSSFontFaceDescriptors {
    pub family_name: LowercaseFontFamilyName,
    pub weight: Option<(FontWeight, FontWeight)>,
    pub stretch: Option<(FontStretch, FontStretch)>,
    pub style: Option<ComputedFontStyleDescriptor>,
    pub unicode_range: Option<Vec<RangeInclusive<u32>>>,
}

impl CSSFontFaceDescriptors {
    pub fn new(family_name: &str) -> Self {
        CSSFontFaceDescriptors {
            family_name: family_name.into(),
            ..Default::default()
        }
    }
}

impl From<&FontFaceRuleData> for CSSFontFaceDescriptors {
    fn from(rule_data: &FontFaceRuleData) -> Self {
        let family_name = rule_data
            .family
            .as_ref()
            .expect("Expected rule to contain a font family.")
            .name
            .clone();
        let weight = rule_data
            .weight
            .as_ref()
            .map(|weight_range| (weight_range.0.compute(), weight_range.1.compute()));

        let stretch_to_computed = |specified: SpecifiedFontStretch| match specified {
            SpecifiedFontStretch::Stretch(percentage) => {
                FontStretch::from_percentage(percentage.compute().0)
            },
            SpecifiedFontStretch::Keyword(keyword) => keyword.compute(),
            SpecifiedFontStretch::System(_) => FontStretch::NORMAL,
        };
        let stretch = rule_data.stretch.as_ref().map(|stretch_range| {
            (
                stretch_to_computed(stretch_range.0),
                stretch_to_computed(stretch_range.1),
            )
        });

        fn style_to_computed(specified: &FontFaceStyle) -> ComputedFontStyleDescriptor {
            match specified {
                FontFaceStyle::Italic => ComputedFontStyleDescriptor::Italic,
                FontFaceStyle::Oblique(angle_a, angle_b) => ComputedFontStyleDescriptor::Oblique(
                    FixedPoint::from_float(angle_a.degrees()),
                    FixedPoint::from_float(angle_b.degrees()),
                ),
            }
        }
        let style = rule_data.style.as_ref().map(style_to_computed);
        let unicode_range = rule_data
            .unicode_range
            .as_ref()
            .map(|ranges| ranges.iter().map(|range| range.start..=range.end).collect());

        CSSFontFaceDescriptors {
            family_name: family_name.into(),
            weight,
            stretch,
            style,
            unicode_range,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
pub struct LowercaseFontFamilyName {
    inner: String,
}

impl<T: AsRef<str>> From<T> for LowercaseFontFamilyName {
    fn from(value: T) -> Self {
        LowercaseFontFamilyName {
            inner: value.as_ref().to_lowercase(),
        }
    }
}

impl Deref for LowercaseFontFamilyName {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        &self.inner
    }
}

impl std::fmt::Display for LowercaseFontFamilyName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}
