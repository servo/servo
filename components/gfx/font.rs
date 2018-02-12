/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use euclid::{Point2D, Rect, Size2D};
use font_context::{FontContext, FontSource};
use font_template::FontTemplateDescriptor;
use ordered_float::NotNaN;
use platform::font::{FontHandle, FontTable};
use platform::font_context::FontContextHandle;
use platform::font_template::FontTemplateData;
use smallvec::SmallVec;
use std::borrow::ToOwned;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::str;
use std::sync::Arc;
use std::sync::atomic::{ATOMIC_USIZE_INIT, AtomicUsize, Ordering};
use style::computed_values::{font_stretch, font_variant_caps, font_weight};
use style::properties::style_structs::Font as FontStyleStruct;
use style::values::computed::font::SingleFontFamily;
use text::Shaper;
use text::glyph::{ByteIndex, GlyphData, GlyphId, GlyphStore};
use text::shaping::ShaperMethods;
use time;
use unicode_script::Script;
use webrender_api;

macro_rules! ot_tag {
    ($t1:expr, $t2:expr, $t3:expr, $t4:expr) => (
        (($t1 as u32) << 24) | (($t2 as u32) << 16) | (($t3 as u32) << 8) | ($t4 as u32)
    );
}

pub const GPOS: u32 = ot_tag!('G', 'P', 'O', 'S');
pub const GSUB: u32 = ot_tag!('G', 'S', 'U', 'B');
pub const KERN: u32 = ot_tag!('k', 'e', 'r', 'n');

static TEXT_SHAPING_PERFORMANCE_COUNTER: AtomicUsize = ATOMIC_USIZE_INIT;

// FontHandle encapsulates access to the platform's font API,
// e.g. quartz, FreeType. It provides access to metrics and tables
// needed by the text shaper as well as access to the underlying font
// resources needed by the graphics layer to draw glyphs.

pub trait FontHandleMethods: Sized {
    fn new_from_template(fctx: &FontContextHandle, template: Arc<FontTemplateData>, pt_size: Option<Au>)
                    -> Result<Self, ()>;
    fn template(&self) -> Arc<FontTemplateData>;
    fn family_name(&self) -> String;
    fn face_name(&self) -> Option<String>;
    fn is_italic(&self) -> bool;
    fn boldness(&self) -> font_weight::T;
    fn stretchiness(&self) -> font_stretch::T;

    fn glyph_index(&self, codepoint: char) -> Option<GlyphId>;
    fn glyph_h_advance(&self, GlyphId) -> Option<FractionalPixel>;
    fn glyph_h_kerning(&self, glyph0: GlyphId, glyph1: GlyphId) -> FractionalPixel;
    /// Can this font do basic horizontal LTR shaping without Harfbuzz?
    fn can_do_fast_shaping(&self) -> bool;
    fn metrics(&self) -> FontMetrics;
    fn table_for_tag(&self, FontTableTag) -> Option<FontTable>;
}

// Used to abstract over the shaper's choice of fixed int representation.
pub type FractionalPixel = f64;

pub type FontTableTag = u32;

trait FontTableTagConversions {
    fn tag_to_str(&self) -> String;
}

impl FontTableTagConversions for FontTableTag {
    fn tag_to_str(&self) -> String {
        let bytes = [(self >> 24) as u8,
                     (self >> 16) as u8,
                     (self >>  8) as u8,
                     (self >>  0) as u8];
        str::from_utf8(&bytes).unwrap().to_owned()
    }
}

pub trait FontTableMethods {
    fn buffer(&self) -> &[u8];
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FontMetrics {
    pub underline_size:   Au,
    pub underline_offset: Au,
    pub strikeout_size:   Au,
    pub strikeout_offset: Au,
    pub leading:          Au,
    pub x_height:         Au,
    pub em_size:          Au,
    pub ascent:           Au,
    pub descent:          Au,
    pub max_advance:      Au,
    pub average_advance:  Au,
    pub line_gap:         Au,
}

/// `FontDescriptor` describes the parameters of a `Font`. It represents rendering a given font
/// template at a particular size, with a particular font-variant-caps applied, etc. This contrasts
/// with `FontTemplateDescriptor` in that the latter represents only the parameters inherent in the
/// font data (weight, stretch, etc.).
#[derive(Clone, Debug, PartialEq)]
pub struct FontDescriptor {
    pub template_descriptor: FontTemplateDescriptor,
    pub variant: font_variant_caps::T,
    pub pt_size: Au,
}

impl<'a> From<&'a FontStyleStruct> for FontDescriptor {
    fn from(style: &'a FontStyleStruct) -> Self {
        FontDescriptor {
            template_descriptor: FontTemplateDescriptor::from(style),
            variant: style.font_variant_caps,
            pt_size: style.font_size.size(),
        }
    }
}

#[derive(Debug)]
pub struct Font {
    pub handle: FontHandle,
    pub metrics: FontMetrics,
    pub descriptor: FontDescriptor,
    pub actual_pt_size: Au,
    shaper: Option<Shaper>,
    shape_cache: RefCell<HashMap<ShapeCacheEntry, Arc<GlyphStore>>>,
    glyph_advance_cache: RefCell<HashMap<u32, FractionalPixel>>,
    pub font_key: webrender_api::FontInstanceKey,
}

impl PartialEq for Font {
    fn eq(&self, other: &Font) -> bool {
        self.handle == other.handle && self.descriptor == other.descriptor
    }
}

impl Font {
    pub fn new(handle: FontHandle,
               descriptor: FontDescriptor,
               actual_pt_size: Au,
               font_key: webrender_api::FontInstanceKey) -> Font {
        let metrics = handle.metrics();

        Font {
            handle: handle,
            shaper: None,
            descriptor,
            actual_pt_size,
            metrics,
            shape_cache: RefCell::new(HashMap::new()),
            glyph_advance_cache: RefCell::new(HashMap::new()),
            font_key,
        }
    }
}

bitflags! {
    pub struct ShapingFlags: u8 {
        #[doc = "Set if the text is entirely whitespace."]
        const IS_WHITESPACE_SHAPING_FLAG = 0x01;
        #[doc = "Set if we are to ignore ligatures."]
        const IGNORE_LIGATURES_SHAPING_FLAG = 0x02;
        #[doc = "Set if we are to disable kerning."]
        const DISABLE_KERNING_SHAPING_FLAG = 0x04;
        #[doc = "Text direction is right-to-left."]
        const RTL_FLAG = 0x08;
        #[doc = "Set if word-break is set to keep-all."]
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
    pub word_spacing: (Au, NotNaN<f32>),
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
        let result = self.shape_cache.borrow_mut().entry(lookup_key).or_insert_with(|| {
            let start_time = time::precise_time_ns();
            let mut glyphs = GlyphStore::new(text.len(),
                                             options.flags.contains(ShapingFlags::IS_WHITESPACE_SHAPING_FLAG),
                                             options.flags.contains(ShapingFlags::RTL_FLAG));

            if self.can_do_fast_shaping(text, options) {
                debug!("shape_text: Using ASCII fast path.");
                self.shape_text_fast(text, options, &mut glyphs);
            } else {
                debug!("shape_text: Using Harfbuzz.");
                if shaper.is_none() {
                    shaper = Some(Shaper::new(this));
                }
                shaper.as_ref().unwrap().shape_text(text, options, &mut glyphs);
            }

            let end_time = time::precise_time_ns();
            TEXT_SHAPING_PERFORMANCE_COUNTER.fetch_add((end_time - start_time) as usize,
                                                       Ordering::Relaxed);
            Arc::new(glyphs)
        }).clone();
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
                let (length, percent) = options.word_spacing;
                advance = (advance + length) + Au((advance.0 as f32 * percent.into_inner()) as i32);
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
        let status = if result.is_some() { "Found" } else { "Didn't find" };

        debug!("{} font table[{}] with family={}, face={}",
               status, tag.tag_to_str(),
               self.handle.family_name(), self.handle.face_name().unwrap_or("unavailable".to_owned()));

        result
    }

    #[inline]
    pub fn glyph_index(&self, codepoint: char) -> Option<GlyphId> {
        let codepoint = match self.descriptor.variant {
            font_variant_caps::T::SmallCaps => codepoint.to_uppercase().next().unwrap(), //FIXME: #5938
            font_variant_caps::T::Normal => codepoint,
        };
        self.handle.glyph_index(codepoint)
    }

    pub fn has_glyph_for(&self, codepoint: char) -> bool {
        self.glyph_index(codepoint).is_some()
    }

    pub fn glyph_h_kerning(&self, first_glyph: GlyphId, second_glyph: GlyphId)
                           -> FractionalPixel {
        self.handle.glyph_h_kerning(first_glyph, second_glyph)
    }

    pub fn glyph_h_advance(&self, glyph: GlyphId) -> FractionalPixel {
        *self.glyph_advance_cache.borrow_mut().entry(glyph).or_insert_with(|| {
            match self.handle.glyph_h_advance(glyph) {
                Some(adv) => adv,
                None => 10f64 as FractionalPixel // FIXME: Need fallback strategy
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
}

impl FontGroup {
    pub fn new(style: &FontStyleStruct) -> FontGroup {
        let descriptor = FontDescriptor::from(style);

        let families =
            style.font_family.0.iter()
                .map(|family| FontGroupFamily::new(descriptor.clone(), family.clone()))
                .collect();

        FontGroup { descriptor, families }
    }

    /// Finds the first font, or else the first fallback font, which contains a glyph for
    /// `codepoint`. If no such font is found, returns the first available font or fallback font
    /// (which will cause a "glyph not found" character to be rendered). If no font at all can be
    /// found, returns None.
    pub fn find_by_codepoint<S: FontSource>(
        &mut self,
        mut font_context: &mut FontContext<S>,
        codepoint: char
    ) -> Option<FontRef> {
        self.find(&mut font_context, |font| font.borrow().has_glyph_for(codepoint))
            .or_else(|| self.first(&mut font_context))
    }

    pub fn first<S: FontSource>(
        &mut self,
        mut font_context: &mut FontContext<S>
    ) -> Option<FontRef> {
        self.find(&mut font_context, |_| true)
    }

    /// Find a font which returns true for `predicate`. This method mutates because we may need to
    /// load new font data in the process of finding a suitable font.
    fn find<S, P>(
        &mut self,
        mut font_context: &mut FontContext<S>,
        mut predicate: P
    ) -> Option<FontRef>
    where
        S: FontSource,
        P: FnMut(&FontRef) -> bool
    {
        self.families.iter_mut()
            .filter_map(|family| family.font(&mut font_context))
            .find(|f| predicate(f))
            .or_else(|| {
                font_context.fallback_font(&self.descriptor)
                    .into_iter().find(predicate)
            })
    }
}

/// A `FontGroupFamily` is a single font family in a `FontGroup`. It corresponds to one of the
/// families listed in the `font-family` CSS property. The corresponding font data is lazy-loaded,
/// only if actually needed.
#[derive(Debug)]
struct FontGroupFamily {
    descriptor: FontDescriptor,
    family: SingleFontFamily,
    loaded: bool,
    font: Option<FontRef>,
}

impl FontGroupFamily {
    fn new(descriptor: FontDescriptor, family: SingleFontFamily) -> FontGroupFamily {
        FontGroupFamily {
            descriptor,
            family,
            loaded: false,
            font: None,
        }
    }

    /// Returns the font within this family which matches the style. We'll fetch the data from the
    /// `FontContext` the first time this method is called, and return a cached reference on
    /// subsequent calls.
    fn font<S: FontSource>(&mut self, font_context: &mut FontContext<S>) -> Option<FontRef> {
        if !self.loaded {
            self.font = font_context.font(&self.descriptor, &self.family);
            self.loaded = true;
        }

        self.font.clone()
    }
}

pub struct RunMetrics {
    // may be negative due to negative width (i.e., kerning of '.' in 'P.T.')
    pub advance_width: Au,
    pub ascent: Au, // nonzero
    pub descent: Au, // nonzero
    // this bounding box is relative to the left origin baseline.
    // so, bounding_box.position.y = -ascent
    pub bounding_box: Rect<Au>
}

impl RunMetrics {
    pub fn new(advance: Au, ascent: Au, descent: Au) -> RunMetrics {
        let bounds = Rect::new(Point2D::new(Au(0), -ascent),
                               Size2D::new(advance, ascent + descent));

        // TODO(Issue #125): support loose and tight bounding boxes; using the
        // ascent+descent and advance is sometimes too generous and
        // looking at actual glyph extents can yield a tighter box.

        RunMetrics {
            advance_width: advance,
            bounding_box: bounds,
            ascent: ascent,
            descent: descent,
        }
    }
}

pub fn get_and_reset_text_shaping_performance_counter() -> usize {
    let value = TEXT_SHAPING_PERFORMANCE_COUNTER.load(Ordering::SeqCst);
    TEXT_SHAPING_PERFORMANCE_COUNTER.store(0, Ordering::SeqCst);
    value
}
