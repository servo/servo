/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Deref;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Instant;
use std::{iter, str};

use app_units::Au;
use base::id::PainterId;
use base::text::{UnicodeBlock, UnicodeBlockMethod};
use bitflags::bitflags;
use euclid::default::{Point2D, Rect};
use euclid::num::Zero;
use fonts_traits::FontDescriptor;
use log::debug;
use malloc_size_of_derive::MallocSizeOf;
use parking_lot::RwLock;
use read_fonts::tables::os2::{Os2, SelectionFlags};
use read_fonts::types::Tag;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use style::computed_values::font_variant_caps;
use style::properties::style_structs::Font as FontStyleStruct;
use style::values::computed::font::{
    FamilyName, FontFamilyNameSyntax, GenericFontFamily, SingleFontFamily,
};
use style::values::computed::{FontStretch, FontStyle, FontSynthesis, FontWeight, XLang};
use unicode_script::Script;
use webrender_api::{FontInstanceFlags, FontInstanceKey, FontVariation};

use crate::platform::font::{FontTable, PlatformFont};
use crate::platform::font_list::fallback_font_families;
use crate::{
    EmojiPresentationPreference, FallbackFontSelectionOptions, FontContext, FontData,
    FontDataAndIndex, FontDataError, FontIdentifier, FontTemplateDescriptor, FontTemplateRef,
    FontTemplateRefMethods, GlyphId, GlyphStore, LocalFontIdentifier, ShapedGlyph, Shaper,
};

pub(crate) const GPOS: Tag = Tag::new(b"GPOS");
pub(crate) const GSUB: Tag = Tag::new(b"GSUB");
pub(crate) const KERN: Tag = Tag::new(b"kern");
pub(crate) const SBIX: Tag = Tag::new(b"sbix");
pub(crate) const CBDT: Tag = Tag::new(b"CBDT");
pub(crate) const COLR: Tag = Tag::new(b"COLR");
pub(crate) const BASE: Tag = Tag::new(b"BASE");
pub(crate) const LIGA: Tag = Tag::new(b"liga");

pub const LAST_RESORT_GLYPH_ADVANCE: FractionalPixel = 10.0;

/// Nanoseconds spent shaping text across all layout threads.
static TEXT_SHAPING_PERFORMANCE_COUNTER: AtomicUsize = AtomicUsize::new(0);

// PlatformFont encapsulates access to the platform's font API,
// e.g. quartz, FreeType. It provides access to metrics and tables
// needed by the text shaper as well as access to the underlying font
// resources needed by the graphics layer to draw glyphs.

pub trait PlatformFontMethods: Sized {
    #[servo_tracing::instrument(name = "PlatformFontMethods::new_from_template", skip_all)]
    fn new_from_template(
        template: FontTemplateRef,
        pt_size: Option<Au>,
        variations: &[FontVariation],
        data: &Option<FontData>,
        synthetic_bold: bool,
    ) -> Result<PlatformFont, &'static str> {
        let template = template.borrow();
        let font_identifier = template.identifier.clone();

        match font_identifier {
            FontIdentifier::Local(font_identifier) => Self::new_from_local_font_identifier(
                font_identifier,
                pt_size,
                variations,
                synthetic_bold,
            ),
            FontIdentifier::Web(_) => Self::new_from_data(
                font_identifier,
                data.as_ref()
                    .expect("Should never create a web font without data."),
                pt_size,
                variations,
                synthetic_bold,
            ),
        }
    }

    fn new_from_local_font_identifier(
        font_identifier: LocalFontIdentifier,
        pt_size: Option<Au>,
        variations: &[FontVariation],
        synthetic_bold: bool,
    ) -> Result<PlatformFont, &'static str>;

    fn new_from_data(
        font_identifier: FontIdentifier,
        data: &FontData,
        pt_size: Option<Au>,
        variations: &[FontVariation],
        synthetic_bold: bool,
    ) -> Result<PlatformFont, &'static str>;

    /// Get a [`FontTemplateDescriptor`] from a [`PlatformFont`]. This is used to get
    /// descriptors for web fonts.
    fn descriptor(&self) -> FontTemplateDescriptor;

    fn glyph_index(&self, codepoint: char) -> Option<GlyphId>;
    fn glyph_h_advance(&self, _: GlyphId) -> Option<FractionalPixel>;
    fn glyph_h_kerning(&self, glyph0: GlyphId, glyph1: GlyphId) -> FractionalPixel;

    fn metrics(&self) -> FontMetrics;
    fn table_for_tag(&self, _: Tag) -> Option<FontTable>;
    fn typographic_bounds(&self, _: GlyphId) -> Rect<f32>;

    /// Get the necessary [`FontInstanceFlags`]` for this font.
    fn webrender_font_instance_flags(&self) -> FontInstanceFlags;

    /// Return all the variation values that the font was instantiated with.
    fn variations(&self) -> &[FontVariation];

    fn descriptor_from_os2_table(os2: &Os2) -> FontTemplateDescriptor {
        let mut style = FontStyle::NORMAL;
        if os2.fs_selection().contains(SelectionFlags::ITALIC) {
            style = FontStyle::ITALIC;
        }

        let weight = FontWeight::from_float(os2.us_weight_class() as f32);
        let stretch = match os2.us_width_class() {
            1 => FontStretch::ULTRA_CONDENSED,
            2 => FontStretch::EXTRA_CONDENSED,
            3 => FontStretch::CONDENSED,
            4 => FontStretch::SEMI_CONDENSED,
            5 => FontStretch::NORMAL,
            6 => FontStretch::SEMI_EXPANDED,
            7 => FontStretch::EXPANDED,
            8 => FontStretch::EXTRA_EXPANDED,
            9 => FontStretch::ULTRA_EXPANDED,
            _ => FontStretch::NORMAL,
        };

        FontTemplateDescriptor::new(weight, stretch, style)
    }
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
}

#[derive(Debug, Default)]
struct CachedShapeData {
    glyph_advances: HashMap<GlyphId, FractionalPixel>,
    glyph_indices: HashMap<char, Option<GlyphId>>,
    shaped_text: HashMap<ShapeCacheEntry, Arc<GlyphStore>>,
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
    pub metrics: Arc<FontMetrics>,
    pub descriptor: FontDescriptor,

    /// The data for this font. And the index of the font within the data (in case it's a TTC)
    /// This might be uninitialized for system fonts.
    data_and_index: OnceLock<FontDataAndIndex>,

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
}

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

        self.metrics.size_of(ops) +
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
    ) -> Result<Font, &'static str> {
        let synthetic_bold = {
            let is_bold = descriptor.weight >= FontWeight::BOLD_THRESHOLD;
            let allows_synthetic_bold = matches!(descriptor.synthesis_weight, FontSynthesis::Auto);

            is_bold && allows_synthetic_bold
        };

        let handle = PlatformFont::new_from_template(
            template.clone(),
            Some(descriptor.pt_size),
            &descriptor.variation_settings,
            &data,
            synthetic_bold,
        )?;
        let metrics = Arc::new(handle.metrics());

        Ok(Font {
            handle,
            template,
            metrics,
            descriptor,
            data_and_index: data
                .map(|data| OnceLock::from(FontDataAndIndex { data, index: 0 }))
                .unwrap_or_default(),
            shaper: OnceLock::new(),
            cached_shape_data: Default::default(),
            font_instance_key: Default::default(),
            synthesized_small_caps,
            has_color_bitmap_or_colr_table: OnceLock::new(),
            can_do_fast_shaping: OnceLock::new(),
        })
    }

    /// A unique identifier for the font, allowing comparison.
    pub fn identifier(&self) -> FontIdentifier {
        self.template.identifier()
    }

    pub(crate) fn webrender_font_instance_flags(&self) -> FontInstanceFlags {
        self.handle.webrender_font_instance_flags()
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

        let FontIdentifier::Local(local_font_identifier) = self.identifier() else {
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
        /// Set if the text is entirely whitespace.
        const IS_WHITESPACE_SHAPING_FLAG = 1 << 0;
        /// Set if the text ends with whitespace.
        const ENDS_WITH_WHITESPACE_SHAPING_FLAG = 1 << 1;
        /// Set if we are to ignore ligatures.
        const IGNORE_LIGATURES_SHAPING_FLAG = 1 << 2;
        /// Set if we are to disable kerning.
        const DISABLE_KERNING_SHAPING_FLAG = 1 << 3;
        /// Text direction is right-to-left.
        const RTL_FLAG = 1 << 4;
        /// Set if word-break is set to keep-all.
        const KEEP_ALL_FLAG = 1 << 5;
    }
}

/// Various options that control text shaping.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ShapingOptions {
    /// Spacing to add between each letter. Corresponds to the CSS 2.1 `letter-spacing` property.
    /// NB: You will probably want to set the `IGNORE_LIGATURES_SHAPING_FLAG` if this is non-null.
    pub letter_spacing: Option<Au>,
    /// Spacing to add between each word. Corresponds to the CSS 2.1 `word-spacing` property.
    pub word_spacing: Au,
    /// The Unicode script property of the characters in this run.
    pub script: Script,
    /// Various flags.
    pub flags: ShapingFlags,
}

/// An entry in the shape cache.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct ShapeCacheEntry {
    text: String,
    options: ShapingOptions,
}

impl Font {
    pub fn shape_text(&self, text: &str, options: &ShapingOptions) -> Arc<GlyphStore> {
        let lookup_key = ShapeCacheEntry {
            text: text.to_owned(),
            options: *options,
        };
        {
            let cache = self.cached_shape_data.read();
            if let Some(shaped_text) = cache.shaped_text.get(&lookup_key) {
                return shaped_text.clone();
            }
        }

        let start_time = Instant::now();
        let glyphs = if self.can_do_fast_shaping(text, options) {
            debug!("shape_text: Using ASCII fast path.");
            self.shape_text_fast(text, options)
        } else {
            debug!("shape_text: Using Harfbuzz.");
            self.shaper
                .get_or_init(|| Shaper::new(self))
                .shape_text(self, text, options)
        };

        let shaped_text = Arc::new(glyphs);
        let mut cache = self.cached_shape_data.write();
        cache.shaped_text.insert(lookup_key, shaped_text.clone());

        TEXT_SHAPING_PERFORMANCE_COUNTER.fetch_add(
            ((Instant::now() - start_time).as_nanos()) as usize,
            Ordering::Relaxed,
        );

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
    fn shape_text_fast(&self, text: &str, options: &ShapingOptions) -> GlyphStore {
        let mut glyph_store = GlyphStore::new(text, text.len(), options);
        let mut prev_glyph_id = None;
        for (string_byte_offset, byte) in text.bytes().enumerate() {
            let character = byte as char;
            let Some(glyph_id) = self.glyph_index(character) else {
                continue;
            };

            let mut advance = advance_for_shaped_glyph(
                Au::from_f64_px(self.glyph_h_advance(glyph_id)),
                character,
                options,
            );
            let offset = prev_glyph_id.map(|prev| {
                let h_kerning = Au::from_f64_px(self.glyph_h_kerning(prev, glyph_id));
                advance += h_kerning;
                Point2D::new(h_kerning, Au::zero())
            });

            glyph_store.add_glyph(
                character,
                &ShapedGlyph {
                    glyph_id,
                    string_byte_offset,
                    advance,
                    offset,
                },
            );
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
        };
        let glyph_index = self.handle.glyph_index(codepoint);

        let mut cache = self.cached_shape_data.write();
        cache.glyph_indices.insert(codepoint, glyph_index);
        glyph_index
    }

    pub(crate) fn has_glyph_for(&self, codepoint: char) -> bool {
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
}

#[derive(Clone, Debug, MallocSizeOf)]
pub struct FontRef(#[conditional_malloc_size_of] pub(crate) Arc<Font>);

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
    lang: XLang,
}

impl FallbackKey {
    fn new(options: &FallbackFontSelectionOptions) -> Self {
        Self {
            script: Script::from(options.character),
            unicode_block: options.character.block(),
            lang: options.lang.clone(),
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
        lang: XLang,
    ) -> Option<FontRef> {
        // Tab characters are converted into spaces when rendering.
        // TODO: We should not render a tab character. Instead they should be converted into tab stops
        // based upon the width of a space character in inline formatting contexts.
        let codepoint = match codepoint {
            '\t' => ' ',
            _ => codepoint,
        };

        let options = FallbackFontSelectionOptions::new(codepoint, next_codepoint, lang);

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
        if let Some(fallback) = self.fallbacks.read().get(&fallback_key) {
            if char_in_template(fallback.template.clone()) &&
                font_has_glyph_and_presentation(fallback)
            {
                return font_or_synthesized_small_caps(fallback.clone());
            }
        }

        if let Some(font) = self.find_fallback(
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

        self.first(font_context)
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
                self.find_fallback(
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
    /// `font_predicate`. The default family (i.e. "serif") will be tried first, followed by
    /// platform-specific family names. If a `codepoint` is provided, then its Unicode block may be
    /// used to refine the list of family names which will be tried.
    fn find_fallback(
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
/// ```rust
/// let mapping = [
///     (0., 0.),
///     (FC_WEIGHT_REGULAR as f64, 400 as f64),
///     (FC_WEIGHT_BOLD as f64, 700 as f64),
///     (FC_WEIGHT_EXTRABLACK as f64, 1000 as f64),
/// ];
/// let mapped_weight = apply_font_config_to_style_mapping(&mapping, weight as f64);
/// ```
#[cfg(all(
    any(target_os = "linux", target_os = "macos"),
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

/// Computes the total advance for a glyph, taking `letter-spacing` and `word-spacing` into account.
pub(super) fn advance_for_shaped_glyph(
    mut advance: Au,
    character: char,
    options: &ShapingOptions,
) -> Au {
    if let Some(letter_spacing) = options.letter_spacing {
        advance += letter_spacing;
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
