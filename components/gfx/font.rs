/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use std::{iter, str};

use app_units::Au;
use bitflags::bitflags;
use euclid::default::{Point2D, Rect, Size2D};
use log::debug;
use serde::{Deserialize, Serialize};
use servo_atoms::{atom, Atom};
use smallvec::SmallVec;
use style::computed_values::font_variant_caps;
use style::properties::style_structs::Font as FontStyleStruct;
use style::values::computed::font::{GenericFontFamily, SingleFontFamily};
use style::values::computed::{FontStretch, FontStyle, FontWeight};
use unicode_script::Script;
use webrender_api::FontInstanceKey;

use crate::font_cache_thread::FontIdentifier;
use crate::font_context::{FontContext, FontSource};
use crate::font_template::{FontTemplateDescriptor, FontTemplateRef, FontTemplateRefMethods};
use crate::platform::font::{FontTable, PlatformFont};
pub use crate::platform::font_list::fallback_font_families;
use crate::text::glyph::{ByteIndex, GlyphData, GlyphId, GlyphStore};
use crate::text::shaping::ShaperMethods;
use crate::text::Shaper;

#[macro_export]
macro_rules! ot_tag {
    ($t1:expr, $t2:expr, $t3:expr, $t4:expr) => {
        (($t1 as u32) << 24) | (($t2 as u32) << 16) | (($t3 as u32) << 8) | ($t4 as u32)
    };
}

pub const GPOS: u32 = ot_tag!('G', 'P', 'O', 'S');
pub const GSUB: u32 = ot_tag!('G', 'S', 'U', 'B');
pub const KERN: u32 = ot_tag!('k', 'e', 'r', 'n');
pub const LAST_RESORT_GLYPH_ADVANCE: FractionalPixel = 10.0;

static TEXT_SHAPING_PERFORMANCE_COUNTER: AtomicUsize = AtomicUsize::new(0);

// PlatformFont encapsulates access to the platform's font API,
// e.g. quartz, FreeType. It provides access to metrics and tables
// needed by the text shaper as well as access to the underlying font
// resources needed by the graphics layer to draw glyphs.

pub trait PlatformFontMethods: Sized {
    fn new_from_template(
        template: FontTemplateRef,
        pt_size: Option<Au>,
    ) -> Result<PlatformFont, &'static str> {
        let data = template.data();
        let face_index = template.identifier().index();
        let font_identifier = template.borrow().identifier.clone();
        Self::new_from_data(font_identifier, data, face_index, pt_size)
    }

    fn new_from_data(
        font_identifier: FontIdentifier,
        data: Arc<Vec<u8>>,
        face_index: u32,
        pt_size: Option<Au>,
    ) -> Result<PlatformFont, &'static str>;

    /// Get a [`FontTemplateDescriptor`] from a [`PlatformFont`]. This is used to get
    /// descriptors for web fonts.
    fn descriptor(&self) -> FontTemplateDescriptor;

    fn glyph_index(&self, codepoint: char) -> Option<GlyphId>;
    fn glyph_h_advance(&self, _: GlyphId) -> Option<FractionalPixel>;
    fn glyph_h_kerning(&self, glyph0: GlyphId, glyph1: GlyphId) -> FractionalPixel;

    /// Can this font do basic horizontal LTR shaping without Harfbuzz?
    fn can_do_fast_shaping(&self) -> bool;
    fn metrics(&self) -> FontMetrics;
    fn table_for_tag(&self, _: FontTableTag) -> Option<FontTable>;
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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
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
}

impl FontMetrics {
    /// Create an empty [`FontMetrics`] mainly to be used in situations where
    /// no font can be found.
    pub fn empty() -> Self {
        Self {
            underline_size: Au(0),
            underline_offset: Au(0),
            strikeout_size: Au(0),
            strikeout_offset: Au(0),
            leading: Au(0),
            x_height: Au(0),
            em_size: Au(0),
            ascent: Au(0),
            descent: Au(0),
            max_advance: Au(0),
            average_advance: Au(0),
            line_gap: Au(0),
        }
    }
}

/// `FontDescriptor` describes the parameters of a `Font`. It represents rendering a given font
/// template at a particular size, with a particular font-variant-caps applied, etc. This contrasts
/// with `FontTemplateDescriptor` in that the latter represents only the parameters inherent in the
/// font data (weight, stretch, etc.).
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
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

#[derive(Debug)]
pub struct Font {
    pub handle: PlatformFont,
    pub template: FontTemplateRef,
    pub metrics: FontMetrics,
    pub descriptor: FontDescriptor,
    shaper: Option<Shaper>,
    shape_cache: RefCell<HashMap<ShapeCacheEntry, Arc<GlyphStore>>>,
    glyph_advance_cache: RefCell<HashMap<u32, FractionalPixel>>,
    pub font_key: FontInstanceKey,

    /// If this is a synthesized small caps font, then this font reference is for
    /// the version of the font used to replace lowercase ASCII letters. It's up
    /// to the consumer of this font to properly use this reference.
    pub synthesized_small_caps: Option<FontRef>,
}

impl Font {
    pub fn new(
        template: FontTemplateRef,
        descriptor: FontDescriptor,
        font_key: FontInstanceKey,
        synthesized_small_caps: Option<FontRef>,
    ) -> Result<Font, &'static str> {
        let handle = PlatformFont::new_from_template(template.clone(), Some(descriptor.pt_size))?;
        let metrics = handle.metrics();

        Ok(Font {
            handle,
            template,
            shaper: None,
            descriptor,
            metrics,
            shape_cache: RefCell::new(HashMap::new()),
            glyph_advance_cache: RefCell::new(HashMap::new()),
            font_key,
            synthesized_small_caps,
        })
    }

    /// A unique identifier for the font, allowing comparison.
    pub fn identifier(&self) -> FontIdentifier {
        self.template.identifier()
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    pub struct ShapingFlags: u8 {
        /// Set if the text is entirely whitespace.
        const IS_WHITESPACE_SHAPING_FLAG = 0x01;
        /// Set if we are to ignore ligatures.
        const IGNORE_LIGATURES_SHAPING_FLAG = 0x02;
        /// Set if we are to disable kerning.
        const DISABLE_KERNING_SHAPING_FLAG = 0x04;
        /// Text direction is right-to-left.
        const RTL_FLAG = 0x08;
        /// Set if word-break is set to keep-all.
        const KEEP_ALL_FLAG = 0x10;
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
    pub fn shape_text(&mut self, text: &str, options: &ShapingOptions) -> Arc<GlyphStore> {
        let this = self as *const Font;
        let mut shaper = self.shaper.take();

        let lookup_key = ShapeCacheEntry {
            text: text.to_owned(),
            options: *options,
        };
        let result = self
            .shape_cache
            .borrow_mut()
            .entry(lookup_key)
            .or_insert_with(|| {
                let start_time = Instant::now();
                let mut glyphs = GlyphStore::new(
                    text.len(),
                    options
                        .flags
                        .contains(ShapingFlags::IS_WHITESPACE_SHAPING_FLAG),
                    options.flags.contains(ShapingFlags::RTL_FLAG),
                );

                if self.can_do_fast_shaping(text, options) {
                    debug!("shape_text: Using ASCII fast path.");
                    self.shape_text_fast(text, options, &mut glyphs);
                } else {
                    debug!("shape_text: Using Harfbuzz.");
                    if shaper.is_none() {
                        shaper = Some(Shaper::new(this));
                    }
                    shaper
                        .as_ref()
                        .unwrap()
                        .shape_text(text, options, &mut glyphs);
                }

                let end_time = Instant::now();
                TEXT_SHAPING_PERFORMANCE_COUNTER.fetch_add(
                    (end_time.duration_since(start_time).as_nanos()) as usize,
                    Ordering::Relaxed,
                );
                Arc::new(glyphs)
            })
            .clone();
        self.shaper = shaper;
        result
    }

    fn can_do_fast_shaping(&self, text: &str, options: &ShapingOptions) -> bool {
        options.script == Script::Latin &&
            !options.flags.contains(ShapingFlags::RTL_FLAG) &&
            self.handle.can_do_fast_shaping() &&
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
                Point2D::new(h_kerning, Au(0))
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
        let codepoint = match self.descriptor.variant {
            font_variant_caps::T::SmallCaps => codepoint.to_ascii_uppercase(),
            font_variant_caps::T::Normal => codepoint,
        };
        self.handle.glyph_index(codepoint)
    }

    pub fn has_glyph_for(&self, codepoint: char) -> bool {
        self.glyph_index(codepoint).is_some()
    }

    pub fn glyph_h_kerning(&self, first_glyph: GlyphId, second_glyph: GlyphId) -> FractionalPixel {
        self.handle.glyph_h_kerning(first_glyph, second_glyph)
    }

    pub fn glyph_h_advance(&self, glyph: GlyphId) -> FractionalPixel {
        *self
            .glyph_advance_cache
            .borrow_mut()
            .entry(glyph)
            .or_insert_with(|| {
                match self.handle.glyph_h_advance(glyph) {
                    Some(adv) => adv,
                    None => LAST_RESORT_GLYPH_ADVANCE as FractionalPixel, // FIXME: Need fallback strategy
                }
            })
    }
}

pub type FontRef = Rc<RefCell<Font>>;

/// A `FontGroup` is a prioritised list of fonts for a given set of font styles. It is used by
/// `TextRun` to decide which font to render a character with. If none of the fonts listed in the
/// styles are suitable, a fallback font may be used.
#[derive(Debug)]
pub struct FontGroup {
    descriptor: FontDescriptor,
    families: SmallVec<[FontGroupFamily; 8]>,
    last_matching_fallback: Option<FontRef>,
}

impl FontGroup {
    pub fn new(style: &FontStyleStruct) -> FontGroup {
        let descriptor = FontDescriptor::from(style);

        let families: SmallVec<[FontGroupFamily; 8]> = style
            .font_family
            .families
            .iter()
            .map(|family| FontGroupFamily::new(family))
            .collect();

        FontGroup {
            descriptor,
            families,
            last_matching_fallback: None,
        }
    }

    /// Finds the first font, or else the first fallback font, which contains a glyph for
    /// `codepoint`. If no such font is found, returns the first available font or fallback font
    /// (which will cause a "glyph not found" character to be rendered). If no font at all can be
    /// found, returns None.
    pub fn find_by_codepoint<S: FontSource>(
        &mut self,
        font_context: &mut FontContext<S>,
        codepoint: char,
    ) -> Option<FontRef> {
        let should_look_for_small_caps = self.descriptor.variant == font_variant_caps::T::SmallCaps &&
            codepoint.is_ascii_lowercase();
        let font_or_synthesized_small_caps = |font: FontRef| {
            if should_look_for_small_caps {
                let font = font.borrow();
                if font.synthesized_small_caps.is_some() {
                    return font.synthesized_small_caps.clone();
                }
            }
            Some(font)
        };

        let glyph_in_font = |font: &FontRef| font.borrow().has_glyph_for(codepoint);
        let char_in_template =
            |template: FontTemplateRef| template.char_in_unicode_range(codepoint);

        if let Some(font) = self.find(font_context, char_in_template, glyph_in_font) {
            return font_or_synthesized_small_caps(font);
        }

        if let Some(ref last_matching_fallback) = self.last_matching_fallback {
            if char_in_template(last_matching_fallback.borrow().template.clone()) &&
                glyph_in_font(last_matching_fallback)
            {
                return font_or_synthesized_small_caps(last_matching_fallback.clone());
            }
        }

        if let Some(font) = self.find_fallback(
            font_context,
            Some(codepoint),
            char_in_template,
            glyph_in_font,
        ) {
            self.last_matching_fallback = Some(font.clone());
            return font_or_synthesized_small_caps(font);
        }

        self.first(font_context)
    }

    /// Find the first available font in the group, or the first available fallback font.
    pub fn first<S: FontSource>(&mut self, font_context: &mut FontContext<S>) -> Option<FontRef> {
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
            .or_else(|| self.find_fallback(font_context, None, space_in_template, font_predicate))
    }

    /// Attempts to find a font which matches the given `template_predicate` and `font_predicate`.
    /// This method mutates because we may need to load new font data in the process of finding
    /// a suitable font.
    fn find<S, TemplatePredicate, FontPredicate>(
        &mut self,
        font_context: &mut FontContext<S>,
        template_predicate: TemplatePredicate,
        font_predicate: FontPredicate,
    ) -> Option<FontRef>
    where
        S: FontSource,
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
    fn find_fallback<S, TemplatePredicate, FontPredicate>(
        &mut self,
        font_context: &mut FontContext<S>,
        codepoint: Option<char>,
        template_predicate: TemplatePredicate,
        font_predicate: FontPredicate,
    ) -> Option<FontRef>
    where
        S: FontSource,
        TemplatePredicate: Fn(FontTemplateRef) -> bool,
        FontPredicate: Fn(&FontRef) -> bool,
    {
        iter::once(FontFamilyDescriptor::serif())
            .chain(fallback_font_families(codepoint).into_iter().map(|family| {
                FontFamilyDescriptor::new(FontFamilyName::from(family), FontSearchScope::Local)
            }))
            .into_iter()
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
#[derive(Debug)]
struct FontGroupFamilyMember {
    template: FontTemplateRef,
    font: Option<FontRef>,
    loaded: bool,
}

/// A `FontGroupFamily` is a single font family in a `FontGroup`. It corresponds to one of the
/// families listed in the `font-family` CSS property. The corresponding font data is lazy-loaded,
/// only if actually needed. A single `FontGroupFamily` can have multiple fonts, in the case that
/// individual fonts only cover part of the Unicode range.
#[derive(Debug)]
struct FontGroupFamily {
    family_descriptor: FontFamilyDescriptor,
    members: Option<Vec<FontGroupFamilyMember>>,
}

impl FontGroupFamily {
    fn new(family: &SingleFontFamily) -> FontGroupFamily {
        let family_descriptor =
            FontFamilyDescriptor::new(FontFamilyName::from(family), FontSearchScope::Any);

        FontGroupFamily {
            family_descriptor,
            members: None,
        }
    }

    fn find<S, TemplatePredicate, FontPredicate>(
        &mut self,
        font_descriptor: &FontDescriptor,
        font_context: &mut FontContext<S>,
        template_predicate: &TemplatePredicate,
        font_predicate: &FontPredicate,
    ) -> Option<FontRef>
    where
        S: FontSource,
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

    fn members<'a, S: FontSource>(
        &'a mut self,
        font_descriptor: &FontDescriptor,
        font_context: &mut FontContext<S>,
    ) -> impl Iterator<Item = &mut FontGroupFamilyMember> + 'a {
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
            Point2D::new(Au(0), -ascent),
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

pub fn get_and_reset_text_shaping_performance_counter() -> usize {
    TEXT_SHAPING_PERFORMANCE_COUNTER.swap(0, Ordering::SeqCst)
}

/// The scope within which we will look for a font.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum FontSearchScope {
    /// All fonts will be searched, including those specified via `@font-face` rules.
    Any,

    /// Only local system fonts will be searched.
    Local,
}

/// A font family name used in font selection.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum FontFamilyName {
    /// A specific name such as `"Arial"`
    Specific(Atom),

    /// A generic name such as `sans-serif`
    Generic(Atom),
}

impl FontFamilyName {
    pub fn name(&self) -> &str {
        match *self {
            FontFamilyName::Specific(ref name) => name,
            FontFamilyName::Generic(ref name) => name,
        }
    }
}

impl<'a> From<&'a SingleFontFamily> for FontFamilyName {
    fn from(other: &'a SingleFontFamily) -> FontFamilyName {
        match *other {
            SingleFontFamily::FamilyName(ref family_name) => {
                FontFamilyName::Specific(family_name.name.clone())
            },

            SingleFontFamily::Generic(generic) => FontFamilyName::Generic(match generic {
                GenericFontFamily::None => panic!("Shouldn't appear in style"),
                GenericFontFamily::Serif => atom!("serif"),
                GenericFontFamily::SansSerif => atom!("sans-serif"),
                GenericFontFamily::Monospace => atom!("monospace"),
                GenericFontFamily::Cursive => atom!("cursive"),
                GenericFontFamily::Fantasy => atom!("fantasy"),
                GenericFontFamily::SystemUi => atom!("system-ui"),
            }),
        }
    }
}

impl<'a> From<&'a str> for FontFamilyName {
    fn from(other: &'a str) -> FontFamilyName {
        FontFamilyName::Specific(Atom::from(other))
    }
}

/// The font family parameters for font selection.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct FontFamilyDescriptor {
    pub name: FontFamilyName,
    pub scope: FontSearchScope,
}

impl FontFamilyDescriptor {
    pub fn new(name: FontFamilyName, scope: FontSearchScope) -> FontFamilyDescriptor {
        FontFamilyDescriptor { name, scope }
    }

    fn serif() -> FontFamilyDescriptor {
        FontFamilyDescriptor {
            name: FontFamilyName::Generic(atom!("serif")),
            scope: FontSearchScope::Local,
        }
    }

    pub fn name(&self) -> &str {
        self.name.name()
    }
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
