/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use euclid::{Point2D, Rect, Size2D};
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

#[derive(Debug)]
pub struct Font {
    pub handle: FontHandle,
    pub metrics: FontMetrics,
    pub variant: font_variant_caps::T,
    pub descriptor: FontTemplateDescriptor,
    pub requested_pt_size: Au,
    pub actual_pt_size: Au,
    shaper: Option<Shaper>,
    shape_cache: RefCell<HashMap<ShapeCacheEntry, Arc<GlyphStore>>>,
    glyph_advance_cache: RefCell<HashMap<u32, FractionalPixel>>,
    pub font_key: webrender_api::FontInstanceKey,
}

impl Font {
    pub fn new(handle: FontHandle,
               variant: font_variant_caps::T,
               descriptor: FontTemplateDescriptor,
               requested_pt_size: Au,
               actual_pt_size: Au,
               font_key: webrender_api::FontInstanceKey) -> Font {
        let metrics = handle.metrics();
        Font {
            handle: handle,
            shaper: None,
            variant: variant,
            descriptor: descriptor,
            requested_pt_size: requested_pt_size,
            actual_pt_size: actual_pt_size,
            metrics: metrics,
            shape_cache: RefCell::new(HashMap::new()),
            glyph_advance_cache: RefCell::new(HashMap::new()),
            font_key: font_key,
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
        let codepoint = match self.variant {
            font_variant_caps::T::SmallCaps => codepoint.to_uppercase().next().unwrap(), //FIXME: #5938
            font_variant_caps::T::Normal => codepoint,
        };
        self.handle.glyph_index(codepoint)
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

#[derive(Debug)]
pub struct FontGroup {
    pub fonts: SmallVec<[Rc<RefCell<Font>>; 8]>,
}

impl FontGroup {
    pub fn new(fonts: SmallVec<[Rc<RefCell<Font>>; 8]>) -> FontGroup {
        FontGroup {
            fonts: fonts,
        }
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
