/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Deref;
use std::sync::{Arc, OnceLock};
use std::{iter, str};

use app_units::Au;
use atomic_refcell::AtomicRef;
use bitflags::bitflags;
use euclid::default::{Point2D, Rect};
use euclid::num::Zero;
use font_types::NameId;
use fonts_traits::FontDescriptor;
use fonts_traits::tag::{CBDT, COLR, GPOS, GSUB, ITAL, KERN, SBIX, SLNT, WDTH, WGHT};
use icu_locale_core::subtags::Language;
use icu_properties::props::{EnumeratedProperty, GeneralCategory};
use log::debug;
use malloc_size_of_derive::MallocSizeOf;
use parking_lot::RwLock;
use read_fonts::collections::int_set::Domain;
use read_fonts::types::Tag;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use servo_base::id::PainterId;
use servo_base::text::{UnicodeBlock, UnicodeBlockMethod};
use skrifa::Font as SkrifaFont;
use skrifa::font::Variation;
use skrifa::font::attribute::Weight;
use smallvec::SmallVec;
use style::Atom;
use style::computed_values::font_optical_sizing::T as FontOpticalSizing;
use style::computed_values::font_variant_caps;
use style::computed_values::font_variant_position::T as FontVariantPosition;
use style::properties::style_structs::Font as FontStyleStruct;
use style::values::computed::font::{
    FamilyName, FontFamilyNameSyntax, GenericFontFamily, SingleFontFamily,
};
use style::values::computed::{
    FontFeatureSettings, FontStyle, FontSynthesis, FontVariantEastAsian, FontVariantLigatures,
    FontVariantNumeric, FontWeight,
};
use unicode_script::Script;
use webrender_api::{
    FontInstanceFlags, FontInstanceKey, FontInstancePlatformOptions, FontVariation,
};

use crate::font_context::SkrifaFontCache;
use crate::font_feature_values::ResolvedFontVariantAlternates;
use crate::platform::font::{FontTable, PlatformFont};
use crate::platform::font_list::fallback_font_families;
use crate::{
    EmojiPresentationPreference, FallbackFontSelectionOptions, FontContext, FontData,
    FontDataAndIndex, FontDataError, FontIdentifier, FontTemplateRef, FontTemplateRefMethods,
    GlyphId, LocalFontIdentifier, ShapedGlyph, ShapedText, Shaper, compute_used_font_features,
};
pub const LAST_RESORT_GLYPH_ADVANCE: FractionalPixel = 10.0;

// PlatformFont encapsulates access to the platform's font API,
// e.g. quartz, FreeType. It provides access to metrics and tables
// needed by the text shaper as well as access to the underlying font
// resources needed by the graphics layer to draw glyphs.

pub trait PlatformFontMethods: Sized {
    #[servo_tracing::instrument(name = "PlatformFontMethods::new_from_template", skip_all)]
    fn new_from_template(
        template: FontTemplateRef,
        pt_size: Option<Au>,
        data: &Option<FontData>,
    ) -> Result<PlatformFont, &'static str> {
        let template = template.borrow();
        let font_identifier = template.identifier.clone();

        match font_identifier {
            FontIdentifier::Local(font_identifier) => {
                Self::new_from_local_font_identifier(font_identifier, pt_size)
            },
            FontIdentifier::Web(_) | FontIdentifier::ArrayBuffer(_) => Self::new_from_data(
                font_identifier,
                data.as_ref()
                    .expect("Should never create a web font without data."),
                pt_size,
            ),
        }
    }

    fn new_from_local_font_identifier(
        font_identifier: LocalFontIdentifier,
        pt_size: Option<Au>,
    ) -> Result<PlatformFont, &'static str>;

    fn new_from_data(
        font_identifier: FontIdentifier,
        data: &FontData,
        pt_size: Option<Au>,
    ) -> Result<PlatformFont, &'static str>;

    /// Create a [`skrifa::Font`] for this `PlatformFont`. Generally, this should not be
    /// used and you should try to retrieve the cached version of the [`SkrifaFont`] from
    /// the `FontContext`.
    fn create_skrifa_font(&self) -> Option<SkrifaFont>;

    /// Create a platform font with the given variations from an existing font.
    ///
    /// `self` is consumed to work around platform differences. On some platforms, changing the
    /// variations requires creating an entirely new font face, whereas on others the returned
    /// font is `self`.
    fn copy_with_variations(
        self,
        _font_identifier: &FontIdentifier,
        _variations: &[FontVariation],
    ) -> Result<Self, &'static str>;

    /// Create a platform font with synthetic bold from an existing font.
    ///
    /// `self` is consumed to work around platform differences. On some platforms, making a font
    /// have synthetic bold requires creating an entirely new font face, whereas on others the
    /// returned font is `self`.
    fn copy_with_synthetic_bold(self) -> Result<Self, &'static str>;

    fn glyph_index(&self, codepoint: char) -> Option<GlyphId>;
    fn glyph_h_advance(&self, _: GlyphId) -> Option<FractionalPixel>;
    fn glyph_h_kerning(&self, glyph0: GlyphId, glyph1: GlyphId) -> FractionalPixel;

    fn metrics(&self) -> FontMetrics;
    fn table_for_tag(&self, _: Tag) -> Option<FontTable>;
    fn typographic_bounds(&self, _: GlyphId) -> Rect<f32>;

    /// Get the necessary [`FontInstanceFlags`] for this font.
    fn webrender_font_instance_flags(&self) -> FontInstanceFlags;

    /// Get the necessary [`FontInstancePlatformOptions`] for this font.
    fn webrender_font_instance_platform_options(&self) -> FontInstancePlatformOptions {
        Default::default()
    }

    /// Return all the variation values that the font was instantiated with.
    fn variations(&self) -> &[FontVariation];
}

// Used to abstract over the shaper's choice of fixed int representation.
pub(crate) type FractionalPixel = f64;

pub(crate) trait FontTableMethods {
    fn buffer(&self) -> &[u8];
}

#[derive(Clone, Debug, Default, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub struct FontMetrics {
    pub underline_size: Au,
    pub underline_offset: Au,
    pub strikeout_size: Au,
    pub strikeout_offset: Au,
    pub leading: Au,
    pub x_height: Au,
    pub em_size: Au,
    pub ascent: Au,
    pub descent: Au,
    pub max_advance: Au,
    pub average_advance: Au,
    pub line_gap: Au,
    pub zero_horizontal_advance: Option<Au>,
    pub ic_horizontal_advance: Option<Au>,
    /// The advance of the space character (' ') in this font or if there is no space,
    /// the average char advance.
    pub space_advance: Au,
}

impl FontMetrics {
    /// Create an empty [`FontMetrics`] mainly to be used in situations where
    /// no font can be found.
    pub fn empty() -> Arc<Self> {
        static EMPTY: OnceLock<Arc<FontMetrics>> = OnceLock::new();
        EMPTY.get_or_init(Default::default).clone()
    }

    /// Whether or not the block metrics of the two `FontMetrics` instances differ in a way
    /// that requires the resulting block size of a containing inline box to change.
    pub fn block_metrics_meaningfully_differ(&self, other: &Self) -> bool {
        self.ascent != other.ascent ||
            self.descent != other.descent ||
            self.line_gap != other.line_gap
    }
}

#[derive(Debug, Default)]
struct CachedShapeData {
    glyph_advances: FxHashMap<GlyphId, FractionalPixel>,
    glyph_indices: FxHashMap<char, Option<GlyphId>>,
    shaped_text: HashMap<ShapeCacheEntry, Arc<ShapedText>>,
}

impl malloc_size_of::MallocSizeOf for CachedShapeData {
    fn size_of(&self, ops: &mut malloc_size_of::MallocSizeOfOps) -> usize {
        // Estimate the size of the shaped text cache. This will be smaller, because
        // HashMap has some overhead, but we are mainly interested in the actual data.
        let shaped_text_size = self
            .shaped_text
            .iter()
            .map(|(key, value)| key.size_of(ops) + (*value).size_of(ops))
            .sum::<usize>();
        self.glyph_advances.size_of(ops) + self.glyph_indices.size_of(ops) + shaped_text_size
    }
}

pub struct Font {
    pub(crate) handle: PlatformFont,
    pub(crate) template: FontTemplateRef,
    pub metrics: OnceLock<Arc<FontMetrics>>,
    pub descriptor: FontDescriptor,

    /// The data for this font. And the index of the font within the data (in case it's a TTC)
    /// This might be uninitialized for system fonts.
    data_and_index: OnceLock<FontDataAndIndex>,

    /// A `skrifa` `Font` instance for this [`Font`] used to access table data in
    /// a platform-independent way.
    skrifa_font: SkrifaFont,

    shaper: OnceLock<Shaper>,
    cached_shape_data: RwLock<CachedShapeData>,
    font_instance_key: RwLock<FxHashMap<PainterId, FontInstanceKey>>,

    /// If this is a synthesized small caps font, then this font reference is for
    /// the version of the font used to replace lowercase ASCII letters. It's up
    /// to the consumer of this font to properly use this reference.
    pub(crate) synthesized_small_caps: Option<FontRef>,

    /// Whether or not this font supports color bitmaps or a COLR table. This is
    /// essentially equivalent to whether or not we use it for emoji presentation.
    /// This is cached, because getting table data is expensive.
    has_color_bitmap_or_colr_table: OnceLock<bool>,

    /// Whether or not this font can do fast shaping, ie whether or not it has
    /// a kern table, but no GSUB and GPOS tables. When this is true, Servo will
    /// shape Latin horizontal left-to-right text without using Harfbuzz.
    ///
    /// FIXME: This should be removed entirely in favor of better caching if necessary.
    /// See <https://github.com/servo/servo/pull/11273#issuecomment-222332873>.
    can_do_fast_shaping: OnceLock<bool>,

    /// The family name of the font.
    ///
    /// This is the name as it is declared in the `name` table, *not* the name provided by the system
    /// font service.
    family_name: OnceLock<Result<Atom, NoUsableFamilyName>>,
}

/// An error indicating that the `name` table contained no usable family names.
///
/// For example, this can happen if the family name uses an incompatible or unknown encoding.
#[derive(Clone)]
struct NoUsableFamilyName;

impl std::fmt::Debug for Font {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Font")
            .field("template", &self.template)
            .field("descriptor", &self.descriptor)
            .finish()
    }
}

impl malloc_size_of::MallocSizeOf for Font {
    fn size_of(&self, ops: &mut malloc_size_of::MallocSizeOfOps) -> usize {
        // TODO: Collect memory usage for platform fonts and for shapers.
        // This skips the template, because they are already stored in the template cache.

        let metrics_size = self.metrics.get().map_or(0, |metrics| metrics.size_of(ops));
        metrics_size +
            self.descriptor.size_of(ops) +
            self.cached_shape_data.read().size_of(ops) +
            self.font_instance_key
                .read()
                .values()
                .map(|key| key.size_of(ops))
                .sum::<usize>()
    }
}

impl Font {
    pub fn new(
        template: FontTemplateRef,
        descriptor: FontDescriptor,
        data: Option<FontData>,
        synthesized_small_caps: Option<FontRef>,
        skrifa_font_cache: &SkrifaFontCache,
    ) -> Result<Font, &'static str> {
        let mut handle =
            PlatformFont::new_from_template(template.clone(), Some(descriptor.pt_size), &data)?;

        let data_and_index = data
            .map(|data| OnceLock::from(FontDataAndIndex { data, index: 0 }))
            .unwrap_or_default();
        let Some(mut skrifa_font) =
            skrifa_font_cache.get(&template.identifier(), || handle.create_skrifa_font())
        else {
            return Err("Could not create Skrifa font for font");
        };

        let variation_axes = servo_config::pref!(layout_css_font_variations_enabled)
            .then(|| VariationAxes::from_skrifa_font(&skrifa_font))
            .unwrap_or_else(VariationAxes::empty);

        // Either apply variations or font synthesis, but not both. Variable fonts do not
        // count as font synthesis and their use is not affected by the font-synthesis
        // property. See <https://www.w3.org/TR/css-fonts-4/#font-synthesis-intro>.
        if variation_axes.contains(VariationAxes::IS_VARIABLE) {
            let used_variations = compute_variations(&descriptor, &template, variation_axes);
            handle = handle.copy_with_variations(&template.identifier(), &used_variations)?;
            skrifa_font = skrifa_font
                .instance_builder()
                .variations(used_variations.iter().map(|variation| Variation {
                    tag: Tag::from_u32(variation.tag),
                    value: variation.value,
                }))
                .build();
        } else {
            let template_descriptor = template.descriptor();
            let apply_synthetic_bold = descriptor.weight >= FontWeight::BOLD_THRESHOLD &&
                matches!(descriptor.synthesis_weight, FontSynthesis::Auto) &&
                template_descriptor.weight.1 < FontWeight::BOLD_THRESHOLD &&
                skrifa_font.default_attributes().weight < Weight::SEMI_BOLD;
            if apply_synthetic_bold {
                handle = handle.copy_with_synthetic_bold()?;
            }
        }

        Ok(Font {
            handle,
            template,
            metrics: OnceLock::new(),
            descriptor,
            data_and_index,
            skrifa_font,
            shaper: OnceLock::new(),
            cached_shape_data: Default::default(),
            font_instance_key: Default::default(),
            synthesized_small_caps,
            has_color_bitmap_or_colr_table: OnceLock::new(),
            can_do_fast_shaping: OnceLock::new(),
            family_name: Default::default(),
        })
    }

    /// A unique identifier for the font, allowing comparison.
    pub fn identifier(&self) -> AtomicRef<'_, FontIdentifier> {
        self.template.identifier()
    }

    pub fn metrics(&self) -> &Arc<FontMetrics> {
        self.metrics.get_or_init(|| Arc::new(self.handle.metrics()))
    }

    pub(crate) fn webrender_font_instance_flags(&self) -> FontInstanceFlags {
        self.handle.webrender_font_instance_flags()
    }

    pub(crate) fn webrender_font_instance_platform_options(&self) -> FontInstancePlatformOptions {
        self.handle.webrender_font_instance_platform_options()
    }

    pub(crate) fn has_color_bitmap_or_colr_table(&self) -> bool {
        *self.has_color_bitmap_or_colr_table.get_or_init(|| {
            self.table_for_tag(SBIX).is_some() ||
                self.table_for_tag(CBDT).is_some() ||
                self.table_for_tag(COLR).is_some()
        })
    }

    pub fn key(&self, painter_id: PainterId, font_context: &FontContext) -> FontInstanceKey {
        *self
            .font_instance_key
            .write()
            .entry(painter_id)
            .or_insert_with(|| font_context.create_font_instance_key(self, painter_id))
    }

    /// Return the data for this `Font`. Note that this is currently highly inefficient for system
    /// fonts and should not be used except in legacy canvas code.
    pub fn font_data_and_index(&self) -> Result<&FontDataAndIndex, FontDataError> {
        if let Some(data_and_index) = self.data_and_index.get() {
            return Ok(data_and_index);
        }

        let FontIdentifier::Local(local_font_identifier) = &*self.identifier() else {
            unreachable!("All web fonts should already have initialized data");
        };
        let Some(data_and_index) = local_font_identifier.font_data_and_index() else {
            return Err(FontDataError::FailedToLoad);
        };

        let data_and_index = self.data_and_index.get_or_init(move || data_and_index);
        Ok(data_and_index)
    }

    pub(crate) fn variations(&self) -> &[FontVariation] {
        self.handle.variations()
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    pub struct ShapingFlags: u8 {
        /// Set if we are to disable kerning.
        const DISABLE_KERNING_SHAPING_FLAG = 1 << 3;
        /// Text direction is right-to-left.
        const RTL_FLAG = 1 << 4;
        /// Set if word-break is set to keep-all.
        const KEEP_ALL_FLAG = 1 << 5;
    }
}

/// Various options that control text shaping.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ShapingOptions {
    /// Spacing to add between each letter. Corresponds to the CSS 2.1 `letter-spacing` property.
    ///
    /// Letter spacing is not applied to all characters. Use [Self::letter_spacing_for_character] to
    /// determine the amount of spacing to apply.
    pub letter_spacing: Au,
    /// Spacing to add between each word. Corresponds to the CSS 2.1 `word-spacing` property.
    pub word_spacing: Au,
    /// The Unicode script property of the characters in this run.
    pub script: Script,
    /// The preferred language, obtained from the `lang` attribute.
    pub language: Atom,
    /// The `font-language-override` value from style as an OpenType language tag or `None` when
    /// `normal`.
    pub language_override: Option<Tag>,
    /// The value of the `font-variant-ligatures` property.
    pub ligatures: FontVariantLigatures,
    /// The value of the `font-variant-numeric` property.
    pub numeric: FontVariantNumeric,
    /// The value of the `font-variant-east-asian` property.
    pub east_asian: FontVariantEastAsian,
    /// The value of the `font-feature-settings` property.
    pub feature_settings: FontFeatureSettings,
    /// The value of the `font-variant-position` property.
    pub position: FontVariantPosition,
    /// The value of the `font-variant-alternates` property.
    pub alternates: ResolvedFontVariantAlternates,
    /// Various flags.
    pub flags: ShapingFlags,
}

impl ShapingOptions {
    pub(crate) fn letter_spacing_for_character(&self, character: char) -> Au {
        // https://drafts.csswg.org/css-text/#letter-spacing-property
        // Letter spacing ignores invisible zero-width formatting characters (such as those from the Unicode Cf category).
        // Spacing must be added as if those characters did not exist in the document.
        if GeneralCategory::for_char(character) == GeneralCategory::Format {
            return Au::zero();
        }
        self.letter_spacing
    }
}

/// An entry in the shape cache.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct ShapeCacheEntry {
    text: String,
    letter_spacing: Au,
    word_spacing: Au,
    script: Script,
    language: Atom,
    language_override: Option<Tag>,
    font_features: Box<[(Tag, u32)]>,
    flags: ShapingFlags,
}

impl Font {
    #[servo_tracing::instrument(name = "Font::shape_text", skip_all)]
    pub fn shape_text(&self, text: &str, options: &ShapingOptions) -> Arc<ShapedText> {
        let font_features =
            compute_used_font_features(options, self.template.borrow().font_face_rule.as_deref())
                .collect();
        let lookup_key = ShapeCacheEntry {
            text: text.to_owned(),
            letter_spacing: options.letter_spacing,
            word_spacing: options.word_spacing,
            script: options.script,
            language: options.language.clone(),
            language_override: options.language_override,
            flags: options.flags,
            font_features,
        };

        if let Some(shaped_text) = self.cached_shape_data.read().shaped_text.get(&lookup_key) {
            return shaped_text.clone();
        }

        let glyphs = if self.can_do_fast_shaping(text, options) {
            debug!("shape_text: Using ASCII fast path.");
            self.shape_text_fast(text, options)
        } else {
            debug!("shape_text: Using Harfbuzz.");
            self.shaper.get_or_init(|| Shaper::new(self)).shape_text(
                text,
                options,
                &lookup_key.font_features,
            )
        };

        let shaped_text = Arc::new(glyphs);
        let mut cache = self.cached_shape_data.write();
        cache.shaped_text.insert(lookup_key, shaped_text.clone());

        shaped_text
    }

    /// Whether not a particular text and [`ShapingOptions`] combination can use
    /// "fast shaping" ie shaping without Harfbuzz.
    ///
    /// Note: This will eventually be removed.
    pub fn can_do_fast_shaping(&self, text: &str, options: &ShapingOptions) -> bool {
        options.script == Script::Latin &&
            !options.flags.contains(ShapingFlags::RTL_FLAG) &&
            *self.can_do_fast_shaping.get_or_init(|| {
                self.table_for_tag(KERN).is_some() &&
                    self.table_for_tag(GPOS).is_none() &&
                    self.table_for_tag(GSUB).is_none()
            }) &&
            text.is_ascii()
    }

    /// Fast path for ASCII text that only needs simple horizontal LTR kerning.
    fn shape_text_fast(&self, text: &str, options: &ShapingOptions) -> ShapedText {
        let mut glyph_store = ShapedText::new(text.len(), false /* is_rtl */);
        let mut prev_glyph_id = None;
        for (string_byte_offset, byte) in text.bytes().enumerate() {
            let character = byte as char;
            let Some(glyph_id) = self.glyph_index(character) else {
                continue;
            };

            let mut advance = Au::from_f64_px(self.glyph_h_advance(glyph_id));
            let offset = prev_glyph_id.map(|prev| {
                let h_kerning = Au::from_f64_px(self.glyph_h_kerning(prev, glyph_id));
                advance += h_kerning;
                Point2D::new(h_kerning, Au::zero())
            });

            let mut glyph = ShapedGlyph {
                glyph_id,
                string_byte_offset,
                advance,
                offset,
            };
            glyph.adjust_for_character(character, options);

            glyph_store.add_glyph(character, &glyph);
            prev_glyph_id = Some(glyph_id);
        }
        glyph_store
    }

    pub(crate) fn table_for_tag(&self, tag: Tag) -> Option<FontTable> {
        let result = self.handle.table_for_tag(tag);
        let status = if result.is_some() {
            "Found"
        } else {
            "Didn't find"
        };

        debug!(
            "{} font table[{}] in {:?},",
            status,
            str::from_utf8(tag.as_ref()).unwrap(),
            self.identifier()
        );
        result
    }

    #[inline]
    pub fn glyph_index(&self, codepoint: char) -> Option<GlyphId> {
        {
            let cache = self.cached_shape_data.read();
            if let Some(glyph) = cache.glyph_indices.get(&codepoint) {
                return *glyph;
            }
        }
        let codepoint = match self.descriptor.variant {
            font_variant_caps::T::SmallCaps => codepoint.to_ascii_uppercase(),
            font_variant_caps::T::Normal => codepoint,
            // The other values are behind `layout.css.font-variant-caps.all-values.enabled`
            // which Servo does not enable, so they are never parsed.
            font_variant_caps::T::AllSmallCaps |
            font_variant_caps::T::PetiteCaps |
            font_variant_caps::T::AllPetiteCaps |
            font_variant_caps::T::Unicase |
            font_variant_caps::T::TitlingCaps => unreachable!(),
        };
        let glyph_index = self.handle.glyph_index(codepoint);

        let mut cache = self.cached_shape_data.write();
        cache.glyph_indices.insert(codepoint, glyph_index);
        glyph_index
    }

    pub fn has_glyph_for(&self, codepoint: char) -> bool {
        self.glyph_index(codepoint).is_some()
    }

    pub(crate) fn glyph_h_kerning(
        &self,
        first_glyph: GlyphId,
        second_glyph: GlyphId,
    ) -> FractionalPixel {
        self.handle.glyph_h_kerning(first_glyph, second_glyph)
    }

    pub fn glyph_h_advance(&self, glyph_id: GlyphId) -> FractionalPixel {
        {
            let cache = self.cached_shape_data.read();
            if let Some(width) = cache.glyph_advances.get(&glyph_id) {
                return *width;
            }
        }

        let new_width = self
            .handle
            .glyph_h_advance(glyph_id)
            .unwrap_or(LAST_RESORT_GLYPH_ADVANCE as FractionalPixel);
        let mut cache = self.cached_shape_data.write();
        cache.glyph_advances.insert(glyph_id, new_width);
        new_width
    }

    pub fn typographic_bounds(&self, glyph_id: GlyphId) -> Rect<f32> {
        self.handle.typographic_bounds(glyph_id)
    }

    /// Get the [`FontBaseline`] for this font.
    pub fn baseline(&self) -> Option<FontBaseline> {
        self.shaper.get_or_init(|| Shaper::new(self)).baseline()
    }

    #[cfg(not(target_os = "macos"))]
    pub(crate) fn find_fallback_using_system_font_api(
        &self,
        _: &FallbackFontSelectionOptions,
        _: &SkrifaFontCache,
    ) -> Option<FontRef> {
        None
    }

    fn family_name_from_font_data(&self) -> Result<Atom, NoUsableFamilyName> {
        self.skrifa_font
            .names()
            .get(NameId::FAMILY_NAME)
            .and_then(|family_name| family_name.english_or_first())
            .map(|name| name.name().to_string().into())
            .ok_or(NoUsableFamilyName)
    }

    /// Return the font's declared family name:
    ///  - Platform: fonts: A value from OpenType `name` table, or `None` if either the platform
    ///    does not support that query or the font does not have a usable name.
    ///  - Web fonts: the family name specified in the `@font-face` rule
    pub fn family_name(&self) -> Option<Atom> {
        self.template
            .font_face_rule()
            .and_then(|font_face_rule| {
                AtomicRef::filter_map(font_face_rule, |rule| rule.descriptors.font_family.as_ref())
            })
            .map(|font_family| font_family.name.clone())
            .or_else(|| {
                self.family_name
                    .get_or_init(|| self.family_name_from_font_data())
                    .clone()
                    .ok()
            })
    }
}

#[derive(Clone, Debug, MallocSizeOf)]
pub struct FontRef(#[conditional_malloc_size_of] pub(crate) Arc<Font>);

impl PartialEq for FontRef {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(self, other)
    }
}

impl Deref for FontRef {
    type Target = Arc<Font>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Hash, MallocSizeOf, PartialEq)]
pub struct FallbackKey {
    script: Script,
    unicode_block: Option<UnicodeBlock>,
    language: Language,
}

impl FallbackKey {
    fn new(options: &FallbackFontSelectionOptions) -> Self {
        Self {
            script: Script::from(options.character),
            unicode_block: options.character.block(),
            language: options.language,
        }
    }
}

/// A `FontGroup` is a prioritised list of fonts for a given set of font styles. It is used by
/// `TextRun` to decide which font to render a character with. If none of the fonts listed in the
/// styles are suitable, a fallback font may be used.
#[derive(MallocSizeOf)]
pub struct FontGroup {
    /// The [`FontDescriptor`] which describes the properties of the fonts that should
    /// be loaded for this [`FontGroup`].
    descriptor: FontDescriptor,
    /// The families that have been loaded for this [`FontGroup`]. This correponds to the
    /// list of fonts specified in CSS.
    families: SmallVec<[FontGroupFamily; 8]>,
    /// A list of fallbacks that have been used in this [`FontGroup`]. Currently this
    /// can grow indefinitely, but maybe in the future it should be an LRU cache.
    /// It's unclear if this is the right thing to do. Perhaps fallbacks should
    /// always be stored here as it's quite likely that they will be used again.
    fallbacks: RwLock<HashMap<FallbackKey, FontRef>>,
}

impl FontGroup {
    pub(crate) fn new(style: &FontStyleStruct, descriptor: FontDescriptor) -> FontGroup {
        let families: SmallVec<[FontGroupFamily; 8]> = style
            .font_family
            .families
            .iter()
            .map(FontGroupFamily::local_or_web)
            .collect();

        FontGroup {
            descriptor,
            families,
            fallbacks: Default::default(),
        }
    }

    /// Finds the first font, or else the first fallback font, which contains a glyph for
    /// `codepoint`. If no such font is found, returns the first available font or fallback font
    /// (which will cause a "glyph not found" character to be rendered). If no font at all can be
    /// found, returns None.
    pub fn find_by_codepoint(
        &self,
        font_context: &FontContext,
        codepoint: char,
        next_codepoint: Option<char>,
        language: Language,
    ) -> Option<FontRef> {
        // Tab characters are converted into spaces when rendering.
        // TODO: We should not render a tab character. Instead they should be converted into tab stops
        // based upon the width of a space character in inline formatting contexts.
        let codepoint = match codepoint {
            '\t' => ' ',
            _ => codepoint,
        };

        let options = FallbackFontSelectionOptions::new(codepoint, next_codepoint, language);

        let should_look_for_small_caps = self.descriptor.variant == font_variant_caps::T::SmallCaps &&
            options.character.is_ascii_lowercase();
        let font_or_synthesized_small_caps = |font: FontRef| {
            if should_look_for_small_caps && font.synthesized_small_caps.is_some() {
                return font.synthesized_small_caps.clone();
            }
            Some(font)
        };

        let font_has_glyph_and_presentation = |font: &FontRef| {
            // Do not select this font if it goes against our emoji preference.
            match options.presentation_preference {
                EmojiPresentationPreference::Text if font.has_color_bitmap_or_colr_table() => {
                    return false;
                },
                EmojiPresentationPreference::Emoji if !font.has_color_bitmap_or_colr_table() => {
                    return false;
                },
                _ => {},
            }
            font.has_glyph_for(options.character)
        };

        let char_in_template =
            |template: FontTemplateRef| template.char_in_unicode_range(options.character);

        if let Some(font) = self.find(
            font_context,
            &char_in_template,
            &font_has_glyph_and_presentation,
        ) {
            return font_or_synthesized_small_caps(font);
        }

        let fallback_key = FallbackKey::new(&options);
        if let Some(fallback) = self.fallbacks.read().get(&fallback_key) &&
            char_in_template(fallback.template.clone()) &&
            font_has_glyph_and_presentation(fallback)
        {
            return font_or_synthesized_small_caps(fallback.clone());
        }

        if let Some(font) = self.find_fallback_using_system_font_list(
            font_context,
            options.clone(),
            &char_in_template,
            &font_has_glyph_and_presentation,
        ) {
            let fallback = font_or_synthesized_small_caps(font);
            if let Some(fallback) = fallback.clone() {
                self.fallbacks.write().insert(fallback_key, fallback);
            }
            return fallback;
        }

        let first_font = self.first(font_context);
        if let Some(fallback) = first_font.as_ref().and_then(|font| {
            font.find_fallback_using_system_font_api(&options, font_context.skrifa_font_cache())
        }) && font_has_glyph_and_presentation(&fallback)
        {
            return Some(fallback);
        }

        first_font
    }

    /// Find the first available font in the group, or the first available fallback font.
    pub fn first(&self, font_context: &FontContext) -> Option<FontRef> {
        // From https://drafts.csswg.org/css-fonts/#first-available-font:
        // > The first available font, used for example in the definition of font-relative lengths
        // > such as ex or in the definition of the line-height property, is defined to be the first
        // > font for which the character U+0020 (space) is not excluded by a unicode-range, given the
        // > font families in the font-family list (or a user agent’s default font if none are
        // > available).
        // > Note: it does not matter whether that font actually has a glyph for the space character.
        let space_in_template = |template: FontTemplateRef| template.char_in_unicode_range(' ');
        let font_predicate = |_: &FontRef| true;
        self.find(font_context, &space_in_template, &font_predicate)
            .or_else(|| {
                self.find_fallback_using_system_font_list(
                    font_context,
                    FallbackFontSelectionOptions::default(),
                    &space_in_template,
                    &font_predicate,
                )
            })
    }

    /// Attempts to find a font which matches the given `template_predicate` and `font_predicate`.
    /// This method mutates because we may need to load new font data in the process of finding
    /// a suitable font.
    fn find(
        &self,
        font_context: &FontContext,
        template_predicate: &impl Fn(FontTemplateRef) -> bool,
        font_predicate: &impl Fn(&FontRef) -> bool,
    ) -> Option<FontRef> {
        self.families
            .iter()
            .flat_map(|family| family.templates(font_context, &self.descriptor))
            .find_map(|template| {
                template.font_if_matches(
                    font_context,
                    &self.descriptor,
                    template_predicate,
                    font_predicate,
                )
            })
    }

    /// Attempts to find a suitable fallback font which matches the given `template_predicate` and
    /// `font_predicate` using the system font list. The default family (i.e. "serif") will be tried
    /// first, followed by platform-specific family names. If a `codepoint` is provided, then its
    /// Unicode block may be used to refine
    /// the list of family names which will be tried.
    fn find_fallback_using_system_font_list(
        &self,
        font_context: &FontContext,
        options: FallbackFontSelectionOptions,
        template_predicate: &impl Fn(FontTemplateRef) -> bool,
        font_predicate: &impl Fn(&FontRef) -> bool,
    ) -> Option<FontRef> {
        iter::once(FontFamilyDescriptor::default())
            .chain(
                fallback_font_families(options)
                    .into_iter()
                    .map(|family_name| {
                        let family = SingleFontFamily::FamilyName(FamilyName {
                            name: family_name.into(),
                            syntax: FontFamilyNameSyntax::Quoted,
                        });
                        FontFamilyDescriptor::new(family, FontSearchScope::Local)
                    }),
            )
            .find_map(|family_descriptor| {
                FontGroupFamily::from(family_descriptor)
                    .templates(font_context, &self.descriptor)
                    .find_map(|template| {
                        template.font_if_matches(
                            font_context,
                            &self.descriptor,
                            template_predicate,
                            font_predicate,
                        )
                    })
            })
    }
}

/// A [`FontGroupFamily`] can have multiple associated `FontTemplate`s if it is a
/// "composite face", meaning that it is defined by multiple `@font-face`
/// declarations which vary only by their `unicode-range` descriptors. In this case,
/// font selection will select a single member that contains the necessary unicode
/// character. Unicode ranges are specified by the [`FontGroupFamilyTemplate::template`]
/// member.
#[derive(MallocSizeOf)]
struct FontGroupFamilyTemplate {
    #[ignore_malloc_size_of = "This measured in the FontContext template cache."]
    template: FontTemplateRef,
    #[ignore_malloc_size_of = "This measured in the FontContext font cache."]
    font: OnceLock<Option<FontRef>>,
}

impl From<FontTemplateRef> for FontGroupFamilyTemplate {
    fn from(template: FontTemplateRef) -> Self {
        Self {
            template,
            font: Default::default(),
        }
    }
}

impl FontGroupFamilyTemplate {
    fn font(
        &self,
        font_context: &FontContext,
        font_descriptor: &FontDescriptor,
    ) -> Option<FontRef> {
        self.font
            .get_or_init(|| font_context.font(self.template.clone(), font_descriptor))
            .clone()
    }

    fn font_if_matches(
        &self,
        font_context: &FontContext,
        font_descriptor: &FontDescriptor,
        template_predicate: &impl Fn(FontTemplateRef) -> bool,
        font_predicate: &impl Fn(&FontRef) -> bool,
    ) -> Option<FontRef> {
        if !template_predicate(self.template.clone()) {
            return None;
        }
        self.font(font_context, font_descriptor)
            .filter(font_predicate)
    }
}

/// A `FontGroupFamily` is a single font family in a `FontGroup`. It corresponds to one of the
/// families listed in the `font-family` CSS property. The corresponding font data is lazy-loaded,
/// only if actually needed. A single `FontGroupFamily` can have multiple fonts, in the case that
/// individual fonts only cover part of the Unicode range.
#[derive(MallocSizeOf)]
struct FontGroupFamily {
    family_descriptor: FontFamilyDescriptor,
    members: OnceLock<Vec<FontGroupFamilyTemplate>>,
}

impl From<FontFamilyDescriptor> for FontGroupFamily {
    fn from(family_descriptor: FontFamilyDescriptor) -> Self {
        Self {
            family_descriptor,
            members: Default::default(),
        }
    }
}

impl FontGroupFamily {
    fn local_or_web(family: &SingleFontFamily) -> FontGroupFamily {
        FontFamilyDescriptor::new(family.clone(), FontSearchScope::Any).into()
    }

    fn templates(
        &self,
        font_context: &FontContext,
        font_descriptor: &FontDescriptor,
    ) -> impl Iterator<Item = &FontGroupFamilyTemplate> {
        self.members
            .get_or_init(|| {
                font_context
                    .matching_templates(font_descriptor, &self.family_descriptor)
                    .into_iter()
                    .map(Into::into)
                    .collect()
            })
            .iter()
    }
}

/// The scope within which we will look for a font.
#[derive(Clone, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
pub enum FontSearchScope {
    /// All fonts will be searched, including those specified via `@font-face` rules.
    Any,

    /// Only local system fonts will be searched.
    Local,
}

/// The font family parameters for font selection.
#[derive(Clone, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
pub struct FontFamilyDescriptor {
    pub(crate) family: SingleFontFamily,
    pub(crate) scope: FontSearchScope,
}

impl FontFamilyDescriptor {
    pub fn new(family: SingleFontFamily, scope: FontSearchScope) -> FontFamilyDescriptor {
        FontFamilyDescriptor { family, scope }
    }

    fn default() -> FontFamilyDescriptor {
        FontFamilyDescriptor {
            family: SingleFontFamily::Generic(GenericFontFamily::None),
            scope: FontSearchScope::Local,
        }
    }
}

pub struct FontBaseline {
    pub ideographic_baseline: f32,
    pub alphabetic_baseline: f32,
    pub hanging_baseline: f32,
}

/// Given a mapping array `mapping` and a value, map that value onto
/// the value specified by the array. For instance, for FontConfig
/// values of weights, we would map these onto the CSS [0..1000] range
/// by creating an array as below. Values that fall between two mapped
/// values, will be adjusted by the weighted mean.
///
/// ```ignore
/// let mapping = [
///     (0., 0.),
///     (FC_WEIGHT_REGULAR as f64, 400 as f64),
///     (FC_WEIGHT_BOLD as f64, 700 as f64),
///     (FC_WEIGHT_EXTRABLACK as f64, 1000 as f64),
/// ];
/// let mapped_weight = apply_font_config_to_style_mapping(&mapping, weight as f64);
/// ```
#[cfg(all(
    any(target_os = "linux", target_os = "macos", target_os = "freebsd"),
    not(target_env = "ohos")
))]
pub(crate) fn map_platform_values_to_style_values(mapping: &[(f64, f64)], value: f64) -> f64 {
    if value < mapping[0].0 {
        return mapping[0].1;
    }

    for window in mapping.windows(2) {
        let (font_config_value_a, css_value_a) = window[0];
        let (font_config_value_b, css_value_b) = window[1];

        if value >= font_config_value_a && value <= font_config_value_b {
            let ratio = (value - font_config_value_a) / (font_config_value_b - font_config_value_a);
            return css_value_a + ((css_value_b - css_value_a) * ratio);
        }
    }

    mapping[mapping.len() - 1].1
}

/// <https://drafts.csswg.org/css-fonts-4/#apply-font-matching-variations>
fn compute_variations(
    descriptor: &FontDescriptor,
    template: &FontTemplateRef,
    variation_axes: VariationAxes,
) -> Vec<FontVariation> {
    let font_face_rule = template.font_face_rule();

    // The steps in this algorithm are inverted order because they are listed in ascending order of precedence.
    let mut variations: Vec<FontVariation> = vec![];

    let mut add_variation = |variation: FontVariation| {
        if !variations
            .iter()
            .any(|existing_variation| existing_variation.tag == variation.tag)
        {
            variations.push(variation);
        }
    };

    // Step 12. Font variations implied by the value of the font-variation-settings property are applied.
    // These values should be clamped to the values that are supported by the font.
    // NOTE: Clamping happens inside the PlatformFont.
    descriptor
        .variation_settings
        .iter()
        .copied()
        .for_each(&mut add_variation);

    // Step 9. Font variations implied by the value of the font-optical-sizing property are applied.
    // NOTE The precise behaviour of font-optical-sizing:auto is not defined.
    // We choose to set "opsz" to the font size if it's not already set elsewhere. This is the easiest
    // at the end of this function, so we move this step down.

    if let Some(font_face_rule) = &font_face_rule {
        // Step 6. If the font is defined via an @font-face rule, the font variations implied by the font-variation-settings
        // descriptor in the @font-face rule are applied.
        if let Some(variation_settings) =
            font_face_rule.descriptors.font_variation_settings.as_ref()
        {
            variation_settings
                .0
                .iter()
                .map(|variation| FontVariation {
                    tag: variation.tag.0,
                    value: variation.value.get().expect(
                        "The value is enforced to be resolvable at parse time \
                        (see FontVariationSettings::parse_for_font_face_rule).",
                    ),
                })
                .for_each(&mut add_variation);
        }
    }

    // Step 2. Font variations as enabled by the font-weight, font-width, and font-style properties are applied.
    //
    // The application of the value enabled by font-style is affected by font selection, because this property might
    // select an italic or an oblique font. The value applied is the closest matching value as determined by the font
    // matching algorithm. User agents must apply at most one value due to the font-style property; both "ital" and
    // "slnt" values must not be set together.
    //
    // If the selected font is defined in an @font-face rule, then the values applied at this step should be clamped
    // to the value of the font-weight, font-width, and font-style descriptors in that @font-face rule.
    // TODO: Clamp weight/width to the descriptors from the @font-face rule, if any
    add_variation(FontVariation {
        tag: WGHT.to_u32(),
        value: descriptor.weight.value(),
    });

    add_variation(FontVariation {
        tag: WDTH.to_u32(),
        value: descriptor.width.0.to_float(),
    });

    if variation_axes.intersects(VariationAxes::ITAL | VariationAxes::SLNT) {
        let clamped_font_style = font_face_rule
            .as_ref()
            .and_then(|font_face_rule| font_face_rule.descriptors.font_style.as_ref())
            .map(|font_style_range| {
                let computed_font_style_range =
                    font_style_range.compute().expect("never returns None");

                if computed_font_style_range.0 == FontStyle::ITALIC {
                    debug_assert_eq!(computed_font_style_range.1, FontStyle::ITALIC);
                    return FontStyle::ITALIC;
                }
                debug_assert_ne!(computed_font_style_range.1, FontStyle::ITALIC);

                let specified_angle = if descriptor.style == FontStyle::ITALIC {
                    FontStyle::DEFAULT_OBLIQUE_DEGREES as f32
                } else {
                    descriptor.style.oblique_degrees()
                };

                let clamped_angle = specified_angle
                    .min(computed_font_style_range.1.oblique_degrees())
                    .max(computed_font_style_range.0.oblique_degrees());
                FontStyle::oblique(clamped_angle)
            })
            .unwrap_or(descriptor.style);

        // TODO: We should recognize when a font has neither a ital nor a slnt axis and then
        // synthesize an appropriate font face if allowed by font-synthesis.

        // When both a `ital` and a `slnt` axis are available then we prefer `ital` for
        // `font-style: italic` and `slnt` for `font-style: oblique`.
        let use_ital_axis = (variation_axes.contains(VariationAxes::ITAL) &&
            clamped_font_style == FontStyle::ITALIC) ||
            !variation_axes.contains(VariationAxes::SLNT);
        if use_ital_axis {
            add_variation(FontVariation {
                tag: ITAL.to_u32(),
                value: (clamped_font_style != FontStyle::NORMAL) as u32 as f32,
            });
        } else {
            // Note: CSS and OpenType measure slnt in opposite directions, so we need to negate the
            // angles.
            if clamped_font_style == FontStyle::ITALIC {
                add_variation(FontVariation {
                    tag: SLNT.to_u32(),
                    value: (-FontStyle::DEFAULT_OBLIQUE_DEGREES) as f32,
                });
            } else {
                add_variation(FontVariation {
                    tag: SLNT.to_u32(),
                    value: -clamped_font_style.oblique_degrees(),
                });
            }
        }
    }

    // This is the implementation for Step 9. Refer to the note on Step 9 for an explanation of why it's here.
    if descriptor.optical_sizing == FontOpticalSizing::Auto {
        add_variation(FontVariation {
            tag: Tag::new(b"opsz").to_u32(),
            value: descriptor.pt_size.to_f32_px(),
        });
    }

    variations
}

bitflags! {
    #[derive(Clone, Copy, Debug)]
    struct VariationAxes: u8 {
        /// Whether the font has any variation axes at all.
        const IS_VARIABLE = 1;

        /// Whether the font as a `ital` axis.
        ///
        /// Never set without `IS_VARIABLE`.
        const ITAL = 1 << 1;

        /// Whether the font as a `slnt` axis.
        ///
        /// Never set without `IS_VARIABLE`.
        const SLNT = 1 << 2;
    }
}

impl VariationAxes {
    fn from_skrifa_font(font: &SkrifaFont) -> Self {
        let variation_axes = font.axes();
        if variation_axes.is_empty() {
            return VariationAxes::empty();
        }
        let mut result = VariationAxes::IS_VARIABLE;
        for axis in variation_axes.iter() {
            match axis.tag() {
                ITAL => result |= VariationAxes::ITAL,
                SLNT => result |= VariationAxes::SLNT,
                _ => {},
            }
        }

        result
    }
}
