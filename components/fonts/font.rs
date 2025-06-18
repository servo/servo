/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Instant;
use std::{iter, str};

use app_units::Au;
use bitflags::bitflags;
use euclid::default::{Point2D, Rect, Size2D};
use euclid::num::Zero;
use log::debug;
use malloc_size_of_derive::MallocSizeOf;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use style::computed_values::font_variant_caps;
use style::properties::style_structs::Font as FontStyleStruct;
use style::values::computed::font::{
    FamilyName, FontFamilyNameSyntax, GenericFontFamily, SingleFontFamily,
};
use style::values::computed::{FontStretch, FontStyle, FontWeight};
use unicode_script::Script;
use webrender_api::{FontInstanceFlags, FontInstanceKey};

use crate::platform::font::{FontTable, PlatformFont};
pub use crate::platform::font_list::fallback_font_families;
use crate::{
    ByteIndex, EmojiPresentationPreference, FallbackFontSelectionOptions, FontContext, FontData,
    FontIdentifier, FontTemplateDescriptor, FontTemplateRef, FontTemplateRefMethods, GlyphData,
    GlyphId, GlyphStore, LocalFontIdentifier, Shaper,
};

#[macro_export]
macro_rules! ot_tag {
    ($t1:expr, $t2:expr, $t3:expr, $t4:expr) => {
        (($t1 as u32) << 24) | (($t2 as u32) << 16) | (($t3 as u32) << 8) | ($t4 as u32)
    };
}

pub const GPOS: u32 = ot_tag!('G', 'P', 'O', 'S');
pub const GSUB: u32 = ot_tag!('G', 'S', 'U', 'B');
pub const KERN: u32 = ot_tag!('k', 'e', 'r', 'n');
pub const SBIX: u32 = ot_tag!('s', 'b', 'i', 'x');
pub const CBDT: u32 = ot_tag!('C', 'B', 'D', 'T');
pub const COLR: u32 = ot_tag!('C', 'O', 'L', 'R');
pub const BASE: u32 = ot_tag!('B', 'A', 'S', 'E');

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
        data: &Option<FontData>,
    ) -> Result<PlatformFont, &'static str> {
        let template = template.borrow();
        let font_identifier = template.identifier.clone();

        match font_identifier {
            FontIdentifier::Local(font_identifier) => {
                Self::new_from_local_font_identifier(font_identifier, pt_size)
            },
            FontIdentifier::Web(_) => Self::new_from_data(
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

    /// Get a [`FontTemplateDescriptor`] from a [`PlatformFont`]. This is used to get
    /// descriptors for web fonts.
    fn descriptor(&self) -> FontTemplateDescriptor;

    fn glyph_index(&self, codepoint: char) -> Option<GlyphId>;
    fn glyph_h_advance(&self, _: GlyphId) -> Option<FractionalPixel>;
    fn glyph_h_kerning(&self, glyph0: GlyphId, glyph1: GlyphId) -> FractionalPixel;

    fn metrics(&self) -> FontMetrics;
    fn table_for_tag(&self, _: FontTableTag) -> Option<FontTable>;
    fn typographic_bounds(&self, _: GlyphId) -> Rect<f32>;

    /// Get the necessary [`FontInstanceFlags`]` for this font.
    fn webrender_font_instance_flags(&self) -> FontInstanceFlags;
}

// Used to abstract over the shaper's choice of fixed int representation.
pub type FractionalPixel = f64;

pub type FontTableTag = u32;

trait FontTableTagConversions {
    fn tag_to_str(&self) -> String;
}

impl FontTableTagConversions for FontTableTag {
    fn tag_to_str(&self) -> String {
        let bytes = [
            (self >> 24) as u8,
            (self >> 16) as u8,
            (self >> 8) as u8,
            *self as u8,
        ];
        str::from_utf8(&bytes).unwrap().to_owned()
    }
}

pub trait FontTableMethods {
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
    pub fn empty() -> Self {
        Self {
            underline_size: Au::zero(),
            underline_offset: Au::zero(),
            strikeout_size: Au::zero(),
            strikeout_offset: Au::zero(),
            leading: Au::zero(),
            x_height: Au::zero(),
            em_size: Au::zero(),
            ascent: Au::zero(),
            descent: Au::zero(),
            max_advance: Au::zero(),
            average_advance: Au::zero(),
            line_gap: Au::zero(),
            zero_horizontal_advance: None,
            ic_horizontal_advance: None,
            space_advance: Au::zero(),
        }
    }
}

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
}

impl Eq for FontDescriptor {}

impl<'a> From<&'a FontStyleStruct> for FontDescriptor {
    fn from(style: &'a FontStyleStruct) -> Self {
        FontDescriptor {
            weight: style.font_weight,
            stretch: style.font_stretch,
            style: style.font_style,
            variant: style.font_variant_caps,
            pt_size: Au::from_f32_px(style.font_size.computed_size().px()),
        }
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
    pub handle: PlatformFont,
    pub template: FontTemplateRef,
    pub metrics: FontMetrics,
    pub descriptor: FontDescriptor,

    /// The data for this font. This might be uninitialized for system fonts.
    data: OnceLock<FontData>,

    shaper: OnceLock<Shaper>,
    cached_shape_data: RwLock<CachedShapeData>,
    pub font_instance_key: OnceLock<FontInstanceKey>,

    /// If this is a synthesized small caps font, then this font reference is for
    /// the version of the font used to replace lowercase ASCII letters. It's up
    /// to the consumer of this font to properly use this reference.
    pub synthesized_small_caps: Option<FontRef>,

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

impl malloc_size_of::MallocSizeOf for Font {
    fn size_of(&self, ops: &mut malloc_size_of::MallocSizeOfOps) -> usize {
        // TODO: Collect memory usage for platform fonts and for shapers.
        // This skips the template, because they are already stored in the template cache.
        self.metrics.size_of(ops) +
            self.descriptor.size_of(ops) +
            self.cached_shape_data.read().size_of(ops) +
            self.font_instance_key
                .get()
                .map_or(0, |key| key.size_of(ops))
    }
}

impl Font {
    pub fn new(
        template: FontTemplateRef,
        descriptor: FontDescriptor,
        data: Option<FontData>,
        synthesized_small_caps: Option<FontRef>,
    ) -> Result<Font, &'static str> {
        let handle =
            PlatformFont::new_from_template(template.clone(), Some(descriptor.pt_size), &data)?;
        let metrics = handle.metrics();

        Ok(Font {
            handle,
            template,
            metrics,
            descriptor,
            data: data.map(OnceLock::from).unwrap_or_default(),
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

    pub fn webrender_font_instance_flags(&self) -> FontInstanceFlags {
        self.handle.webrender_font_instance_flags()
    }

    pub fn has_color_bitmap_or_colr_table(&self) -> bool {
        *self.has_color_bitmap_or_colr_table.get_or_init(|| {
            self.table_for_tag(SBIX).is_some() ||
                self.table_for_tag(CBDT).is_some() ||
                self.table_for_tag(COLR).is_some()
        })
    }

    pub fn key(&self, font_context: &FontContext) -> FontInstanceKey {
        *self
            .font_instance_key
            .get_or_init(|| font_context.create_font_instance_key(self))
    }

    /// Return the data for this `Font`. Note that this is currently highly inefficient for system
    /// fonts and should not be used except in legacy canvas code.
    pub fn data(&self) -> &FontData {
        self.data.get_or_init(|| {
            let FontIdentifier::Local(local_font_identifier) = self.identifier() else {
                unreachable!("All web fonts should already have initialized data");
            };
            FontData::from_bytes(
                &local_font_identifier
                    .read_data_from_file()
                    .unwrap_or_default(),
            )
        })
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

        let is_single_preserved_newline = text.len() == 1 && text.starts_with('\n');
        let start_time = Instant::now();
        let mut glyphs = GlyphStore::new(
            text.len(),
            options
                .flags
                .contains(ShapingFlags::IS_WHITESPACE_SHAPING_FLAG),
            options
                .flags
                .contains(ShapingFlags::ENDS_WITH_WHITESPACE_SHAPING_FLAG),
            is_single_preserved_newline,
            options.flags.contains(ShapingFlags::RTL_FLAG),
        );

        if self.can_do_fast_shaping(text, options) {
            debug!("shape_text: Using ASCII fast path.");
            self.shape_text_fast(text, options, &mut glyphs);
        } else {
            debug!("shape_text: Using Harfbuzz.");
            self.shape_text_harfbuzz(text, options, &mut glyphs);
        }

        let shaped_text = Arc::new(glyphs);
        let mut cache = self.cached_shape_data.write();
        cache.shaped_text.insert(lookup_key, shaped_text.clone());

        let end_time = Instant::now();
        TEXT_SHAPING_PERFORMANCE_COUNTER.fetch_add(
            (end_time.duration_since(start_time).as_nanos()) as usize,
            Ordering::Relaxed,
        );

        shaped_text
    }

    fn shape_text_harfbuzz(&self, text: &str, options: &ShapingOptions, glyphs: &mut GlyphStore) {
        let this = self as *const Font;
        self.shaper
            .get_or_init(|| Shaper::new(this))
            .shape_text(text, options, glyphs);
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
    fn shape_text_fast(&self, text: &str, options: &ShapingOptions, glyphs: &mut GlyphStore) {
        let mut prev_glyph_id = None;
        for (i, byte) in text.bytes().enumerate() {
            let character = byte as char;
            let glyph_id = match self.glyph_index(character) {
                Some(id) => id,
                None => continue,
            };

            let mut advance = Au::from_f64_px(self.glyph_h_advance(glyph_id));
            if character == ' ' {
                // https://drafts.csswg.org/css-text-3/#word-spacing-property
                advance += options.word_spacing;
            }
            if let Some(letter_spacing) = options.letter_spacing {
                advance += letter_spacing;
            }
            let offset = prev_glyph_id.map(|prev| {
                let h_kerning = Au::from_f64_px(self.glyph_h_kerning(prev, glyph_id));
                advance += h_kerning;
                Point2D::new(h_kerning, Au::zero())
            });

            let glyph = GlyphData::new(glyph_id, advance, offset, true, true);
            glyphs.add_glyph_for_byte_index(ByteIndex(i as isize), character, &glyph);
            prev_glyph_id = Some(glyph_id);
        }
        glyphs.finalize_changes();
    }

    pub fn table_for_tag(&self, tag: FontTableTag) -> Option<FontTable> {
        let result = self.handle.table_for_tag(tag);
        let status = if result.is_some() {
            "Found"
        } else {
            "Didn't find"
        };

        debug!(
            "{} font table[{}] in {:?},",
            status,
            tag.tag_to_str(),
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

    pub fn has_glyph_for(&self, codepoint: char) -> bool {
        self.glyph_index(codepoint).is_some()
    }

    pub fn glyph_h_kerning(&self, first_glyph: GlyphId, second_glyph: GlyphId) -> FractionalPixel {
        self.handle.glyph_h_kerning(first_glyph, second_glyph)
    }

    pub fn glyph_h_advance(&self, glyph_id: GlyphId) -> FractionalPixel {
        {
            let cache = self.cached_shape_data.read();
            if let Some(width) = cache.glyph_advances.get(&glyph_id) {
                return *width;
            }
        }

        // TODO: Need a fallback strategy.
        let new_width = match self.handle.glyph_h_advance(glyph_id) {
            Some(adv) => adv,
            None => LAST_RESORT_GLYPH_ADVANCE as FractionalPixel,
        };

        let mut cache = self.cached_shape_data.write();
        cache.glyph_advances.insert(glyph_id, new_width);
        new_width
    }

    pub fn typographic_bounds(&self, glyph_id: GlyphId) -> Rect<f32> {
        self.handle.typographic_bounds(glyph_id)
    }

    /// Get the [`FontBaseline`] for this font.
    pub fn baseline(&self) -> Option<FontBaseline> {
        let this = self as *const Font;
        self.shaper.get_or_init(|| Shaper::new(this)).baseline()
    }
}

pub type FontRef = Arc<Font>;

/// A `FontGroup` is a prioritised list of fonts for a given set of font styles. It is used by
/// `TextRun` to decide which font to render a character with. If none of the fonts listed in the
/// styles are suitable, a fallback font may be used.
#[derive(MallocSizeOf)]
pub struct FontGroup {
    descriptor: FontDescriptor,
    families: SmallVec<[FontGroupFamily; 8]>,
}

impl FontGroup {
    pub fn new(style: &FontStyleStruct, descriptor: FontDescriptor) -> FontGroup {
        let families: SmallVec<[FontGroupFamily; 8]> = style
            .font_family
            .families
            .iter()
            .map(FontGroupFamily::new)
            .collect();

        FontGroup {
            descriptor,
            families,
        }
    }

    /// Finds the first font, or else the first fallback font, which contains a glyph for
    /// `codepoint`. If no such font is found, returns the first available font or fallback font
    /// (which will cause a "glyph not found" character to be rendered). If no font at all can be
    /// found, returns None.
    pub fn find_by_codepoint(
        &mut self,
        font_context: &FontContext,
        codepoint: char,
        next_codepoint: Option<char>,
        first_fallback: Option<FontRef>,
    ) -> Option<FontRef> {
        // Tab characters are converted into spaces when rendering.
        // TODO: We should not render a tab character. Instead they should be converted into tab stops
        // based upon the width of a space character in inline formatting contexts.
        let codepoint = match codepoint {
            '\t' => ' ',
            _ => codepoint,
        };

        let options = FallbackFontSelectionOptions::new(codepoint, next_codepoint);

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
            char_in_template,
            font_has_glyph_and_presentation,
        ) {
            return font_or_synthesized_small_caps(font);
        }

        if let Some(ref first_fallback) = first_fallback {
            if char_in_template(first_fallback.template.clone()) &&
                font_has_glyph_and_presentation(first_fallback)
            {
                return font_or_synthesized_small_caps(first_fallback.clone());
            }
        }

        if let Some(font) = self.find_fallback(
            font_context,
            options,
            char_in_template,
            font_has_glyph_and_presentation,
        ) {
            return font_or_synthesized_small_caps(font);
        }

        self.first(font_context)
    }

    /// Find the first available font in the group, or the first available fallback font.
    pub fn first(&mut self, font_context: &FontContext) -> Option<FontRef> {
        // From https://drafts.csswg.org/css-fonts/#first-available-font:
        // > The first available font, used for example in the definition of font-relative lengths
        // > such as ex or in the definition of the line-height property, is defined to be the first
        // > font for which the character U+0020 (space) is not excluded by a unicode-range, given the
        // > font families in the font-family list (or a user agent’s default font if none are
        // > available).
        // > Note: it does not matter whether that font actually has a glyph for the space character.
        let space_in_template = |template: FontTemplateRef| template.char_in_unicode_range(' ');
        let font_predicate = |_: &FontRef| true;
        self.find(font_context, space_in_template, font_predicate)
            .or_else(|| {
                self.find_fallback(
                    font_context,
                    FallbackFontSelectionOptions::default(),
                    space_in_template,
                    font_predicate,
                )
            })
    }

    /// Attempts to find a font which matches the given `template_predicate` and `font_predicate`.
    /// This method mutates because we may need to load new font data in the process of finding
    /// a suitable font.
    fn find<TemplatePredicate, FontPredicate>(
        &mut self,
        font_context: &FontContext,
        template_predicate: TemplatePredicate,
        font_predicate: FontPredicate,
    ) -> Option<FontRef>
    where
        TemplatePredicate: Fn(FontTemplateRef) -> bool,
        FontPredicate: Fn(&FontRef) -> bool,
    {
        let font_descriptor = self.descriptor.clone();
        self.families
            .iter_mut()
            .filter_map(|font_group_family| {
                font_group_family.find(
                    &font_descriptor,
                    font_context,
                    &template_predicate,
                    &font_predicate,
                )
            })
            .next()
    }

    /// Attempts to find a suitable fallback font which matches the given `template_predicate` and
    /// `font_predicate`. The default family (i.e. "serif") will be tried first, followed by
    /// platform-specific family names. If a `codepoint` is provided, then its Unicode block may be
    /// used to refine the list of family names which will be tried.
    fn find_fallback<TemplatePredicate, FontPredicate>(
        &mut self,
        font_context: &FontContext,
        options: FallbackFontSelectionOptions,
        template_predicate: TemplatePredicate,
        font_predicate: FontPredicate,
    ) -> Option<FontRef>
    where
        TemplatePredicate: Fn(FontTemplateRef) -> bool,
        FontPredicate: Fn(&FontRef) -> bool,
    {
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
            .filter_map(|family_descriptor| {
                FontGroupFamily {
                    family_descriptor,
                    members: None,
                }
                .find(
                    &self.descriptor,
                    font_context,
                    &template_predicate,
                    &font_predicate,
                )
            })
            .next()
    }
}

/// A [`FontGroupFamily`] can have multiple members if it is a "composite face", meaning
/// that it is defined by multiple `@font-face` declarations which vary only by their
/// `unicode-range` descriptors. In this case, font selection will select a single member
/// that contains the necessary unicode character. Unicode ranges are specified by the
/// [`FontGroupFamilyMember::template`] member.
#[derive(MallocSizeOf)]
struct FontGroupFamilyMember {
    #[ignore_malloc_size_of = "This measured in the FontContext template cache."]
    template: FontTemplateRef,
    #[ignore_malloc_size_of = "This measured in the FontContext font cache."]
    font: Option<FontRef>,
    loaded: bool,
}

/// A `FontGroupFamily` is a single font family in a `FontGroup`. It corresponds to one of the
/// families listed in the `font-family` CSS property. The corresponding font data is lazy-loaded,
/// only if actually needed. A single `FontGroupFamily` can have multiple fonts, in the case that
/// individual fonts only cover part of the Unicode range.
#[derive(MallocSizeOf)]
struct FontGroupFamily {
    family_descriptor: FontFamilyDescriptor,
    members: Option<Vec<FontGroupFamilyMember>>,
}

impl FontGroupFamily {
    fn new(family: &SingleFontFamily) -> FontGroupFamily {
        FontGroupFamily {
            family_descriptor: FontFamilyDescriptor::new(family.clone(), FontSearchScope::Any),
            members: None,
        }
    }

    fn find<TemplatePredicate, FontPredicate>(
        &mut self,
        font_descriptor: &FontDescriptor,
        font_context: &FontContext,
        template_predicate: &TemplatePredicate,
        font_predicate: &FontPredicate,
    ) -> Option<FontRef>
    where
        TemplatePredicate: Fn(FontTemplateRef) -> bool,
        FontPredicate: Fn(&FontRef) -> bool,
    {
        self.members(font_descriptor, font_context)
            .filter_map(|member| {
                if !template_predicate(member.template.clone()) {
                    return None;
                }

                if !member.loaded {
                    member.font = font_context.font(member.template.clone(), font_descriptor);
                    member.loaded = true;
                }
                if matches!(&member.font, Some(font) if font_predicate(font)) {
                    return member.font.clone();
                }

                None
            })
            .next()
    }

    fn members(
        &mut self,
        font_descriptor: &FontDescriptor,
        font_context: &FontContext,
    ) -> impl Iterator<Item = &mut FontGroupFamilyMember> {
        let family_descriptor = &self.family_descriptor;
        let members = self.members.get_or_insert_with(|| {
            font_context
                .matching_templates(font_descriptor, family_descriptor)
                .into_iter()
                .map(|template| FontGroupFamilyMember {
                    template,
                    loaded: false,
                    font: None,
                })
                .collect()
        });

        members.iter_mut()
    }
}

pub struct RunMetrics {
    // may be negative due to negative width (i.e., kerning of '.' in 'P.T.')
    pub advance_width: Au,
    pub ascent: Au,  // nonzero
    pub descent: Au, // nonzero
    // this bounding box is relative to the left origin baseline.
    // so, bounding_box.position.y = -ascent
    pub bounding_box: Rect<Au>,
}

impl RunMetrics {
    pub fn new(advance: Au, ascent: Au, descent: Au) -> RunMetrics {
        let bounds = Rect::new(
            Point2D::new(Au::zero(), -ascent),
            Size2D::new(advance, ascent + descent),
        );

        // TODO(Issue #125): support loose and tight bounding boxes; using the
        // ascent+descent and advance is sometimes too generous and
        // looking at actual glyph extents can yield a tighter box.

        RunMetrics {
            advance_width: advance,
            bounding_box: bounds,
            ascent,
            descent,
        }
    }
}

/// Get the number of nanoseconds spent shaping text across all threads.
pub fn get_and_reset_text_shaping_performance_counter() -> usize {
    TEXT_SHAPING_PERFORMANCE_COUNTER.swap(0, Ordering::SeqCst)
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
    pub family: SingleFontFamily,
    pub scope: FontSearchScope,
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
#[cfg(any(target_os = "linux", target_os = "macos"))]
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
